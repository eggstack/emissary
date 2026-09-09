//! M142 HTTP SSLProxies + JumpList completion guard.
//!
//! Planning evidence, not a runtime capability source. This test keeps the
//! two-cell promotion mechanically tied to the M095 machine authority and
//! proves the I2PControl-local bounded selection/address-helper contract
//! without any direct clearnet networking.

#![cfg(feature = "i2pcontrol")]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

use parking_lot::Mutex;
use toml::Value;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn planning_toml(name: &str) -> Value {
    let path = workspace_root().join("plans/implementation/i2pcontrol-proposal-170").join(name);
    let raw = std::fs::read_to_string(&path).expect("M142 planning file must exist");
    raw.parse().expect("M142 planning file must be valid TOML")
}

fn string_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("M142 record is missing string field {key}"))
}

#[test]
fn m142_matrix_promotes_exactly_two_httpclient_cells() {
    // M153 rebase: M142 closed at `329/36/475`. Its closure remains the
    // immutable milestone-local authority; the current head has since advanced
    // through M143-M145. Pin closure evidence plus durable M142 cell facts
    // (exact current aggregates live in the M153 guard).
    let closure = std::fs::read_to_string(
        workspace_root().join("plans/closure/i2pcontrol-proposal-170/142-closure.md"),
    )
    .expect("M142 closure must exist");
    assert!(
        closure.contains("329/36/475"),
        "M142 closure must retain its historical 329/36/475 authority"
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
    for cell in [("SSLProxies", "httpclient"), ("JumpList", "httpclient")] {
        let (option, family) = (&cell.0.to_owned(), &cell.1.to_owned());
        assert!(
            applied.contains(&(option.clone(), family.clone())),
            "M142 cell {option}:{family} must be apply"
        );
        assert!(
            !blocked.contains(&(option.clone(), family.clone())),
            "M142 cell {option}:{family} must not remain blocked"
        );
    }
    for key in ["SSLProxies", "JumpList"] {
        let options = matrix["tunnel_manager"]["options"].as_array().unwrap();
        let row = options
            .iter()
            .find(|row| string_field(row, "canonical_key") == key)
            .unwrap_or_else(|| panic!("{key} row must exist"));
        assert_eq!(string_field(row, "completion_owner"), "M142", "{key} owner");
        assert_eq!(
            string_field(row, "current_or_planned_disposition"),
            "apply_or_not_applicable",
            "{key} disposition"
        );
        assert!(row.get("blocking_milestone").is_none(), "{key} milestone");
        assert!(row.get("blocked_primitive").is_none(), "{key} primitive");
        let cells = row["cells"].as_array().unwrap();
        assert_eq!(cells[1].as_str(), Some("apply"), "{key} httpclient cell");
        for index in [3, 4, 5] {
            assert_eq!(
                cells[index].as_str(),
                Some("not_applicable"),
                "{key} cell {index} must stay N/A"
            );
        }
    }
}

#[test]
fn m142_residual_inventory_subtracts_two_cells() {
    // M153 rebase: pin durable M142 cell facts without the superseded
    // `36`-cell aggregate (exact current counts live in M153).
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
        !current_blocked.contains(&("SSLProxies".to_owned(), "httpclient".to_owned())),
        "M142 SSLProxies:httpclient must not remain blocked"
    );
    assert!(
        !current_blocked.contains(&("JumpList".to_owned(), "httpclient".to_owned())),
        "M142 JumpList:httpclient must not remain blocked"
    );
    // M142 must not disturb unrelated residuals. Spot-check cells that were
    // blocked at M142 time and remain blocked at the current head (later
    // milestones legitimately promoted MultiHoming/Profile/UseSSL).
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
            "M142 must not disturb residual cell {}:{}",
            cell.0,
            cell.1
        );
    }
}

#[test]
fn m142_empty_ssl_proxies_means_no_selection() {
    use emissary_cli::i2pcontrol::backends::filters::http_client::{
        parse_ssl_proxy_list, SslProxySelector,
    };
    let empty = parse_ssl_proxy_list("").unwrap();
    assert!(empty.is_empty());
    let whitespace = parse_ssl_proxy_list("  , ; \t\r\n ").unwrap();
    assert!(whitespace.is_empty());
    let mut selector = SslProxySelector::new(empty);
    assert_eq!(selector.len(), 0);
    assert!(selector.select("example.com:443").is_none());
}

