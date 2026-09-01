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
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 request ID.
///
/// Preserves string or numeric request IDs as required by JSON-RPC 2.0.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
    Null,
}

/// JSON-RPC 2.0 request.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    /// Must be exactly `"2.0"`.
    pub jsonrpc: String,

    /// Method name.
    pub method: String,

    /// Named parameters (object). Positional params are rejected.
    #[serde(default)]
    pub params: Option<serde_json::Map<String, serde_json::Value>>,

    /// Request ID. Null for notifications (no response sent).
    #[serde(default)]
    pub id: Option<RequestId>,
}

impl JsonRpcRequest {
    /// Return whether this request is a notification.
    ///
    /// Notification status is represented by an absent ID. An explicit JSON
    /// `null` remains a valid, serializable request ID and is not conflated
    /// with notification dispatch.
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// JSON-RPC 2.0 success response envelope.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcSuccess {
    pub jsonrpc: &'static str,
    pub id: RequestId,
    pub result: serde_json::Value,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcErrorObject {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 error response envelope.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: &'static str,
    pub id: RequestId,
    pub error: JsonRpcErrorObject,
}

/// JSON-RPC 2.0 error codes.
#[allow(dead_code)]
pub mod error_codes {
    /// Parse error: invalid JSON.
    pub const PARSE_ERROR: i32 = -32700;

    /// Invalid request: valid JSON but not a valid JSON-RPC request.
    pub const INVALID_REQUEST: i32 = -32600;

    /// Method not found.
    pub const METHOD_NOT_FOUND: i32 = -32601;

    /// Invalid params.
    pub const INVALID_PARAMS: i32 = -32602;

    /// Internal error.
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Invalid password provided to Authenticate.
    pub const INVALID_PASSWORD: i32 = -32001;

    /// No authentication token was presented for a protected method.
    pub const NO_TOKEN: i32 = -32002;

    /// The presented authentication token does not exist.
    pub const INVALID_TOKEN: i32 = -32003;

    /// The presented authentication token expired and was removed.
    pub const TOKEN_EXPIRED: i32 = -32004;

    /// Authenticate did not include an API version.
    pub const UNSPECIFIED_API_VERSION: i32 = -32005;

    /// Authenticate included an unsupported API version.
    pub const UNSUPPORTED_API_VERSION: i32 = -32006;

    /// Legacy application-defined error retained for method-specific failures.
    pub const APP_ERROR: i32 = -1;

    pub const INVALID_PASSWORD_MESSAGE: &str = "Invalid password provided";
    pub const NO_TOKEN_MESSAGE: &str = "No authentication token presented";
    pub const INVALID_TOKEN_MESSAGE: &str = "Authentication token doesn't exist";
    pub const TOKEN_EXPIRED_MESSAGE: &str =
        "The provided authentication token was expired and will be removed";
    pub const UNSPECIFIED_API_VERSION_MESSAGE: &str =
        "The version of the I2PControl API used wasn't specified, but is required to be specified";
    pub const UNSUPPORTED_API_VERSION_MESSAGE: &str =
        "The version of the I2PControl API specified is not supported by I2PControl";
}

impl JsonRpcSuccess {
    /// Create a success response.
    pub fn new(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result,
        }
    }
}

impl JsonRpcErrorResponse {
    /// Create an error response.
    pub fn new(id: RequestId, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            error: JsonRpcErrorObject {
                code,
                message: message.into(),
                data: None,
            },
        }
    }

    /// Create an error response with data.
    #[allow(dead_code)]
    pub fn with_data(
        id: RequestId,
        code: i32,
        message: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            error: JsonRpcErrorObject {
                code,
                message: message.into(),
                data: Some(data),
            },
        }
    }
}

/// Parse a JSON-RPC request from a raw body string.
///
/// Returns the parsed request or an error response.
pub fn parse_request(body: &str) -> Result<JsonRpcRequest, JsonRpcErrorResponse> {
    let raw: serde_json::Value = serde_json::from_str(body).map_err(|e| {
        JsonRpcErrorResponse::new(
            RequestId::Null,
            error_codes::PARSE_ERROR,
            format!("Parse error: {e}"),
        )
    })?;

    // Must be a JSON object at top level
    let obj = raw.as_object().ok_or_else(|| {
        JsonRpcErrorResponse::new(
            RequestId::Null,
            error_codes::INVALID_REQUEST,
            "Request must be a JSON object",
        )
    })?;

    // Extract jsonrpc version
    let jsonrpc = obj.get("jsonrpc").and_then(|v| v.as_str()).ok_or_else(|| {
        JsonRpcErrorResponse::new(
            RequestId::Null,
            error_codes::INVALID_REQUEST,
            "Missing 'jsonrpc' field",
        )
    })?;

    if jsonrpc != "2.0" {
        return Err(JsonRpcErrorResponse::new(
            RequestId::Null,
            error_codes::INVALID_REQUEST,
            "jsonrpc must be exactly \"2.0\"",
        ));
    }

    // Extract method
    let method = obj.get("method").and_then(|v| v.as_str()).ok_or_else(|| {
        JsonRpcErrorResponse::new(
            RequestId::Null,
            error_codes::INVALID_REQUEST,
            "Missing 'method' field",
        )
    })?;

    if method.is_empty() {
        return Err(JsonRpcErrorResponse::new(
            RequestId::Null,
            error_codes::INVALID_REQUEST,
            "Method must not be empty",
        ));
    }

    // Extract and validate id before any other request-specific validation. An
    // invalid ID is an invalid request; its arbitrary JSON value must never be
    // coerced into null or zero and reflected in an error response.
    let id = match obj.get("id") {
        Some(value) => match RequestId::from_json(value) {
            Ok(id) => Some(id),
            Err(()) => {
                return Err(JsonRpcErrorResponse::new(
                    RequestId::Null,
                    error_codes::INVALID_REQUEST,
                    "Request id must be a string, integer, or null",
                ));
            }
        },
        None => None,
    };

    // Extract params — must be an object if present (named params)
    let params = match obj.get("params") {
        Some(serde_json::Value::Object(map)) => Some(map.clone()),
        Some(_) => {
            return Err(JsonRpcErrorResponse::new(
                id.unwrap_or(RequestId::Null),
                error_codes::INVALID_PARAMS,
                "Params must be a JSON object (named parameters)",
            ));
        }
        None => None,
    };

    Ok(JsonRpcRequest {
        jsonrpc: jsonrpc.to_string(),
        method: method.to_string(),
        params,
        id,
    })
}

impl RequestId {
    fn from_json(val: &serde_json::Value) -> Result<Self, ()> {
        match val {
            serde_json::Value::String(s) => Ok(RequestId::String(s.clone())),
            serde_json::Value::Number(n) => n.as_i64().map(RequestId::Number).ok_or(()),
            serde_json::Value::Null => Ok(RequestId::Null),
            _ => Err(()),
        }
    }
}

/// I2PControl method names.
#[allow(dead_code)]
pub mod methods {
    /// Authenticate method.
    pub const AUTHENTICATE: &str = "Authenticate";

    /// RouterInfo method.
    pub const ROUTER_INFO: &str = "RouterInfo";

    /// AddressBook method.
    pub const ADDRESS_BOOK: &str = "AddressBook";

    /// TunnelManager method.
    pub const TUNNEL_MANAGER: &str = "TunnelManager";

    /// ClientServicesInfo method.
    pub const CLIENT_SERVICES_INFO: &str = "ClientServicesInfo";

    /// Existing base I2PControl methods retained as explicit unsupported
    /// entries in the dispatcher inventory.
    pub const GET_KEYS: &str = "GetKeys";
    pub const GET_RATE: &str = "GetRate";
    pub const ROUTER_MANAGER: &str = "RouterManager";
    pub const NETWORK_SETTING: &str = "NetworkSetting";
    pub const ADVANCED_SETTINGS: &str = "AdvancedSettings";

    /// SetConfig method.
    pub const SET_CONFIG: &str = "SetConfig";

    /// SetSubscriptions method.
    pub const SET_SUBSCRIPTIONS: &str = "SetSubscriptions";

    /// Methods that the protected dispatcher actually routes to a handler.
    pub const PROTECTED_DISPATCH: &[&str] = &[
        ADDRESS_BOOK,
        SET_SUBSCRIPTIONS,
        SET_CONFIG,
        TUNNEL_MANAGER,
        ROUTER_INFO,
        CLIENT_SERVICES_INFO,
    ];

    /// Existing base methods that remain standard `METHOD_NOT_FOUND`
    /// responses. Keeping these names explicit prevents the support claim
    /// from growing merely because a client happens to request them.
    pub const UNSUPPORTED_BASE: &[&str] = &[
        GET_KEYS,
        GET_RATE,
        ROUTER_MANAGER,
        NETWORK_SETTING,
        ADVANCED_SETTINGS,
    ];

    /// Proposal 170 methods routed by the protected dispatcher.
    pub const PROPOSAL_170: &[&str] = &[ADDRESS_BOOK, TUNNEL_MANAGER, CLIENT_SERVICES_INFO];

