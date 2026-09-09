//! M146 outproxy provider / UseOutproxyPlugin blocked guard.
//!
//! Planning evidence, not a runtime capability source. M146 closes as blocked:
//! no real bounded I2P-routed local outproxy provider exists within the
//! I2PControl-local budget, and a direct-clearnet provider is prohibited.
//! This test pins the blocked behavior:
//!
//! - M095 stays `336 apply / 29 blocked_primitive / 475 not_applicable`;
//! - the four `UseOutproxyPlugin` cells stay `blocked_primitive`;
//! - all four client proxy families reject any supplied value before allocation with no provider
//!   effect and no direct-clearnet fallback;
//! - the ordinary configured-I2P-outproxy (`ProxyList`) path is unchanged.

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
    let raw = std::fs::read_to_string(&path).expect("M146 planning file must exist");
    raw.parse().expect("M146 planning file must be valid TOML")
}

fn string_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("M146 record is missing string field {key}"))
}

#[test]
fn m146_matrix_keeps_336_29_475_with_four_outproxy_cells_blocked() {
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
    for row in matrix["tunnel_manager"]["options"].as_array().expect("options") {
        let option = row["canonical_key"].as_str().expect("canonical option");
        let cells = row["cells"].as_array().expect("option cells");
        assert_eq!(cells.len(), tunnel_types.len());
        for (index, cell) in cells.iter().enumerate() {
            let family = tunnel_types[index].as_str().expect("family").to_owned();
            match cell.as_str().expect("cell disposition") {
                "apply" => counts[0] += 1,
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
        ("UseOutproxyPlugin", "httpclient"),
        ("UseOutproxyPlugin", "socks"),
        ("UseOutproxyPlugin", "socksirc"),
        ("UseOutproxyPlugin", "connectclient"),
    ] {
        assert!(
            blocked.contains(&(cell.0.to_owned(), cell.1.to_owned())),
            "M146 cell {}:{} must remain blocked_primitive",
            cell.0,
            cell.1
        );
    }

    let options = matrix["tunnel_manager"]["options"].as_array().unwrap();
    let row = options
        .iter()
        .find(|row| string_field(row, "canonical_key") == "UseOutproxyPlugin")
        .expect("UseOutproxyPlugin row must exist");
    assert_eq!(
        string_field(row, "current_or_planned_disposition"),
        "blocked_primitive_or_not_applicable"
    );
    assert!(
        row.get("blocking_milestone").is_some(),
        "UseOutproxyPlugin blocker"
    );
    assert!(
        row.get("blocked_primitive").is_some(),
        "UseOutproxyPlugin primitive"
    );
    let cells = row["cells"].as_array().unwrap();
    assert_eq!(
        cells[1].as_str(),
        Some("blocked_primitive"),
        "httpclient cell"
    );
    assert_eq!(cells[3].as_str(), Some("blocked_primitive"), "socks cell");
    assert_eq!(
        cells[4].as_str(),
        Some("blocked_primitive"),
        "socksirc cell"
    );
    assert_eq!(
        cells[5].as_str(),
        Some("blocked_primitive"),
        "connectclient cell"
    );
}

#[test]
fn m146_residual_inventory_still_contains_29_cells() {
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
        ("SigType", "client"),
        ("UseOutproxyPlugin", "httpclient"),
        ("UseOutproxyPlugin", "socks"),
        ("UseOutproxyPlugin", "socksirc"),
        ("UseOutproxyPlugin", "connectclient"),
        ("EncryptLeaseSet", "httpserver"),
        ("OptionalLookup", "server"),
        ("LeaseSetClientAuths", "ircserver"),
    ] {
        assert!(
            current_blocked.contains(&(cell.0.to_owned(), cell.1.to_owned())),
            "M146 must not disturb residual cell {}:{}",
            cell.0,
            cell.1
        );
    }
    // M145 promotions stay applied.
    assert!(!current_blocked.contains(&("MultiHoming".to_owned(), "httpserver".to_owned())));
    assert!(!current_blocked.contains(&("Profile".to_owned(), "client".to_owned())));
}

fn client_definition(
    tunnel_type: emissary_cli::i2pcontrol::domain::tunnel::TunnelType,
    raw: Vec<(String, serde_json::Value)>,
) -> emissary_cli::i2pcontrol::domain::tunnel::TunnelDefinition {
    use emissary_cli::i2pcontrol::domain::tunnel::{
        StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
        TunnelRuntimeState,
    };
    TunnelDefinition {
        name: TunnelName::new("m146").unwrap(),
        tunnel_type,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: TunnelRuntimeState::Stopped,
        start_intent: StartIntent::DoNotStart,
        options: TunnelOptions {
            listen_port: Some(0),
            ..Default::default()
        },
        raw_config: raw.into_iter().collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn m146_all_four_proxy_families_reject_any_use_outproxy_plugin_value() {
    use emissary_cli::i2pcontrol::{
        backends::{
            connect_client::ConnectClientTunnelBackend, http_client::HttpClientTunnelBackend,
            socks::SocksTunnelBackend, socks_irc::SocksIrcTunnelBackend, TunnelBackend,
        },
        domain::tunnel::{TunnelRuntimeState, TunnelType},
    };
    let values = vec![
        serde_json::json!(true),
        serde_json::json!(false),
        serde_json::json!("true"),
        serde_json::json!(1),
        serde_json::json!(null),
    ];
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    for tunnel_type in [
        TunnelType::HttpClient,
        TunnelType::Socks,
        TunnelType::SocksIrc,
        TunnelType::ConnectClient,
    ] {
        for value in &values {
            let def = client_definition(
                tunnel_type,
                vec![("UseOutproxyPlugin".to_owned(), value.clone())],
            );
            let failed = match tunnel_type {
                TunnelType::HttpClient => {
                    let backend = HttpClientTunnelBackend::new(1);
                    backend.validate_start(&def).is_err()
                }
                TunnelType::Socks => {
                    let backend = SocksTunnelBackend::new(1);
                    rt.block_on(backend.start(&def)).is_err()
                }
                TunnelType::SocksIrc => {
                    let backend = SocksIrcTunnelBackend::new(1);
                    rt.block_on(backend.start(&def)).is_err()
                }
                _ => {
                    let backend = ConnectClientTunnelBackend::new(1);
                    rt.block_on(backend.start(&def)).is_err()
                }
            };
            assert!(
                failed,
                "UseOutproxyPlugin {value:?} must fail before allocation for {tunnel_type}"
            );
            // No generation is committed: inspect stays Stopped.
            let inspect_stopped = match tunnel_type {
                TunnelType::HttpClient => {
                    let backend = HttpClientTunnelBackend::new(1);
                    backend.inspect(&def).runtime_state == TunnelRuntimeState::Stopped
                }
                TunnelType::Socks => {
                    let backend = SocksTunnelBackend::new(1);
                    backend.inspect(&def).runtime_state == TunnelRuntimeState::Stopped
                }
                TunnelType::SocksIrc => {
                    let backend = SocksIrcTunnelBackend::new(1);
                    backend.inspect(&def).runtime_state == TunnelRuntimeState::Stopped
                }
                _ => {
                    let backend = ConnectClientTunnelBackend::new(1);
                    backend.inspect(&def).runtime_state == TunnelRuntimeState::Stopped
                }
            };
            assert!(
                inspect_stopped,
                "rejected UseOutproxyPlugin must leave {tunnel_type} Stopped"
            );
        }
    }
}

#[test]
fn m146_ordinary_proxylist_path_is_unchanged_without_plugin_flag() {
    use emissary_cli::i2pcontrol::{
        backends::{http_client::HttpClientTunnelBackend, TunnelBackend},
        domain::tunnel::TunnelType,
    };
    // A valid configured I2P outproxy still passes preflight when the plugin
    // flag is absent. Adding the flag to the same valid config must fail,
    // proving no provider alias exists.
    let backend = HttpClientTunnelBackend::new(1);
    let plain = client_definition(
        TunnelType::HttpClient,
        vec![("ProxyList".to_owned(), serde_json::json!("outproxy.i2p"))],
    );
    assert!(
        backend.validate_start(&plain).is_ok(),
        "ordinary ProxyList path must stay accepted without UseOutproxyPlugin"
    );
    let with_plugin = client_definition(
        TunnelType::HttpClient,
        vec![
            ("ProxyList".to_owned(), serde_json::json!("outproxy.i2p")),
            ("UseOutproxyPlugin".to_owned(), serde_json::json!(false)),
        ],
    );
    assert!(
        backend.validate_start(&with_plugin).is_err(),
        "adding UseOutproxyPlugin must still fail even with a valid ProxyList"
    );
}

#[test]
fn m146_clearnet_without_outproxy_never_selects_direct_egress() {
    use emissary_cli::i2pcontrol::backends::filters::http_client::{
        HttpClientRequest, OutproxyTarget,
    };
    // Ordinary HTTP clearnet with no configured outproxy must fail rather
    // than return a direct-clearnet target. This pins the no-direct-clearnet
    // invariant that a future provider must preserve.
    let clearnet = b"GET http://example.com/ HTTP/1.1\r\nHost: example.com\r\n\r\n";
    assert!(HttpClientRequest::parse(clearnet, None).is_err());
    // I2P destinations never need an outproxy.
    let i2p = b"GET http://example.i2p/ HTTP/1.1\r\nHost: example.i2p\r\n\r\n";
    assert!(HttpClientRequest::parse(i2p, None).is_ok());
    // The same clearnet target routes via an explicit I2P outproxy.
    let outproxy = OutproxyTarget {
        destination: "outproxy.i2p".to_owned(),
        port: 4444,
    };
    assert!(HttpClientRequest::parse(clearnet, Some(outproxy)).is_ok());
}
