//! M130 post-M127–M129 corrective requalification guards.
//!
//! Integrated current-head requalification after the three concrete
//! M126-missed shared-control-plane correctives:
//!
//! - M127 finite token lifetime with reachable `TOKEN_EXPIRED` (`-32004`);
//! - M128 bounded JSON-RPC batch conformance (`MAX_BATCH_ELEMENTS = 32`);
//! - M129 fail-closed non-loopback managed-TLS policy (loopback-only).
//!
//! This file does not duplicate every lower-level unit case from
//! M127–M129. It provides the durable composition guard required by M130
//! §8: it fails if token storage loses finite expiry, `TOKEN_EXPIRED`
//! becomes unreachable, valid batches regress to blanket rejection,
//! all-notification batches emit a body, over-cap batches execute side
//! effects, non-loopback managed TLS is accepted, plaintext reaches
//! dispatch, fake adapters appear in production composition, resource
//! limits are lost, or active matrix/docs disagree with current authority.

#![cfg(feature = "i2pcontrol")]

use std::{path::Path, process::Command};

use emissary_cli::i2pcontrol::{
    auth::{TokenService, TokenValidation},
    rpc,
    server::{I2pControlConfig, ServerInitContext},
    tls::TlsConfig,
};

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

fn config(bind: &str, cert: bool, key: bool) -> I2pControlConfig {
    I2pControlConfig {
        enabled: true,
        bind: bind.parse().unwrap(),
        password: "m130-test-password".to_string(),
        tls: TlsConfig {
            certificate: cert.then(|| "/operator/cert.pem".into()),
            private_key: key.then(|| "/operator/key.pem".into()),
        },
    }
}

// ──────────────────────────────────────────────────────────────
// WP1: post-corrective head inventory
// ──────────────────────────────────────────────────────────────

#[test]
fn m127_m128_m129_commits_are_present_in_reviewed_head() {
    // M130 hard dependencies: the three corrective implementations must be
    // ancestors of the reviewed head. Missing commits mean the
    // requalification ran against the wrong baseline.
    for commit in ["098c9d1", "0ed60eb", "39ccdd7"] {
        let status = Command::new("git")
            .args(["merge-base", "--is-ancestor", commit, "HEAD"])
            .current_dir(workspace_root())
            .status()
            .expect("git merge-base");
        assert!(
            status.success(),
            "M130 reviewed head must contain M127/M128/M129 commit {commit}"
        );
    }
}

// ──────────────────────────────────────────────────────────────
// WP2: token lifetime (M127 composition)
// ──────────────────────────────────────────────────────────────

#[test]
fn token_storage_has_finite_expiry_behavior() {
    // Error contract must stay reachable protocol surface, not dead text.
    assert_eq!(rpc::error_codes::TOKEN_EXPIRED, -32004);
    assert_eq!(rpc::error_codes::INVALID_TOKEN, -32003);

    // Live pre-expiry proof: freshly issued tokens authorize, unknown and
    // oversized credentials fail as unknown without echo or allocation.
    let svc = TokenService::new();
    let token = svc.issue();
    assert_eq!(token.len(), 64, "opaque hex token shape unchanged");
    assert_eq!(svc.validate(&token), TokenValidation::Valid);
    assert_eq!(svc.validate("never-issued"), TokenValidation::Unknown);
    assert_eq!(svc.validate(&"x".repeat(1024)), TokenValidation::Unknown);
    assert_eq!(svc.validate(""), TokenValidation::Unknown);

    // Static composition guard: membership-only storage must be gone and
    // finite monotonic expiry state must be present.
    let auth = production_section("emissary-cli/src/i2pcontrol/auth.rs");
    assert!(auth.contains("TOKEN_LIFETIME"), "missing lifetime constant");
    assert!(
        auth.contains("24 * 60 * 60"),
        "lifetime must be the documented one-day reference value"
    );
    assert!(
        !auth.contains("tokens: HashMap<String, ()>"),
        "membership-only token state regressed"
    );
    assert!(
        auth.contains("HashMap<String, Instant>"),
        "token record must carry monotonic expiry state"
    );
    assert!(
        auth.contains("enum TokenValidation"),
        "missing outcome enum"
    );
    assert!(auth.contains("Expired"), "missing expired outcome");
    assert!(auth.contains("Instant::now"), "must use monotonic clock");
    assert!(
        auth.contains("MAX_PRESENTED_TOKEN_LEN"),
        "missing presented-credential bound"
    );
    // No background scanner or timer task for expiry.
    for forbidden in ["tokio::spawn", "set_interval", "interval("] {
        assert!(
            !auth.contains(forbidden),
            "auth must not introduce a background scanner: found {forbidden}"
        );
    }
}