    /// Machine-readable method support disposition.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SupportDisposition {
        Authenticate,
        ImplementedBase,
        CompatibilityAlias,
        Proposal170,
        UnsupportedBase,
    }

    /// Exact method support inventory used by documentation and tests.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MethodSupport {
        pub method: &'static str,
        pub disposition: SupportDisposition,
    }

    pub const SUPPORT_INVENTORY: &[MethodSupport] = &[
        MethodSupport {
            method: AUTHENTICATE,
            disposition: SupportDisposition::Authenticate,
        },
        MethodSupport {
            method: ROUTER_INFO,
            disposition: SupportDisposition::ImplementedBase,
        },
        MethodSupport {
            method: GET_KEYS,
            disposition: SupportDisposition::UnsupportedBase,
        },
        MethodSupport {
            method: GET_RATE,
            disposition: SupportDisposition::UnsupportedBase,
        },
        MethodSupport {
            method: ROUTER_MANAGER,
            disposition: SupportDisposition::UnsupportedBase,
        },
        MethodSupport {
            method: NETWORK_SETTING,
            disposition: SupportDisposition::UnsupportedBase,
        },
        MethodSupport {
            method: ADVANCED_SETTINGS,
            disposition: SupportDisposition::UnsupportedBase,
        },
        MethodSupport {
            method: ADDRESS_BOOK,
            disposition: SupportDisposition::Proposal170,
        },
        MethodSupport {
            method: TUNNEL_MANAGER,
            disposition: SupportDisposition::Proposal170,
        },
        MethodSupport {
            method: CLIENT_SERVICES_INFO,
            disposition: SupportDisposition::Proposal170,
        },
        MethodSupport {
            method: SET_SUBSCRIPTIONS,
            disposition: SupportDisposition::CompatibilityAlias,
        },
        MethodSupport {
            method: SET_CONFIG,
            disposition: SupportDisposition::CompatibilityAlias,
        },
    ];
}

/// Proposal 170 tunnel types.
#[allow(dead_code)]
pub mod tunnel_types {
    pub const CLIENT: &str = "client";
    pub const HTTP_CLIENT: &str = "httpclient";
    pub const IRC_CLIENT: &str = "ircclient";
    pub const SOCKS: &str = "socks";
    pub const SOCKS_IRC: &str = "socksirc";
    pub const CONNECT_CLIENT: &str = "connectclient";
    pub const STREAMR_CLIENT: &str = "streamrclient";
    pub const SERVER: &str = "server";
    pub const HTTP_SERVER: &str = "httpserver";
    pub const HTTP_BIDIR_SERVER: &str = "httpbidirserver";
    pub const IRC_SERVER: &str = "ircserver";
    pub const STREAMR_SERVER: &str = "streamrserver";

    /// All valid Proposal 170 tunnel types.
    pub const ALL: &[&str] = &[
        CLIENT,
        HTTP_CLIENT,
        IRC_CLIENT,
        SOCKS,
        SOCKS_IRC,
        CONNECT_CLIENT,
        STREAMR_CLIENT,
        SERVER,
        HTTP_SERVER,
        HTTP_BIDIR_SERVER,
        IRC_SERVER,
        STREAMR_SERVER,
    ];
}

/// TunnelManager actions.
#[allow(dead_code)]
pub mod tunnel_actions {
    /// Emissary compatibility extension; not part of Proposal 170's
    /// canonical action vocabulary.
    pub const LIST: &str = "List";
    pub const CREATE: &str = "Create";
    pub const EDIT: &str = "Edit";
    pub const GET: &str = "Get";
    pub const DELETE: &str = "Delete";
    pub const START: &str = "Start";
    pub const STOP: &str = "Stop";
    pub const RESTART: &str = "Restart";

    /// Canonical Proposal 170 TunnelManager actions.
    pub const CANONICAL: &[&str] = &[
        "create", "edit", "get", "start", "stop", "restart", "delete",
    ];

    /// Already-shipped Emissary action aliases.
    pub const COMPATIBILITY: &[&str] = &[
        "List", "Create", "Edit", "Get", "Start", "Stop", "Restart", "Delete",
    ];
}

/// AddressBook books.
#[allow(dead_code)]
pub mod address_books {
    pub const PRIVATE: &str = "private";
    pub const LOCAL: &str = "local";
    pub const ROUTER: &str = "router";
    pub const PUBLISHED: &str = "published";

    pub const ALL: &[&str] = &[PRIVATE, LOCAL, ROUTER, PUBLISHED];
}

/// AddressBook request modes.
#[allow(dead_code)]
pub mod address_book_requests {
    pub const LIST: &str = "List";
    pub const LOOKUP: &str = "Lookup";
    pub const ADD: &str = "Add";
    pub const UPDATE: &str = "Update";
    pub const DELETE: &str = "Delete";
}

/// Authenticate request parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct AuthenticateParams {
    /// API version. Emissary supports I2PControl API version 1.
    #[serde(rename = "API")]
    pub api: Option<i32>,

    /// Password.
    #[serde(rename = "Password")]
    pub password: Option<String>,
}

/// Authenticate result.
#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
pub struct AuthenticateResult {
    /// Authentication token.
    pub Token: String,

    /// Negotiated API version.
    pub API: i32,
}

/// Proposal 170 RouterInfo selector keys.
/// https://i2p.net/en/proposals/170-i2pcontrol-expansion/
#[allow(dead_code)]
pub mod router_info_keys {
    // --- UDP transport selectors ---
    pub const UDP_ACTIVE: &str = "i2p.router.udp.active";
    pub const UDP_COOKIE_ACTIVE: &str = "i2p.router.udp.cookie.active";
    pub const UDP_INTEGRATED_PEERS: &str = "i2p.router.udp.integratedPeers";
    pub const UDP_FIREWALLED: &str = "i2p.router.udp.firewalled";
    pub const UDP_HIDDEN: &str = "i2p.router.udp.hidden";
    pub const UDP_COINFICIENT_PEERS: &str = "i2p.router.udp.coinficientPeers";
    pub const UDP_CRITICAL_PEERS: &str = "i2p.router.udp.criticalPeers";
    pub const UDP_FAST_PEERS: &str = "i2p.router.udp.fastPeers";
    pub const UDP_HIGH_CAPACITY_PEERS: &str = "i2p.router.udp.highCapacityPeers";
    pub const UDP_INTERLEAVED_PEERS: &str = "i2p.router.udp.interleavedPeers";
    pub const UDP_LIT_PEERS: &str = "i2p.router.udp.litPeers";
    pub const UDP_LOW_CAPACITY_PEERS: &str = "i2p.router.udp.lowCapacityPeers";
    pub const UDP_ON_DEMAND_PEERS: &str = "i2p.router.udp.onDemandPeers";
    pub const UDP_PEER_STATS: &str = "i2p.router.udp.peerStats";
    pub const UDP_STANDARD_PEERS: &str = "i2p.router.udp.standardPeers";
    pub const UDP_UNREACHABLE_PEERS: &str = "i2p.router.udp.unreachablePeers";
    pub const UDP_TOTAL_PEERS: &str = "i2p.router.udp.totalPeers";
    pub const UDP_CURRENT_PEERS: &str = "i2p.router.udp.currentPeers";

    // --- General router selectors ---
    pub const VERSION: &str = "i2p.router.version";
    pub const UPTIME: &str = "i2p.router.uptime";

    // --- NetDB selectors ---
    pub const NETDB_ACTIVE: &str = "i2p.router.netdb.active";
    pub const NETDB_ACTIVE_PROFILES: &str = "i2p.router.netdb.activeProfiles";
    pub const NETDB_HIGHEST_VERSION: &str = "i2p.router.netdb.highestVersion";
    pub const NETDB_KNOWN_PROFILES: &str = "i2p.router.netdb.knownProfiles";
    pub const NETDB_NEW_PROFILES: &str = "i2p.router.netdb.newProfiles";
    pub const NETDB_ACTIVE_ROUTERS: &str = "i2p.router.netdb.activeRouters";
    pub const NETDB_ALREADY_EXPERIENCED_PEERS: &str = "i2p.router.netdb.alreadyExperiencedPeers";
    pub const NETDB_BANLIST_SIZE: &str = "i2p.router.netdb.banlistSize";
    pub const NETDB_EXPLORATORY_PEERS: &str = "i2p.router.netdb.exploratoryPeers";
    pub const NETDB_FAST_PEERS: &str = "i2p.router.netdb.fastPeers";
    pub const NETDB_HIGH_CAPACITY_PEERS: &str = "i2p.router.netdb.highCapacityPeers";
    pub const NETDB_IS_BACKLOGGED: &str = "i2p.router.netdb.isBacklogged";
    pub const NETDB_KNOWN_ACTIVE: &str = "i2p.router.netdb.knownActive";
    pub const NETDB_KNOWN_IDLE: &str = "i2p.router.netdb.knownIdle";
    pub const NETDB_KNOWN_USED: &str = "i2p.router.netdb.knownUsed";
    pub const NETDB_KNOWN_VANILLA: &str = "i2p.router.netdb.knownVanilla";
    pub const NETDB_KNOWN_VOLATILE: &str = "i2p.router.netdb.knownVolatile";
    pub const NETDB_LAST_EXPLORED: &str = "i2p.router.netdb.lastExplored";
    pub const NETDB_LAST_PROFILE_LOOKUP: &str = "i2p.router.netdb.lastProfileLookup";
    pub const NETDB_LAST_ROUTER_LOOKUP: &str = "i2p.router.netdb.lastRouterLookup";
    pub const NETDB_LAST_UNSAVED: &str = "i2p.router.netdb.lastUnsaved";
    pub const NETDB_LEASE_SETS: &str = "i2p.router.netdb.leaseSets";
    pub const NETDB_NEW_ACTIVE: &str = "i2p.router.netdb.newActive";
    pub const NETDB_NEW_IDLE: &str = "i2p.router.netdb.newIdle";
    pub const NETDB_OLD_ACTIVE: &str = "i2p.router.netdb.oldActive";
    pub const NETDB_OLD_IDLE: &str = "i2p.router.netdb.oldIdle";
    pub const NETDB_PEER_PROFILES: &str = "i2p.router.netdb.peerProfiles";
    pub const NETDB_PLAINTEXT_PEERS: &str = "i2p.router.netdb.plaintextPeers";
    pub const NETDB_RESERVE_ACTIVE: &str = "i2p.router.netdb.reserveActive";
    pub const NETDB_RESERVE_ACTIVE_PEERS: &str = "i2p.router.netdb.reserveActivePeers";
    pub const NETDB_RESERVE_HIGH_CAPACITY: &str = "i2p.router.netdb.reserveHighCapacity";
    pub const NETDB_RESERVE_INTEGRATED: &str = "i2p.router.netdb.reserveIntegrated";
    pub const NETDB_RESERVE_KNOWN: &str = "i2p.router.netdb.reserveKnown";
    pub const NETDB_RESERVE_LOOKUP: &str = "i2p.router.netdb.reserveLookup";
    pub const NETDB_RESERVE_PENDING: &str = "i2p.router.netdb.reservePending";
    pub const NETDB_RESERVE_RESERVED: &str = "i2p.router.netdb.reserveReserved";
    pub const NETDB_RESERVE_STANDARD: &str = "i2p.router.netdb.reserveStandard";
    pub const NETDB_RESERVE_TIER2: &str = "i2p.router.netdb.reserveTier2";
    pub const NETDB_RESERVE_USED: &str = "i2p.router.netdb.reserveUsed";
    pub const NETDB_RESERVE_VOLATILE: &str = "i2p.router.netdb.reserveVolatile";
    pub const NETDB_STANDARD_PEERS: &str = "i2p.router.netdb.standardPeers";
    pub const NETDB_LOW_CAPACITY_PEERS: &str = "i2p.router.netdb.lowCapacityPeers";
    pub const NETDB_TUNNELS: &str = "i2p.router.netdb.tunnels";
    pub const NETDB_USED_PEERS: &str = "i2p.router.netdb.usedPeers";
    pub const NETDB_VOLATILE_PEERS: &str = "i2p.router.netdb.volatilePeers";
    pub const NETDB_ADDRESS_BOOKS: &str = "i2p.router.netdb.addressBooks";
    pub const NETDB_ADDRESS_BOOK_ENTRIES: &str = "i2p.router.netdb.addressBookEntries";
    pub const NETDB_ADDRESS_BOOK_SOURCES: &str = "i2p.router.netdb.addressBookSources";
    pub const NETDB_ADDRESS_BOOK_SUBSCRIPTIONS: &str = "i2p.router.netdb.addressBookSubscriptions";
    pub const NETDB_ADDRESS_BOOK_UPDATES: &str = "i2p.router.netdb.addressBookUpdates";

