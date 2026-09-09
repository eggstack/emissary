//! M145 LeaseSet reply bundling / MultiHoming completion guard.
//!
//! Planning evidence, not a runtime capability source. This test keeps the
//! two-cell promotion (`MultiHoming` × `httpserver`/`httpbidirserver`)
//! mechanically tied to the M095 machine authority and proves the pinned
//! `shouldBundleReplyInfo` contract:
//!
//! - omitted/`true` preserves the default bundle behavior (no wire option);
//! - `false` maps to neutral `shouldBundleReplyInfo = "false"` and suppresses `ExistingSession`
//!   update bundling (`NewSession` retains mandatory handshake bundling for liveness);
//! - malformed values fail before allocation with no echo;
//! - non-target families keep exact rejection;
//! - shared sessions distinguish effective policies.

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
    let raw = std::fs::read_to_string(&path).expect("M145 planning file must exist");
    raw.parse().expect("M145 planning file must be valid TOML")
}

fn string_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("M145 record is missing string field {key}"))
}

#[test]
fn m145_matrix_promotes_exactly_two_multihoming_cells() {
    let matrix = planning_toml("095-full-support-matrix.toml");
    let declared = matrix["current_matrix_counts"].as_table().expect("counts");
    assert_eq!(declared["total"].as_integer(), Some(840));
    assert_eq!(declared["apply"].as_integer(), Some(336));
    assert_eq!(declared["blocked_primitive"].as_integer(), Some(29));
    assert_eq!(declared["not_applicable"].as_integer(), Some(475));

    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("canonical tunnel types");
    assert_eq!(tunnel_types.len(), 12);
    let mut counts = [0usize; 3];
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
                    counts[0] += 1;
                    applied.insert((option.to_owned(), family));
                }
                "blocked_primitive" => {
                    counts[1] += 1;
                    blocked.insert((option.to_owned(), family));
                }
                "not_applicable" => counts[2] += 1,
                other => panic!("unexpected cell disposition {other}"),
            }
        }
    }
    assert_eq!(counts, [336, 29, 475]);
    for cell in [
        ("MultiHoming", "httpserver"),
        ("MultiHoming", "httpbidirserver"),
    ] {
        let (option, family) = (&cell.0.to_owned(), &cell.1.to_owned());
        assert!(
            applied.contains(&(option.clone(), family.clone())),
            "M145 cell {option}:{family} must be apply"
        );
        assert!(
            !blocked.contains(&(option.clone(), family.clone())),
            "M145 cell {option}:{family} must not remain blocked"
        );
    }

    let options = matrix["tunnel_manager"]["options"].as_array().unwrap();
    let row = options
        .iter()
        .find(|row| string_field(row, "canonical_key") == "MultiHoming")
        .expect("MultiHoming row must exist");
    assert_eq!(
        string_field(row, "completion_owner"),
        "M145",
        "MultiHoming owner"
    );
    assert_eq!(
        string_field(row, "current_or_planned_disposition"),
        "apply_or_not_applicable",
        "MultiHoming disposition"
    );
    assert!(
        row.get("blocking_milestone").is_none(),
        "MultiHoming milestone"
    );
    assert!(
        row.get("blocked_primitive").is_none(),
        "MultiHoming primitive"
    );
    let cells = row["cells"].as_array().unwrap();
    assert_eq!(
        cells[8].as_str(),
        Some("apply"),
        "MultiHoming httpserver cell"
    );
    assert_eq!(
        cells[9].as_str(),
        Some("apply"),
        "MultiHoming httpbidir cell"
    );
    for (index, cell) in cells.iter().enumerate() {
        if index != 8 && index != 9 {
            assert_eq!(
                cell.as_str(),
                Some("not_applicable"),
                "MultiHoming cell {index} must stay N/A"
            );
        }
    }
}

