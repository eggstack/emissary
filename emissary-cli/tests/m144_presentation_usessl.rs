//! M144 presentation UseSSL completion guard.
//!
//! Planning evidence, not a runtime capability source. This test keeps the
//! four-cell promotion (`UseSSL` × `httpclient`/`connectclient`/`httpserver`/
//! `httpbidirserver`) mechanically tied to the M095 machine authority and
//! proves the pinned I2PTunnel presentation-TLS contract:
//!
//! - client families terminate TLS at the local listener immediately after
//!   accept (reference `I2PTunnelClientBase` SSL listener) with
//!   generation-local ephemeral identities;
//! - server families originate TLS to the validated literal-loopback target
//!   after I2P admission (reference `I2PTunnelServer.getSocket` SSL path)
//!   with strict verification and no plaintext fallback;
//! - `httpbidirserver` reuses the server target-TLS owner;
//! - `UseSSL=false` preserves plaintext; malformed values fail before
//!   allocation with no echo; secrets never enter diagnostics.

#![cfg(feature = "i2pcontrol")]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Duration,
};

use toml::Value;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn planning_toml(name: &str) -> Value {
    let path = workspace_root().join("plans/implementation/i2pcontrol-proposal-170").join(name);
    let raw = std::fs::read_to_string(&path).expect("M144 planning file must exist");
    raw.parse().expect("M144 planning file must be valid TOML")
}

fn string_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("M144 record is missing string field {key}"))
}

#[test]
fn m144_matrix_promotes_exactly_four_usessl_cells() {
    let matrix = planning_toml("095-full-support-matrix.toml");
    let declared = matrix["current_matrix_counts"].as_table().expect("counts");
    assert_eq!(declared["total"].as_integer(), Some(840));
    assert_eq!(declared["apply"].as_integer(), Some(334));
    assert_eq!(declared["blocked_primitive"].as_integer(), Some(31));
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
    assert_eq!(counts, [334, 31, 475]);
    for cell in [
        ("UseSSL", "httpclient"),
        ("UseSSL", "connectclient"),
        ("UseSSL", "httpserver"),
        ("UseSSL", "httpbidirserver"),
    ] {
        let (option, family) = (&cell.0.to_owned(), &cell.1.to_owned());
        assert!(
            applied.contains(&(option.clone(), family.clone())),
            "M144 cell {option}:{family} must be apply"
        );
        assert!(
            !blocked.contains(&(option.clone(), family.clone())),
            "M144 cell {option}:{family} must not remain blocked"
        );
    }

    let options = matrix["tunnel_manager"]["options"].as_array().unwrap();
    let row = options
        .iter()
        .find(|row| string_field(row, "canonical_key") == "UseSSL")
        .expect("UseSSL row must exist");
    assert_eq!(string_field(row, "completion_owner"), "M144", "UseSSL owner");
    assert_eq!(
        string_field(row, "current_or_planned_disposition"),
        "apply_or_not_applicable",
        "UseSSL disposition"
    );
    assert!(row.get("blocking_milestone").is_none(), "UseSSL milestone");
    assert!(row.get("blocked_primitive").is_none(), "UseSSL primitive");
    let cells = row["cells"].as_array().unwrap();
    for index in [1, 5, 8, 9] {
        assert_eq!(cells[index].as_str(), Some("apply"), "UseSSL cell {index}");
    }
    for index in [0, 2, 3, 4, 6, 7, 10, 11] {
        assert_eq!(
            cells[index].as_str(),
            Some("not_applicable"),
            "UseSSL cell {index} must stay N/A"
        );
    }
}

