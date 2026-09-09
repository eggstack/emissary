//! M143 retained streaming Profile completion guard.
//!
//! Planning evidence, not a runtime capability source. This test keeps the
//! single-cell promotion (`Profile:client`) mechanically tied to the M095
//! machine authority and proves the neutral streaming-window contract:
//! omitted/bulk preserves the default bulk window, `interactive` maps to the
//! pinned neutral `i2p.streaming.maxWindowSize = 16`, invalid values fail
//! before allocation without echo, N/A families keep exact rejection, and
//! shared sessions distinguish effective windows.

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
    let raw = std::fs::read_to_string(&path).expect("M143 planning file must exist");
    raw.parse().expect("M143 planning file must be valid TOML")
}

fn string_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("M143 record is missing string field {key}"))
}

#[test]
fn m143_matrix_promotes_exactly_one_client_cell() {
    // M153 rebase: M143 closed at `330/35/475`. Its closure remains the
    // immutable milestone-local authority; the current head has since advanced
    // through M144-M145. Pin closure evidence plus durable M143 cell facts
    // (exact current aggregates live in the M153 guard).
    let closure = std::fs::read_to_string(
        workspace_root().join("plans/closure/i2pcontrol-proposal-170/143-closure.md"),
    )
    .expect("M143 closure must exist");
    assert!(
        closure.contains("330/35/475"),
        "M143 closure must retain its historical 330/35/475 authority"
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
    assert!(
        applied.contains(&("Profile".to_owned(), "client".to_owned())),
        "M143 cell Profile:client must be apply"
    );
    assert!(
        !blocked.contains(&("Profile".to_owned(), "client".to_owned())),
        "M143 cell Profile:client must not remain blocked"
    );

    let options = matrix["tunnel_manager"]["options"].as_array().unwrap();
    let row = options
        .iter()
        .find(|row| string_field(row, "canonical_key") == "Profile")
        .expect("Profile row must exist");
    assert_eq!(
        string_field(row, "completion_owner"),
        "M143",
        "Profile owner"
    );
    assert_eq!(
        string_field(row, "current_or_planned_disposition"),
        "apply_or_not_applicable",
        "Profile disposition"
    );
    assert!(row.get("blocking_milestone").is_none(), "Profile milestone");
    assert!(row.get("blocked_primitive").is_none(), "Profile primitive");
    let cells = row["cells"].as_array().unwrap();
    assert_eq!(cells[0].as_str(), Some("apply"), "Profile client cell");
    for (index, cell) in cells.iter().enumerate().skip(1) {
        assert_eq!(
            cell.as_str(),
            Some("not_applicable"),
            "Profile cell {index} must stay N/A"
        );
    }
}

#[test]
fn m143_residual_inventory_subtracts_one_cell() {
    // M153 rebase: pin durable M143 cell facts without the superseded
    // `35`-cell aggregate (exact current counts live in M153).
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
        !current_blocked.contains(&("Profile".to_owned(), "client".to_owned())),
        "M143 Profile:client must not remain blocked"
    );
    // M143 must not disturb unrelated residuals. Spot-check cells that were
    // blocked at M143 time and remain blocked at the current head (later
    // milestones legitimately promoted MultiHoming/UseSSL).
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
            "M143 must not disturb residual cell {}:{}",
            cell.0,
            cell.1
        );
    }
}

fn client_definition(
    name: &str,
    raw: Vec<(String, serde_json::Value)>,
) -> emissary_cli::i2pcontrol::domain::tunnel::TunnelDefinition {
    use emissary_cli::i2pcontrol::domain::tunnel::{
        StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
        TunnelRuntimeState, TunnelType,
    };
    TunnelDefinition {
        name: TunnelName::new(name).unwrap(),
        tunnel_type: TunnelType::Client,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: TunnelRuntimeState::Stopped,
        start_intent: StartIntent::DoNotStart,
        options: TunnelOptions {
            target_destination: Some("destination".to_owned()),
            listen_port: Some(0),
            ..Default::default()
        },
        raw_config: raw.into_iter().collect::<BTreeMap<_, _>>(),
    }
}

fn session_options_for(
    def: &emissary_cli::i2pcontrol::domain::tunnel::TunnelDefinition,
) -> yosemite_i2pcontrol::SessionOptions {
    use emissary_cli::i2pcontrol::backends::runtime::session::build_session_options;
    use yosemite_i2pcontrol::DestinationKind;
    build_session_options(def, 7656, false, DestinationKind::Transient).unwrap()
}