#[test]
fn token_expired_is_reachable_and_mapped_distinctly() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    assert!(
        server.contains("TokenValidation::Expired"),
        "dispatch must observe the expired outcome"
    );
    assert!(
        server.contains("TokenValidation::Unknown"),
        "dispatch must observe the unknown outcome"
    );
    assert!(
        server.contains("TOKEN_EXPIRED"),
        "expired must map to -32004"
    );
    let expired_pos = server.find("TokenValidation::Expired").expect("expired arm");
    let window = &server[expired_pos..server.len().min(expired_pos + 600)];
    assert!(
        window.contains("TOKEN_EXPIRED"),
        "expired arm must return TOKEN_EXPIRED, not INVALID_TOKEN"
    );
    let unknown_pos = server.find("TokenValidation::Unknown").expect("unknown arm");
    let window = &server[unknown_pos..server.len().min(unknown_pos + 600)];
    assert!(
        window.contains("INVALID_TOKEN"),
        "unknown arm must return INVALID_TOKEN"
    );
}

// ──────────────────────────────────────────────────────────────
// WP2: bounded batch conformance (M128 composition)
// ──────────────────────────────────────────────────────────────

#[test]
fn valid_batch_arrays_do_not_regress_to_blanket_rejection() {
    // Exact cardinality bound within the in-flight budget.
    const _: () = assert!(rpc::MAX_BATCH_ELEMENTS > 0 && rpc::MAX_BATCH_ELEMENTS <= 64);
    assert_eq!(rpc::MAX_BATCH_ELEMENTS, 32);

    // Valid batch arrays must parse as batches, not blanket invalid-request.
    let body = r#"[{"jsonrpc":"2.0","method":"Authenticate","params":{"API":1},"id":1},
        {"jsonrpc":"2.0","method":"RouterInfo","id":2}]"#;
    match rpc::parse_envelope(body).expect("valid batch must parse") {
        rpc::JsonRpcEnvelope::Batch(entries) => assert_eq!(entries.len(), 2),
        rpc::JsonRpcEnvelope::Single(_) => panic!("batch array must not parse as single"),
    }

    // Single-request entry point keeps its historical contract.
    let err = rpc::parse_request(body).unwrap_err();
    assert_eq!(err.error.code, rpc::error_codes::INVALID_REQUEST);

    // At-cap parses; over-cap is a single invalid-request with null ID and
    // executes zero elements (rejected during envelope parsing before any
    // element dispatches).
    let at_cap = (0..rpc::MAX_BATCH_ELEMENTS)
        .map(|i| format!(r#"{{"jsonrpc":"2.0","method":"Authenticate","id":{i}}}"#))
        .collect::<Vec<_>>()
        .join(",");
    match rpc::parse_envelope(&format!("[{at_cap}]")).expect("max-size batch must parse") {
        rpc::JsonRpcEnvelope::Batch(entries) => assert_eq!(entries.len(), rpc::MAX_BATCH_ELEMENTS),
        rpc::JsonRpcEnvelope::Single(_) => panic!("array must parse as batch"),
    }
    let over_cap = (0..rpc::MAX_BATCH_ELEMENTS + 1)
        .map(|i| format!(r#"{{"jsonrpc":"2.0","method":"Authenticate","id":{i}}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let err = rpc::parse_envelope(&format!("[{over_cap}]")).unwrap_err();
    assert_eq!(err.error.code, rpc::error_codes::INVALID_REQUEST);
    assert_eq!(err.id, rpc::RequestId::Null);

    // Empty batch remains a single invalid-request error.
    let empty = rpc::parse_envelope("[]").unwrap_err();
    assert_eq!(empty.error.code, rpc::error_codes::INVALID_REQUEST);

    // Non-object entries are per-entry invalid requests, never
    // sibling-invalidating.
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
fn batch_dispatch_stays_sequential_without_sharing_or_fanout() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    let batch_fn = section_between(&server, "async fn handle_batch_request", "fn error_value");
    let dispatch_fn = section_between(&server, "async fn dispatch_one", "struct DispatchResult");

    // Batch entries reuse the exact single-request parser and dispatch path.
    assert!(
        batch_fn.contains("parse_batch_entry"),
        "batch must validate via the shared parser"
    );
    assert!(
        batch_fn.contains("dispatch_one"),
        "batch elements must take the single dispatch path"
    );
    // No unbounded task fan-out.
    for region in [&batch_fn, &dispatch_fn] {
        for forbidden in [
            "tokio::spawn",
            "spawn(",
            "spawn_blocking",
            "JoinSet",
            "join_all",
            "buffer_unordered",
        ] {
            assert!(
                !region.contains(forbidden),
                "batch dispatch must stay sequential: found {forbidden}"
            );
        }
    }
    // Bounded response collection, notification suppression, no-content body.
    assert!(
        batch_fn.contains("with_capacity"),
        "responses must be pre-bounded"
    );
    assert!(
        batch_fn.contains("emit_response"),
        "batch must honor notification suppression"
    );
    assert!(
        batch_fn.contains("NO_CONTENT"),
        "all-notification batches must emit no JSON-RPC body"
    );
    // Over-cap batches are rejected during envelope parsing before any
    // element executes; the batch handler asserts the bound.
    assert!(
        server.contains("over-cap batches must be rejected during envelope parsing")
            || server.contains("Batch too large"),
        "over-cap zero-effect rationale must be documented"
    );

    // Per-element authentication with M127 semantics and no intra-batch
    // credential sharing.
    assert!(
        dispatch_fn.contains("handle_authenticate_with_source"),
        "Authenticate elements must take the single auth path"
    );
    assert!(
        dispatch_fn.contains("authenticate_protected_request"),
        "protected elements must take the single protected-auth path"
    );
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
        "batch must not forward Token params between entries"
    );
}

// ──────────────────────────────────────────────────────────────
// WP2: TLS fail-closed composition (M129)
// ──────────────────────────────────────────────────────────────

#[test]
fn non_loopback_managed_tls_is_rejected_loopback_remains() {
    for bind in ["127.0.0.1:7650", "[::1]:7650"] {
        assert!(
            config(bind, false, false).validate().is_ok(),
            "loopback managed must stay allowed: {bind}"
        );
        assert!(
            config(bind, true, true).validate().is_ok(),
            "loopback explicit must stay allowed: {bind}"
        );
    }
    for bind in [
        "192.0.2.10:7650",
        "203.0.113.7:7650",
        "[2001:db8::1]:7650",
        "0.0.0.0:7650",
        "[::]:7650",
    ] {
        for (cert, key) in [(false, false), (true, false), (false, true)] {
            let err = config(bind, cert, key).validate().unwrap_err();
            let message = err.to_string();
            assert!(
                message.contains("non-loopback") && message.contains("explicit"),
                "must state explicit-material requirement ({bind}): {message}"
            );
            assert!(
                message.contains("loopback-only"),
                "must state managed identity is loopback-only: {message}"
            );
        }
        assert!(
            config(bind, true, true).validate().is_ok(),
            "complete explicit material must pass: {bind}"
        );
    }

    // Managed SAN set stays loopback-only without remote synthesis.
    let tls = production_section("emissary-cli/src/i2pcontrol/tls.rs");
    for san in ["\"localhost\"", "\"127.0.0.1\"", "\"::1\""] {
        assert!(tls.contains(san), "managed SAN missing {san}");
    }
    for forbidden in ["get_if_addrs", "lookup_host", "gethostname"] {
        assert!(
            !tls.contains(forbidden),
            "managed TLS must not synthesize remote identities: {forbidden}"
        );
    }
}

#[test]
fn tls_rejection_precedes_side_effects_and_never_falls_back() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    let init_start = server.find("pub async fn init_server").expect("init_server");
    let init = &server[init_start..];
    let validate = init.find("config.validate()").expect("validate call");
    let tls = init.find("build_tls_config").expect("TLS setup");
    let bind = init.find("TcpListener::bind").expect("listener bind");
    let stores = init.find("addressbooks").expect("store setup");
    assert!(
        validate < tls && validate < bind && validate < stores,
        "validation must precede TLS, store setup, and bind"
    );

    let validate_fn = server
        .find("pub fn validate(&self)")
        .map(|start| &server[start..start + 2500])
        .expect("validate body");
    assert!(
        validate_fn.contains("is_loopback"),
        "must branch on loopback"
    );
    assert!(
        validate_fn.contains("has_complete_explicit_material"),
        "must require complete explicit material"
    );
    assert!(
        validate_fn.contains("return Err"),
        "must reject, not merely warn"
    );

    // Explicit TLS failures never fall back to managed or plaintext.
    let tls_src = production_section("emissary-cli/src/i2pcontrol/tls.rs");
    assert!(
        tls_src.contains("with_safe_default_protocol_versions"),
        "safe rustls defaults must be retained"
    );
    assert!(
        server.contains("TlsAcceptor"),
        "serving must stay behind TLS"
    );
    assert!(
        server.contains("TLS handshake failed") || server.contains("TLS handshake timed out"),
        "handshake failures must be contained, not downgraded"
    );
}

#[tokio::test]
async fn rejected_remote_creates_no_side_effects_live() {
    let tmp = tempfile::tempdir().unwrap();
    let port = {
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        probe.local_addr().unwrap().port()
    };
    let failing = I2pControlConfig {
        enabled: true,
        bind: format!("0.0.0.0:{port}").parse().unwrap(),
        password: "m130-no-leak-password".to_string(),
        tls: TlsConfig {
            certificate: None,
            private_key: None,
        },
    };
    let ctx = ServerInitContext::new("test-id".to_string(), vec![]);
    let err = match emissary_cli::i2pcontrol::server::init_server(&failing, tmp.path(), ctx).await {
        Ok(_) => panic!("non-loopback managed startup must fail"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("non-loopback"));
    assert!(!tmp.path().join("i2pcontrol-certs").exists());
    assert!(!tmp.path().join("addressbooks").exists());
    assert!(!tmp.path().join("tunnels").exists());
}

#[test]
fn plaintext_never_reaches_dispatch_tls_only() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    assert!(
        server.contains("TlsAcceptor"),
        "TLS acceptor must own serving"
    );
    assert!(
        server.contains("RequestBodyLimitLayer"),
        "bounded body gate must remain"
    );
    // No plaintext HTTP listener path may dispatch JSON-RPC.
    assert!(
        !server.contains("serve_plaintext") && !server.contains("without_tls"),
        "no plaintext serving path may exist"
    );
}

// ──────────────────────────────────────────────────────────────
// WP3–WP5 representative production inventory (requalification)
// ──────────────────────────────────────────────────────────────

#[test]
fn production_composition_has_no_fake_fallback() {
    let root = workspace_root();
    let server = std::fs::read_to_string(root.join("emissary-cli/src/i2pcontrol/server.rs"))
        .expect("server");
    let init_start = server.find("pub async fn init_server").expect("init_server");
    let init_end = server.find("/// Zero-cost event metrics stub").unwrap_or(server.len());
    let init = &server[init_start..init_end];
    assert!(init.contains("ctx.address_book_handle.ok_or_else"));
    assert!(init.contains("address_books.load().await"));
    assert!(!init.contains("Fake"));
    assert!(!init.contains("new_test"));

    let main = std::fs::read_to_string(root.join("emissary-cli/src/main.rs")).expect("main source");
    assert!(main.contains("with_address_book_handle"));
    assert!(main.contains("with_startup_tunnel_inventory"));
    assert!(main.contains("init_server(&server_config"));
}

#[test]
fn resource_limits_remain_effective() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    let auth = production_section("emissary-cli/src/i2pcontrol/auth.rs");
    let rpc_src = production_section("emissary-cli/src/i2pcontrol/rpc.rs");

    // Documented bounds must still be present with their exact values.
    assert!(server.contains("MAX_BODY_SIZE"), "missing body cap");
    assert!(server.contains("1024 * 1024"), "body cap must stay 1 MiB");
    assert!(
        server.contains("MAX_CONCURRENT_REQUESTS"),
        "missing concurrency bound"
    );
    assert!(
        server.contains("MAX_CONNECTION_TASKS"),
        "missing connection-task bound"
    );
    assert!(
        server.contains("TLS_HANDSHAKE_TIMEOUT"),
        "missing handshake bound"
    );
    assert!(
        server.contains("REQUEST_DEADLINE"),
        "missing request deadline"
    );
    assert!(auth.contains("MAX_TOKENS"), "missing token capacity bound");
    assert!(
        auth.contains("MAX_THROTTLE_ENTRIES"),
        "missing auth-throttle bound"
    );
    assert!(
        rpc_src.contains("MAX_BATCH_ELEMENTS"),
        "missing batch cardinality bound"
    );
    // Batch bound stays within the in-flight budget.
    const _: () = assert!(rpc::MAX_BATCH_ELEMENTS <= 64);
    assert_eq!(rpc::MAX_BATCH_ELEMENTS, 32);
    // Enforcement seams must remain wired.
    assert!(server.contains("RequestBodyLimitLayer::new(MAX_BODY_SIZE)"));
    assert!(server.contains("Semaphore::new(MAX_CONCURRENT_REQUESTS)"));
    assert!(server.contains("Semaphore::new(MAX_CONNECTION_TASKS)"));
}

