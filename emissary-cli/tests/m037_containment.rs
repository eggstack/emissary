//! M037 static containment guards.

#![cfg(feature = "i2pcontrol")]

use std::{collections::BTreeSet, path::Path};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BoundaryManifest {
    ownership: Ownership,
}

#[derive(Debug, Deserialize)]
struct Ownership {
    i2pcontrol: Vec<String>,
    adapter: Vec<String>,
    approved_core_passive_hook: Vec<String>,
    tests_and_docs: Vec<String>,
    prohibited_production_prefixes: Vec<String>,
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn source(path: &str) -> String {
    std::fs::read_to_string(workspace_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn manifest() -> BoundaryManifest {
    let path = workspace_root()
        .join("plans/implementation/i2pcontrol-proposal-170/037-changed-path-boundary.toml");
    toml::from_str(&std::fs::read_to_string(path).expect("M037 boundary manifest"))
        .expect("valid M037 boundary manifest")
}

#[test]
fn approved_changed_path_manifest_is_enforced() {
    let manifest = manifest().ownership;
    let mut seen = BTreeSet::new();
    for path in manifest
        .i2pcontrol
        .iter()
        .chain(&manifest.adapter)
        .chain(&manifest.approved_core_passive_hook)
        .chain(&manifest.tests_and_docs)
    {
        assert!(seen.insert(path), "duplicate M037 boundary path: {path}");
        assert!(
            workspace_root().join(path).exists(),
            "missing boundary path: {path}"
        );
    }
    for path in &manifest.approved_core_passive_hook {
        assert!(path.starts_with("emissary-core/src/"));
        assert!(!path.contains("crypto/") && !path.contains("transport/"));
    }
    for prefix in &manifest.prohibited_production_prefixes {
        assert!(!manifest.approved_core_passive_hook.iter().any(|path| path.starts_with(prefix)));
    }
}

#[test]
fn original_runtime_modules_do_not_depend_on_jsonrpc_handlers() {
    for path in [
        "emissary-cli/src/address_book.rs",
        "emissary-cli/src/i2pcontrol/address_book_runtime.rs",
    ] {
        let text = source(path);
        assert!(
            !text.contains("JsonRpc"),
            "{path} imports JSON-RPC handler types"
        );
        assert!(
            !text.contains("jsonrpc"),
            "{path} imports JSON-RPC handlers"
        );
    }
}

#[test]
fn core_hook_exposes_no_live_or_secret_types() {
    let sam = source("emissary-core/src/sam/mod.rs");
    let event = sam
        .split("pub enum SamObservationEvent")
        .nth(1)
        .and_then(|text| text.split("pub struct SamObservationHookError").next())
        .expect("SAM event declaration");
    for forbidden in [
        "SamSocket",
        "TcpStream",
        "SigningPrivateKey",
        "StaticPrivateKey",
        "Sender<",
        "Receiver<",
    ] {
        assert!(
            !event.contains(forbidden),
            "SAM hook event exposes {forbidden}"
        );
    }
    assert!(sam.contains("fn sanitized_peer"));
    assert!(sam.contains("fn sanitized_text"));
}

#[test]
fn unsupported_tunnel_backends_remain_resource_free() {
    let source = source("emissary-cli/src/i2pcontrol/backends/unsupported.rs");
    assert!(source.contains("does not allocate") || source.contains("must not allocate"));
    assert!(!source.contains("TcpListener::bind"));
    assert!(!source.contains("spawn("));
}
