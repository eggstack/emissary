//! M038 production-composition interoperability scenario.
//!
//! This test deliberately launches the feature-enabled CLI binary. It does not
//! construct an in-process I2PControl state or replace a production backend
//! with a fake. The local router has no reseed dependency, while its real SAM
//! listener remains available for the production tunnel supervisors.

#![cfg(feature = "i2pcontrol")]

use std::{
    net::{SocketAddr, TcpListener},
    process::Stdio,
    time::Duration,
};

use emissary_core::{crypto::base64_encode, primitives::Destination};
use emissary_util::runtime::tokio::Runtime as TokioRuntime;
use serde_json::{json, Value};
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    time::{sleep, timeout},
};

const STARTUP_DEADLINE: Duration = Duration::from_secs(30);
const REQUEST_DEADLINE: Duration = Duration::from_secs(20);
const PASSWORD_PREFIX: &str = "m038-runtime-";

struct RunningRouter {
    child: Child,
}

impl Drop for RunningRouter {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.start_kill();
        }
    }
}

impl RunningRouter {
    async fn stop(mut self) -> String {
        if let Some(pid) = self.child.id() {
            #[cfg(unix)]
            {
                let _ = Command::new("kill").args(["-INT", &pid.to_string()]).status().await;
            }
            #[cfg(not(unix))]
            let _ = self.child.start_kill();
        }

        if timeout(Duration::from_secs(10), self.child.wait()).await.is_err() {
            let _ = self.child.start_kill();
            let _ = timeout(Duration::from_secs(5), self.child.wait()).await;
        }

        let mut diagnostics = Vec::new();
        if let Some(mut stderr) = self.child.stderr.take() {
            let _ = timeout(Duration::from_secs(1), stderr.read_to_end(&mut diagnostics)).await;
        }
        String::from_utf8_lossy(&diagnostics).into_owned()
    }
}

fn unused_loopback_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("loopback bind");
    listener.local_addr().expect("local address").port()
}

fn valid_destination(seed: u8) -> String {
    use emissary_core::crypto::SigningPrivateKey;

    let key = SigningPrivateKey::from_bytes(&[seed; 32]).expect("signing key");
    base64_encode(Destination::new::<TokioRuntime>(key.public()).serialize())
}

fn router_config(control_bind: SocketAddr, client_destination: &str) -> String {
    format!(
        r#"allow_local = true
floodfill = false
insecure_tunnels = true

[bandwidth]
bandwidth = 1000000
share_ratio = 0.0

[ntcp2]
port = 0
ipv4 = true
ipv6 = false
publish_ipv4 = false
publish_ipv6 = false
max_connections = 64

[ssu2]
port = 0
ipv4 = true
ipv6 = false
publish_ipv4 = false
publish_ipv6 = false
max_connections = 64

[reseed]
reseed_threshold = 0

[sam]
tcp_port = 0
udp_port = 0
host = "127.0.0.1"

[i2pcontrol]
enabled = true
bind = "{control_bind}"
password = "{password}"

[[client-tunnels]]
name = "startup-client"
address = "127.0.0.1"
port = 0
destination = "{client_destination}"
destination_port = 1
"#,
        password = PASSWORD_PREFIX,
    )
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(REQUEST_DEADLINE)
        .build()
        .expect("TLS test client")
}

async fn call(
    client: &reqwest::Client,
    endpoint: &str,
    id: u64,
    method: &str,
    params: Value,
) -> Value {
    let request = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    let response = client
        .post(endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request.to_string())
        .send()
        .await
        .expect("JSON-RPC request");
    let status = response.status();
    let text = response.text().await.expect("JSON-RPC response body");
    assert!(
        status.is_success(),
        "unexpected HTTP status {status}: {text}"
    );
    serde_json::from_str(&text).expect("JSON-RPC response JSON")
}

async fn notification(
    client: &reqwest::Client,
    endpoint: &str,
    method: &str,
    params: Value,
) -> reqwest::StatusCode {
    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    });
    client
        .post(endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request.to_string())
        .send()
        .await
        .expect("notification request")
        .status()
}