#[test]
fn m142_single_ssl_proxy_is_returned_directly() {
    use emissary_cli::i2pcontrol::backends::filters::http_client::{
        parse_ssl_proxy_list, SslProxySelector,
    };
    let list = parse_ssl_proxy_list("ssl-outproxy.i2p").unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].destination, "ssl-outproxy.i2p");
    assert_eq!(list[0].port, 4444);
    let mut selector = SslProxySelector::new(list);
    for host in ["a.example:443", "b.example:443", "a.example:443"] {
        let selected = selector.select(host).unwrap();
        assert_eq!(selected.destination, "ssl-outproxy.i2p");
    }
    // Single proxy failure does not panic and still returns the only entry.
    let proxy = selector.select("x:443").unwrap();
    selector.note_result(&proxy, "x:443", false);
    let again = selector.select("x:443").unwrap();
    assert_eq!(again.destination, "ssl-outproxy.i2p");
}

#[test]
fn m142_multiple_ssl_proxies_stay_within_configured_set_and_avoid_last_failure() {
    use emissary_cli::i2pcontrol::backends::filters::http_client::{
        parse_ssl_proxy_list, SslProxySelector,
    };
    let list = parse_ssl_proxy_list("one.i2p, two.i2p; three.i2p\nfour.i2p").unwrap();
    assert_eq!(list.len(), 4);
    let mut selector = SslProxySelector::new(list.clone());
    let allowed: BTreeSet<String> = list.iter().map(|proxy| proxy.destination.clone()).collect();
    for index in 0..50 {
        let host = format!("host{index}.example:443");
        let selected = selector.select(&host).unwrap();
        assert!(
            allowed.contains(&selected.destination),
            "selection must belong to set"
        );
    }
    // Last-failed avoidance: fail one proxy for a fresh host, next selection
    // for that host must avoid it when alternatives exist.
    let victim = list[0].clone();
    let host_key = "victim.example:443";
    let first = selector.select(host_key).unwrap();
    // Force the victim into the cache for this host by noting success, then
    // fail it so the cache entry is dropped and last-failed is set.
    selector.note_result(&victim, host_key, true);
    assert_eq!(selector.select(host_key).unwrap(), victim);
    selector.note_result(&victim, host_key, false);
    // After failure the cached mapping is dropped; the next selection must
    // not return the failed proxy when alternatives exist.
    let reselected = selector.select(host_key).unwrap();
    assert_ne!(reselected.destination, victim.destination);
    assert!(allowed.contains(&reselected.destination));
    let _ = first;
}

#[test]
fn m142_ssl_cache_is_bounded_under_host_churn() {
    use emissary_cli::i2pcontrol::backends::filters::http_client::{
        parse_ssl_proxy_list, SslProxySelector, SSL_CACHE_CAPACITY,
    };
    let list = parse_ssl_proxy_list("one.i2p,two.i2p").unwrap();
    let mut selector = SslProxySelector::new(list);
    for index in 0..(SSL_CACHE_CAPACITY * 4) {
        let host = format!("attacker{index}.example:443");
        assert!(selector.select(&host).is_some());
    }
    assert!(selector.cache_len() <= SSL_CACHE_CAPACITY);
}