#[test]
fn m145_residual_inventory_subtracts_two_cells() {
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
    assert_eq!(current_blocked.len(), 29);
    for cell in [
        ("MultiHoming", "httpserver"),
        ("MultiHoming", "httpbidirserver"),
    ] {
        assert!(
            !current_blocked.contains(&(cell.0.to_owned(), cell.1.to_owned())),
            "M145 cell {}:{} must not remain blocked",
            cell.0,
            cell.1
        );
    }
    for cell in [
        ("SigType", "client"),
        ("UseOutproxyPlugin", "httpclient"),
        ("EncryptLeaseSet", "httpserver"),
        ("UseSSL", "httpclient"),
    ] {
        // UseSSL:httpclient is apply (not blocked); assert it stays apply.
        if cell == ("UseSSL", "httpclient") {
            assert!(
                !current_blocked.contains(&(cell.0.to_owned(), cell.1.to_owned())),
                "M145 must not disturb promoted cell {}:{}",
                cell.0,
                cell.1
            );
            continue;
        }
        assert!(
            current_blocked.contains(&(cell.0.to_owned(), cell.1.to_owned())),
            "M145 must not disturb residual cell {}:{}",
            cell.0,
            cell.1
        );
    }
    // Promoted Profile:client and UseSSL cells stay apply (not blocked).
    assert!(!current_blocked.contains(&("Profile".to_owned(), "client".to_owned())));
}

fn server_definition(
    tunnel_type: emissary_cli::i2pcontrol::domain::tunnel::TunnelType,
    raw: Vec<(String, serde_json::Value)>,
) -> emissary_cli::i2pcontrol::domain::tunnel::TunnelDefinition {
    use emissary_cli::i2pcontrol::domain::tunnel::{
        StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
        TunnelRuntimeState,
    };
    let mut options = TunnelOptions::default();
    match tunnel_type {
        emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpServer => {
            options.target_port = Some(8080);
        }
        emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpBidirServer => {
            options.target_port = Some(8080);
            options.listen_port = Some(0);
        }
        _ => {}
    }
    TunnelDefinition {
        name: TunnelName::new("m145").unwrap(),
        tunnel_type,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: TunnelRuntimeState::Stopped,
        start_intent: StartIntent::DoNotStart,
        options,
        raw_config: raw.into_iter().collect::<BTreeMap<_, _>>(),
    }
}

fn session_options_for(
    def: &emissary_cli::i2pcontrol::domain::tunnel::TunnelDefinition,
) -> yosemite_i2pcontrol::SessionOptions {
    use emissary_cli::i2pcontrol::backends::runtime::session::build_session_options;
    use yosemite_i2pcontrol::DestinationKind;
    build_session_options(def, 7656, true, DestinationKind::Transient).unwrap()
}

#[test]
fn m145_omitted_preserves_default_without_wire() {
    use emissary_cli::i2pcontrol::domain::tunnel::TunnelType;
    for tunnel_type in [TunnelType::HttpServer, TunnelType::HttpBidirServer] {
        let def = server_definition(tunnel_type, vec![]);
        let options = session_options_for(&def);
        assert!(
            options.additional_options.is_empty(),
            "omitted MultiHoming must emit no neutral wire option for {tunnel_type}"
        );
    }
}

#[test]
fn m145_true_and_false_map_to_pinned_policy() {
    use emissary_cli::i2pcontrol::domain::tunnel::TunnelType;
    // Explicit true is accepted and maps to absent (same effective as omitted).
    let enabled = server_definition(
        TunnelType::HttpServer,
        vec![("MultiHoming".to_owned(), serde_json::json!(true))],
    );
    let enabled_options = session_options_for(&enabled);
    assert!(
        enabled_options.additional_options.is_empty(),
        "explicit true must emit no wire option (bundle default)"
    );

    // False maps to the pinned neutral shouldBundleReplyInfo=false.
    let disabled = server_definition(
        TunnelType::HttpServer,
        vec![("MultiHoming".to_owned(), serde_json::json!(false))],
    );
    let disabled_options = session_options_for(&disabled);
    assert_eq!(disabled_options.additional_options.len(), 1);
    let option = &disabled_options.additional_options[0];
    assert_eq!(option.key(), "shouldBundleReplyInfo");
    assert_eq!(option.value(), "false");

    // Bidir server half carries the same mapping.
    let bidir_disabled = server_definition(
        TunnelType::HttpBidirServer,
        vec![("MultiHoming".to_owned(), serde_json::json!(false))],
    );
    let bidir_options = session_options_for(&bidir_disabled);
    assert_eq!(bidir_options.additional_options.len(), 1);
    assert_eq!(
        bidir_options.additional_options[0].key(),
        "shouldBundleReplyInfo"
    );
    assert_eq!(bidir_options.additional_options[0].value(), "false");
}

