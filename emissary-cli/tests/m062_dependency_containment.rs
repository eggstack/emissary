//! M062 dependency-surface containment guards.
//!
//! Semantic TOML ownership checks for the `subtle` direct dependency used by
//! I2PControl authentication. The test enforces that:
//!
//! * the root `[workspace.dependencies]` table does not own an I2PControl-only `subtle`
//!   declaration;
//! * `emissary-cli` declares `subtle` as `optional = true` with default features disabled;
//! * only the `i2pcontrol` feature activates the optional `subtle` dependency;
//! * `Cargo.lock` is unchanged relative to the M062 fork baseline.
//!
//! The guard is intentionally semantic rather than comment-text based so that a
//! future regression in dependency ownership fails closed.

#![cfg(feature = "i2pcontrol")]

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DependencyManifest {
    version: u32,
    fork_baseline: String,
    upstream_baseline: String,
    direct_dependencies: toml::Value,
    forbidden_activations: toml::Value,
    workspace_dependencies: toml::Value,
    lockfile: Lockfile,
    allowed_production_paths: AllowedPaths,
    prohibited_production_paths: ProhibitedPaths,
    dependency_rule: DependencyRule,
    evidence: Vec<DependencyEvidence>,
}

#[derive(Debug, Deserialize)]
struct Lockfile {
    expected: String,
    baseline_commit: String,
}

