//! M163 post-M152 corrective registration guard.
//!
//! This is intentionally separate from the long historical M062 helper chain.
//! It binds the current registration to the exact M163 three-path I2PControl
//! budget and prevents deferred M164/M165 work from being mistaken for active
//! production authority.

#![cfg(feature = "i2pcontrol")]

use std::{collections::BTreeSet, path::Path};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(workspace_root().join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

#[test]
fn m163_current_registration_is_exact_and_zero_dependency() {
    let raw = read("plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml");
    let manifest: toml::Value = toml::from_str(&raw).expect("valid M062 TOML");
    let registration = manifest
        .get("current_registration")
        .and_then(toml::Value::as_table)
        .expect("M062 current_registration table");

    assert_eq!(
        registration.get("milestone").and_then(toml::Value::as_str),
        Some("M163 (registered / dependency-ready)")
    );

    let paths = registration
        .get("production_paths")
        .and_then(toml::Value::as_array)
        .expect("M163 production_paths")
        .iter()
        .map(|value| value.as_str().expect("production path string"))
        .collect::<BTreeSet<_>>();
    let expected = [
        "emissary-cli/src/i2pcontrol/tunnel_manager.rs",
        "emissary-cli/src/i2pcontrol/stores/tunnel_store.rs",
        "emissary-cli/src/i2pcontrol/stores/generation_store.rs",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(paths, expected, "M163 production budget must remain exact");

    for key in ["new_files", "new_direct_dependencies", "manifest_changes"] {
        assert!(
            registration
                .get(key)
                .and_then(toml::Value::as_array)
                .is_some_and(Vec::is_empty),
            "M163 {key} must remain empty"
        );
    }
    assert_eq!(
        registration.get("lockfile_change").and_then(toml::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        registration.get("yosemite_change").and_then(toml::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        registration
            .get("i2pcontrol_source_change")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
}

#[test]
fn m163_is_sole_registered_handoff_and_successors_are_deferred() {
    let registry = read("plans/registry.md");
    assert!(registry.contains("M163 is the **sole registered Proposal-170 implementation handoff**"));
    assert!(registry.contains("M164 may be registered only after a clean M163 closure"));
    assert!(registry.contains("M165 may be registered only after clean M163+M164 closures"));

    let m163 = read(
        "plans/implementation/i2pcontrol-proposal-170/163-blocked-leaseset-state-persistence-corrective.md",
    );
    assert!(m163.contains("Status: **registered / dependency-ready**"));
    assert!(m163.contains("Promotion budget: **zero Proposal cells**"));

    let m164 = read(
        "plans/implementation/i2pcontrol-proposal-170/164-sam-invalid-command-secret-redaction-corrective.md",
    );
    assert!(m164.contains("Status: **deferred / unregistered**"));
    assert!(m164.contains("Hard dependency: M163 closure"));

    let m165 = read(
        "plans/implementation/i2pcontrol-proposal-170/165-post-corrective-current-head-requalification.md",
    );
    assert!(m165.contains("Status: **deferred / unregistered**"));
    assert!(m165.contains("Hard dependencies: M163 closure + M164 closure"));
}

#[test]
fn m163_budget_stays_inside_i2pcontrol_policy_root() {
    let raw = read("plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml");
    let manifest: toml::Value = toml::from_str(&raw).expect("valid M062 TOML");
    let paths = manifest["current_registration"]["production_paths"]
        .as_array()
        .expect("M163 production paths");
    assert!(paths.iter().all(|path| {
        path.as_str()
            .is_some_and(|path| path.starts_with("emissary-cli/src/i2pcontrol/"))
    }));

    let m061 = read("plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml");
    let boundary: toml::Value = toml::from_str(&m061).expect("valid M061 TOML");
    assert_eq!(
        boundary.get("policy_root").and_then(toml::Value::as_str),
        Some("emissary-cli/src/i2pcontrol/")
    );
}
