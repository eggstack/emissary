use std::{collections::BTreeSet, fs, path::PathBuf};

use toml::Value;

fn planning_file(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../plans/implementation/i2pcontrol-proposal-170")
        .join(name)
}

fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string field {key}"))
}

fn main() {}

#[test]
fn audit_covers_the_exact_m104_residual_inventory() {
    let matrix: Value =
        toml::from_str(&fs::read_to_string(planning_file("095-full-support-matrix.toml")).unwrap())
            .unwrap();
    let audit: Value = toml::from_str(
        &fs::read_to_string(planning_file("105-residual-option-audit.toml")).unwrap(),
    )
    .unwrap();

    assert_eq!(string(&audit["audit"], "milestone"), "M105");
    assert_eq!(string(&audit["audit"], "status"), "closed");
    assert_eq!(
        string(&audit["audit"], "input_disposition"),
        "blocked_primitive"
    );
    assert_eq!(audit["audit"]["record_count"].as_integer(), Some(164));
    assert_eq!(
        audit["audit"]["m095_matrix_sha256"].as_str(),
        Some("fcc7d21dd886cd96ac614507abba5e3cfc806cee942ebbb09eb387e1a60078ac")
    );

    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"].as_array().unwrap();
    let options = matrix["tunnel_manager"]["options"].as_array().unwrap();
    let current_blocked: BTreeSet<(String, String)> = options
        .iter()
        .flat_map(|row| {
            let key = string(row, "canonical_key").to_owned();
            row["cells"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.as_str() == Some("blocked_primitive"))
                .map(move |(index, _)| {
                    (
                        key.clone(),
                        tunnel_types[index].as_str().unwrap().to_owned(),
                    )
                })
        })
        .collect();

    let allowed = BTreeSet::from([
        "i2pcontrol_local_candidate",
        "neutral_owner_candidate",
        "dependency_blocked",
        "architecture_decision_required",
        "not_applicable_candidate",
        "semantic_blocked",
    ]);
    let cells = audit["cells"].as_array().unwrap();
    assert_eq!(cells.len(), 164);

    let mut actual = BTreeSet::new();
    for cell in cells {
        let identity = (
            string(cell, "canonical_option").to_owned(),
            string(cell, "tunnel_type").to_owned(),
        );
        assert!(actual.insert(identity), "duplicate audit cell");
        assert_eq!(
            cell["applicable_under_pinned_contract"].as_bool(),
            Some(true)
        );
        assert!(!string(cell, "pinned_semantic_summary").is_empty());
        assert!(!string(cell, "current_m095_blocker").is_empty());
        assert!(!string(cell, "current_emissary_owner").is_empty());
        assert!(!string(cell, "current_yosemite_sam_primitive_or_wire_path").is_empty());
        assert!(!string(cell, "reference_implementation_behavior").is_empty());
        assert!(!string(cell, "required_runtime_effect").is_empty());
        assert!(!string(cell, "security_anonymity_implications").is_empty());
        assert!(!string(cell, "persistence_key_secret_path_implications").is_empty());

        let disposition = string(cell, "audit_disposition");
        assert!(allowed.contains(disposition), "invalid audit disposition");
        let paths = cell["exact_candidate_production_paths"].as_array().unwrap();
        if disposition == "i2pcontrol_local_candidate" || disposition == "neutral_owner_candidate" {
            assert!(!string(cell, "candidate_implementation_owner").is_empty());
            assert!(
                !paths.is_empty(),
                "candidate must name exact production paths"
            );
        }
        if disposition == "dependency_blocked" {
            assert!(cell["dependency_or_cargo_lock_change_required"].as_bool() == Some(true));
            assert!(string(cell, "current_yosemite_sam_primitive_or_wire_path").len() > 20);
        }
        if disposition == "not_applicable_candidate" {
            assert!(string(cell, "applicability_evidence").contains("affirmative"));
        }
        if disposition == "semantic_blocked" {
            assert!(
                string(cell, "reference_implementation_behavior").contains("unresolved")
                    || string(cell, "reference_implementation_behavior").contains("define")
                    || string(cell, "reference_implementation_behavior").contains("definition")
                    || string(cell, "reference_implementation_behavior").contains("speculative")
            );
        }
    }

    let candidates: BTreeSet<(String, String)> = cells
        .iter()
        .filter(|cell| string(cell, "audit_disposition") == "i2pcontrol_local_candidate")
        .map(|cell| {
            (
                string(cell, "canonical_option").to_owned(),
                string(cell, "tunnel_type").to_owned(),
            )
        })
        .collect();
    assert_eq!(candidates.len(), 6);
    assert_eq!(actual.len(), 164);
    let m110_completed = BTreeSet::from([
        ("Shared", "client"),
        ("Shared", "httpclient"),
        ("Shared", "ircclient"),
        ("Shared", "socks"),
        ("Shared", "socksirc"),
        ("Shared", "connectclient"),
        ("Shared", "streamrclient"),
        ("NewDest", "client"),
        ("NewDest", "httpclient"),
        ("NewDest", "ircclient"),
        ("NewDest", "socks"),
        ("NewDest", "socksirc"),
        ("NewDest", "connectclient"),
        ("NewDest", "streamrclient"),
        ("PersistentClientKey", "client"),
        ("PersistentClientKey", "httpclient"),
        ("PersistentClientKey", "ircclient"),
        ("PersistentClientKey", "socks"),
        ("PersistentClientKey", "socksirc"),
        ("PersistentClientKey", "connectclient"),
        ("PersistentClientKey", "streamrclient"),
        ("PrivKeyFile", "client"),
        ("PrivKeyFile", "httpclient"),
        ("PrivKeyFile", "ircclient"),
        ("PrivKeyFile", "socks"),
        ("PrivKeyFile", "socksirc"),
        ("PrivKeyFile", "connectclient"),
        ("PrivKeyFile", "server"),
        ("PrivKeyFile", "httpserver"),
        ("PrivKeyFile", "httpbidirserver"),
        ("PrivKeyFile", "ircserver"),
    ]);
    assert_eq!(m110_completed.len(), 31);
    let m116_reclassified = BTreeSet::from([
        ("NewDest".to_owned(), "client".to_owned()),
        ("NewDest".to_owned(), "httpclient".to_owned()),
        ("NewDest".to_owned(), "ircclient".to_owned()),
        ("NewDest".to_owned(), "socks".to_owned()),
        ("NewDest".to_owned(), "socksirc".to_owned()),
        ("NewDest".to_owned(), "connectclient".to_owned()),
        ("NewDest".to_owned(), "streamrclient".to_owned()),
    ]);
    assert_eq!(m116_reclassified.len(), 7);
    let expected: BTreeSet<(String, String)> = actual
        .difference(&candidates)
        .filter(|cell| !m110_completed.contains(&(cell.0.as_str(), cell.1.as_str())))
        .cloned()
        .collect();
    let expected_with_m116 = expected.union(&m116_reclassified).cloned().collect::<BTreeSet<_>>();
    let m111_completed: BTreeSet<(String, String)> = [
        "TunnelVariance",
        "TunnelBackupQuantity",
        "SigType",
        "CustomOptions",
    ]
    .into_iter()
    .flat_map(|option| {
        [
            "client",
            "httpclient",
            "ircclient",
            "socks",
            "socksirc",
            "connectclient",
            "server",
            "httpserver",
            "httpbidirserver",
            "ircserver",
        ]
        .into_iter()
        .map(move |tunnel_type| (option.to_owned(), tunnel_type.to_owned()))
    })
    .collect();
    assert_eq!(m111_completed.len(), 40);
    let expected_post_m111 =
        expected_with_m116.difference(&m111_completed).cloned().collect::<BTreeSet<_>>();
    let m112_applied: BTreeSet<(String, String)> =
        ["ConnectDelay", "Close", "CloseTime", "NewDest"]
            .into_iter()
            .flat_map(|option| {
                [
                    "client",
                    "httpclient",
                    "ircclient",
                    "socks",
                    "socksirc",
                    "connectclient",
                ]
                .into_iter()
                .map(move |tunnel_type| (option.to_owned(), tunnel_type.to_owned()))
            })
            .collect();
    assert_eq!(m112_applied.len(), 24);
    let expected_post_m112 =
        expected_post_m111.difference(&m112_applied).cloned().collect::<BTreeSet<_>>();
    // M121 demotes 10 SigType + 18 Close/CloseTime/NewDest cells back to
    // blocked_primitive. The M105 input inventory is historical (164 rows);
    // the current matrix blocked set must equal the post-M112 blocked set
    // plus exactly the M121 demoted cells, less the two M125 classification
    // corrections.
    let m121_demoted: BTreeSet<(String, String)> = ["SigType"]
        .into_iter()
        .flat_map(|option| {
            [
                "client",
                "httpclient",
                "ircclient",
                "socks",
                "socksirc",
                "connectclient",
                "server",
                "httpserver",
                "httpbidirserver",
                "ircserver",
            ]
            .into_iter()
            .map(move |tunnel_type| (option.to_owned(), tunnel_type.to_owned()))
        })
        .chain(
            ["Close", "CloseTime", "NewDest"].into_iter().flat_map(|option| {
                [
                    "client",
                    "httpclient",
                    "ircclient",
                    "socks",
                    "socksirc",
                    "connectclient",
                ]
                .into_iter()
                .map(move |tunnel_type| (option.to_owned(), tunnel_type.to_owned()))
            }),
        )
        .collect();
    assert_eq!(m121_demoted.len(), 28);
    let expected_post_m121 =
        expected_post_m112.union(&m121_demoted).cloned().collect::<BTreeSet<_>>();
    let m125_reclassified = ["httpserver", "httpbidirserver"]
        .into_iter()
        .map(|tunnel_type| ("AllowInternalSSL".to_owned(), tunnel_type.to_owned()))
        .collect::<BTreeSet<_>>();
    let expected_post_m125 = expected_post_m121
        .difference(&m125_reclassified)
        .cloned()
        .collect::<BTreeSet<_>>();
    let m131_reclassified = [
        ("SSLProxies", "socks"),
        ("SSLProxies", "socksirc"),
        ("SSLProxies", "connectclient"),
        ("JumpList", "socks"),
        ("JumpList", "socksirc"),
        ("JumpList", "connectclient"),
        ("DelayOpen", "streamrclient"),
        ("NewDest", "streamrclient"),
    ]
    .into_iter()
    .map(|(option, tunnel_type)| (option.to_owned(), tunnel_type.to_owned()))
    .collect::<BTreeSet<_>>();
    let expected_post_m131 = expected_post_m125
        .difference(&m131_reclassified)
        .cloned()
        .collect::<BTreeSet<_>>();
    // M136 promotes the 21 Reduce* client cells to apply.
    let m136_completed: BTreeSet<(String, String)> = ["Reduce", "ReduceCount", "ReduceTime"]
        .into_iter()
        .flat_map(|option| {
            [
                "client",
                "httpclient",
                "ircclient",
                "socks",
                "socksirc",
                "connectclient",
                "streamrclient",
            ]
            .into_iter()
            .map(move |tunnel_type| (option.to_owned(), tunnel_type.to_owned()))
        })
        .collect();
    assert_eq!(m136_completed.len(), 21);
    let expected_post_m136 =
        expected_post_m131.difference(&m136_completed).cloned().collect::<BTreeSet<_>>();
    // M137 promotes the 14 Close/CloseTime client cells to apply.
    let m137_completed: BTreeSet<(String, String)> = ["Close", "CloseTime"]
        .into_iter()
        .flat_map(|option| {
            [
                "client",
                "httpclient",
                "ircclient",
                "socks",
                "socksirc",
                "connectclient",
                "streamrclient",
            ]
            .into_iter()
            .map(move |tunnel_type| (option.to_owned(), tunnel_type.to_owned()))
        })
        .collect();
    assert_eq!(m137_completed.len(), 14);
    let expected_post_m137 =
        expected_post_m136.difference(&m137_completed).cloned().collect::<BTreeSet<_>>();
    // M134 promotes the six non-Streamr TCP NewDest cells to apply.
    let m134_completed: BTreeSet<(String, String)> = ["NewDest"]
        .into_iter()
        .flat_map(|option| {
            [
                "client",
                "httpclient",
                "ircclient",
                "socks",
                "socksirc",
                "connectclient",
            ]
            .into_iter()
            .map(move |tunnel_type| (option.to_owned(), tunnel_type.to_owned()))
        })
        .collect();
    assert_eq!(m134_completed.len(), 6);
    let expected_post_m134 =
        expected_post_m137.difference(&m134_completed).cloned().collect::<BTreeSet<_>>();
    assert_eq!(current_blocked, expected_post_m134);
    assert_eq!(
        audit["summary"]["post_m116_reclassified_cells"].as_integer(),
        Some(7)
    );
    assert_eq!(
        audit["summary"]["post_m116_blocking_milestone"].as_str(),
        Some("M112")
    );
    assert_eq!(
        audit["summary"]["post_m112_matrix_apply_cells"].as_integer(),
        Some(312)
    );
    assert_eq!(
        audit["summary"]["post_m112_matrix_blocked_primitive_cells"].as_integer(),
        Some(70)
    );
    assert_eq!(
        audit["summary"]["post_m112_completed_cell_count"].as_integer(),
        Some(24)
    );
    assert_eq!(
        audit["summary"]["post_m121_matrix_apply_cells"].as_integer(),
        Some(284)
    );
    assert_eq!(
        audit["summary"]["post_m121_matrix_blocked_primitive_cells"].as_integer(),
        Some(98)
    );
    assert_eq!(
        audit["summary"]["post_m121_demoted_cell_count"].as_integer(),
        Some(28)
    );
    assert_eq!(
        audit["summary"]["post_m131_matrix_apply_cells"].as_integer(),
        Some(284)
    );
    assert_eq!(
        audit["summary"]["post_m131_matrix_blocked_primitive_cells"].as_integer(),
        Some(88)
    );
    assert_eq!(
        audit["summary"]["post_m131_matrix_not_applicable_cells"].as_integer(),
        Some(468)
    );
    assert_eq!(
        audit["summary"]["post_m131_reclassified_cell_count"].as_integer(),
        Some(8)
    );
}
