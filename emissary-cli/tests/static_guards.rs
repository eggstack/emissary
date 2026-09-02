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

//! Static and structural guards for the I2PControl M005 boundary.
//!
//! These tests verify the invariants required by the M005 plan and Proposal
//! 170's read-only inspection architecture:
//!
//! - No `EventSubscriber` use anywhere in the I2PControl code path.
//! - No UI / frontend module imports in inspection code.
//! - No HTTP / JSON-RPC / server dependencies in the emissary-core crate.
//! - `RouterInfoControl` and its DTOs do not expose private key types or mutable core handles.
//! - The selector registry contains exactly the Proposal 170 keys.
//! - The handler only returns requested selector keys.
//! - The router info adapter never mutates state.

#![cfg(feature = "i2pcontrol")]

use std::path::Path;

use emissary_cli::i2pcontrol::{
    control_plane::{AddressBookControl, TunnelManagerControl},
    production::{
        EventMetrics, ProductionAddressBookControl, ProductionControlPlane,
        ProductionRouterInfoControl, ProductionTunnelManagerControl,
    },
    router_info::{
        ActivePeerStats, BannedPeerSource, I2PTunnelStats, InspectionError, LogSnapshot,
        NetworkSnapshot, PeerLimits, RecentTransitTraffic, RouterInfoControl, TransitBytes,
        TransportBytes, TunnelBuildStats, TunnelSummary, BANNED_PEER_SOURCE,
    },
    rpc,
};

// --- Source-level structural guards (probed by file reads) ---

const I2PCONTROL_FILES: &[&str] = &[
    "src/i2pcontrol/address_book.rs",
    "src/i2pcontrol/auth.rs",
    "src/i2pcontrol/control_plane.rs",
    "src/i2pcontrol/observability.rs",
    "src/i2pcontrol/production.rs",
    "src/i2pcontrol/router_info.rs",
    "src/i2pcontrol/router_info_handler.rs",
    "src/i2pcontrol/rpc.rs",
    "src/i2pcontrol/server.rs",
    "src/i2pcontrol/tunnel_manager.rs",
];

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn read_source(rel: &str) -> String {
    let p = workspace_root().join("emissary-cli").join(rel);
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("read {}: {e}", rel))
}

#[test]
fn i2pcontrol_does_not_consume_event_subscriber() {
    for f in I2PCONTROL_FILES {
        let src = read_source(f);
        // Disallow imports or usages; doc comments mentioning the name
        // are acceptable because they document the invariant.
        for line in src.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            assert!(
                !line.contains("EventSubscriber"),
                "I2PControl file {f} must not reference EventSubscriber: {line}"
            );
        }
    }
}

#[test]
fn i2pcontrol_does_not_import_ui_modules() {
    for f in I2PCONTROL_FILES {
        let src = read_source(f);
        assert!(
            !src.contains("crate::ui") && !src.contains("crate::dioxus"),
            "I2PControl file {f} must not import the UI module"
        );
    }
}

#[test]
fn i2pcontrol_does_not_import_http_or_serde_json_server_libs() {
    for f in I2PCONTROL_FILES {
        let src = read_source(f);
        // axum is the HTTP framework; allowed in server.rs only because
        // the I2PControl server is HTTP-based. Other files must not use it.
        if f.ends_with("server.rs") {
            continue;
        }
        assert!(
            !src.contains("use axum") && !src.contains("axum::"),
            "I2PControl file {f} must not import axum (HTTP server framework)"
        );
    }
}

#[test]
fn emissary_core_cargo_has_no_i2pcontrol_dependencies() {
    let p = workspace_root().join("emissary-core").join("Cargo.toml");
    let s = std::fs::read_to_string(p).unwrap();
    for forbidden in [
        "axum",
        "hyper",
        "tokio-rustls",
        "rustls-pemfile",
        "serde_json",
    ] {
        assert!(
            !s.contains(forbidden),
            "emissary-core must not depend on {forbidden}"
        );
    }
}