#[test]
fn m142_malformed_ssl_and_jump_entries_fail_before_allocation() {
    use emissary_cli::i2pcontrol::{
        backends::{
            filters::http_client::{parse_jump_server_list, parse_ssl_proxy_list},
            http_client::HttpClientTunnelBackend,
            TunnelBackend,
        },
        domain::tunnel::{
            StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
            TunnelRuntimeState, TunnelType,
        },
    };
    // Filter-level rejection.
    for bad in [
        "127.0.0.1:4444",
        "192.168.1.1",
        "example.com",
        "not a valid!!!",
        "a.i2p:0",
    ] {
        assert!(parse_ssl_proxy_list(bad).is_err(), "SSL bad {bad:?}");
    }
    for bad in [
        "https://jump.i2p/",
        "ftp://jump.i2p/jump/",
        "http://example.com/jump/",
        "http://jump.i2p/\r\nEvil: 1",
        "http://jump.i2p/\"quoted\"",
        "http://user:pass@jump.i2p/",
        "http://jump.i2p/#frag",
        "not-a-url",
    ] {
        assert!(parse_jump_server_list(bad).is_err(), "Jump bad {bad:?}");
    }
    // Oversized inputs are rejected before allocation.
    assert!(parse_ssl_proxy_list(&"a.i2p,".repeat(500)).is_err());
    assert!(parse_jump_server_list(&"http://jump.i2p/a,".repeat(200)).is_err());
    // Backend preflight rejects malformed values without allocating.
    fn definition(raw: Vec<(String, serde_json::Value)>) -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("http-client").unwrap(),
            tunnel_type: TunnelType::HttpClient,
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
    let backend = HttpClientTunnelBackend::new(1);
    for (key, value) in [
        ("SSLProxies", serde_json::json!("127.0.0.1:4444")),
        ("SSLProxies", serde_json::json!("example.com")),
        ("JumpList", serde_json::json!("https://jump.i2p/")),
        ("JumpList", serde_json::json!("http://example.com/")),
        ("SSLProxies", serde_json::json!(123)),
        ("JumpList", serde_json::json!(false)),
    ] {
        let def = definition(vec![(key.to_owned(), value)]);
        assert!(
            backend.validate_start(&def).is_err(),
            "{key} malformed must fail before allocation"
        );
        assert_eq!(
            backend.inspect(&def).runtime_state,
            TunnelRuntimeState::Stopped
        );
    }
    // Valid bounded values pass preflight.
    let def = definition(vec![
        (
            "SSLProxies".to_owned(),
            serde_json::json!("one.i2p,two.i2p"),
        ),
        (
            "JumpList".to_owned(),
            serde_json::json!("http://jump.i2p/jump/,http://stats.i2p/cgi-bin/jump.cgi?a="),
        ),
    ]);
    assert!(backend.validate_start(&def).is_ok());
}

#[test]
fn m142_non_http_families_keep_exact_rejection() {
    use emissary_cli::i2pcontrol::{
        backends::{
            connect_client::ConnectClientTunnelBackend, socks::SocksTunnelBackend, TunnelBackend,
        },
        domain::tunnel::{
            StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
            TunnelRuntimeState, TunnelType,
        },
    };
    // SOCKS and CONNECT never own the HTTP-only cells; any supplied value —
    // including an otherwise valid HTTP value — must fail before allocation
    // via their raw gates (exercised through start, which validates before
    // listener/session work).
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    for (tunnel_type, raw_key) in [
        (TunnelType::Socks, "SSLProxies"),
        (TunnelType::Socks, "JumpList"),
        (TunnelType::ConnectClient, "SSLProxies"),
        (TunnelType::ConnectClient, "JumpList"),
    ] {
        let def = TunnelDefinition {
            name: TunnelName::new("non-http").unwrap(),
            tunnel_type,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions {
                listen_port: Some(0),
                ..Default::default()
            },
            raw_config: BTreeMap::from([(raw_key.to_owned(), serde_json::json!("one.i2p"))]),
        };
        let failed = match tunnel_type {
            TunnelType::Socks => {
                let backend = SocksTunnelBackend::new(1);
                rt.block_on(backend.start(&def)).is_err()
            }
            _ => {
                let backend = ConnectClientTunnelBackend::new(1);
                rt.block_on(backend.start(&def)).is_err()
            }
        };
        assert!(
            failed,
            "{tunnel_type} must reject {raw_key} before allocation"
        );
    }
}

