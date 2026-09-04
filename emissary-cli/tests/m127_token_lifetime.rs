//! M127 token-lifetime corrective guards.
//!
//! Proves `TOKEN_EXPIRED` (`-32004`) is reachable protocol surface (not dead),
//! token storage carries finite monotonic expiry state, dispatch maps
//! expired/unknown distinctly, input/capacity remain bounded, production
//! changes stay under `i2pcontrol`, and the Proposal matrix is unchanged.
//! Deterministic expiry-boundary behavior itself is covered by unit tests in
//! `auth.rs`/`server.rs` with a manual monotonic clock; this file provides
//! the durable static/contract regression guard plus live pre-expiry proof.

#![cfg(feature = "i2pcontrol")]

use std::{path::Path, process::Command};

use emissary_cli::i2pcontrol::{
    auth::{TokenService, TokenValidation},
    rpc,
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

#[test]
fn token_expired_error_contract_is_declared() {
    assert_eq!(rpc::error_codes::TOKEN_EXPIRED, -32004);
    assert_eq!(rpc::error_codes::INVALID_TOKEN, -32003);
    assert_eq!(
        rpc::error_codes::TOKEN_EXPIRED_MESSAGE,
        "The provided authentication token was expired and will be removed"
    );
    assert_eq!(
        rpc::error_codes::INVALID_TOKEN_MESSAGE,
        "Authentication token doesn't exist"
    );
}

#[test]
fn issued_tokens_authorize_before_expiry_live() {
    let svc = TokenService::new();
    let token = svc.issue();
    assert_eq!(token.len(), 64, "opaque hex token shape unchanged");
    assert_eq!(svc.validate(&token), TokenValidation::Valid);
    assert_eq!(svc.validate("never-issued"), TokenValidation::Unknown);
}

#[test]
fn oversized_credentials_fail_as_unknown() {
    let svc = TokenService::new();
    let oversized = "x".repeat(1024);
    assert_eq!(svc.validate(&oversized), TokenValidation::Unknown);
    let huge = "y".repeat(1024 * 1024);
    assert_eq!(svc.validate(&huge), TokenValidation::Unknown);
    assert_eq!(svc.validate(""), TokenValidation::Unknown);
}

#[test]
fn token_store_has_finite_expiry_state() {
    let auth = production_section("emissary-cli/src/i2pcontrol/auth.rs");
    // Named compatibility lifetime, not a magic value.
    assert!(auth.contains("TOKEN_LIFETIME"), "missing lifetime constant");
    assert!(
        auth.contains("24 * 60 * 60"),
        "lifetime must be the documented one-day reference value"
    );
    // Membership-only storage must be gone.
    assert!(
        !auth.contains("tokens: HashMap<String, ()>"),
        "membership-only token state regressed"
    );
    assert!(
        auth.contains("HashMap<String, Instant>"),
        "token record must carry monotonic expiry state"
    );
    // Three internal outcomes, atomically distinguished.
    assert!(auth.contains("enum TokenValidation"), "missing outcome enum");
    assert!(auth.contains("Expired"), "missing expired outcome");
    assert!(auth.contains("Unknown"), "missing unknown outcome");
    assert!(
        auth.contains("remove_expired") || auth.contains("expires_at"),
        "missing expiry bookkeeping"
    );
    // Monotonic source, no wall-clock expiry.
    assert!(auth.contains("Instant::now"), "must use monotonic clock");
    // Bounded presented-credential gate before lookup.
    assert!(
        auth.contains("MAX_PRESENTED_TOKEN_LEN"),
        "missing input bound"
    );
    // Test-only clock must not leak into production composition.
    assert!(
        !auth.contains("new_manual_for_test") || auth.contains("#[cfg(test)]"),
        "manual clock must stay test-gated"
    );
}

#[test]
fn dispatch_maps_expired_and_unknown_distinctly() {
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
    // Both mappings must exist; collapsing expired into INVALID_TOKEN regresses M127.
    let expired_pos = server.find("TokenValidation::Expired").expect("expired arm");
    let unknown_pos = server.find("TokenValidation::Unknown").expect("unknown arm");
    let window = &server[expired_pos..server.len().min(expired_pos + 600)];
    assert!(
        window.contains("TOKEN_EXPIRED"),
        "expired arm must return TOKEN_EXPIRED, not INVALID_TOKEN"
    );
    let window = &server[unknown_pos..server.len().min(unknown_pos + 600)];
    assert!(
        window.contains("INVALID_TOKEN"),
        "unknown arm must return INVALID_TOKEN"
    );
}

#[test]
fn no_background_scanner_or_timer_task() {
    let auth = production_section("emissary-cli/src/i2pcontrol/auth.rs");
    for forbidden in ["tokio::spawn", "spawn(", "set_interval", "interval(", "sleep("] {
        assert!(
            !auth.contains(forbidden),
            "auth must not introduce a background scanner/timer: found {forbidden}"
        );
    }
    // No new file persistence or dependency for tokens.
    for forbidden in ["std::fs::", "tokio::fs::", "std::time::SystemTime"] {
        assert!(
            !auth.contains(forbidden),
            "token lifetime must stay process-local/monotonic: found {forbidden}"
        );
    }
}

#[test]
fn error_and_log_paths_do_not_echo_token_material() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    // Static messages only; the presented token value must never be interpolated.
    assert!(!server.contains("format!(\"{token"), "error must not echo token");
    assert!(
        !server.contains("token}"),
        "server production section must not interpolate token material"
    );
    let auth = production_section("emissary-cli/src/i2pcontrol/auth.rs");
    assert!(
        !auth.contains("tracing"),
        "auth must not log token/expiry material"
    );
}

#[test]
fn production_changes_stay_under_i2pcontrol() {
    // M127 authorizes no core/util/router/frontend/dependency change.
    // Diff the M127 planning baseline against the working tree for
    // production source outside the I2PControl boundary.
    let baseline = "9948cfd0782a3defbd5f68cf2d4523603bdc7940";
    let output = Command::new("git")
        .args([
            "diff",
            "--name-only",
            baseline,
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
        "M127 must not change core/util/config/dependency paths: {changed:?}"
    );

    // Changed production files under emissary-cli/src must all be I2PControl-owned.
    let output = Command::new("git")
        .args(["diff", "--name-only", baseline, "--", "emissary-cli/src"])
        .current_dir(workspace_root())
        .output()
        .expect("git diff for i2pcontrol boundary");
    assert!(output.status.success());
    let changed = String::from_utf8_lossy(&output.stdout);
    for path in changed.lines().filter(|l| !l.trim().is_empty()) {
        assert!(
            path.starts_with("emissary-cli/src/i2pcontrol/"),
            "M127 production change outside i2pcontrol: {path}"
        );
    }
}

#[test]
fn proposal_matrix_unchanged_by_token_lifetime() {
    let matrix: toml::Value = std::fs::read_to_string(
        workspace_root().join("plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"),
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
    assert_eq!(counts.get("apply"), Some(&284));
    assert_eq!(counts.get("blocked_primitive"), Some(&96));
    assert_eq!(counts.get("not_applicable"), Some(&460));
}