#[test]
fn router_info_dtos_do_not_expose_signing_or_static_key() {
    // The router info DTOs should not have fields of type SigningPrivateKey or
    // StaticPrivateKey or any other private key material. The DTOs are pure
    // protocol-required primitives. Allow doc comments but no actual
    // type references.
    let d = read_source("src/i2pcontrol/router_info.rs");
    for line in d.lines() {
        if line.trim().starts_with("//") {
            continue;
        }
        assert!(
            !line.contains("SigningPrivateKey"),
            "router_info DTOs must not reference SigningPrivateKey: {line}"
        );
        assert!(
            !line.contains("StaticPrivateKey"),
            "router_info DTOs must not reference StaticPrivateKey: {line}"
        );
        assert!(
            !line.contains("NoiseContext"),
            "router_info DTOs must not reference NoiseContext: {line}"
        );
    }
}

#[test]
fn router_info_control_trait_is_send_sync_and_async() {
    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<dyn RouterInfoControl>();
    assert_send_sync::<dyn AddressBookControl>();
    assert_send_sync::<dyn TunnelManagerControl>();
}

#[test]
fn production_router_info_does_not_mutate_state() {
    // The production adapter's read methods must be visible. There is no
    // `set_*` or `mutate_*` method exposed on the trait.
    let d = read_source("src/i2pcontrol/production.rs");
    for line in d.lines() {
        let l = line.trim();
        if l.starts_with("pub fn ") || l.starts_with("pub(crate) fn ") {
            let forbidden = l.contains("fn set_")
                || l.contains("fn mutate_")
                || l.contains("fn write_")
                || l.contains("fn update_")
                || l.contains("fn trigger_");
            assert!(
                !forbidden,
                "Production router info adapter must not expose mutation: {l}"
            );
        }
    }
}

// --- Runtime structural guards (asserted against in-memory state) ---

fn make_production_router_info() -> ProductionRouterInfoControl {
    let metrics: Arc<dyn EventMetrics> = Arc::new(NullMetrics);
    let tunnel_mgr = Arc::new(
        ProductionTunnelManagerControl::new(
            std::env::temp_dir().join("emissary-i2pcontrol-static-guard"),
        )
        .expect("tunnel manager"),
    );
    let log_ring = Arc::new(emissary_cli::i2pcontrol::observability::LogRing::default());
    ProductionRouterInfoControl::new(
        "test".to_string(),
        "test".to_string(),
        0.0,
        0,
        0,
        metrics,
        log_ring,
        tunnel_mgr,
    )
}

use std::sync::Arc;

struct NullMetrics;

impl EventMetrics for NullMetrics {
    fn transport_inbound_bytes(&self) -> u64 {
        0
    }
    fn transport_outbound_bytes(&self) -> u64 {
        0
    }
    fn transit_inbound_bytes(&self) -> u64 {
        0
    }
    fn transit_outbound_bytes(&self) -> u64 {
        0
    }
    fn connected_routers(&self) -> usize {
        0
    }
    fn transit_tunnel_count(&self) -> usize {
        0
    }
    fn tunnel_build_successes(&self) -> u64 {
        0
    }
    fn tunnel_build_failures(&self) -> u64 {
        0
    }
    fn ipv4_firewall_status(&self) -> emissary_core::FirewallStatus {
        emissary_core::FirewallStatus::Unknown
    }
    fn ipv6_firewall_status(&self) -> emissary_core::FirewallStatus {
        emissary_core::FirewallStatus::Unknown
    }
}

#[test]
fn router_info_dtos_clone_and_default() {
    // DTOs are plain data; they must support clone, default, debug, send, sync.
    fn assert_data<T: Clone + Default + std::fmt::Debug + Send + Sync>() {}

    assert_data::<NetworkSnapshot>();
    assert_data::<TransportBytes>();
    assert_data::<TransitBytes>();
    assert_data::<RecentTransitTraffic>();
    assert_data::<TunnelBuildStats>();
    assert_data::<TunnelSummary>();
    assert_data::<ActivePeerStats>();
    assert_data::<PeerLimits>();
    assert_data::<I2PTunnelStats>();
    assert_data::<LogSnapshot>();
}

#[test]
fn selector_registry_is_complete() {
    // The registry contains the legacy/base selectors plus the exact
    // Proposal 170 additions. The additions themselves are checked against
    // the normative manifest in conformance_manifest.rs.
    assert_eq!(rpc::router_info_keys::ALL.len(), 161);
}