#[test]
fn secret_safety_no_password_token_key_echo() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    assert!(!server.contains("format!(\"{token"), "must not echo token");
    assert!(!server.contains("token}"), "must not interpolate token");
    let auth = production_section("emissary-cli/src/i2pcontrol/auth.rs");
    assert!(!auth.contains("tracing"), "auth must not log secrets");
    let tls = production_section("emissary-cli/src/i2pcontrol/tls.rs");
    assert!(
        !tls.contains("danger_accept_invalid"),
        "must not weaken client verification"
    );
}

#[test]
fn proposal_matrix_is_mechanically_recomputed() {
    let matrix: toml::Value = std::fs::read_to_string(
        workspace_root()
            .join("plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"),
    )
    .expect("matrix")
    .parse()
    .expect("valid matrix");
    assert_eq!(matrix["proposal_number"].as_integer(), Some(170));
    assert_eq!(matrix["proposal_revision"].as_str(), Some("2026-05-20"));
    assert_eq!(matrix["proposal_status"].as_str(), Some("Open"));

    let router_rows = matrix["router_info"]["rows"].as_array().expect("rows");
    assert_eq!(router_rows.len(), 43);
    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("tunnel types");
    assert_eq!(tunnel_types.len(), 12);
    let setconfig = matrix["addressbook_setconfig"]["rows"].as_array().expect("setconfig");
    assert_eq!(setconfig.len(), 13);
    let selectors = matrix["contract_names"]["client_services_selectors"]
        .as_array()
        .expect("selectors");
    assert_eq!(selectors.len(), 6);

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

    let declared = matrix["current_matrix_counts"].as_table().expect("declared counts");
    assert_eq!(declared["apply"].as_integer(), Some(325));
    assert_eq!(declared["blocked_primitive"].as_integer(), Some(47));
    assert_eq!(declared["not_applicable"].as_integer(), Some(468));
}

