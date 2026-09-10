//! M153 post-M146 current-head requalification and authority-rebase guard.
//!
//! M153 is qualification/test/documentation work only: zero Proposal
//! promotions, zero production Rust/dependency/Yosemite changes. This suite is
//! the sole owner of the current-head aggregate assertion (`336 apply / 29
//! blocked_primitive / 475 not_applicable` across 840 TunnelManager
//! option/family cells) and of the exact 29-cell residual identity. All
//! earlier milestone suites (M126-M144) pin their milestone-local facts to
//! immutable closure evidence and must not re-pin current aggregates.
//!
//! M153 supersedes M139 as the current runtime/security qualification
//! authority; M139 remains the historical whole-surface authority for its
//! `325/47/468` head.

#![cfg(feature = "i2pcontrol")]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use toml::Value;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn planning_toml(name: &str) -> Value {
    let path = workspace_root().join("plans/implementation/i2pcontrol-proposal-170").join(name);
    let raw = std::fs::read_to_string(&path).expect("M153 planning file must exist");
    raw.parse().expect("M153 planning file must be valid TOML")
}

fn string_field<'a>(row: &'a Value, key: &str) -> &'a str {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("M153 record is missing string field {key}"))
}

/// The exact 29-cell residual identity at the M153 head: 10 `SigType`, 5
/// `EncryptLeaseSet`, 5 `OptionalLookup`, 5 `LeaseSetClientAuths`, 4
/// `UseOutproxyPlugin`.
fn expected_residuals() -> BTreeSet<(String, String)> {
    [
        ("SigType", "client"),
        ("SigType", "httpclient"),
        ("SigType", "ircclient"),
        ("SigType", "socks"),
        ("SigType", "socksirc"),
        ("SigType", "connectclient"),
        ("SigType", "server"),
        ("SigType", "httpserver"),
        ("SigType", "httpbidirserver"),
        ("SigType", "ircserver"),
        ("EncryptLeaseSet", "server"),
        ("EncryptLeaseSet", "httpserver"),
        ("EncryptLeaseSet", "httpbidirserver"),
        ("EncryptLeaseSet", "ircserver"),
        ("EncryptLeaseSet", "streamrserver"),
        ("OptionalLookup", "server"),
        ("OptionalLookup", "httpserver"),
        ("OptionalLookup", "httpbidirserver"),
        ("OptionalLookup", "ircserver"),
        ("OptionalLookup", "streamrserver"),
        ("LeaseSetClientAuths", "server"),
        ("LeaseSetClientAuths", "httpserver"),
        ("LeaseSetClientAuths", "httpbidirserver"),
        ("LeaseSetClientAuths", "ircserver"),
        ("LeaseSetClientAuths", "streamrserver"),
        ("UseOutproxyPlugin", "httpclient"),
        ("UseOutproxyPlugin", "socks"),
        ("UseOutproxyPlugin", "socksirc"),
        ("UseOutproxyPlugin", "connectclient"),
    ]
    .into_iter()
    .map(|(option, family)| (option.to_owned(), family.to_owned()))
    .collect()
}

