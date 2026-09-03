//! M062 dependency-surface containment guards.
//!
//! Semantic TOML ownership checks for the `subtle` direct dependency used by
//! I2PControl authentication. The test enforces that:
//!
//! * the root `[workspace.dependencies]` table does not own an I2PControl-only `subtle`
//!   declaration;
//! * `emissary-cli` declares `subtle` as `optional = true` with default features disabled;
//! * only the `i2pcontrol` feature activates the optional `subtle` dependency;
//! * the forbidden root features (`default`, `ui`, `metrics`) cannot transitively activate the
//!   optional `subtle` dependency through local feature composition;
//! * the I2PControl-only Yosemite fork is optional, exact-revision pinned, and feature-owned;
//! * the lockfile retains ordinary registry Yosemite and records the exact fork source.
//!
//! The guard is intentionally semantic rather than comment-text based so that a
//! future regression in dependency ownership fails closed. The transitive helper
//! tracks visited local features to bound traversal across cycles and treats weak
//! dependency-feature syntax (`subtle?/feature`) as non-activating.

#![cfg(feature = "i2pcontrol")]

use std::{
    collections::{BTreeMap, BTreeSet},
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
    fork_dependency: ForkDependency,
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
struct ForkDependency {
    alias: String,
    package: String,
    source: String,
    revision: String,
    owner: String,
    owning_feature: String,
    features: Vec<String>,
    optional: bool,
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
    assert_eq!(
        manifest.lockfile.expected,
        "ordinary registry Yosemite plus exact optional git fork package when i2pcontrol is enabled"
    );
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
    assert_eq!(manifest.fork_dependency.alias, "yosemite-i2pcontrol");
    assert_eq!(manifest.fork_dependency.package, "yosemite");
    assert_eq!(
        manifest.fork_dependency.source,
        "https://github.com/eggstack/yosemite"
    );
    assert_eq!(
        manifest.fork_dependency.revision,
        "59140a2277bf296928d2e8ce39a148182eeff044"
    );
    assert_eq!(manifest.fork_dependency.owner, "emissary-cli");
    assert_eq!(manifest.fork_dependency.owning_feature, "i2pcontrol");
    assert_eq!(manifest.fork_dependency.features, ["async-extra"]);
    assert!(manifest.fork_dependency.optional);
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
fn i2pcontrol_yosemite_alias_is_optional_and_exactly_pinned() {
    let manifest = load_manifest();
    let raw = std::fs::read_to_string(workspace_root().join("emissary-cli/Cargo.toml"))
        .expect("emissary-cli manifest");
    let parsed: toml::Value = toml::from_str(&raw).expect("valid emissary-cli manifest");
    let dependency = parsed
        .get("dependencies")
        .and_then(|dependencies| dependencies.get("yosemite-i2pcontrol"))
        .and_then(toml::Value::as_table)
        .expect("I2PControl Yosemite alias must be a dependency table");
    assert_eq!(
        dependency.get("package").and_then(toml::Value::as_str),
        Some("yosemite")
    );
    assert_eq!(
        dependency.get("git").and_then(toml::Value::as_str),
        Some("https://github.com/eggstack/yosemite")
    );
    assert_eq!(
        dependency.get("rev").and_then(toml::Value::as_str),
        Some(manifest.fork_dependency.revision.as_str())
    );
    assert!(dependency.get("branch").is_none());
    assert!(dependency.get("tag").is_none());
    assert!(dependency.get("path").is_none());
    assert_eq!(
        dependency.get("optional").and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        dependency
            .get("features")
            .and_then(toml::Value::as_array)
            .map(|features| features.iter().filter_map(toml::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["async-extra"])
    );

    let features = parsed.get("features").and_then(toml::Value::as_table).expect("features table");
    let graph = LocalFeatureGraph::from_features_value(&toml::Value::Table(features.clone()));
    assert!(graph.transitively_activates("i2pcontrol", "yosemite-i2pcontrol"));
    for feature in ["default", "ui", "metrics"] {
        assert!(
            !graph.transitively_activates(feature, "yosemite-i2pcontrol"),
            "{feature} must not activate the I2PControl-only Yosemite alias"
        );
    }
}

#[test]
fn ordinary_workspace_yosemite_remains_registry_owned() {
    let raw = std::fs::read_to_string(workspace_root().join("Cargo.toml"))
        .expect("root workspace manifest");
    let parsed: toml::Value = toml::from_str(&raw).expect("valid root workspace manifest");
    let dependency = parsed
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(|dependencies| dependencies.get("yosemite"))
        .and_then(toml::Value::as_table)
        .expect("workspace Yosemite dependency");
    assert_eq!(
        dependency.get("version").and_then(toml::Value::as_str),
        Some("0.7.0")
    );
    assert!(dependency.get("git").is_none());
    assert!(dependency.get("path").is_none());
    assert!(dependency.get("branch").is_none());
    assert!(dependency.get("tag").is_none());
    assert_eq!(
        dependency
            .get("features")
            .and_then(toml::Value::as_array)
            .map(|features| features.iter().filter_map(toml::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["async-extra"])
    );
}

#[test]
fn fork_alias_is_feature_isolated_in_cargo_tree() {
    let disabled = Command::new("cargo")
        .args([
            "tree",
            "-p",
            "emissary-cli",
            "--no-default-features",
            "--edges",
            "normal",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("feature-disabled cargo tree");
    assert!(
        disabled.status.success(),
        "feature-disabled cargo tree failed"
    );
    let disabled = String::from_utf8_lossy(&disabled.stdout);
    assert!(!disabled.contains("github.com/eggstack/yosemite"));

    let enabled = Command::new("cargo")
        .args([
            "tree",
            "-p",
            "emissary-cli",
            "--no-default-features",
            "--features",
            "i2pcontrol",
            "--edges",
            "normal",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("I2PControl cargo tree");
    assert!(enabled.status.success(), "I2PControl cargo tree failed");
    let enabled = String::from_utf8_lossy(&enabled.stdout);
    assert!(enabled.contains("github.com/eggstack/yosemite"));
    assert!(enabled.contains("yosemite v0.7.0"));
}

#[test]
fn fork_imports_are_contained_to_i2pcontrol_source() {
    let output = Command::new("rg")
        .args(["-l", "yosemite_i2pcontrol", "emissary-cli/src"])
        .current_dir(workspace_root())
        .output()
        .expect("fork import search");
    assert!(output.status.success(), "fork import search failed");
    for path in String::from_utf8_lossy(&output.stdout).lines() {
        assert!(
            path.starts_with("emissary-cli/src/i2pcontrol/"),
            "fork alias escaped the I2PControl source boundary: {path}"
        );
    }
}

#[test]
fn only_i2pcontrol_activates_subtle() {
    let manifest = load_manifest();
    let raw = std::fs::read_to_string(workspace_root().join("emissary-cli/Cargo.toml"))
        .expect("emissary-cli manifest");
    let parsed: toml::Value = toml::from_str(&raw).expect("valid emissary-cli manifest");
    let features_table = parsed
        .get("features")
        .and_then(|features| features.as_table())
        .expect("features table");
    let graph = LocalFeatureGraph::from_features_value(&toml::Value::Table(features_table.clone()));

    let i2pcontrol = features_table
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

        if let Some(value) = features_table.get(forbidden_feature) {
            if let Some(list) = value.as_array() {
                for entry in list {
                    let entry = entry.as_str().unwrap_or_default();
                    assert!(
                        entry != "dep:subtle" && entry != "subtle",
                        "{forbidden_feature} must not activate subtle directly"
                    );
                }
            }
        }

        assert!(
            !graph.transitively_activates(forbidden_feature, "subtle"),
            "{forbidden_feature} must not reach an activation of the subtle dependency through \
             local feature composition"
        );
    }

    assert!(
        graph.transitively_activates("i2pcontrol", "subtle"),
        "i2pcontrol must reach an activation of the subtle dependency"
    );
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
        changed.lines().all(|path| {
            path == "plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml"
        }),
        "M118 may amend only the exact construction seam in M061 authority: {changed}"
    );
}

#[test]
fn lockfile_records_registry_and_exact_fork_yosemite_sources() {
    let manifest = load_manifest();
    assert_eq!(
        manifest.lockfile.baseline_commit,
        "a70dd3ac82f12fbea1f8fba51e30a9e2e516650a"
    );
    let raw = std::fs::read_to_string(workspace_root().join("Cargo.lock")).expect("Cargo.lock");
    let lock: toml::Value = toml::from_str(&raw).expect("valid Cargo.lock");
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("Cargo.lock package array");
    let yosemite = packages
        .iter()
        .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some("yosemite"))
        .collect::<Vec<_>>();
    assert!(
        yosemite.iter().any(|package| {
            package.get("source").and_then(toml::Value::as_str).is_some_and(|source| {
                source.starts_with("registry+https://github.com/rust-lang/crates.io-index")
            })
        }),
        "ordinary registry Yosemite package is missing from Cargo.lock"
    );
    let expected_source = format!(
        "git+{}?rev={}#{}",
        manifest.fork_dependency.source,
        manifest.fork_dependency.revision,
        manifest.fork_dependency.revision
    );
    assert!(
        yosemite.iter().any(|package| {
            package.get("source").and_then(toml::Value::as_str) == Some(expected_source.as_str())
        }),
        "exact Yosemite fork source/revision is missing from Cargo.lock"
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
        let authorized_m064 = path == "emissary-core/src/events.rs";
        let authorized_m065 = is_authorized_m065_path(path);
        let authorized_m096 = is_authorized_m096_path(path);
        let authorized_m097 = is_authorized_m097_path(path);
        let authorized_m100 = is_authorized_m100_path(path);
        let authorized_m101 = is_authorized_m101_path(path);
        let authorized_m102 = is_authorized_m102_path(path);
        let authorized_m099 = is_authorized_m099_path(path);
        let authorized_m107 = is_authorized_m107_path(path);
        let authorized_m109 = is_authorized_m109_path(path);
        let authorized_m115 = is_authorized_m115_path(path);
        let authorized_m110 = is_authorized_m110_path(path);
        let authorized_m116 = is_authorized_m116_path(path);
        let authorized_m117 = is_authorized_m117_path(path);
        let authorized_m118 = is_authorized_m118_path(path);
        let authorized_m119 = is_authorized_m119_path(path);
        let authorized_m120 = is_authorized_m120_path(path);
        let authorized_m121 = is_authorized_m121_path(path);
        let authorized_m122 = is_authorized_m122_path(path);
        let authorized_m124 = is_authorized_m124_path(path);
        let authorized_m111 = is_authorized_m111_path(path);
        let authorized_tunnel_runtime = is_authorized_tunnel_runtime_path(path);
        assert!(
            permitted
                || authorized_m064
                || authorized_m065
                || authorized_m096
                || authorized_m097
                || authorized_m100
                || authorized_m101
                || authorized_m102
                || authorized_m099
                || authorized_m107
                || authorized_m109
                || authorized_m115
                || authorized_m110
                || authorized_m116
                || authorized_m117
                || authorized_m118
                || authorized_m119
                || authorized_m120
                || authorized_m121
                || authorized_m122
                || authorized_m124
                || authorized_m111
                || authorized_tunnel_runtime
                || is_authorized_planning_path(path),
            "M062 changed an unauthorized production path: {path}"
        );

        for pattern in &manifest.prohibited_production_paths.patterns {
            assert!(
                authorized_m064
                    || authorized_m065
                    || authorized_m096
                    || authorized_m097
                    || authorized_m100
                    || authorized_m101
                    || authorized_m102
                    || authorized_m099
                    || authorized_m107
                    || authorized_m109
                    || authorized_m115
                    || authorized_m110
                    || authorized_m116
                    || authorized_m117
                    || authorized_m118
                    || authorized_m119
                    || authorized_m120
                    || authorized_m121
                    || authorized_m122
                    || authorized_m124
                    || authorized_tunnel_runtime
                    || !glob_matches(pattern, path),
                "M062 changed a path under prohibited pattern {pattern}: {path}"
            );
        }
    }
}

fn is_authorized_m117_path(path: &str) -> bool {
    matches!(
        path,
        "Cargo.lock"
            | "emissary-cli/Cargo.toml"
            | "emissary-cli/src/i2pcontrol/backends/client.rs"
            | "emissary-cli/src/i2pcontrol/backends/connect_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_bidir.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/peer_identity_impl.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/session.rs"
            | "emissary-cli/src/i2pcontrol/backends/server.rs"
            | "emissary-cli/src/i2pcontrol/backends/socks.rs"
            | "emissary-cli/src/i2pcontrol/backends/streamr.rs"
            | "emissary-cli/src/i2pcontrol/client_secret_store.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "plans/closure/i2pcontrol-proposal-170/117-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml"
            | "plans/implementation/i2pcontrol-proposal-170/117-internal-yosemite-fork-pin-and-i2pcontrol-adapter-integration.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m116_path(path: &str) -> bool {
    matches!(
        path,
        "emissary-cli/src/i2pcontrol/backends/options.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/session.rs"
            | "emissary-cli/src/i2pcontrol/backends/streamr.rs"
            | "emissary-cli/src/i2pcontrol/client_secret_store.rs"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "emissary-cli/tests/m095_full_support_matrix.rs"
            | "plans/closure/i2pcontrol-proposal-170/116-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"
            | "plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml"
            | "plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml"
            | "plans/implementation/i2pcontrol-proposal-170/112-client-proxy-and-session-lifecycle-residual-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/116-m110-shared-session-and-newdest-corrective-pass.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m110_path(path: &str) -> bool {
    matches!(
        path,
        "emissary-cli/src/i2pcontrol/backends/client.rs"
            | "emissary-cli/src/i2pcontrol/backends/connect_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_bidir.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/options.rs"
            | "emissary-cli/src/i2pcontrol/backends/registry.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/mod.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/session.rs"
            | "emissary-cli/src/i2pcontrol/backends/server.rs"
            | "emissary-cli/src/i2pcontrol/backends/socks.rs"
            | "emissary-cli/src/i2pcontrol/backends/socks_irc.rs"
            | "emissary-cli/src/i2pcontrol/backends/streamr.rs"
            | "emissary-cli/src/i2pcontrol/client_secret_store.rs"
            | "emissary-cli/src/i2pcontrol/mod.rs"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/src/i2pcontrol/server_secret_store.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "emissary-cli/tests/m095_full_support_matrix.rs"
            | "emissary-cli/tests/m105_residual_option_audit.rs"
            | "plans/closure/i2pcontrol-proposal-170/110-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"
            | "plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml"
            | "plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml"
            | "plans/implementation/i2pcontrol-proposal-170/110-shared-client-session-and-destination-key-ownership-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m099_path(path: &str) -> bool {
    matches!(
        path,
        "docs/i2pcontrol/README.md"
            | "docs/i2pcontrol/proposal-170-support.md"
            | "docs/i2pcontrol/proposal-170-conformance.md"
            | "docs/i2pcontrol/security.md"
            | "docs/i2pcontrol/tunnel-manager.md"
            | "docs/i2pcontrol/tunnel-backends.md"
            | "emissary-cli/src/i2pcontrol/backends/filters/http.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_bidir.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/access.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/admission.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/mod.rs"
            | "emissary-cli/src/i2pcontrol/backends/server.rs"
            | "emissary-cli/src/i2pcontrol/tunnel_manager.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "emissary-cli/tests/m095_full_support_matrix.rs"
            | "plans/closure/i2pcontrol-proposal-170/098-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/099-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m107_path(path: &str) -> bool {
    matches!(
        path,
        "docs/i2pcontrol/README.md"
            | "docs/i2pcontrol/address-book.md"
            | "emissary-cli/src/i2pcontrol/address_book_runtime.rs"
            | "emissary-cli/src/i2pcontrol/auth.rs"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/src/i2pcontrol/rpc.rs"
            | "emissary-cli/src/i2pcontrol/server.rs"
            | "emissary-cli/src/i2pcontrol/tls.rs"
            | "emissary-cli/tests/adversarial.rs"
            | "emissary-cli/tests/fixtures/mod.rs"
            | "emissary-cli/tests/golden_fixtures.rs"
            | "emissary-cli/tests/i2pcontrol.rs"
            | "emissary-cli/tests/i2pcontrol_live_runtime.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "plans/closure/i2pcontrol-proposal-170/107-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/107-i2pcontrol-conformance-and-managed-tls-corrective-pass.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m109_path(path: &str) -> bool {
    matches!(
        path,
        "emissary-cli/src/main.rs"
            | "emissary-cli/src/tunnel/client.rs"
            | "emissary-cli/src/tunnel/server.rs"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/src/i2pcontrol/tunnel_manager.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "plans/closure/i2pcontrol-proposal-170/109-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/109-startup-managed-tunnel-action-semantics-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m115_path(path: &str) -> bool {
    matches!(
        path,
        "emissary-cli/src/main.rs"
            | "emissary-cli/src/tunnel/client.rs"
            | "emissary-cli/src/tunnel/server.rs"
            | "emissary-cli/tests/static_guards.rs"
            | "plans/closure/i2pcontrol-proposal-170/115-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/115-m109-runtime-disable-and-lifecycle-truthfulness-corrective-pass.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_tunnel_runtime_path(path: &str) -> bool {
    matches!(
        path,
        "emissary-cli/src/i2pcontrol/backends/filters/http.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/admission.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/peer_identity.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/peer_identity_impl.rs"
            | "emissary-cli/src/i2pcontrol/server.rs"
            | "plans/closure/i2pcontrol-proposal-170/067-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/068-closure.md"
            | "emissary-cli/src/i2pcontrol/backends/connect_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/filters/http_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/filters/proxy.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/socks.rs"
            | "emissary-cli/src/i2pcontrol/backends/socks_irc.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_bidir.rs"
            | "emissary-cli/src/proxy/socks.rs"
    )
}

fn is_authorized_m118_path(path: &str) -> bool {
    matches!(
        path,
        "emissary-core/src/config.rs"
            | "plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml"
            | "emissary-core/src/sam/parser.rs"
            | "emissary-core/src/sam/mod.rs"
            | "emissary-core/src/tunnel/pool/mod.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "plans/closure/i2pcontrol-proposal-170/118-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/118-neutral-sam-tunnel-pool-variance-backup-capability.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m119_path(path: &str) -> bool {
    matches!(
        path,
        "emissary-core/src/tunnel/pool/mod.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "plans/closure/i2pcontrol-proposal-170/119-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/119-m118-standby-expiry-and-variance-semantics-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/120-server-start-preallocation-validation-and-secret-transactionality-corrective.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md"
    )
}

fn is_authorized_m120_path(path: &str) -> bool {
    matches!(
        path,
        "emissary-cli/src/i2pcontrol/backends/mod.rs"
            | "emissary-cli/src/i2pcontrol/backends/server.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_bidir.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/streamr.rs"
            | "emissary-cli/src/i2pcontrol/backends/unsupported.rs"
            | "emissary-cli/src/i2pcontrol/backends/options.rs"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/src/i2pcontrol/server_secret_store.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "plans/closure/i2pcontrol-proposal-170/120-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/120-server-start-preallocation-validation-and-secret-transactionality-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/121-m111-m112-semantic-truthfulness-corrective.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md"
    )
}

fn is_authorized_m121_path(path: &str) -> bool {
    matches!(
        path,
        "plans/closure/i2pcontrol-proposal-170/121-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/121-m111-m112-semantic-truthfulness-corrective.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md"
    )
}

fn is_authorized_m122_path(path: &str) -> bool {
    matches!(
        path,
        "Cargo.lock"
            | "emissary-cli/Cargo.toml"
            | "emissary-cli/src/i2pcontrol/backends/runtime/session.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "plans/closure/i2pcontrol-proposal-170/122-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml"
            | "plans/implementation/i2pcontrol-proposal-170/122-corrected-yosemite-leaseset-pin-adoption.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
            | "docs/i2pcontrol/tunnel-manager.md"
    )
}

fn is_authorized_m124_path(path: &str) -> bool {
    matches!(
        path,
        "Cargo.lock"
            | "emissary-cli/Cargo.toml"
            | "emissary-cli/src/i2pcontrol/backends/runtime/session.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "plans/closure/i2pcontrol-proposal-170/124-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/123-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml"
            | "plans/implementation/i2pcontrol-proposal-170/123-m120-commit-phase-cancellation-atomicity-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/124-y005-auth-consistency-pin-adoption.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
            | "docs/i2pcontrol/tunnel-manager.md"
    )
}

fn is_authorized_m111_path(path: &str) -> bool {
    matches!(path, "emissary-cli/tests/m060_containment.rs")
}

fn is_authorized_m065_path(path: &str) -> bool {
    matches!(
        path,
        "AGENTS.md"
            | "README.md"
            | "docs/i2pcontrol/README.md"
            | "docs/i2pcontrol/inspection-architecture.md"
            | "docs/i2pcontrol/proposal-170-support.md"
            | "docs/i2pcontrol/proposal-170-conformance.md"
            | "docs/i2pcontrol/tunnel-manager.md"
            | "docs/i2pcontrol/tunnel-backends.md"
            | "plans/closure/i2pcontrol-proposal-170/064-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/065-closure.md"
            | "emissary-cli/src/i2pcontrol/backends/mod.rs"
            | "emissary-cli/src/i2pcontrol/backends/client.rs"
            | "emissary-cli/src/i2pcontrol/backends/server.rs"
            | "emissary-cli/src/i2pcontrol/backends/registry.rs"
            | "emissary-cli/src/i2pcontrol/backends/filters/mod.rs"
            | "emissary-cli/src/i2pcontrol/backends/filters/irc.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/options.rs"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/mod.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/task_group.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/streamr.rs"
            | "docs/i2pcontrol/streamr-runtime.md"
            | "plans/closure/i2pcontrol-proposal-170/071-closure.md"
    )
}

fn is_authorized_m096_path(path: &str) -> bool {
    matches!(
        path,
        "docs/i2pcontrol/README.md"
            | "docs/i2pcontrol/address-book.md"
            | "docs/i2pcontrol/administrative-state.md"
            | "docs/i2pcontrol/proposal-170-conformance.md"
            | "docs/i2pcontrol/proposal-170-support.md"
            | "docs/i2pcontrol/security.md"
            | "plans/closure/i2pcontrol-proposal-170/096-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"
            | "plans/implementation/i2pcontrol-proposal-170/096-addressbook-setconfig-operational-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
            | "emissary-cli/src/address_book.rs"
            | "emissary-cli/src/i2pcontrol/address_book.rs"
            | "emissary-cli/src/i2pcontrol/address_book_runtime.rs"
            | "emissary-cli/src/i2pcontrol/control_plane.rs"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
    )
}

fn is_authorized_m097_path(path: &str) -> bool {
    matches!(
        path,
        "emissary-cli/src/i2pcontrol/backends/client.rs"
            | "emissary-cli/src/i2pcontrol/backends/connect_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_bidir.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/http_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_client.rs"
            | "emissary-cli/src/i2pcontrol/backends/irc_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/options.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/mod.rs"
            | "emissary-cli/src/i2pcontrol/backends/runtime/session.rs"
            | "emissary-cli/src/i2pcontrol/backends/server.rs"
            | "emissary-cli/src/i2pcontrol/backends/socks.rs"
            | "emissary-cli/src/i2pcontrol/backends/socks_irc.rs"
            | "emissary-cli/src/i2pcontrol/backends/streamr.rs"
            | "emissary-cli/src/i2pcontrol/domain/tunnel.rs"
            | "emissary-cli/src/i2pcontrol/tunnel_manager.rs"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"
            | "plans/implementation/i2pcontrol-proposal-170/097-tunnel-common-session-and-key-option-completion.md"
            | "plans/closure/i2pcontrol-proposal-170/097-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m100_path(path: &str) -> bool {
    matches!(
        path,
        "AGENTS.md"
            | "docs/i2pcontrol/README.md"
            | "docs/i2pcontrol/proposal-170-conformance.md"
            | "docs/i2pcontrol/proposal-170-support.md"
            | "docs/i2pcontrol/router-info-source-map.md"
            | "docs/i2pcontrol/router-info.md"
            | "emissary-cli/src/i2pcontrol/mod.rs"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/src/i2pcontrol/rpc.rs"
            | "emissary-cli/src/i2pcontrol/server.rs"
            | "emissary-cli/src/i2pcontrol/transit_sampler.rs"
            | "emissary-cli/tests/conformance_manifest.rs"
            | "emissary-cli/tests/m095_full_support_matrix.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "emissary-cli/tests/m027_literal_fixtures.rs"
            | "emissary-cli/tests/production_adapter.rs"
            | "emissary-cli/tests/router_info_truthfulness.rs"
            | "emissary-cli/tests/static_guards.rs"
            | "plans/closure/i2pcontrol-proposal-170/100-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"
            | "plans/implementation/i2pcontrol-proposal-170/100-routerinfo-transit-15s-source-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m101_path(path: &str) -> bool {
    matches!(
        path,
        "AGENTS.md"
            | "docs/i2pcontrol/README.md"
            | "docs/i2pcontrol/proposal-170-conformance.md"
            | "docs/i2pcontrol/proposal-170-support.md"
            | "docs/i2pcontrol/router-info-source-map.md"
            | "docs/i2pcontrol/router-info.md"
            | "emissary-cli/src/i2pcontrol/mod.rs"
            | "emissary-cli/src/i2pcontrol/news.rs"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/src/i2pcontrol/router_info.rs"
            | "emissary-cli/src/i2pcontrol/router_info_handler.rs"
            | "emissary-cli/src/i2pcontrol/rpc.rs"
            | "emissary-cli/src/i2pcontrol/server.rs"
            | "emissary-cli/src/main.rs"
            | "emissary-cli/tests/conformance_manifest.rs"
            | "emissary-cli/tests/m027_literal_fixtures.rs"
            | "emissary-cli/tests/m062_dependency_containment.rs"
            | "emissary-cli/tests/m095_full_support_matrix.rs"
            | "emissary-cli/tests/static_guards.rs"
            | "emissary-util/src/certificates.rs"
            | "emissary-util/src/su3.rs"
            | "emissary-util/assets/certificates/news/echelon_at_mail.i2p.crt"
            | "emissary-util/assets/certificates/news/hankhill19580_at_gmail.com.crt"
            | "emissary-util/assets/certificates/news/zzz_at_mail.i2p.crt"
            | "plans/closure/i2pcontrol-proposal-170/101-closure.md"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"
            | "plans/implementation/i2pcontrol-proposal-170/101-routerinfo-news-source-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_m102_path(path: &str) -> bool {
    matches!(
        path,
        "docs/i2pcontrol/README.md"
            | "docs/i2pcontrol/proposal-170-conformance.md"
            | "docs/i2pcontrol/proposal-170-support.md"
            | "docs/i2pcontrol/router-info-source-map.md"
            | "docs/i2pcontrol/router-info.md"
            | "emissary-cli/src/i2pcontrol/production.rs"
            | "emissary-cli/src/i2pcontrol/router_info.rs"
            | "emissary-cli/src/i2pcontrol/router_info_handler.rs"
            | "emissary-cli/src/i2pcontrol/rpc.rs"
            | "emissary-cli/tests/conformance_manifest.rs"
            | "emissary-cli/tests/i2pcontrol_live_runtime.rs"
            | "emissary-cli/tests/m027_literal_fixtures.rs"
            | "emissary-cli/tests/m095_full_support_matrix.rs"
            | "emissary-cli/tests/static_guards.rs"
            | "emissary-core/src/events.rs"
            | "emissary-core/src/inspection.rs"
            | "emissary-core/src/transport/mod.rs"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"
            | "plans/implementation/i2pcontrol-proposal-170/102-routerinfo-network-error-owner-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
    )
}

fn is_authorized_planning_path(path: &str) -> bool {
    matches!(
        path,
        "plans/README.md"
            | "plans/000-long-term-specification.md"
            | "plans/002-long-term-roadmap.md"
            | "plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md"
            | "plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md"
            | "plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md"
            | "plans/adrs/ADR-0005-internal-yosemite-fork-dependency-boundary.md"
            | "plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md"
            | "plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml"
            | "plans/implementation/i2pcontrol-proposal-170/063-m062-closure-and-feature-guard-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/064-proposal-170-tunnel-runtime-baseline-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/065-i2pcontrol-tunnel-runtime-primitives.md"
            | "plans/implementation/i2pcontrol-proposal-170/066-irc-client-server-tunnel-family.md"
            | "plans/implementation/i2pcontrol-proposal-170/067-http-server-tunnel.md"
            | "plans/implementation/i2pcontrol-proposal-170/068-http-client-and-connect-tunnels.md"
            | "plans/implementation/i2pcontrol-proposal-170/069-socks-and-socks-irc-tunnels.md"
            | "plans/implementation/i2pcontrol-proposal-170/070-http-bidirectional-server-composition.md"
            | "plans/implementation/i2pcontrol-proposal-170/071-streamr-client-server-tunnels.md"
            | "plans/implementation/i2pcontrol-proposal-170/072-tunnel-runtime-completion-reclosure.md"
            | "plans/implementation/i2pcontrol-proposal-170/073-generic-tunnel-option-truthfulness-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md"
            | "plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md"
            | "plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md"
            | "plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md"
            | "plans/implementation/i2pcontrol-proposal-170/078-streamr-local-boundary-hardening.md"
            | "plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md"
            | "plans/implementation/i2pcontrol-proposal-170/080-server-admission-transactionality-and-cardinality-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md"
            | "plans/implementation/i2pcontrol-proposal-170/086-post-m085-documentation-and-evidence-reconciliation-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md"
            | "plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md"
            | "plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md"
            | "plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md"
            | "plans/implementation/i2pcontrol-proposal-170/094-post-m093-planning-state-reconciliation.md"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-contract-matrix-and-containment-budget.md"
            | "plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml"
            | "plans/implementation/i2pcontrol-proposal-170/096-addressbook-setconfig-operational-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/097-tunnel-common-session-and-key-option-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/098-client-proxy-management-and-http-option-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/099-server-access-throttle-and-leaseset-option-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/100-routerinfo-transit-15s-source-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/101-routerinfo-news-source-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/102-routerinfo-network-error-owner-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/103-routerinfo-banned-peer-semantic-completion.md"
            | "plans/implementation/i2pcontrol-proposal-170/104-full-proposal-170-live-interoperability-and-reclosure.md"
            | "plans/implementation/i2pcontrol-proposal-170/105-residual-tunnel-option-primitive-audit.md"
            | "plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml"
            | "plans/implementation/i2pcontrol-proposal-170/106-delay-open-client-listener.md"
            | "plans/implementation/i2pcontrol-proposal-170/108-managed-tls-upgrade-permission-corrective-pass.md"
            | "plans/implementation/i2pcontrol-proposal-170/README.md"
            | "plans/registry.md"
            | "plans/subsystems/i2pcontrol-proposal-170-roadmap.md"
            | "plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md"
            | "plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md"
            | "plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md"
            | "plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md"
            | "plans/closure/i2pcontrol-proposal-170/108-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/062-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/063-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/066-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/069-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/070-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/072-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/072-option-capability-matrix.toml"
            | "plans/closure/i2pcontrol-proposal-170/073-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/074-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/075-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/076-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/077-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/078-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/079-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/080-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/081-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/082-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/083-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/084-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/085-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/086-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/087-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/088-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/089-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/090-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/091-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/092-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/093-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/094-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/095-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/098-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/099-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/102-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/103-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/104-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/105-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/106-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/111-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/112-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/113-closure.md"
            | "plans/closure/i2pcontrol-proposal-170/114-closure.md"
            | "emissary-cli/tests/m105_residual_option_audit.rs"
            | "emissary-cli/tests/m095_full_support_matrix.rs"
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

/// Semantic local-feature reachability helper for indirect Cargo feature activation.
///
/// Parses the `[features]` table of a Cargo manifest and computes whether a
/// root feature reaches an activation of a target dependency through local
/// feature composition. The helper is intentionally scoped to this guard and
/// does not attempt a full Cargo resolver/fingerprint model.
///
/// Activation rules:
/// - `dep:NAME` — explicit dependency activation;
/// - `NAME/feature` — strong dependency feature, activates `NAME`;
/// - `NAME?/feature` — weak dependency feature, does NOT activate `NAME`;
/// - `?/NAME` — weak feature reference, does NOT activate `NAME`;
/// - `NAME` — local feature reference (recurses) OR bare dependency activation when `NAME` is not a
///   declared local feature.
///
/// Traversal uses a visited set so cycle-bearing feature maps terminate.
#[derive(Debug, Clone)]
struct LocalFeatureGraph {
    features: BTreeMap<String, Vec<String>>,
}

impl LocalFeatureGraph {
    fn from_features_value(value: &toml::Value) -> Self {
        let mut features: BTreeMap<String, Vec<String>> = BTreeMap::new();
        if let Some(table) = value.as_table() {
            for (name, entries) in table {
                let mut edge_list: Vec<String> = Vec::new();
                if let Some(arr) = entries.as_array() {
                    for entry in arr {
                        if let Some(s) = entry.as_str() {
                            edge_list.push(s.to_owned());
                        }
                    }
                }
                features.insert(name.clone(), edge_list);
            }
        }
        Self { features }
    }

    fn from_toml_str(raw: &str) -> Self {
        let value: toml::Value = toml::from_str(raw).expect("valid feature-only TOML");
        let features = value
            .get("features")
            .unwrap_or_else(|| panic!("features table missing in fixture: {raw}"));
        Self::from_features_value(features)
    }

    fn edges(&self, feature: &str) -> &[String] {
        self.features.get(feature).map(Vec::as_slice).unwrap_or(&[])
    }

    fn transitively_activates(&self, root_feature: &str, dependency: &str) -> bool {
        let mut visited: BTreeSet<String> = BTreeSet::new();
        self.visit(root_feature, dependency, &mut visited)
    }

    fn visit(&self, feature: &str, dependency: &str, visited: &mut BTreeSet<String>) -> bool {
        if !visited.insert(feature.to_owned()) {
            return false;
        }
        for edge in self.edges(feature) {
            if edge_activates_dependency(edge, dependency) {
                return true;
            }
            if is_local_feature(edge, &self.features) && self.visit(edge, dependency, visited) {
                return true;
            }
        }
        false
    }
}

fn edge_activates_dependency(edge: &str, dependency: &str) -> bool {
    if edge.starts_with("?/") {
        return false;
    }
    if let Some(name) = edge.strip_prefix("dep:") {
        return name == dependency;
    }
    if let Some((dep_part, _feat)) = edge.split_once('/') {
        if dep_part == dependency {
            return !dep_part.ends_with('?');
        }
        return false;
    }
    edge == dependency
}

fn is_local_feature(edge: &str, features: &BTreeMap<String, Vec<String>>) -> bool {
    !edge.starts_with("?/")
        && !edge.starts_with("dep:")
        && !edge.contains('/')
        && features.contains_key(edge)
}

#[test]
fn current_manifest_forbidden_features_cannot_reach_subtle() {
    let raw = std::fs::read_to_string(workspace_root().join("emissary-cli/Cargo.toml"))
        .expect("emissary-cli manifest");
    let parsed: toml::Value = toml::from_str(&raw).expect("valid emissary-cli manifest");
    let graph =
        LocalFeatureGraph::from_features_value(parsed.get("features").expect("features table"));

    for forbidden_feature in ["default", "ui", "metrics"] {
        assert!(
            !graph.transitively_activates(forbidden_feature, "subtle"),
            "{forbidden_feature} must not reach an activation of the subtle dependency through \
             local feature composition"
        );
    }

    assert!(
        graph.transitively_activates("i2pcontrol", "subtle"),
        "i2pcontrol must reach an activation of the subtle dependency"
    );
}

#[test]
fn direct_forbidden_feature_activation_is_rejected() {
    let graph = LocalFeatureGraph::from_toml_str(
        r#"
        [features]
        default = ["dep:subtle"]
        "#,
    );
    assert!(graph.transitively_activates("default", "subtle"));
}

#[test]
fn indirect_forbidden_feature_chain_is_rejected() {
    let graph = LocalFeatureGraph::from_toml_str(
        r#"
        [features]
        ui = ["i2pcontrol"]
        i2pcontrol = ["dep:subtle"]
        "#,
    );
    assert!(graph.transitively_activates("ui", "subtle"));
    assert!(graph.transitively_activates("i2pcontrol", "subtle"));
}

#[test]
fn indirect_forbidden_feature_chain_via_strong_dep_feature_is_rejected() {
    let graph = LocalFeatureGraph::from_toml_str(
        r#"
        [features]
        metrics = ["ui"]
        ui = ["i2pcontrol"]
        i2pcontrol = ["subtle/ct"]
        "#,
    );
    assert!(graph.transitively_activates("metrics", "subtle"));
    assert!(graph.transitively_activates("ui", "subtle"));
}

#[test]
fn unrelated_local_feature_chain_that_does_not_reach_subtle_passes() {
    let graph = LocalFeatureGraph::from_toml_str(
        r#"
        [features]
        default = ["ui"]
        ui = ["alpha"]
        alpha = ["beta"]
        beta = ["dep:other"]
        "#,
    );
    assert!(!graph.transitively_activates("default", "subtle"));
    assert!(!graph.transitively_activates("ui", "subtle"));
    assert!(!graph.transitively_activates("alpha", "subtle"));
    assert!(!graph.transitively_activates("beta", "subtle"));
}

#[test]
fn feature_cycle_terminates_and_still_detects_activation() {
    let graph = LocalFeatureGraph::from_toml_str(
        r#"
        [features]
        ui = ["alpha"]
        alpha = ["beta"]
        beta = ["alpha", "i2pcontrol"]
        i2pcontrol = ["dep:subtle"]
        "#,
    );
    assert!(
        graph.transitively_activates("ui", "subtle"),
        "cycle must not prevent activation detection"
    );
    assert!(
        graph.transitively_activates("alpha", "subtle"),
        "cycle must not prevent activation detection"
    );
    assert!(
        graph.transitively_activates("beta", "subtle"),
        "cycle must not prevent activation detection"
    );
}

#[test]
fn weak_dependency_feature_alone_is_not_independent_activation() {
    let graph = LocalFeatureGraph::from_toml_str(
        r#"
        [features]
        default = ["subtle?/ct"]
        ui = ["?/i2pcontrol"]
        "#,
    );
    assert!(
        !graph.transitively_activates("default", "subtle"),
        "weak subtle?/feature must not independently activate subtle"
    );
    assert!(
        !graph.transitively_activates("ui", "subtle"),
        "weak ?/feature must not independently activate the referenced feature"
    );
}

#[test]
fn weak_dependency_feature_alongside_strong_still_activates() {
    let graph = LocalFeatureGraph::from_toml_str(
        r#"
        [features]
        default = ["subtle?/ct", "dep:subtle"]
        "#,
    );
    assert!(
        graph.transitively_activates("default", "subtle"),
        "weak subtle?/feature must not block a sibling strong activation"
    );
}

#[test]
fn direct_bare_dependency_activation_is_detected() {
    let graph = LocalFeatureGraph::from_toml_str(
        r#"
        [features]
        default = ["subtle"]
        "#,
    );
    assert!(graph.transitively_activates("default", "subtle"));
}

#[test]
fn local_feature_with_no_entry_terminates_safely() {
    let graph = LocalFeatureGraph::from_toml_str(
        r#"
        [features]
        default = ["ui"]
        ui = []
        "#,
    );
    assert!(!graph.transitively_activates("default", "subtle"));
}

#[test]
fn forbidden_activations_manifest_is_self_consistent_with_graph() {
    let manifest = load_manifest();
    let raw = std::fs::read_to_string(workspace_root().join("emissary-cli/Cargo.toml"))
        .expect("emissary-cli manifest");
    let parsed: toml::Value = toml::from_str(&raw).expect("valid emissary-cli manifest");
    let graph =
        LocalFeatureGraph::from_features_value(parsed.get("features").expect("features table"));

    for forbidden_feature in ["default", "ui", "metrics"] {
        let forbidden_activations = manifest
            .forbidden_activations
            .get(forbidden_feature)
            .and_then(|value| value.as_array())
            .unwrap_or_else(|| panic!("forbidden_activations must list {forbidden_feature}"));
        assert!(
            forbidden_activations.is_empty(),
            "{forbidden_feature} must list no forbidden activations"
        );
        assert!(
            !graph.transitively_activates(forbidden_feature, "subtle"),
            "{forbidden_feature} must not reach subtle even via local feature composition"
        );
    }
}
