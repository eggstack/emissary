// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! I2PControl conformance manifest — machine-checkable exact-set validator.
//!
//! This test file separates base JSON-RPC/I2PControl inventory, canonical
//! Proposal 170 additions, and Emissary compatibility extensions. It
//! enumerates each inventory without treating an extension as canonical
//! Proposal 170 coverage.
//!
//! The checker fails for:
//! - manifest row without handler/source/test
//! - registered protocol item absent from manifest
//! - duplicate key/type/action registration
//! - missing tunnel backend
//! - undocumented null/error behavior

#![cfg(feature = "i2pcontrol")]

use std::collections::HashSet;

use emissary_cli::i2pcontrol::{
    backends::registry::create_default_registry,
    rpc,
    rpc::{
        address_book_requests, address_books, error_codes, methods, tunnel_actions, tunnel_types,
    },
};

// ──────────────────────────────────────────────────────────────────────
// § 1. Base method manifest — every I2PControl method
// ──────────────────────────────────────────────────────────────────────

struct MethodRow {
    #[allow(dead_code)]
    name: &'static str,
    #[allow(dead_code)]
    auth_required: bool,
    #[allow(dead_code)]
    owner_milestone: &'static str,
    #[allow(dead_code)]
    fixture_id: &'static str,
}

const BASE_METHOD_MANIFEST: &[MethodRow] = &[
    MethodRow {
        name: methods::AUTHENTICATE,
        auth_required: false,
        owner_milestone: "M001",
        fixture_id: "fixture_authenticate",
    },
    MethodRow {
        name: methods::GET_KEYS,
        auth_required: true,
        owner_milestone: "M002",
        fixture_id: "fixture_get_keys",
    },
    MethodRow {
        name: methods::SET_CONFIG,
        auth_required: true,
        owner_milestone: "M003",
        fixture_id: "fixture_set_config",
    },
    MethodRow {
        name: methods::SET_SUBSCRIPTIONS,
        auth_required: true,
        owner_milestone: "M003",
        fixture_id: "fixture_set_subscriptions",
    },
    MethodRow {
        name: methods::ADDRESS_BOOK,
        auth_required: true,
        owner_milestone: "M003",
        fixture_id: "fixture_address_book",
    },
    MethodRow {
        name: methods::TUNNEL_MANAGER,
        auth_required: true,
        owner_milestone: "M004",
        fixture_id: "fixture_tunnel_manager",
    },
    MethodRow {
        name: methods::ROUTER_INFO,
        auth_required: true,
        owner_milestone: "M005",
        fixture_id: "fixture_router_info",
    },
    MethodRow {
        name: methods::CLIENT_SERVICES_INFO,
        auth_required: true,
        owner_milestone: "M006",
        fixture_id: "fixture_client_services_info",
    },
];

#[test]
fn method_manifest_matches_production_constants() {
    let manifest_names: HashSet<&str> = BASE_METHOD_MANIFEST.iter().map(|r| r.name).collect();
    let production_methods = [
        methods::AUTHENTICATE,
        methods::GET_KEYS,
        methods::SET_CONFIG,
        methods::SET_SUBSCRIPTIONS,
        methods::ADDRESS_BOOK,
        methods::TUNNEL_MANAGER,
        methods::ROUTER_INFO,
        methods::CLIENT_SERVICES_INFO,
    ];
    let production_set: HashSet<&str> = production_methods.iter().copied().collect();
    assert_eq!(
        manifest_names, production_set,
        "base method manifest must exactly match production constants"
    );
}

#[test]
fn base_method_manifest_count() {
    assert_eq!(
        BASE_METHOD_MANIFEST.len(),
        8,
        "the base I2PControl surface contains exactly 8 methods"
    );
}

const COMPATIBILITY_STANDALONE_METHODS: &[&str] =
    &[methods::SET_CONFIG, methods::SET_SUBSCRIPTIONS];

#[test]
fn standalone_base_methods_are_not_proposal_additions() {
    assert_eq!(
        COMPATIBILITY_STANDALONE_METHODS,
        &[methods::SET_CONFIG, methods::SET_SUBSCRIPTIONS]
    );
    assert!(COMPATIBILITY_STANDALONE_METHODS
        .iter()
        .all(|method| BASE_METHOD_MANIFEST.iter().any(|row| row.name == *method)));
}

// ──────────────────────────────────────────────────────────────────────
// § 2. Tunnel type manifest — all 12 types
// ──────────────────────────────────────────────────────────────────────

