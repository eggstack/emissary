//! Static exhaustiveness guard for the M095 Proposal 170 completion matrix.
//!
//! The matrix is planning evidence, not a runtime capability source. This test
//! keeps its row counts and canonical names aligned with the production
//! inventories without requiring a router, network access, or feature work.

#![cfg(feature = "i2pcontrol")]

use std::{collections::BTreeSet, path::Path};

use toml::Value;

fn matrix() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml");
    let raw = std::fs::read_to_string(path).expect("M095 matrix must exist");
    raw.parse().expect("M095 matrix must be valid TOML")
}

fn rows<'a>(root: &'a Value, section: &str, key: &str) -> &'a [Value] {
    root.get(section)
        .and_then(Value::as_table)
        .and_then(|table| table.get(key))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .expect("matrix array is present")
}

fn string_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("matrix row is missing string field {key}"))
}

fn string_set(rows: &[Value], key: &str) -> BTreeSet<String> {
    rows.iter().map(|row| string_field(row, key).to_owned()).collect()
}

#[test]
fn matrix_is_exhaustive_and_truthful_at_the_current_baseline() {
    let root = matrix();
    assert_eq!(root["proposal_number"].as_integer(), Some(170));
    assert_eq!(root["proposal_revision"].as_str(), Some("2026-05-20"));
    assert_eq!(root["proposal_status"].as_str(), Some("Open"));
    assert_eq!(
        root["source_sha256"].as_str(),
        Some("f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc")
    );

    let router_rows = rows(&root, "router_info", "rows");
    assert_eq!(router_rows.len(), 43);
    assert_eq!(string_set(router_rows, "key").len(), 43);
    let counts = router_rows.iter().fold((0, 0, 0), |mut counts, row| {
        match string_field(row, "current_disposition") {
            "available" => counts.0 += 1,
            "neutral" => counts.1 += 1,
            "unavailable" => counts.2 += 1,
            other => panic!("unexpected RouterInfo disposition {other}"),
        }
        counts
    });
    assert_eq!(counts, (37, 1, 5));

    let expected_unavailable = BTreeSet::from([
        "i2p.router.net.bw.transit.15s".to_owned(),
        "i2p.router.news".to_owned(),
        "i2p.router.net.error".to_owned(),
        "i2p.router.net.error.v6".to_owned(),
        "i2p.router.netdb.bannedpeers".to_owned(),
    ]);
    let actual_unavailable: BTreeSet<String> = router_rows
        .iter()
        .filter(|row| string_field(row, "current_disposition") == "unavailable")
        .map(|row| string_field(row, "key").to_owned())
        .collect();
    assert_eq!(actual_unavailable, expected_unavailable);
    for row in router_rows {
        let current = string_field(row, "current_disposition");
        let target = string_field(row, "final_target_disposition");
        if current == "unavailable" {
            assert!(matches!(
                string_field(row, "owning_completion_milestone"),
                "M100" | "M101" | "M102" | "M103"
            ));
            assert_eq!(target, "available");
        }
    }

    let setconfig_rows = rows(&root, "addressbook_setconfig", "rows");
    assert_eq!(setconfig_rows.len(), 13);
    assert_eq!(string_set(setconfig_rows, "key").len(), 13);
    for row in setconfig_rows {
        assert_ne!(string_field(row, "current_disposition"), "unknown");
        assert_ne!(string_field(row, "current_disposition"), "accept_inert");
        assert_eq!(string_field(row, "owning_milestone"), "M096");
    }

    let tunnel_types = root["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("canonical tunnel types")
        .iter()
        .map(|value| value.as_str().expect("tunnel type string"))
        .collect::<Vec<_>>();
    assert_eq!(tunnel_types.len(), 12);
    assert_eq!(tunnel_types.iter().collect::<BTreeSet<_>>().len(), 12);

    let options = rows(&root, "tunnel_manager", "options");
    let expected_options = [
        "Port",
        "TargetHost",
        "Host",
        "TargetPort",
        "TargetDestination",
        "Destination",
        "StartOnLoad",
        "Description",
        "ReachableBy",
        "Shared",
        "UseSSL",
        "TunnelLength",
        "TunnelVariance",
        "TunnelQuantity",
        "TunnelBackupQuantity",
        "SigType",
        "EncType",
        "CustomOptions",
        "ProxyList",
        "UseOutproxyPlugin",
        "ProxyAuth",
        "ProxyUsername",
        "ProxyPassword",
        "OutproxyAuth",
        "OutproxyUsername",
        "OutproxyPassword",
        "OutproxyType",
        "SSLProxies",
        "JumpList",
        "ConnectDelay",
        "Profile",
        "DelayOpen",
        "Reduce",
        "ReduceCount",
        "ReduceTime",
        "Close",
        "CloseTime",
        "NewDest",
        "PersistentClientKey",
        "PrivKeyFile",
        "AllowUserAgent",
        "AllowReferer",
        "AllowAccept",
        "AllowInternalSSL",
        "WebsiteHostname",
        "SpoofedHost",
        "BlockAccessInProxies",
        "BlockUserAgents",
        "UserAgents",
        "UniqueLocalAddressPerClient",
        "BlockReferers",
        "MultiHoming",
        "AccessOption",
        "AccessList",
        "FilterFilePath",
        "MaxConcurrentConns",
        "ClientPerMinute",
        "ClientPerHour",
        "ClientPerDay",
        "TotalInPerMinute",
        "TotalInPerHour",
        "TotalInPerDay",
        "PostLimit",
        "PostLimitTime",
        "PerClientPeriod",
        "TotalPeriod",
        "TotalBanTime",
        "EncryptLeaseSet",
        "OptionalLookup",
        "LeaseSetClientAuths",
    ];
    assert_eq!(options.len(), expected_options.len());
    assert_eq!(
        options.iter().map(|row| string_field(row, "canonical_key")).collect::<Vec<_>>(),
        expected_options
    );
    let allowed = BTreeSet::from([
        "apply",
        "planned_apply",
        "not_applicable",
        "blocked_primitive",
    ]);
    assert_eq!(tunnel_types.len(), 12);
    for row in options {
        let cells = row["cells"].as_array().expect("one cell per canonical tunnel type");
        assert_eq!(cells.len(), tunnel_types.len());
        let notes = row["cell_notes"].as_table().expect("cell rationale table");
        for (index, cell) in cells.iter().enumerate() {
            let disposition = cell.as_str().expect("cell disposition string");
            assert!(
                allowed.contains(disposition),
                "unknown cell disposition {disposition}"
            );
            if disposition != "apply" && disposition != "planned_apply" {
                assert!(notes.contains_key(tunnel_types[index]));
            }
        }
        if string_field(row, "canonical_key") == "PrivKeyFile" {
            assert_eq!(string_field(row, "blocking_milestone"), "M097");
            assert!(string_field(row, "blocked_primitive").contains("private-key import"));
        }
    }

    let selectors = root["contract_names"]["client_services_selectors"]
        .as_array()
        .expect("ClientServicesInfo selectors");
    assert_eq!(selectors.len(), 6);
    let methods = rows(&root, "method_inventory", "methods");
    assert!(methods
        .iter()
        .all(|row| string_field(row, "disposition").contains("outside_proposal_170_scope")));
    let budgets = rows(&root, "containment", "budgets");
    assert_eq!(budgets.len(), 8);
    assert_eq!(
        string_set(budgets, "milestone"),
        BTreeSet::from([
            "M096".to_owned(),
            "M097".to_owned(),
            "M098".to_owned(),
            "M099".to_owned(),
            "M100".to_owned(),
            "M101".to_owned(),
            "M102".to_owned(),
            "M103".to_owned(),
        ])
    );
    assert!(
        budgets.iter().any(|row| string_field(row, "milestone") == "M102"
            && row["non_i2pcontrol_production_path_required"] == Value::Boolean(true))
    );

    fn assert_no_forbidden(value: &Value) {
        match value {
            Value::String(text) => {
                assert!(!matches!(text.as_str(), "unknown" | "accept_inert"));
            }
            Value::Array(values) => values.iter().for_each(assert_no_forbidden),
            Value::Table(table) => table.values().for_each(assert_no_forbidden),
            _ => {}
        }
    }
    assert_no_forbidden(&root);
}