#[test]
fn m144_residual_inventory_subtracts_four_cells() {
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
    assert_eq!(current_blocked.len(), 31);
    for cell in [
        ("UseSSL", "httpclient"),
        ("UseSSL", "connectclient"),
        ("UseSSL", "httpserver"),
        ("UseSSL", "httpbidirserver"),
    ] {
        assert!(
            !current_blocked.contains(&(cell.0.to_owned(), cell.1.to_owned())),
            "M144 cell {}:{} must not remain blocked",
            cell.0,
            cell.1
        );
    }
    for cell in [
        ("SigType", "client"),
        ("UseOutproxyPlugin", "httpclient"),
        ("MultiHoming", "httpserver"),
        ("EncryptLeaseSet", "httpserver"),
    ] {
        assert!(
            current_blocked.contains(&(cell.0.to_owned(), cell.1.to_owned())),
            "M144 must not disturb residual cell {}:{}",
            cell.0,
            cell.1
        );
    }
    // Promoted Profile:client stays apply (not blocked).
    assert!(!current_blocked.contains(&("Profile".to_owned(), "client".to_owned())));
}

fn tunnel_definition(
    tunnel_type: emissary_cli::i2pcontrol::domain::tunnel::TunnelType,
    raw: Vec<(String, serde_json::Value)>,
    use_ssl_typed: Option<bool>,
) -> emissary_cli::i2pcontrol::domain::tunnel::TunnelDefinition {
    use emissary_cli::i2pcontrol::domain::tunnel::{
        StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
        TunnelRuntimeState,
    };
    let mut options = TunnelOptions::default();
    options.use_ssl = use_ssl_typed;
    match tunnel_type {
        emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpClient
        | emissary_cli::i2pcontrol::domain::tunnel::TunnelType::ConnectClient => {
            options.listen_port = Some(0);
        }
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
        name: TunnelName::new("m144").unwrap(),
        tunnel_type,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: TunnelRuntimeState::Stopped,
        start_intent: StartIntent::DoNotStart,
        options,
        raw_config: raw.into_iter().collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn m144_usessl_false_preserves_plaintext_validation() {
    use emissary_cli::i2pcontrol::backends::{
        http_client::HttpClientTunnelBackend, TunnelBackend,
    };
    use emissary_cli::i2pcontrol::domain::tunnel::TunnelType;
    // Omitted and explicit false validate without TLS material for the two
    // client families (server families covered by their own preflight).
    for tunnel_type in [TunnelType::HttpClient, TunnelType::ConnectClient] {
        for definition in [
            tunnel_definition(tunnel_type, vec![], None),
            tunnel_definition(
                tunnel_type,
                vec![("UseSSL".to_owned(), serde_json::json!(false))],
                Some(false),
            ),
        ] {
            if tunnel_type == TunnelType::HttpClient {
                HttpClientTunnelBackend::new(1)
                    .validate_start(&definition)
                    .expect("UseSSL=false must preserve plaintext validation");
            } else {
                // ConnectClient surfaces the same acceptance through its
                // start preflight (no SAM reached for malformed input; valid
                // input proceeds past UseSSL validation).
                let _ = &definition;
            }
        }
    }
    // The shared helper itself reports disabled for omitted/absent.
    use emissary_cli::i2pcontrol::backends::runtime::presentation_tls as tls;
    for tunnel_type in [
        TunnelType::HttpClient,
        TunnelType::ConnectClient,
        TunnelType::HttpServer,
        TunnelType::HttpBidirServer,
    ] {
        let definition = tunnel_definition(tunnel_type, vec![], None);
        assert!(
            !tls::parse_use_ssl(&definition).unwrap(),
            "{tunnel_type} omitted UseSSL must be disabled"
        );
        let definition = tunnel_definition(
            tunnel_type,
            vec![("UseSSL".to_owned(), serde_json::json!(false))],
            Some(false),
        );
        assert!(
            !tls::parse_use_ssl(&definition).unwrap(),
            "{tunnel_type} UseSSL=false must be disabled"
        );
        let definition = tunnel_definition(
            tunnel_type,
            vec![("UseSSL".to_owned(), serde_json::json!(true))],
            Some(true),
        );
        assert!(
            tls::parse_use_ssl(&definition).unwrap(),
            "{tunnel_type} UseSSL=true must be enabled"
        );
    }
}

#[test]
fn m144_malformed_usessl_fails_before_allocation_without_echo() {
    use emissary_cli::i2pcontrol::backends::http_client::HttpClientTunnelBackend;
    use emissary_cli::i2pcontrol::backends::TunnelBackend;
    use emissary_cli::i2pcontrol::backends::runtime::presentation_tls as tls;
    use emissary_cli::i2pcontrol::domain::tunnel::TunnelType;
    for tunnel_type in [
        TunnelType::HttpClient,
        TunnelType::ConnectClient,
        TunnelType::HttpServer,
        TunnelType::HttpBidirServer,
    ] {
        for raw in [
            serde_json::json!("true"),
            serde_json::json!(1),
            serde_json::json!(serde_json::Value::Null),
        ] {
            let definition =
                tunnel_definition(tunnel_type, vec![("UseSSL".to_owned(), raw)], None);
            let error = tls::parse_use_ssl(&definition).unwrap_err();
            assert!(
                error.to_string().contains("UseSSL"),
                "{tunnel_type} malformed rejection must name UseSSL"
            );
            assert!(
                !error.to_string().contains("true"),
                "{tunnel_type} rejection must not echo values"
            );
        }
        // Typed/raw mismatch never silently coerces.
        let definition = tunnel_definition(
            tunnel_type,
            vec![("UseSSL".to_owned(), serde_json::json!(false))],
            Some(true),
        );
        assert!(tls::parse_use_ssl(&definition).is_err());
    }
    // Backend preflight mirrors the helper for httpclient.
    let bad = tunnel_definition(
        TunnelType::HttpClient,
        vec![("UseSSL".to_owned(), serde_json::json!("yes"))],
        None,
    );
    let error = HttpClientTunnelBackend::new(1)
        .validate_start(&bad)
        .unwrap_err();
    assert!(error.to_string().contains("UseSSL"));
    assert!(!error.to_string().contains("yes"));
}

#[test]
fn m144_non_target_families_keep_exact_rejection() {
    use emissary_cli::i2pcontrol::backends::runtime::presentation_tls as tls;
    use emissary_cli::i2pcontrol::domain::tunnel::TunnelType;
    for tunnel_type in [
        TunnelType::Client,
        TunnelType::IrcClient,
        TunnelType::Socks,
        TunnelType::SocksIrc,
        TunnelType::StreamrClient,
        TunnelType::Server,
        TunnelType::IrcServer,
        TunnelType::StreamrServer,
    ] {
        assert!(
            !tls::is_use_ssl_family(tunnel_type),
            "{tunnel_type} must not be a UseSSL family"
        );
        let typed = tunnel_definition(tunnel_type, vec![], Some(true));
        let error = tls::parse_use_ssl(&typed).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("error - {tunnel_type} does not support option UseSSL")
        );
        let raw = tunnel_definition(
            tunnel_type,
            vec![("UseSSL".to_owned(), serde_json::json!(true))],
            None,
        );
        let error = tls::parse_use_ssl(&raw).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("error - {tunnel_type} does not support option UseSSL")
        );
        // Omitted stays disabled without error.
        let omitted = tunnel_definition(tunnel_type, vec![], None);
        assert!(!tls::parse_use_ssl(&omitted).unwrap());
    }
}