#[test]
fn selector_registry_has_unique_keys() {
    use std::collections::HashSet;
    let set: HashSet<&str> = rpc::router_info_keys::ALL.iter().copied().collect();
    assert_eq!(set.len(), rpc::router_info_keys::ALL.len());
}

#[test]
fn selector_registry_address_book_partition() {
    use std::collections::HashSet;
    let all: HashSet<&str> = rpc::router_info_keys::ALL.iter().copied().collect();
    let ab: HashSet<&str> = rpc::router_info_keys::ADDRESS_BOOK_KEYS.iter().copied().collect();
    let core: HashSet<&str> = rpc::router_info_keys::CORE_KEYS.iter().copied().collect();

    // CORE ∪ ADDRESS_BOOK is the legacy/base partition. Exact Proposal 170
    // additions extend that partition without changing either legacy set.
    let legacy: HashSet<&str> = core.union(&ab).copied().collect();
    assert_eq!(legacy.len(), 121);
    assert!(all.is_superset(&legacy));
    // CORE ∩ ADDRESS_BOOK = ∅
    assert!(core.is_disjoint(&ab));
}

#[test]
fn production_adapter_returns_unavailable_for_unimplemented_selectors() {
    // The production adapter does not yet wire active peers, peer limits, or
    // netdb summaries. Banned peers are the explicit by-design-empty source.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let ri = make_production_router_info();
    rt.block_on(async {
        assert!(matches!(
            ri.known_peers().await,
            Err(InspectionError::Unavailable { .. })
        ));
        assert!(matches!(
            ri.active_peers().await,
            Err(InspectionError::Unavailable { .. })
        ));
        assert!(ri.banned_peers().await.unwrap().is_empty());
        assert!(matches!(
            ri.active_peer_stats().await,
            Err(InspectionError::Unavailable { .. })
        ));
        assert!(matches!(
            ri.peer_router_info("any").await,
            Err(InspectionError::Unavailable { .. })
        ));
        assert!(matches!(
            ri.peer_limits().await,
            Err(InspectionError::Unavailable { .. })
        ));
    });
}

#[test]
fn banned_peer_source_is_explicitly_by_design_empty() {
    assert_eq!(BANNED_PEER_SOURCE, BannedPeerSource::ByDesignEmpty);
    let source = read_source("src/i2pcontrol/production.rs");
    assert!(source.contains("BANNED_PEER_SOURCE.snapshot()"));
}

#[test]
fn transit_bandwidth_15s_has_no_request_local_sampler() {
    let source = read_source("src/i2pcontrol/production.rs");
    let sampler = read_source("src/i2pcontrol/transit_sampler.rs");
    assert!(source.contains("TransitBandwidthSampler::start"));
    assert!(source.contains("transit_bandwidth_sampler"));
    assert!(sampler.contains("interval_at"));
    assert!(sampler.contains("MissedTickBehavior::Skip"));
    assert!(!source.contains("fn sample(&mut self"));
    assert!(!source.contains("self.transit_bandwidth_sampler.lock"));
}

#[test]
fn router_news_fetch_is_owned_and_bounded_outside_the_handler() {
    let handler = read_source("src/i2pcontrol/router_info_handler.rs");
    let news = read_source("src/i2pcontrol/news.rs");
    let main = read_source("src/main.rs");

    assert!(!handler.contains("reqwest"));
    assert!(!handler.contains("RouterNewsSource::start"));
    assert!(news.contains("Policy::none"));
    assert!(news.contains("MAX_COMPRESSED_BYTES"));
    assert!(news.contains("MAX_RENDERED_BYTES"));
    assert!(news.contains("MAX_ENTRIES"));
    assert!(news.contains("MAX_STALENESS"));
    assert!(news.contains(".chunk()"));
    assert!(main.contains("#[cfg(feature = \"i2pcontrol\")]"));
}

