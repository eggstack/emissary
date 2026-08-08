// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! M027 literal Proposal 170 fixtures.
//!
//! These fixtures are copied from the pinned Proposal 170 examples or are
//! minimal exact variants. They intentionally do not obtain expected values
//! from Emissary serializers. The older `golden_fixtures` test remains a
//! separately named compatibility/parser corpus and is not counted here.
//!
//! Authority: https://i2p.net/en/proposals/170-i2pcontrol-expansion/
//! Pinned revision: Open, created and last updated 2026-05-20.

#![cfg(feature = "i2pcontrol")]

use emissary_cli::i2pcontrol::rpc::{self, JsonRpcRequest, RequestId};
use serde_json::{json, Value};

fn parse_literal(body: Value) -> JsonRpcRequest {
    rpc::parse_request(&body.to_string()).expect("literal fixture must be valid JSON-RPC")
}

fn assert_exact_envelope(value: &Value, id: Value, member: &str) {
    let object = value.as_object().expect("fixture envelope must be an object");
    assert_eq!(object.len(), 3);
    assert_eq!(object.get("jsonrpc"), Some(&json!("2.0")));
    assert_eq!(object.get("id"), Some(&id));
    assert!(object.contains_key(member));
    assert!(!object.contains_key(if member == "result" {
        "error"
    } else {
        "result"
    }));
}

#[test]
fn base_authenticate_and_protected_router_info_fixtures_are_literal() {
    let authenticate = parse_literal(json!({
        "jsonrpc": "2.0",
        "method": "Authenticate",
        "params": {"API": 1, "Password": "fixture-password"},
        "id": "auth-1"
    }));
    assert_eq!(authenticate.method, "Authenticate");
    assert_eq!(authenticate.id, Some(RequestId::String("auth-1".into())));
    assert_eq!(authenticate.params.unwrap()["API"], json!(1));

    let protected = parse_literal(json!({
        "jsonrpc": "2.0",
        "method": "RouterInfo",
        "params": {
            "Token": "fixture-token",
            "i2p.router.version": ""
        },
        "id": 2
    }));
    assert_eq!(protected.method, "RouterInfo");
    assert_eq!(protected.params.unwrap()["Token"], json!("fixture-token"));

    let success = json!({
        "jsonrpc": "2.0",
        "id": "auth-1",
        "result": {"Token": "fixture-token", "API": 1}
    });
    assert_exact_envelope(&success, json!("auth-1"), "result");
    assert!(success["result"]["Token"].is_string());
    assert!(success["result"]["API"].is_number());
}

#[test]
fn authentication_errors_and_jsonrpc_id_matrix_are_literal() {
    let errors = [
        (json!({"API": 1}), -32001),
        (json!({"API": 1, "Password": "wrong"}), -32001),
        (json!({"Password": "fixture-password"}), -32005),
        (json!({"API": 99, "Password": "fixture-password"}), -32006),
    ];
    for (params, code) in errors {
        let request = parse_literal(json!({
            "jsonrpc": "2.0",
            "method": "Authenticate",
            "params": params,
            "id": 3
        }));
        assert_eq!(request.method, "Authenticate");
        assert!((-32006..=-32001).contains(&code));
        let response = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "error": {"code": code, "message": "contract-defined error"}
        });
        assert_exact_envelope(&response, json!(3), "error");
    }

    let string_id = parse_literal(json!({
        "jsonrpc": "2.0", "method": "Authenticate", "params": {}, "id": "abc"
    }));
    assert_eq!(string_id.id, Some(RequestId::String("abc".into())));

    let explicit_null = parse_literal(json!({
        "jsonrpc": "2.0", "method": "Authenticate", "params": {}, "id": null
    }));
    assert_eq!(explicit_null.id, Some(RequestId::Null));
    assert!(!explicit_null.is_notification());

    let notification = parse_literal(json!({
        "jsonrpc": "2.0", "method": "Authenticate", "params": {}
    }));
    assert!(notification.is_notification());

    for invalid_id in [json!(true), json!(1.5), json!({}), json!([])] {
        let body = json!({
            "jsonrpc": "2.0", "method": "Authenticate", "params": {}, "id": invalid_id
        });
        let error = rpc::parse_request(&body.to_string()).expect_err("invalid ID must fail");
        assert_eq!(error.error.code, rpc::error_codes::INVALID_REQUEST);
        assert_eq!(error.id, RequestId::Null);
    }
}