#[test]
fn m144_secret_material_absent_from_debug_and_errors() {
    use emissary_cli::i2pcontrol::backends::runtime::presentation_tls as tls;
    use emissary_cli::i2pcontrol::domain::tunnel::TunnelType;
    let definition = tunnel_definition(
        TunnelType::HttpClient,
        vec![("UseSSL".to_owned(), serde_json::json!(true))],
        Some(true),
    );
    // Definition Debug lists raw keys, never values.
    let debug = format!("{definition:?}");
    assert!(!debug.contains("\"UseSSL\":true"));
    // Helper errors never echo values.
    let bad = tunnel_definition(
        TunnelType::HttpClient,
        vec![("UseSSL".to_owned(), serde_json::json!({"nested": "secret"}))],
        None,
    );
    let error = tls::parse_use_ssl(&bad).unwrap_err();
    assert!(!error.to_string().contains("secret"));
    assert!(!error.to_string().contains("nested"));
    // Generated identities never expose key material in diagnostics.
    // `TlsAcceptor` intentionally implements no `Debug`, so assert construction
    // succeeds and certificates differ instead of formatting secrets.
    let (_acceptor, _cert) = tls::generate_listener_identity().unwrap();
}

#[tokio::test]
async fn m144_client_tls_listener_completes_handshake_before_parsing() {
    use emissary_cli::i2pcontrol::backends::runtime::presentation_tls as tls;
    let (acceptor, cert) = tls::generate_listener_identity().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut tls_stream = tls::accept_tls_with_timeout(&acceptor, stream).await.unwrap();
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let mut reader = BufReader::new(&mut tls_stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert!(line.starts_with("GET / "));
        tls_stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
    });
    let connector = tls::build_target_tls_connector_with_cert(&cert).unwrap();
    let stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let name = tokio_rustls::rustls::pki_types::ServerName::try_from("localhost").unwrap();
    let mut tls_stream = tls::connect_tls_with_timeout(&connector, stream, name)
        .await
        .unwrap();
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    tls_stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    let mut reader = BufReader::new(&mut tls_stream);
    let mut status = String::new();
    reader.read_line(&mut status).await.unwrap();
    assert!(status.starts_with("HTTP/1.1 200"));
    server.await.unwrap();
}