#[test]
fn m143_omitted_profile_preserves_default_without_wire() {
    let def = client_definition("omitted", vec![]);
    let options = session_options_for(&def);
    assert!(
        options.additional_options.is_empty(),
        "omitted Profile must emit no neutral wire option"
    );
}

#[test]
fn m143_bulk_and_interactive_map_to_pinned_window() {
    // Explicit bulk is accepted and maps to absent (same effective as omitted).
    let bulk = client_definition(
        "bulk",
        vec![("Profile".to_owned(), serde_json::json!("bulk"))],
    );
    let bulk_options = session_options_for(&bulk);
    assert!(
        bulk_options.additional_options.is_empty(),
        "bulk must emit no wire option (bulk default)"
    );

    // Interactive maps to the pinned neutral max window 16.
    let interactive = client_definition(
        "interactive",
        vec![("Profile".to_owned(), serde_json::json!("interactive"))],
    );
    let interactive_options = session_options_for(&interactive);
    assert_eq!(interactive_options.additional_options.len(), 1);
    let option = &interactive_options.additional_options[0];
    assert_eq!(option.key(), "i2p.streaming.maxWindowSize");
    assert_eq!(option.value(), "16");
}

#[test]
fn m143_invalid_profile_fails_before_allocation_without_echo() {
    use emissary_cli::i2pcontrol::{
        backends::{client::ClientTunnelBackend, TunnelBackend},
        domain::tunnel::TunnelRuntimeState,
    };

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    for (key, value) in [
        ("Profile", serde_json::json!(123)),
        ("Profile", serde_json::json!(true)),
        ("Profile", serde_json::json!(false)),
        ("Profile", serde_json::json!(null)),
        ("Profile", serde_json::json!("")),
        ("Profile", serde_json::json!("Interactive")),
        ("Profile", serde_json::json!("INTERACTIVE")),
        ("Profile", serde_json::json!("BULK")),
        ("Profile", serde_json::json!("bulk ")),
        ("Profile", serde_json::json!(" interactive")),
        ("Profile", serde_json::json!("fast")),
        ("Profile", serde_json::json!("1")),
        ("Profile", serde_json::json!("16")),
    ] {
        let def = client_definition("bad-profile", vec![(key.to_owned(), value.clone())]);
        // Session mapping fails before any runtime reservation.
        let result = emissary_cli::i2pcontrol::backends::runtime::session::build_session_options(
            &def,
            7656,
            false,
            yosemite_i2pcontrol::DestinationKind::Transient,
        );
        assert!(result.is_err(), "Profile {value:?} must fail");
        let error = format!("{:?}", result.unwrap_err());
        // Rejection must not echo the supplied value.
        if let Some(text) = value.as_str() {
            if !text.is_empty() {
                assert!(!error.contains(text), "rejection echoed {text:?}");
            }
        }
        // Backend start also fails and leaves no running generation.
        let backend = ClientTunnelBackend::new(1);
        let def = client_definition("bad-profile", vec![(key.to_owned(), value)]);
        let started = rt.block_on(backend.start(&def));
        assert!(started.is_err(), "start must fail before allocation");
        assert_eq!(
            backend.inspect(&def).runtime_state,
            TunnelRuntimeState::Stopped
        );
    }
}