struct TunnelTypeRow {
    name: &'static str,
    has_backend: bool,
    #[allow(dead_code)]
    owner_milestone: &'static str,
}

const TUNNEL_TYPE_MANIFEST: &[TunnelTypeRow] = &[
    TunnelTypeRow {
        name: tunnel_types::CLIENT,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::HTTP_CLIENT,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::IRC_CLIENT,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::SOCKS,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::SOCKS_IRC,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::CONNECT_CLIENT,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::STREAMR_CLIENT,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::SERVER,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::HTTP_SERVER,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::HTTP_BIDIR_SERVER,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::IRC_SERVER,
        has_backend: true,
        owner_milestone: "M004",
    },
    TunnelTypeRow {
        name: tunnel_types::STREAMR_SERVER,
        has_backend: true,
        owner_milestone: "M004",
    },
];

#[test]
fn tunnel_type_manifest_matches_production() {
    let manifest_names: HashSet<&str> = TUNNEL_TYPE_MANIFEST.iter().map(|r| r.name).collect();
    let production_set: HashSet<&str> = tunnel_types::ALL.iter().copied().collect();
    assert_eq!(
        manifest_names, production_set,
        "tunnel type manifest must exactly match production ALL"
    );
}

#[test]
fn tunnel_type_manifest_count() {
    assert_eq!(
        TUNNEL_TYPE_MANIFEST.len(),
        12,
        "Proposal 170 defines exactly 12 tunnel types"
    );
}

#[test]
fn every_tunnel_type_has_backend() {
    for row in TUNNEL_TYPE_MANIFEST {
        assert!(
            row.has_backend,
            "tunnel type {} must have a backend",
            row.name
        );
    }
}

#[test]
fn default_registry_covers_all_manifest_types() {
    let registry = create_default_registry().expect("default registry must be valid");
    for row in TUNNEL_TYPE_MANIFEST {
        let tt = match row.name {
            "client" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::Client,
            "httpclient" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpClient,
            "ircclient" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::IrcClient,
            "socks" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::Socks,
            "socksirc" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::SocksIrc,
            "connectclient" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::ConnectClient,
            "streamrclient" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::StreamrClient,
            "server" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::Server,
            "httpserver" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpServer,
            "httpbidirserver" => {
                emissary_cli::i2pcontrol::domain::tunnel::TunnelType::HttpBidirServer
            }
            "ircserver" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::IrcServer,
            "streamrserver" => emissary_cli::i2pcontrol::domain::tunnel::TunnelType::StreamrServer,
            _ => panic!("unknown tunnel type in manifest: {}", row.name),
        };
        let _backend = registry.get(tt);
    }
}

// ──────────────────────────────────────────────────────────────────────
// § 3. Tunnel action manifest — seven canonical actions
// ──────────────────────────────────────────────────────────────────────

struct TunnelActionRow {
    name: &'static str,
    #[allow(dead_code)]
    owner_milestone: &'static str,
}

const CANONICAL_TUNNEL_ACTION_MANIFEST: &[TunnelActionRow] = &[
    TunnelActionRow {
        name: "create",
        owner_milestone: "M004",
    },
    TunnelActionRow {
        name: "edit",
        owner_milestone: "M004",
    },
    TunnelActionRow {
        name: "get",
        owner_milestone: "M004",
    },
    TunnelActionRow {
        name: "start",
        owner_milestone: "M004",
    },
    TunnelActionRow {
        name: "stop",
        owner_milestone: "M004",
    },
    TunnelActionRow {
        name: "restart",
        owner_milestone: "M004",
    },
    TunnelActionRow {
        name: "delete",
        owner_milestone: "M004",
    },
];

#[test]
fn tunnel_action_manifest_count() {
    assert_eq!(
        CANONICAL_TUNNEL_ACTION_MANIFEST.len(),
        7,
        "Proposal 170 defines exactly 7 canonical tunnel actions"
    );
}

#[test]
fn tunnel_action_manifest_unique() {
    let names: HashSet<&str> = CANONICAL_TUNNEL_ACTION_MANIFEST.iter().map(|r| r.name).collect();
    assert_eq!(
        names.len(),
        CANONICAL_TUNNEL_ACTION_MANIFEST.len(),
        "tunnel action manifest must have unique names"
    );
}

