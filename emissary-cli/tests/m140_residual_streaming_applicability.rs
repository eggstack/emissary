//! M140 residual streaming applicability guard.
//!
//! The M140 map is planning evidence, not a runtime capability source. This
//! test keeps the eight-cell re-freeze mechanically tied to the M095 machine
//! authority without requiring a router, network access, or production work.

#![cfg(feature = "i2pcontrol")]

use std::{collections::BTreeSet, path::Path};

use toml::Value;

fn planning_toml(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../plans/implementation/i2pcontrol-proposal-170")
        .join(name);
    let raw = std::fs::read_to_string(&path).expect("M140 planning file must exist");
    raw.parse().expect("M140 planning file must be valid TOML")
}

fn string_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("M140 record is missing string field {key}"))
}

#[test]
fn m140_map_has_exactly_eight_source_backed_records() {
    let root = planning_toml("140-residual-streaming-applicability-map.toml");

    assert_eq!(root["baseline"]["milestone"].as_str(), Some("M140"));
    assert_eq!(root["baseline"]["starting_apply"].as_integer(), Some(325));
    assert_eq!(
        root["baseline"]["starting_blocked_primitive"].as_integer(),
        Some(47)
    );
    assert_eq!(
        root["baseline"]["starting_not_applicable"].as_integer(),
        Some(468)
    );
    assert_eq!(root["baseline"]["final_apply"].as_integer(), Some(325));
    assert_eq!(
        root["baseline"]["final_blocked_primitive"].as_integer(),
        Some(40)
    );
    assert_eq!(
        root["baseline"]["final_not_applicable"].as_integer(),
        Some(475)
    );
    assert_eq!(root["baseline"]["apply_promotions"].as_integer(), Some(0));
    assert_eq!(root["baseline"]["cell_record_count"].as_integer(), Some(8));
    assert_eq!(
        root["baseline"]["reclassified_cell_count"].as_integer(),
        Some(7)
    );
    assert_eq!(
        root["baseline"]["retained_cell_count"].as_integer(),
        Some(1)
    );

    // Pinned source authority must be exact.
    assert_eq!(
        root["references"]["proposal_sha256"].as_str(),
        Some("f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc")
    );
    assert_eq!(
        root["references"]["java_i2pcontrol_head"].as_str(),
        Some("45bb593000408071dd376b78848fdc246dccd964")
    );
    assert_eq!(
        root["references"]["java_i2ptunnel_head"].as_str(),
        Some("2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243")
    );
    assert_eq!(
        root["references"]["yosemite_revision"].as_str(),
        Some("59140a2277bf296928d2e8ce39a148182eeff044")
    );
    assert_eq!(
        root["disposition_rules"]["apply_promotions"].as_integer(),
        Some(0)
    );

    let cells = root["cells"].as_array().expect("M140 cell records");
    assert_eq!(cells.len(), 8);
    let identities = cells
        .iter()
        .map(|cell| {
            (
                string_field(cell, "canonical_option").to_owned(),
                string_field(cell, "tunnel_family").to_owned(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(identities.len(), 8);
    assert_eq!(
        identities,
        BTreeSet::from([
            ("Profile".to_owned(), "client".to_owned()),
            ("Profile".to_owned(), "httpclient".to_owned()),
            ("Profile".to_owned(), "ircclient".to_owned()),
            ("Profile".to_owned(), "socks".to_owned()),
            ("Profile".to_owned(), "socksirc".to_owned()),
            ("Profile".to_owned(), "connectclient".to_owned()),
            ("Profile".to_owned(), "streamrclient".to_owned()),
            ("ConnectDelay".to_owned(), "streamrclient".to_owned()),
        ])
    );

    // Every record must carry cell-complete evidence: starting disposition,
    // Proposal/parser/runtime/override/wire/data-plane fields, final
    // disposition, rationale/sources, and an explicit future-primitive flag.
    for cell in cells {
        assert_eq!(
            string_field(cell, "starting_disposition"),
            "blocked_primitive"
        );
        for key in [
            "proposal_evidence",
            "java_parser_behavior",
            "java_runtime_class",
            "java_constructor_hierarchy",
            "emitted_property",
            "runtime_consumer",
            "constructor_override",
            "yosemite_wire_capability",
            "emissary_data_plane",
            "rationale",
            "source_locations",
        ] {
            assert!(!string_field(cell, key).is_empty(), "M140 cell needs {key}");
        }
        assert!(cell
            .get("future_primitive_required")
            .and_then(|value| value.as_bool())
            .is_some());
        assert!(
            matches!(
                string_field(cell, "final_disposition"),
                "blocked_primitive" | "not_applicable"
            ),
            "M140 has zero apply-promotion budget"
        );
    }

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
            "Profile:httpclient".to_owned(),
            "Profile:ircclient".to_owned(),
            "Profile:socks".to_owned(),
            "Profile:socksirc".to_owned(),
            "Profile:connectclient".to_owned(),
            "Profile:streamrclient".to_owned(),
            "ConnectDelay:streamrclient".to_owned(),
        ])
    );
    let retained = cells
        .iter()
        .filter(|cell| string_field(cell, "final_disposition") == "blocked_primitive")
        .map(|cell| {
            format!(
                "{}:{}",
                string_field(cell, "canonical_option"),
                string_field(cell, "tunnel_family")
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(retained, BTreeSet::from(["Profile:client".to_owned()]));

    // Only the retained client cell may still require a future primitive.
    for cell in cells {
        let needs_primitive = cell["future_primitive_required"].as_bool().unwrap();
        if string_field(cell, "final_disposition") == "not_applicable" {
            assert!(!needs_primitive, "N/A cells require no future primitive");
        } else {
            assert!(needs_primitive, "retained Profile:client still needs M143");
        }
    }

    // Counts must be mechanically derived: 47 - 7 = 40 blocked, 468 + 7 = 475 N/A.
    let counts = root["counts"].as_table().expect("M140 counts");
    assert_eq!(counts["pre_apply"].as_integer(), Some(325));
    assert_eq!(counts["pre_blocked"].as_integer(), Some(47));
    assert_eq!(counts["pre_not_applicable"].as_integer(), Some(468));
    assert_eq!(counts["post_apply"].as_integer(), Some(325));
    assert_eq!(counts["post_blocked"].as_integer(), Some(40));
    assert_eq!(counts["post_not_applicable"].as_integer(), Some(475));
    assert_eq!(counts["delta_blocked_to_na"].as_integer(), Some(7));
    assert_eq!(counts["apply_delta"].as_integer(), Some(0));
}

#[test]
fn m140_matrix_reconciliation_is_exact_and_contained() {
    let matrix = planning_toml("095-full-support-matrix.toml");
    let map = planning_toml("140-residual-streaming-applicability-map.toml");

    // Declared M095 counts must match the M140 post counts and recomputation.
    let declared = matrix["current_matrix_counts"].as_table().expect("counts");
    assert_eq!(declared["apply"].as_integer(), Some(325));
    assert_eq!(declared["blocked_primitive"].as_integer(), Some(40));
    assert_eq!(declared["not_applicable"].as_integer(), Some(475));
    assert_eq!(
        declared["blocked_primitive"].as_integer(),
        map["counts"]["post_blocked"].as_integer()
    );
    assert_eq!(
        declared["not_applicable"].as_integer(),
        map["counts"]["post_not_applicable"].as_integer()
    );

    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("canonical tunnel types");
    let mut counts = [0usize; 3];
    let mut blocked = BTreeSet::new();
    for row in matrix["tunnel_manager"]["options"].as_array().expect("TunnelManager options") {
        let option = row["canonical_key"].as_str().expect("canonical option");
        let cells = row["cells"].as_array().expect("option cells");
        assert_eq!(cells.len(), tunnel_types.len());
        for (index, cell) in cells.iter().enumerate() {
            match cell.as_str().expect("cell disposition") {
                "apply" => counts[0] += 1,
                "blocked_primitive" => {
                    counts[1] += 1;
                    blocked.insert((
                        option.to_owned(),
                        tunnel_types[index].as_str().expect("family").to_owned(),
                    ));
                }
                "not_applicable" => counts[2] += 1,
                other => panic!("unexpected cell disposition {other}"),
            }
        }
    }
    assert_eq!(counts, [325, 40, 475]);

    // Every changed disposition must be among the eight M140 candidates and no
    // cell may have been promoted to apply (apply count unchanged at 325).
    let candidates = BTreeSet::from([
        ("Profile".to_owned(), "client".to_owned()),
        ("Profile".to_owned(), "httpclient".to_owned()),
        ("Profile".to_owned(), "ircclient".to_owned()),
        ("Profile".to_owned(), "socks".to_owned()),
        ("Profile".to_owned(), "socksirc".to_owned()),
        ("Profile".to_owned(), "connectclient".to_owned()),
        ("Profile".to_owned(), "streamrclient".to_owned()),
        ("ConnectDelay".to_owned(), "streamrclient".to_owned()),
    ]);
    let pre_blocked = BTreeSet::from([
        ("ConnectDelay".to_owned(), "streamrclient".to_owned()),
        ("EncryptLeaseSet".to_owned(), "server".to_owned()),
        ("EncryptLeaseSet".to_owned(), "httpserver".to_owned()),
        ("EncryptLeaseSet".to_owned(), "httpbidirserver".to_owned()),
        ("EncryptLeaseSet".to_owned(), "ircserver".to_owned()),
        ("EncryptLeaseSet".to_owned(), "streamrserver".to_owned()),
        ("JumpList".to_owned(), "httpclient".to_owned()),
        ("LeaseSetClientAuths".to_owned(), "server".to_owned()),
        ("LeaseSetClientAuths".to_owned(), "httpserver".to_owned()),
        (
            "LeaseSetClientAuths".to_owned(),
            "httpbidirserver".to_owned(),
        ),
        ("LeaseSetClientAuths".to_owned(), "ircserver".to_owned()),
        ("LeaseSetClientAuths".to_owned(), "streamrserver".to_owned()),
        ("MultiHoming".to_owned(), "httpserver".to_owned()),
        ("MultiHoming".to_owned(), "httpbidirserver".to_owned()),
        ("OptionalLookup".to_owned(), "server".to_owned()),
        ("OptionalLookup".to_owned(), "httpserver".to_owned()),
        ("OptionalLookup".to_owned(), "httpbidirserver".to_owned()),
        ("OptionalLookup".to_owned(), "ircserver".to_owned()),
        ("OptionalLookup".to_owned(), "streamrserver".to_owned()),
        ("Profile".to_owned(), "client".to_owned()),
        ("Profile".to_owned(), "httpclient".to_owned()),
        ("Profile".to_owned(), "ircclient".to_owned()),
        ("Profile".to_owned(), "socks".to_owned()),
        ("Profile".to_owned(), "socksirc".to_owned()),
        ("Profile".to_owned(), "connectclient".to_owned()),
        ("Profile".to_owned(), "streamrclient".to_owned()),
        ("SSLProxies".to_owned(), "httpclient".to_owned()),
        ("SigType".to_owned(), "client".to_owned()),
        ("SigType".to_owned(), "httpclient".to_owned()),
        ("SigType".to_owned(), "ircclient".to_owned()),
        ("SigType".to_owned(), "socks".to_owned()),
        ("SigType".to_owned(), "socksirc".to_owned()),
        ("SigType".to_owned(), "connectclient".to_owned()),
        ("SigType".to_owned(), "server".to_owned()),
        ("SigType".to_owned(), "httpserver".to_owned()),
        ("SigType".to_owned(), "httpbidirserver".to_owned()),
        ("SigType".to_owned(), "ircserver".to_owned()),
        (
            "UniqueLocalAddressPerClient".to_owned(),
            "httpserver".to_owned(),
        ),
        (
            "UniqueLocalAddressPerClient".to_owned(),
            "httpbidirserver".to_owned(),
        ),
        ("UseOutproxyPlugin".to_owned(), "httpclient".to_owned()),
        ("UseOutproxyPlugin".to_owned(), "socks".to_owned()),
        ("UseOutproxyPlugin".to_owned(), "socksirc".to_owned()),
        ("UseOutproxyPlugin".to_owned(), "connectclient".to_owned()),
        ("UseSSL".to_owned(), "httpclient".to_owned()),
        ("UseSSL".to_owned(), "connectclient".to_owned()),
        ("UseSSL".to_owned(), "httpserver".to_owned()),
        ("UseSSL".to_owned(), "httpbidirserver".to_owned()),
    ]);
    assert_eq!(pre_blocked.len(), 47);
    let removed = pre_blocked.difference(&blocked).cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        removed,
        BTreeSet::from([
            ("Profile".to_owned(), "httpclient".to_owned()),
            ("Profile".to_owned(), "ircclient".to_owned()),
            ("Profile".to_owned(), "socks".to_owned()),
            ("Profile".to_owned(), "socksirc".to_owned()),
            ("Profile".to_owned(), "connectclient".to_owned()),
            ("Profile".to_owned(), "streamrclient".to_owned()),
            ("ConnectDelay".to_owned(), "streamrclient".to_owned()),
        ])
    );
    for cell in &removed {
        assert!(
            candidates.contains(cell),
            "changed cell must be an M140 candidate"
        );
    }
    assert!(
        blocked.is_subset(&pre_blocked),
        "no new blockers may appear"
    );
    assert!(blocked.contains(&("Profile".to_owned(), "client".to_owned())));
}