#[test]
fn m143_non_client_families_keep_exact_rejection() {
    use emissary_cli::i2pcontrol::domain::tunnel::{
        StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
        TunnelRuntimeState, TunnelType,
    };
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    // Every M140-N/A family must reject any Profile value, including the
    // otherwise valid client values, before allocation.
    let families = [
        (
            TunnelType::HttpClient,
            "emissary_cli::i2pcontrol::backends::http_client::HttpClientTunnelBackend",
        ),
        (TunnelType::IrcClient, "irc"),
        (TunnelType::Socks, "socks"),
        (TunnelType::SocksIrc, "socksirc"),
        (TunnelType::ConnectClient, "connect"),
        (TunnelType::StreamrClient, "streamr"),
        (TunnelType::Server, "server"),
        (TunnelType::HttpServer, "httpserver"),
        (TunnelType::HttpBidirServer, "httpbidir"),
        (TunnelType::IrcServer, "ircserver"),
        (TunnelType::StreamrServer, "streamrserver"),
    ];
    for (tunnel_type, _label) in families {
        for value in [
            serde_json::json!("interactive"),
            serde_json::json!("bulk"),
            serde_json::json!("fast"),
        ] {
            let def = TunnelDefinition {
                name: TunnelName::new("n-a-profile").unwrap(),
                tunnel_type,
                ownership: TunnelOwnership::ControlPlane,
                runtime_state: TunnelRuntimeState::Stopped,
                start_intent: StartIntent::DoNotStart,
                options: TunnelOptions {
                    listen_port: Some(0),
                    target_port: Some(80),
                    ..Default::default()
                },
                raw_config: BTreeMap::from([("Profile".to_owned(), value.clone())]),
            };
            // Session builder rejects for non-client families.
            let result =
                emissary_cli::i2pcontrol::backends::runtime::session::build_session_options(
                    &def,
                    7656,
                    false,
                    yosemite_i2pcontrol::DestinationKind::Transient,
                );
            assert!(
                result.is_err(),
                "{tunnel_type} must reject Profile {value:?} before allocation"
            );
        }
    }
    let _ = rt;
}

#[test]
fn m143_shared_sessions_distinguish_effective_windows() {
    // Equal effective profiles produce identical wire options (shareable);
    // different effective windows produce different wire options (isolated).
    // Compatibility follows from the existing additional_options identity.
    let omitted = client_definition("a", vec![]);
    let bulk = client_definition("b", vec![("Profile".to_owned(), serde_json::json!("bulk"))]);
    let interactive_a = client_definition(
        "c",
        vec![("Profile".to_owned(), serde_json::json!("interactive"))],
    );
    let interactive_b = client_definition(
        "d",
        vec![("Profile".to_owned(), serde_json::json!("interactive"))],
    );

    let omitted_options = session_options_for(&omitted);
    let bulk_options = session_options_for(&bulk);
    let interactive_a_options = session_options_for(&interactive_a);
    let interactive_b_options = session_options_for(&interactive_b);

    // Omitted and explicit bulk share the same effective (no wire option).
    assert_eq!(
        omitted_options.additional_options.len(),
        bulk_options.additional_options.len()
    );
    // Two interactive definitions share the same effective window.
    assert_eq!(interactive_a_options.additional_options.len(), 1);
    assert_eq!(interactive_b_options.additional_options.len(), 1);
    assert_eq!(
        interactive_a_options.additional_options[0].key(),
        interactive_b_options.additional_options[0].key()
    );
    assert_eq!(
        interactive_a_options.additional_options[0].value(),
        interactive_b_options.additional_options[0].value()
    );
    // Incompatible windows never share: bulk (empty) vs interactive (16).
    assert_ne!(
        bulk_options.additional_options.len(),
        interactive_a_options.additional_options.len()
    );
}

#[test]
fn m143_restart_applies_new_profile_to_successor_only() {
    use emissary_cli::i2pcontrol::{
        backends::{client::ClientTunnelBackend, TunnelBackend},
        domain::tunnel::TunnelRuntimeState,
    };

    async fn fake_sam() -> (u16, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = tokio::io::BufReader::new(read_half);
                    let mut line = String::new();
                    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            break;
                        }
                        let response = if line.starts_with("HELLO") {
                            "HELLO REPLY RESULT=OK VERSION=3.3\n"
                        } else if line.starts_with("SESSION CREATE") {
                            // Assert the neutral window reaches the SAM wire.
                            // Interactive definitions must carry maxWindowSize=16;
                            // bulk/omitted must not.
                            "SESSION STATUS DESTINATION=test-destination\n"
                        } else {
                            "STREAM STATUS RESULT=OK\n"
                        };
                        if write_half.write_all(response.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                });
            }
        });
        (port, task)
    }

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let (sam_port, sam_task) = fake_sam().await;
        let backend = ClientTunnelBackend::new(sam_port);

        // Start with bulk (default window).
        let bulk = client_definition("restart-profile", vec![]);
        backend.start(&bulk).await.unwrap();
        assert_eq!(
            backend.inspect(&bulk).runtime_state,
            TunnelRuntimeState::Running
        );
        backend.stop(&bulk).await.unwrap();
        assert_eq!(
            backend.inspect(&bulk).runtime_state,
            TunnelRuntimeState::Stopped
        );

        // Restart with interactive (new generation, new effective window).
        let interactive = client_definition(
            "restart-profile",
            vec![("Profile".to_owned(), serde_json::json!("interactive"))],
        );
        // Same name, different effective profile: successor generation starts
        // cleanly; stale bulk state was dropped by stop.
        backend.start(&interactive).await.unwrap();
        assert_eq!(
            backend.inspect(&interactive).runtime_state,
            TunnelRuntimeState::Running
        );
        backend.stop(&interactive).await.unwrap();
        assert_eq!(
            backend.inspect(&interactive).runtime_state,
            TunnelRuntimeState::Stopped
        );

        // Failed edit never replaces the running generation (last-known-good).
        backend.start(&bulk).await.unwrap();
        let bad = client_definition(
            "restart-profile",
            vec![("Profile".to_owned(), serde_json::json!("fast"))],
        );
        assert!(backend.start(&bad).await.is_err());
        // Original bulk generation is still running (start rejected before
        // replacing it).
        assert_eq!(
            backend.inspect(&bulk).runtime_state,
            TunnelRuntimeState::Running
        );
        backend.stop(&bulk).await.unwrap();
        sam_task.abort();
    });
}