#[test]
fn m153_current_matrix_is_336_29_475_with_exact_residuals() {
    let matrix = planning_toml("095-full-support-matrix.toml");
    assert_eq!(matrix["proposal_number"].as_integer(), Some(170));
    assert_eq!(matrix["proposal_revision"].as_str(), Some("2026-05-20"));
    assert_eq!(matrix["proposal_status"].as_str(), Some("Open"));
    assert_eq!(
        matrix["source_sha256"].as_str(),
        Some("f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc")
    );
    // M153 historically reconciled the stale pointer to the M145 no-std/format
    // follow-up. M165 now owns the current-head refresh after M163/M164; the
    // M164 SAM socket commit is the last production-bearing commit.
    assert_eq!(
        matrix["current_production_head"].as_str(),
        Some("0dbaa6b1082762f5f2c42b47dcb22c1002ea4bf9")
    );

    let tunnel_types = matrix["contract_names"]["canonical_tunnel_types"]
        .as_array()
        .expect("canonical tunnel types");
    assert_eq!(tunnel_types.len(), 12);
    let families: Vec<String> = tunnel_types
        .iter()
        .map(|family| family.as_str().expect("family").to_owned())
        .collect();

    let mut counts = BTreeMap::new();
    let mut blocked = BTreeSet::new();
    let mut applied = BTreeSet::new();
    for row in matrix["tunnel_manager"]["options"].as_array().expect("options") {
        let option = row["canonical_key"].as_str().expect("canonical option");
        let cells = row["cells"].as_array().expect("option cells");
        assert_eq!(cells.len(), families.len());
        for (index, cell) in cells.iter().enumerate() {
            let family = families[index].clone();
            match cell.as_str().expect("cell disposition") {
                "apply" => {
                    *counts.entry("apply").or_insert(0usize) += 1;
                    applied.insert((option.to_owned(), family));
                }
                "blocked_primitive" => {
                    *counts.entry("blocked_primitive").or_insert(0usize) += 1;
                    blocked.insert((option.to_owned(), family));
                }
                "not_applicable" => {
                    *counts.entry("not_applicable").or_insert(0usize) += 1;
                }
                other => panic!("unexpected cell disposition {other}"),
            }
        }
    }
    assert_eq!(counts.get("apply"), Some(&336));
    assert_eq!(counts.get("blocked_primitive"), Some(&29));
    assert_eq!(counts.get("not_applicable"), Some(&475));
    let total: usize = counts.values().sum();
    assert_eq!(total, 840);

    let declared = matrix["current_matrix_counts"].as_table().expect("declared counts");
    assert_eq!(declared["total"].as_integer(), Some(840));
    assert_eq!(declared["apply"].as_integer(), Some(336));
    assert_eq!(declared["blocked_primitive"].as_integer(), Some(29));
    assert_eq!(declared["not_applicable"].as_integer(), Some(475));

    // Exact residual identity: no cell disposition may change in M153.
    assert_eq!(blocked, expected_residuals());
    let by_option: BTreeMap<&str, usize> =
        blocked.iter().fold(BTreeMap::new(), |mut acc, (option, _)| {
            *acc.entry(option.as_str()).or_insert(0) += 1;
            acc
        });
    assert_eq!(by_option.get("SigType"), Some(&10));
    assert_eq!(by_option.get("EncryptLeaseSet"), Some(&5));
    assert_eq!(by_option.get("OptionalLookup"), Some(&5));
    assert_eq!(by_option.get("LeaseSetClientAuths"), Some(&5));
    assert_eq!(by_option.get("UseOutproxyPlugin"), Some(&4));

    // M140-M146 composition at the current head: every accepted promotion is
    // still apply, every M140 reclassification is still affirmative N/A, and
    // every M146 blocked cell is still blocked.
    for cell in [
        ("UniqueLocalAddressPerClient", "httpserver"),
        ("UniqueLocalAddressPerClient", "httpbidirserver"),
        ("SSLProxies", "httpclient"),
        ("JumpList", "httpclient"),
        ("Profile", "client"),
        ("UseSSL", "httpclient"),
        ("UseSSL", "connectclient"),
        ("UseSSL", "httpserver"),
        ("UseSSL", "httpbidirserver"),
        ("MultiHoming", "httpserver"),
        ("MultiHoming", "httpbidirserver"),
    ] {
        assert!(
            applied.contains(&(cell.0.to_owned(), cell.1.to_owned())),
            "M141-M145 promotion {}:{} must be apply at the M153 head",
            cell.0,
            cell.1
        );
    }
    let options = matrix["tunnel_manager"]["options"].as_array().unwrap();
    let disposition_of = |key: &str, family: &str| -> &str {
        let row = options
            .iter()
            .find(|row| string_field(row, "canonical_key") == key)
            .unwrap_or_else(|| panic!("{key} row must exist"));
        let index = families.iter().position(|name| name == family).expect("family index");
        row["cells"][index].as_str().expect("cell disposition")
    };
    for cell in [
        ("Profile", "httpclient"),
        ("Profile", "ircclient"),
        ("Profile", "socks"),
        ("Profile", "socksirc"),
        ("Profile", "connectclient"),
        ("Profile", "streamrclient"),
        ("ConnectDelay", "streamrclient"),
    ] {
        assert_eq!(
            disposition_of(cell.0, cell.1),
            "not_applicable",
            "M140 cell {}:{} must remain affirmative N/A",
            cell.0,
            cell.1
        );
    }
    for cell in [
        ("UseOutproxyPlugin", "httpclient"),
        ("UseOutproxyPlugin", "socks"),
        ("UseOutproxyPlugin", "socksirc"),
        ("UseOutproxyPlugin", "connectclient"),
    ] {
        assert!(
            blocked.contains(&(cell.0.to_owned(), cell.1.to_owned())),
            "M146 cell {}:{} must remain blocked",
            cell.0,
            cell.1
        );
    }
}