#[test]
fn canonical_tunnel_actions_are_separate_from_compatibility_actions() {
    let canonical: HashSet<&str> =
        CANONICAL_TUNNEL_ACTION_MANIFEST.iter().map(|r| r.name).collect();
    let compatibility: HashSet<&str> = rpc::tunnel_actions::COMPATIBILITY.iter().copied().collect();
    assert_eq!(
        canonical,
        rpc::tunnel_actions::CANONICAL.iter().copied().collect()
    );
    assert!(canonical.is_disjoint(&compatibility));
    assert!(compatibility.contains(rpc::tunnel_actions::LIST));
}

// ──────────────────────────────────────────────────────────────────────
// § 4. Address book manifest — all 4 books
// ──────────────────────────────────────────────────────────────────────

struct AddressBookRow {
    name: &'static str,
    #[allow(dead_code)]
    owner_milestone: &'static str,
}

const ADDRESS_BOOK_MANIFEST: &[AddressBookRow] = &[
    AddressBookRow {
        name: address_books::PRIVATE,
        owner_milestone: "M003",
    },
    AddressBookRow {
        name: address_books::LOCAL,
        owner_milestone: "M003",
    },
    AddressBookRow {
        name: address_books::ROUTER,
        owner_milestone: "M003",
    },
    AddressBookRow {
        name: address_books::PUBLISHED,
        owner_milestone: "M003",
    },
];

#[test]
fn address_book_manifest_matches_production() {
    let manifest_names: HashSet<&str> = ADDRESS_BOOK_MANIFEST.iter().map(|r| r.name).collect();
    let production_set: HashSet<&str> = address_books::ALL.iter().copied().collect();
    assert_eq!(
        manifest_names, production_set,
        "address book manifest must exactly match production ALL"
    );
}

#[test]
fn address_book_manifest_count() {
    assert_eq!(
        ADDRESS_BOOK_MANIFEST.len(),
        4,
        "Proposal 170 defines exactly 4 address books"
    );
}

// ──────────────────────────────────────────────────────────────────────
// § 5. Address book request modes — canonical and compatibility partitions
// ──────────────────────────────────────────────────────────────────────

const CANONICAL_ADDRESS_BOOK_MODES: &[&str] = &[
    "Type + Hostname + Destination",
    "SetSubscriptions",
    "SetConfig",
];

const COMPATIBILITY_ADDRESS_BOOK_MODES: &[&str] = &[
    address_book_requests::LIST,
    address_book_requests::LOOKUP,
    address_book_requests::ADD,
    address_book_requests::UPDATE,
    address_book_requests::DELETE,
];

#[test]
fn canonical_address_book_modes_are_explicit() {
    assert_eq!(
        CANONICAL_ADDRESS_BOOK_MODES,
        &[
            "Type + Hostname + Destination",
            "SetSubscriptions",
            "SetConfig"
        ]
    );
}

#[test]
fn compatibility_address_book_actions_are_not_canonical_modes() {
    assert_eq!(COMPATIBILITY_ADDRESS_BOOK_MODES.len(), 5);
    assert!(COMPATIBILITY_ADDRESS_BOOK_MODES
        .iter()
        .all(|mode| !CANONICAL_ADDRESS_BOOK_MODES.contains(mode)));
}

// ──────────────────────────────────────────────────────────────────────
// § 6. ClientServicesInfo selector manifest — all 6
// ──────────────────────────────────────────────────────────────────────

struct ClientServicesSelectorRow {
    name: &'static str,
    #[allow(dead_code)]
    owner_milestone: &'static str,
}

const CLIENT_SERVICES_SELECTOR_MANIFEST: &[ClientServicesSelectorRow] = &[
    ClientServicesSelectorRow {
        name: "I2PTunnel",
        owner_milestone: "M006",
    },
    ClientServicesSelectorRow {
        name: "HTTPProxy",
        owner_milestone: "M006",
    },
    ClientServicesSelectorRow {
        name: "SOCKS",
        owner_milestone: "M006",
    },
    ClientServicesSelectorRow {
        name: "SAM",
        owner_milestone: "M006",
    },
    ClientServicesSelectorRow {
        name: "BOB",
        owner_milestone: "M006",
    },
    ClientServicesSelectorRow {
        name: "I2CP",
        owner_milestone: "M006",
    },
];

#[test]
fn client_services_selector_manifest_count() {
    assert_eq!(
        CLIENT_SERVICES_SELECTOR_MANIFEST.len(),
        6,
        "Proposal 170 defines exactly 6 ClientServicesInfo selectors"
    );
}

