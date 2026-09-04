//! M129 non-loopback managed-TLS fail-closed guards.
//!
//! Proves the managed self-signed identity is loopback-only: every
//! non-loopback bind (including wildcard/unspecified) requires complete
//! explicit certificate + private-key configuration, rejection happens
//! before listener/task/managed-file side effects, loopback managed TLS
//! stays operational with a loopback-only SAN set, explicit TLS never
//! falls back to managed or plaintext, production changes stay under
//! `i2pcontrol`, and the Proposal matrix is unchanged.

#![cfg(feature = "i2pcontrol")]

use std::{path::Path, process::Command};

use emissary_cli::i2pcontrol::{
    server::{I2pControlConfig, ServerInitContext},
    tls::TlsConfig,
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

fn config(bind: &str, cert: bool, key: bool) -> I2pControlConfig {
    I2pControlConfig {
        enabled: true,
        bind: bind.parse().unwrap(),
        password: "m129-test-password".to_string(),
        tls: TlsConfig {
            certificate: cert.then(|| "/operator/cert.pem".into()),
            private_key: key.then(|| "/operator/key.pem".into()),
        },
    }
}

#[test]
fn loopback_binds_accept_managed_and_explicit() {
    for bind in ["127.0.0.1:7650", "[::1]:7650"] {
        assert!(
            config(bind, false, false).validate().is_ok(),
            "loopback managed must stay allowed: {bind}"
        );
        assert!(
            config(bind, true, true).validate().is_ok(),
            "loopback explicit must stay allowed: {bind}"
        );
    }
}

#[test]
fn non_loopback_and_wildcard_reject_managed_and_partial() {
    for bind in [
        "192.0.2.10:7650",
        "203.0.113.7:7650",
        "[2001:db8::1]:7650",
        "0.0.0.0:7650",
        "[::]:7650",
    ] {
        for (cert, key) in [(false, false), (true, false), (false, true)] {
            let err = config(bind, cert, key).validate().unwrap_err();
            let message = err.to_string();
            assert!(
                message.contains("non-loopback") && message.contains("explicit"),
                "must state the explicit-material requirement ({bind} cert={cert} key={key}): {message}"
            );
            assert!(
                message.contains("loopback-only"),
                "must state managed identity is loopback-only: {message}"
            );
        }
        assert!(
            config(bind, true, true).validate().is_ok(),
            "complete explicit material must pass validation: {bind}"
        );
    }
}

#[test]
fn rejection_runs_before_tls_generation_and_listener_bind() {
    // Static ordering guard: validate() must precede TLS setup, directory
    // creation, and listener bind inside init_server. If a future refactor
    // reorders init_server, this fails loud.
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    let init_start = server.find("pub async fn init_server").expect("init_server");
    let init = &server[init_start..];
    let validate = init.find("config.validate()").expect("validate call");
    let tls = init.find("build_tls_config").expect("TLS setup");
    let bind = init.find("TcpListener::bind").expect("listener bind");
    let addressbooks = init.find("addressbooks").expect("store setup");
    assert!(
        validate < tls && validate < bind && validate < addressbooks,
        "validation must precede TLS, store setup, and bind"
    );

    // The validator itself must gate on loopback state plus complete
    // explicit material, not merely warn.
    let validate_fn = server
        .find("pub fn validate(&self)")
        .map(|start| &server[start..start + 2000])
        .expect("validate body");
    assert!(
        validate_fn.contains("is_loopback"),
        "validate must branch on loopback state"
    );
    assert!(
        validate_fn.contains("has_complete_explicit_material"),
        "validate must require complete explicit material"
    );
}

#[test]
fn validation_is_fail_closed_not_warn_only() {
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    let validate_start = server.find("pub fn validate(&self)").expect("validate");
    let validate = &server[validate_start..validate_start + 2500];
    // Rejection must be an Err return on the non-loopback incomplete path.
    assert!(
        validate.contains("return Err"),
        "non-loopback managed configuration must return an error"
    );
    // The warning may remain for the allowed explicit-remote path, but it
    // must not be the sole control: rejection precedes it.
    let reject = validate.find("return Err").expect("reject");
    let warn = validate.find("tracing::warn").expect("explicit-remote warning");
    assert!(
        reject < warn,
        "rejection must precede any non-loopback warning"
    );
}

#[tokio::test]
async fn rejected_remote_creates_no_managed_files_and_binds_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let port = {
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        probe.local_addr().unwrap().port()
    };
    let bind = format!("0.0.0.0:{port}");
    let password = "m129-no-leak-password".to_string();
    let failing = I2pControlConfig {
        enabled: true,
        bind: bind.parse().unwrap(),
        password: password.clone(),
        tls: TlsConfig {
            certificate: None,
            private_key: None,
        },
    };
    let ctx = ServerInitContext::new("test-id".to_string(), vec![]);
    let err = match emissary_cli::i2pcontrol::server::init_server(&failing, tmp.path(), ctx).await {
        Ok(_) => panic!("non-loopback managed startup must fail"),
        Err(err) => err,
    };
    let message = err.to_string();
    assert!(
        message.contains("non-loopback"),
        "startup error must state the remote-material requirement: {message}"
    );
    assert!(
        !message.contains(&password),
        "startup error must not echo the password"
    );
    assert!(
        !message.contains("Token"),
        "startup error must not mention token material"
    );

    // No managed certificate directory or file may have been created.
    assert!(
        !tmp.path().join("i2pcontrol-certs").exists(),
        "rejected startup must not create managed TLS state"
    );
    // No address-book or tunnel store side effects either (validation is
    // the earliest boundary).
    assert!(
        !tmp.path().join("addressbooks").exists(),
        "rejected startup must not create store state"
    );
    assert!(
        !tmp.path().join("tunnels").exists(),
        "rejected startup must not create store state"
    );

    // The configured port must remain free: nothing was bound and no
    // service task was started (init_server returns Err, so no instance
    // exists to serve).
    let rebound = std::net::TcpListener::bind(format!("127.0.0.1:{port}"));
    assert!(
        rebound.is_ok(),
        "rejected startup must not hold the configured port"
    );
}