    // --- Bandwidth selectors ---
    pub const BW_INBOUND_1S: &str = "i2p.router.bw.inbound.1s";
    pub const BW_INBOUND_15S: &str = "i2p.router.bw.inbound.15s";
    pub const BW_INBOUND_1M: &str = "i2p.router.bw.inbound.1m";
    pub const BW_INBOUND_1H: &str = "i2p.router.bw.inbound.1h";
    pub const BW_INBOUND_1D: &str = "i2p.router.bw.inbound.1d";
    pub const BW_INBOUND_TOTAL: &str = "i2p.router.bw.inbound.total";
    pub const BW_OUTBOUND_1S: &str = "i2p.router.bw.outbound.1s";
    pub const BW_OUTBOUND_15S: &str = "i2p.router.bw.outbound.15s";
    pub const BW_OUTBOUND_1M: &str = "i2p.router.bw.outbound.1m";
    pub const BW_OUTBOUND_1H: &str = "i2p.router.bw.outbound.1h";
    pub const BW_OUTBOUND_1D: &str = "i2p.router.bw.outbound.1d";
    pub const BW_OUTBOUND_TOTAL: &str = "i2p.router.bw.outbound.total";

    // --- TCP transport selectors ---
    pub const TCP_ACTIVE: &str = "i2p.router.tcp.active";
    pub const TCP_INTEGRATED_PEERS: &str = "i2p.router.tcp.integratedPeers";
    pub const TCP_FIREWALLED: &str = "i2p.router.tcp.firewalled";
    pub const TCP_HOSTS: &str = "i2p.router.tcp.hosts";
    pub const TCP_STATUS: &str = "i2p.router.tcp.status";
    pub const TCP_VERSION: &str = "i2p.router.tcp.version";

    // --- Identity and network selectors ---
    pub const IDENTITY: &str = "i2p.router.identity";
    pub const NET_BW_INBOUND: &str = "i2p.router.net.bw.inbound";
    pub const NET_BW_OUTBOUND: &str = "i2p.router.net.bw.outbound";

    // --- Router news ---
    pub const ROUTER_NEWS: &str = "i2p.router.news";

    // --- Proposal 170 exact additions ---
    pub const P170_ID: &str = "i2p.router.id";
    pub const P170_CLOCKSKEW: &str = "i2p.router.clockskew";
    pub const P170_INFO: &str = "i2p.router.info";
    pub const P170_LOGS: &str = "i2p.router.logs";
    pub const P170_LOGS_CLEAR: &str = "i2p.router.logs.clear";
    pub const P170_NET_TOTAL_RECEIVED_BYTES: &str = "i2p.router.net.total.received.bytes";
    pub const P170_NET_TOTAL_SENT_BYTES: &str = "i2p.router.net.total.sent.bytes";
    pub const P170_NET_TOTAL_TRANSIT_BYTES: &str = "i2p.router.net.total.transit.bytes";
    pub const P170_NET_BW_TRANSIT_15S: &str = "i2p.router.net.bw.transit.15s";
    pub const P170_NET_TUNNELS_SHARE_RATIO: &str = "i2p.router.net.tunnels.shareratio";
    pub const P170_NET_TUNNELS_PARTICIPATING_INFO: &str =
        "i2p.router.net.tunnels.participating.info";
    pub const P170_NET_TUNNELS_I2PTUNNEL: &str = "i2p.router.net.tunnels.i2ptunnel";
    pub const P170_NET_TUNNELS_EXPLORATORY_INBOUND: &str =
        "i2p.router.net.tunnels.exploratory.inbound";
    pub const P170_NET_TUNNELS_EXPLORATORY_OUTBOUND: &str =
        "i2p.router.net.tunnels.exploratory.outbound";
    pub const P170_NET_TUNNELS_EXPLORATORY_INFO_LIST: &str =
        "i2p.router.net.tunnels.exploratory.info.list";
    pub const P170_NET_TUNNELS_CLIENT_INBOUND: &str = "i2p.router.net.tunnels.client.inbound";
    pub const P170_NET_TUNNELS_CLIENT_OUTBOUND: &str = "i2p.router.net.tunnels.client.outbound";
    pub const P170_NET_TUNNELS_CLIENT_INFO_LIST: &str = "i2p.router.net.tunnels.client.info.list";
    pub const P170_NET_STATUS_V6: &str = "i2p.router.net.status.v6";
    pub const P170_NET_ERROR: &str = "i2p.router.net.error";
    pub const P170_NET_ERROR_V6: &str = "i2p.router.net.error.v6";
    pub const P170_NET_TESTING: &str = "i2p.router.net.testing";
    pub const P170_NET_TESTING_V6: &str = "i2p.router.net.testing.v6";
    pub const P170_NET_TUNNELS_SUCCESS_RATE: &str = "i2p.router.net.tunnels.successrate";
    pub const P170_NET_TUNNELS_TOTAL_SUCCESS_RATE: &str = "i2p.router.net.tunnels.totalsuccessrate";
    pub const P170_NET_TUNNELS_QUEUE: &str = "i2p.router.net.tunnels.queue";
    pub const P170_NET_TUNNELS_TBM_QUEUE: &str = "i2p.router.net.tunnels.tbmqueue";
    pub const P170_NETDB_PEERS: &str = "i2p.router.netdb.peers";
    pub const P170_NETDB_ACTIVE_PEERS_INFO: &str = "i2p.router.netdb.activepeers.info";
    pub const P170_NETDB_NTCP_LIMIT: &str = "i2p.router.netdb.ntcp.limit";
    pub const P170_NETDB_SSU_LIMIT: &str = "i2p.router.netdb.ssu.limit";
    pub const P170_NETDB_BANNED_PEERS: &str = "i2p.router.netdb.bannedpeers";
    pub const P170_NETDB_ACTIVE_PEERS_LIST: &str = "i2p.router.netdb.activepeers.list";
    pub const P170_NETDB_PEERS_LIST: &str = "i2p.router.netdb.peers.list";
    pub const P170_NETDB_PEERS_INFO: &str = "i2p.router.netdb.peers.info";
    pub const P170_NETDB_ACTIVE_PEERS_STATS: &str = "i2p.router.netdb.activepeers.stats";

    // --- Clock skew ---
    pub const CLOCK_SKEW: &str = "i2p.router.clock.skew";

    // --- Share ratio and configured BW ---
    pub const SHARE_RATIO: &str = "i2p.router.shareRatio";
    pub const CONFIGURED_BW_INBOUND: &str = "i2p.router.configuredBwInbound";
    pub const CONFIGURED_BW_OUTBOUND: &str = "i2p.router.configuredBwOutbound";

    // --- Tunnel selectors ---
    pub const TUNNELS_PARTICIPATING: &str = "i2p.router.tunnels.participating";
    pub const TUNNELS_EXPLORATORY_IN: &str = "i2p.router.tunnels.exploratoryIn";
    pub const TUNNELS_EXPLORATORY_OUT: &str = "i2p.router.tunnels.exploratoryOut";
    pub const TUNNELS_CLIENT_IN: &str = "i2p.router.tunnels.clientIn";
    pub const TUNNELS_CLIENT_OUT: &str = "i2p.router.tunnels.clientOut";
    pub const TUNNELS_CONFIGURED: &str = "i2p.router.tunnels.configured";
    pub const TUNNELS_QUEUE: &str = "i2p.router.tunnels.queue";