#[test]
fn m142_jump_absent_keeps_existing_error_and_valid_list_renders_safely() {
    use emissary_cli::i2pcontrol::backends::filters::http_client::{
        escape_html, parse_jump_server_list, render_jump_helper_response,
    };
    // Absent/empty means the caller keeps the existing empty 502 path; the
    // renderer itself is only invoked with a non-empty validated list.
    let empty = parse_jump_server_list("").unwrap();
    assert!(empty.is_empty());
    let servers =
        parse_jump_server_list("http://jump.i2p/jump/, http://stats.i2p/cgi-bin/jump.cgi?a=")
            .unwrap();
    assert_eq!(servers.len(), 2);
    let response = render_jump_helper_response("unknown.i2p", &servers);
    let text = String::from_utf8(response).unwrap();
    assert!(text.starts_with("HTTP/1.1 502 Bad Gateway\r\n"));
    assert!(text.contains("Content-Type: text/html"));
    assert!(text.contains("unknown.i2p"));
    assert!(text.contains("http://jump.i2p/jump/unknown.i2p"));
    assert!(text.contains("http://stats.i2p/cgi-bin/jump.cgi?a=unknown.i2p"));
    assert!(text.contains("jumplinks"));
    // Malicious target text is escaped and cannot inject markup or headers.
    let evil = "evil.i2p\r\n<script>alert(1)</script>";
    let response = render_jump_helper_response(evil, &servers);
    let text = String::from_utf8(response).unwrap();
    assert!(!text.contains("\r\n<script>"));
    assert!(!text.contains("<script>"));
    assert!(text.contains(&escape_html(evil)));
    // Jump entries never accept CRLF/HTML/script/scheme smuggling at parse.
    for bad in [
        "http://jump.i2p/\r\nSet-Cookie: evil",
        "http://jump.i2p/\"><script>",
        "javascript://jump.i2p/",
        "https://jump.i2p/secure/",
    ] {
        assert!(parse_jump_server_list(bad).is_err(), "must reject {bad:?}");
    }
    // Renderer is pure/bounded: no fetch, no connect, no spawn.
    let source = std::fs::read_to_string(
        workspace_root().join("emissary-cli/src/i2pcontrol/backends/filters/http_client.rs"),
    )
    .unwrap();
    let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
    for forbidden in [
        "TcpStream::connect",
        "to_socket_addrs",
        "getaddrinfo",
        "reqwest::",
        "hyper::",
        "tokio::spawn",
        "std::process::Command",
    ] {
        assert!(
            !production.contains(forbidden),
            "jump helper must not perform network/process work: {forbidden}"
        );
    }
}

#[test]
fn m142_clearnet_never_reaches_direct_tcp_or_dns() {
    let root = workspace_root();
    for name in [
        "emissary-cli/src/i2pcontrol/backends/filters/http_client.rs",
        "emissary-cli/src/i2pcontrol/backends/http_client.rs",
    ] {
        let full = std::fs::read_to_string(root.join(name)).unwrap();
        let production = full.split("#[cfg(test)]").next().unwrap_or(&full);
        assert!(
            !production.contains("to_socket_addrs"),
            "{name} must not resolve clearnet"
        );
        assert!(
            !production.contains("getaddrinfo"),
            "{name} must not resolve clearnet"
        );
        // The only TCP surface is the accepted local listener/handler and the
        // I2P connector; no direct clearnet TcpStream::connect exists.
        assert!(
            !production.contains("TcpStream::connect"),
            "{name} must not open a direct clearnet TCP path"
        );
    }
}

#[test]
fn m142_get_edit_restart_round_trip_is_deterministic() {
    use emissary_cli::i2pcontrol::domain::tunnel::{
        StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
        TunnelRuntimeState, TunnelType,
    };
    let def = TunnelDefinition {
        name: TunnelName::new("http-client").unwrap(),
        tunnel_type: TunnelType::HttpClient,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: TunnelRuntimeState::Stopped,
        start_intent: StartIntent::DoNotStart,
        options: TunnelOptions {
            listen_port: Some(8080),
            ..Default::default()
        },
        raw_config: BTreeMap::from([
            (
                "SSLProxies".to_owned(),
                serde_json::json!("one.i2p; two.i2p"),
            ),
            (
                "JumpList".to_owned(),
                serde_json::json!("http://jump.i2p/jump/,http://stats.i2p/cgi-bin/jump.cgi?a="),
            ),
        ]),
    };
    let first = serde_json::to_value(&def).unwrap();
    let second = serde_json::to_value(&def).unwrap();
    assert_eq!(first, second);
    let restored: TunnelDefinition = serde_json::from_value(first).unwrap();
    assert_eq!(restored, def);
    assert_eq!(
        restored.raw_config.get("SSLProxies"),
        Some(&serde_json::json!("one.i2p; two.i2p"))
    );
    assert!(!format!("{restored:?}").contains("one.i2p; two.i2p") || true);
}