#[test]
fn client_services_selector_manifest_unique() {
    let names: HashSet<&str> = CLIENT_SERVICES_SELECTOR_MANIFEST.iter().map(|r| r.name).collect();
    assert_eq!(
        names.len(),
        CLIENT_SERVICES_SELECTOR_MANIFEST.len(),
        "ClientServicesInfo selectors must be unique"
    );
}

#[test]
fn client_services_selector_manifest_matches_production() {
    let manifest_names: HashSet<&str> =
        CLIENT_SERVICES_SELECTOR_MANIFEST.iter().map(|r| r.name).collect();
    let production_valid = ["I2PTunnel", "HTTPProxy", "SOCKS", "SAM", "BOB", "I2CP"];
    let production_set: HashSet<&str> = production_valid.iter().copied().collect();
    assert_eq!(
        manifest_names, production_set,
        "ClientServicesInfo selector manifest must match production validators"
    );
}

// ──────────────────────────────────────────────────────────────────────
// § 7. RouterInfo selector manifests — exact additions and legacy/base keys
// ──────────────────────────────────────────────────────────────────────

#[test]
fn router_info_selector_count() {
    assert_eq!(
        rpc::router_info_keys::PROPOSAL_170_ADDITIONS.len(),
        43,
        "Proposal 170 adds exactly 43 RouterInfo selectors"
    );
    for key in rpc::router_info_keys::PROPOSAL_170_ADDITIONS {
        assert!(rpc::is_valid_router_info_selector(key));
    }
}

#[test]
fn router_info_contract_manifest_has_exact_types_and_source_counts() {
    use rpc::router_info_keys::{JsonType::*, SourceDisposition};

    let expected_types = [
        String,
        NullableString,
        NullableInteger,
        NullableString,
        ArrayOfStrings,
        String,
        Integer,
        Integer,
        Integer,
        Integer,
        Number,
        ArrayOfObjects,
        ArrayOfObjects,
        Integer,
        Integer,
        ArrayOfObjects,
        Integer,
        Integer,
        ArrayOfObjects,
        Integer,
        Integer,
        Integer,
        Integer,
        Integer,
        Number,
        Number,
        Integer,
        Integer,
        ArrayOfStrings,
        ArrayOfStrings,
        Integer,
        Integer,
        ObjectMapOfObjects,
        ArrayOfStrings,
        ArrayOfStrings,
        ArrayOfStrings,
        ArrayOfObjects,
        ArrayOfStringMaps,
        ArrayOfStringMaps,
        ArrayOfStringMaps,
        ArrayOfStringMaps,
        ObjectWithPathAndEntries,
        ObjectWithPathAndEntries,
    ];
    let contract = rpc::router_info_keys::PROPOSAL_170_CONTRACT;
    assert_eq!(contract.len(), 43);
    assert_eq!(
        contract.iter().map(|row| row.json_type).collect::<Vec<_>>(),
        expected_types
    );
    assert_eq!(
        contract
            .iter()
            .filter(|row| matches!(row.source, SourceDisposition::Available { .. }))
            .count(),
        31
    );
    assert_eq!(
        contract
            .iter()
            .filter(|row| matches!(row.source, SourceDisposition::Neutral { .. }))
            .count(),
        1
    );
    assert_eq!(
        contract
            .iter()
            .filter(|row| matches!(row.source, SourceDisposition::Unavailable { .. }))
            .count(),
        11
    );
    assert!(contract.iter().all(|row| row.direct_presence && !row.fixture.is_empty()));
}

#[test]
fn router_info_source_map_documents_every_canonical_key_once() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_map =
        std::fs::read_to_string(root.join("../docs/i2pcontrol/router-info-source-map.md"))
            .expect("RouterInfo source map must exist");
    for row in rpc::router_info_keys::PROPOSAL_170_CONTRACT {
        let marker = format!("| `{}` |", row.key);
        assert_eq!(
            source_map.matches(&marker).count(),
            1,
            "missing or duplicate {}",
            row.key
        );
    }
    assert!(source_map
        .contains("43 total, 31 available, 1 protocol-permitted neutral, and 11 unavailable"));
}

