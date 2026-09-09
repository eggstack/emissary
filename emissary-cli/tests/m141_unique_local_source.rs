//! M141 HTTP unique-local source-address completion guard.
//!
//! Planning evidence, not a runtime capability source. This test keeps the
//! two-cell promotion mechanically tied to the M095 machine authority and
//! proves the I2PControl-local validation/round-trip/containment contract
//! without requiring a router or network access.

#![cfg(feature = "i2pcontrol")]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use toml::Value;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn planning_toml(name: &str) -> Value {
    let path = workspace_root().join("plans/implementation/i2pcontrol-proposal-170").join(name);
    let raw = std::fs::read_to_string(&path).expect("M141 planning file must exist");
    raw.parse().expect("M141 planning file must be valid TOML")
}

fn string_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("M141 record is missing string field {key}"))
}

#[test]
fn m141_matrix_promotes_exactly_two_unique_local_cells() {
    // M153 rebase: M141 closed at `327/38/475`. Its closure remains the
    // immutable milestone-local authority; the current head has since advanced
    // through M142-M145. Pin closure evidence plus the durable M141 cell facts
    // (exact current aggregates live in the M153 guard).
    let closure = std::fs::read_to_string(
        workspace_root().join("plans/closure/i2pcontrol-proposal-170/141-closure.md"),
    )
    .expect("M141 closure must exist");
    assert!(
        closure.contains("327/38/475"),
        "M141 closure must retain its historical 327/38/475 authority"
    );
    let matrix = planning_toml("095-full-support-matrix.toml");

    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("canonical tunnel types");
    assert_eq!(tunnel_types.len(), 12);
    let mut blocked = BTreeSet::new();
    let mut applied = BTreeSet::new();
    for row in matrix["tunnel_manager"]["options"].as_array().expect("options") {
        let option = row["canonical_key"].as_str().expect("canonical option");
        let cells = row["cells"].as_array().expect("option cells");
        assert_eq!(cells.len(), tunnel_types.len());
        for (index, cell) in cells.iter().enumerate() {
            let family = tunnel_types[index].as_str().expect("family").to_owned();
            match cell.as_str().expect("cell disposition") {
                "apply" => {
                    applied.insert((option.to_owned(), family));
                }
                "blocked_primitive" => {
                    blocked.insert((option.to_owned(), family));
                }
                "not_applicable" => {}
                other => panic!("unexpected cell disposition {other}"),
            }
        }
    }
    // Exactly the two M141 target cells are apply at the current head.
    for cell in [
        ("UniqueLocalAddressPerClient", "httpserver"),
        ("UniqueLocalAddressPerClient", "httpbidirserver"),
    ] {
        let (option, family) = (&cell.0.to_owned(), &cell.1.to_owned());
        assert!(
            applied.contains(&(option.clone(), family.clone())),
            "M141 cell {option}:{family} must be apply"
        );
        assert!(
            !blocked.contains(&(option.clone(), family.clone())),
            "M141 cell {option}:{family} must not remain blocked"
        );
    }
    // No other UniqueLocal cell may be apply.
    for (option, family) in &applied {
        if option == "UniqueLocalAddressPerClient" {
            assert!(
                family == "httpserver" || family == "httpbidirserver",
                "unexpected UniqueLocal apply cell {option}:{family}"
            );
        }
    }
    // The M141 row owns the promotion with exact evidence.
    let options = matrix["tunnel_manager"]["options"].as_array().unwrap();
    let row = options
        .iter()
        .find(|row| string_field(row, "canonical_key") == "UniqueLocalAddressPerClient")
        .expect("UniqueLocal row must exist");
    assert_eq!(string_field(row, "completion_owner"), "M141");
    assert_eq!(
        string_field(row, "current_or_planned_disposition"),
        "apply_or_not_applicable"
    );
    assert!(row.get("blocking_milestone").is_none());
    assert!(row.get("blocked_primitive").is_none());
    let cells = row["cells"].as_array().unwrap();
    assert_eq!(cells[8].as_str(), Some("apply"));
    assert_eq!(cells[9].as_str(), Some("apply"));
    for (index, cell) in cells.iter().enumerate() {
        if index != 8 && index != 9 {
            assert_eq!(
                cell.as_str(),
                Some("not_applicable"),
                "UniqueLocal cell {index} must stay N/A"
            );
        }
    }
}