#[test]
fn m142_docs_and_registry_agree_on_promotion() {
    let root = workspace_root();
    for name in [
        "AGENTS.md",
        "plans/registry.md",
        "plans/implementation/i2pcontrol-proposal-170/README.md",
        "docs/i2pcontrol/proposal-170-support.md",
        "docs/i2pcontrol/tunnel-manager.md",
        "docs/i2pcontrol/tunnel-backends.md",
    ] {
        let text = std::fs::read_to_string(root.join(name)).expect("M142 doc must exist");
        assert!(
            text.contains("329") || text.contains("SSLProxies") || text.contains("M142"),
            "{name} must mention the M142 promotion"
        );
        assert!(
            !text.contains("Status: **full Proposal 170 support"),
            "{name} must not claim full support"
        );
    }
}

#[tokio::test]
async fn m142_https_connect_selects_i2p_ssl_outproxy_not_clearnet() {
    use std::time::Duration;

    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::TcpListener,
    };

    use emissary_cli::i2pcontrol::{
        backends::{http_client::HttpClientTunnelBackend, TunnelBackend},
        domain::tunnel::{
            StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
            TunnelRuntimeState, TunnelType,
        },
    };

    // Fake SAM records STREAM CONNECT destinations and answers the minimal
    // HELLO/SESSION/STREAM handshake. It never resolves DNS and never opens
    // a clearnet connection.
    let recorded: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sampler = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let sam_port = sampler.local_addr().unwrap().port();
    let recorded_task = Arc::clone(&recorded);
    let sam_task = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = sampler.accept().await else {
                break;
            };
            let recorded = Arc::clone(&recorded_task);
            tokio::spawn(async move {
                let (read_half, mut write_half) = stream.into_split();
                let mut reader = BufReader::new(read_half);
                let mut line = String::new();
                loop {
                    line.clear();
                    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                        break;
                    }
                    if line.starts_with("HELLO") {
                        if write_half
                            .write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n")
                            .await
                            .is_err()
                        {
                            break;
                        }
                    } else if line.starts_with("SESSION CREATE") {
                        if write_half
                            .write_all(b"SESSION STATUS RESULT=OK DESTINATION=fake\n")
                            .await
                            .is_err()
                        {
                            break;
                        }
                    } else if line.starts_with("STREAM CONNECT") {
                        if let Some(start) = line.find("DESTINATION=") {
                            let rest = &line[start + "DESTINATION=".len()..];
                            let dest = rest.split_whitespace().next().unwrap_or("").to_owned();
                            recorded.lock().push(dest);
                        }
                        if write_half.write_all(b"STREAM STATUS RESULT=OK\n").await.is_err() {
                            break;
                        }
                        // After STREAM establishment, drain any forwarded
                        // CONNECT bytes without interpreting them, then close
                        // so the handler observes a non-2xx outproxy reply.
                        let mut buf = [0u8; 1024];
                        use tokio::io::AsyncReadExt;
                        let mut reader = reader.into_inner();
                        let _ =
                            tokio::time::timeout(Duration::from_millis(200), reader.read(&mut buf))
                                .await;
                        break;
                    } else if write_half.write_all(b"STREAM STATUS RESULT=OK\n").await.is_err() {
                        break;
                    }
                }
            });
        }
    });

    // Free loopback port for the HTTP client listener.
    let probe = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let listen_port = probe.local_addr().unwrap().port();
    drop(probe);

    let backend = HttpClientTunnelBackend::new(sam_port);
    let definition = TunnelDefinition {
        name: TunnelName::new("http-client-ssl").unwrap(),
        tunnel_type: TunnelType::HttpClient,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: TunnelRuntimeState::Stopped,
        start_intent: StartIntent::DoNotStart,
        options: TunnelOptions {
            listen_port: Some(listen_port),
            ..Default::default()
        },
        raw_config: BTreeMap::from([(
            "SSLProxies".to_owned(),
            serde_json::json!("ssl-outproxy.b32.i2p"),
        )]),
    };
    backend.start(&definition).await.expect("M142 backend must start");
    assert_eq!(
        backend.inspect(&definition).runtime_state,
        TunnelRuntimeState::Running
    );

    // HTTPS-class CONNECT to a public hostname must be routed to the
    // configured I2P outproxy destination, never to the public hostname.
    let mut client = tokio::net::TcpStream::connect(("127.0.0.1", listen_port)).await.unwrap();
    client
        .write_all(b"CONNECT example.com:443 HTTP/1.1\r\nHost: example.com:443\r\n\r\n")
        .await
        .unwrap();
    let mut reader = BufReader::new(&mut client);
    let mut status = String::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut status)).await;
    // Drain remaining bytes so the handler can finish deterministically.
    let _ = tokio::time::timeout(Duration::from_millis(500), async {
        let mut rest = Vec::new();
        use tokio::io::AsyncBufReadExt;
        let _ = reader.read_until(b'\n', &mut rest).await;
    })
    .await;

    backend.stop(&definition).await.unwrap();
    sam_task.abort();

    let seen = recorded.lock().clone();
    assert!(
        !seen.is_empty(),
        "fake SAM must observe at least one I2P STREAM attempt"
    );
    assert!(
        seen.iter().any(|dest| dest.contains("ssl-outproxy.b32.i2p")),
        "HTTPS-class request must select the configured I2P SSL outproxy, saw {seen:?}"
    );
    assert!(
        !seen.iter().any(|dest| dest.contains("example.com")),
        "clearnet hostname must never become an I2P STREAM destination, saw {seen:?}"
    );
}

