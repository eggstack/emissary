//! M128 bounded JSON-RPC batch conformance guards.
//!
//! Proves valid top-level batch arrays execute instead of regressing to the
//! pre-M128 blanket invalid-request rejection, the batch cardinality bound
//! is documented and within the in-flight budget, batch entries reuse the
//! exact single-request parser, dispatch stays sequential without task
//! fan-out, per-element authentication keeps M127 expired/unknown semantics
//! with no intra-batch credential sharing, production changes stay under
//! `i2pcontrol`, and the Proposal matrix is unchanged. Full dispatch
//! behavior (ordering, notifications, mixed entries, over-cap zero-effect,
//! per-element auth, no propagation) is covered by unit tests in `rpc.rs`
//! and `server.rs` plus the live runtime batch phases; this file provides
//! the durable static/contract regression guard.

#![cfg(feature = "i2pcontrol")]

use std::{path::Path, process::Command};

use emissary_cli::i2pcontrol::rpc;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn source(path: &str) -> String {
    std::fs::read_to_string(workspace_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn production_section(path: &str) -> String {
    source(path)
        .split("#[cfg(test)]\nmod tests")
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// Slice `source` from the first `start_marker` to the first `end_marker`
/// after it. Panics when either marker is missing so guard drift fails loud.
fn section_between(source: &str, start_marker: &str, end_marker: &str) -> String {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing marker {start_marker}"));
    let rest = &source[start..];
    let end = rest
        .find(end_marker)
        .unwrap_or_else(|| panic!("missing marker {end_marker} after {start_marker}"));
    rest[..end].to_owned()
}

#[test]
fn batch_cardinality_bound_is_documented_and_within_budget() {
    const _: () = assert!(rpc::MAX_BATCH_ELEMENTS > 0 && rpc::MAX_BATCH_ELEMENTS <= 64);
    assert_eq!(rpc::MAX_BATCH_ELEMENTS, 32);
    let rpc_source = production_section("emissary-cli/src/i2pcontrol/rpc.rs");
    let has_bound = rpc_source.contains("MAX_BATCH_ELEMENTS");
    assert!(has_bound, "bound must be a named constant");
    assert!(
        rpc_source.contains("MAX_CONCURRENT_REQUESTS"),
        "bound rationale must cite the in-flight budget"
    );
}

#[test]
fn valid_batch_arrays_are_not_blanket_rejected() {
    // WP5 static guard: a valid batch must parse as a batch. If batch
    // support regresses to blanket invalid-request handling, this fails.
    let body = r#"[{"jsonrpc":"2.0","method":"Authenticate","params":{"API":1},"id":1},
        {"jsonrpc":"2.0","method":"RouterInfo","id":2}]"#;
    match rpc::parse_envelope(body).expect("valid batch must parse") {
        rpc::JsonRpcEnvelope::Batch(entries) => assert_eq!(entries.len(), 2),
        rpc::JsonRpcEnvelope::Single(_) => panic!("batch array must not parse as single"),
    }

    // The single-request entry point keeps its historical contract: arrays
    // stay invalid there so single/batch handling cannot be conflated.
    let err = rpc::parse_request(body).unwrap_err();
    assert_eq!(err.error.code, rpc::error_codes::INVALID_REQUEST);
}

#[test]
fn envelope_preserves_parse_error_and_empty_batch_distinctions() {
    let parse = rpc::parse_envelope("not json {{{").unwrap_err();
    assert_eq!(parse.error.code, rpc::error_codes::PARSE_ERROR);
    assert_eq!(parse.id, rpc::RequestId::Null);

    let empty = rpc::parse_envelope("[]").unwrap_err();
    assert_eq!(empty.error.code, rpc::error_codes::INVALID_REQUEST);
    assert_eq!(empty.id, rpc::RequestId::Null);

    let over_cap = (0..rpc::MAX_BATCH_ELEMENTS + 1)
        .map(|i| format!(r#"{{"jsonrpc":"2.0","method":"Authenticate","id":{i}}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let over_cap = rpc::parse_envelope(&format!("[{over_cap}]")).unwrap_err();
    assert_eq!(over_cap.error.code, rpc::error_codes::INVALID_REQUEST);
    assert_eq!(over_cap.id, rpc::RequestId::Null);

    let at_cap = (0..rpc::MAX_BATCH_ELEMENTS)
        .map(|i| format!(r#"{{"jsonrpc":"2.0","method":"Authenticate","id":{i}}}"#))
        .collect::<Vec<_>>()
        .join(",");
    match rpc::parse_envelope(&format!("[{at_cap}]")).expect("max-size batch must parse") {
        rpc::JsonRpcEnvelope::Batch(entries) => assert_eq!(entries.len(), rpc::MAX_BATCH_ELEMENTS),
        rpc::JsonRpcEnvelope::Single(_) => panic!("array must parse as batch"),
    }
}

#[test]
fn batch_entries_reuse_single_request_parser() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    let batch_fn = section_between(&server, "async fn handle_batch_request", "fn error_value");
    assert!(
        batch_fn.contains("parse_batch_entry"),
        "batch must validate entries via the shared parser"
    );
    assert!(
        batch_fn.contains("dispatch_one"),
        "batch elements must take the single dispatch path"
    );

    let rpc_source = production_section("emissary-cli/src/i2pcontrol/rpc.rs");
    let entry_fn = section_between(
        &rpc_source,
        "pub fn parse_batch_entry",
        "fn parse_single_object",
    );
    assert!(
        entry_fn.contains("parse_single_object"),
        "batch entries must reuse the exact single-request rules"
    );
    // Non-object entries (scalars, null, nested arrays) are per-entry
    // invalid requests with null ID, never sibling-invalidating.
    for entry in [
        serde_json::json!(42),
        serde_json::json!(null),
        serde_json::json!([{"jsonrpc": "2.0", "method": "Authenticate", "id": 1}]),
    ] {
        let err = rpc::parse_batch_entry(&entry).unwrap_err();
        assert_eq!(err.error.code, rpc::error_codes::INVALID_REQUEST);
        assert_eq!(err.id, rpc::RequestId::Null);
    }
}

#[test]
fn batch_dispatch_is_sequential_without_task_fanout() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    let batch_fn = section_between(&server, "async fn handle_batch_request", "fn error_value");
    let dispatch_fn = section_between(&server, "async fn dispatch_one", "struct DispatchResult");
    for region in [&batch_fn, &dispatch_fn] {
        for forbidden in [
            "tokio::spawn",
            "spawn(",
            "spawn_blocking",
            "JoinSet",
            "join_all",
            "buffer_unordered",
            "buffered(",
        ] {
            assert!(
                !region.contains(forbidden),
                "batch dispatch must stay sequential: found {forbidden}"
            );
        }
    }
    // Bounded response collection: capacity is the bounded entry count.
    assert!(
        batch_fn.contains("with_capacity"),
        "batch responses must be pre-bounded"
    );
    // Input order is preserved; notifications suppress their element.
    assert!(
        batch_fn.contains("emit_response"),
        "batch must honor notification suppression"
    );
    assert!(
        batch_fn.contains("NO_CONTENT"),
        "all-notification batches must emit no body"
    );
}

#[test]
fn batch_auth_keeps_per_element_m127_semantics_without_sharing() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    let dispatch_fn = section_between(&server, "async fn dispatch_one", "struct DispatchResult");
    assert!(
        dispatch_fn.contains("handle_authenticate_with_source"),
        "Authenticate elements must take the single auth path"
    );
    assert!(
        dispatch_fn.contains("authenticate_protected_request"),
        "protected elements must take the single protected-auth path"
    );
    // The per-element auth boundary itself distinguishes expired/unknown.
    assert!(
        server.contains("TokenValidation::Expired"),
        "expiry mapping must be reachable"
    );
    assert!(
        server.contains("TOKEN_EXPIRED"),
        "expired elements must map to -32004"
    );

    // The batch plumber must not issue, inject, or forward tokens between
    // entries: no token-service issuance and no Token param mutation in the
    // batch function itself (sanitization stays inside the shared
    // per-element auth path).
    let batch_fn = section_between(&server, "async fn handle_batch_request", "fn error_value");
    assert!(
        !batch_fn.contains("token_service"),
        "batch must not touch the token store directly"
    );
    assert!(
        !batch_fn.contains(".issue()"),
        "batch must not issue tokens outside dispatch"
    );
    assert!(
        !batch_fn.contains("\"Token\""),
        "batch must not read or forward Token params"
    );
}

#[test]
fn production_changes_stay_under_i2pcontrol() {
    // M128 authorizes no core/util/router/frontend/dependency change.
    // Keep this historical guard scoped to the M128 implementation range;
    // later lifecycle milestones own their separately accepted seams.
    let baseline = "c16934bc2dafca3bf27b912dac4998e788d10ae2";
    let reviewed_head = "0ed60eb";
    let output = Command::new("git")
        .args([
            "diff",
            "--name-only",
            baseline,
            reviewed_head,
            "--",
            "emissary-core/src",
            "emissary-util/src",
            "emissary-cli/src/main.rs",
            "emissary-cli/src/config.rs",
            "emissary-cli/Cargo.toml",
            "emissary-core/Cargo.toml",
            "emissary-util/Cargo.toml",
            "Cargo.toml",
            "Cargo.lock",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("git diff for containment");
    assert!(output.status.success(), "git diff failed");
    let changed = String::from_utf8_lossy(&output.stdout);
    let changed: Vec<_> = changed.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        changed.is_empty(),
        "M128 must not change core/util/config/dependency paths: {changed:?}"
    );

    // Changed production files under emissary-cli/src must all be I2PControl-owned.
    let output = Command::new("git")
        .args([
            "diff",
            "--name-only",
            baseline,
            reviewed_head,
            "--",
            "emissary-cli/src",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("git diff for i2pcontrol boundary");
    assert!(output.status.success());
    let changed = String::from_utf8_lossy(&output.stdout);
    for path in changed.lines().filter(|l| !l.trim().is_empty()) {
        assert!(
            path.starts_with("emissary-cli/src/i2pcontrol/"),
            "M128 production change outside i2pcontrol: {path}"
        );
    }
}

#[test]
fn proposal_matrix_unchanged_by_batch_conformance() {
    let matrix: toml::Value = std::fs::read_to_string(
        workspace_root()
            .join("plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"),
    )
    .expect("matrix")
    .parse()
    .expect("valid matrix");
    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("tunnel types");
    assert_eq!(tunnel_types.len(), 12);
    let options = matrix["tunnel_manager"]["options"].as_array().expect("options");
    let mut counts = std::collections::BTreeMap::new();
    for option in options {
        for cell in option["cells"].as_array().expect("cells") {
            *counts.entry(cell.as_str().expect("cell").to_owned()).or_insert(0usize) += 1;
        }
    }
    assert_eq!(counts.get("apply"), Some(&325));
    assert_eq!(counts.get("blocked_primitive"), Some(&47));
    assert_eq!(counts.get("not_applicable"), Some(&468));
}