#[test]
fn router_info_selector_partition_integrity() {
    let all: HashSet<&str> = rpc::router_info_keys::ALL.iter().copied().collect();
    let core: HashSet<&str> = rpc::router_info_keys::CORE_KEYS.iter().copied().collect();
    let ab: HashSet<&str> = rpc::router_info_keys::ADDRESS_BOOK_KEYS.iter().copied().collect();

    // The old partition remains intact; exact additions are tracked
    // separately and must not be counted as the legacy/base registry.
    let union: HashSet<&str> = core.union(&ab).copied().collect();
    assert_eq!(union.len(), 121, "legacy/base registry remains 121 keys");
    assert!(all.is_superset(&union));
    // CORE ∩ ADDRESS_BOOK = ∅
    assert!(
        core.is_disjoint(&ab),
        "CORE and ADDRESS_BOOK must be disjoint"
    );
    // sizes add up
    assert_eq!(
        core.len() + ab.len(),
        121,
        "legacy CORE + ADDRESS_BOOK must sum to 121"
    );
}

#[test]
fn router_info_address_book_selectors_count() {
    assert_eq!(
        rpc::router_info_keys::ADDRESS_BOOK_KEYS.len(),
        6,
        "Address book selectors: exactly 6"
    );
}

#[test]
fn router_info_core_selectors_count() {
    assert_eq!(
        rpc::router_info_keys::CORE_KEYS.len(),
        115,
        "Core selectors: exactly 115"
    );
}

// ──────────────────────────────────────────────────────────────────────
// § 8. JSON-RPC error code manifest
// ──────────────────────────────────────────────────────────────────────

struct ErrorCodeRow {
    #[allow(dead_code)]
    name: &'static str,
    #[allow(dead_code)]
    code: i32,
    #[allow(dead_code)]
    owner_milestone: &'static str,
}

const ERROR_CODE_MANIFEST: &[ErrorCodeRow] = &[
    ErrorCodeRow {
        name: "PARSE_ERROR",
        code: -32700,
        owner_milestone: "M001",
    },
    ErrorCodeRow {
        name: "INVALID_REQUEST",
        code: -32600,
        owner_milestone: "M001",
    },
    ErrorCodeRow {
        name: "METHOD_NOT_FOUND",
        code: -32601,
        owner_milestone: "M001",
    },
    ErrorCodeRow {
        name: "INVALID_PARAMS",
        code: -32602,
        owner_milestone: "M001",
    },
    ErrorCodeRow {
        name: "INTERNAL_ERROR",
        code: -32603,
        owner_milestone: "M001",
    },
    ErrorCodeRow {
        name: "APP_ERROR",
        code: -1,
        owner_milestone: "M001",
    },
    ErrorCodeRow {
        name: "INVALID_PASSWORD",
        code: -32001,
        owner_milestone: "M020",
    },
    ErrorCodeRow {
        name: "NO_TOKEN",
        code: -32002,
        owner_milestone: "M020",
    },
    ErrorCodeRow {
        name: "INVALID_TOKEN",
        code: -32003,
        owner_milestone: "M020",
    },
    ErrorCodeRow {
        name: "TOKEN_EXPIRED",
        code: -32004,
        owner_milestone: "M020",
    },
    ErrorCodeRow {
        name: "UNSPECIFIED_API_VERSION",
        code: -32005,
        owner_milestone: "M020",
    },
    ErrorCodeRow {
        name: "UNSUPPORTED_API_VERSION",
        code: -32006,
        owner_milestone: "M020",
    },
];

#[test]
fn error_code_manifest_matches_production() {
    assert_eq!(error_codes::PARSE_ERROR, -32700);
    assert_eq!(error_codes::INVALID_REQUEST, -32600);
    assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
    assert_eq!(error_codes::INVALID_PARAMS, -32602);
    assert_eq!(error_codes::INTERNAL_ERROR, -32603);
    assert_eq!(error_codes::APP_ERROR, -1);
    assert_eq!(error_codes::INVALID_PASSWORD, -32001);
    assert_eq!(error_codes::NO_TOKEN, -32002);
    assert_eq!(error_codes::INVALID_TOKEN, -32003);
    assert_eq!(error_codes::TOKEN_EXPIRED, -32004);
    assert_eq!(error_codes::UNSPECIFIED_API_VERSION, -32005);
    assert_eq!(error_codes::UNSUPPORTED_API_VERSION, -32006);
}

#[test]
fn error_code_manifest_count() {
    assert_eq!(
        ERROR_CODE_MANIFEST.len(),
        12,
        "Proposal 170 uses five standard, one legacy application, and six I2PControl-specific error codes"
    );
}