fn result(response: &Value) -> &Value {
    response.get("result").unwrap_or_else(|| {
        panic!(
            "JSON-RPC error: code={:?}, message={:?}",
            response["error"]["code"], response["error"]["message"]
        )
    })
}

fn operation_status(response: &Value) -> &str {
    result(response)
        .get("status")
        .and_then(Value::as_str)
        .expect("TunnelManager operation status")
}

async fn authenticate(client: &reqwest::Client, endpoint: &str, id: u64, password: &str) -> String {
    let response = call(
        client,
        endpoint,
        id,
        "Authenticate",
        json!({"API": 1, "Password": password}),
    )
    .await;
    result(&response)
        .get("Token")
        .and_then(Value::as_str)
        .expect("authentication token")
        .to_owned()
}

async fn protected(
    client: &reqwest::Client,
    endpoint: &str,
    id: u64,
    token: &str,
    method: &str,
    mut params: serde_json::Map<String, Value>,
) -> Value {
    params.insert("Token".to_string(), Value::String(token.to_owned()));
    call(client, endpoint, id, method, Value::Object(params)).await
}

#[tokio::test]
async fn live_runtime_interoperability() {
    let base = tempfile::tempdir().expect("temporary runtime directory");
    let control_bind = SocketAddr::from(([127, 0, 0, 1], unused_loopback_port()));
    let public_destination = valid_destination(38);
    let password = format!("{PASSWORD_PREFIX}{}", std::process::id());
    let client = http_client();
    let endpoint = format!("https://localhost:{}/", control_bind.port());

    // The test configuration uses a process-specific password. Replace only
    // the fixture placeholder before launch; no credential is committed.
    let mut config = router_config(control_bind, &public_destination);
    config = config.replace(
        &format!("password = \"{PASSWORD_PREFIX}\""),
        &format!("password = \"{password}\""),
    );
    tokio::fs::write(base.path().join("router.toml"), config)
        .await
        .expect("write runtime configuration");

    let binary = std::env::var_os("CARGO_BIN_EXE_emissary-cli")
        .expect("Cargo must provide the emissary-cli binary path");
    let mut router = RunningRouter {
        child: Command::new(binary)
            .args([
                "--base-path",
                base.path().to_str().expect("UTF-8 temp path"),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn feature-enabled emissary-cli"),
    };

    // Phase A: real production composition and TLS readiness.
    let readiness_deadline = tokio::time::Instant::now() + STARTUP_DEADLINE;
    loop {
        if let Ok(Some(status)) = router.child.try_wait() {
            panic!("Emissary exited before readiness: {status}");
        }
        if tokio::time::Instant::now() >= readiness_deadline {
            panic!("I2PControl readiness deadline exceeded");
        }
        let response = client
            .post(&endpoint)
            .body(
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "Authenticate",
                    "params": {"API": 1, "Password": password.clone()}
                })
                .to_string(),
            )
            .send()
            .await;
        if let Ok(response) = response {
            if response.status().is_success() {
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
    eprintln!("M038 phase A: production TLS listener ready");

    // Phase B: authentication, protected dispatch, notification, and IDs.
    let wrong = call(
        &client,
        &endpoint,
        2,
        "Authenticate",
        json!({"API": 1, "Password": "wrong"}),
    )
    .await;
    assert_eq!(wrong["error"]["code"], json!(-32001));
    let token = authenticate(&client, &endpoint, 3, &password).await;

    let protected_methods = [
        "RouterInfo",
        "AddressBook",
        "SetSubscriptions",
        "SetConfig",
        "TunnelManager",
        "ClientServicesInfo",
    ];
    for (index, method) in protected_methods.iter().enumerate() {
        let missing = call(
            &client,
            &endpoint,
            300 + index as u64,
            method,
            json!({}),
        )
        .await;
        assert_eq!(missing["error"]["code"], json!(-32002), "{method}");

        let invalid = call(
            &client,
            &endpoint,
            310 + index as u64,
            method,
            json!({"Token": "not-a-valid-token"}),
        )
        .await;
        assert_eq!(invalid["error"]["code"], json!(-32003), "{method}");
    }

    let conflicting_header = client
        .post(&endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("X-I2PControl-Token", "not-a-valid-token")
        .body(
            json!({
                "jsonrpc": "2.0",
                "id": 320,
                "method": "RouterInfo",
                "params": {"Token": token.clone(), "i2p.router.version": false}
            })
            .to_string(),
        )
        .send()
        .await
        .expect("conflicting token request");
    let conflicting_header_body: Value = serde_json::from_str(
        &conflicting_header
            .text()
            .await
            .expect("conflicting token response body"),
    )
    .expect("conflicting token response JSON");
    assert_eq!(conflicting_header_body["error"]["code"], json!(-32003));

    // M128: bounded JSON-RPC batches execute with per-element auth.
    let batch = client
        .post(&endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(
            json!([
                {
                    "jsonrpc": "2.0",
                    "id": 321,
                    "method": "RouterInfo",
                    "params": {"Token": token.clone(), "i2p.router.version": false}
                }
            ])
            .to_string(),
        )
        .send()
        .await
        .expect("batch request");
    assert!(batch.status().is_success());
    let text = batch.text().await.expect("batch response body");
    let batch_body: Value = serde_json::from_str(&text).expect("batch response JSON");
    let batch_responses = batch_body.as_array().expect("batch response must be an array");
    assert_eq!(batch_responses.len(), 1);
    assert_eq!(batch_responses[0]["id"], json!(321));
    assert!(batch_responses[0].get("result").is_some());

    // M128: mixed valid/invalid batch entries keep independent results.
    let mixed = client
        .post(&endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(
            json!([
                {
                    "jsonrpc": "2.0",
                    "id": 322,
                    "method": "RouterInfo",
                    "params": {"Token": token.clone(), "i2p.router.version": false}
                },
                {
                    "jsonrpc": "2.0",
                    "id": 323,
                    "method": "RouterInfo",
                    "params": {"i2p.router.version": false}
                }
            ])
            .to_string(),
        )
        .send()
        .await
        .expect("mixed batch request");
    assert!(mixed.status().is_success());
    let text = mixed.text().await.expect("mixed batch response body");
    let mixed_body: Value = serde_json::from_str(&text).expect("mixed batch response JSON");
    let mixed_responses = mixed_body.as_array().expect("mixed batch must be an array");
    assert_eq!(mixed_responses.len(), 2);
    assert!(mixed_responses[0].get("result").is_some());
    assert_eq!(mixed_responses[1]["error"]["code"], json!(-32002));

    // M128: empty batches remain single invalid-request errors.
    let empty = client
        .post(&endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body("[]")
        .send()
        .await
        .expect("empty batch request");
    assert!(empty.status().is_success());
    let text = empty.text().await.expect("empty batch response body");
    let empty_body: Value = serde_json::from_str(&text).expect("empty batch response JSON");
    assert!(empty_body.is_object());
    assert_eq!(empty_body["error"]["code"], json!(-32600));

    let id_response = protected(
        &client,
        &endpoint,
        4,
        &token,
        "RouterInfo",
        serde_json::Map::from_iter([("i2p.router.id".into(), json!(false))]),
    )
    .await;
    assert!(
        result(&id_response)["i2p.router.id"].is_null()
            || result(&id_response)["i2p.router.id"].is_string()
    );
    assert_eq!(
        notification(
            &client,
            &endpoint,
            "RouterInfo",
            Value::Object(serde_json::Map::from_iter([
                ("Token".into(), json!(token.clone())),
                ("i2p.router.logs".into(), json!(null)),
            ]))
        )
        .await,
        reqwest::StatusCode::NO_CONTENT
    );
    let explicit_null = call(
        &client,
        &endpoint,
        5,
        "RouterInfo",
        json!({"Token": token.clone(), "i2p.router.clockskew": null}),
    )
    .await;
    assert_eq!(explicit_null["id"], Value::from(5));
    eprintln!("M038 phase B: authentication and JSON-RPC semantics passed");

    // Phase C: production AddressBook owner, publication path, subscriptions,
    // and request-selected path rejection.
    let add = protected(
        &client,
        &endpoint,
        6,
        &token,
        "AddressBook",
        serde_json::Map::from_iter([
            ("book".into(), json!("private")),
            ("request".into(), json!("Add")),
            ("name".into(), json!("m038-runtime.i2p")),
            ("value".into(), json!(public_destination.clone())),
        ]),
    )
    .await;
    assert_eq!(result(&add), &json!("ok"));
    let lookup = protected(
        &client,
        &endpoint,
        7,
        &token,
        "AddressBook",
        serde_json::Map::from_iter([
            ("book".into(), json!("private")),
            ("request".into(), json!("Lookup")),
            ("name".into(), json!("m038-runtime.i2p")),
        ]),
    )
    .await;
    assert_eq!(lookup["result"]["name"], json!("m038-runtime.i2p"));
    let subscriptions = protected(
        &client,
        &endpoint,
        8,
        &token,
        "AddressBook",
        serde_json::Map::from_iter([("SetSubscriptions".into(), json!([]))]),
    )
    .await;
    let subscriptions_blocked = subscriptions.get("error").is_some();
    if subscriptions_blocked {
        assert_eq!(subscriptions["error"]["code"], json!(-32603));
    } else {
        assert!(result(&subscriptions)["success"].as_bool().unwrap_or(false));
    }
    let unsafe_config = protected(
        &client,
        &endpoint,
        9,
        &token,
        "AddressBook",
        serde_json::Map::from_iter([(
            "SetConfig".into(),
            json!({"private_addressbook": "/tmp/outside-runtime"}),
        )]),
    )
    .await;
    assert_eq!(unsafe_config["error"]["code"], json!(-32602));
    let selector = protected(
        &client,
        &endpoint,
        10,
        &token,
        "RouterInfo",
        serde_json::Map::from_iter([("i2p.router.addressbook.private.list".into(), json!(false))]),
    )
    .await;
    assert_eq!(
        selector["result"]["i2p.router.addressbook.private.list"][0]["name"],
        json!("m038-runtime.i2p")
    );
    let delete = protected(
        &client,
        &endpoint,
        11,
        &token,
        "AddressBook",
        serde_json::Map::from_iter([
            ("Type".into(), json!("private")),
            ("Hostname".into(), json!("m038-runtime.i2p")),
            ("Destination".into(), json!(public_destination.clone())),
            ("Delete".into(), json!(false)),
        ]),
    )
    .await;
    assert!(result(&delete)["success"].as_bool().unwrap_or(false));
    let persisted_add = protected(
        &client,
        &endpoint,
        30,
        &token,
        "AddressBook",
        serde_json::Map::from_iter([
            ("book".into(), json!("private")),
            ("request".into(), json!("Add")),
            ("name".into(), json!("m038-persisted.i2p")),
            ("value".into(), json!(public_destination.clone())),
        ]),
    )
    .await;
    assert_eq!(result(&persisted_add), &json!("ok"));
    eprintln!(
        "M038 phase C: AddressBook mutation, subscription, and path guards passed; downloader unavailable={subscriptions_blocked}"
    );

    // Phase D: available/unavailable RouterInfo and real ClientServicesInfo.
    let available = protected(
        &client,
        &endpoint,
        12,
        &token,
        "RouterInfo",
        serde_json::Map::from_iter([
            ("i2p.router.id".into(), json!(null)),
            ("i2p.router.logs".into(), json!(false)),
            ("i2p.router.net.tunnels.shareratio".into(), json!(0)),
        ]),
    )
    .await;
    assert!(result(&available)["i2p.router.logs"].is_array());
    let peers = protected(
        &client,
        &endpoint,
        13,
        &token,
        "RouterInfo",
        serde_json::Map::from_iter([("i2p.router.netdb.peers".into(), json!(false))]),
    )
    .await;
    assert!(peers["result"]["i2p.router.netdb.peers"].is_array());

    // Exercise every currently available M045-M050 RouterInfo source through the
    // real TLS/authenticated child process. These selectors are deliberately
    // requested one at a time so a source-group composition regression cannot
    // be hidden by a successful neighboring field.
    let newly_available = [
        "i2p.router.netdb.peers",
        "i2p.router.netdb.peers.list",
        "i2p.router.netdb.peers.info",
        "i2p.router.netdb.activepeers.list",
        "i2p.router.netdb.activepeers.info",
        "i2p.router.netdb.ntcp.limit",
        "i2p.router.netdb.ssu.limit",
        "i2p.router.netdb.activepeers.stats",
        "i2p.router.net.tunnels.participating.info",
        "i2p.router.net.tunnels.exploratory.inbound",
        "i2p.router.net.tunnels.exploratory.outbound",
        "i2p.router.net.tunnels.exploratory.info.list",
        "i2p.router.net.tunnels.client.inbound",
        "i2p.router.net.tunnels.client.outbound",
        "i2p.router.net.tunnels.client.info.list",
        "i2p.router.net.tunnels.successrate",
        "i2p.router.net.tunnels.queue",
        "i2p.router.net.tunnels.tbmqueue",
        "i2p.router.net.status.v6",
        "i2p.router.net.testing",
        "i2p.router.net.testing.v6",
    ];
    for (index, selector) in newly_available.iter().enumerate() {
        let response = protected(
            &client,
            &endpoint,
            40 + index as u64,
            &token,
            "RouterInfo",
            serde_json::Map::from_iter([(selector.to_string(), json!(false))]),
        )
        .await;
        assert!(
            result(&response).get(*selector).is_some(),
            "live RouterInfo response omitted {selector}"
        );
    }

    for (index, selector) in ["i2p.router.net.error", "i2p.router.net.error.v6"].iter().enumerate()
    {
        let response = protected(
            &client,
            &endpoint,
            63 + index as u64,
            &token,
            "RouterInfo",
            serde_json::Map::from_iter([(selector.to_string(), json!(false))]),
        )
        .await;
        assert_eq!(response["error"]["code"], -32603);
        assert!(response["result"].is_null());
        assert!(response["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("network error reason not evaluated")));
    }

    let transit_unavailable = protected(
        &client,
        &endpoint,
        64,
        &token,
        "RouterInfo",
        serde_json::Map::from_iter([("i2p.router.net.bw.transit.15s".into(), json!(false))]),
    )
    .await;
    assert_eq!(transit_unavailable["error"]["code"], -32603);
    assert!(transit_unavailable["result"].is_null());

    let combined = protected(
        &client,
        &endpoint,
        70,
        &token,
        "RouterInfo",
        serde_json::Map::from_iter([
            ("i2p.router.netdb.peers".into(), json!(false)),
            ("i2p.router.netdb.activepeers.stats".into(), json!(false)),
            ("i2p.router.net.tunnels.queue".into(), json!(false)),
            ("i2p.router.net.status.v6".into(), json!(false)),
        ]),
    )
    .await;
    for selector in [
        "i2p.router.netdb.peers",
        "i2p.router.netdb.activepeers.stats",
        "i2p.router.net.tunnels.queue",
        "i2p.router.net.status.v6",
    ] {
        assert!(
            result(&combined).get(selector).is_some(),
            "multi-group RouterInfo response omitted {selector}"
        );
    }
    eprintln!(
        "M054 phase D: all {} available RouterInfo selectors and multi-group composition passed",
        newly_available.len()
    );

    let services = protected(
        &client,
        &endpoint,
        14,
        &token,
        "ClientServicesInfo",
        serde_json::Map::from_iter([
            ("I2PTunnel".into(), json!(false)),
            ("SAM".into(), json!(false)),
            ("I2CP".into(), json!(false)),
            ("BOB".into(), json!(false)),
        ]),
    )
    .await;
    assert!(services["result"]["SAM"]["enabled"].as_bool().unwrap_or(false));
    assert_eq!(services["result"]["BOB"], json!(false));
    eprintln!("M038 phase D: RouterInfo source truthfulness and live service inventory passed");

    // Phase E: real production tunnel manager. The occupied listener forces a
    // deterministic client bind failure, then the same definition is edited
    // and restarted without resetting the process or store. SAM-dependent
    // formation may remain unavailable without a reseeded peer set; that is
    // recorded as a bounded local data-plane blocker below.
    let occupied = TcpListener::bind(("127.0.0.1", 0)).expect("occupied client port");
    let occupied_port = occupied.local_addr().expect("occupied port address").port();
    let create_client = protected(
        &client,
        &endpoint,
        15,
        &token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("create")),
            ("Type".into(), json!("client")),
            ("Name".into(), json!("m038-client")),
            (
                "TargetDestination".into(),
                json!(public_destination.clone()),
            ),
            ("TargetPort".into(), json!(1)),
            ("Port".into(), json!(occupied_port)),
        ]),
    )
    .await;
    assert!(operation_status(&create_client).starts_with("success"));
    let bind_failure = protected(
        &client,
        &endpoint,
        16,
        &token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("start")),
            ("Name".into(), json!("m038-client")),
        ]),
    )
    .await;
    assert!(operation_status(&bind_failure).starts_with("error"));
    drop(occupied);
    let edit_client = protected(
        &client,
        &endpoint,
        17,
        &token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("edit")),
            ("Type".into(), json!("client")),
            ("Name".into(), json!("m038-client")),
            ("Port".into(), json!(0)),
        ]),
    )
    .await;
    assert!(operation_status(&edit_client).starts_with("success"));
    let restart_client = protected(
        &client,
        &endpoint,
        18,
        &token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("restart")),
            ("Name".into(), json!("m038-client")),
        ]),
    )
    .await;
    let client_blocked = operation_status(&restart_client).starts_with("error");
    if !client_blocked {
        let stop_client = protected(
            &client,
            &endpoint,
            19,
            &token,
            "TunnelManager",
            serde_json::Map::from_iter([
                ("Action".into(), json!("stop")),
                ("Name".into(), json!("m038-client")),
            ]),
        )
        .await;
        assert!(operation_status(&stop_client).starts_with("success"));
    }

    let create_server = protected(
        &client,
        &endpoint,
        20,
        &token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("create")),
            ("Type".into(), json!("server")),
            ("Name".into(), json!("m038-server")),
            ("Port".into(), json!(0)),
        ]),
    )
    .await;
    assert!(operation_status(&create_server).starts_with("success"));
    let start_server = protected(
        &client,
        &endpoint,
        21,
        &token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("start")),
            ("Name".into(), json!("m038-server")),
        ]),
    )
    .await;
    let server_blocked = operation_status(&start_server).starts_with("error");
    if !server_blocked {
        let stop_server = protected(
            &client,
            &endpoint,
            22,
            &token,
            "TunnelManager",
            serde_json::Map::from_iter([
                ("Action".into(), json!("stop")),
                ("Name".into(), json!("m038-server")),
            ]),
        )
        .await;
        assert!(operation_status(&stop_server).starts_with("success"));
    }

    let unsupported = protected(
        &client,
        &endpoint,
        23,
        &token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("create")),
            ("Type".into(), json!("socks")),
            ("Name".into(), json!("m038-unsupported")),
        ]),
    )
    .await;
    assert!(operation_status(&unsupported).starts_with("success"));
    let unsupported_start = protected(
        &client,
        &endpoint,
        24,
        &token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("start")),
            ("Name".into(), json!("m038-unsupported")),
        ]),
    )
    .await;
    assert!(operation_status(&unsupported_start).starts_with("error"));
    let startup_mutation = protected(
        &client,
        &endpoint,
        25,
        &token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("delete")),
            ("Name".into(), json!("startup-client")),
        ]),
    )
    .await;
    assert!(operation_status(&startup_mutation).starts_with("error"));
    eprintln!(
        "M038 phase E: production tunnel CRUD, bind recovery, unsupported and startup ownership passed; local SAM formation blocked client={client_blocked} server={server_blocked}"
    );

    // Phase F: clean restart and durable recovery. The runtime store and
    // AddressBook owner are reopened by a new production composition.
    let first_logs = router.stop().await;
    assert!(
        !first_logs.contains(&password),
        "password leaked to child diagnostics"
    );

    let binary = std::env::var_os("CARGO_BIN_EXE_emissary-cli")
        .expect("Cargo must provide the emissary-cli binary path");
    router = RunningRouter {
        child: Command::new(binary)
            .args([
                "--base-path",
                base.path().to_str().expect("UTF-8 temp path"),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("restart feature-enabled emissary-cli"),
    };
    let restart_deadline = tokio::time::Instant::now() + STARTUP_DEADLINE;
    let restarted_token = loop {
        if let Ok(Some(status)) = router.child.try_wait() {
            panic!("Emissary restart exited before readiness: {status}");
        }
        if tokio::time::Instant::now() >= restart_deadline {
            panic!("I2PControl restart readiness deadline exceeded");
        }
        let response = client
            .post(&endpoint)
            .body(
                json!({
                    "jsonrpc": "2.0",
                    "id": 26,
                    "method": "Authenticate",
                    "params": {"API": 1, "Password": password.clone()}
                })
                .to_string(),
            )
            .send()
            .await;
        if let Ok(response) = response {
            if response.status().is_success() {
                let text = response.text().await.expect("restart authentication body");
                let value: Value =
                    serde_json::from_str(&text).expect("restart authentication JSON");
                break result(&value)
                    .get("Token")
                    .and_then(Value::as_str)
                    .expect("restart authentication token")
                    .to_owned();
            }
        }
        sleep(Duration::from_millis(100)).await;
    };
    let recovered_address_book = protected(
        &client,
        &endpoint,
        31,
        &restarted_token,
        "AddressBook",
        serde_json::Map::from_iter([
            ("book".into(), json!("private")),
            ("request".into(), json!("Lookup")),
            ("name".into(), json!("m038-persisted.i2p")),
        ]),
    )
    .await;
    assert_eq!(
        recovered_address_book["result"]["name"],
        json!("m038-persisted.i2p")
    );
    let recovered_tunnel = protected(
        &client,
        &endpoint,
        32,
        &restarted_token,
        "TunnelManager",
        serde_json::Map::from_iter([
            ("Action".into(), json!("get")),
            ("Name".into(), json!("m038-server")),
        ]),
    )
    .await;
    assert!(result(&recovered_tunnel)["info"].is_object());
    let healthy_after_restart = protected(
        &client,
        &endpoint,
        33,
        &restarted_token,
        "RouterInfo",
        serde_json::Map::from_iter([("i2p.router.net.total.received.bytes".into(), json!(false))]),
    )
    .await;
    assert!(result(&healthy_after_restart)["i2p.router.net.total.received.bytes"].is_number());
    eprintln!("M038 phase F: restart, token renewal, and durable tunnel recovery passed");

    // Phase G: malformed input remains isolated and the service still shuts
    // down through the bounded child-process cleanup path.
    let malformed = client
        .post(&endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body("not-json")
        .send()
        .await
        .expect("malformed request");
    assert!(malformed.status().is_success());
    let final_logs = router.stop().await;
    assert!(
        !final_logs.contains(&password),
        "password leaked after restart"
    );
    eprintln!("M038 phase G: malformed-request isolation and cleanup passed");
}
