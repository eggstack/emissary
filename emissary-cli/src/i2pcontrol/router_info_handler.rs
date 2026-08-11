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

//! Proposal 170 RouterInfo JSON-RPC method handler.
//!
//! Implements the `RouterInfo` method with exact selector-by-presence
//! behavior. Only requested selector keys appear in the response.

use std::collections::HashSet;

use crate::i2pcontrol::{
    address_book::{resolve_address_book_selectors_with_mode, RouterInfoAddressBookMode},
    router_info::{InspectionError, NetworkStatus, RouterInfoControl},
    rpc::{self, JsonRpcErrorResponse, JsonRpcRequest, JsonRpcSuccess, RequestId},
};

const LOG_TARGET: &str = "emissary::i2pcontrol::router_info_handler";

/// Maximum number of peer identities in a single response.
const MAX_PEER_IDENTITIES: usize = 10000;

/// Maximum total byte size of peer RouterInfo responses.
const MAX_PEER_RI_BYTES: usize = 4 * 1024 * 1024;

/// Maximum number of active peer stat entries.
const MAX_ACTIVE_PEER_STATS: usize = 10000;

/// Maximum number of log entries in a snapshot.
const MAX_LOG_ENTRIES: usize = 10000;

/// Maximum number of banned peers.
const MAX_BANNED_PEERS: usize = 10000;

/// Maximum number of entries exposed by the shared startup tunnel inventory.
const MAX_I2PTUNNEL_INFO_ENTRIES: usize = 1000;

/// Maximum number of rows in any live tunnel-detail response.
const MAX_TUNNEL_DETAIL_ENTRIES: usize = 10000;

const TUNNEL_DETAIL_KEYS: &[&str] = &[
    rpc::router_info_keys::P170_NET_TUNNELS_PARTICIPATING_INFO,
    rpc::router_info_keys::P170_NET_TUNNELS_EXPLORATORY_INBOUND,
    rpc::router_info_keys::P170_NET_TUNNELS_EXPLORATORY_OUTBOUND,
    rpc::router_info_keys::P170_NET_TUNNELS_EXPLORATORY_INFO_LIST,
    rpc::router_info_keys::P170_NET_TUNNELS_CLIENT_INBOUND,
    rpc::router_info_keys::P170_NET_TUNNELS_CLIENT_OUTBOUND,
    rpc::router_info_keys::P170_NET_TUNNELS_CLIENT_INFO_LIST,
    rpc::router_info_keys::P170_NET_TUNNELS_QUEUE,
    rpc::router_info_keys::P170_NET_TUNNELS_TBM_QUEUE,
];

/// Maximum serialized RouterInfo response size, including the JSON-RPC
/// envelope. The final response is checked after actual serialization.
const MAX_RESPONSE_BYTES: usize = 10 * 1024 * 1024;

