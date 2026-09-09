//! M126 historical requalification guards (rebased by M153).
//!
//! M126 closed at `284/96/460`. M139 rebased the then-current count checks to
//! `325/47/468`; M153 rebases them again so milestone-local history stays
//! pinned to immutable closure evidence while current-head aggregate counts
//! are owned solely by the M153 guard (`m153_post_m146_requalification`).
//! Runtime behavior is covered by the existing authenticated live-runtime,
//! production-adapter, and adversarial suites; this file prevents their
//! evidence from drifting away from the active planning authority.

#![cfg(feature = "i2pcontrol")]

use std::{collections::BTreeMap, fs, path::PathBuf};

use toml::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn planning_file(name: &str) -> String {
    fs::read_to_string(
        workspace_root().join("plans/implementation/i2pcontrol-proposal-170").join(name),
    )
    .unwrap_or_else(|error| panic!("failed to read planning file {name}: {error}"))
}

fn table<'a>(root: &'a Value, section: &str) -> &'a toml::value::Table {
    root.get(section)
        .and_then(Value::as_table)
        .unwrap_or_else(|| panic!("missing matrix table {section}"))
}

fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string field {key}"))
}

#[test]
fn current_matrix_is_mechanically_requalified() {
    let matrix: Value = planning_file("095-full-support-matrix.toml")
        .parse()
        .expect("valid M095 matrix");

    assert_eq!(matrix["proposal_number"].as_integer(), Some(170));
    assert_eq!(matrix["proposal_revision"].as_str(), Some("2026-05-20"));
    assert_eq!(matrix["proposal_status"].as_str(), Some("Open"));
    assert_eq!(
        matrix["source_sha256"].as_str(),
        Some("f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc",)
    );

    let router_rows = table(&matrix, "router_info")["rows"].as_array().expect("RouterInfo rows");
    assert_eq!(router_rows.len(), 43);
    let router_counts = router_rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(string(row, "current_disposition")).or_insert(0usize) += 1;
        counts
    });
    assert_eq!(router_counts.get("available"), Some(&42));
    assert_eq!(router_counts.get("neutral"), Some(&1));
    assert_eq!(
        router_counts.get("unavailable").copied().unwrap_or_default(),
        0
    );

    let setconfig_rows = table(&matrix, "addressbook_setconfig")["rows"]
        .as_array()
        .expect("SetConfig rows");
    assert_eq!(setconfig_rows.len(), 13);
    assert!(setconfig_rows.iter().all(|row| {
        matches!(
            string(row, "current_disposition"),
            "available_operational" | "available_metadata"
        )
    }));
    let theme = setconfig_rows
        .iter()
        .find(|row| string(row, "key") == "theme")
        .expect("theme SetConfig key");
    assert_eq!(string(theme, "current_disposition"), "available_metadata");

    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("canonical tunnel types");
    assert_eq!(tunnel_types.len(), 12);
    let options = table(&matrix, "tunnel_manager")["options"]
        .as_array()
        .expect("TunnelManager options");
    let mut counts = BTreeMap::new();
    for option in options {
        let cells = option["cells"].as_array().expect("option cells");
        assert_eq!(cells.len(), tunnel_types.len());
        for cell in cells {
            *counts.entry(cell.as_str().expect("cell disposition")).or_insert(0usize) += 1;
        }
    }
    // M153 owns the exact current-head aggregate (`336/29/475`). This
    // historical suite pins only mechanical self-consistency (declared counts
    // agree with recomputation, 840 cells total, no unknown disposition) so
    // later legitimate promotions do not red-flag M126 behavior.
    let declared = matrix["current_matrix_counts"].as_table().expect("declared counts");
    let declared_total = declared["total"].as_integer().expect("declared total");
    assert_eq!(declared_total, 840);
    for key in ["apply", "blocked_primitive", "not_applicable"] {
        let declared_value = declared[key].as_integer().expect("declared count");
        assert_eq!(
            counts.get(key).copied().unwrap_or_default() as i64,
            declared_value
        );
    }
    let recomputed_total: usize = counts.values().sum();
    assert_eq!(recomputed_total, 840);
    assert_eq!(counts.get("planned_apply").copied().unwrap_or_default(), 0);

    // Historical invariant: M126's own closure remains immutable evidence of
    // the `284/96/460` head it qualified.
    let closure = fs::read_to_string(
        workspace_root().join("plans/closure/i2pcontrol-proposal-170/126-closure.md"),
    )
    .expect("M126 closure must exist");
    assert!(
        closure.contains("284 apply / 96 blocked_primitive / 460"),
        "M126 closure must retain its historical 284/96/460 authority"
    );
}