#[test]
fn m145_invalid_fails_before_allocation_without_echo() {
    use emissary_cli::i2pcontrol::{
        backends::{
            http_bidir::HttpBidirServerTunnelBackend, http_server::HttpServerTunnelBackend,
            TunnelBackend,
        },
        domain::tunnel::TunnelType,
    };
    for bad in [
        serde_json::json!("yes"),
        serde_json::json!("true"),
        serde_json::json!(1),
        serde_json::json!(0),
        serde_json::json!(null),
    ] {
        for tunnel_type in [TunnelType::HttpServer, TunnelType::HttpBidirServer] {
            let def = server_definition(tunnel_type, vec![("MultiHoming".to_owned(), bad.clone())]);
            // Backend preflight must fail naming only MultiHoming.
            let result = match tunnel_type {
                TunnelType::HttpServer => {
                    let root = tempfile::tempdir().unwrap();
                    let backend = HttpServerTunnelBackend::new(
                        7656,
                        emissary_cli::i2pcontrol::server_secret_store::ServerDestinationStore::new(
                            root.path(),
                        ),
                    );
                    backend.validate_start(&def)
                }
                _ => {
                    let root = tempfile::tempdir().unwrap();
                    let backend = HttpBidirServerTunnelBackend::new(
                        7656,
                        emissary_cli::i2pcontrol::server_secret_store::ServerDestinationStore::new(
                            root.path(),
                        ),
                        None,
                    );
                    backend.validate_start(&def)
                }
            };
            match result {
                Err(emissary_cli::i2pcontrol::backends::BackendError::UnsupportedOption {
                    option,
                    ..
                }) => assert_eq!(option, "MultiHoming"),
                Err(emissary_cli::i2pcontrol::backends::BackendError::Internal { message }) => {
                    assert!(
                        message.contains("MultiHoming"),
                        "malformed MultiHoming must name the option"
                    );
                    assert!(
                        !message.contains("yes") && !message.contains("true"),
                        "rejection must not echo values"
                    );
                }
                Ok(()) => panic!("malformed MultiHoming {bad:?} must fail for {tunnel_type}"),
                Err(other) => panic!("unexpected error for {bad:?}: {other:?}"),
            }
        }
    }
}

#[test]
fn m145_non_server_families_keep_exact_rejection() {
    use emissary_cli::i2pcontrol::{
        backends::runtime::session::build_session_options,
        domain::tunnel::{
            StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
            TunnelRuntimeState, TunnelType,
        },
    };
    use yosemite_i2pcontrol::DestinationKind;
    let rejected = [
        TunnelType::Client,
        TunnelType::HttpClient,
        TunnelType::IrcClient,
        TunnelType::Socks,
        TunnelType::SocksIrc,
        TunnelType::ConnectClient,
        TunnelType::StreamrClient,
        TunnelType::Server,
        TunnelType::IrcServer,
        TunnelType::StreamrServer,
    ];
    assert_eq!(rejected.len(), 10);
    for tunnel_type in rejected {
        let def = TunnelDefinition {
            name: TunnelName::new("m145-nontarget").unwrap(),
            tunnel_type,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions::default(),
            raw_config: [("MultiHoming".to_owned(), serde_json::json!(true))].into_iter().collect(),
        };
        match build_session_options(&def, 7656, true, DestinationKind::Transient) {
            Err(emissary_cli::i2pcontrol::backends::BackendError::UnsupportedOption {
                option,
                ..
            }) => assert_eq!(option, "MultiHoming"),
            Ok(_) => panic!("MultiHoming must be rejected for N/A family {tunnel_type}"),
            Err(other) => panic!("unexpected error for {tunnel_type}: {other:?}"),
        }
    }
}