#[test]
fn no_unrelated_base_method_parity_is_smuggled() {
    // Canonical scope: unrelated base methods stay explicit METHOD_NOT_FOUND.
    for method in rpc::methods::UNSUPPORTED_BASE {
        let entry = rpc::methods::SUPPORT_INVENTORY
            .iter()
            .find(|entry| entry.method == *method)
            .unwrap_or_else(|| panic!("missing inventory entry for {method}"));
        assert_eq!(
            entry.disposition,
            rpc::methods::SupportDisposition::UnsupportedBase,
            "{method} must stay unsupported"
        );
    }
    // Proposal inventory shape unchanged: Authenticate + RouterInfo base,
    // AddressBook/TunnelManager/ClientServicesInfo Proposal, aliases kept.
    assert!(rpc::methods::SUPPORT_INVENTORY
        .iter()
        .any(|entry| entry.method == rpc::methods::AUTHENTICATE));
    assert!(rpc::methods::SUPPORT_INVENTORY
        .iter()
        .any(|entry| entry.method == rpc::methods::ROUTER_INFO));
    assert_eq!(rpc::methods::PROTECTED_DISPATCH.len(), 6);
}

// ──────────────────────────────────────────────────────────────
// WP6: containment and dependency audit
// ──────────────────────────────────────────────────────────────