#[test]
fn m153_authority_docs_name_m153_and_retain_partial_support() {
    let root = workspace_root();
    // Active planning/docs authority: M153 is the current runtime/security
    // qualification; M139 is historical; support remains partial at
    // `336/29/475`; no full-support claim.
    // M155: the post-M146 roadmap was superseded for execution after M154
    // (historical through M154); the active corrective authority is now the
    // post-M154 roadmap, so the active list tracks the post-M154 file.
    for name in [
        "AGENTS.md",
        "plans/registry.md",
        "plans/implementation/i2pcontrol-proposal-170/README.md",
        "plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md",
        "docs/i2pcontrol/README.md",
        "docs/i2pcontrol/proposal-170-support.md",
    ] {
        let text = std::fs::read_to_string(root.join(name))
            .unwrap_or_else(|_| panic!("{name} must exist"));
        let lower = text.to_ascii_lowercase();
        assert!(
            lower.contains("partial"),
            "{name} must retain partial support"
        );
        assert!(text.contains("M153"), "{name} must name M153");
        assert!(
            !text.contains("Status: full Proposal 170 support")
                && !text.contains("Status: **full Proposal 170 support"),
            "{name} must not claim full Proposal 170 support"
        );
    }
    // Current-head counts in active authority docs.
    for name in [
        "AGENTS.md",
        "plans/registry.md",
        "plans/implementation/i2pcontrol-proposal-170/README.md",
        "docs/i2pcontrol/proposal-170-support.md",
    ] {
        let text = std::fs::read_to_string(root.join(name)).expect("authority doc");
        assert!(
            text.contains("336") && text.contains("29") && text.contains("475"),
            "{name} must state the current 336/29/475 authority"
        );
    }
    // M139 remains named as historical evidence, never as current-head
    // qualification.
    let agents = std::fs::read_to_string(root.join("AGENTS.md")).expect("AGENTS.md");
    assert!(
        agents.contains("M139"),
        "AGENTS.md must retain M139 as historical evidence"
    );
}

#[test]
fn m153_historical_closures_remain_immutable() {
    // M153 rewrites no historical closure. Each milestone-local aggregate
    // stays pinned to its own closure record independently of the
    // current-head authority owned above.
    let cases = [
        ("139-closure.md", "325/47/468"),
        ("140-closure.md", "325/40/475"),
        ("141-closure.md", "327/38/475"),
        ("142-closure.md", "329/36/475"),
        ("143-closure.md", "330/35/475"),
        ("144-closure.md", "334/31/475"),
        ("145-closure.md", "336/29/475"),
        ("146-closure.md", "336/29/475"),
    ];
    for (file, counts) in cases {
        let text = std::fs::read_to_string(
            workspace_root().join("plans/closure/i2pcontrol-proposal-170").join(file),
        )
        .unwrap_or_else(|_| panic!("{file} must exist"));
        assert!(
            text.contains(counts),
            "{file} must retain its historical {counts} authority"
        );
    }
    for file in [
        "130-closure.md",
        "129-closure.md",
        "128-closure.md",
        "127-closure.md",
        "126-closure.md",
    ] {
        let text = std::fs::read_to_string(
            workspace_root().join("plans/closure/i2pcontrol-proposal-170").join(file),
        )
        .unwrap_or_else(|_| panic!("{file} must exist"));
        assert!(
            text.contains("284") && text.contains("96") && text.contains("460"),
            "{file} must retain its historical 284/96/460 evidence"
        );
    }
}