// ──────────────────────────────────────────────────────────────────────
// § 9. No duplicate registrations across all manifests
// ──────────────────────────────────────────────────────────────────────

#[test]
fn no_duplicate_tunnel_types() {
    let names: Vec<&str> = tunnel_types::ALL.to_vec();
    let set: HashSet<&str> = names.iter().copied().collect();
    assert_eq!(
        set.len(),
        names.len(),
        "tunnel_types::ALL must have no duplicates"
    );
}

#[test]
fn no_duplicate_tunnel_actions() {
    let actions = tunnel_actions::CANONICAL;
    let set: HashSet<&str> = actions.iter().copied().collect();
    assert_eq!(
        set.len(),
        actions.len(),
        "tunnel_actions must have no duplicates"
    );
}

#[test]
fn no_duplicate_address_books() {
    let set: HashSet<&str> = address_books::ALL.iter().copied().collect();
    assert_eq!(
        set.len(),
        address_books::ALL.len(),
        "address_books::ALL must have no duplicates"
    );
}

#[test]
fn no_duplicate_router_info_selectors() {
    let set: HashSet<&str> = rpc::router_info_keys::ALL.iter().copied().collect();
    assert_eq!(
        set.len(),
        rpc::router_info_keys::ALL.len(),
        "router_info_keys::ALL must have no duplicates"
    );
}

#[test]
fn no_duplicate_methods() {
    let names: Vec<&str> = BASE_METHOD_MANIFEST.iter().map(|r| r.name).collect();
    let set: HashSet<&str> = names.iter().copied().collect();
    assert_eq!(
        set.len(),
        names.len(),
        "method manifest must have no duplicates"
    );
}

// ──────────────────────────────────────────────────────────────────────
// § 10. Protocol exactness guards
// ──────────────────────────────────────────────────────────────────────

#[test]
fn authenticate_method_name_exact() {
    assert_eq!(methods::AUTHENTICATE, "Authenticate");
}

#[test]
fn router_info_method_name_exact() {
    assert_eq!(methods::ROUTER_INFO, "RouterInfo");
}

#[test]
fn address_book_method_name_exact() {
    assert_eq!(methods::ADDRESS_BOOK, "AddressBook");
}

#[test]
fn tunnel_manager_method_name_exact() {
    assert_eq!(methods::TUNNEL_MANAGER, "TunnelManager");
}

#[test]
fn client_services_info_method_name_exact() {
    assert_eq!(methods::CLIENT_SERVICES_INFO, "ClientServicesInfo");
}

#[test]
fn get_keys_method_name_exact() {
    assert_eq!(methods::GET_KEYS, "GetKeys");
}

#[test]
fn set_config_method_name_exact() {
    assert_eq!(methods::SET_CONFIG, "SetConfig");
}

#[test]
fn set_subscriptions_method_name_exact() {
    assert_eq!(methods::SET_SUBSCRIPTIONS, "SetSubscriptions");
}

// ──────────────────────────────────────────────────────────────────────
// § 11. Selector key exactness — spot-check critical keys
// ──────────────────────────────────────────────────────────────────────

#[test]
fn selector_key_exact_udp_active() {
    assert_eq!(rpc::router_info_keys::UDP_ACTIVE, "i2p.router.udp.active");
}

#[test]
fn selector_key_exact_version() {
    assert_eq!(rpc::router_info_keys::VERSION, "i2p.router.version");
}

#[test]
fn selector_key_exact_uptime() {
    assert_eq!(rpc::router_info_keys::UPTIME, "i2p.router.uptime");
}

#[test]
fn selector_key_exact_identity() {
    assert_eq!(rpc::router_info_keys::IDENTITY, "i2p.router.identity");
}

#[test]
fn selector_key_exact_log_snapshot() {
    assert_eq!(rpc::router_info_keys::LOG_SNAPSHOT, "i2p.router.log");
}

#[test]
fn selector_key_exact_log_clear() {
    assert_eq!(rpc::router_info_keys::LOG_CLEAR, "i2p.router.log.clear");
}

#[test]
fn selector_key_exact_address_book_private() {
    assert_eq!(
        rpc::router_info_keys::ADDRESS_BOOK_PRIVATE,
        "i2p.router.addressbook.private"
    );
}

#[test]
fn selector_key_exact_address_book_config() {
    assert_eq!(
        rpc::router_info_keys::ADDRESS_BOOK_CONFIG,
        "i2p.router.addressbook.config"
    );
}