/// Map the supported neutral reachability states to the pinned i2pd status
/// vocabulary. Emissary currently produces only OK, Firewalled, Unknown, and
/// Symmetric NAT; the latter has no distinct i2pd status code and remains an
/// unknown status until a canonical error owner exists.
fn network_status_code(status: NetworkStatus) -> i64 {
    match status {
        NetworkStatus::Ok => 0,
        NetworkStatus::Firewalled => 1,
        NetworkStatus::Unknown | NetworkStatus::SymmetricNat => 2,
        NetworkStatus::Hidden
        | NetworkStatus::Testing
        | NetworkStatus::Fail
        | NetworkStatus::FailTcp
        | NetworkStatus::FailUdp
        | NetworkStatus::FailNat => 2,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RouterInfoRequestMode {
    CanonicalDirect,
    CompatibilityNested,
}

/// Base UDP selectors that have no truthful source yet. Canonical additions
/// are adjudicated by `PROPOSAL_170_CONTRACT`.
/// If any of these are requested, the entire request fails with Unavailable.
const UDP_UNSUPPORTED: &[&str] = &[
    rpc::router_info_keys::UDP_COOKIE_ACTIVE,
    rpc::router_info_keys::UDP_INTEGRATED_PEERS,
    rpc::router_info_keys::UDP_HIDDEN,
    rpc::router_info_keys::UDP_COINFICIENT_PEERS,
    rpc::router_info_keys::UDP_CRITICAL_PEERS,
    rpc::router_info_keys::UDP_FAST_PEERS,
    rpc::router_info_keys::UDP_HIGH_CAPACITY_PEERS,
    rpc::router_info_keys::UDP_INTERLEAVED_PEERS,
    rpc::router_info_keys::UDP_LIT_PEERS,
    rpc::router_info_keys::UDP_LOW_CAPACITY_PEERS,
    rpc::router_info_keys::UDP_ON_DEMAND_PEERS,
    rpc::router_info_keys::UDP_PEER_STATS,
    rpc::router_info_keys::UDP_STANDARD_PEERS,
    rpc::router_info_keys::UDP_UNREACHABLE_PEERS,
    rpc::router_info_keys::UDP_TOTAL_PEERS,
    rpc::router_info_keys::UDP_CURRENT_PEERS,
];

/// Base NetDB selectors that have no truthful source yet.
/// If any of these are requested, the entire request fails with Unavailable.
const NETDB_UNSUPPORTED: &[&str] = &[
    rpc::router_info_keys::NETDB_ALREADY_EXPERIENCED_PEERS,
    rpc::router_info_keys::NETDB_IS_BACKLOGGED,
    rpc::router_info_keys::NETDB_LAST_EXPLORED,
    rpc::router_info_keys::NETDB_LAST_PROFILE_LOOKUP,
    rpc::router_info_keys::NETDB_LAST_ROUTER_LOOKUP,
    rpc::router_info_keys::NETDB_LAST_UNSAVED,
    rpc::router_info_keys::NETDB_NEW_ACTIVE,
    rpc::router_info_keys::NETDB_NEW_IDLE,
    rpc::router_info_keys::NETDB_OLD_ACTIVE,
    rpc::router_info_keys::NETDB_OLD_IDLE,
    rpc::router_info_keys::NETDB_RESERVE_ACTIVE,
    rpc::router_info_keys::NETDB_RESERVE_ACTIVE_PEERS,
    rpc::router_info_keys::NETDB_RESERVE_HIGH_CAPACITY,
    rpc::router_info_keys::NETDB_RESERVE_INTEGRATED,
    rpc::router_info_keys::NETDB_RESERVE_KNOWN,
    rpc::router_info_keys::NETDB_RESERVE_LOOKUP,
    rpc::router_info_keys::NETDB_RESERVE_PENDING,
    rpc::router_info_keys::NETDB_RESERVE_RESERVED,
    rpc::router_info_keys::NETDB_RESERVE_STANDARD,
    rpc::router_info_keys::NETDB_RESERVE_TIER2,
    rpc::router_info_keys::NETDB_RESERVE_USED,
    rpc::router_info_keys::NETDB_RESERVE_VOLATILE,
    rpc::router_info_keys::NETDB_PLAINTEXT_PEERS,
    rpc::router_info_keys::NETDB_TUNNELS,
    rpc::router_info_keys::NETDB_ADDRESS_BOOKS,
    rpc::router_info_keys::NETDB_ADDRESS_BOOK_ENTRIES,
    rpc::router_info_keys::NETDB_ADDRESS_BOOK_SOURCES,
    rpc::router_info_keys::NETDB_ADDRESS_BOOK_SUBSCRIPTIONS,
    rpc::router_info_keys::NETDB_ADDRESS_BOOK_UPDATES,
];

/// Base tunnel selectors that have no truthful source yet.
const TUNNEL_UNSUPPORTED: &[&str] = &[
    rpc::router_info_keys::TUNNELS_EXPLORATORY_IN,
    rpc::router_info_keys::TUNNELS_EXPLORATORY_OUT,
    rpc::router_info_keys::TUNNELS_CLIENT_IN,
    rpc::router_info_keys::TUNNELS_CLIENT_OUT,
    rpc::router_info_keys::TUNNELS_QUEUE,
];

/// Convert an `InspectionError` into a sanitized error message suitable
/// for the JSON-RPC error envelope. No internal paths, backtraces, or
/// secret material are included.
fn inspection_error_message(err: &InspectionError) -> String {
    match err {
        InspectionError::Unavailable { group } => {
            format!("{group} data unavailable")
        }
        InspectionError::UnavailableReason { group, reason } => {
            format!("{group} data unavailable: {reason}")
        }
        InspectionError::TemporarilyUnavailable { group } => {
            format!("{group} temporarily unavailable")
        }
        InspectionError::QueryFailed { group } => {
            format!("{group} query failed")
        }
        InspectionError::ResultTooLarge { group, limit } => {
            format!("{group} result exceeds bound of {limit} items")
        }
        InspectionError::InvalidPeerId => "invalid peer identifier".into(),
        InspectionError::InternalInvariant => "internal inspection error".into(),
    }
}

/// Map an `InspectionError` to a JSON-RPC error code.
fn inspection_error_code(err: &InspectionError) -> i32 {
    match err {
        InspectionError::ResultTooLarge { .. } => rpc::error_codes::INTERNAL_ERROR,
        InspectionError::InvalidPeerId => rpc::error_codes::INVALID_PARAMS,
        _ => rpc::error_codes::INTERNAL_ERROR,
    }
}

/// Estimate worst-case output bytes for the requested selector set.
///
/// Returns `Err` if the aggregate response would exceed safe bounds
/// before any expensive queries are issued.
fn estimate_response_budget(key_set: &HashSet<&str>) -> Result<(), String> {
    let mut estimated_bytes: usize = 0;

    // Identity (Base64 router info ~4KB)
    if key_set.contains(rpc::router_info_keys::IDENTITY) {
        estimated_bytes += 4096;
    }

    // Peer identity lists
    if key_set.contains(rpc::router_info_keys::PEERS_KNOWN)
        || key_set.contains(rpc::router_info_keys::PEERS_KNOWN_COUNT)
    {
        // Each peer ID ~52 bytes Base64, max 10000 peers
        estimated_bytes += MAX_PEER_IDENTITIES * 64;
    }
    if key_set.contains(rpc::router_info_keys::PEERS_ACTIVE)
        || key_set.contains(rpc::router_info_keys::PEERS_ACTIVE_COUNT)
    {
        estimated_bytes += MAX_PEER_IDENTITIES * 64;
    }

    // Peer RouterInfo (large payloads)
    if key_set.contains(rpc::router_info_keys::PEERS_ROUTER_INFO) {
        estimated_bytes += MAX_PEER_RI_BYTES;
    }

    // Active peer stats
    if key_set.contains(rpc::router_info_keys::PEERS_ACTIVE_STATS)
        || key_set.contains(rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_STATS)
    {
        estimated_bytes += MAX_ACTIVE_PEER_STATS * 128;
    }

    // Banned peers
    if key_set.contains(rpc::router_info_keys::PEERS_BANNED)
        || key_set.contains(rpc::router_info_keys::PEERS_BANNED_COUNT)
    {
        estimated_bytes += MAX_BANNED_PEERS * 128;
    }

    // Log snapshot
    if key_set.contains(rpc::router_info_keys::LOG_SNAPSHOT) {
        estimated_bytes += MAX_LOG_ENTRIES * 256;
    }

    // Proposal 170 tunnel pool detail lists contain only bounded primitive
    // rows. Account for every requested list before querying the shared source.
    let tunnel_detail_lists = TUNNEL_DETAIL_KEYS
        .iter()
        .filter(|key| {
            key_set.contains(**key)
                && **key != rpc::router_info_keys::P170_NET_TUNNELS_QUEUE
                && **key != rpc::router_info_keys::P170_NET_TUNNELS_TBM_QUEUE
        })
        .count();
    estimated_bytes += tunnel_detail_lists * MAX_TUNNEL_DETAIL_ENTRIES * 128;

    // Coarse pre-query response cap. Actual serialized output is checked after
    // assembly because source payload sizes are not predictable.
    if estimated_bytes > MAX_RESPONSE_BYTES {
        return Err(format!(
            "Estimated response size ({estimated_bytes} bytes) exceeds maximum ({MAX_RESPONSE_BYTES} bytes)"
        ));
    }

    Ok(())
}

/// Handle the RouterInfo JSON-RPC method.
///
/// Proposal 170 additions are selected by direct parameter presence. The
/// older nested `Selector` map remains available as a compatibility form and
/// is rejected when mixed with direct canonical parameters.
pub async fn handle_router_info(
    state: &crate::i2pcontrol::server::I2pControlState,
    request: &JsonRpcRequest,
) -> serde_json::Value {
    let id = resolve_id(&request.id);

    // Parse parameters
    let params = match &request.params {
        Some(params) => params,
        None => {
            return error_response(id, rpc::error_codes::INVALID_PARAMS, "Missing parameters");
        }
    };

    let has_nested_selector = params.contains_key("Selector");
    let has_direct_parameters = params.keys().any(|key| key != "Selector");
    if has_nested_selector && has_direct_parameters {
        return error_response(
            id,
            rpc::error_codes::INVALID_PARAMS,
            "Direct Proposal 170 selectors cannot be mixed with compatibility 'Selector'",
        );
    }

    let mode = if has_nested_selector {
        RouterInfoRequestMode::CompatibilityNested
    } else {
        RouterInfoRequestMode::CanonicalDirect
    };

    let mut requested_keys: Vec<&str> = Vec::new();
    let mut peer_ri_id: Option<&str> = None;
    if has_nested_selector {
        // Compatibility form: the historical nested selector map only
        // selects truthy boolean values (with its established peer lookup
        // string exception).
        let selector_map = match params.get("Selector") {
            Some(serde_json::Value::Object(map)) => map,
            _ => {
                return error_response(
                    id,
                    rpc::error_codes::INVALID_PARAMS,
                    "Invalid compatibility 'Selector'; expected a JSON object",
                );
            }
        };
        for (key, value) in selector_map {
            if key.as_str() == rpc::router_info_keys::PEERS_ROUTER_INFO {
                if let Some(id_str) = value.as_str() {
                    if !id_str.is_empty() {
                        peer_ri_id = Some(id_str);
                        if !rpc::router_info_keys::is_base_router_info_selector(key) {
                            return error_response(
                                id,
                                rpc::error_codes::INVALID_PARAMS,
                                format!("Unknown selector: '{key}'"),
                            );
                        }
                        requested_keys.push(key.as_str());
                    }
                }
            } else if value.as_bool() == Some(true) {
                if !rpc::router_info_keys::is_base_router_info_selector(key) {
                    return error_response(
                        id,
                        rpc::error_codes::INVALID_PARAMS,
                        format!("Unknown selector: '{key}'"),
                    );
                }
                requested_keys.push(key.as_str());
            }
        }
    } else {
        // Direct form: every supported base or Proposal 170 key is selected
        // by presence, and its value is intentionally ignored. The standard
        // Token metadata has already been removed by the dispatcher.
        for key in params.keys() {
            if !rpc::router_info_keys::is_direct_router_info_selector(key) {
                return error_response(
                    id,
                    rpc::error_codes::INVALID_PARAMS,
                    format!("Unknown direct RouterInfo parameter: '{key}'"),
                );
            }
            requested_keys.push(key.as_str());
        }
    }

    // Estimate response budget before expensive queries
    let key_set: HashSet<&str> = requested_keys.iter().copied().collect();

    if let Err(e) = estimate_response_budget(&key_set) {
        return error_response(id, rpc::error_codes::INTERNAL_ERROR, e);
    }

    // Dispatch and assemble response
    match assemble_response(state, &requested_keys, peer_ri_id, mode).await {
        Ok(result) => {
            let response = JsonRpcSuccess::new(id.clone(), serde_json::Value::Object(result));
            let serialized = match serde_json::to_vec(&response) {
                Ok(serialized) => serialized,
                Err(_) => {
                    return error_response(
                        id,
                        rpc::error_codes::INTERNAL_ERROR,
                        "RouterInfo response serialization failed",
                    )
                }
            };
            if serialized.len() > MAX_RESPONSE_BYTES {
                return error_response(
                    id,
                    rpc::error_codes::INTERNAL_ERROR,
                    format!(
                        "RouterInfo response exceeds serialized bound of {MAX_RESPONSE_BYTES} bytes"
                    ),
                );
            }
            serde_json::from_slice(&serialized).unwrap_or_else(|_| {
                error_response(
                    id,
                    rpc::error_codes::INTERNAL_ERROR,
                    "RouterInfo response serialization failed",
                )
            })
        }
        Err(e) => {
            let code = inspection_error_code(&e);
            let msg = inspection_error_message(&e);
            tracing::error!(
                target: LOG_TARGET,
                "RouterInfo assembly failed: {msg}"
            );
            error_response(id, code, msg)
        }
    }
}

/// Assemble the response object containing only requested keys.
///
/// Each snapshot group is queried at most once per request. A failure
/// in any non-nullable group aborts the entire request with no partial
/// result.
async fn assemble_response(
    state: &crate::i2pcontrol::server::I2pControlState,
    requested_keys: &[&str],
    peer_ri_id: Option<&str>,
    mode: RouterInfoRequestMode,
) -> Result<serde_json::Map<String, serde_json::Value>, InspectionError> {
    let router_info = state.router_info();
    let address_book = state.address_book_control();
    let mut result = serde_json::Map::new();

    if requested_keys.is_empty() {
        return Ok(result);
    }

    let key_set: HashSet<&str> = requested_keys.iter().copied().collect();

    // Only direct mode applies Proposal 170 availability/source rules. A
    // nested request is historical base compatibility and must not inherit a
    // direct addition's disposition merely because the spelling overlaps.
    if mode == RouterInfoRequestMode::CanonicalDirect {
        for key in requested_keys {
            if let Some(field) = rpc::router_info_keys::PROPOSAL_170_CONTRACT
                .iter()
                .find(|field| field.key == *key)
            {
                if !field.source.is_requestable() {
                    return Err(InspectionError::UnavailableReason {
                        group: canonical_group_for_key(key),
                        reason: field.source.reason().unwrap_or("source unavailable"),
                    });
                }
            }
        }
    }

    // Per-owner snapshots are acquired once for the duration of one request.
    // This prevents canonical/compatibility aliases from querying the same
    // source twice and keeps paired counters coherent enough for the request.
    let clock_skew_snapshot = if key_set.contains(rpc::router_info_keys::P170_CLOCKSKEW)
        || key_set.contains(rpc::router_info_keys::CLOCK_SKEW)
    {
        Some(router_info.clock_skew().await?)
    } else {
        None
    };
    let transport_bytes_snapshot = if key_set
        .contains(rpc::router_info_keys::P170_NET_TOTAL_RECEIVED_BYTES)
        || key_set.contains(rpc::router_info_keys::P170_NET_TOTAL_SENT_BYTES)
        || key_set.contains(rpc::router_info_keys::BW_INBOUND_TOTAL)
        || key_set.contains(rpc::router_info_keys::BW_OUTBOUND_TOTAL)
    {
        Some(router_info.transport_bytes().await?)
    } else {
        None
    };
    let share_ratio_snapshot = if key_set
        .contains(rpc::router_info_keys::P170_NET_TUNNELS_SHARE_RATIO)
        || key_set.contains(rpc::router_info_keys::SHARE_RATIO)
    {
        Some(router_info.share_ratio().await?)
    } else {
        None
    };
    let tunnel_build_stats_snapshot =
        if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_TOTAL_SUCCESS_RATE) {
            Some(router_info.tunnel_build_stats().await?)
        } else {
            None
        };
    let transit_bandwidth_15s_snapshot =
        if key_set.contains(rpc::router_info_keys::P170_NET_BW_TRANSIT_15S) {
            Some(router_info.transit_bandwidth_15s().await?)
        } else {
            None
        };
    let recent_tunnel_success_rate_snapshot =
        if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_SUCCESS_RATE) {
            Some(router_info.recent_tunnel_success_rate().await?)
        } else {
            None
        };
    let tunnel_details_snapshot = if key_set.iter().any(|key| TUNNEL_DETAIL_KEYS.contains(key)) {
        Some(router_info.tunnel_details().await?)
    } else {
        None
    };
    let network_snapshot = if key_set.contains(rpc::router_info_keys::NET_BW_INBOUND)
        || key_set.contains(rpc::router_info_keys::NET_BW_OUTBOUND)
        || key_set.contains(rpc::router_info_keys::P170_NET_STATUS_V6)
        || key_set.contains(rpc::router_info_keys::P170_NET_TESTING)
        || key_set.contains(rpc::router_info_keys::P170_NET_TESTING_V6)
    {
        Some(router_info.network_snapshot().await?)
    } else {
        None
    };

    // Exact Proposal 170 retained and metric fields.
    if key_set.contains(rpc::router_info_keys::P170_ID) {
        let id = state.router_id();
        result.insert(
            rpc::router_info_keys::P170_ID.to_string(),
            if id.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::json!(id)
            },
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_INFO) {
        let info = state.router_info_b64();
        result.insert(
            rpc::router_info_keys::P170_INFO.to_string(),
            if info.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::json!(info)
            },
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_CLOCKSKEW) {
        let skew = clock_skew_snapshot.as_ref().expect("clock skew was queried");
        result.insert(
            rpc::router_info_keys::P170_CLOCKSKEW.to_string(),
            skew.skew_seconds
                .map_or(serde_json::Value::Null, |value| serde_json::json!(value)),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TOTAL_RECEIVED_BYTES)
        || key_set.contains(rpc::router_info_keys::P170_NET_TOTAL_SENT_BYTES)
    {
        let bytes = transport_bytes_snapshot.as_ref().expect("transport bytes were queried");
        if key_set.contains(rpc::router_info_keys::P170_NET_TOTAL_RECEIVED_BYTES) {
            result.insert(
                rpc::router_info_keys::P170_NET_TOTAL_RECEIVED_BYTES.to_string(),
                serde_json::json!(bytes.received),
            );
        }
        if key_set.contains(rpc::router_info_keys::P170_NET_TOTAL_SENT_BYTES) {
            result.insert(
                rpc::router_info_keys::P170_NET_TOTAL_SENT_BYTES.to_string(),
                serde_json::json!(bytes.sent),
            );
        }
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TOTAL_TRANSIT_BYTES) {
        let bytes = router_info.transit_bytes().await?;
        result.insert(
            rpc::router_info_keys::P170_NET_TOTAL_TRANSIT_BYTES.to_string(),
            serde_json::json!(bytes.sent),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_BW_TRANSIT_15S) {
        result.insert(
            rpc::router_info_keys::P170_NET_BW_TRANSIT_15S.to_string(),
            serde_json::json!(transit_bandwidth_15s_snapshot.expect("transit 15s was queried")),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_SHARE_RATIO) {
        let ratio = share_ratio_snapshot.as_ref().expect("share ratio was queried");
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_SHARE_RATIO.to_string(),
            serde_json::json!(ratio),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_I2PTUNNEL) {
        let definitions = state.tunnel_list().await.map_err(|_| InspectionError::QueryFailed {
            group: crate::i2pcontrol::router_info::InspectionGroup::I2PTunnel,
        })?;
        if definitions.len() > MAX_I2PTUNNEL_INFO_ENTRIES {
            return Err(InspectionError::ResultTooLarge {
                group: crate::i2pcontrol::router_info::InspectionGroup::I2PTunnel,
                limit: MAX_I2PTUNNEL_INFO_ENTRIES,
            });
        }
        let infos: Vec<serde_json::Value> = definitions
            .iter()
            .map(crate::i2pcontrol::tunnel_manager::tunnel_definition_to_get_result)
            .collect();
        if serde_json::to_vec(&infos).map_or(true, |bytes| bytes.len() > 4 * 1024 * 1024) {
            return Err(InspectionError::ResultTooLarge {
                group: crate::i2pcontrol::router_info::InspectionGroup::I2PTunnel,
                limit: MAX_I2PTUNNEL_INFO_ENTRIES,
            });
        }
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_I2PTUNNEL.to_string(),
            serde_json::Value::Array(infos),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_TOTAL_SUCCESS_RATE) {
        let stats = tunnel_build_stats_snapshot.as_ref().expect("tunnel build stats were queried");
        let total = stats.successes.saturating_add(stats.failures);
        let rate: f64 = if total == 0 {
            0.0
        } else {
            (stats.successes as f64 / total as f64) * 100.0
        };
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_TOTAL_SUCCESS_RATE.to_string(),
            serde_json::json!(rate),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_SUCCESS_RATE) {
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_SUCCESS_RATE.to_string(),
            serde_json::json!(recent_tunnel_success_rate_snapshot
                .expect("recent tunnel success rate was queried")),
        );
    }

    // --- Proposal 170 live tunnel pools (one owner snapshot) ---
    if key_set.iter().any(|key| TUNNEL_DETAIL_KEYS.contains(key)) {
        let details = tunnel_details_snapshot.as_ref().expect("tunnel details were queried");
        resolve_proposal_tunnel_details(&mut result, &key_set, details)?;
        if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_QUEUE) {
            result.insert(
                rpc::router_info_keys::P170_NET_TUNNELS_QUEUE.to_string(),
                serde_json::json!(details.queue_depth),
            );
        }
        if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_TBM_QUEUE) {
            result.insert(
                rpc::router_info_keys::P170_NET_TUNNELS_TBM_QUEUE.to_string(),
                serde_json::json!(details.tbm_queue_depth),
            );
        }
    }

    // --- Identity and static router data (retained group) ---
    if key_set.contains(rpc::router_info_keys::IDENTITY) {
        // The old alias has the same semantic value as the canonical router
        // hash. Use the retained state when available so requesting both
        // spellings does not perform two source reads.
        let identity = if state.router_id().is_empty() {
            router_info.router_identity()?
        } else {
            state.router_id().to_string()
        };
        result.insert(
            rpc::router_info_keys::IDENTITY.to_string(),
            serde_json::json!(identity),
        );
    }

    if key_set.contains(rpc::router_info_keys::VERSION) {
        let version = router_info.router_version()?;
        result.insert(
            rpc::router_info_keys::VERSION.to_string(),
            serde_json::json!(version),
        );
    }

    if key_set.contains(rpc::router_info_keys::UPTIME) {
        let uptime = router_info.router_uptime_ms()?;
        result.insert(
            rpc::router_info_keys::UPTIME.to_string(),
            serde_json::json!(uptime),
        );
    }

    // --- Router news ---
    if key_set.contains(rpc::router_info_keys::ROUTER_NEWS) {
        let news = router_info.router_news()?;
        result.insert(
            rpc::router_info_keys::ROUTER_NEWS.to_string(),
            serde_json::json!(news),
        );
    }

    // --- Clock skew ---
    if key_set.contains(rpc::router_info_keys::CLOCK_SKEW) {
        let skew = clock_skew_snapshot.as_ref().expect("clock skew was queried");
        let value = match skew.skew_seconds {
            Some(s) => serde_json::json!(s),
            None => serde_json::json!(null),
        };
        result.insert(rpc::router_info_keys::CLOCK_SKEW.to_string(), value);
    }

    // --- Network status ---
    if let Some(network) = network_snapshot.as_ref() {
        if key_set.contains(rpc::router_info_keys::NET_BW_INBOUND) {
            result.insert(
                rpc::router_info_keys::NET_BW_INBOUND.to_string(),
                serde_json::json!(network.ipv4_status.as_str()),
            );
        }
        if key_set.contains(rpc::router_info_keys::NET_BW_OUTBOUND) {
            result.insert(
                rpc::router_info_keys::NET_BW_OUTBOUND.to_string(),
                serde_json::json!(network.ipv6_status.as_str()),
            );
        }
        if key_set.contains(rpc::router_info_keys::P170_NET_STATUS_V6) {
            result.insert(
                rpc::router_info_keys::P170_NET_STATUS_V6.to_string(),
                serde_json::json!(network_status_code(network.ipv6_status)),
            );
        }
        if key_set.contains(rpc::router_info_keys::P170_NET_TESTING) {
            result.insert(
                rpc::router_info_keys::P170_NET_TESTING.to_string(),
                serde_json::json!(if network.ipv4_testing { 1 } else { 0 }),
            );
        }
        if key_set.contains(rpc::router_info_keys::P170_NET_TESTING_V6) {
            result.insert(
                rpc::router_info_keys::P170_NET_TESTING_V6.to_string(),
                serde_json::json!(if network.ipv6_testing { 1 } else { 0 }),
            );
        }
    }

    // --- Share ratio and configured BW ---
    if key_set.contains(rpc::router_info_keys::SHARE_RATIO) {
        let ratio = share_ratio_snapshot.as_ref().expect("share ratio was queried");
        result.insert(
            rpc::router_info_keys::SHARE_RATIO.to_string(),
            serde_json::json!(ratio),
        );
    }
    if key_set.contains(rpc::router_info_keys::CONFIGURED_BW_INBOUND)
        || key_set.contains(rpc::router_info_keys::CONFIGURED_BW_OUTBOUND)
    {
        let (inbound, outbound) = router_info.configured_bw_limits().await?;
        if key_set.contains(rpc::router_info_keys::CONFIGURED_BW_INBOUND) {
            result.insert(
                rpc::router_info_keys::CONFIGURED_BW_INBOUND.to_string(),
                serde_json::json!(inbound),
            );
        }
        if key_set.contains(rpc::router_info_keys::CONFIGURED_BW_OUTBOUND) {
            result.insert(
                rpc::router_info_keys::CONFIGURED_BW_OUTBOUND.to_string(),
                serde_json::json!(outbound),
            );
        }
    }

    // --- UDP transport (one group query) ---
    if key_set.iter().any(|k| k.starts_with("i2p.router.udp.")) {
        if UDP_UNSUPPORTED.iter().any(|k| key_set.contains(k)) {
            return Err(InspectionError::Unavailable {
                group: crate::i2pcontrol::router_info::InspectionGroup::UdpTransport,
            });
        }
        let udp = router_info.udp_snapshot().await?;
        resolve_udp_selectors(&mut result, &key_set, &udp);
    }

    // --- TCP transport (one group query) ---
    if key_set.iter().any(|k| k.starts_with("i2p.router.tcp.")) {
        let tcp = router_info.tcp_snapshot().await?;
        resolve_tcp_selectors(&mut result, &key_set, &tcp);
    }

    // --- NetDB (one group query) ---
    if key_set
        .iter()
        .any(|k| k.starts_with("i2p.router.netdb.") && rpc::router_info_keys::CORE_KEYS.contains(k))
    {
        if NETDB_UNSUPPORTED.iter().any(|k| key_set.contains(k)) {
            return Err(InspectionError::Unavailable {
                group: crate::i2pcontrol::router_info::InspectionGroup::NetDb,
            });
        }
        let netdb = router_info.netdb_snapshot().await?;
        resolve_netdb_selectors(&mut result, &key_set, &netdb);
    }

    // --- Proposal 170 public peer directory ---
    if key_set.iter().any(|key| {
        matches!(
            *key,
            rpc::router_info_keys::P170_NETDB_PEERS
                | rpc::router_info_keys::P170_NETDB_PEERS_LIST
                | rpc::router_info_keys::P170_NETDB_PEERS_INFO
        )
    }) {
        resolve_proposal_peer_directory(&mut result, &key_set, router_info).await?;
    }

    // --- Proposal 170 active peers and finite transport limits ---
    if key_set.iter().any(|key| {
        matches!(
            *key,
            rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_LIST
                | rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_INFO
        )
    }) {
        resolve_proposal_active_peers(&mut result, &key_set, router_info).await?;
    }
    if key_set.contains(rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_STATS) {
        resolve_active_peer_stats(
            &mut result,
            rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_STATS,
            router_info,
        )
        .await?;
    }
    if key_set.iter().any(|key| {
        matches!(
            *key,
            rpc::router_info_keys::P170_NETDB_NTCP_LIMIT
                | rpc::router_info_keys::P170_NETDB_SSU_LIMIT
        )
    }) {
        let limits = router_info.transport_limits().await?;
        if key_set.contains(rpc::router_info_keys::P170_NETDB_NTCP_LIMIT) {
            let Some(limit) = limits.ntcp_limit else {
                return Err(InspectionError::UnavailableReason {
                    group: crate::i2pcontrol::router_info::InspectionGroup::PeerStats,
                    reason: "finite NTCP2 connection limit unavailable",
                });
            };
            result.insert(
                rpc::router_info_keys::P170_NETDB_NTCP_LIMIT.to_string(),
                serde_json::json!(limit),
            );
        }
        if key_set.contains(rpc::router_info_keys::P170_NETDB_SSU_LIMIT) {
            let Some(limit) = limits.ssu_limit else {
                return Err(InspectionError::UnavailableReason {
                    group: crate::i2pcontrol::router_info::InspectionGroup::PeerStats,
                    reason: "finite SSU2 connection limit unavailable",
                });
            };
            result.insert(
                rpc::router_info_keys::P170_NETDB_SSU_LIMIT.to_string(),
                serde_json::json!(limit),
            );
        }
    }

    // --- Bandwidth ---
    if key_set.iter().any(|k| k.starts_with("i2p.router.bw.")) {
        let router_info = state.router_info();
        let transport = if key_set.contains(rpc::router_info_keys::BW_INBOUND_TOTAL)
            || key_set.contains(rpc::router_info_keys::BW_OUTBOUND_TOTAL)
        {
            transport_bytes_snapshot.clone()
        } else {
            None
        };
        let recent = if key_set.iter().any(|key| {
            matches!(
                *key,
                rpc::router_info_keys::BW_INBOUND_1S
                    | rpc::router_info_keys::BW_INBOUND_15S
                    | rpc::router_info_keys::BW_INBOUND_1M
                    | rpc::router_info_keys::BW_INBOUND_1H
                    | rpc::router_info_keys::BW_INBOUND_1D
                    | rpc::router_info_keys::BW_OUTBOUND_1S
                    | rpc::router_info_keys::BW_OUTBOUND_15S
                    | rpc::router_info_keys::BW_OUTBOUND_1M
                    | rpc::router_info_keys::BW_OUTBOUND_1H
                    | rpc::router_info_keys::BW_OUTBOUND_1D
            )
        }) {
            Some(router_info.recent_transit_traffic().await?)
        } else {
            None
        };
        resolve_bw_selectors(&mut result, &key_set, transport.as_ref(), recent.as_ref());
    }

    // --- Tunnel selectors (one group query) ---
    if key_set.iter().any(|k| k.starts_with("i2p.router.tunnels.")) {
        if TUNNEL_UNSUPPORTED.iter().any(|k| key_set.contains(k)) {
            return Err(InspectionError::Unavailable {
                group: crate::i2pcontrol::router_info::InspectionGroup::TunnelSummary,
            });
        }
        let summary = router_info.tunnel_summary().await?;
        resolve_tunnel_selectors(&mut result, &key_set, &summary);
    }

    // --- I2PTunnel ---
    if key_set.contains(rpc::router_info_keys::NET_IPTUNNELS) {
        let stats = router_info.i2ptunnel_stats().await?;
        result.insert(
            rpc::router_info_keys::NET_IPTUNNELS.to_string(),
            serde_json::json!(stats.configured_count),
        );
    }

    // --- Peer selectors ---
    if key_set.iter().any(|k| k.starts_with("i2p.router.peers.")) {
        resolve_peer_selectors(&mut result, &key_set, router_info, peer_ri_id).await?;
    }

    // --- Log selectors. The canonical `logs` list is string-valued; the
    // legacy `log` alias retains its historical structured entry shape. ---
    if key_set.contains(rpc::router_info_keys::LOG_SNAPSHOT)
        || key_set.contains(rpc::router_info_keys::P170_LOGS)
    {
        let snap = router_info.log_snapshot().await?;
        if snap.entries.len() > MAX_LOG_ENTRIES {
            return Err(InspectionError::ResultTooLarge {
                group: crate::i2pcontrol::router_info::InspectionGroup::Log,
                limit: MAX_LOG_ENTRIES,
            });
        }
        let entries: Vec<serde_json::Value> = snap
            .entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "timestamp": e.timestamp_ms,
                    "level": e.level,
                    "target": e.target,
                    "message": e.message,
                })
            })
            .collect();
        if key_set.contains(rpc::router_info_keys::LOG_SNAPSHOT) {
            result.insert(
                rpc::router_info_keys::LOG_SNAPSHOT.to_string(),
                serde_json::json!(entries),
            );
        }
        if key_set.contains(rpc::router_info_keys::P170_LOGS) {
            let messages: Vec<&str> =
                snap.entries.iter().map(|entry| entry.message.as_str()).collect();
            result.insert(
                rpc::router_info_keys::P170_LOGS.to_string(),
                serde_json::json!(messages),
            );
        }
    }
    if key_set.contains(rpc::router_info_keys::LOG_CLEAR)
        || key_set.contains(rpc::router_info_keys::P170_LOGS_CLEAR)
    {
        router_info.log_clear().await?;
        if key_set.contains(rpc::router_info_keys::LOG_CLEAR) {
            result.insert(
                rpc::router_info_keys::LOG_CLEAR.to_string(),
                serde_json::json!(true),
            );
        }
        if key_set.contains(rpc::router_info_keys::P170_LOGS_CLEAR) {
            result.insert(
                rpc::router_info_keys::P170_LOGS_CLEAR.to_string(),
                serde_json::json!("success"),
            );
        }
    }

    // --- Address-book selectors ---
    let address_book_keys: Vec<&str> = key_set
        .iter()
        .copied()
        .filter(|k| {
            rpc::router_info_keys::ADDRESS_BOOK_KEYS.contains(k)
                || matches!(
                    *k,
                    rpc::router_info_keys::P170_ADDRESS_BOOK_PRIVATE_LIST
                        | rpc::router_info_keys::P170_ADDRESS_BOOK_LOCAL_LIST
                        | rpc::router_info_keys::P170_ADDRESS_BOOK_ROUTER_LIST
                        | rpc::router_info_keys::P170_ADDRESS_BOOK_PUBLISHED_LIST
                )
        })
        .collect();
    if !address_book_keys.is_empty() {
        let address_book_mode = match mode {
            RouterInfoRequestMode::CanonicalDirect => RouterInfoAddressBookMode::CanonicalDirect,
            RouterInfoRequestMode::CompatibilityNested => {
                RouterInfoAddressBookMode::CompatibilityNested
            }
        };
        let ab_result = resolve_address_book_selectors_with_mode(
            address_book,
            &address_book_keys,
            address_book_mode,
        )
        .await
        .map_err(|_| InspectionError::QueryFailed {
            group: crate::i2pcontrol::router_info::InspectionGroup::AddressBook,
        })?;
        for (k, v) in ab_result {
            result.insert(k, v);
        }
    }

    Ok(result)
}