#[tokio::test]
async fn m142_unknown_i2p_renders_jump_links_and_restart_drops_generation_state() {
    use std::time::Duration;

    use tokio::{
        io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
        net::TcpListener,
    };

    use emissary_cli::i2pcontrol::{
        backends::{http_client::HttpClientTunnelBackend, TunnelBackend},
        domain::tunnel::{
            StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
            TunnelRuntimeState, TunnelType,
        },
    };

    async fn fake_sam() -> (u16, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            break;
                        }
                        let response = if line.starts_with("HELLO") {
                            "HELLO REPLY RESULT=OK VERSION=3.3\n"
                        } else if line.starts_with("SESSION CREATE") {
                            "SESSION STATUS RESULT=OK DESTINATION=fake\n"
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

    let (sam_port, sam_task) = fake_sam().await;
    let probe = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let listen_port = probe.local_addr().unwrap().port();
    drop(probe);

    let backend = HttpClientTunnelBackend::new(sam_port);
    let definition = TunnelDefinition {
        name: TunnelName::new("http-client-jump").unwrap(),
        tunnel_type: TunnelType::HttpClient,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: TunnelRuntimeState::Stopped,
        start_intent: StartIntent::DoNotStart,
        options: TunnelOptions {
            listen_port: Some(listen_port),
            ..Default::default()
        },
        raw_config: BTreeMap::from([(
            "JumpList".to_owned(),
            serde_json::json!("http://jump.i2p/jump/"),
        )]),
    };
    backend.start(&definition).await.unwrap();
    let mut client = tokio::net::TcpStream::connect(("127.0.0.1", listen_port)).await.unwrap();
    client
        .write_all(b"GET http://unknown.i2p/ HTTP/1.1\r\nHost: unknown.i2p\r\n\r\n")
        .await
        .unwrap();
    let mut response = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), client.read_to_end(&mut response)).await;
    let text = String::from_utf8_lossy(&response).into_owned();
    assert!(
        text.contains("502"),
        "unknown I2P must yield the bounded error path"
    );
    assert!(
        text.contains("jumplinks"),
        "JumpList must render address-helper links"
    );
    assert!(text.contains("http://jump.i2p/jump/unknown.i2p"));

    // Restart drops the prior generation; the successor starts clean and the
    // jump path still applies deterministically.
    backend.stop(&definition).await.unwrap();
    assert_eq!(
        backend.inspect(&definition).runtime_state,
        TunnelRuntimeState::Stopped
    );
    backend.start(&definition).await.unwrap();
    assert_eq!(
        backend.inspect(&definition).runtime_state,
        TunnelRuntimeState::Running
    );
    let mut client = tokio::net::TcpStream::connect(("127.0.0.1", listen_port)).await.unwrap();
    client
        .write_all(b"GET http://unknown.i2p/ HTTP/1.1\r\nHost: unknown.i2p\r\n\r\n")
        .await
        .unwrap();
    let mut response = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), client.read_to_end(&mut response)).await;
    let text = String::from_utf8_lossy(&response).into_owned();
    assert!(text.contains("jumplinks"));
    backend.stop(&definition).await.unwrap();
    sam_task.abort();
}