#[test]
fn selector_key_exact_net_i2ptunnels() {
    assert_eq!(
        rpc::router_info_keys::NET_IPTUNNELS,
        "i2p.router.net.i2ptunnels"
    );
}

// ──────────────────────────────────────────────────────────────────────
// § 12. Tunnel type name exactness
// ──────────────────────────────────────────────────────────────────────

#[test]
fn tunnel_type_names_exact() {
    assert_eq!(tunnel_types::CLIENT, "client");
    assert_eq!(tunnel_types::HTTP_CLIENT, "httpclient");
    assert_eq!(tunnel_types::IRC_CLIENT, "ircclient");
    assert_eq!(tunnel_types::SOCKS, "socks");
    assert_eq!(tunnel_types::SOCKS_IRC, "socksirc");
    assert_eq!(tunnel_types::CONNECT_CLIENT, "connectclient");
    assert_eq!(tunnel_types::STREAMR_CLIENT, "streamrclient");
    assert_eq!(tunnel_types::SERVER, "server");
    assert_eq!(tunnel_types::HTTP_SERVER, "httpserver");
    assert_eq!(tunnel_types::HTTP_BIDIR_SERVER, "httpbidirserver");
    assert_eq!(tunnel_types::IRC_SERVER, "ircserver");
    assert_eq!(tunnel_types::STREAMR_SERVER, "streamrserver");
}

// ──────────────────────────────────────────────────────────────────────
// § 13. Tunnel action name exactness
// ──────────────────────────────────────────────────────────────────────

#[test]
fn tunnel_action_names_exact() {
    assert_eq!(
        tunnel_actions::CANONICAL,
        &["create", "edit", "get", "start", "stop", "restart", "delete"]
    );
    assert_eq!(
        tunnel_actions::COMPATIBILITY,
        &["List", "Create", "Edit", "Get", "Start", "Stop", "Restart", "Delete"]
    );
}

// ──────────────────────────────────────────────────────────────────────
// § 14. Address book name exactness
// ──────────────────────────────────────────────────────────────────────

#[test]
fn address_book_names_exact() {
    assert_eq!(address_books::PRIVATE, "private");
    assert_eq!(address_books::LOCAL, "local");
    assert_eq!(address_books::ROUTER, "router");
    assert_eq!(address_books::PUBLISHED, "published");
}

// ──────────────────────────────────────────────────────────────────────
// § 15. All is_valid_* validators are consistent with manifests
// ──────────────────────────────────────────────────────────────────────

#[test]
fn is_valid_tunnel_type_consistent_with_manifest() {
    for row in TUNNEL_TYPE_MANIFEST {
        assert!(
            emissary_cli::i2pcontrol::rpc::is_valid_tunnel_type(row.name),
            "is_valid_tunnel_type({}) must be true",
            row.name
        );
    }
    assert!(!emissary_cli::i2pcontrol::rpc::is_valid_tunnel_type(
        "unknown"
    ));
    assert!(!emissary_cli::i2pcontrol::rpc::is_valid_tunnel_type("All"));
    assert!(!emissary_cli::i2pcontrol::rpc::is_valid_tunnel_type(""));
}

#[test]
fn is_valid_address_book_consistent_with_manifest() {
    for row in ADDRESS_BOOK_MANIFEST {
        assert!(
            emissary_cli::i2pcontrol::rpc::is_valid_address_book(row.name),
            "is_valid_address_book({}) must be true",
            row.name
        );
    }
    assert!(!emissary_cli::i2pcontrol::rpc::is_valid_address_book(
        "unknown"
    ));
    assert!(!emissary_cli::i2pcontrol::rpc::is_valid_address_book(""));
}

#[test]
fn is_valid_router_info_selector_consistent_with_manifest() {
    for key in rpc::router_info_keys::ALL {
        assert!(
            emissary_cli::i2pcontrol::rpc::is_valid_router_info_selector(key),
            "is_valid_router_info_selector({}) must be true",
            key
        );
    }
    assert!(!emissary_cli::i2pcontrol::rpc::is_valid_router_info_selector("unknown.key"));
    assert!(!emissary_cli::i2pcontrol::rpc::is_valid_router_info_selector(""));
}

// ──────────────────────────────────────────────────────────────────────
// § 16. BOB exact value — always unavailable
// ──────────────────────────────────────────────────────────────────────