#[test]
fn canonical_address_book_requests_and_result_envelopes_are_literal() {
    for book in ["private", "local", "router", "published"] {
        let add = parse_literal(json!({
            "jsonrpc": "2.0",
            "method": "AddressBook",
            "params": {
                "Type": book,
                "Hostname": "example.i2p",
                "Destination": "fixture-destination"
            },
            "id": 10
        }));
        assert_eq!(add.params.unwrap()["Type"], json!(book));

        let delete = parse_literal(json!({
            "jsonrpc": "2.0",
            "method": "AddressBook",
            "params": {
                "Type": book,
                "Hostname": "example.i2p",
                "Destination": "fixture-destination",
                "Delete": ""
            },
            "id": 11
        }));
        assert!(delete.params.unwrap().contains_key("Delete"));
    }

    for request in [
        json!({
            "jsonrpc": "2.0", "method": "AddressBook",
            "params": {"SetSubscriptions": ["https://example.i2p/hosts.txt"]}, "id": 12
        }),
        json!({
            "jsonrpc": "2.0", "method": "AddressBook",
            "params": {"SetConfig": {"update_delay": "12"}}, "id": 13
        }),
    ] {
        let parsed = parse_literal(request);
        assert_eq!(parsed.method, "AddressBook");
    }

    let add_response = json!({
        "jsonrpc": "2.0", "id": 10,
        "result": {"success": true, "message": "Added example.i2p in private addressbook"}
    });
    assert_exact_envelope(&add_response, json!(10), "result");
    assert_eq!(add_response["result"]["success"], json!(true));

    let metadata_response = json!({
        "jsonrpc": "2.0", "id": 12,
        "result": {"success": true, "message": "Successfully modified subscriptions"}
    });
    assert_exact_envelope(&metadata_response, json!(12), "result");
}

#[test]
fn canonical_tunnel_actions_types_and_get_fixture_are_literal() {
    let actions = [
        "create", "edit", "get", "start", "stop", "restart", "delete",
    ];
    let types = [
        "client",
        "httpclient",
        "ircclient",
        "socks",
        "socksirc",
        "connectclient",
        "streamrclient",
        "server",
        "httpserver",
        "httpbidirserver",
        "ircserver",
        "streamrserver",
    ];

    for action in actions {
        let request = parse_literal(json!({
            "jsonrpc": "2.0", "method": "TunnelManager",
            "params": {"Name": "example-client", "Action": action}, "id": 20
        }));
        assert_eq!(request.params.unwrap()["Action"], json!(action));
    }
    for tunnel_type in types {
        for action in ["create", "edit", "get", "delete"] {
            let request = parse_literal(json!({
                "jsonrpc": "2.0", "method": "TunnelManager",
                "params": {"Name": "example-tunnel", "Action": action, "Type": tunnel_type},
                "id": 21
            }));
            let params = request.params.unwrap();
            assert_eq!(params["Action"], json!(action));
            assert_eq!(params["Type"], json!(tunnel_type));
        }
        for action in ["start", "stop", "restart"] {
            let request = parse_literal(json!({
                "jsonrpc": "2.0", "method": "TunnelManager",
                "params": {"Name": "example-tunnel", "Action": action}, "id": 22
            }));
            assert_eq!(request.params.unwrap()["Action"], json!(action));
        }
        assert!(rpc::is_valid_tunnel_type(tunnel_type));
    }

    let get_response = json!({
        "jsonrpc": "2.0", "id": 23,
        "result": {
            "status": "success - options for example-client",
            "info": {
                "client": true,
                "status": "running",
                "persistentClientKey": false,
                "offlineKeys": false,
                "targetDestination": "fixture-destination",
                "localDestination": "fixture-local-destination",
                "destination": "fixture-local-destination",
                "destinationB32": "fixture.b32.i2p",
                "rawConfig": {"name": "example-client", "type": "client"}
            }
        }
    });
    assert_exact_envelope(&get_response, json!(23), "result");
    assert_eq!(
        get_response["result"]["info"]["rawConfig"]["type"],
        json!("client")
    );
}

#[test]
fn client_services_info_and_router_info_fixtures_are_literal() {
    let services = parse_literal(json!({
        "jsonrpc": "2.0", "method": "ClientServicesInfo",
        "params": {
            "I2PTunnel": "", "HTTPProxy": "", "SOCKS": "", "SAM": "", "BOB": "", "I2CP": ""
        }, "id": 30
    }));
    assert_eq!(services.params.unwrap().len(), 6);

    let services_response = json!({
        "jsonrpc": "2.0", "id": 30,
        "result": {
            "I2PTunnel": {
                "client": {"example-client": {"address": "fixture-client.b32.i2p"}},
                "server": {"example-server": {"address": "fixture-server.b32.i2p", "port": 8080}}
            },
            "BOB": false,
            "SAM": {"enabled": true, "sessions": {}}
        }
    });
    assert_exact_envelope(&services_response, json!(30), "result");
    assert_eq!(services_response["result"]["BOB"], json!(false));

    let router_info = parse_literal(json!({
        "jsonrpc": "2.0", "method": "RouterInfo",
        "params": {
            "Token": "fixture-token",
            "i2p.router.id": "",
            "i2p.router.logs": "",
            "i2p.router.logs.clear": "",
            "i2p.router.net.tunnels.i2ptunnel": "",
            "i2p.router.netdb.peers": ""
        }, "id": 31
    }));
    assert_eq!(router_info.params.unwrap().len(), 6);

    let mixed_unavailable = json!({
        "jsonrpc": "2.0", "id": 31,
        "error": {"code": -32603, "message": "source unavailable"}
    });
    assert_exact_envelope(&mixed_unavailable, json!(31), "error");
}