fn canonical_group_for_key(key: &str) -> crate::i2pcontrol::router_info::InspectionGroup {
    if key.starts_with("i2p.router.netdb.") {
        crate::i2pcontrol::router_info::InspectionGroup::NetDb
    } else if key.starts_with("i2p.router.addressbook.") {
        crate::i2pcontrol::router_info::InspectionGroup::AddressBook
    } else if key.starts_with("i2p.router.net.tunnels.") {
        crate::i2pcontrol::router_info::InspectionGroup::TunnelSummary
    } else if key == rpc::router_info_keys::P170_NET_BW_TRANSIT_15S
        || key == rpc::router_info_keys::P170_NET_TUNNELS_SUCCESS_RATE
    {
        crate::i2pcontrol::router_info::InspectionGroup::TrafficMetrics
    } else {
        crate::i2pcontrol::router_info::InspectionGroup::Network
    }
}

/// Resolve UDP transport selectors into response entries.
fn resolve_udp_selectors(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key_set: &HashSet<&str>,
    udp: &crate::i2pcontrol::router_info::UdpSnapshot,
) {
    if key_set.contains(rpc::router_info_keys::UDP_ACTIVE) {
        result.insert(
            rpc::router_info_keys::UDP_ACTIVE.to_string(),
            serde_json::json!(udp.active),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_COOKIE_ACTIVE) {
        result.insert(
            rpc::router_info_keys::UDP_COOKIE_ACTIVE.to_string(),
            serde_json::json!(udp.cookie_active),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_INTEGRATED_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_INTEGRATED_PEERS.to_string(),
            serde_json::json!(udp.integrated_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_FIREWALLED) {
        result.insert(
            rpc::router_info_keys::UDP_FIREWALLED.to_string(),
            serde_json::json!(udp.firewalled),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_HIDDEN) {
        result.insert(
            rpc::router_info_keys::UDP_HIDDEN.to_string(),
            serde_json::json!(udp.hidden),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_COINFICIENT_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_COINFICIENT_PEERS.to_string(),
            serde_json::json!(udp.coinficient_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_CRITICAL_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_CRITICAL_PEERS.to_string(),
            serde_json::json!(udp.critical_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_FAST_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_FAST_PEERS.to_string(),
            serde_json::json!(udp.fast_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_HIGH_CAPACITY_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_HIGH_CAPACITY_PEERS.to_string(),
            serde_json::json!(udp.high_capacity_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_INTERLEAVED_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_INTERLEAVED_PEERS.to_string(),
            serde_json::json!(udp.interleaved_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_LIT_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_LIT_PEERS.to_string(),
            serde_json::json!(udp.lit_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_LOW_CAPACITY_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_LOW_CAPACITY_PEERS.to_string(),
            serde_json::json!(udp.low_capacity_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_ON_DEMAND_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_ON_DEMAND_PEERS.to_string(),
            serde_json::json!(udp.on_demand_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_STANDARD_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_STANDARD_PEERS.to_string(),
            serde_json::json!(udp.standard_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_UNREACHABLE_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_UNREACHABLE_PEERS.to_string(),
            serde_json::json!(udp.unreachable_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_TOTAL_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_TOTAL_PEERS.to_string(),
            serde_json::json!(udp.total_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::UDP_CURRENT_PEERS) {
        result.insert(
            rpc::router_info_keys::UDP_CURRENT_PEERS.to_string(),
            serde_json::json!(udp.current_peers),
        );
    }
}

/// Resolve TCP transport selectors into response entries.
fn resolve_tcp_selectors(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key_set: &HashSet<&str>,
    tcp: &crate::i2pcontrol::router_info::TcpSnapshot,
) {
    if key_set.contains(rpc::router_info_keys::TCP_ACTIVE) {
        result.insert(
            rpc::router_info_keys::TCP_ACTIVE.to_string(),
            serde_json::json!(tcp.active),
        );
    }
    if key_set.contains(rpc::router_info_keys::TCP_INTEGRATED_PEERS) {
        result.insert(
            rpc::router_info_keys::TCP_INTEGRATED_PEERS.to_string(),
            serde_json::json!(tcp.integrated_peers),
        );
    }
    if key_set.contains(rpc::router_info_keys::TCP_FIREWALLED) {
        result.insert(
            rpc::router_info_keys::TCP_FIREWALLED.to_string(),
            serde_json::json!(tcp.firewalled),
        );
    }
    if key_set.contains(rpc::router_info_keys::TCP_HOSTS) {
        result.insert(
            rpc::router_info_keys::TCP_HOSTS.to_string(),
            serde_json::json!(tcp.hosts),
        );
    }
    if key_set.contains(rpc::router_info_keys::TCP_STATUS) {
        result.insert(
            rpc::router_info_keys::TCP_STATUS.to_string(),
            serde_json::json!(tcp.status),
        );
    }
    if key_set.contains(rpc::router_info_keys::TCP_VERSION) {
        result.insert(
            rpc::router_info_keys::TCP_VERSION.to_string(),
            serde_json::json!(tcp.version),
        );
    }
}

/// Resolve NetDB selectors into response entries.
fn resolve_netdb_selectors(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key_set: &HashSet<&str>,
    netdb: &crate::i2pcontrol::router_info::NetDbSnapshot,
) {
    type NetDbMapping = (
        &'static str,
        fn(&crate::i2pcontrol::router_info::NetDbSnapshot) -> serde_json::Value,
    );
    let mappings: &[NetDbMapping] = &[
        (rpc::router_info_keys::NETDB_ACTIVE, |n| {
            serde_json::json!(n.active)
        }),
        (rpc::router_info_keys::NETDB_ACTIVE_PROFILES, |n| {
            serde_json::json!(n.active_profiles)
        }),
        (rpc::router_info_keys::NETDB_HIGHEST_VERSION, |n| {
            serde_json::json!(n.highest_version)
        }),
        (rpc::router_info_keys::NETDB_KNOWN_PROFILES, |n| {
            serde_json::json!(n.known_profiles)
        }),
        (rpc::router_info_keys::NETDB_NEW_PROFILES, |n| {
            serde_json::json!(n.new_profiles)
        }),
        (rpc::router_info_keys::NETDB_ACTIVE_ROUTERS, |n| {
            serde_json::json!(n.active_routers)
        }),
        (rpc::router_info_keys::NETDB_BANLIST_SIZE, |n| {
            serde_json::json!(n.banlist_size)
        }),
        (rpc::router_info_keys::NETDB_LEASE_SETS, |n| {
            serde_json::json!(n.lease_sets)
        }),
        (rpc::router_info_keys::NETDB_EXPLORATORY_PEERS, |n| {
            serde_json::json!(n.exploratory_peers)
        }),
        (rpc::router_info_keys::NETDB_FAST_PEERS, |n| {
            serde_json::json!(n.fast_peers)
        }),
        (rpc::router_info_keys::NETDB_HIGH_CAPACITY_PEERS, |n| {
            serde_json::json!(n.high_capacity_peers)
        }),
        (rpc::router_info_keys::NETDB_STANDARD_PEERS, |n| {
            serde_json::json!(n.standard_peers)
        }),
        (rpc::router_info_keys::NETDB_LOW_CAPACITY_PEERS, |n| {
            serde_json::json!(n.low_capacity_peers)
        }),
        (rpc::router_info_keys::NETDB_KNOWN_ACTIVE, |n| {
            serde_json::json!(
                n.active_fast_profiles
                    + n.active_high_capacity_profiles
                    + n.active_standard_profiles
                    + n.active_low_capacity_profiles
            )
        }),
        (rpc::router_info_keys::NETDB_KNOWN_IDLE, |n| {
            serde_json::json!(
                n.idle_fast_profiles
                    + n.idle_high_capacity_profiles
                    + n.idle_standard_profiles
                    + n.idle_low_capacity_profiles
            )
        }),
        (rpc::router_info_keys::NETDB_KNOWN_USED, |n| {
            serde_json::json!(n.used_peers)
        }),
        (rpc::router_info_keys::NETDB_KNOWN_VANILLA, |n| {
            serde_json::json!(n.total_reject_profiles)
        }),
        (rpc::router_info_keys::NETDB_KNOWN_VOLATILE, |n| {
            serde_json::json!(n.volatile_peers)
        }),
        (rpc::router_info_keys::NETDB_USED_PEERS, |n| {
            serde_json::json!(n.used_peers)
        }),
        (rpc::router_info_keys::NETDB_VOLATILE_PEERS, |n| {
            serde_json::json!(n.volatile_peers)
        }),
        (rpc::router_info_keys::NETDB_PEER_PROFILES, |n| {
            serde_json::json!(n.total_reject_profiles)
        }),
    ];

    for (key, extractor) in mappings {
        if key_set.contains(key) {
            result.insert(key.to_string(), extractor(netdb));
        }
    }
}

/// Resolve bandwidth selectors into response entries.
fn resolve_bw_selectors(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key_set: &HashSet<&str>,
    transport: Option<&crate::i2pcontrol::router_info::TransportBytes>,
    recent: Option<&crate::i2pcontrol::router_info::RecentTransitTraffic>,
) {
    // Cumulative total
    if key_set.contains(rpc::router_info_keys::BW_INBOUND_TOTAL) {
        let transport = transport.expect("transport bytes queried for requested total");
        result.insert(
            rpc::router_info_keys::BW_INBOUND_TOTAL.to_string(),
            serde_json::json!(transport.received),
        );
    }
    if key_set.contains(rpc::router_info_keys::BW_OUTBOUND_TOTAL) {
        let transport = transport.expect("transport bytes queried for requested total");
        result.insert(
            rpc::router_info_keys::BW_OUTBOUND_TOTAL.to_string(),
            serde_json::json!(transport.sent),
        );
    }

    // Rolling 1-second
    if key_set.contains(rpc::router_info_keys::BW_INBOUND_1S) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_INBOUND_1S.to_string(),
            serde_json::json!(recent.inbound_1s),
        );
    }
    if key_set.contains(rpc::router_info_keys::BW_OUTBOUND_1S) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_OUTBOUND_1S.to_string(),
            serde_json::json!(recent.outbound_1s),
        );
    }

    // Rolling 15-second
    if key_set.contains(rpc::router_info_keys::BW_INBOUND_15S) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_INBOUND_15S.to_string(),
            serde_json::json!(recent.inbound_15s),
        );
    }
    if key_set.contains(rpc::router_info_keys::BW_OUTBOUND_15S) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_OUTBOUND_15S.to_string(),
            serde_json::json!(recent.outbound_15s),
        );
    }

    // Rolling 1-minute
    if key_set.contains(rpc::router_info_keys::BW_INBOUND_1M) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_INBOUND_1M.to_string(),
            serde_json::json!(recent.inbound_1m),
        );
    }
    if key_set.contains(rpc::router_info_keys::BW_OUTBOUND_1M) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_OUTBOUND_1M.to_string(),
            serde_json::json!(recent.outbound_1m),
        );
    }

    // Rolling 1-hour
    if key_set.contains(rpc::router_info_keys::BW_INBOUND_1H) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_INBOUND_1H.to_string(),
            serde_json::json!(recent.inbound_1h),
        );
    }
    if key_set.contains(rpc::router_info_keys::BW_OUTBOUND_1H) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_OUTBOUND_1H.to_string(),
            serde_json::json!(recent.outbound_1h),
        );
    }

    // Rolling 1-day
    if key_set.contains(rpc::router_info_keys::BW_INBOUND_1D) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_INBOUND_1D.to_string(),
            serde_json::json!(recent.inbound_1d),
        );
    }
    if key_set.contains(rpc::router_info_keys::BW_OUTBOUND_1D) {
        let recent = recent.expect("recent traffic queried for requested interval");
        result.insert(
            rpc::router_info_keys::BW_OUTBOUND_1D.to_string(),
            serde_json::json!(recent.outbound_1d),
        );
    }
}