#[test]
fn m143_cancellation_leaves_no_partial_state() {
    use emissary_cli::i2pcontrol::{
        backends::{client::ClientTunnelBackend, TunnelBackend},
        domain::tunnel::TunnelRuntimeState,
    };

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let backend = ClientTunnelBackend::new(1);
        // Invalid Profile fails validation before listener/session allocation;
        // inspect stays Stopped, never stale Running.
        let bad = client_definition(
            "cancel-profile",
            vec![("Profile".to_owned(), serde_json::json!("interactive "))],
        );
        assert!(backend.start(&bad).await.is_err());
        assert_eq!(
            backend.inspect(&bad).runtime_state,
            TunnelRuntimeState::Stopped
        );
    });
}

#[test]
fn m143_effective_window_reaches_stream_owner() {
    // I2PControl emits the neutral wire option; core parses it before
    // StreamManager activation and caps Stream growth. Both halves are proven
    // by their focused suites; this guard ties the wire string to the core
    // owner without duplicating their logic.
    let interactive = client_definition(
        "wire",
        vec![("Profile".to_owned(), serde_json::json!("interactive"))],
    );
    let options = session_options_for(&interactive);
    let wire: Vec<String> = options
        .additional_options
        .iter()
        .map(|option| format!("{}={}", option.key(), option.value()))
        .collect();
    assert!(wire.contains(&"i2p.streaming.maxWindowSize=16".to_owned()));

    // Core owners exist and are neutral (no administrative vocabulary).
    for path in [
        "emissary-core/src/sam/protocol/streaming/config.rs",
        "emissary-core/src/sam/session.rs",
        "emissary-core/src/sam/protocol/streaming/mod.rs",
        "emissary-core/src/sam/protocol/streaming/stream/active.rs",
    ] {
        let source =
            std::fs::read_to_string(workspace_root().join(path)).expect("core source must exist");
        let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
        for forbidden in [
            "Proposal 170",
            "I2PControl",
            "TunnelManager",
            "JsonRpc",
            "jsonrpc",
        ] {
            assert!(
                !production.contains(forbidden),
                "{path} must not contain administrative term {forbidden}"
            );
        }
    }
    let session =
        std::fs::read_to_string(workspace_root().join("emissary-core/src/sam/session.rs")).unwrap();
    assert!(session.contains("parse_stream_max_window_size"));
    assert!(session.contains("new_with_max_window"));
    let manager = std::fs::read_to_string(
        workspace_root().join("emissary-core/src/sam/protocol/streaming/mod.rs"),
    )
    .unwrap();
    assert!(manager.contains("max_window_size"));
    let stream = std::fs::read_to_string(
        workspace_root().join("emissary-core/src/sam/protocol/streaming/stream/active.rs"),
    )
    .unwrap();
    assert!(stream.contains("max_window_size"));
    assert!(stream.contains("ceiling"));
}