#[test]
fn active_support_docs_agree_with_the_current_partial_claim() {
    let root = workspace_root();
    // Durable current-head authority is owned by M153 (`336/29/475`); this
    // historical suite pins only the durable partial-support claim plus the
    // M126 lineage, and keeps the M126-era counts in immutable closure
    // evidence rather than in active docs.
    let active = [
        root.join("AGENTS.md"),
        root.join("plans/registry.md"),
        root.join("plans/implementation/i2pcontrol-proposal-170/README.md"),
        root.join("plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md"),
    ];

    for path in &active {
        let text = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        assert!(
            text.to_ascii_lowercase().contains("partial"),
            "{} must retain the partial-support status",
            path.display()
        );
        assert!(
            !text.contains("Status: full Proposal 170 support")
                && !text.contains("Status: **full Proposal 170 support"),
            "{} must not claim full Proposal 170 support",
            path.display()
        );
    }
    // Current-head counts live in the M153 guard; active docs must agree with
    // them there, not with the superseded M126-era `325/47/468` wording.
    let registry = fs::read_to_string(root.join("plans/registry.md")).expect("registry");
    assert!(
        registry.contains("336") && registry.contains("29") && registry.contains("475"),
        "registry must state the current M153 336/29/475 authority"
    );
    let closure =
        fs::read_to_string(root.join("plans/closure/i2pcontrol-proposal-170/126-closure.md"))
            .expect("M126 closure must exist");
    assert!(
        closure.contains("284") && closure.contains("96") && closure.contains("460"),
        "M126 closure must retain its historical 284/96/460 evidence"
    );
    for path in &active {
        let text = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        assert!(
            !text.contains("312\napply / 70") && !text.contains("312 / 70 / 458"),
            "{} retains the superseded active matrix count",
            path.display()
        );
    }
}

#[test]
fn production_composition_has_no_fake_fallback_and_requires_runtime_owners() {
    let root = workspace_root();
    let server = fs::read_to_string(root.join("emissary-cli/src/i2pcontrol/server.rs"))
        .expect("server source");
    let init_start = server.find("pub async fn init_server").expect("init_server");
    let init_end = server.find("/// Zero-cost event metrics stub").expect("init_server end");
    let init = &server[init_start..init_end];

    assert!(init.contains("ctx.address_book_handle.ok_or_else"));
    assert!(init.contains("address_books.load().await"));
    assert!(init.contains("tunnels\n        .load()\n        .await"));
    assert!(!init.contains("Fake"));
    assert!(!init.contains("new_test"));

    let main = fs::read_to_string(root.join("emissary-cli/src/main.rs")).expect("main source");
    assert!(main.contains("with_address_book_handle"));
    assert!(main.contains("with_startup_tunnel_inventory"));
    assert!(main.contains("init_server(&server_config"));
}

#[test]
fn yosemite_fork_remains_optional_and_i2pcontrol_owned() {
    let manifest =
        fs::read_to_string(workspace_root().join("emissary-cli/Cargo.toml")).expect("CLI manifest");
    assert!(manifest.contains("yosemite-i2pcontrol"));
    assert!(manifest.contains("optional = true"));
    assert!(manifest.contains("dep:yosemite-i2pcontrol"));

    let lockfile = fs::read_to_string(workspace_root().join("Cargo.lock")).expect("Cargo.lock");
    assert!(lockfile.contains(
        "git+https://github.com/eggstack/yosemite?rev=59140a2277bf296928d2e8ce39a148182eeff044"
    ));
    assert!(lockfile.contains("registry+https://github.com/rust-lang/crates.io-index"));
}