#[test]
fn network_error_rows_require_an_authoritative_owner() {
    let handler = read_source("src/i2pcontrol/router_info_handler.rs");
    let events = std::fs::read_to_string(workspace_root().join("emissary-core/src/events.rs"))
        .expect("core events source must exist");
    let inspection =
        std::fs::read_to_string(workspace_root().join("emissary-core/src/inspection.rs"))
            .expect("core inspection source must exist");

    for key in [
        rpc::router_info_keys::P170_NET_ERROR,
        rpc::router_info_keys::P170_NET_ERROR_V6,
    ] {
        let row = rpc::router_info_keys::PROPOSAL_170_CONTRACT
            .iter()
            .find(|field| field.key == key)
            .expect("network-error selector must remain in the canonical contract");
        assert!(matches!(
            row.source,
            rpc::router_info_keys::SourceDisposition::Available { .. }
        ));
        assert_eq!(row.source.owner(), "network-error-state");
        assert_eq!(
            row.serializer,
            if key == rpc::router_info_keys::P170_NET_ERROR {
                "serialize_network_error"
            } else {
                "serialize_network_error_v6"
            }
        );
    }

    assert!(handler.contains("fn network_error_code"));
    assert!(handler.contains("NetworkErrorReason::NoError"));
    assert!(events.contains("set_ipv4_network_error"));
    assert!(events.contains("set_ipv6_network_error"));
    assert!(inspection.contains("enum NetworkErrorReason"));
    for core_source in [&events, &inspection] {
        assert!(!core_source.contains("Proposal 170"));
        assert!(!core_source.contains("jsonrpc"));
        assert!(!core_source.contains("i2p.router.net.error"));
    }
}

#[test]
fn m053_composes_live_peer_directory_without_startup_snapshot() {
    let main = read_source("src/main.rs");
    assert!(main.contains("LivePeerDirectorySource::new"));
    assert!(!main.contains("inspection_snapshot"));
    assert!(!main.contains("CoreSnapshot"));
}

#[test]
fn transport_inspection_handle_contains_only_owned_snapshot_state() {
    let src = std::fs::read_to_string(workspace_root().join("emissary-core/src/inspection.rs"))
        .expect("core inspection source must exist");
    let start = src
        .find("pub struct TransportInspection {")
        .expect("transport inspection handle must exist");
    let end = src[start..]
        .find("\n}\n\nimpl TransportInspection")
        .map(|offset| start + offset);
    let fields = &src[start..end.expect("transport inspection handle must be closed")];
    assert!(fields.contains("snapshot: Arc<RwLock<TransportInspectionSnapshot>>"));
    for forbidden in [
        "Socket",
        "Session",
        "Sender",
        "Receiver",
        "EventHandle",
        "RouterContext",
        "PrivateKey",
    ] {
        assert!(
            !fields.contains(forbidden),
            "live/control type leaked into handle: {forbidden}"
        );
    }
}

#[test]
fn transport_peer_inspection_contains_only_sanitized_facts() {
    let src = std::fs::read_to_string(workspace_root().join("emissary-core/src/inspection.rs"))
        .expect("core inspection source must exist");
    let start = src
        .find("pub struct TransportPeerInspection {")
        .expect("peer inspection DTO must exist");
    let end = src[start..].find("\n}\n").map(|offset| start + offset);
    let fields = &src[start..end.expect("peer inspection DTO must be closed")];
    for forbidden in [
        "Socket",
        "Session",
        "Sender",
        "Receiver",
        "EventHandle",
        "RouterContext",
        "Key",
        "Channel",
        "Private",
    ] {
        assert!(
            !fields.contains(forbidden),
            "sensitive type leaked into peer DTO: {forbidden}"
        );
    }
    for required in [
        "peer_id",
        "inbound",
        "connected",
        "bytes_received",
        "bytes_sent",
    ] {
        assert!(
            fields.contains(required),
            "peer DTO lost required neutral fact: {required}"
        );
    }
}