#[tokio::test]
async fn rejected_remote_does_not_mutate_existing_managed_material() {
    let tmp = tempfile::tempdir().unwrap();
    // Establish existing managed loopback material first.
    let (certs_before, _) =
        emissary_cli::i2pcontrol::tls::load_or_generate_managed_tls(tmp.path()).unwrap();
    let cert_dir = tmp.path().join("i2pcontrol-certs");
    let cert_bytes = std::fs::read(cert_dir.join("cert.pem")).unwrap();
    let key_bytes = std::fs::read(cert_dir.join("key.pem")).unwrap();
    assert!(!certs_before.is_empty());

    let failing = I2pControlConfig {
        enabled: true,
        bind: "192.0.2.10:7650".parse().unwrap(),
        password: "testpass".to_string(),
        tls: TlsConfig {
            certificate: None,
            private_key: None,
        },
    };
    let ctx = ServerInitContext::new("test-id".to_string(), vec![]);
    assert!(
        emissary_cli::i2pcontrol::server::init_server(&failing, tmp.path(), ctx)
            .await
            .is_err(),
        "must reject"
    );

    assert_eq!(
        std::fs::read(cert_dir.join("cert.pem")).unwrap(),
        cert_bytes,
        "rejected startup must not regenerate the managed certificate"
    );
    assert_eq!(
        std::fs::read(cert_dir.join("key.pem")).unwrap(),
        key_bytes,
        "rejected startup must not regenerate the managed key"
    );
}

#[test]
fn managed_san_set_stays_loopback_only_without_remote_synthesis() {
    let tls = production_section("emissary-cli/src/i2pcontrol/tls.rs");
    // The generated identity must remain exactly the loopback set.
    for san in ["\"localhost\"", "\"127.0.0.1\"", "\"::1\""] {
        assert!(
            tls.contains(san),
            "managed certificate must keep loopback SAN {san}"
        );
    }
    // No automatic remote identity synthesis: no wildcard, host-derived,
    // DNS-resolved, interface-enumerated, or request-controlled names.
    for forbidden in [
        "\"*\"",
        "get_if_addrs",
        "getifaddrs",
        "interfaces()",
        "lookup_host",
        "to_socket_addrs",
        "gethostname",
        "local_ip",
        "request-controlled",
    ] {
        assert!(
            !tls.contains(forbidden),
            "managed TLS must not synthesize remote identities: found {forbidden}"
        );
    }
    // No trust-store or client-verification weakening to make remote
    // managed service appear valid.
    for forbidden in [
        "danger_accept_invalid",
        "dangerous(",
        "add_server_trust_anchors",
        "install_default",
        "with_custom_certificate_verifier",
    ] {
        // The production section (non-test) must not weaken verification.
        assert!(
            !tls.contains(forbidden),
            "managed TLS must not weaken client verification: found {forbidden}"
        );
    }
}