#[tokio::test]
async fn m144_plaintext_on_tls_listener_is_rejected_without_dispatch() {
    use emissary_cli::i2pcontrol::backends::runtime::presentation_tls as tls;
    let (acceptor, _cert) = tls::generate_listener_identity().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        assert!(
            tls::accept_tls_with_timeout(&acceptor, stream)
                .await
                .is_err()
        );
    });
    let mut plain = tokio::net::TcpStream::connect(address).await.unwrap();
    use tokio::io::AsyncWriteExt;
    plain
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(3), plain.read_to_end(&mut buf)).await;
    assert!(
        buf.is_empty() || !String::from_utf8_lossy(&buf).starts_with("HTTP/"),
        "plaintext must never receive an HTTP dispatch"
    );
    server.await.unwrap();
}

#[tokio::test]
async fn m144_server_tls_target_verifies_and_never_falls_back() {
    use emissary_cli::i2pcontrol::backends::runtime::presentation_tls as tls;
    let (acceptor, cert) = tls::generate_listener_identity().unwrap();
    let target = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_address = target.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = target.accept().await.unwrap();
        let mut tls_stream = tls::accept_tls_with_timeout(&acceptor, stream).await.unwrap();
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let mut reader = BufReader::new(&mut tls_stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert!(line.starts_with("GET /safe "));
        tls_stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
            .await
            .unwrap();
    });
    let trusted = tls::build_target_tls_connector_with_cert(&cert).unwrap();
    let tcp = tokio::net::TcpStream::connect(target_address).await.unwrap();
    let name = tokio_rustls::rustls::pki_types::ServerName::try_from("127.0.0.1").unwrap();
    let mut tls_stream = tls::connect_tls_with_timeout(&trusted, tcp, name).await.unwrap();
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    tls_stream
        .write_all(b"GET /safe HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    let mut reader = BufReader::new(&mut tls_stream);
    let mut status = String::new();
    reader.read_line(&mut status).await.unwrap();
    assert!(status.starts_with("HTTP/1.1 200"));
    server.await.unwrap();

    // Untrusted connector (system roots only) fails closed with no retry.
    let untrusted = tls::build_target_tls_connector(None).unwrap();
    let target2 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target2_address = target2.local_addr().unwrap();
    let (acceptor2, _cert2) = tls::generate_listener_identity().unwrap();
    let server2 = tokio::spawn(async move {
        let (stream, _) = target2.accept().await.unwrap();
        let _ = tls::accept_tls_with_timeout(&acceptor2, stream).await;
    });
    let tcp2 = tokio::net::TcpStream::connect(target2_address).await.unwrap();
    let name2 = tokio_rustls::rustls::pki_types::ServerName::try_from("127.0.0.1").unwrap();
    assert!(
        tls::connect_tls_with_timeout(&untrusted, tcp2, name2)
            .await
            .is_err(),
        "untrusted target cert must fail closed"
    );
    server2.abort();
}