#[test]
fn production_changes_stay_under_i2pcontrol() {
    // M130 authorizes no core/util/router/frontend/dependency change.
    // Scope this historical guard to the M130 implementation range so later
    // accepted lifecycle seams are evaluated by M061/M062 and M139.
    let baseline = "579a22c";
    let reviewed_head = "fe1a981";
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
        "M130 must not change core/util/config/dependency paths: {changed:?}"
    );

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
            "M130 production change outside i2pcontrol: {path}"
        );
    }
}

#[test]
fn yosemite_alias_remains_optional_exact_and_isolated() {
    let manifest = source("emissary-cli/Cargo.toml");
    assert!(manifest.contains("yosemite-i2pcontrol"));
    assert!(manifest.contains("optional = true"));
    assert!(manifest.contains("dep:yosemite-i2pcontrol"));
    assert!(manifest.contains("59140a2277bf296928d2e8ce39a148182eeff044"));

    let lockfile = source("Cargo.lock");
    assert!(lockfile.contains(
        "git+https://github.com/eggstack/yosemite?rev=59140a2277bf296928d2e8ce39a148182eeff044"
    ));
    assert!(lockfile.contains("registry+https://github.com/rust-lang/crates.io-index"));
}

// ──────────────────────────────────────────────────────────────
// WP7: active authority reconciliation
// ──────────────────────────────────────────────────────────────