#[tokio::test]
async fn managed_identity_fails_remote_verification_but_serves_loopback() {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio_rustls::rustls::{
        crypto::ring, pki_types::ServerName, ClientConfig, RootCertStore, ServerConfig,
    };
    use tokio_rustls::{TlsAcceptor, TlsConnector};

    let dir = tempfile::tempdir().unwrap();
    let (certs, key) =
        emissary_cli::i2pcontrol::tls::load_or_generate_managed_tls(dir.path()).unwrap();
    let server_config = Arc::new(
        ServerConfig::builder_with_provider(Arc::new(ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(certs.clone(), key)
            .unwrap(),
    );
    let mut roots = RootCertStore::empty();
    roots.add(certs[0].clone()).unwrap();
    let client_config = Arc::new(
        ClientConfig::builder_with_provider(Arc::new(ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    );

    async fn handshake(
        server_config: &Arc<ServerConfig>,
        client_config: &Arc<ClientConfig>,
        name: ServerName<'static>,
    ) -> Result<(), String> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let acceptor = TlsAcceptor::from(Arc::clone(server_config));
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            acceptor.accept(stream).await.map(|_| ()).map_err(|e| e.to_string())
        });
        let connector = TlsConnector::from(Arc::clone(client_config));
        let client = connector
            .connect(name, tokio::net::TcpStream::connect(address).await.unwrap())
            .await
            .map(|_| ())
            .map_err(|e| e.to_string());
        let _ = server.await;
        client
    }

    // Loopback identities remain usable.
    for name in [
        ServerName::try_from("localhost").unwrap(),
        ServerName::IpAddress(IpAddr::V4(Ipv4Addr::LOCALHOST).into()),
        ServerName::IpAddress(IpAddr::V6(Ipv6Addr::LOCALHOST).into()),
    ] {
        handshake(&server_config, &client_config, name)
            .await
            .expect("loopback identity must validate");
    }

    // A remote address must not validate against the managed loopback
    // identity with a standards-validating client.
    let remote = handshake(
        &server_config,
        &client_config,
        ServerName::IpAddress(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10)).into()),
    )
    .await;
    assert!(
        remote.is_err(),
        "managed loopback identity must not validate for a remote address"
    );
}

#[tokio::test]
async fn explicit_tls_never_falls_back_to_managed_or_plaintext() {
    // Partial explicit material fails at load (no managed fallback).
    let partial = TlsConfig {
        certificate: Some("/operator/cert.pem".into()),
        private_key: None,
    };
    let dir = tempfile::tempdir().unwrap();
    let err = emissary_cli::i2pcontrol::tls::build_tls_config(&partial, dir.path())
        .expect_err("partial explicit TLS must fail");
    assert!(
        !dir.path().join("i2pcontrol-certs").exists(),
        "explicit failure must not create managed state"
    );
    assert!(
        err.to_string().contains("Private key") || err.to_string().contains("Certificate"),
        "explicit failure must name the missing material: {err}"
    );

    // Complete-but-unreadable explicit material also fails without
    // managed fallback.
    let missing = TlsConfig {
        certificate: Some(dir.path().join("no-cert.pem")),
        private_key: Some(dir.path().join("no-key.pem")),
    };
    let err = emissary_cli::i2pcontrol::tls::build_tls_config(&missing, dir.path())
        .expect_err("missing explicit files must fail");
    assert!(
        !dir.path().join("i2pcontrol-certs").exists(),
        "missing explicit files must not create managed state: {err}"
    );

    // The production TLS boundary stays TLS-only with safe defaults and
    // no plaintext fallback after any failure.
    let tls = production_section("emissary-cli/src/i2pcontrol/tls.rs");
    assert!(
        tls.contains("with_safe_default_protocol_versions"),
        "safe rustls defaults must be retained"
    );
    assert!(
        tls.contains("with_no_client_auth"),
        "no-mTLS posture must be retained (no scope expansion)"
    );
    let server = production_section("emissary-cli/src/i2pcontrol/server.rs");
    assert!(
        server.contains("TlsAcceptor"),
        "serving must stay behind the TLS acceptor"
    );
    assert!(
        server.contains("TLS handshake failed") || server.contains("TLS handshake timed out"),
        "handshake failures must be contained, not downgraded"
    );
}