#[derive(Debug, Deserialize)]
struct AllowedPaths {
    root_manifests: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProhibitedPaths {
    patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DependencyRule {
    direct_ownership: String,
    #[serde(default)]
    transitive_permitted: bool,
    #[serde(default)]
    crate_name_absence_not_required: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DependencyEvidence {
    #[serde(rename = "crate")]
    crate_name: String,
    #[serde(default)]
    direct_consumer: String,
    #[serde(default)]
    feature_gated_consumer: bool,
    #[serde(default)]
    feature: Option<String>,
    #[serde(default)]
    constant_time_primitive: Option<String>,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    reference: String,
    #[serde(default)]
    non_i2pcontrol_workspace_consumer: Option<bool>,
    #[serde(default)]
    non_i2pcontrol_literal_consumer: String,
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn manifest_path() -> PathBuf {
    workspace_root()
        .join("plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml")
}

fn load_manifest() -> DependencyManifest {
    let raw = std::fs::read_to_string(manifest_path()).expect("M062 dependency manifest");
    toml::from_str(&raw).expect("valid M062 dependency manifest")
}

#[test]
fn manifest_is_well_formed_and_self_consistent() {
    let manifest = load_manifest();
    assert_eq!(manifest.version, 1);
    assert_eq!(
        manifest.fork_baseline,
        "a70dd3ac82f12fbea1f8fba51e30a9e2e516650a"
    );
    assert_eq!(
        manifest.upstream_baseline,
        "9b43484a21d5a1291c4881cdae62a36c527f8c0f"
    );
    assert_eq!(manifest.lockfile.expected, "byte-identical to baseline");
    assert_eq!(
        manifest.lockfile.baseline_commit,
        "a70dd3ac82f12fbea1f8fba51e30a9e2e516650a"
    );
    assert!(
        manifest
            .allowed_production_paths
            .root_manifests
            .contains(&"Cargo.toml".to_owned()),
        "allowed production paths must include the root workspace manifest"
    );
    assert!(
        manifest
            .allowed_production_paths
            .root_manifests
            .contains(&"emissary-cli/Cargo.toml".to_owned()),
        "allowed production paths must include the emissary-cli package manifest"
    );
    assert!(
        manifest
            .prohibited_production_paths
            .patterns
            .iter()
            .any(|pattern| pattern == "emissary-cli/src/**"),
        "M062 must not modify emissary-cli/src/**"
    );
    assert!(
        manifest
            .prohibited_production_paths
            .patterns
            .iter()
            .any(|pattern| pattern == "emissary-core/**"),
        "M062 must not modify emissary-core/**"
    );
    assert!(
        manifest
            .prohibited_production_paths
            .patterns
            .iter()
            .any(|pattern| pattern == "emissary-util/**"),
        "M062 must not modify emissary-util/**"
    );
    assert!(!manifest.dependency_rule.direct_ownership.is_empty());
    assert!(
        manifest.dependency_rule.transitive_permitted,
        "manifest must record that transitive crate-name presence is permitted"
    );
    assert!(
        manifest.dependency_rule.crate_name_absence_not_required,
        "manifest must record that crate-name absence is not an acceptance gate"
    );
}

#[test]
fn root_workspace_does_not_declare_subtle() {
    let manifest = load_manifest();
    let entry = manifest
        .workspace_dependencies
        .get("subtle")
        .expect("manifest must declare the workspace expectation for subtle");
    let disposition = entry.as_str().expect("workspace expectation must be a string").to_owned();
    assert_eq!(
        disposition, "absent",
        "root workspace must not own an I2PControl-only subtle declaration"
    );

    let raw = std::fs::read_to_string(workspace_root().join("Cargo.toml"))
        .expect("root workspace manifest");
    let parsed: toml::Value = toml::from_str(&raw).expect("valid root workspace manifest");
    let workspace_deps = parsed
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .expect("workspace.dependencies table");
    assert!(
        workspace_deps.get("subtle").is_none(),
        "root Cargo.toml must not declare subtle in [workspace.dependencies]"
    );
}

#[test]
fn emissary_cli_owns_subtle_locally_as_optional_with_no_default_features() {
    let manifest = load_manifest();
    let raw = std::fs::read_to_string(workspace_root().join("emissary-cli/Cargo.toml"))
        .expect("emissary-cli manifest");
    let parsed: toml::Value = toml::from_str(&raw).expect("valid emissary-cli manifest");
    let deps = parsed
        .get("dependencies")
        .and_then(|deps| deps.get("subtle"))
        .expect("emissary-cli must declare subtle");

    let table = deps.as_table().expect("subtle dependency is a table");
    assert!(
        !table.contains_key("workspace"),
        "emissary-cli must own subtle locally; the entry must not be {{ workspace = true }}"
    );
    assert_eq!(
        table.get("version").and_then(|v| v.as_str()),
        Some("2.6.1"),
        "subtle version must remain 2.6.1"
    );
    assert_eq!(
        table.get("optional").and_then(|v| v.as_bool()),
        Some(true),
        "subtle must be declared optional = true"
    );
    assert_eq!(
        table.get("default-features").and_then(|v| v.as_bool()),
        Some(false),
        "subtle must keep default-features = false"
    );

    let manifest_entry = manifest
        .direct_dependencies
        .get("subtle")
        .expect("manifest must describe the subtle direct dependency");
    assert_eq!(
        manifest_entry.get("owning_feature").and_then(|v| v.as_str()),
        Some("i2pcontrol"),
        "manifest must record i2pcontrol as the owning feature for subtle"
    );
    assert_eq!(
        manifest_entry.get("default_features").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        manifest_entry.get("optional").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn only_i2pcontrol_activates_subtle() {
    let manifest = load_manifest();
    let raw = std::fs::read_to_string(workspace_root().join("emissary-cli/Cargo.toml"))
        .expect("emissary-cli manifest");
    let parsed: toml::Value = toml::from_str(&raw).expect("valid emissary-cli manifest");
    let features = parsed
        .get("features")
        .and_then(|features| features.as_table())
        .expect("features table");

    let i2pcontrol = features
        .get("i2pcontrol")
        .and_then(|value| value.as_array())
        .expect("i2pcontrol feature must exist");
    let activates_subtle = i2pcontrol
        .iter()
        .any(|entry| entry.as_str() == Some("dep:subtle") || entry.as_str() == Some("subtle"));
    assert!(
        activates_subtle,
        "i2pcontrol must explicitly activate the optional subtle dependency"
    );

    for forbidden_feature in ["default", "ui", "metrics"] {
        let forbidden_activations = manifest
            .forbidden_activations
            .get(forbidden_feature)
            .and_then(|value| value.as_array())
            .unwrap_or_else(|| panic!("forbidden_activations must list {forbidden_feature}"));
        assert!(
            forbidden_activations.is_empty(),
            "{forbidden_feature} must not activate subtle"
        );

        if let Some(value) = features.get(forbidden_feature) {
            if let Some(list) = value.as_array() {
                for entry in list {
                    let entry = entry.as_str().unwrap_or_default();
                    assert!(
                        entry != "dep:subtle" && entry != "subtle",
                        "{forbidden_feature} must not activate subtle"
                    );
                }
            }
        }
    }
}

#[test]
fn m061_source_boundary_files_remain_unchanged() {
    let manifest = load_manifest();
    let m061_boundary = workspace_root()
        .join("plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml");
    assert!(
        m061_boundary.is_file(),
        "M061 source boundary manifest must remain present"
    );

    let m061_test = workspace_root().join("emissary-cli/tests/m061_containment.rs");
    assert!(
        m061_test.is_file(),
        "M061 source-boundary guard must remain present"
    );

    let diff = Command::new("git")
        .args([
            "diff",
            "--name-only",
            &manifest.fork_baseline,
            "--",
            "plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml",
            "emissary-cli/tests/m061_containment.rs",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("git diff M061 boundary");
    assert!(diff.status.success(), "git diff failed");

    let changed = String::from_utf8_lossy(&diff.stdout);
    assert!(
        changed.trim().is_empty(),
        "M062 must not modify the retained M061 source boundary authority: {changed}"
    );
}

#[test]
fn lockfile_is_byte_identical_to_fork_baseline() {
    let manifest = load_manifest();
    let diff = Command::new("git")
        .args([
            "diff",
            "--name-only",
            &manifest.lockfile.baseline_commit,
            "--",
            "Cargo.lock",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("git diff Cargo.lock");
    assert!(diff.status.success(), "git diff Cargo.lock failed");

    let changed = String::from_utf8_lossy(&diff.stdout).trim().to_owned();
    assert!(
        changed.is_empty(),
        "M062 must not change Cargo.lock relative to the fork baseline; changed: {changed}"
    );
}

#[test]
fn allowed_production_paths_match_the_m062_budget() {
    let manifest = load_manifest();
    let m062_commit = workspace_root().join(".git").join("HEAD");
    assert!(m062_commit.is_file(), "repository must be a git checkout");

    let diff = Command::new("git")
        .args(["diff", "--name-only", &manifest.fork_baseline, "--"])
        .current_dir(workspace_root())
        .output()
        .expect("git diff m062 range");
    assert!(diff.status.success(), "git diff m062 range failed");

    let changed: Vec<String> =
        String::from_utf8_lossy(&diff.stdout).lines().map(str::to_owned).collect();

    for path in &changed {
        let permitted = manifest
            .allowed_production_paths
            .root_manifests
            .iter()
            .any(|allowed| allowed == path);
        assert!(
            permitted || is_authorized_planning_path(path),
            "M062 changed an unauthorized production path: {path}"
        );

        for pattern in &manifest.prohibited_production_paths.patterns {
            assert!(
                !glob_matches(pattern, path),
                "M062 changed a path under prohibited pattern {pattern}: {path}"
            );
        }
    }
}

fn is_authorized_planning_path(path: &str) -> bool {
    matches!(
        path,
        "plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md"
            | "plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md"
            | "plans/closure/i2pcontrol-proposal-170/062-closure.md"
            | "emissary-cli/tests/m062_dependency_containment.rs"
    )
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/**") {
        path.starts_with(prefix) && path.len() > prefix.len()
    } else {
        pattern == path
    }
}

#[test]
fn dependency_evidence_describes_subtle_ownership() {
    let manifest = load_manifest();
    assert!(
        manifest.evidence.iter().any(|entry| entry.crate_name == "subtle"
            && entry.direct_consumer == "emissary-cli/src/i2pcontrol/auth.rs"
            && entry.feature_gated_consumer
            && entry.feature.as_deref() == Some("i2pcontrol")),
        "manifest must record the i2pcontrol auth consumer of subtle"
    );
}
