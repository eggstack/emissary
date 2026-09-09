//! M139 historical qualification and post-lifecycle composition guards.
//!
//! M139 closed at `325/47/468` and was the last whole-surface qualification
//! before M141-M145 production work. M153 supersedes M139 for current-head
//! runtime/security qualification; this suite now owns M139's immutable
//! historical evidence (closure counts, production head, 47-cell residual
//! set) plus the durable lifecycle composition checks. Current-head aggregate
//! counts are owned by the M153 guard.
//!
//! M126/M130 retain their historical milestone evidence, while the M153 suite
//! owns the durable current matrix/documentation checks. The lifecycle checks
//! execute the existing deterministic fake-runtime tests from the package
//! library so this test remains an integration boundary without wall-clock
//! sleeps or production-only hooks.

#![cfg(feature = "i2pcontrol")]

use std::{collections::BTreeSet, path::Path, process::Command};

use toml::Value;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn planning_file(name: &str) -> Value {
    std::fs::read_to_string(
        workspace_root().join("plans/implementation/i2pcontrol-proposal-170").join(name),
    )
    .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
    .parse()
    .unwrap_or_else(|error| panic!("invalid TOML in {name}: {error}"))
}

#[test]
fn current_matrix_is_exhaustive_and_residuals_are_exact() {
    // M153 rebase: M139's `325/47/468` authority is historical. Pin it to the
    // immutable M139 closure, and prove the current head only ever removed
    // cells through later accepted promotions (current blocked ⊆ M139 blocked,
    // no new blockers). Exact current-head aggregates live in the M153 guard.
    let closure = std::fs::read_to_string(
        workspace_root().join("plans/closure/i2pcontrol-proposal-170/139-closure.md"),
    )
    .expect("M139 closure must exist");
    assert!(
        closure.contains("325/47/468"),
        "M139 closure must retain its historical 325/47/468 authority"
    );
    assert!(
        closure.contains("e4f217cb1459e26bf011da46b67fc2c83cd192b5"),
        "M139 closure must retain its historical production head"
    );

    let matrix = planning_file("095-full-support-matrix.toml");
    assert_eq!(matrix["proposal_number"].as_integer(), Some(170));
    assert_eq!(matrix["proposal_revision"].as_str(), Some("2026-05-20"));
    assert_eq!(matrix["proposal_status"].as_str(), Some("Open"));

    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("canonical tunnel types");
    assert_eq!(tunnel_types.len(), 12);

    let mut blocked = BTreeSet::new();
    let mut total = 0usize;
    for row in matrix["tunnel_manager"]["options"].as_array().expect("TunnelManager options") {
        let option = row["canonical_key"].as_str().expect("canonical option");
        let cells = row["cells"].as_array().expect("option cells");
        assert_eq!(cells.len(), tunnel_types.len());
        for (index, cell) in cells.iter().enumerate() {
            total += 1;
            if cell.as_str().expect("cell disposition") == "blocked_primitive" {
                blocked.insert((
                    option.to_owned(),
                    tunnel_types[index].as_str().expect("tunnel family").to_owned(),
                ));
            }
        }
    }
    assert_eq!(total, 840);

    let historical = [
        ("ConnectDelay", "streamrclient"),
        ("EncryptLeaseSet", "server"),
        ("EncryptLeaseSet", "httpserver"),
        ("EncryptLeaseSet", "httpbidirserver"),
        ("EncryptLeaseSet", "ircserver"),
        ("EncryptLeaseSet", "streamrserver"),
        ("JumpList", "httpclient"),
        ("LeaseSetClientAuths", "server"),
        ("LeaseSetClientAuths", "httpserver"),
        ("LeaseSetClientAuths", "httpbidirserver"),
        ("LeaseSetClientAuths", "ircserver"),
        ("LeaseSetClientAuths", "streamrserver"),
        ("MultiHoming", "httpserver"),
        ("MultiHoming", "httpbidirserver"),
        ("OptionalLookup", "server"),
        ("OptionalLookup", "httpserver"),
        ("OptionalLookup", "httpbidirserver"),
        ("OptionalLookup", "ircserver"),
        ("OptionalLookup", "streamrserver"),
        ("Profile", "client"),
        ("Profile", "httpclient"),
        ("Profile", "ircclient"),
        ("Profile", "socks"),
        ("Profile", "socksirc"),
        ("Profile", "connectclient"),
        ("Profile", "streamrclient"),
        ("SSLProxies", "httpclient"),
        ("SigType", "client"),
        ("SigType", "httpclient"),
        ("SigType", "ircclient"),
        ("SigType", "socks"),
        ("SigType", "socksirc"),
        ("SigType", "connectclient"),
        ("SigType", "server"),
        ("SigType", "httpserver"),
        ("SigType", "httpbidirserver"),
        ("SigType", "ircserver"),
        ("UniqueLocalAddressPerClient", "httpserver"),
        ("UniqueLocalAddressPerClient", "httpbidirserver"),
        ("UseOutproxyPlugin", "httpclient"),
        ("UseOutproxyPlugin", "socks"),
        ("UseOutproxyPlugin", "socksirc"),
        ("UseOutproxyPlugin", "connectclient"),
        ("UseSSL", "httpclient"),
        ("UseSSL", "connectclient"),
        ("UseSSL", "httpserver"),
        ("UseSSL", "httpbidirserver"),
    ]
    .into_iter()
    .map(|(option, family)| (option.to_owned(), family.to_owned()))
    .collect::<BTreeSet<_>>();
    assert_eq!(historical.len(), 47);
    // Later accepted milestones (M140-M145) only promoted or reclassified
    // cells; no new blocked_primitive cell may appear outside M139's set.
    assert!(
        blocked.is_subset(&historical),
        "current blocked set must be a subset of the M139 historical 47-cell set: {blocked:?}"
    );
    // M141-M145 legitimately promoted 11 cells plus M140 reclassified 7, so
    // the current head must be strictly smaller than the historical set.
    assert!(
        blocked.len() < historical.len(),
        "current blocked set must reflect later accepted promotions"
    );

    // Declared counts must be mechanically self-consistent; exact values are
    // owned by the M153 guard.
    let declared = matrix["current_matrix_counts"].as_table().expect("declared counts");
    assert_eq!(declared["total"].as_integer(), Some(840));
    assert_eq!(
        declared["blocked_primitive"].as_integer(),
        Some(blocked.len() as i64)
    );
}