#[test]
fn tunnel_inspection_contains_only_bounded_public_facts() {
    let src = std::fs::read_to_string(workspace_root().join("emissary-core/src/inspection.rs"))
        .expect("core inspection source must exist");
    let start = src
        .find("pub struct TunnelInspectionEntry {")
        .expect("tunnel inspection DTO must exist");
    let end = src[start..]
        .find("\n}\n")
        .map(|offset| start + offset)
        .expect("tunnel inspection DTO must be closed");
    let fields = &src[start..start + end];
    for forbidden in [
        "TunnelPool<",
        "InboundTunnel",
        "OutboundTunnel",
        "TransitTunnel",
        "RouterContext",
        "PrivateKey",
        "Receiver",
        "Sender",
    ] {
        assert!(
            !fields.contains(forbidden),
            "live/control type leaked into tunnel DTO: {forbidden}"
        );
    }
    for required in ["pool_id", "tunnel_id", "pool_kind", "direction"] {
        assert!(
            fields.contains(required),
            "tunnel DTO lost required neutral fact: {required}"
        );
    }
}

#[test]
fn production_adapter_does_not_silently_truncate() {
    // Log snapshot and netdb use real sources; empty/zero is a truthful
    // result when the source reports it, not a fabrication.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let ri = make_production_router_info();
    rt.block_on(async {
        let snap = ri.log_snapshot().await.unwrap();
        assert!(snap.entries.is_empty());
        // NetDb is unavailable (not yet wired), not zero
        assert!(matches!(
            ri.netdb_snapshot().await,
            Err(InspectionError::Unavailable { .. })
        ));
    });
}

#[test]
fn production_address_book_adapter_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ProductionAddressBookControl>();
}

#[test]
fn production_tunnel_manager_adapter_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    let tm = ProductionTunnelManagerControl::new(
        std::env::temp_dir().join("emissary-i2pcontrol-static-guard-tm"),
    )
    .unwrap();
    assert_send_sync::<ProductionTunnelManagerControl>();
    let _ = tm;
}

#[test]
fn production_control_plane_adapter_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    let metrics: Arc<dyn EventMetrics> = Arc::new(NullMetrics);
    let cp = ProductionControlPlane::new("test".to_string(), "test".to_string(), metrics);
    assert_send_sync::<ProductionControlPlane>();
    let _ = cp;
}

// --- M008 static guards ---

/// Guard: no enabled production path falls back to fake adapters.
///
/// If any production code contains "falling back to fake", this test fails.
/// This catches defects where init_server logs a warning and continues
/// with a fake adapter instead of failing startup.
#[test]
fn no_fallback_to_fake_in_production() {
    let files = &["src/i2pcontrol/server.rs", "src/i2pcontrol/production.rs"];
    for f in files {
        let src = read_source(f);
        // Split at #[cfg(test)] so test-only code is excluded
        let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
        assert!(
            !non_test.contains("falling back to fake"),
            "Production code {f} must not contain 'falling back to fake'"
        );
    }
}

/// Guard: no temporary fallback tunnel directory in production code.
///
/// Catches the defect where init_server creates a temp fallback directory
/// instead of failing.
#[test]
fn no_temp_fallback_tunnel_dir() {
    let files = &["src/i2pcontrol/server.rs", "src/i2pcontrol/production.rs"];
    for f in files {
        let src = read_source(f);
        let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
        assert!(
            !non_test.contains("emissary-i2pcontrol-tunnels-fallback"),
            "Production code {f} must not contain temp fallback tunnel directory"
        );
    }
}

/// Guard: no production construction of Fake* adapters in init_server.
///
/// Catches the defect where init_server installs fake adapters as
/// production defaults.
#[test]
fn no_production_fake_adapter_construction() {
    let src = read_source("src/i2pcontrol/server.rs");
    // Only check non-test code
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    assert!(
        !non_test.contains("FakeControlPlane::new()"),
        "Production init_server must not construct FakeControlPlane"
    );
    assert!(
        !non_test.contains("FakeAddressBookControl::new()"),
        "Production init_server must not construct FakeAddressBookControl"
    );
    assert!(
        !non_test.contains("FakeTunnelManagerControl::new()"),
        "Production init_server must not construct FakeTunnelManagerControl"
    );
    assert!(
        !non_test.contains("FakeRouterInfoControl::new()"),
        "Production init_server must not construct FakeRouterInfoControl"
    );
}