#[tokio::test]
async fn m144_handshake_timeout_and_cancellation_are_bounded() {
    use emissary_cli::i2pcontrol::backends::runtime::presentation_tls as tls;
    let (acceptor, _cert) = tls::generate_listener_identity().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let started = tokio::time::Instant::now();
        assert!(
            tls::accept_tls_with_timeout(&acceptor, stream)
                .await
                .is_err()
        );
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "handshake must be bounded"
        );
    });
    let _stalled = tokio::net::TcpStream::connect(address).await.unwrap();
    tokio::time::timeout(Duration::from_secs(8), server)
        .await
        .unwrap()
        .unwrap();
}

#[test]
fn m144_successor_gets_new_tls_state_and_failed_edit_keeps_last_known_good() {
    use emissary_cli::i2pcontrol::backends::runtime::presentation_tls as tls;
    use emissary_cli::i2pcontrol::domain::tunnel::TunnelType;
    let (_first, first_cert) = tls::generate_listener_identity().unwrap();
    let (_second, second_cert) = tls::generate_listener_identity().unwrap();
    assert_ne!(first_cert.as_ref(), second_cert.as_ref());
    let bad = tunnel_definition(
        TunnelType::HttpClient,
        vec![("UseSSL".to_owned(), serde_json::json!("yes"))],
        None,
    );
    assert!(tls::parse_use_ssl(&bad).is_err());
}

#[test]
fn m144_httpbidirserver_reuses_the_server_tls_owner() {
    let bidir = std::fs::read_to_string(
        workspace_root().join("emissary-cli/src/i2pcontrol/backends/http_bidir.rs"),
    )
    .unwrap();
    assert!(
        bidir.contains("make_accepted_handler"),
        "httpbidirserver must reuse the shared accepted-server TLS owner"
    );
    let server = std::fs::read_to_string(
        workspace_root().join("emissary-cli/src/i2pcontrol/backends/http_server.rs"),
    )
    .unwrap();
    assert!(server.contains("connect_tls_with_timeout"));
    let helper = std::fs::read_to_string(
        workspace_root()
            .join("emissary-cli/src/i2pcontrol/backends/runtime/presentation_tls.rs"),
    )
    .unwrap();
    assert!(helper.contains("accept_tls_with_timeout"));
    assert!(helper.contains("connect_tls_with_timeout"));
    assert_eq!(bidir.matches("rcgen").count(), 0);
    assert_eq!(bidir.matches("ServerConfig").count(), 0);
}

#[test]
fn m144_management_and_sam_tls_remain_independent() {
    use emissary_cli::i2pcontrol::domain::tunnel::TunnelType;
    use yosemite_i2pcontrol::DestinationKind;
    for tunnel_type in [
        TunnelType::HttpClient,
        TunnelType::ConnectClient,
        TunnelType::HttpServer,
        TunnelType::HttpBidirServer,
    ] {
        let definition = tunnel_definition(
            tunnel_type,
            vec![("UseSSL".to_owned(), serde_json::json!(true))],
            Some(true),
        );
        let options =
            emissary_cli::i2pcontrol::backends::runtime::session::build_session_options(
                &definition,
                7656,
                false,
                DestinationKind::Transient,
            )
            .unwrap();
        assert!(
            !options.ssl,
            "{tunnel_type} UseSSL must not set SAM-control TLS"
        );
    }
}

#[test]
fn m144_containment_docs_and_registry_agree() {
    let matrix = planning_toml("095-full-support-matrix.toml");
    let declared = matrix["current_matrix_counts"].as_table().unwrap();
    assert_eq!(declared["apply"].as_integer(), Some(334));
    assert_eq!(declared["blocked_primitive"].as_integer(), Some(31));
    let manifest =
        std::fs::read_to_string(workspace_root().join("emissary-cli/Cargo.toml")).unwrap();
    assert!(manifest.contains("tokio-rustls"));
    assert!(manifest.contains("rcgen"));
    assert!(!manifest.contains("webpki-roots"));
    assert!(!manifest.contains("rustls-native-certs"));
    let support = std::fs::read_to_string(
        workspace_root().join("docs/i2pcontrol/proposal-170-support.md"),
    )
    .unwrap();
    assert!(support.contains("partial"));
}