#[test]
fn active_authority_retains_partial_support_and_m130_lineage() {
    let root = workspace_root();
    let planning = [
        root.join("plans/registry.md"),
        root.join("plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md"),
        root.join("plans/implementation/i2pcontrol-proposal-170/README.md"),
    ];
    for path in planning {
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        assert!(
            text.contains("325") && text.contains("47") && text.contains("468"),
            "{} does not state the current 325/47/468 authority",
            path.display()
        );
        assert!(
            text.to_ascii_lowercase().contains("partial"),
            "{} must retain partial-support status",
            path.display()
        );
        assert!(
            text.contains("M130"),
            "{} must name the M130 requalification lineage",
            path.display()
        );
        assert!(
            text.contains("M127") && text.contains("M128") && text.contains("M129"),
            "{} must retain the M127–M129 corrective lineage",
            path.display()
        );
        // M130 must not be left blocked/unregistered after the hard
        // dependencies closed.
        assert!(
            !text.contains("M130") || !text.contains("BLOCKED / UNREGISTERED"),
            "{} retains a stale blocked M130 marker",
            path.display()
        );
    }

    // Active user-facing docs retain partial wording and never present a
    // standalone full-support status while residuals remain blocked.
    for name in [
        "docs/i2pcontrol/README.md",
        "docs/i2pcontrol/proposal-170-support.md",
        "AGENTS.md",
    ] {
        let text = std::fs::read_to_string(root.join(name)).expect("active doc");
        assert!(
            text.to_ascii_lowercase().contains("partial"),
            "{name} must retain partial-support wording"
        );
        assert!(
            !text.contains("Status: full Proposal 170 support")
                && !text.contains("Status: **full Proposal 170 support"),
            "{name} must not claim full Proposal 170 support"
        );
    }
}