#[tokio::test]
async fn explicit_material_serves_non_loopback_bind() {
    // Controlled local topology: an explicit certificate serves a
    // wildcard (non-loopback) bind; the client reaches it over loopback
    // with the operator-provided trust anchor. Verification stays enabled
    // (no danger_accept_invalid_certs).
    let tmp = tempfile::tempdir().unwrap();
    let (_manager, control) =
        emissary_cli::i2pcontrol::address_book_runtime::new_controlled_manager(
            tmp.path().to_owned(),
            emissary_cli::config::AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;

    // Operator-provided identity covering the client-visible endpoint.
    let key_pair = rcgen::KeyPair::generate().unwrap();
    let mut params =
        rcgen::CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()])
            .unwrap();
    params.distinguished_name.push(rcgen::DnType::CommonName, "M129 explicit test");
    let cert = params.self_signed(&key_pair).unwrap();
    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();
    let cert_path = tmp.path().join("operator-cert.pem");
    let key_path = tmp.path().join("operator-key.pem");
    std::fs::write(&cert_path, cert_pem.as_bytes()).unwrap();
    std::fs::write(&key_path, key_pem.as_bytes()).unwrap();

    let explicit_port = {
        let probe = std::net::TcpListener::bind("127.0.0.1:0").expect("probe port");
        probe.local_addr().expect("probe addr").port()
    };
    let config = I2pControlConfig {
        enabled: true,
        bind: format!("0.0.0.0:{explicit_port}").parse().unwrap(),
        password: "m129-explicit".to_string(),
        tls: TlsConfig {
            certificate: Some(cert_path.clone()),
            private_key: Some(key_path.clone()),
        },
    };
    let ctx =
        ServerInitContext::new("test-id".to_string(), vec![]).with_address_book_handle(control);
    let instance = emissary_cli::i2pcontrol::server::init_server(&config, tmp.path(), ctx)
        .await
        .expect("explicit non-loopback startup must succeed");
    let bound = instance.bind();
    assert!(
        !bound.ip().is_loopback(),
        "test must exercise a non-loopback bind: {bound}"
    );

    // Serve in the background, then authenticate over verified TLS via
    // the loopback route to the wildcard listener.
    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
    let serve_handle = tokio::spawn(async move {
        let _ = emissary_cli::i2pcontrol::server::serve(instance, shutdown_tx.subscribe()).await;
    });
    // Give the accept loop a moment to start.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let trust_anchor =
        reqwest::Certificate::from_pem(cert_pem.as_bytes()).expect("explicit cert PEM");
    let client = reqwest::Client::builder()
        .add_root_certificate(trust_anchor)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("verified TLS client");
    let endpoint = format!("https://localhost:{}/", bound.port());
    let response = client
        .post(&endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "Authenticate",
                "params": {"API": 1, "Password": "m129-explicit"},
            })
            .to_string(),
        )
        .send()
        .await
        .expect("verified TLS request to explicit non-loopback listener");
    assert!(
        response.status().is_success(),
        "explicit non-loopback TLS must serve authenticated requests"
    );
    let text = response.text().await.expect("response body");
    let value: serde_json::Value = serde_json::from_str(&text).expect("response JSON");
    assert!(
        value.get("result").is_some(),
        "explicit non-loopback Authenticate must succeed: {value}"
    );
    serve_handle.abort();
}

#[test]
fn production_changes_stay_under_i2pcontrol() {
    // M129 authorizes no core/util/router/frontend/dependency change.
    // Diff the reopened-line planning baseline against the working tree
    // for production source outside the I2PControl boundary.
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
        "M129 must not change core/util/config/dependency paths: {changed:?}"
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
            "M129 production change outside i2pcontrol: {path}"
        );
    }
}

#[test]
fn proposal_matrix_unchanged_by_tls_fail_closed() {
    let matrix: toml::Value = std::fs::read_to_string(
        workspace_root()
            .join("plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"),
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