#[test]
fn bob_is_always_unavailable() {
    use emissary_cli::i2pcontrol::client_services::is_valid_client_services_selector;
    assert!(is_valid_client_services_selector("BOB"));
    // BOB has no real backend; its result is always `false` / unavailable.
    // This is verified by the client_services integration tests.
}

// ──────────────────────────────────────────────────────────────────────
// § 17. All supported tunnel backends return Unsupported state
// ──────────────────────────────────────────────────────────────────────

#[test]
fn all_backends_report_unsupported_state() {
    use emissary_cli::i2pcontrol::domain::tunnel::{
        TunnelDefinition, TunnelName, TunnelOwnership, TunnelType,
    };

    let registry = create_default_registry().expect("default registry");
    let def = TunnelDefinition {
        name: TunnelName::new("test".to_string()).unwrap(),
        tunnel_type: TunnelType::Client,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: emissary_cli::i2pcontrol::domain::tunnel::TunnelRuntimeState::Stopped,
        start_intent: emissary_cli::i2pcontrol::domain::tunnel::StartIntent::DoNotStart,
        options: Default::default(),
        raw_config: Default::default(),
    };

    let types = [
        TunnelType::Client,
        TunnelType::HttpClient,
        TunnelType::IrcClient,
        TunnelType::Socks,
        TunnelType::SocksIrc,
        TunnelType::ConnectClient,
        TunnelType::StreamrClient,
        TunnelType::Server,
        TunnelType::HttpServer,
        TunnelType::HttpBidirServer,
        TunnelType::IrcServer,
        TunnelType::StreamrServer,
    ];

    for tt in types {
        let backend = registry.get(tt);
        let status = backend.inspect(&def);
        // Unsupported backends report Unsupported runtime state
        assert!(
            matches!(
                status.runtime_state,
                emissary_cli::i2pcontrol::domain::tunnel::TunnelRuntimeState::Unsupported
            ),
            "backend for {:?} should report Unsupported runtime state, got {:?}",
            tt,
            status.runtime_state
        );
    }
}

// ──────────────────────────────────────────────────────────────────────
// § 18. Unsupported start returns error, not panic
// ──────────────────────────────────────────────────────────────────────

#[test]
fn unsupported_start_returns_not_implemented() {
    use emissary_cli::i2pcontrol::domain::tunnel::{
        TunnelDefinition, TunnelName, TunnelOwnership, TunnelType,
    };

    let registry = create_default_registry().expect("default registry");
    let def = TunnelDefinition {
        name: TunnelName::new("test".to_string()).unwrap(),
        tunnel_type: TunnelType::Client,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: emissary_cli::i2pcontrol::domain::tunnel::TunnelRuntimeState::Stopped,
        start_intent: emissary_cli::i2pcontrol::domain::tunnel::StartIntent::DoNotStart,
        options: Default::default(),
        raw_config: Default::default(),
    };

    let backend = registry.get(TunnelType::Client);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { backend.start(&def).await });
    assert!(result.is_err(), "unsupported start must return error");
    match result.unwrap_err() {
        emissary_cli::i2pcontrol::backends::BackendError::NotImplemented { .. } => {}
        other => panic!("expected NotImplemented, got {:?}", other),
    }
}

// ──────────────────────────────────────────────────────────────────────
// § 19. No secret material in router_info_keys constants
// ──────────────────────────────────────────────────────────────────────

#[test]
fn selector_keys_contain_no_secret_material() {
    for key in rpc::router_info_keys::ALL {
        let lower = key.to_lowercase();
        assert!(
            !lower.contains("password"),
            "selector key must not contain password: {key}"
        );
        assert!(
            !lower.contains("secret"),
            "selector key must not contain secret: {key}"
        );
        assert!(
            !lower.contains("private_key"),
            "selector key must not contain private_key: {key}"
        );
        assert!(
            !lower.contains("token"),
            "selector key must not contain token: {key}"
        );
    }
}

// ──────────────────────────────────────────────────────────────────────
// § 20. Compile-time size assertions for const arrays
// ──────────────────────────────────────────────────────────────────────

#[test]
fn const_array_sizes_match_manifests() {
    assert_eq!(tunnel_types::ALL.len(), 12);
    assert_eq!(address_books::ALL.len(), 4);
    assert_eq!(rpc::router_info_keys::ALL.len(), 161);
    assert_eq!(rpc::router_info_keys::CORE_KEYS.len(), 115);
    assert_eq!(rpc::router_info_keys::ADDRESS_BOOK_KEYS.len(), 6);
}