    // --- Peer selectors ---
    pub const PEERS_KNOWN_COUNT: &str = "i2p.router.peers.knownCount";
    pub const PEERS_KNOWN: &str = "i2p.router.peers.known";
    pub const PEERS_ACTIVE_COUNT: &str = "i2p.router.peers.activeCount";
    pub const PEERS_ACTIVE: &str = "i2p.router.peers.active";
    pub const PEERS_ROUTER_INFO: &str = "i2p.router.peers.routerInfo";
    pub const PEERS_BANNED: &str = "i2p.router.peers.banned";
    pub const PEERS_BANNED_COUNT: &str = "i2p.router.peers.bannedCount";
    pub const PEERS_LIMITS: &str = "i2p.router.peers.limits";
    pub const PEERS_ACTIVE_STATS: &str = "i2p.router.peers.activeStats";

    // --- I2PTunnel selectors ---
    pub const NET_IPTUNNELS: &str = "i2p.router.net.i2ptunnels";

    // --- Log selectors ---
    pub const LOG_SNAPSHOT: &str = "i2p.router.log";
    pub const LOG_CLEAR: &str = "i2p.router.log.clear";

    // --- Address-book selectors (owned by M003) ---
    pub const ADDRESS_BOOK_PRIVATE: &str = "i2p.router.addressbook.private";
    pub const ADDRESS_BOOK_LOCAL: &str = "i2p.router.addressbook.local";
    pub const ADDRESS_BOOK_ROUTER: &str = "i2p.router.addressbook.router";
    pub const ADDRESS_BOOK_PUBLISHED: &str = "i2p.router.addressbook.published";
    pub const ADDRESS_BOOK_SUBSCRIPTIONS: &str = "i2p.router.addressbook.subscriptions";
    pub const ADDRESS_BOOK_CONFIG: &str = "i2p.router.addressbook.config";

    pub const P170_ADDRESS_BOOK_PRIVATE_LIST: &str = "i2p.router.addressbook.private.list";
    pub const P170_ADDRESS_BOOK_LOCAL_LIST: &str = "i2p.router.addressbook.local.list";
    pub const P170_ADDRESS_BOOK_ROUTER_LIST: &str = "i2p.router.addressbook.router.list";
    pub const P170_ADDRESS_BOOK_PUBLISHED_LIST: &str = "i2p.router.addressbook.published.list";
    pub const P170_ADDRESS_BOOK_SUBSCRIPTIONS: &str = "i2p.router.addressbook.subscriptions";
    pub const P170_ADDRESS_BOOK_CONFIG: &str = "i2p.router.addressbook.config";