const LITERAL_ROUTER_INFO_MANIFEST: &[(&str, &str)] = &[
    ("i2p.router.news", "string"),
    ("i2p.router.id", "string|null"),
    ("i2p.router.clockskew", "integer|null"),
    ("i2p.router.info", "string|null"),
    ("i2p.router.logs", "array<string>"),
    ("i2p.router.logs.clear", "string"),
    ("i2p.router.net.total.received.bytes", "integer"),
    ("i2p.router.net.total.sent.bytes", "integer"),
    ("i2p.router.net.total.transit.bytes", "integer"),
    ("i2p.router.net.bw.transit.15s", "integer"),
    ("i2p.router.net.tunnels.shareratio", "number"),
    ("i2p.router.net.tunnels.participating.info", "array<object>"),
    ("i2p.router.net.tunnels.i2ptunnel", "array<object>"),
    ("i2p.router.net.tunnels.exploratory.inbound", "integer"),
    ("i2p.router.net.tunnels.exploratory.outbound", "integer"),
    (
        "i2p.router.net.tunnels.exploratory.info.list",
        "array<object>",
    ),
    ("i2p.router.net.tunnels.client.inbound", "integer"),
    ("i2p.router.net.tunnels.client.outbound", "integer"),
    ("i2p.router.net.tunnels.client.info.list", "array<object>"),
    ("i2p.router.net.status.v6", "integer"),
    ("i2p.router.net.error", "integer"),
    ("i2p.router.net.error.v6", "integer"),
    ("i2p.router.net.testing", "integer"),
    ("i2p.router.net.testing.v6", "integer"),
    ("i2p.router.net.tunnels.successrate", "number"),
    ("i2p.router.net.tunnels.totalsuccessrate", "number"),
    ("i2p.router.net.tunnels.queue", "integer"),
    ("i2p.router.net.tunnels.tbmqueue", "integer"),
    ("i2p.router.netdb.peers", "array<string>"),
    ("i2p.router.netdb.activepeers.info", "array<string>"),
    ("i2p.router.netdb.ntcp.limit", "integer"),
    ("i2p.router.netdb.ssu.limit", "integer"),
    (
        "i2p.router.netdb.bannedpeers",
        "map<string,map<string,object>>",
    ),
    ("i2p.router.netdb.activepeers.list", "array<string>"),
    ("i2p.router.netdb.peers.list", "array<string>"),
    ("i2p.router.netdb.peers.info", "array<string>"),
    ("i2p.router.netdb.activepeers.stats", "array<object>"),
    (
        "i2p.router.addressbook.private.list",
        "array<map<string,string>>",
    ),
    (
        "i2p.router.addressbook.local.list",
        "array<map<string,string>>",
    ),
    (
        "i2p.router.addressbook.router.list",
        "array<map<string,string>>",
    ),
    (
        "i2p.router.addressbook.published.list",
        "array<map<string,string>>",
    ),
    (
        "i2p.router.addressbook.subscriptions",
        "object{path,entries}",
    ),
    ("i2p.router.addressbook.config", "object{path,entries}"),
];

#[test]
fn exact_literal_43_selector_type_manifest_matches_the_pinned_contract() {
    let contract = rpc::router_info_keys::PROPOSAL_170_CONTRACT;
    assert_eq!(LITERAL_ROUTER_INFO_MANIFEST.len(), 43);
    assert_eq!(contract.len(), LITERAL_ROUTER_INFO_MANIFEST.len());
    for (field, (key, json_type)) in contract.iter().zip(LITERAL_ROUTER_INFO_MANIFEST) {
        assert_eq!(field.key, *key);
        assert_eq!(field.json_type.as_str(), *json_type);
        assert!(field.direct_presence);
    }
}

#[test]
fn router_info_literal_availability_and_bounds_are_partitioned() {
    let contract = rpc::router_info_keys::PROPOSAL_170_CONTRACT;
    assert_eq!(
        contract.iter().filter(|f| f.source.class() == "available").count(),
        16
    );
    assert_eq!(
        contract
            .iter()
            .filter(|f| f.source.class() == "protocol-permitted neutral")
            .count(),
        1
    );
    assert_eq!(
        contract.iter().filter(|f| f.source.class() == "unavailable").count(),
        26
    );
    assert!(contract.iter().any(|f| f.fixture == "p170.logs.string_list"));
    assert!(contract.iter().any(|f| f.fixture == "p170.clockskew.nullable_integer"));
    assert!(contract.iter().any(|f| f.fixture == "p170.router_news.string"));
}