#[test]
fn active_authority_is_m139_and_support_remains_partial() {
    let root = workspace_root();
    let active = [
        "AGENTS.md",
        "plans/registry.md",
        "plans/implementation/i2pcontrol-proposal-170/README.md",
        "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md",
        "plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md",
        "docs/i2pcontrol/README.md",
        "docs/i2pcontrol/proposal-170-support.md",
    ];
    for name in active {
        let text = std::fs::read_to_string(root.join(name)).expect("active authority document");
        let lower = text.to_ascii_lowercase();
        assert!(
            lower.contains("partial"),
            "{name} must retain partial support"
        );
        assert!(text.contains("M139"), "{name} must name M139");
        assert!(
            !text.contains("Status: full Proposal 170 support")
                && !text.contains("Status: **full Proposal 170 support"),
            "{name} must not claim full Proposal 170 support"
        );
        for line in text.lines() {
            let lower = line.to_ascii_lowercase();
            let old_matrix = (line.contains("284") && line.contains("96") && line.contains("460"))
                || (line.contains("284") && line.contains("88") && line.contains("468"));
            assert!(
                !(old_matrix && (lower.contains("current") || lower.contains("authority"))),
                "{name} retains an obsolete matrix as current authority: {line}"
            );
        }
    }

    let post_m114 = std::fs::read_to_string(
        root.join("plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md"),
    )
    .expect("post-M114 roadmap");
    assert!(post_m114.contains("M130 historical"));
    assert!(post_m114.contains("M139 later supersedes M130"));
}

fn run_deterministic_library_test(filter: &str) {
    let output = Command::new("cargo")
        .args([
            "test",
            "-p",
            "emissary-cli",
            "--no-default-features",
            "--features",
            "i2pcontrol",
            "--lib",
            filter,
        ])
        .current_dir(workspace_root())
        .output()
        .unwrap_or_else(|error| panic!("failed to run deterministic test {filter}: {error}"));
    assert!(
        output.status.success(),
        "deterministic lifecycle test {filter} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn lifecycle_composition_and_manual_negative_paths_are_deterministic() {
    // M137 drives idle reduction followed by close with fake time; M134 then
    // consumes the authoritative IdlePolicy fact and rotates exactly once via
    // fake SAM. The dedicated test also covers manual Stop and Restart. The
    // process-restart and reason tests provide the negative no-rotation gates.
    for filter in [
        "m137_close_after_reduce_allows_reduce_then_later_close",
        "m134_dedicated_proven_resume_rotates_once_end_to_end",
        "m134_process_restart_reuses_committed_without_replay",
        "m137_manual_stop_and_failure_reasons_are_not_idle",
    ] {
        run_deterministic_library_test(filter);
    }
}