#[test]
fn m141_residual_inventory_subtracts_two_cells() {
    // M153 rebase: pin the durable M141 cell facts (the two promoted cells are
    // gone from the blocked set; unrelated residuals remain) without pinning
    // the superseded `38`-cell aggregate. Exact current counts live in M153.
    let matrix = planning_toml("095-full-support-matrix.toml");
    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"].as_array().unwrap();
    let current_blocked: BTreeSet<(String, String)> = matrix["tunnel_manager"]["options"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| {
            let key = string_field(row, "canonical_key").to_owned();
            row["cells"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.as_str() == Some("blocked_primitive"))
                .map(move |(index, _)| {
                    (
                        key.clone(),
                        tunnel_types[index].as_str().unwrap().to_owned(),
                    )
                })
        })
        .collect();
    assert!(
        !current_blocked.contains(&(
            "UniqueLocalAddressPerClient".to_owned(),
            "httpserver".to_owned()
        )),
        "M141 httpserver cell must not remain blocked"
    );
    assert!(
        !current_blocked.contains(&(
            "UniqueLocalAddressPerClient".to_owned(),
            "httpbidirserver".to_owned()
        )),
        "M141 httpbidirserver cell must not remain blocked"
    );
    // M141 must not disturb unrelated residuals. Spot-check cells that were
    // blocked at M141 time and remain blocked at the current head (later
    // milestones legitimately promoted MultiHoming/Profile/UseSSL, so those
    // are not durable spot-checks here).
    for cell in [
        ("SigType", "client"),
        ("SigType", "httpserver"),
        ("EncryptLeaseSet", "server"),
        ("OptionalLookup", "httpserver"),
        ("LeaseSetClientAuths", "ircserver"),
        ("UseOutproxyPlugin", "httpclient"),
    ] {
        assert!(
            current_blocked.contains(&(cell.0.to_owned(), cell.1.to_owned())),
            "M141 must not disturb residual cell {}:{}",
            cell.0,
            cell.1
        );
    }
}

#[test]
fn m141_http_targets_validate_boolean_before_allocation() {
    use emissary_cli::i2pcontrol::{
        backends::{
            http_bidir::HttpBidirServerTunnelBackend, http_server::HttpServerTunnelBackend,
            TunnelBackend,
        },
        domain::tunnel::{
            StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
            TunnelRuntimeState, TunnelType,
        },
        server_secret_store::ServerDestinationStore,
    };

    fn http_server_def(raw: Vec<(String, serde_json::Value)>) -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("http-server").unwrap(),
            tunnel_type: TunnelType::HttpServer,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions {
                target_port: Some(8080),
                ..Default::default()
            },
            raw_config: raw.into_iter().collect::<BTreeMap<_, _>>(),
        }
    }

    fn http_bidir_def(raw: Vec<(String, serde_json::Value)>) -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("http-bidir").unwrap(),
            tunnel_type: TunnelType::HttpBidirServer,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions {
                target_port: Some(8080),
                listen_port: Some(0),
                ..Default::default()
            },
            raw_config: raw.into_iter().collect::<BTreeMap<_, _>>(),
        }
    }

    let root = tempfile::tempdir().unwrap();
    let http = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
    let bidir =
        HttpBidirServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()), None);

    // Absent and explicit false are disabled; true is enabled. All validate.
    for raw in [
        vec![],
        vec![(
            "UniqueLocalAddressPerClient".to_owned(),
            serde_json::json!(false),
        )],
        vec![(
            "UniqueLocalAddressPerClient".to_owned(),
            serde_json::json!(true),
        )],
    ] {
        assert!(http.validate_start(&http_server_def(raw.clone())).is_ok());
        assert!(bidir.validate_start(&http_bidir_def(raw)).is_ok());
    }
    // Malformed types fail before allocation/effect.
    for bad in [
        serde_json::json!("true"),
        serde_json::json!(1),
        serde_json::json!(0),
        serde_json::json!({"enabled": true}),
    ] {
        let def = http_server_def(vec![(
            "UniqueLocalAddressPerClient".to_owned(),
            bad.clone(),
        )]);
        assert!(http.validate_start(&def).is_err());
        let def = http_bidir_def(vec![("UniqueLocalAddressPerClient".to_owned(), bad)]);
        assert!(bidir.validate_start(&def).is_err());
    }
    // Hostile targets stay rejected even with the option enabled.
    for hostile in ["10.0.0.1", "example.com", "127.0.0.2"] {
        let def = http_server_def(vec![
            ("TargetHost".to_owned(), serde_json::json!(hostile)),
            (
                "UniqueLocalAddressPerClient".to_owned(),
                serde_json::json!(true),
            ),
        ]);
        assert!(http.validate_start(&def).is_err());
        let def = http_bidir_def(vec![
            ("TargetHost".to_owned(), serde_json::json!(hostile)),
            (
                "UniqueLocalAddressPerClient".to_owned(),
                serde_json::json!(true),
            ),
        ]);
        assert!(bidir.validate_start(&def).is_err());
    }
    // Get round-trip preserves exact boolean spelling without peer data.
    let def = http_server_def(vec![(
        "UniqueLocalAddressPerClient".to_owned(),
        serde_json::json!(true),
    )]);
    let json = serde_json::to_value(&def).unwrap();
    let restored: TunnelDefinition = serde_json::from_value(json).unwrap();
    assert_eq!(
        restored.raw_config.get("UniqueLocalAddressPerClient"),
        Some(&serde_json::json!(true))
    );
    let debug = format!("{restored:?}");
    assert!(!debug.contains("true") || debug.contains("raw_config_keys"));
}