    /// JSON types used by the pinned Proposal 170 RouterInfo additions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum JsonType {
        String,
        NullableString,
        NullableInteger,
        Integer,
        Number,
        ArrayOfStrings,
        ArrayOfObjects,
        ArrayOfStringMaps,
        ObjectMapOfObjects,
        ObjectWithPathAndEntries,
    }

    impl JsonType {
        /// Human-readable contract spelling used by the source-map guard.
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::String => "string",
                Self::NullableString => "string|null",
                Self::NullableInteger => "integer|null",
                Self::Integer => "integer",
                Self::Number => "number",
                Self::ArrayOfStrings => "array<string>",
                Self::ArrayOfObjects => "array<object>",
                Self::ArrayOfStringMaps => "array<map<string,string>>",
                Self::ObjectMapOfObjects => "map<string,map<string,object>>",
                Self::ObjectWithPathAndEntries => "object{path,entries}",
            }
        }
    }

    /// Whether a selector is read-only or is the one permitted log mutation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Mutation {
        ReadOnly,
        ClearLogBuffer,
    }

    /// Result bound recorded in the canonical source matrix.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Bound {
        None,
        Items(usize),
        Bytes(usize),
        ItemsAndBytes { items: usize, bytes: usize },
    }

    /// Truthful current-source disposition for a canonical RouterInfo addition.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SourceDisposition {
        Available {
            owner: &'static str,
        },
        Neutral {
            owner: &'static str,
            reason: &'static str,
        },
        Unavailable {
            owner: &'static str,
            reason: &'static str,
        },
    }

    impl SourceDisposition {
        /// Whether this disposition may be serialized for a request.
        pub const fn is_requestable(self) -> bool {
            matches!(self, Self::Available { .. } | Self::Neutral { .. })
        }

        /// Stable source classification used in documentation and tests.
        pub const fn class(self) -> &'static str {
            match self {
                Self::Available { .. } => "available",
                Self::Neutral { .. } => "protocol-permitted neutral",
                Self::Unavailable { .. } => "unavailable",
            }
        }

        /// The authoritative owner or the owner group that must implement it.
        pub const fn owner(self) -> &'static str {
            match self {
                Self::Available { owner }
                | Self::Neutral { owner, .. }
                | Self::Unavailable { owner, .. } => owner,
            }
        }

        /// The adjudication reason for neutral or unavailable fields.
        pub const fn reason(self) -> Option<&'static str> {
            match self {
                Self::Available { .. } => None,
                Self::Neutral { reason, .. } | Self::Unavailable { reason, .. } => Some(reason),
            }
        }
    }

    /// Machine-checkable contract/source inventory for the pinned revision.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ContractField {
        pub key: &'static str,
        pub json_type: JsonType,
        pub direct_presence: bool,
        pub mutation: Mutation,
        pub source: SourceDisposition,
        pub serializer: &'static str,
        pub fixture: &'static str,
        pub bound: Bound,
        /// Exact base selector spelling when this canonical key is also a base key.
        pub base_alias: Option<&'static str>,
    }

    /// The exact 43-key Proposal 170 RouterInfo addition set.
    pub const PROPOSAL_170_ADDITIONS: &[&str; 43] = &[
        ROUTER_NEWS,
        P170_ID,
        P170_CLOCKSKEW,
        P170_INFO,
        P170_LOGS,
        P170_LOGS_CLEAR,
        P170_NET_TOTAL_RECEIVED_BYTES,
        P170_NET_TOTAL_SENT_BYTES,
        P170_NET_TOTAL_TRANSIT_BYTES,
        P170_NET_BW_TRANSIT_15S,
        P170_NET_TUNNELS_SHARE_RATIO,
        P170_NET_TUNNELS_PARTICIPATING_INFO,
        P170_NET_TUNNELS_I2PTUNNEL,
        P170_NET_TUNNELS_EXPLORATORY_INBOUND,
        P170_NET_TUNNELS_EXPLORATORY_OUTBOUND,
        P170_NET_TUNNELS_EXPLORATORY_INFO_LIST,
        P170_NET_TUNNELS_CLIENT_INBOUND,
        P170_NET_TUNNELS_CLIENT_OUTBOUND,
        P170_NET_TUNNELS_CLIENT_INFO_LIST,
        P170_NET_STATUS_V6,
        P170_NET_ERROR,
        P170_NET_ERROR_V6,
        P170_NET_TESTING,
        P170_NET_TESTING_V6,
        P170_NET_TUNNELS_SUCCESS_RATE,
        P170_NET_TUNNELS_TOTAL_SUCCESS_RATE,
        P170_NET_TUNNELS_QUEUE,
        P170_NET_TUNNELS_TBM_QUEUE,
        P170_NETDB_PEERS,
        P170_NETDB_ACTIVE_PEERS_INFO,
        P170_NETDB_NTCP_LIMIT,
        P170_NETDB_SSU_LIMIT,
        P170_NETDB_BANNED_PEERS,
        P170_NETDB_ACTIVE_PEERS_LIST,
        P170_NETDB_PEERS_LIST,
        P170_NETDB_PEERS_INFO,
        P170_NETDB_ACTIVE_PEERS_STATS,
        P170_ADDRESS_BOOK_PRIVATE_LIST,
        P170_ADDRESS_BOOK_LOCAL_LIST,
        P170_ADDRESS_BOOK_ROUTER_LIST,
        P170_ADDRESS_BOOK_PUBLISHED_LIST,
        P170_ADDRESS_BOOK_SUBSCRIPTIONS,
        P170_ADDRESS_BOOK_CONFIG,
    ];

    macro_rules! field {
        ($key:expr, $json_type:expr, $source:expr, $serializer:literal, $fixture:literal,
            $mutation:expr, $bound:expr, $base_alias:expr) => {
            ContractField {
                key: $key,
                json_type: $json_type,
                direct_presence: true,
                mutation: $mutation,
                source: $source,
                serializer: $serializer,
                fixture: $fixture,
                bound: $bound,
                base_alias: $base_alias,
            }
        };
    }

    /// Types, source dispositions, serializers, fixtures, and bounds for every
    /// canonical addition. This is the reviewed M025 matrix; the handler and
    /// documentation tests consume this table rather than maintaining a second
    /// availability list.
    pub const PROPOSAL_170_CONTRACT: &[ContractField; 43] = &[
        field!(
            ROUTER_NEWS,
            JsonType::String,
            SourceDisposition::Available {
                owner: "i2pcontrol-router-news",
            },
            "serialize_router_news",
            "p170.router_news.string",
            Mutation::ReadOnly,
            Bound::Bytes(1024 * 1024),
            Some(ROUTER_NEWS)
        ),
        field!(
            P170_ID,
            JsonType::NullableString,
            SourceDisposition::Available {
                owner: "startup-retained"
            },
            "serialize_router_id",
            "p170.router_id.nullable_string",
            Mutation::ReadOnly,
            Bound::Bytes(4 * 1024),
            None
        ),
        field!(
            P170_CLOCKSKEW,
            JsonType::NullableInteger,
            SourceDisposition::Neutral {
                owner: "router-info-control",
                reason: "protocol permits null when no peer estimate exists"
            },
            "serialize_clockskew",
            "p170.clockskew.nullable_integer",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_INFO,
            JsonType::NullableString,
            SourceDisposition::Available {
                owner: "startup-retained"
            },
            "serialize_router_info",
            "p170.router_info.nullable_string",
            Mutation::ReadOnly,
            Bound::Bytes(4 * 1024 * 1024),
            None
        ),
        field!(
            P170_LOGS,
            JsonType::ArrayOfStrings,
            SourceDisposition::Available {
                owner: "i2pcontrol-log-ring"
            },
            "serialize_log_messages",
            "p170.logs.string_list",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 10 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_LOGS_CLEAR,
            JsonType::String,
            SourceDisposition::Available {
                owner: "i2pcontrol-log-ring"
            },
            "serialize_log_clear",
            "p170.logs_clear.success",
            Mutation::ClearLogBuffer,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TOTAL_RECEIVED_BYTES,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "event-metrics"
            },
            "serialize_total_received_bytes",
            "p170.total_received.integer",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TOTAL_SENT_BYTES,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "event-metrics"
            },
            "serialize_total_sent_bytes",
            "p170.total_sent.integer",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TOTAL_TRANSIT_BYTES,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "event-metrics"
            },
            "serialize_total_transit_bytes",
            "p170.total_transit.integer",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_BW_TRANSIT_15S,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "i2pcontrol-transit-bandwidth-sampler"
            },
            "serialize_transit_bandwidth_15s",
            "p170.transit_15s.bytes_per_second",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_SHARE_RATIO,
            JsonType::Number,
            SourceDisposition::Available {
                owner: "retained-configuration"
            },
            "serialize_tunnel_share_ratio",
            "p170.share_ratio.number",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_PARTICIPATING_INFO,
            JsonType::ArrayOfObjects,
            SourceDisposition::Available {
                owner: "tunnel-inspection"
            },
            "serialize_participating_tunnel_info",
            "p170.participating_info.rows",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NET_TUNNELS_I2PTUNNEL,
            JsonType::ArrayOfObjects,
            SourceDisposition::Available {
                owner: "startup-tunnel-inventory"
            },
            "serialize_i2ptunnel_quick_info",
            "p170.i2ptunnel.quick_info",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 1_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NET_TUNNELS_EXPLORATORY_INBOUND,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "tunnel-inspection"
            },
            "serialize_exploratory_inbound",
            "p170.exploratory_inbound.count",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_EXPLORATORY_OUTBOUND,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "tunnel-inspection"
            },
            "serialize_exploratory_outbound",
            "p170.exploratory_outbound.count",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_EXPLORATORY_INFO_LIST,
            JsonType::ArrayOfObjects,
            SourceDisposition::Available {
                owner: "tunnel-inspection"
            },
            "serialize_exploratory_info_list",
            "p170.exploratory_info.rows",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NET_TUNNELS_CLIENT_INBOUND,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "tunnel-inspection"
            },
            "serialize_client_inbound",
            "p170.client_inbound.count",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_CLIENT_OUTBOUND,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "tunnel-inspection"
            },
            "serialize_client_outbound",
            "p170.client_outbound.count",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_CLIENT_INFO_LIST,
            JsonType::ArrayOfObjects,
            SourceDisposition::Available {
                owner: "tunnel-inspection"
            },
            "serialize_client_info_list",
            "p170.client_info.rows",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NET_STATUS_V6,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "network-state",
            },
            "serialize_network_status_v6",
            "p170.status_v6.integer",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_ERROR,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "network-error-state",
            },
            "serialize_network_error",
            "p170.error_v4.integer",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_ERROR_V6,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "network-error-state",
            },
            "serialize_network_error_v6",
            "p170.error_v6.integer",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TESTING,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "network-state",
            },
            "serialize_network_testing",
            "p170.testing_v4.integer",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TESTING_V6,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "network-state",
            },
            "serialize_network_testing_v6",
            "p170.testing_v6.integer",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_SUCCESS_RATE,
            JsonType::Number,
            SourceDisposition::Available {
                owner: "tunnel-build-metrics"
            },
            "serialize_tunnel_success_rate",
            "p170.success_rate.recent.percent",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_TOTAL_SUCCESS_RATE,
            JsonType::Number,
            SourceDisposition::Available {
                owner: "event-metrics"
            },
            "serialize_total_tunnel_success_rate",
            "p170.success_rate.total.percent",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_QUEUE,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "tunnel-inspection"
            },
            "serialize_tunnel_queue",
            "p170.tunnel_queue.depth",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NET_TUNNELS_TBM_QUEUE,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "tunnel-inspection"
            },
            "serialize_tbm_queue",
            "p170.tbm_queue.depth",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NETDB_PEERS,
            JsonType::ArrayOfStrings,
            SourceDisposition::Available {
                owner: "live-profile-storage-inspection"
            },
            "serialize_netdb_peer_hashes",
            "p170.netdb.peer_hashes",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NETDB_ACTIVE_PEERS_INFO,
            JsonType::ArrayOfStrings,
            SourceDisposition::Available {
                owner: "transport-and-live-profile-storage-inspection"
            },
            "serialize_active_peer_router_infos",
            "p170.netdb.active_peer_info",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NETDB_NTCP_LIMIT,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "transport-manager-configuration"
            },
            "serialize_ntcp_limit",
            "p170.netdb.ntcp_limit",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NETDB_SSU_LIMIT,
            JsonType::Integer,
            SourceDisposition::Available {
                owner: "transport-manager-configuration"
            },
            "serialize_ssu_limit",
            "p170.netdb.ssu_limit",
            Mutation::ReadOnly,
            Bound::None,
            None
        ),
        field!(
            P170_NETDB_BANNED_PEERS,
            JsonType::ObjectMapOfObjects,
            SourceDisposition::Available {
                owner: "router-ban-empty-marker"
            },
            "serialize_banned_peers",
            "p170.netdb.banned_peers.empty_map",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NETDB_ACTIVE_PEERS_LIST,
            JsonType::ArrayOfStrings,
            SourceDisposition::Available {
                owner: "transport-manager-inspection"
            },
            "serialize_active_peer_hashes",
            "p170.netdb.active_peers",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NETDB_PEERS_LIST,
            JsonType::ArrayOfStrings,
            SourceDisposition::Available {
                owner: "live-profile-storage-inspection"
            },
            "serialize_known_peer_hashes",
            "p170.netdb.peer_list",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NETDB_PEERS_INFO,
            JsonType::ArrayOfStrings,
            SourceDisposition::Available {
                owner: "live-profile-storage-inspection"
            },
            "serialize_peer_router_infos",
            "p170.netdb.peer_info",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_NETDB_ACTIVE_PEERS_STATS,
            JsonType::ArrayOfObjects,
            SourceDisposition::Available {
                owner: "transport-manager-inspection"
            },
            "serialize_active_peer_stats",
            "p170.netdb.active_peer_stats",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_ADDRESS_BOOK_PRIVATE_LIST,
            JsonType::ArrayOfStringMaps,
            SourceDisposition::Available {
                owner: "runtime-address-book-handle"
            },
            "serialize_address_book_private_list",
            "p170.addressbook.private.list",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_ADDRESS_BOOK_LOCAL_LIST,
            JsonType::ArrayOfStringMaps,
            SourceDisposition::Available {
                owner: "runtime-address-book-handle"
            },
            "serialize_address_book_local_list",
            "p170.addressbook.local.list",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_ADDRESS_BOOK_ROUTER_LIST,
            JsonType::ArrayOfStringMaps,
            SourceDisposition::Available {
                owner: "runtime-address-book-handle"
            },
            "serialize_address_book_router_list",
            "p170.addressbook.router.list",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_ADDRESS_BOOK_PUBLISHED_LIST,
            JsonType::ArrayOfStringMaps,
            SourceDisposition::Available {
                owner: "runtime-address-book-handle"
            },
            "serialize_address_book_published_list",
            "p170.addressbook.published.list",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 10_000,
                bytes: 4 * 1024 * 1024
            },
            None
        ),
        field!(
            P170_ADDRESS_BOOK_SUBSCRIPTIONS,
            JsonType::ObjectWithPathAndEntries,
            SourceDisposition::Available {
                owner: "runtime-address-book-handle"
            },
            "serialize_address_book_subscriptions",
            "p170.addressbook.subscriptions.object",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 1_000,
                bytes: 4 * 1024 * 1024
            },
            Some(ADDRESS_BOOK_SUBSCRIPTIONS)
        ),
        field!(
            P170_ADDRESS_BOOK_CONFIG,
            JsonType::ObjectWithPathAndEntries,
            SourceDisposition::Available {
                owner: "runtime-address-book-handle"
            },
            "serialize_address_book_config",
            "p170.addressbook.config.object",
            Mutation::ReadOnly,
            Bound::ItemsAndBytes {
                items: 1_000,
                bytes: 4 * 1024 * 1024
            },
            Some(ADDRESS_BOOK_CONFIG)
        ),
    ];

    /// True when a selector belongs to the exact Proposal 170 addition set.
    pub fn is_proposal_170_addition(key: &str) -> bool {
        PROPOSAL_170_ADDITIONS.contains(&key)
    }

    /// All address-book selector keys.
    pub const ADDRESS_BOOK_KEYS: &[&str] = &[
        ADDRESS_BOOK_PRIVATE,
        ADDRESS_BOOK_LOCAL,
        ADDRESS_BOOK_ROUTER,
        ADDRESS_BOOK_PUBLISHED,
        ADDRESS_BOOK_SUBSCRIPTIONS,
        ADDRESS_BOOK_CONFIG,
    ];

    /// The historical base selector inventories remain separate so a nested
    /// compatibility request cannot accidentally select a Proposal 170-only
    /// key. `CORE_KEYS` and `ADDRESS_BOOK_KEYS` are the literal legacy
    /// inventories; their serializers are recorded below for the exact names
    /// that overlap the Proposal 170 additions.
    pub const BASE_ROUTER_INFO_SERIALIZERS: &[(&str, &str)] = &[
        (ROUTER_NEWS, "serialize_legacy_router_news"),
        (
            ADDRESS_BOOK_SUBSCRIPTIONS,
            "serialize_legacy_address_book_subscriptions",
        ),
        (ADDRESS_BOOK_CONFIG, "serialize_legacy_address_book_config"),
    ];

    /// Exact-name overlap between the retained base inventory and Proposal
    /// 170. Every row has a mode-specific serializer disposition.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SelectorOverlap {
        pub key: &'static str,
        pub base_serializer: &'static str,
        pub proposal_170_serializer: &'static str,
    }

    pub const ROUTER_INFO_SELECTOR_OVERLAPS: &[SelectorOverlap; 3] = &[
        SelectorOverlap {
            key: ROUTER_NEWS,
            base_serializer: "serialize_legacy_router_news",
            proposal_170_serializer: "serialize_router_news",
        },
        SelectorOverlap {
            key: ADDRESS_BOOK_SUBSCRIPTIONS,
            base_serializer: "serialize_legacy_address_book_subscriptions",
            proposal_170_serializer: "serialize_address_book_subscriptions",
        },
        SelectorOverlap {
            key: ADDRESS_BOOK_CONFIG,
            base_serializer: "serialize_legacy_address_book_config",
            proposal_170_serializer: "serialize_address_book_config",
        },
    ];

    /// Existing I2PControl RouterInfo inventories, kept separate from the
    /// Proposal 170 additions. These aliases make the partition explicit to
    /// callers and static guards without changing the established key sets.
    pub const BASE_ROUTER_INFO_CORE_KEYS: &[&str] = CORE_KEYS;
    pub const BASE_ROUTER_INFO_ADDRESS_BOOK_KEYS: &[&str] = ADDRESS_BOOK_KEYS;

    /// Test whether a selector belongs to the retained/base RouterInfo
    /// inventory, independent of the Proposal 170 additions.
    pub fn is_base_router_info_selector(key: &str) -> bool {
        BASE_ROUTER_INFO_CORE_KEYS.contains(&key)
            || BASE_ROUTER_INFO_ADDRESS_BOOK_KEYS.contains(&key)
    }

    /// Test whether a selector is accepted by direct Proposal 170 mode.
    pub fn is_direct_router_info_selector(key: &str) -> bool {
        is_base_router_info_selector(key) || PROPOSAL_170_ADDITIONS.contains(&key)
    }

    /// Emissary compatibility form; it is a request envelope, not a canonical
    /// selector and must never be counted as one of the 43 additions.
    pub const COMPATIBILITY_ROUTER_INFO_FORMS: &[&str] = &["Selector"];

    /// All legacy/base and exact Proposal 170 RouterInfo selector keys.
    pub const ALL: &[&str] = &[
        // UDP
        UDP_ACTIVE,
        UDP_COOKIE_ACTIVE,
        UDP_INTEGRATED_PEERS,
        UDP_FIREWALLED,
        UDP_HIDDEN,
        UDP_COINFICIENT_PEERS,
        UDP_CRITICAL_PEERS,
        UDP_FAST_PEERS,
        UDP_HIGH_CAPACITY_PEERS,
        UDP_INTERLEAVED_PEERS,
        UDP_LIT_PEERS,
        UDP_LOW_CAPACITY_PEERS,
        UDP_ON_DEMAND_PEERS,
        UDP_PEER_STATS,
        UDP_STANDARD_PEERS,
        UDP_UNREACHABLE_PEERS,
        UDP_TOTAL_PEERS,
        UDP_CURRENT_PEERS,
        // General
        VERSION,
        UPTIME,
        // NetDB
        NETDB_ACTIVE,
        NETDB_ACTIVE_PROFILES,
        NETDB_HIGHEST_VERSION,
        NETDB_KNOWN_PROFILES,
        NETDB_NEW_PROFILES,
        NETDB_ACTIVE_ROUTERS,
        NETDB_ALREADY_EXPERIENCED_PEERS,
        NETDB_BANLIST_SIZE,
        NETDB_EXPLORATORY_PEERS,
        NETDB_FAST_PEERS,
        NETDB_HIGH_CAPACITY_PEERS,
        NETDB_IS_BACKLOGGED,
        NETDB_KNOWN_ACTIVE,
        NETDB_KNOWN_IDLE,
        NETDB_KNOWN_USED,
        NETDB_KNOWN_VANILLA,
        NETDB_KNOWN_VOLATILE,
        NETDB_LAST_EXPLORED,
        NETDB_LAST_PROFILE_LOOKUP,
        NETDB_LAST_ROUTER_LOOKUP,
        NETDB_LAST_UNSAVED,
        NETDB_LEASE_SETS,
        NETDB_NEW_ACTIVE,
        NETDB_NEW_IDLE,
        NETDB_OLD_ACTIVE,
        NETDB_OLD_IDLE,
        NETDB_PEER_PROFILES,
        NETDB_PLAINTEXT_PEERS,
        NETDB_RESERVE_ACTIVE,
        NETDB_RESERVE_ACTIVE_PEERS,
        NETDB_RESERVE_HIGH_CAPACITY,
        NETDB_RESERVE_INTEGRATED,
        NETDB_RESERVE_KNOWN,
        NETDB_RESERVE_LOOKUP,
        NETDB_RESERVE_PENDING,
        NETDB_RESERVE_RESERVED,
        NETDB_RESERVE_STANDARD,
        NETDB_RESERVE_TIER2,
        NETDB_RESERVE_USED,
        NETDB_RESERVE_VOLATILE,
        NETDB_STANDARD_PEERS,
        NETDB_LOW_CAPACITY_PEERS,
        NETDB_TUNNELS,
        NETDB_USED_PEERS,
        NETDB_VOLATILE_PEERS,
        NETDB_ADDRESS_BOOKS,
        NETDB_ADDRESS_BOOK_ENTRIES,
        NETDB_ADDRESS_BOOK_SOURCES,
        NETDB_ADDRESS_BOOK_SUBSCRIPTIONS,
        NETDB_ADDRESS_BOOK_UPDATES,
        // Bandwidth
        BW_INBOUND_1S,
        BW_INBOUND_15S,
        BW_INBOUND_1M,
        BW_INBOUND_1H,
        BW_INBOUND_1D,
        BW_INBOUND_TOTAL,
        BW_OUTBOUND_1S,
        BW_OUTBOUND_15S,
        BW_OUTBOUND_1M,
        BW_OUTBOUND_1H,
        BW_OUTBOUND_1D,
        BW_OUTBOUND_TOTAL,
        // TCP
        TCP_ACTIVE,
        TCP_INTEGRATED_PEERS,
        TCP_FIREWALLED,
        TCP_HOSTS,
        TCP_STATUS,
        TCP_VERSION,
        // Identity/Network
        IDENTITY,
        NET_BW_INBOUND,
        NET_BW_OUTBOUND,
        // Router news, clock, share ratio, configured BW
        ROUTER_NEWS,
        CLOCK_SKEW,
        SHARE_RATIO,
        CONFIGURED_BW_INBOUND,
        CONFIGURED_BW_OUTBOUND,
        // Tunnels
        TUNNELS_PARTICIPATING,
        TUNNELS_EXPLORATORY_IN,
        TUNNELS_EXPLORATORY_OUT,
        TUNNELS_CLIENT_IN,
        TUNNELS_CLIENT_OUT,
        TUNNELS_CONFIGURED,
        TUNNELS_QUEUE,
        // Peers
        PEERS_KNOWN_COUNT,
        PEERS_KNOWN,
        PEERS_ACTIVE_COUNT,
        PEERS_ACTIVE,
        PEERS_ROUTER_INFO,
        PEERS_BANNED,
        PEERS_BANNED_COUNT,
        PEERS_LIMITS,
        PEERS_ACTIVE_STATS,
        // I2PTunnel
        NET_IPTUNNELS,
        // Logs
        LOG_SNAPSHOT,
        LOG_CLEAR,
        // Address book
        ADDRESS_BOOK_PRIVATE,
        ADDRESS_BOOK_LOCAL,
        ADDRESS_BOOK_ROUTER,
        ADDRESS_BOOK_PUBLISHED,
        ADDRESS_BOOK_SUBSCRIPTIONS,
        ADDRESS_BOOK_CONFIG,
        // Exact Proposal 170 additions not already present above.
        P170_ID,
        P170_CLOCKSKEW,
        P170_INFO,
        P170_LOGS,
        P170_LOGS_CLEAR,
        P170_NET_TOTAL_RECEIVED_BYTES,
        P170_NET_TOTAL_SENT_BYTES,
        P170_NET_TOTAL_TRANSIT_BYTES,
        P170_NET_BW_TRANSIT_15S,
        P170_NET_TUNNELS_SHARE_RATIO,
        P170_NET_TUNNELS_PARTICIPATING_INFO,
        P170_NET_TUNNELS_I2PTUNNEL,
        P170_NET_TUNNELS_EXPLORATORY_INBOUND,
        P170_NET_TUNNELS_EXPLORATORY_OUTBOUND,
        P170_NET_TUNNELS_EXPLORATORY_INFO_LIST,
        P170_NET_TUNNELS_CLIENT_INBOUND,
        P170_NET_TUNNELS_CLIENT_OUTBOUND,
        P170_NET_TUNNELS_CLIENT_INFO_LIST,
        P170_NET_STATUS_V6,
        P170_NET_ERROR,
        P170_NET_ERROR_V6,
        P170_NET_TESTING,
        P170_NET_TESTING_V6,
        P170_NET_TUNNELS_SUCCESS_RATE,
        P170_NET_TUNNELS_TOTAL_SUCCESS_RATE,
        P170_NET_TUNNELS_QUEUE,
        P170_NET_TUNNELS_TBM_QUEUE,
        P170_NETDB_PEERS,
        P170_NETDB_ACTIVE_PEERS_INFO,
        P170_NETDB_NTCP_LIMIT,
        P170_NETDB_SSU_LIMIT,
        P170_NETDB_BANNED_PEERS,
        P170_NETDB_ACTIVE_PEERS_LIST,
        P170_NETDB_PEERS_LIST,
        P170_NETDB_PEERS_INFO,
        P170_NETDB_ACTIVE_PEERS_STATS,
        P170_ADDRESS_BOOK_PRIVATE_LIST,
        P170_ADDRESS_BOOK_LOCAL_LIST,
        P170_ADDRESS_BOOK_ROUTER_LIST,
        P170_ADDRESS_BOOK_PUBLISHED_LIST,
    ];

    /// All non-address-book selector keys (owned by M005).
    pub const CORE_KEYS: &[&str] = &[
        UDP_ACTIVE,
        UDP_COOKIE_ACTIVE,
        UDP_INTEGRATED_PEERS,
        UDP_FIREWALLED,
        UDP_HIDDEN,
        UDP_COINFICIENT_PEERS,
        UDP_CRITICAL_PEERS,
        UDP_FAST_PEERS,
        UDP_HIGH_CAPACITY_PEERS,
        UDP_INTERLEAVED_PEERS,
        UDP_LIT_PEERS,
        UDP_LOW_CAPACITY_PEERS,
        UDP_ON_DEMAND_PEERS,
        UDP_PEER_STATS,
        UDP_STANDARD_PEERS,
        UDP_UNREACHABLE_PEERS,
        UDP_TOTAL_PEERS,
        UDP_CURRENT_PEERS,
        VERSION,
        UPTIME,
        NETDB_ACTIVE,
        NETDB_ACTIVE_PROFILES,
        NETDB_HIGHEST_VERSION,
        NETDB_KNOWN_PROFILES,
        NETDB_NEW_PROFILES,
        NETDB_ACTIVE_ROUTERS,
        NETDB_ALREADY_EXPERIENCED_PEERS,
        NETDB_BANLIST_SIZE,
        NETDB_EXPLORATORY_PEERS,
        NETDB_FAST_PEERS,
        NETDB_HIGH_CAPACITY_PEERS,
        NETDB_IS_BACKLOGGED,
        NETDB_KNOWN_ACTIVE,
        NETDB_KNOWN_IDLE,
        NETDB_KNOWN_USED,
        NETDB_KNOWN_VANILLA,
        NETDB_KNOWN_VOLATILE,
        NETDB_LAST_EXPLORED,
        NETDB_LAST_PROFILE_LOOKUP,
        NETDB_LAST_ROUTER_LOOKUP,
        NETDB_LAST_UNSAVED,
        NETDB_LEASE_SETS,
        NETDB_NEW_ACTIVE,
        NETDB_NEW_IDLE,
        NETDB_OLD_ACTIVE,
        NETDB_OLD_IDLE,
        NETDB_PEER_PROFILES,
        NETDB_PLAINTEXT_PEERS,
        NETDB_RESERVE_ACTIVE,
        NETDB_RESERVE_ACTIVE_PEERS,
        NETDB_RESERVE_HIGH_CAPACITY,
        NETDB_RESERVE_INTEGRATED,
        NETDB_RESERVE_KNOWN,
        NETDB_RESERVE_LOOKUP,
        NETDB_RESERVE_PENDING,
        NETDB_RESERVE_RESERVED,
        NETDB_RESERVE_STANDARD,
        NETDB_RESERVE_TIER2,
        NETDB_RESERVE_USED,
        NETDB_RESERVE_VOLATILE,
        NETDB_STANDARD_PEERS,
        NETDB_LOW_CAPACITY_PEERS,
        NETDB_TUNNELS,
        NETDB_USED_PEERS,
        NETDB_VOLATILE_PEERS,
        NETDB_ADDRESS_BOOKS,
        NETDB_ADDRESS_BOOK_ENTRIES,
        NETDB_ADDRESS_BOOK_SOURCES,
        NETDB_ADDRESS_BOOK_SUBSCRIPTIONS,
        NETDB_ADDRESS_BOOK_UPDATES,
        BW_INBOUND_1S,
        BW_INBOUND_15S,
        BW_INBOUND_1M,
        BW_INBOUND_1H,
        BW_INBOUND_1D,
        BW_INBOUND_TOTAL,
        BW_OUTBOUND_1S,
        BW_OUTBOUND_15S,
        BW_OUTBOUND_1M,
        BW_OUTBOUND_1H,
        BW_OUTBOUND_1D,
        BW_OUTBOUND_TOTAL,
        TCP_ACTIVE,
        TCP_INTEGRATED_PEERS,
        TCP_FIREWALLED,
        TCP_HOSTS,
        TCP_STATUS,
        TCP_VERSION,
        IDENTITY,
        NET_BW_INBOUND,
        NET_BW_OUTBOUND,
        ROUTER_NEWS,
        CLOCK_SKEW,
        SHARE_RATIO,
        CONFIGURED_BW_INBOUND,
        CONFIGURED_BW_OUTBOUND,
        TUNNELS_PARTICIPATING,
        TUNNELS_EXPLORATORY_IN,
        TUNNELS_EXPLORATORY_OUT,
        TUNNELS_CLIENT_IN,
        TUNNELS_CLIENT_OUT,
        TUNNELS_CONFIGURED,
        TUNNELS_QUEUE,
        PEERS_KNOWN_COUNT,
        PEERS_KNOWN,
        PEERS_ACTIVE_COUNT,
        PEERS_ACTIVE,
        PEERS_ROUTER_INFO,
        PEERS_BANNED,
        PEERS_BANNED_COUNT,
        PEERS_LIMITS,
        PEERS_ACTIVE_STATS,
        NET_IPTUNNELS,
        LOG_SNAPSHOT,
        LOG_CLEAR,
    ];
}

