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

fn residual_primitive_map() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../plans/implementation/i2pcontrol-proposal-170/131-residual-primitive-map.toml");
    let raw = std::fs::read_to_string(path).expect("M131 residual map must exist");
    raw.parse().expect("M131 residual map must be valid TOML")
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
    assert_eq!(counts, (42, 1, 0));

    let expected_unavailable = BTreeSet::new();
    let actual_unavailable: BTreeSet<String> = router_rows
        .iter()
        .filter(|row| string_field(row, "current_disposition") == "unavailable")
        .map(|row| string_field(row, "key").to_owned())
        .collect();
    assert_eq!(actual_unavailable, expected_unavailable);
    let banned = router_rows
        .iter()
        .find(|row| string_field(row, "key") == "i2p.router.netdb.bannedpeers")
        .expect("banned-peer row must remain in the matrix");
    assert_eq!(string_field(banned, "current_disposition"), "available");
    assert_eq!(
        string_field(banned, "production_owner_source"),
        "router-ban-empty-marker"
    );
    for row in router_rows {
        let current = string_field(row, "current_disposition");
        let target = string_field(row, "final_target_disposition");
        if current == "unavailable" {
            assert!(matches!(
                string_field(row, "owning_completion_milestone"),
                "M102" | "M103"
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
                assert!(
                    notes.contains_key(tunnel_types[index]),
                    "{} cell {} needs a rationale",
                    string_field(row, "canonical_key"),
                    tunnel_types[index]
                );
            }
        }
        if string_field(row, "canonical_key") == "PrivKeyFile" {
            assert_eq!(string_field(row, "completion_owner"), "M110");
            assert_eq!(cells[6].as_str(), Some("not_applicable"));
        }
    }

    let option = |key: &str| {
        options
            .iter()
            .find(|row| string_field(row, "canonical_key") == key)
            .unwrap_or_else(|| panic!("M098 option {key} must be present"))
    };
    let new_dest = option("NewDest");
    assert_eq!(string_field(new_dest, "completion_owner"), "M112");
    assert_eq!(
        string_field(new_dest, "current_or_planned_disposition"),
        "apply_or_blocked_primitive_or_not_applicable"
    );
    let new_dest_cells = new_dest["cells"].as_array().unwrap();
    // M121 §5.2 demotes all six TCP NewDest cells to blocked_primitive.
    assert!(new_dest_cells[..6]
        .iter()
        .all(|cell| cell.as_str() == Some("blocked_primitive")));
    assert!(new_dest_cells[6..].iter().all(|cell| cell.as_str() == Some("not_applicable")));
    assert_eq!(string_field(new_dest, "blocking_milestone"), "M121");
    for tunnel_type in &tunnel_types[..7] {
        assert!(new_dest["cell_notes"].as_table().unwrap().contains_key(*tunnel_type));
    }
    // M121 Outcome C demotes all ten applicable SigType cells.
    let sig_type = option("SigType");
    assert_eq!(string_field(sig_type, "completion_owner"), "M111");
    let sig_cells = sig_type["cells"].as_array().unwrap();
    for index in [0, 1, 2, 3, 4, 5, 7, 8, 9, 10] {
        assert_eq!(
            sig_cells[index].as_str(),
            Some("blocked_primitive"),
            "SigType cell {index}"
        );
    }
    for index in [6, 11] {
        assert_eq!(
            sig_cells[index].as_str(),
            Some("not_applicable"),
            "SigType cell {index}"
        );
    }
    assert_eq!(string_field(sig_type, "blocking_milestone"), "M121");
    assert!(!string_field(sig_type, "blocked_primitive").is_empty());
    for key in [
        "ProxyList",
        "ProxyAuth",
        "ProxyUsername",
        "ProxyPassword",
        "OutproxyAuth",
        "OutproxyUsername",
        "OutproxyPassword",
        "OutproxyType",
    ] {
        let row = option(key);
        let cells = row["cells"].as_array().unwrap();
        for index in [1, 3, 4, 5] {
            assert_eq!(cells[index].as_str(), Some("apply"), "{key} cell {index}");
        }
        assert_eq!(string_field(row, "completion_owner"), "M098");
    }
    for key in ["UseOutproxyPlugin"] {
        let row = option(key);
        let cells = row["cells"].as_array().unwrap();
        for index in [1, 3, 4, 5] {
            assert_eq!(
                cells[index].as_str(),
                Some("blocked_primitive"),
                "{key} cell {index}"
            );
        }
        assert_eq!(string_field(row, "completion_owner"), "M112");
        assert!(!string_field(row, "blocked_primitive").is_empty());
        assert!(!string_field(row, "blocking_milestone").is_empty());
    }
    for key in ["SSLProxies", "JumpList"] {
        let row = option(key);
        let cells = row["cells"].as_array().unwrap();
        assert_eq!(
            cells[1].as_str(),
            Some("blocked_primitive"),
            "{key} HTTP cell"
        );
        for index in [3, 4, 5] {
            assert_eq!(
                cells[index].as_str(),
                Some("not_applicable"),
                "{key} cell {index}"
            );
        }
        assert_eq!(string_field(row, "completion_owner"), "M112");
        assert!(!string_field(row, "blocked_primitive").is_empty());
        assert!(!string_field(row, "blocking_milestone").is_empty());
    }
    for key in [
        "ConnectDelay",
        "Profile",
        "DelayOpen",
        "Reduce",
        "ReduceCount",
        "ReduceTime",
        "Close",
        "CloseTime",
    ] {
        let row = option(key);
        let cells = row["cells"].as_array().unwrap();
        if key == "DelayOpen" {
            for (index, cell) in cells.iter().enumerate().take(6) {
                assert_eq!(cell.as_str(), Some("apply"), "{key} cell {index}");
            }
            assert!(string_field(row, "completion_owner").starts_with("M106"));
            assert_eq!(cells[6].as_str(), Some("not_applicable"));
            continue;
        }
        if matches!(key, "ConnectDelay") {
            for (index, cell) in cells.iter().enumerate().take(6) {
                assert_eq!(cell.as_str(), Some("apply"), "{key} cell {index}");
            }
            assert_eq!(cells[6].as_str(), Some("blocked_primitive"));
        } else if matches!(key, "Reduce" | "ReduceCount" | "ReduceTime") {
            // M136 promotes all seven client Reduce* cells to apply.
            for (index, cell) in cells.iter().enumerate().take(7) {
                assert_eq!(cell.as_str(), Some("apply"), "{key} cell {index}");
            }
            for (index, cell) in cells.iter().enumerate().skip(7) {
                assert_eq!(
                    cell.as_str(),
                    Some("not_applicable"),
                    "{key} server cell {index}"
                );
            }
            assert_eq!(string_field(row, "completion_owner"), "M136");
            assert_eq!(
                string_field(row, "current_or_planned_disposition"),
                "apply_or_not_applicable"
            );
            continue;
        } else if matches!(key, "Close" | "CloseTime") {
            // M121 §5.2 demotes all six TCP Close/CloseTime cells.
            for (index, cell) in cells.iter().enumerate().take(7) {
                assert_eq!(
                    cell.as_str(),
                    Some("blocked_primitive"),
                    "{key} cell {index}"
                );
            }
            assert_eq!(string_field(row, "blocking_milestone"), "M121");
        } else {
            for (index, cell) in cells.iter().enumerate().take(7) {
                assert_eq!(
                    cell.as_str(),
                    Some("blocked_primitive"),
                    "{key} cell {index}"
                );
            }
        }
        assert_eq!(string_field(row, "completion_owner"), "M112");
        assert!(!string_field(row, "blocked_primitive").is_empty());
        assert!(!string_field(row, "blocking_milestone").is_empty());
    }
    for key in ["AllowUserAgent", "AllowReferer", "AllowAccept"] {
        let row = option(key);
        let cells = row["cells"].as_array().unwrap();
        assert_eq!(cells[1].as_str(), Some("apply"), "{key} HTTP client cell");
        assert_eq!(
            cells[8].as_str(),
            Some("apply"),
            "{key} HTTP server handoff"
        );
        assert_eq!(cells[9].as_str(), Some("apply"), "{key} HTTP bidir handoff");
    }
    let internal_ssl = option("AllowInternalSSL");
    assert_eq!(
        internal_ssl["cells"].as_array().unwrap()[1].as_str(),
        Some("not_applicable")
    );
    assert_eq!(
        internal_ssl["cells"].as_array().unwrap()[8].as_str(),
        Some("not_applicable")
    );
    assert_eq!(
        internal_ssl["cells"].as_array().unwrap()[9].as_str(),
        Some("not_applicable")
    );

    for key in [
        "WebsiteHostname",
        "SpoofedHost",
        "BlockAccessInProxies",
        "BlockUserAgents",
        "UserAgents",
        "BlockReferers",
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
    ] {
        let row = option(key);
        let cells = row["cells"].as_array().unwrap();
        assert_eq!(string_field(row, "completion_owner"), "M099", "{key} owner");
        let expected = if matches!(
            key,
            "WebsiteHostname"
                | "SpoofedHost"
                | "BlockAccessInProxies"
                | "BlockUserAgents"
                | "UserAgents"
                | "BlockReferers"
        ) {
            [false, true, true, false]
        } else if key == "FilterFilePath" || key == "PostLimit" || key == "PostLimitTime" {
            [false, true, true, true]
        } else {
            [true, true, true, true]
        };
        for (offset, should_apply) in expected.into_iter().enumerate() {
            let index = offset + 7;
            assert_eq!(
                cells[index].as_str(),
                Some(if should_apply {
                    "apply"
                } else {
                    "not_applicable"
                }),
                "{key} cell {index}"
            );
        }
    }
    for key in [
        "UniqueLocalAddressPerClient",
        "MultiHoming",
        "EncryptLeaseSet",
        "OptionalLookup",
        "LeaseSetClientAuths",
    ] {
        let row = option(key);
        let cells = row["cells"].as_array().unwrap();
        assert_eq!(
            string_field(row, "completion_owner"),
            "residual-option-line",
            "{key} owner"
        );
        assert!(
            !string_field(row, "blocked_primitive").is_empty(),
            "{key} primitive"
        );
        assert!(
            !string_field(row, "blocking_milestone").is_empty(),
            "{key} milestone"
        );
        let (first, last) = match key {
            "AllowInternalSSL" | "UniqueLocalAddressPerClient" | "MultiHoming" => (8, 9),
            _ => (7, 11),
        };
        for (index, cell) in cells.iter().enumerate().skip(first).take(last - first + 1) {
            assert_eq!(
                cell.as_str(),
                Some("blocked_primitive"),
                "{key} cell {index}"
            );
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

#[test]
fn current_matrix_counts_are_explicit_and_exact() {
    let root = matrix();
    let options = rows(&root, "tunnel_manager", "options");
    let counts = options.iter().flat_map(|row| row["cells"].as_array().unwrap()).fold(
        (0, 0, 0),
        |mut counts, cell| {
            match cell.as_str().expect("cell disposition") {
                "apply" => counts.0 += 1,
                "blocked_primitive" => counts.1 += 1,
                "not_applicable" => counts.2 += 1,
                "planned_apply" => panic!("planned_apply must not remain in the closed matrix"),
                other => panic!("unexpected cell disposition {other}"),
            }
            counts
        },
    );
    assert_eq!(counts, (305, 67, 468));
    let declared = root
        .get("current_matrix_counts")
        .and_then(Value::as_table)
        .expect("current matrix counts are declared");
    assert_eq!(declared["apply"].as_integer(), Some(305));
    assert_eq!(declared["blocked_primitive"].as_integer(), Some(67));
    assert_eq!(declared["not_applicable"].as_integer(), Some(468));
}

#[test]
fn m131_residual_map_is_cell_complete_and_closed() {
    let root = residual_primitive_map();
    assert_eq!(
        root["baseline"]["starting_cell_count"].as_integer(),
        Some(96)
    );
    assert_eq!(root["baseline"]["cell_record_count"].as_integer(), Some(96));
    assert_eq!(root["baseline"]["final_apply"].as_integer(), Some(284));
    assert_eq!(
        root["baseline"]["final_blocked_primitive"].as_integer(),
        Some(88)
    );
    assert_eq!(
        root["baseline"]["final_not_applicable"].as_integer(),
        Some(468)
    );
    assert_eq!(root["baseline"]["apply_promotions"].as_integer(), Some(0));

    let cells = root["cells"].as_array().expect("M131 cell records");
    let identities = cells
        .iter()
        .map(|cell| {
            (
                string_field(cell, "canonical_option").to_owned(),
                string_field(cell, "tunnel_family").to_owned(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(cells.len(), 96);
    assert_eq!(identities.len(), 96);
    assert!(cells.iter().all(|cell| {
        string_field(cell, "starting_disposition") == "blocked_primitive"
            && !string_field(cell, "applicability_evidence").is_empty()
            && !string_field(cell, "canonical_owner").is_empty()
            && !string_field(cell, "path_budget").is_empty()
    }));
    let final_counts = cells.iter().fold((0, 0), |mut counts, cell| {
        match string_field(cell, "final_disposition") {
            "blocked_primitive" => counts.0 += 1,
            "not_applicable" => counts.1 += 1,
            other => panic!("unexpected M131 final disposition {other}"),
        }
        counts
    });
    assert_eq!(final_counts, (88, 8));
    let reclassified = cells
        .iter()
        .filter(|cell| string_field(cell, "final_disposition") == "not_applicable")
        .map(|cell| {
            format!(
                "{}:{}",
                string_field(cell, "canonical_option"),
                string_field(cell, "tunnel_family")
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        reclassified,
        BTreeSet::from([
            "SSLProxies:socks".to_owned(),
            "SSLProxies:socksirc".to_owned(),
            "SSLProxies:connectclient".to_owned(),
            "JumpList:socks".to_owned(),
            "JumpList:socksirc".to_owned(),
            "JumpList:connectclient".to_owned(),
            "DelayOpen:streamrclient".to_owned(),
            "NewDest:streamrclient".to_owned(),
        ])
    );
    assert_eq!(root["clusters"].as_array().unwrap().len(), 10);
    assert!(!root["dependency_graph"]["ordered_edges"].as_array().unwrap().is_empty());
    assert_eq!(
        root["future_handoff"]["next_dependency_ready"].as_bool(),
        Some(false)
    );
}