#[test]
fn m141_non_target_families_reject_before_allocation() {
    use emissary_cli::i2pcontrol::{
        backends::{
            irc_server::IrcServerTunnelBackend, server::ServerTunnelBackend, TunnelBackend,
        },
        domain::tunnel::{
            StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
            TunnelRuntimeState, TunnelType,
        },
        server_secret_store::ServerDestinationStore,
    };

    // Generic server and IRC server never own the HTTP unique-local primitive.
    // Any supplied value — including explicit false — fails before allocation.
    let root = tempfile::tempdir().unwrap();
    let server = ServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
    let irc = IrcServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
    for (tunnel_type, backend) in [
        (TunnelType::Server, &server as &dyn TunnelBackend),
        (TunnelType::IrcServer, &irc as &dyn TunnelBackend),
    ] {
        for value in [serde_json::json!(true), serde_json::json!(false)] {
            let def = TunnelDefinition {
                name: TunnelName::new("non-target").unwrap(),
                tunnel_type,
                ownership: TunnelOwnership::ControlPlane,
                runtime_state: TunnelRuntimeState::Stopped,
                start_intent: StartIntent::DoNotStart,
                options: TunnelOptions {
                    target_port: Some(8080),
                    ..Default::default()
                },
                raw_config: BTreeMap::from([("UniqueLocalAddressPerClient".to_owned(), value)]),
            };
            assert!(
                backend.validate_start(&def).is_err(),
                "{tunnel_type} must reject UniqueLocalAddressPerClient before allocation"
            );
        }
    }
}

#[test]
fn m141_shared_helper_has_no_second_source_algorithm() {
    let root = workspace_root();
    let server_full =
        std::fs::read_to_string(root.join("emissary-cli/src/i2pcontrol/backends/http_server.rs"))
            .expect("http_server source");
    let bidir_full =
        std::fs::read_to_string(root.join("emissary-cli/src/i2pcontrol/backends/http_bidir.rs"))
            .expect("http_bidir source");
    // Production code is everything before the trailing `mod tests` module.
    let server = server_full
        .split("#[cfg(test)]\nmod tests")
        .next()
        .unwrap_or(server_full.as_str());
    let bidir = bidir_full
        .split("#[cfg(test)]\nmod tests")
        .next()
        .unwrap_or(bidir_full.as_str());
    // The single derivation owner lives in http_server.rs.
    assert!(server_full.contains("fn derive_unique_local_source"));
    assert!(server_full.contains("0xfd"));
    assert!(bidir_full.contains("make_accepted_handler"));
    assert!(bidir_full.contains("unique_local"));
    // The composite must not grow a second derivation algorithm.
    assert!(
        !bidir.contains("fn derive_unique_local_source"),
        "httpbidirserver must reuse the shared helper, not duplicate it"
    );
    assert!(
        !bidir.contains("0xfd"),
        "httpbidirserver must not duplicate the IPv6 ULA layout"
    );
    // No DNS or listener is introduced by the feature (production code only).
    for source in [&server, &bidir] {
        assert!(!source.contains("to_socket_addrs"));
        assert!(!source.contains("getaddrinfo"));
    }
    assert!(!server.contains("TcpListener::bind"));
}

#[test]
fn m141_docs_and_registry_agree_on_promotion() {
    let root = workspace_root();
    for name in [
        "AGENTS.md",
        "plans/registry.md",
        "plans/implementation/i2pcontrol-proposal-170/README.md",
        "docs/i2pcontrol/proposal-170-support.md",
        "docs/i2pcontrol/tunnel-manager.md",
        "docs/i2pcontrol/tunnel-backends.md",
    ] {
        let text = std::fs::read_to_string(root.join(name)).expect("M141 doc must exist");
        assert!(
            text.contains("327") || text.contains("UniqueLocal") || text.contains("M141"),
            "{name} must mention the M141 promotion"
        );
        assert!(
            !text.contains("Status: **full Proposal 170 support"),
            "{name} must not claim full support"
        );
    }
}