/// Test if a string is a valid tunnel type.
#[allow(dead_code)]
pub fn is_valid_tunnel_type(s: &str) -> bool {
    tunnel_types::ALL.contains(&s)
}

/// Test if a string is a valid address book name.
#[allow(dead_code)]
pub fn is_valid_address_book(s: &str) -> bool {
    address_books::ALL.contains(&s)
}

/// Test if a string is a valid Proposal 170 RouterInfo selector key.
#[allow(dead_code)]
pub fn is_valid_router_info_selector(s: &str) -> bool {
    router_info_keys::ALL.contains(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_request() {
        let body = r#"{"jsonrpc":"2.0","method":"Authenticate","params":{"API":1,"Password":"secret"},"id":1}"#;
        let req = parse_request(body).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "Authenticate");
        assert_eq!(req.id, Some(RequestId::Number(1)));
        assert!(req.params.is_some());
    }

    #[test]
    fn parse_request_missing_jsonrpc() {
        let body = r#"{"method":"Authenticate","id":1}"#;
        let err = parse_request(body).unwrap_err();
        assert_eq!(err.error.code, error_codes::INVALID_REQUEST);
    }

    #[test]
    fn parse_request_wrong_version() {
        let body = r#"{"jsonrpc":"1.0","method":"Authenticate","id":1}"#;
        let err = parse_request(body).unwrap_err();
        assert_eq!(err.error.code, error_codes::INVALID_REQUEST);
    }

    #[test]
    fn parse_request_missing_method() {
        let body = r#"{"jsonrpc":"2.0","id":1}"#;
        let err = parse_request(body).unwrap_err();
        assert_eq!(err.error.code, error_codes::INVALID_REQUEST);
    }

    #[test]
    fn parse_request_invalid_json() {
        let body = "not json";
        let err = parse_request(body).unwrap_err();
        assert_eq!(err.error.code, error_codes::PARSE_ERROR);
    }

    #[test]
    fn parse_request_positional_params() {
        let body = r#"{"jsonrpc":"2.0","method":"Authenticate","params":["a","b"],"id":1}"#;
        let err = parse_request(body).unwrap_err();
        assert_eq!(err.error.code, error_codes::INVALID_PARAMS);
    }

    #[test]
    fn parse_request_notification() {
        let body = r#"{"jsonrpc":"2.0","method":"Authenticate"}"#;
        let req = parse_request(body).unwrap();
        assert!(req.id.is_none());
    }

    #[test]
    fn parse_request_string_id() {
        let body = r#"{"jsonrpc":"2.0","method":"Authenticate","id":"abc"}"#;
        let req = parse_request(body).unwrap();
        assert_eq!(req.id, Some(RequestId::String("abc".to_string())));
    }

    #[test]
    fn parse_request_ids_preserve_null_and_notification_status() {
        let explicit_null =
            parse_request(r#"{"jsonrpc":"2.0","method":"Authenticate","id":null}"#).unwrap();
        assert_eq!(explicit_null.id, Some(RequestId::Null));
        assert!(!explicit_null.is_notification());

        let notification = parse_request(r#"{"jsonrpc":"2.0","method":"Authenticate"}"#).unwrap();
        assert!(notification.id.is_none());
        assert!(notification.is_notification());
    }

    #[test]
    fn parse_request_rejects_invalid_ids_without_coercion() {
        for id in ["true", "1.5", "{}", "[]", "9223372036854775808"] {
            let body = format!(r#"{{"jsonrpc":"2.0","method":"Authenticate","id":{id}}}"#);
            let err = parse_request(&body).unwrap_err();
            assert_eq!(err.error.code, error_codes::INVALID_REQUEST);
            assert_eq!(err.id, RequestId::Null);
        }
    }

    #[test]
    fn parse_request_accepts_integral_ids_at_i64_bounds() {
        for (literal, expected) in [
            ("-9223372036854775808", i64::MIN),
            ("9223372036854775807", i64::MAX),
        ] {
            let body = format!(r#"{{"jsonrpc":"2.0","method":"Authenticate","id":{literal}}}"#);
            let request = parse_request(&body).unwrap();
            assert_eq!(request.id, Some(RequestId::Number(expected)));
        }
    }

    #[test]
    fn tunnel_types_complete() {
        assert_eq!(tunnel_types::ALL.len(), 12);
        for tt in tunnel_types::ALL {
            assert!(is_valid_tunnel_type(tt));
        }
        assert!(!is_valid_tunnel_type("unknown"));
    }

    #[test]
    fn address_books_complete() {
        assert_eq!(address_books::ALL.len(), 4);
        for ab in address_books::ALL {
            assert!(is_valid_address_book(ab));
        }
        assert!(!is_valid_address_book("unknown"));
    }

    #[test]
    fn router_info_selectors_complete() {
        assert_eq!(router_info_keys::PROPOSAL_170_ADDITIONS.len(), 43);
        assert_eq!(router_info_keys::PROPOSAL_170_CONTRACT.len(), 43);
        let additions: std::collections::HashSet<&str> =
            router_info_keys::PROPOSAL_170_ADDITIONS.iter().copied().collect();
        assert_eq!(additions.len(), 43);
        assert_eq!(
            additions,
            router_info_keys::PROPOSAL_170_CONTRACT.iter().map(|field| field.key).collect()
        );
        for key in router_info_keys::ALL {
            assert!(is_valid_router_info_selector(key));
        }
        for key in router_info_keys::PROPOSAL_170_ADDITIONS {
            assert!(is_valid_router_info_selector(key));
        }
        assert!(!is_valid_router_info_selector("unknown.selector"));
    }

    #[test]
    fn conformance_manifest_has_exact_keys_types_and_metadata() {
        use std::collections::HashSet;

        let keys: HashSet<&str> =
            router_info_keys::PROPOSAL_170_ADDITIONS.iter().copied().collect();
        let fields: HashSet<&str> =
            router_info_keys::PROPOSAL_170_CONTRACT.iter().map(|field| field.key).collect();
        assert_eq!(keys, fields);
        assert_eq!(fields.len(), 43);
        for field in router_info_keys::PROPOSAL_170_CONTRACT {
            assert!(
                field.direct_presence,
                "{} is not direct-presence selected",
                field.key
            );
            assert!(
                !field.serializer.is_empty(),
                "{} has no serializer",
                field.key
            );
            assert!(!field.fixture.is_empty(), "{} has no fixture", field.key);
            assert!(
                !field.source.class().is_empty(),
                "{} has no source class",
                field.key
            );
            assert!(
                !field.source.owner().is_empty(),
                "{} has no source owner",
                field.key
            );
            if matches!(
                field.source,
                router_info_keys::SourceDisposition::Unavailable { .. }
            ) {
                assert!(
                    field.source.reason().is_some(),
                    "{} has no unavailable reason",
                    field.key
                );
            }
        }
    }

    #[test]
    fn conformance_manifest_has_m047_source_counts() {
        use router_info_keys::{SourceDisposition, PROPOSAL_170_CONTRACT as C};

        let available = C
            .iter()
            .filter(|field| matches!(field.source, SourceDisposition::Available { .. }))
            .count();
        let neutral = C
            .iter()
            .filter(|field| matches!(field.source, SourceDisposition::Neutral { .. }))
            .count();
        let unavailable = C
            .iter()
            .filter(|field| matches!(field.source, SourceDisposition::Unavailable { .. }))
            .count();

        assert_eq!((available, neutral, unavailable), (42, 1, 0));
    }

    #[test]
    fn conformance_manifest_has_literal_type_table() {
        use router_info_keys::{JsonType::*, PROPOSAL_170_CONTRACT as C};

        let expected = [
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
        assert_eq!(
            C.iter().map(|field| field.json_type).collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn base_canonical_and_compatibility_inventories_are_separate() {
        use std::collections::HashSet;

        let canonical: HashSet<&str> =
            router_info_keys::PROPOSAL_170_ADDITIONS.iter().copied().collect();
        let base: HashSet<&str> = router_info_keys::BASE_ROUTER_INFO_CORE_KEYS
            .iter()
            .chain(router_info_keys::BASE_ROUTER_INFO_ADDRESS_BOOK_KEYS.iter())
            .copied()
            .collect();
        let overlap: HashSet<&str> = canonical.intersection(&base).copied().collect();
        assert_eq!(
            overlap,
            HashSet::from([
                router_info_keys::ROUTER_NEWS,
                router_info_keys::ADDRESS_BOOK_SUBSCRIPTIONS,
                router_info_keys::ADDRESS_BOOK_CONFIG,
            ])
        );
        assert_eq!(
            router_info_keys::ROUTER_INFO_SELECTOR_OVERLAPS
                .iter()
                .map(|row| row.key)
                .collect::<HashSet<_>>(),
            overlap
        );
        for row in router_info_keys::ROUTER_INFO_SELECTOR_OVERLAPS {
            assert!(router_info_keys::is_base_router_info_selector(row.key));
            assert!(router_info_keys::is_proposal_170_addition(row.key));
            assert!(router_info_keys::BASE_ROUTER_INFO_SERIALIZERS
                .iter()
                .any(|(key, serializer)| *key == row.key && *serializer == row.base_serializer));
        }
        assert_eq!(
            router_info_keys::COMPATIBILITY_ROUTER_INFO_FORMS,
            &["Selector"]
        );
        assert_eq!(canonical.len(), 43);
    }

    #[test]
    fn overlap_table_contains_every_exact_name_intersection() {
        use std::collections::HashSet;

        let base: HashSet<&str> = router_info_keys::BASE_ROUTER_INFO_CORE_KEYS
            .iter()
            .chain(router_info_keys::BASE_ROUTER_INFO_ADDRESS_BOOK_KEYS.iter())
            .copied()
            .collect();
        let additions: HashSet<&str> =
            router_info_keys::PROPOSAL_170_ADDITIONS.iter().copied().collect();
        let table: HashSet<&str> = router_info_keys::ROUTER_INFO_SELECTOR_OVERLAPS
            .iter()
            .map(|row| row.key)
            .collect();

        assert_eq!(
            base.intersection(&additions).copied().collect::<HashSet<_>>(),
            table
        );
        assert_eq!(table.len(), 3);
        for row in router_info_keys::ROUTER_INFO_SELECTOR_OVERLAPS {
            assert_ne!(row.base_serializer, row.proposal_170_serializer);
        }
    }

    #[test]
    fn method_support_inventory_matches_dispatcher_surface() {
        use std::collections::HashSet;

        let inventory: HashSet<&str> =
            methods::SUPPORT_INVENTORY.iter().map(|entry| entry.method).collect();
        let protected: HashSet<&str> = methods::PROTECTED_DISPATCH.iter().copied().collect();
        let proposal: HashSet<&str> = methods::PROPOSAL_170.iter().copied().collect();
        let unsupported: HashSet<&str> = methods::UNSUPPORTED_BASE.iter().copied().collect();

        assert_eq!(inventory.len(), methods::SUPPORT_INVENTORY.len());
        assert!(protected.is_subset(&inventory));
        assert!(proposal.is_subset(&protected));
        assert!(unsupported.is_subset(&inventory));
        assert!(protected.is_disjoint(&unsupported));
        assert_eq!(
            methods::SUPPORT_INVENTORY
                .iter()
                .filter(|entry| entry.disposition == methods::SupportDisposition::UnsupportedBase)
                .map(|entry| entry.method)
                .collect::<HashSet<_>>(),
            unsupported
        );
    }

    #[test]
    fn router_info_core_keys_excludes_address_book() {
        for key in router_info_keys::CORE_KEYS {
            assert!(
                !router_info_keys::ADDRESS_BOOK_KEYS.contains(key),
                "CORE_KEYS should not contain address-book key: {key}"
            );
        }
    }

    #[test]
    fn router_info_all_keys_is_superset_of_core_and_address_book() {
        let all_set: std::collections::HashSet<&str> =
            router_info_keys::ALL.iter().copied().collect();
        for key in router_info_keys::CORE_KEYS {
            assert!(all_set.contains(key), "ALL missing CORE_KEY: {key}");
        }
        for key in router_info_keys::ADDRESS_BOOK_KEYS {
            assert!(all_set.contains(key), "ALL missing ADDRESS_BOOK_KEY: {key}");
        }
    }

    #[test]
    fn serialize_success_response() {
        let resp = JsonRpcSuccess::new(
            RequestId::Number(1),
            serde_json::json!({"Token": "abc", "API": 1}),
        );
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
    }

    #[test]
    fn serialize_error_response() {
        let resp = JsonRpcErrorResponse::new(
            RequestId::Number(1),
            error_codes::METHOD_NOT_FOUND,
            "Method not found",
        );
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601"));
    }
}
