//! M061 current containment authority and static guards.

#![cfg(feature = "i2pcontrol")]

use std::{collections::BTreeSet, path::Path, process::Command};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BoundaryManifest {
    version: u32,
    fork_baseline: String,
    upstream_baseline: String,
    policy_root: String,
    allowed: Allowed,
    prohibited: Prohibited,
    evidence: Vec<PathEvidence>,
}

#[derive(Debug, Deserialize)]
struct Allowed {
    composition: Vec<String>,
    original_runtime_adapters: Vec<String>,
    core_inspection: Vec<String>,
    core_owner_hooks: Vec<String>,
    build_feature_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Prohibited {
    production_prefixes: Vec<String>,
    forbidden_terms_outside_policy_root: Vec<String>,
    forbidden_terms_in_core_declarations: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PathEvidence {
    path: String,
    owner: String,
    purpose: String,
    consumer: String,
    why_upstream_insufficient: String,
    sensitivity: String,
    seam: String,
    reference: String,
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn source(path: &str) -> String {
    std::fs::read_to_string(workspace_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn production_source(path: &str) -> String {
    source(path).split("#[cfg(test)]").next().unwrap_or_default().to_owned()
}

fn manifest() -> BoundaryManifest {
    let path = workspace_root()
        .join("plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml");
    toml::from_str(&std::fs::read_to_string(path).expect("M061 boundary manifest"))
        .expect("valid M061 boundary manifest")
}

fn allowed_paths(manifest: &BoundaryManifest) -> BTreeSet<String> {
    manifest
        .allowed
        .composition
        .iter()
        .chain(&manifest.allowed.original_runtime_adapters)
        .chain(&manifest.allowed.core_inspection)
        .chain(&manifest.allowed.core_owner_hooks)
        .chain(&manifest.allowed.build_feature_paths)
        .cloned()
        .collect()
}

fn changed_non_policy_paths(manifest: &BoundaryManifest) -> BTreeSet<String> {
    let output = Command::new("git")
        .args([
            "diff",
            "--name-only",
            &manifest.upstream_baseline,
            "--",
            "emissary-cli/src",
            "emissary-core/src",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("git diff");
    assert!(output.status.success(), "pinned upstream comparison failed");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|path| !path.starts_with(&manifest.policy_root))
        .map(str::to_owned)
        .collect()
}

#[test]
fn current_changed_paths_match_the_exact_manifest() {
    let manifest = manifest();
    assert_eq!(manifest.version, 1);
    assert_eq!(
        manifest.fork_baseline,
        "c958c4d998b1abde9ace730b4bdadcf5a838afc6"
    );
    assert_eq!(
        changed_non_policy_paths(&manifest),
        allowed_paths(&manifest),
        "a new or removed non-policy production path requires a manifest update"
    );
}

#[test]
fn every_allowed_path_is_exact_and_has_owner_evidence() {
    let manifest = manifest();
    let allowed = allowed_paths(&manifest);
    let evidence = manifest
        .evidence
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        evidence, allowed,
        "manifest evidence must cover each allowed path once"
    );
    for path in &allowed {
        assert!(
            !path.ends_with('/'),
            "allowed path is a broad prefix: {path}"
        );
        assert!(
            workspace_root().join(path).is_file(),
            "missing allowed path: {path}"
        );
    }
    for entry in &manifest.evidence {
        for field in [
            &entry.owner,
            &entry.purpose,
            &entry.consumer,
            &entry.why_upstream_insufficient,
            &entry.sensitivity,
            &entry.seam,
            &entry.reference,
        ] {
            assert!(
                !field.trim().is_empty(),
                "empty evidence for {}",
                entry.path
            );
        }
    }
}

#[test]
fn high_sensitivity_core_paths_are_individually_named() {
    let manifest = manifest();
    let allowed = allowed_paths(&manifest);
    for path in allowed.iter().filter(|path| path.starts_with("emissary-core/src/")) {
        assert!(
            !path.ends_with('/'),
            "core path uses a broad prefix: {path}"
        );
        assert!(!path.contains("*"), "core path uses a glob: {path}");
    }
    for prefix in &manifest.prohibited.production_prefixes {
        assert!(
            allowed.iter().all(|path| !path.starts_with(prefix)),
            "prohibited production prefix was allowed: {prefix}"
        );
    }
}

#[test]
fn policy_terms_do_not_leak_into_non_policy_production_paths() {
    let manifest = manifest();
    for path in allowed_paths(&manifest)
        .into_iter()
        .filter(|path| !path.starts_with(&manifest.policy_root))
    {
        let contents = production_source(&path);
        for term in &manifest.prohibited.forbidden_terms_outside_policy_root {
            assert!(
                !contents.contains(term),
                "{path} contains prohibited term {term}"
            );
        }
    }
}

#[test]
fn inspection_and_sam_declarations_expose_only_public_bounded_facts() {
    let manifest = manifest();
    let sam = source("emissary-core/src/sam/mod.rs");
    let event = sam
        .split("pub enum SamObservationEvent")
        .nth(1)
        .and_then(|text| text.split("pub struct SamObservationHookError").next())
        .expect("SAM event declaration");
    for term in &manifest.prohibited.forbidden_terms_in_core_declarations {
        assert!(
            !event.contains(term),
            "SAM event exposes forbidden type {term}"
        );
    }

    let inspection = production_source("emissary-core/src/inspection.rs");
    for term in &manifest.prohibited.forbidden_terms_in_core_declarations {
        assert!(
            !inspection.contains(term),
            "inspection exposes forbidden type {term}"
        );
    }
    assert!(sam.contains("fn sanitized_peer"));
    assert!(sam.contains("fn sanitized_text"));
}

#[test]
fn unsupported_tunnel_backends_remain_resource_free() {
    let source = production_source("emissary-cli/src/i2pcontrol/backends/unsupported.rs");
    assert!(
        source.contains("does not allocate")
            || source.contains("must not allocate")
            || source.contains("resource-free")
    );
    assert!(!source.contains("TcpListener::bind"));
    assert!(!source.contains("spawn("));
}

#[test]
fn negative_guard_inventory_covers_the_containment_contract() {
    let manifest = manifest();
    for term in ["Proposal 170", "JsonRpc", "jsonrpc", "control-state.json"] {
        assert!(
            manifest
                .prohibited
                .forbidden_terms_outside_policy_root
                .iter()
                .any(|candidate| candidate == term),
            "missing policy-leak guard for {term}"
        );
    }
    for term in ["TcpStream", "SigningPrivateKey", "Sender<", "Receiver<"] {
        assert!(
            manifest
                .prohibited
                .forbidden_terms_in_core_declarations
                .iter()
                .any(|candidate| candidate == term),
            "missing live/secret boundary guard for {term}"
        );
    }
}