/// Resolve tunnel selectors into response entries.
fn resolve_tunnel_selectors(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key_set: &HashSet<&str>,
    summary: &crate::i2pcontrol::router_info::TunnelSummary,
) {
    if key_set.contains(rpc::router_info_keys::TUNNELS_PARTICIPATING) {
        result.insert(
            rpc::router_info_keys::TUNNELS_PARTICIPATING.to_string(),
            serde_json::json!(summary.active_participating),
        );
    }
    if key_set.contains(rpc::router_info_keys::TUNNELS_EXPLORATORY_IN) {
        result.insert(
            rpc::router_info_keys::TUNNELS_EXPLORATORY_IN.to_string(),
            serde_json::json!(summary.exploratory_inbound),
        );
    }
    if key_set.contains(rpc::router_info_keys::TUNNELS_EXPLORATORY_OUT) {
        result.insert(
            rpc::router_info_keys::TUNNELS_EXPLORATORY_OUT.to_string(),
            serde_json::json!(summary.exploratory_outbound),
        );
    }
    if key_set.contains(rpc::router_info_keys::TUNNELS_CLIENT_IN) {
        result.insert(
            rpc::router_info_keys::TUNNELS_CLIENT_IN.to_string(),
            serde_json::json!(summary.client_inbound),
        );
    }
    if key_set.contains(rpc::router_info_keys::TUNNELS_CLIENT_OUT) {
        result.insert(
            rpc::router_info_keys::TUNNELS_CLIENT_OUT.to_string(),
            serde_json::json!(summary.client_outbound),
        );
    }
    if key_set.contains(rpc::router_info_keys::TUNNELS_CONFIGURED) {
        result.insert(
            rpc::router_info_keys::TUNNELS_CONFIGURED.to_string(),
            serde_json::json!(summary.configured),
        );
    }
    if key_set.contains(rpc::router_info_keys::TUNNELS_QUEUE) {
        result.insert(
            rpc::router_info_keys::TUNNELS_QUEUE.to_string(),
            serde_json::json!(summary.queue_depth),
        );
    }
}