#[test]
fn m145_shared_sessions_distinguish_effective_policies() {
    // Omitted and explicit true share one effective policy (both absent wire);
    // false carries a distinct wire option and never shares with them.
    let omitted = server_definition(
        emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpServer,
        vec![],
    );
    let enabled = server_definition(
        emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpServer,
        vec![("MultiHoming".to_owned(), serde_json::json!(true))],
    );
    let disabled = server_definition(
        emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpServer,
        vec![("MultiHoming".to_owned(), serde_json::json!(false))],
    );
    let omitted_options = session_options_for(&omitted);
    let enabled_options = session_options_for(&enabled);
    let disabled_options = session_options_for(&disabled);
    assert!(omitted_options.additional_options.is_empty());
    assert!(enabled_options.additional_options.is_empty());
    assert_eq!(disabled_options.additional_options.len(), 1);
    // Wire identity: omitted and true are identical (shareable), false differs.
    let omitted_wire: Vec<(String, String)> = omitted_options
        .additional_options
        .iter()
        .map(|o| (o.key().to_owned(), o.value().to_owned()))
        .collect();
    let enabled_wire: Vec<(String, String)> = enabled_options
        .additional_options
        .iter()
        .map(|o| (o.key().to_owned(), o.value().to_owned()))
        .collect();
    let disabled_wire: Vec<(String, String)> = disabled_options
        .additional_options
        .iter()
        .map(|o| (o.key().to_owned(), o.value().to_owned()))
        .collect();
    assert_eq!(omitted_wire, enabled_wire);
    assert_ne!(omitted_wire, disabled_wire);
}

#[test]
fn m145_wire_reaches_neutral_owner_without_proposal_vocabulary() {
    // The neutral core owner parses the standard session property; the
    // I2PControl layer is the only place Proposal vocabulary appears.
    let disabled = server_definition(
        emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpServer,
        vec![("MultiHoming".to_owned(), serde_json::json!(false))],
    );
    let options = session_options_for(&disabled);
    let wire: Vec<String> = options
        .additional_options
        .iter()
        .map(|o| format!("{}={}", o.key(), o.value()))
        .collect();
    assert_eq!(wire, vec!["shouldBundleReplyInfo=false".to_owned()]);
    // No Proposal/I2PControl terms on the wire.
    for entry in &wire {
        assert!(!entry.contains("MultiHoming"));
        assert!(!entry.contains("Proposal"));
        assert!(!entry.contains("I2PControl"));
    }
}

#[test]
fn m145_exact_containment_holds_for_touched_owners() {
    // Static guards: core stays Proposal-free; I2PControl owns the mapping.
    let core_path = workspace_root().join("emissary-core/src/destination/session/mod.rs");
    let core = std::fs::read_to_string(&core_path).expect("core owner must exist");
    for term in [
        "Proposal 170",
        "I2PControl",
        "TunnelManager",
        "JsonRpc",
        "MultiHoming",
    ] {
        assert!(
            !core.contains(term),
            "neutral core owner must not contain Proposal term {term:?}"
        );
    }
    assert!(
        core.contains("shouldBundleReplyInfo"),
        "neutral owner must consume the standard session property"
    );
    assert!(
        core.contains("bundle_reply_lease_set"),
        "neutral owner must expose the Proposal-free policy"
    );
    let session_path =
        workspace_root().join("emissary-cli/src/i2pcontrol/backends/runtime/session.rs");
    let session = std::fs::read_to_string(&session_path).expect("session owner must exist");
    assert!(session.contains("MultiHoming"));
    assert!(session.contains("shouldBundleReplyInfo"));
}

#[test]
fn m145_docs_agree_on_partial_support_and_counts() {
    let support =
        std::fs::read_to_string(workspace_root().join("docs/i2pcontrol/proposal-170-support.md"))
            .expect("support doc must exist");
    assert!(
        support.contains("336") || support.contains("partial"),
        "support doc must reflect M145 counts/partial support"
    );
}