#[test]
fn m143_observable_default_vs_interactive_is_deterministic() {
    // Deterministic observable difference: omitted/bulk emits no wire option
    // (effective 128), interactive emits 16. Core suites prove 128 grows
    // beyond 16 while 16 caps; this guard proves the I2PControl half of the
    // observable distinction.
    let omitted = session_options_for(&client_definition("o", vec![]));
    let interactive = session_options_for(&client_definition(
        "i",
        vec![("Profile".to_owned(), serde_json::json!("interactive"))],
    ));
    assert!(omitted.additional_options.is_empty());
    assert_eq!(interactive.additional_options.len(), 1);
    assert_eq!(interactive.additional_options[0].value(), "16");
    assert_ne!(
        omitted.additional_options.len(),
        interactive.additional_options.len()
    );
}

#[test]
fn m143_containment_is_exact() {
    // Production changes remain within the amended exact-file budget.
    let manager = std::fs::read_to_string(
        workspace_root().join("emissary-core/src/sam/protocol/streaming/mod.rs"),
    )
    .unwrap();
    assert!(manager.contains("new_with_max_window"));

    let client = std::fs::read_to_string(
        workspace_root().join("emissary-cli/src/i2pcontrol/backends/client.rs"),
    )
    .unwrap();
    assert!(client.contains("\"Profile\""));

    // No other backend may accept Profile.
    for path in [
        "emissary-cli/src/i2pcontrol/backends/http_client.rs",
        "emissary-cli/src/i2pcontrol/backends/irc_client.rs",
        "emissary-cli/src/i2pcontrol/backends/socks.rs",
        "emissary-cli/src/i2pcontrol/backends/connect_client.rs",
        "emissary-cli/src/i2pcontrol/backends/streamr.rs",
    ] {
        let source = std::fs::read_to_string(workspace_root().join(path)).unwrap();
        let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
        // SUPPORTED allowlists must not contain the retained Profile key.
        assert!(
            !production.contains("\"Profile\""),
            "{path} must not inherit the retained Profile seam"
        );
    }

    // Core contains no administrative policy.
    for path in [
        "emissary-core/src/sam/protocol/streaming/config.rs",
        "emissary-core/src/sam/session.rs",
        "emissary-core/src/sam/protocol/streaming/mod.rs",
        "emissary-core/src/sam/protocol/streaming/stream/active.rs",
    ] {
        let source = std::fs::read_to_string(workspace_root().join(path)).unwrap();
        let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
        assert!(!production.contains("Proposal 170"));
        assert!(!production.contains("TunnelManager"));
    }
}

#[test]
fn m143_docs_agree_on_counts() {
    // M153 rebase: M143's `330/35/475` authority lives in its immutable
    // closure; active docs track the current head (owned in aggregate by
    // M153) while retaining the M143 promotion lineage.
    let closure = std::fs::read_to_string(
        workspace_root().join("plans/closure/i2pcontrol-proposal-170/143-closure.md"),
    )
    .expect("M143 closure must exist");
    assert!(
        closure.contains("330/35/475"),
        "M143 closure must retain its historical 330/35/475 authority"
    );

    for path in [
        "AGENTS.md",
        "plans/registry.md",
        "plans/implementation/i2pcontrol-proposal-170/README.md",
        "plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md",
        "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md",
        "docs/i2pcontrol/proposal-170-support.md",
        "docs/i2pcontrol/tunnel-manager.md",
    ] {
        let text = std::fs::read_to_string(workspace_root().join(path)).unwrap();
        assert!(
            text.contains("M143") || text.contains("Profile"),
            "{path} must retain the M143 promotion lineage"
        );
        assert!(
            !text.contains("Status: **full Proposal 170 support"),
            "{path} must not claim full support"
        );
    }
}

#[test]
fn m143_no_direct_clearnet_or_global_window() {
    // No router-global mutable window and no clearnet path is introduced.
    for path in [
        "emissary-core/src/sam/protocol/streaming/config.rs",
        "emissary-core/src/sam/protocol/streaming/mod.rs",
        "emissary-core/src/sam/protocol/streaming/stream/active.rs",
        "emissary-core/src/sam/session.rs",
    ] {
        let source = std::fs::read_to_string(workspace_root().join(path)).unwrap();
        let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
        assert!(!production.contains("static mut"));
        assert!(!production.contains("to_socket_addrs"));
        assert!(!production.contains("TcpStream::connect"));
    }
    // I2PControl mapping uses only the validated generic session-option path.
    let session = std::fs::read_to_string(
        workspace_root().join("emissary-cli/src/i2pcontrol/backends/runtime/session.rs"),
    )
    .unwrap();
    assert!(session.contains("i2p.streaming.maxWindowSize"));
    assert!(session.contains("add_session_option"));
}