/// Resolve peer selectors into response entries.
///
/// Enforces per-selector item bounds from `MAX_*` constants.
/// Each peer list source (known, active, banned) is queried at most once
/// per request for both count and list selectors.
async fn resolve_peer_selectors(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key_set: &HashSet<&str>,
    router_info: &dyn RouterInfoControl,
    peer_ri_id: Option<&str>,
) -> Result<(), InspectionError> {
    // Known peers: query once, use for both count and list
    let needs_known = key_set.contains(rpc::router_info_keys::PEERS_KNOWN_COUNT)
        || key_set.contains(rpc::router_info_keys::PEERS_KNOWN);
    if needs_known {
        let peers = router_info.known_peers().await?;
        if key_set.contains(rpc::router_info_keys::PEERS_KNOWN_COUNT) {
            result.insert(
                rpc::router_info_keys::PEERS_KNOWN_COUNT.to_string(),
                serde_json::json!(peers.len()),
            );
        }
        if key_set.contains(rpc::router_info_keys::PEERS_KNOWN) {
            if peers.len() > MAX_PEER_IDENTITIES {
                return Err(InspectionError::ResultTooLarge {
                    group: crate::i2pcontrol::router_info::InspectionGroup::PeerList,
                    limit: MAX_PEER_IDENTITIES,
                });
            }
            let ids: Vec<String> = peers.iter().map(|p| p.id.clone()).collect();
            result.insert(
                rpc::router_info_keys::PEERS_KNOWN.to_string(),
                serde_json::json!(ids),
            );
        }
    }

    // Active peers: query once, use for both count and list
    let needs_active = key_set.contains(rpc::router_info_keys::PEERS_ACTIVE_COUNT)
        || key_set.contains(rpc::router_info_keys::PEERS_ACTIVE);
    if needs_active {
        let peers = router_info.active_peers().await?;
        if key_set.contains(rpc::router_info_keys::PEERS_ACTIVE_COUNT) {
            result.insert(
                rpc::router_info_keys::PEERS_ACTIVE_COUNT.to_string(),
                serde_json::json!(peers.len()),
            );
        }
        if key_set.contains(rpc::router_info_keys::PEERS_ACTIVE) {
            if peers.len() > MAX_PEER_IDENTITIES {
                return Err(InspectionError::ResultTooLarge {
                    group: crate::i2pcontrol::router_info::InspectionGroup::PeerList,
                    limit: MAX_PEER_IDENTITIES,
                });
            }
            let ids: Vec<String> = peers.iter().map(|p| p.id.clone()).collect();
            result.insert(
                rpc::router_info_keys::PEERS_ACTIVE.to_string(),
                serde_json::json!(ids),
            );
        }
    }

    // Peer RouterInfo lookup
    if key_set.contains(rpc::router_info_keys::PEERS_ROUTER_INFO) {
        match peer_ri_id {
            Some(peer_id) => {
                let ri = router_info.peer_router_info(peer_id).await?;
                result.insert(
                    rpc::router_info_keys::PEERS_ROUTER_INFO.to_string(),
                    serde_json::json!(ri),
                );
            }
            None => {
                result.insert(
                    rpc::router_info_keys::PEERS_ROUTER_INFO.to_string(),
                    serde_json::json!(null),
                );
            }
        }
    }

    // Banned peers: query once, use for both count and list
    let needs_banned = key_set.contains(rpc::router_info_keys::PEERS_BANNED)
        || key_set.contains(rpc::router_info_keys::PEERS_BANNED_COUNT);
    if needs_banned {
        let banned = router_info.banned_peers().await?;
        if key_set.contains(rpc::router_info_keys::PEERS_BANNED_COUNT) {
            result.insert(
                rpc::router_info_keys::PEERS_BANNED_COUNT.to_string(),
                serde_json::json!(banned.len()),
            );
        }
        if key_set.contains(rpc::router_info_keys::PEERS_BANNED) {
            if banned.len() > MAX_BANNED_PEERS {
                return Err(InspectionError::ResultTooLarge {
                    group: crate::i2pcontrol::router_info::InspectionGroup::PeerStats,
                    limit: MAX_BANNED_PEERS,
                });
            }
            let entries: Vec<serde_json::Value> = banned
                .iter()
                .map(|b| {
                    serde_json::json!({
                        "id": b.id,
                        "reason": b.reason,
                        "expiresAt": b.expires_at,
                    })
                })
                .collect();
            result.insert(
                rpc::router_info_keys::PEERS_BANNED.to_string(),
                serde_json::json!(entries),
            );
        }
    }
    if key_set.contains(rpc::router_info_keys::PEERS_LIMITS) {
        let limits = router_info.peer_limits().await?;
        result.insert(
            rpc::router_info_keys::PEERS_LIMITS.to_string(),
            serde_json::json!({
                "inbound": limits.configured_inbound,
                "outbound": limits.configured_outbound,
            }),
        );
    }
    if key_set.contains(rpc::router_info_keys::PEERS_ACTIVE_STATS) {
        resolve_active_peer_stats(
            result,
            rpc::router_info_keys::PEERS_ACTIVE_STATS,
            router_info,
        )
        .await?;
    }
    Ok(())
}

async fn resolve_active_peer_stats(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    router_info: &dyn RouterInfoControl,
) -> Result<(), InspectionError> {
    let stats = router_info.active_peer_stats().await?;
    if stats.len() > MAX_ACTIVE_PEER_STATS {
        return Err(InspectionError::ResultTooLarge {
            group: crate::i2pcontrol::router_info::InspectionGroup::PeerStats,
            limit: MAX_ACTIVE_PEER_STATS,
        });
    }
    let entries: Vec<serde_json::Value> = stats
        .iter()
        .map(|s| {
            serde_json::json!({
                "peerId": s.peer_id,
                "direction": s.direction,
                "state": s.state,
                "bytesReceived": s.bytes_received,
                "bytesSent": s.bytes_sent,
            })
        })
        .collect();
    result.insert(key.to_owned(), serde_json::json!(entries));
    Ok(())
}

