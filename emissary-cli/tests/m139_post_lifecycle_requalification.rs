//! M139 current-head authority and post-lifecycle composition guards.
//!
//! M126/M130 retain their historical milestone evidence, while this suite
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
        workspace_root()
            .join("plans/implementation/i2pcontrol-proposal-170")
            .join(name),
    )
    .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
    .parse()
    .unwrap_or_else(|error| panic!("invalid TOML in {name}: {error}"))
}

#[test]
fn current_matrix_is_exhaustive_and_residuals_are_exact() {
    let matrix = planning_file("095-full-support-matrix.toml");
    assert_eq!(matrix["proposal_number"].as_integer(), Some(170));
    assert_eq!(matrix["proposal_revision"].as_str(), Some("2026-05-20"));
    assert_eq!(matrix["proposal_status"].as_str(), Some("Open"));
    assert_eq!(
        matrix["current_production_head"].as_str(),
        Some("e4f217cb1459e26bf011da46b67fc2c83cd192b5")
    );

    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("canonical tunnel types");
    assert_eq!(tunnel_types.len(), 12);

    let mut counts = [0usize; 3];
    let mut blocked = BTreeSet::new();
    for row in matrix["tunnel_manager"]["options"]
        .as_array()
        .expect("TunnelManager options")
    {
        let option = row["canonical_key"].as_str().expect("canonical option");
        let cells = row["cells"].as_array().expect("option cells");
        assert_eq!(cells.len(), tunnel_types.len());
        for (index, cell) in cells.iter().enumerate() {
            match cell.as_str().expect("cell disposition") {
                "apply" => counts[0] += 1,
                "blocked_primitive" => {
                    counts[1] += 1;
                    blocked.insert((
                        option.to_owned(),
                        tunnel_types[index].as_str().expect("tunnel family").to_owned(),
                    ));
                }
                "not_applicable" => counts[2] += 1,
                other => panic!("unexpected cell disposition {other}"),
            }
        }
    }
    assert_eq!(counts, [325, 47, 468]);
    assert_eq!(
        matrix["current_matrix_counts"],
        Value::Table(
            [
                ("total".to_owned(), Value::Integer(840)),
                ("apply".to_owned(), Value::Integer(325)),
                ("blocked_primitive".to_owned(), Value::Integer(47)),
                ("not_applicable".to_owned(), Value::Integer(468)),
            ]
            .into_iter()
            .collect(),
        )
    );

    let expected = [
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
    assert_eq!(blocked, expected);
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
        assert!(lower.contains("partial"), "{name} must retain partial support");
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