/// Guard: no second ProductionTunnelManagerControl::new() in RouterInfo wiring.
///
/// Catches the defect where init_server opens a second tunnel store
/// instance for RouterInfo instead of sharing the one already loaded.
#[test]
fn no_duplicate_tunnel_manager_in_init_server() {
    let src = read_source("src/i2pcontrol/server.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    // Count occurrences of ProductionTunnelManagerControl::new outside test code
    let count = non_test.matches("ProductionTunnelManagerControl::new").count();
    assert!(
        count <= 1,
        "Production init_server must construct ProductionTunnelManagerControl exactly once, found {count}"
    );
}

/// Guard: no error-suppressing unwrap_or_default or unwrap_or(0) in
/// production state query helpers.
///
/// Catches the defect where tunnel_list, tunnel_get, address_book_list,
/// address_book_lookup, address_book_subscriptions, or
/// address_book_configuration silently turn errors into empty/zero state.
#[test]
fn no_error_suppressing_helpers() {
    let src = read_source("src/i2pcontrol/server.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    assert!(
        !non_test.contains("unwrap_or_default()"),
        "Production state helpers must not use unwrap_or_default() to suppress errors"
    );
}

/// Guard: ControlPlane no longer includes tunnel methods.
///
/// Catches the defect where ControlPlane still has tunnel_list, tunnel_get,
/// or is_tunnel_type_supported, which creates dual-path access.
#[test]
fn control_plane_has_no_tunnel_methods() {
    let src = read_source("src/i2pcontrol/control_plane.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    // Check the trait definition (between "pub trait ControlPlane" and the closing "}")
    if let Some(trait_start) = non_test.find("pub trait ControlPlane") {
        if let Some(trait_body_end) = non_test[trait_start..].find("\n}") {
            let trait_body = &non_test[trait_start..trait_start + trait_body_end + 2];
            assert!(
                !trait_body.contains("fn tunnel_list"),
                "ControlPlane must not have tunnel_list method"
            );
            assert!(
                !trait_body.contains("fn tunnel_get"),
                "ControlPlane must not have tunnel_get method"
            );
            assert!(
                !trait_body.contains("fn is_tunnel_type_supported"),
                "ControlPlane must not have is_tunnel_type_supported method"
            );
        }
    }
}

// --- M009 static guards ---

/// Guard: no fabricated NetDbSnapshot::default() in production RouterInfo.
///
/// Catches the defect where netdb_snapshot returns a default struct
/// instead of returning Unavailable when the source is not wired.
#[test]
fn no_fabricated_netdb_default_in_production() {
    let src = read_source("src/i2pcontrol/production.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    assert!(
        !non_test.contains("NetDbSnapshot::default()"),
        "Production RouterInfo must not return fabricated NetDbSnapshot::default()"
    );
}

/// Guard: no fabricated TcpSnapshot::default() in production RouterInfo.
///
/// Catches the defect where tcp_snapshot returns a default struct
/// instead of returning Unavailable when the source is not wired.
#[test]
fn no_fabricated_tcp_default_in_production() {
    let src = read_source("src/i2pcontrol/production.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    assert!(
        !non_test.contains("TcpSnapshot::default()"),
        "Production RouterInfo must not return fabricated TcpSnapshot::default()"
    );
}

/// Guard: no Vec::new() used as an unavailable source response.
///
/// Catches the defect where empty vectors are returned for sources
/// that are not wired, making them indistinguishable from legitimate
/// empty peer lists.
#[test]
fn no_vec_new_as_unavailable_response() {
    let src = read_source("src/i2pcontrol/production.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    // Only check within RouterInfoControl impl methods (between
    // "impl RouterInfoControl" and the closing "}").
    if let Some(impl_start) = non_test.find("impl RouterInfoControl") {
        if let Some(impl_end) = non_test[impl_start..].find("\n}") {
            let impl_body = &non_test[impl_start..impl_start + impl_end + 2];
            assert!(
                !impl_body.contains("Vec::new()"),
                "Production RouterInfo must not use Vec::new() for unavailable sources; use Err(Unavailable) instead"
            );
        }
    }
}

/// Guard: no PeerLimits::default() in production RouterInfo.
///
/// Catches the defect where peer_limits returns a default struct
/// instead of returning Unavailable when the source is not wired.
#[test]
fn no_fabricated_peer_limits_default_in_production() {
    let src = read_source("src/i2pcontrol/production.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    assert!(
        !non_test.contains("PeerLimits::default()"),
        "Production RouterInfo must not return fabricated PeerLimits::default()"
    );
}

/// Guard: no unwrap_or(0) error suppression in production RouterInfo.
///
/// Catches the defect where tunnel query failures are silently
/// converted to zero counts.
#[test]
fn no_error_suppressing_unwrap_or_zero_in_router_info() {
    let src = read_source("src/i2pcontrol/production.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    if let Some(impl_start) = non_test.find("impl RouterInfoControl") {
        if let Some(impl_end) = non_test[impl_start..].find("\n}") {
            let impl_body = &non_test[impl_start..impl_start + impl_end + 2];
            assert!(
                !impl_body.contains("unwrap_or(0)"),
                "Production RouterInfo must not use unwrap_or(0) to suppress errors"
            );
        }
    }
}

/// Guard: no fabricated RecentTransitTraffic::default() in production RouterInfo.
///
/// Catches the defect where recent_transit_traffic returns a default struct
/// with all-zero counters instead of returning Unavailable when the rolling
/// window source is not wired.
#[test]
fn no_fabricated_recent_transit_default_in_production() {
    let src = read_source("src/i2pcontrol/production.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    if let Some(impl_start) = non_test.find("impl RouterInfoControl") {
        if let Some(impl_end) = non_test[impl_start..].find("\n}") {
            let impl_body = &non_test[impl_start..impl_start + impl_end + 2];
            assert!(
                !impl_body.contains("RecentTransitTraffic::default()"),
                "Production RouterInfo must not return fabricated RecentTransitTraffic::default()"
            );
        }
    }
}

/// Guard: no fabricated active: true hardcoded in UDP snapshot.
///
/// Catches the defect where udp_snapshot hardcodes active: true instead
/// of deriving from actual transport metrics.
#[test]
fn no_hardcoded_udp_active_true_in_production() {
    let src = read_source("src/i2pcontrol/production.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    if let Some(method_start) = non_test.find("async fn udp_snapshot") {
        if let Some(method_end) = non_test[method_start..].find("\n    async fn") {
            let method_body = &non_test[method_start..method_start + method_end];
            assert!(
                !method_body.contains("active: true"),
                "Production RouterInfo must not hardcode active: true in UDP snapshot"
            );
        }
    }
}

// --- M011 static guards ---

/// Guard: no startup-only I2PTunnel population in main.rs.
///
/// Catches the defect where I2PTunnel inventory is populated once at
/// startup and never updated after TunnelManager mutations. The handler
/// now queries TunnelManagerControl at request time.
#[test]
fn no_startup_only_i2ptunnel_population() {
    let src = read_source("src/main.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    assert!(
        !non_test.contains("observe_i2ptunnel_inventory"),
        "main.rs must not contain startup-only I2PTunnel inventory population; the handler queries TunnelManagerControl at request time"
    );
}

/// Guard: production composition must pass the parsed startup inventory into
/// the real tunnel manager, rather than constructing a disconnected fake or
/// relying on a registry-only snapshot.
#[test]
fn production_composition_uses_shared_startup_inventory() {
    let main = read_source("src/main.rs");
    let server = read_source("src/i2pcontrol/server.rs");
    let production = read_source("src/i2pcontrol/production.rs");
    assert!(main.contains("StartupTunnelInventory::from_configs"));
    assert!(main.contains("with_startup_tunnel_inventory"));
    assert!(server.contains("new_with_startup_inventory"));
    assert!(production.contains("startup and persisted tunnel definitions"));
    assert!(production.contains("StartupManaged"));
}

/// Guard: no unconditional SAM sessions placeholder in production handler.
///
/// Catches the defect where resolve_sam always returns "sessions": {}
/// without checking the observation source. The SAM session snapshot
/// requires a bounded accessor at the canonical SamServer.
#[test]
fn no_unconditional_sam_sessions_placeholder() {
    let src = read_source("src/i2pcontrol/client_services.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    // The SAM resolver should not have an unconditional "sessions": {}
    // that ignores the canonical bounded snapshot.
    if let Some(sam_start) = non_test.find("fn resolve_sam") {
        if let Some(sam_end) = non_test[sam_start..].find("\nfn ") {
            let sam_body = &non_test[sam_start..sam_start + sam_end];
            // The function should retain the shared session bound.
            assert!(
                sam_body.contains("SAM_SESSION_OBSERVATION_LIMIT"),
                "resolve_sam must check the shared SAM session observation bound"
            );
        }
    }
}

/// Guard: HTTP/SOCKS Configured/Starting states must not report enabled.
///
/// Catches the defect where Configured or Starting proxy states report
/// enabled: true before a listener has actually bound.
#[test]
fn configured_starting_proxy_not_reported_as_enabled() {
    let src = read_source("src/i2pcontrol/client_services.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    // Both resolve_httpproxy and resolve_socks should map Configured/Starting
    // to "enabled": false, not "enabled": entry.metadata.enabled.
    for fn_name in ["resolve_httpproxy", "resolve_socks"] {
        if let Some(fn_start) = non_test.find(&format!("fn {fn_name}")) {
            if let Some(fn_end) = non_test[fn_start..].find("\nfn ") {
                let fn_body = &non_test[fn_start..fn_start + fn_end];
                // In the Configured/Starting branch, enabled should be literal false
                if fn_body.contains("Configured") && fn_body.contains("Starting") {
                    assert!(
                        !fn_body.contains("entry.metadata.enabled"),
                        "{fn_name} must not use entry.metadata.enabled for Configured/Starting; enabled must be false until Listening"
                    );
                }
            }
        }
    }
}

/// Guard: handler queries TunnelManagerControl for I2PTunnel, not registry.
///
/// Catches the defect where assemble_response reads I2PTunnel from the
/// service registry snapshot instead of querying the live tunnel manager.
#[test]
fn handler_uses_live_tunnel_manager_for_i2ptunnel() {
    let src = read_source("src/i2pcontrol/client_services.rs");
    let non_test = src.split("#[cfg(test)]").next().unwrap_or(&src);
    // assemble_response should accept a tunnel_manager parameter
    assert!(
        non_test.contains("tunnel_manager: &dyn TunnelManagerControl"),
        "assemble_response must accept a tunnel_manager parameter for live I2PTunnel queries"
    );
    // resolve_i2ptunnel_live should query tunnel_manager.list()
    assert!(
        non_test.contains("resolve_i2ptunnel_live"),
        "Handler must use resolve_i2ptunnel_live for live tunnel queries"
    );
    // The old resolve_i2ptunnel (registry-based) should not exist
    assert!(
        !non_test.contains("fn resolve_i2ptunnel("),
        "Old registry-based resolve_i2ptunnel must be removed"
    );
}

/// Guard: runtime-disabled feature builds retain the historical startup path.
///
/// This is deliberately source-structural because setup_router owns the
/// application configuration and starts real router/SAM resources. It catches
/// the M109 regression where `#[cfg(feature = "i2pcontrol")]` selected the
/// controlled managers before the runtime `enabled` value was known.
#[test]
fn runtime_disabled_does_not_select_controlled_startup_path() {
    let main = read_source("src/main.rs");
    let enabled = main
        .find("let i2pcontrol_enabled =")
        .expect("runtime enablement must be computed");
    let inventory = main
        .find("StartupTunnelInventory::from_configs")
        .expect("startup inventory must remain composed");
    assert!(
        enabled < inventory,
        "runtime enablement must precede M109 inventory construction"
    );

    let composition = &main[enabled..];
    assert!(
        composition.contains("if i2pcontrol_enabled")
            && composition.contains("ClientTunnelManager::new_with_lifecycle")
            && composition.contains("ClientTunnelManager::new(client_tunnels"),
        "controlled and historical startup client constructors must be runtime-selected"
    );
    assert!(
        composition.contains("ServerTunnelManager::new_with_lifecycle")
            && composition.contains("ServerTunnelManager::new(\n"),
        "controlled and historical startup server constructors must be runtime-selected"
    );
}