fn tunnel_detail_value(
    details: &[crate::i2pcontrol::router_info::TunnelDetail],
) -> Result<serde_json::Value, InspectionError> {
    if details.len() > MAX_TUNNEL_DETAIL_ENTRIES {
        return Err(InspectionError::ResultTooLarge {
            group: crate::i2pcontrol::router_info::InspectionGroup::TunnelSummary,
            limit: MAX_TUNNEL_DETAIL_ENTRIES,
        });
    }
    let entries: Vec<serde_json::Value> = details
        .iter()
        .map(|detail| {
            let mut object = serde_json::Map::new();
            object.insert("tunnelId".to_owned(), serde_json::json!(detail.tunnel_id));
            if let Some(pool_id) = detail.pool_id {
                object.insert("poolId".to_owned(), serde_json::json!(pool_id));
            }
            if let Some(direction) = &detail.direction {
                object.insert("direction".to_owned(), serde_json::json!(direction));
            }
            serde_json::Value::Object(object)
        })
        .collect();
    let value = serde_json::Value::Array(entries);
    if serde_json::to_vec(&value).map_or(true, |bytes| bytes.len() > MAX_PEER_RI_BYTES) {
        return Err(InspectionError::ResultTooLarge {
            group: crate::i2pcontrol::router_info::InspectionGroup::TunnelSummary,
            limit: MAX_TUNNEL_DETAIL_ENTRIES,
        });
    }
    Ok(value)
}

fn resolve_proposal_tunnel_details(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key_set: &HashSet<&str>,
    details: &crate::i2pcontrol::router_info::TunnelDetails,
) -> Result<(), InspectionError> {
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_PARTICIPATING_INFO) {
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_PARTICIPATING_INFO.to_owned(),
            tunnel_detail_value(&details.participating)?,
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_EXPLORATORY_INBOUND) {
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_EXPLORATORY_INBOUND.to_owned(),
            serde_json::json!(details
                .exploratory
                .iter()
                .filter(|detail| detail.direction.as_deref() == Some("inbound"))
                .count()),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_EXPLORATORY_OUTBOUND) {
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_EXPLORATORY_OUTBOUND.to_owned(),
            serde_json::json!(details
                .exploratory
                .iter()
                .filter(|detail| detail.direction.as_deref() == Some("outbound"))
                .count()),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_EXPLORATORY_INFO_LIST) {
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_EXPLORATORY_INFO_LIST.to_owned(),
            tunnel_detail_value(&details.exploratory)?,
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_CLIENT_INBOUND) {
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_CLIENT_INBOUND.to_owned(),
            serde_json::json!(details
                .client
                .iter()
                .filter(|detail| detail.direction.as_deref() == Some("inbound"))
                .count()),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_CLIENT_OUTBOUND) {
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_CLIENT_OUTBOUND.to_owned(),
            serde_json::json!(details
                .client
                .iter()
                .filter(|detail| detail.direction.as_deref() == Some("outbound"))
                .count()),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NET_TUNNELS_CLIENT_INFO_LIST) {
        result.insert(
            rpc::router_info_keys::P170_NET_TUNNELS_CLIENT_INFO_LIST.to_owned(),
            tunnel_detail_value(&details.client)?,
        );
    }
    Ok(())
}

fn resolve_id(id: &Option<RequestId>) -> RequestId {
    id.clone().unwrap_or(RequestId::Null)
}

/// Resolve active peer IDs and, when requested, join them to the live public
/// RouterInfo directory. The active source is authoritative for membership;
/// a missing directory entry is a churn/incomplete-join error, never an
/// invented RouterInfo value.
async fn resolve_proposal_active_peers(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key_set: &HashSet<&str>,
    router_info: &dyn RouterInfoControl,
) -> Result<(), InspectionError> {
    let active = router_info.active_peers().await?;
    if active.len() > MAX_PEER_IDENTITIES {
        return Err(InspectionError::ResultTooLarge {
            group: crate::i2pcontrol::router_info::InspectionGroup::PeerList,
            limit: MAX_PEER_IDENTITIES,
        });
    }

    let mut ids: Vec<String> = active.into_iter().map(|peer| peer.id).collect();
    ids.sort_unstable();
    ids.dedup();
    if key_set.contains(rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_LIST) {
        result.insert(
            rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_LIST.to_string(),
            serde_json::json!(ids),
        );
    }

    if key_set.contains(rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_INFO) {
        let directory = router_info.peer_directory().await?;
        let mut infos = Vec::with_capacity(ids.len());
        let mut total_bytes = 0usize;
        for peer_id in &ids {
            let Some(bytes) = directory.router_infos.get(peer_id) else {
                return Err(InspectionError::TemporarilyUnavailable {
                    group: crate::i2pcontrol::router_info::InspectionGroup::PeerLookup,
                });
            };
            total_bytes = total_bytes.saturating_add(bytes.len());
            if total_bytes > MAX_PEER_RI_BYTES {
                return Err(InspectionError::ResultTooLarge {
                    group: crate::i2pcontrol::router_info::InspectionGroup::PeerLookup,
                    limit: MAX_PEER_RI_BYTES,
                });
            }
            infos.push(emissary_core::crypto::base64_encode(bytes.clone()));
        }
        result.insert(
            rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_INFO.to_string(),
            serde_json::json!(infos),
        );
    }

    Ok(())
}

/// Resolve the three canonical fields owned by the live public peer directory.
/// A missing serialized RouterInfo is an incomplete source snapshot and fails
/// the request; it is never replaced with an empty or adjacent value.
async fn resolve_proposal_peer_directory(
    result: &mut serde_json::Map<String, serde_json::Value>,
    key_set: &HashSet<&str>,
    router_info: &dyn RouterInfoControl,
) -> Result<(), InspectionError> {
    let snapshot = router_info.peer_directory().await?;
    if snapshot.peer_ids.len() > MAX_PEER_IDENTITIES {
        return Err(InspectionError::ResultTooLarge {
            group: crate::i2pcontrol::router_info::InspectionGroup::PeerList,
            limit: MAX_PEER_IDENTITIES,
        });
    }

    let mut ids = snapshot.peer_ids;
    ids.sort_unstable();
    ids.dedup();
    if key_set.contains(rpc::router_info_keys::P170_NETDB_PEERS) {
        result.insert(
            rpc::router_info_keys::P170_NETDB_PEERS.to_string(),
            serde_json::json!(ids),
        );
    }
    if key_set.contains(rpc::router_info_keys::P170_NETDB_PEERS_LIST) {
        result.insert(
            rpc::router_info_keys::P170_NETDB_PEERS_LIST.to_string(),
            serde_json::json!(ids),
        );
    }

    if key_set.contains(rpc::router_info_keys::P170_NETDB_PEERS_INFO) {
        let mut infos = Vec::with_capacity(ids.len());
        let mut total_bytes = 0usize;
        for peer_id in &ids {
            let Some(bytes) = snapshot.router_infos.get(peer_id) else {
                return Err(InspectionError::TemporarilyUnavailable {
                    group: crate::i2pcontrol::router_info::InspectionGroup::PeerLookup,
                });
            };
            total_bytes = total_bytes.saturating_add(bytes.len());
            if total_bytes > MAX_PEER_RI_BYTES {
                return Err(InspectionError::ResultTooLarge {
                    group: crate::i2pcontrol::router_info::InspectionGroup::PeerLookup,
                    limit: MAX_PEER_RI_BYTES,
                });
            }
            infos.push(emissary_core::crypto::base64_encode(bytes.clone()));
        }
        result.insert(
            rpc::router_info_keys::P170_NETDB_PEERS_INFO.to_string(),
            serde_json::json!(infos),
        );
    }

    Ok(())
}

fn error_response(id: RequestId, code: i32, message: impl Into<String>) -> serde_json::Value {
    serde_json::to_value(JsonRpcErrorResponse::new(id, code, message)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2pcontrol::{
        domain::address_book::{
            AddressBookConfiguration, AddressBookEntry, AdministrativeAddressBookType,
            SubscriptionSet,
        },
        router_info::*,
        rpc::JsonRpcRequest,
    };

    fn test_request(selectors: serde_json::Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "RouterInfo".to_string(),
            params: Some(serde_json::json!({"Selector": selectors}).as_object().cloned().unwrap()),
            id: Some(rpc::RequestId::Number(1)),
        }
    }

    fn direct_request(params: serde_json::Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "RouterInfo".to_string(),
            params: Some(params.as_object().cloned().unwrap()),
            id: Some(rpc::RequestId::Number(1)),
        }
    }

    fn test_state(ri: FakeRouterInfoControl) -> crate::i2pcontrol::server::I2pControlState {
        let mut state = crate::i2pcontrol::server::I2pControlState::new_test("test".to_string());
        state.set_router_info(Box::new(ri));
        state
    }

    #[tokio::test]
    async fn handle_router_info_empty_selector() {
        let ri = FakeRouterInfoControl::new();
        let state = test_state(ri);
        let req = test_request(serde_json::json!({}));
        let resp = handle_router_info(&state, &req).await;
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp["result"].is_object());
        assert_eq!(resp["result"].as_object().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn canonical_direct_wire_fixture_returns_exact_fields() {
        let ri = FakeRouterInfoControl::new();
        ri.set_clock_skew(ClockSkew {
            skew_seconds: Some(-3),
        });
        ri.set_transport_bytes(TransportBytes {
            received: 11,
            sent: 22,
        });
        ri.set_share_ratio(1.25);
        ri.set_build_stats(TunnelBuildStats {
            successes: 3,
            failures: 1,
        });
        let mut state = test_state(ri);
        state.set_startup_values("router-hash".into(), vec![1, 2, 3], "router-info".into());
        let req = direct_request(serde_json::json!({
            "i2p.router.id": false,
            "i2p.router.info": null,
            "i2p.router.clockskew": 0,
            "i2p.router.net.total.received.bytes": "present",
            "i2p.router.net.total.sent.bytes": true,
            "i2p.router.net.tunnels.shareratio": {},
            "i2p.router.net.tunnels.i2ptunnel": true,
            "i2p.router.net.tunnels.totalsuccessrate": true,
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"]
            .as_object()
            .unwrap_or_else(|| panic!("canonical fixture response: {resp}"));
        assert_eq!(result["i2p.router.id"], "router-hash");
        assert_eq!(result["i2p.router.info"], "router-info");
        assert_eq!(result["i2p.router.clockskew"], -3);
        assert_eq!(result["i2p.router.net.total.received.bytes"], 11);
        assert_eq!(result["i2p.router.net.total.sent.bytes"], 22);
        assert_eq!(result["i2p.router.net.tunnels.shareratio"], 1.25);
        assert_eq!(
            result["i2p.router.net.tunnels.i2ptunnel"],
            serde_json::json!([])
        );
        assert_eq!(result["i2p.router.net.tunnels.totalsuccessrate"], 75.0);
    }

    #[tokio::test]
    async fn m049_wire_fixture_returns_recent_success_and_live_queues() {
        let ri = FakeRouterInfoControl::new();
        ri.set_recent_tunnel_success_rate(73.0);
        ri.set_tunnel_details(TunnelDetails {
            queue_depth: 4,
            tbm_queue_depth: 7,
            ..Default::default()
        });
        let state = test_state(ri);
        let req = direct_request(serde_json::json!({
            "i2p.router.net.tunnels.successrate": null,
            "i2p.router.net.tunnels.queue": true,
            "i2p.router.net.tunnels.tbmqueue": {},
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"]
            .as_object()
            .unwrap_or_else(|| panic!("M049 fixture response: {resp}"));
        assert_eq!(result["i2p.router.net.tunnels.successrate"], 73.0);
        assert_eq!(result["i2p.router.net.tunnels.queue"], 4);
        assert_eq!(result["i2p.router.net.tunnels.tbmqueue"], 7);
    }

    #[tokio::test]
    async fn canonical_transit_bytes_returns_forwarded_counter_only() {
        let ri = FakeRouterInfoControl::new();
        ri.set_transit_bytes(TransitBytes {
            received: 11,
            sent: 22,
        });
        let state = test_state(ri);
        let resp = handle_router_info(
            &state,
            &direct_request(serde_json::json!({
                "i2p.router.net.total.transit.bytes": false,
            })),
        )
        .await;

        assert_eq!(
            resp["result"]["i2p.router.net.total.transit.bytes"], 22,
            "transit bytes are forwarded/transmitted bytes, not received plus sent"
        );
        assert!(resp["result"]["i2p.router.net.total.transit.bytes"].is_u64());
    }

    #[tokio::test]
    async fn canonical_logs_and_presence_semantics_are_literal() {
        let ri = FakeRouterInfoControl::new();
        ri.add_log_entry(LogEntry {
            timestamp_ms: 1,
            level: "INFO".into(),
            target: "test".into(),
            message: "hello".into(),
        });
        let state = test_state(ri);
        let req = direct_request(serde_json::json!({
            "i2p.router.logs": true,
            "i2p.router.logs.clear": null,
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result["i2p.router.logs"], serde_json::json!(["hello"]));
        assert_eq!(result["i2p.router.logs.clear"], "success");
    }

    #[tokio::test]
    async fn router_news_without_an_owner_is_unavailable() {
        let state = test_state(FakeRouterInfoControl::new());
        let resp = handle_router_info(
            &state,
            &direct_request(serde_json::json!({rpc::router_info_keys::ROUTER_NEWS: true})),
        )
        .await;
        assert_eq!(resp["error"]["code"], -32603);
        assert!(resp["error"]["message"].as_str().unwrap().contains("no router news owner"));
    }

    #[tokio::test]
    async fn direct_and_nested_routerinfo_modes_are_distinct() {
        let ri = FakeRouterInfoControl::new();
        ri.set_router_news("legacy news".into());
        let state = test_state(ri);

        let nested = handle_router_info(
            &state,
            &test_request(serde_json::json!({rpc::router_info_keys::ROUTER_NEWS: true})),
        )
        .await;
        assert_eq!(
            nested["result"][rpc::router_info_keys::ROUTER_NEWS],
            "legacy news"
        );

        let direct = handle_router_info(
            &state,
            &direct_request(serde_json::json!({rpc::router_info_keys::ROUTER_NEWS: false})),
        )
        .await;
        assert_eq!(direct["error"]["code"], rpc::error_codes::INTERNAL_ERROR);
        assert!(direct["error"]["message"].as_str().unwrap().contains("no router news owner"));
    }

    #[tokio::test]
    async fn nested_news_uses_legacy_disposition() {
        let ri = FakeRouterInfoControl::new();
        ri.set_router_news("historical news".into());
        let response = handle_router_info(
            &test_state(ri),
            &test_request(serde_json::json!({rpc::router_info_keys::ROUTER_NEWS: true})),
        )
        .await;

        assert_eq!(
            response["result"][rpc::router_info_keys::ROUTER_NEWS],
            "historical news"
        );
    }

    #[tokio::test]
    async fn direct_news_uses_p170_disposition() {
        let ri = FakeRouterInfoControl::new();
        ri.set_router_news("must not bypass direct disposition".into());
        let response = handle_router_info(
            &test_state(ri),
            &direct_request(serde_json::json!({rpc::router_info_keys::ROUTER_NEWS: true})),
        )
        .await;

        assert_eq!(response["error"]["code"], rpc::error_codes::INTERNAL_ERROR);
        assert!(response["result"].is_null());
    }

    #[tokio::test]
    async fn direct_banned_peers_do_not_promote_fake_or_empty_values() {
        let ri = FakeRouterInfoControl::new();
        ri.set_banned_peers(vec![BannedPeer {
            id: "peer-id".into(),
            reason: "test reason".into(),
            expires_at: Some(123),
        }]);
        let response = handle_router_info(
            &test_state(ri),
            &direct_request(serde_json::json!({
                rpc::router_info_keys::P170_NETDB_BANNED_PEERS: true
            })),
        )
        .await;

        assert_eq!(response["error"]["code"], rpc::error_codes::INTERNAL_ERROR);
        assert!(response["result"].is_null());
    }

    #[tokio::test]
    async fn canonical_peer_directory_fields_return_exact_wire_values() {
        let ri = FakeRouterInfoControl::new();
        ri.set_peer_directory(PeerDirectorySnapshot {
            peer_ids: vec!["peer-b".into(), "peer-a".into(), "peer-a".into()],
            router_infos: std::collections::BTreeMap::from([
                ("peer-a".into(), vec![1, 2]),
                ("peer-b".into(), vec![3, 4]),
            ]),
        });
        let state = test_state(ri);
        let req = direct_request(serde_json::json!({
            "i2p.router.netdb.peers": true,
            "i2p.router.netdb.peers.list": true,
            "i2p.router.netdb.peers.info": true,
        }));
        let resp = handle_router_info(&state, &req).await;
        assert_eq!(
            resp["result"]["i2p.router.netdb.peers"],
            serde_json::json!(["peer-a", "peer-b"])
        );
        assert_eq!(
            resp["result"]["i2p.router.netdb.peers.list"],
            serde_json::json!(["peer-a", "peer-b"])
        );
        assert_eq!(
            resp["result"]["i2p.router.netdb.peers.info"],
            serde_json::json!(["AQI=", "AwQ="])
        );
    }

    #[tokio::test]
    async fn canonical_active_peer_inventory_and_limits_return_exact_wire_values() {
        let ri = FakeRouterInfoControl::new();
        ri.set_active_peers(vec![
            PeerIdentity {
                id: "peer-b".into(),
                is_active: true,
            },
            PeerIdentity {
                id: "peer-a".into(),
                is_active: true,
            },
            PeerIdentity {
                id: "peer-a".into(),
                is_active: true,
            },
        ]);
        ri.set_peer_directory(PeerDirectorySnapshot {
            peer_ids: vec!["peer-a".into(), "peer-b".into()],
            router_infos: std::collections::BTreeMap::from([
                ("peer-a".into(), vec![1, 2]),
                ("peer-b".into(), vec![3, 4]),
            ]),
        });
        ri.set_transport_limits(TransportLimits {
            ntcp_limit: Some(64),
            ssu_limit: Some(128),
        });
        let response = handle_router_info(
            &test_state(ri),
            &direct_request(serde_json::json!({
                rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_LIST: true,
                rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_INFO: true,
                rpc::router_info_keys::P170_NETDB_NTCP_LIMIT: true,
                rpc::router_info_keys::P170_NETDB_SSU_LIMIT: true,
            })),
        )
        .await;

        assert_eq!(
            response["result"][rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_LIST],
            serde_json::json!(["peer-a", "peer-b"])
        );
        assert_eq!(
            response["result"][rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_INFO],
            serde_json::json!(["AQI=", "AwQ="])
        );
        assert_eq!(
            response["result"][rpc::router_info_keys::P170_NETDB_NTCP_LIMIT],
            64
        );
        assert_eq!(
            response["result"][rpc::router_info_keys::P170_NETDB_SSU_LIMIT],
            128
        );
    }

    #[tokio::test]
    async fn active_peer_router_info_join_fails_closed_on_source_churn() {
        let ri = FakeRouterInfoControl::new();
        ri.set_active_peers(vec![PeerIdentity {
            id: "peer-missing".into(),
            is_active: true,
        }]);
        ri.set_peer_directory(PeerDirectorySnapshot::default());
        let response = handle_router_info(
            &test_state(ri),
            &direct_request(serde_json::json!({
                rpc::router_info_keys::P170_NETDB_ACTIVE_PEERS_INFO: true,
            })),
        )
        .await;

        assert_eq!(response["error"]["code"], rpc::error_codes::INTERNAL_ERROR);
        assert!(response["result"].is_null());
    }

    #[tokio::test]
    async fn unlimited_transport_limit_is_unavailable_not_a_sentinel() {
        let ri = FakeRouterInfoControl::new();
        ri.set_transport_limits(TransportLimits {
            ntcp_limit: None,
            ssu_limit: Some(128),
        });
        let response = handle_router_info(
            &test_state(ri),
            &direct_request(serde_json::json!({
                rpc::router_info_keys::P170_NETDB_NTCP_LIMIT: true,
            })),
        )
        .await;

        assert_eq!(response["error"]["code"], rpc::error_codes::INTERNAL_ERROR);
        assert!(response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("finite NTCP2 connection limit unavailable"));
        assert!(response["result"].is_null());
    }

    #[tokio::test]
    async fn every_frozen_m026_unavailable_field_fails_without_fabricated_result() {
        for field in rpc::router_info_keys::PROPOSAL_170_CONTRACT.iter().filter(|field| {
            matches!(
                field.source,
                rpc::router_info_keys::SourceDisposition::Unavailable { .. }
            )
        }) {
            let mut params = serde_json::Map::new();
            params.insert(field.key.to_string(), serde_json::Value::Bool(false));
            let response = handle_router_info(
                &test_state(FakeRouterInfoControl::new()),
                &direct_request(serde_json::Value::Object(params)),
            )
            .await;

            assert_eq!(response["error"]["code"], -32603, "selector: {}", field.key);
            assert!(response["result"].is_null(), "selector: {}", field.key);
            assert!(
                response["error"]["message"]
                    .as_str()
                    .is_some_and(|message| message.contains(field.source.reason().unwrap())),
                "selector: {}",
                field.key
            );
        }
    }

    #[tokio::test]
    async fn canonical_address_book_shapes_are_literal_and_requested_only() {
        let state = test_state(FakeRouterInfoControl::new());
        state
            .address_book_add(
                AdministrativeAddressBookType::Private,
                AddressBookEntry::new("private.i2p", "destination"),
            )
            .await
            .unwrap();
        let mut subscriptions = SubscriptionSet::new();
        subscriptions.push("https://example.i2p/hosts.txt".to_string());
        state.address_book_set_subscriptions(subscriptions).await.unwrap();
        let mut config = AddressBookConfiguration::new();
        config.insert("updateInterval".to_string(), "3600".to_string());
        state.address_book_set_configuration(config).await.unwrap();

        let resp = handle_router_info(
            &state,
            &direct_request(serde_json::json!({
                rpc::router_info_keys::P170_ADDRESS_BOOK_PRIVATE_LIST: null,
                rpc::router_info_keys::P170_ADDRESS_BOOK_SUBSCRIPTIONS: false,
                rpc::router_info_keys::P170_ADDRESS_BOOK_CONFIG: true,
            })),
        )
        .await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(
            result["i2p.router.addressbook.private.list"],
            serde_json::json!([{
                "name": "private.i2p",
                "value": "destination"
            }])
        );
        assert_eq!(
            result["i2p.router.addressbook.subscriptions"],
            serde_json::json!({"path": null, "entries": ["https://example.i2p/hosts.txt"]})
        );
        assert_eq!(
            result["i2p.router.addressbook.config"],
            serde_json::json!({"path": null, "entries": {"updateInterval": "3600"}})
        );
    }

    #[tokio::test]
    async fn nested_addressbook_metadata_uses_legacy_shape() {
        let state = test_state(FakeRouterInfoControl::new());
        let mut subscriptions = SubscriptionSet::new();
        subscriptions.push("https://example.i2p/hosts.txt".to_string());
        state.address_book_set_subscriptions(subscriptions).await.unwrap();
        let mut config = AddressBookConfiguration::new();
        config.insert("updateInterval".to_string(), "3600".to_string());
        state.address_book_set_configuration(config).await.unwrap();

        let response = handle_router_info(
            &state,
            &test_request(serde_json::json!({
                rpc::router_info_keys::ADDRESS_BOOK_SUBSCRIPTIONS: true,
                rpc::router_info_keys::ADDRESS_BOOK_CONFIG: true,
            })),
        )
        .await;
        let result = response["result"].as_object().unwrap();
        assert_eq!(
            result[rpc::router_info_keys::ADDRESS_BOOK_SUBSCRIPTIONS],
            serde_json::json!(["https://example.i2p/hosts.txt"])
        );
        assert_eq!(
            result[rpc::router_info_keys::ADDRESS_BOOK_CONFIG],
            serde_json::json!({"updateInterval": "3600"})
        );
    }

    #[tokio::test]
    async fn mixed_modes_are_rejected_before_query() {
        let response = handle_router_info(
            &test_state(FakeRouterInfoControl::new()),
            &JsonRpcRequest {
                jsonrpc: "2.0".into(),
                method: "RouterInfo".into(),
                params: Some(
                    serde_json::json!({
                        "Selector": {rpc::router_info_keys::VERSION: true},
                        rpc::router_info_keys::P170_ID: true,
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
                id: Some(rpc::RequestId::Number(1)),
            },
        )
        .await;

        assert_eq!(response["error"]["code"], rpc::error_codes::INVALID_PARAMS);
        assert!(response["error"]["message"].as_str().unwrap().contains("cannot be mixed"));
    }

    #[tokio::test]
    async fn nested_proposal_only_selector_is_rejected() {
        let response = handle_router_info(
            &test_state(FakeRouterInfoControl::new()),
            &test_request(serde_json::json!({rpc::router_info_keys::P170_ID: true})),
        )
        .await;

        assert_eq!(response["error"]["code"], rpc::error_codes::INVALID_PARAMS);
        assert!(response["result"].is_null());
    }

    #[tokio::test]
    async fn actual_serialized_response_bound_rejects_underestimated_log_payload() {
        let ri = FakeRouterInfoControl::new();
        for index in 0..10_000 {
            ri.add_log_entry(LogEntry {
                timestamp_ms: index,
                level: "INFO".to_string(),
                target: "test".to_string(),
                message: "x".repeat(1_100),
            });
        }
        let state = test_state(ri);
        let resp = handle_router_info(
            &state,
            &direct_request(serde_json::json!({rpc::router_info_keys::P170_LOGS: true})),
        )
        .await;
        assert_eq!(resp["error"]["code"], -32603);
        assert!(resp["error"]["message"].as_str().unwrap().contains("serialized bound"));
        assert!(resp["result"].is_null());
    }

    #[tokio::test]
    async fn handle_router_info_version_only() {
        let ri = FakeRouterInfoControl::new();
        ri.set_version("Test 2.0".to_string());
        let state = test_state(ri);
        let req = test_request(serde_json::json!({"i2p.router.version": true}));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result["i2p.router.version"], "Test 2.0");
    }

    #[tokio::test]
    async fn handle_router_info_uptime_and_version() {
        let ri = FakeRouterInfoControl::new();
        ri.set_version("Emissary 0.5.0".to_string());
        ri.set_uptime_ms(120000);
        let state = test_state(ri);
        let req = test_request(serde_json::json!({
            "i2p.router.version": true,
            "i2p.router.uptime": true
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result["i2p.router.version"], "Emissary 0.5.0");
        assert_eq!(result["i2p.router.uptime"], 120000);
    }

    #[tokio::test]
    async fn handle_router_info_unknown_selector() {
        let ri = FakeRouterInfoControl::new();
        let state = test_state(ri);
        let req = test_request(serde_json::json!({"unknown.selector": true}));
        let resp = handle_router_info(&state, &req).await;
        assert_eq!(resp["error"]["code"], -32602);
    }

    #[tokio::test]
    async fn handle_router_info_false_selector_ignored() {
        let ri = FakeRouterInfoControl::new();
        ri.set_version("Test".to_string());
        let state = test_state(ri);
        let req = test_request(serde_json::json!({"i2p.router.version": false}));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn handle_router_info_missing_selector_param() {
        let ri = FakeRouterInfoControl::new();
        let state = test_state(ri);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "RouterInfo".to_string(),
            params: Some(serde_json::json!({"Token": "abc"}).as_object().cloned().unwrap()),
            id: Some(rpc::RequestId::Number(1)),
        };
        let resp = handle_router_info(&state, &req).await;
        assert_eq!(resp["error"]["code"], -32602);
    }

    #[tokio::test]
    async fn handle_router_info_udp_selectors() {
        let ri = FakeRouterInfoControl::new();
        ri.set_udp(UdpSnapshot {
            active: true,
            cookie_active: true,
            integrated_peers: 5,
            firewalled: false,
            hidden: false,
            ..Default::default()
        });
        let state = test_state(ri);
        let req = test_request(serde_json::json!({
            "i2p.router.udp.active": true,
            "i2p.router.udp.firewalled": true,
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result["i2p.router.udp.active"], true);
        assert_eq!(result["i2p.router.udp.firewalled"], false);
    }

    #[tokio::test]
    async fn handle_router_info_unsupported_udp_returns_error() {
        let ri = FakeRouterInfoControl::new();
        ri.set_udp(UdpSnapshot {
            active: true,
            ..Default::default()
        });
        let state = test_state(ri);
        let req = test_request(serde_json::json!({
            "i2p.router.udp.active": true,
            "i2p.router.udp.integratedPeers": true,
        }));
        let resp = handle_router_info(&state, &req).await;
        // integratedPeers is unsupported — entire request fails
        assert!(resp.get("error").is_some());
        assert!(resp.get("result").is_none());
    }

    #[tokio::test]
    async fn handle_router_info_tcp_selectors() {
        let ri = FakeRouterInfoControl::new();
        ri.set_tcp(TcpSnapshot {
            active: true,
            integrated_peers: 3,
            firewalled: false,
            hosts: "0.0.0.0:4444".to_string(),
            status: "Active".to_string(),
            version: "NTCP2".to_string(),
        });
        let state = test_state(ri);
        let req = test_request(serde_json::json!({
            "i2p.router.tcp.active": true,
            "i2p.router.tcp.status": true,
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result["i2p.router.tcp.active"], true);
        assert_eq!(result["i2p.router.tcp.status"], "Active");
    }

    #[tokio::test]
    async fn handle_router_info_netdb_selectors() {
        let ri = FakeRouterInfoControl::new();
        ri.set_netdb(NetDbSnapshot {
            active: true,
            known_profiles: 100,
            active_profiles: 50,
            ..Default::default()
        });
        let state = test_state(ri);
        let req = test_request(serde_json::json!({
            "i2p.router.netdb.active": true,
            "i2p.router.netdb.knownProfiles": true,
            "i2p.router.netdb.activeProfiles": true,
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result["i2p.router.netdb.active"], true);
        assert_eq!(result["i2p.router.netdb.knownProfiles"], 100);
        assert_eq!(result["i2p.router.netdb.activeProfiles"], 50);
    }

    #[tokio::test]
    async fn handle_router_info_bw_selectors() {
        let ri = FakeRouterInfoControl::new();
        ri.set_transport_bytes(crate::i2pcontrol::router_info::TransportBytes {
            received: 1000000,
            sent: 500000,
        });
        let state = test_state(ri);
        let req = test_request(serde_json::json!({
            "i2p.router.bw.inbound.total": true,
            "i2p.router.bw.outbound.total": true,
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result["i2p.router.bw.inbound.total"], 1000000);
        assert_eq!(result["i2p.router.bw.outbound.total"], 500000);
    }

    #[tokio::test]
    async fn handle_router_info_unrelated_keys_absent() {
        let ri = FakeRouterInfoControl::new();
        ri.set_version("Test".to_string());
        let state = test_state(ri);
        let req = test_request(serde_json::json!({
            "i2p.router.version": true
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("i2p.router.version"));
        assert!(!result.contains_key("i2p.router.uptime"));
        assert!(!result.contains_key("i2p.router.identity"));
    }

    #[tokio::test]
    async fn handle_router_info_network_status() {
        let ri = FakeRouterInfoControl::new();
        ri.set_network(NetworkSnapshot {
            ipv4_status: NetworkStatus::Ok,
            ipv6_status: NetworkStatus::Firewalled,
            ..Default::default()
        });
        let state = test_state(ri);
        let req = test_request(serde_json::json!({
            "i2p.router.net.bw.inbound": true,
            "i2p.router.net.bw.outbound": true,
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result["i2p.router.net.bw.inbound"], "OK");
        assert_eq!(result["i2p.router.net.bw.outbound"], "Firewalled");
    }

    #[tokio::test]
    async fn handle_router_info_network_state_wire_fixture() {
        let ri = FakeRouterInfoControl::new();
        ri.set_network(NetworkSnapshot {
            ipv4_status: NetworkStatus::Ok,
            ipv6_status: NetworkStatus::Firewalled,
            ipv4_testing: true,
            ipv6_testing: false,
            ..Default::default()
        });
        let state = test_state(ri);
        let req = direct_request(serde_json::json!({
            "i2p.router.net.status.v6": false,
            "i2p.router.net.testing": 0,
            "i2p.router.net.testing.v6": true,
        }));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result["i2p.router.net.status.v6"], 1);
        assert_eq!(result["i2p.router.net.testing"], 1);
        assert_eq!(result["i2p.router.net.testing.v6"], 0);
    }

    #[tokio::test]
    async fn network_errors_are_unavailable_without_partial_results() {
        for request in [
            direct_request(serde_json::json!({
                rpc::router_info_keys::P170_NET_ERROR: false,
            })),
            direct_request(serde_json::json!({
                rpc::router_info_keys::P170_NET_ERROR_V6: false,
            })),
            direct_request(serde_json::json!({
                rpc::router_info_keys::P170_NET_STATUS_V6: false,
                rpc::router_info_keys::P170_NET_TESTING: false,
                rpc::router_info_keys::P170_NET_TESTING_V6: false,
                rpc::router_info_keys::P170_NET_ERROR: false,
                rpc::router_info_keys::P170_NET_ERROR_V6: false,
            })),
        ] {
            let response =
                handle_router_info(&test_state(FakeRouterInfoControl::new()), &request).await;
            assert_eq!(response["error"]["code"], rpc::error_codes::INTERNAL_ERROR);
            assert!(response["result"].is_null());
            assert!(response["error"]["message"]
                .as_str()
                .is_some_and(|message| message.contains("no canonical network-error owner")));
        }
    }

    #[tokio::test]
    async fn handle_router_info_unavailable_returns_error() {
        // Fake defaults to Unavailable for all groups
        let ri = FakeRouterInfoControl::new();
        let state = test_state(ri);
        let req = test_request(serde_json::json!({"i2p.router.netdb.active": true}));
        let resp = handle_router_info(&state, &req).await;
        assert!(resp.get("error").is_some());
        assert!(resp.get("result").is_none());
        assert_eq!(resp["error"]["code"], -32603);
    }

    #[tokio::test]
    async fn handle_router_info_available_zero_is_success() {
        let ri = FakeRouterInfoControl::new();
        ri.set_known_peers(Vec::new());
        let state = test_state(ri);
        let req = test_request(serde_json::json!({"i2p.router.peers.knownCount": true}));
        let resp = handle_router_info(&state, &req).await;
        let result = resp["result"].as_object().unwrap();
        assert_eq!(result["i2p.router.peers.knownCount"], 0);
    }

    #[tokio::test]
    async fn handle_router_info_no_partial_on_failure() {
        let ri = FakeRouterInfoControl::new();
        ri.set_version("Test".to_string());
        // netdb defaults to Unavailable
        let state = test_state(ri);
        let req = test_request(serde_json::json!({
            "i2p.router.version": true,
            "i2p.router.netdb.active": true
        }));
        let resp = handle_router_info(&state, &req).await;
        // netdb failure should abort the entire request
        assert!(resp.get("error").is_some());
        assert!(resp.get("result").is_none());
    }
}
