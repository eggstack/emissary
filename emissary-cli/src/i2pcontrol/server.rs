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

use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as HyperBuilder,
    service::TowerToHyperService,
};
use tokio::{net::TcpListener, sync::Semaphore};
use tokio_rustls::TlsAcceptor;
use tower::ServiceExt;
use tower_http::limit::RequestBodyLimitLayer;
use tracing;

use super::{
    auth::{self, AuthThrottle, TokenService},
    control_plane::{AddressBookControl, ControlPlane, TunnelManagerControl},
    errors::I2pControlError,
    production::{
        EventMetrics, ProductionAddressBookControl, ProductionControlPlane,
        ProductionRouterInfoControl, ProductionTunnelManagerControl, StartupTunnelInventory,
    },
    router_info::{ActivePeerSource, PeerDirectorySource, RouterInfoControl, TunnelSource},
    rpc::{
        self, AuthenticateParams, AuthenticateResult, JsonRpcErrorResponse, JsonRpcRequest,
        JsonRpcSuccess, RequestId,
    },
    service_registry::{ServiceRegistry, ServiceSnapshot},
    tls::TlsConfig,
};

use crate::i2pcontrol::address_book_runtime::RuntimeAddressBookHandle;

use emissary_core::crypto::base64_encode;

use super::sam_observer::SamSessionObservationHandle;

const LOG_TARGET: &str = "emissary::i2pcontrol::server";

/// Maximum request body size (1 MiB).
const MAX_BODY_SIZE: usize = 1024 * 1024;

/// Maximum concurrent in-flight requests.
const MAX_CONCURRENT_REQUESTS: usize = 64;

/// Timeout for TLS handshake completion (seconds).
const TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);

/// Total request deadline from connection accept to response completion (seconds).
const REQUEST_DEADLINE: Duration = Duration::from_secs(60);

/// Maximum concurrent TLS connection tasks spawned by the accept loop.
///
/// Bounds the number of in-progress TLS handshakes plus active HTTP
/// connections to prevent unbounded task creation from rapid reconnects.
/// This is independent of `MAX_CONCURRENT_REQUESTS` which limits
/// in-flight JSON-RPC dispatch within already-established connections.
const MAX_CONNECTION_TASKS: usize = 128;

/// I2PControl server configuration.
#[derive(Debug, Clone)]
pub struct I2pControlConfig {
    /// Whether I2PControl is enabled.
    pub enabled: bool,
    /// Bind address.
    pub bind: SocketAddr,
    /// Password for authentication.
    pub password: String,
    /// TLS configuration.
    pub tls: TlsConfig,
}

impl I2pControlConfig {
    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), I2pControlError> {
        if self.enabled && self.password.is_empty() {
            return Err(I2pControlError::Config(
                "I2PControl enabled but no password configured".into(),
            ));
        }

        // Warn on non-loopback binding
        if self.enabled && !self.bind.ip().is_loopback() {
            tracing::warn!(
                target: LOG_TARGET,
                bind = %self.bind,
                "I2PControl bound to non-loopback address; ensure this is intentional",
            );
        }

        Ok(())
    }
}

/// Shared application state for the I2PControl server.
///
/// Production state is constructed via [`new_production`] with all required
/// dependencies supplied explicitly. Test state is constructed via
/// [`new_test`] which installs fake adapters. The generic `new()` is
/// retained only for internal composition in `init_server`.
pub struct I2pControlState {
    token_service: TokenService,
    auth_throttle: AuthThrottle,
    #[allow(dead_code)]
    password: String,
    #[allow(dead_code)]
    control_plane: Arc<dyn ControlPlane>,
    address_book_control: Arc<dyn AddressBookControl>,
    tunnel_manager: Arc<dyn TunnelManagerControl>,
    router_info: Arc<dyn RouterInfoControl>,
    semaphore: Semaphore,
    /// Local router identity in Base64 (retained at startup, never re-read).
    router_id: String,
    /// Serialized local RouterInfo bytes (retained at startup).
    router_info_bytes: Vec<u8>,
    /// Base64 encoding of serialized RouterInfo.
    router_info_b64: String,
    /// Startup time for uptime calculation.
    startup_time: std::time::Instant,
    /// Shared log ring for I2PControl snapshot/clear.
    #[allow(dead_code)]
    log_ring: Arc<super::observability::LogRing>,
    /// Passive client-service registry for ClientServicesInfo.
    service_registry: ServiceRegistry,

    /// Canonical bounded SAM observation source, absent when SAM is disabled.
    sam_session_observation: Option<SamSessionObservationHandle>,
}

impl I2pControlState {
    /// Create production state from required, already-validated dependencies.
    ///
    /// All adapter objects are constructed and loaded by the caller before
    /// this call. The state takes ownership of the `Arc` clones and never
    /// falls back to fake adapters.
    #[allow(dead_code)]
    pub fn new_production(password: String, controls: ProductionControls) -> Self {
        Self::new_production_with_sam_observation(password, controls, None)
    }

    /// Create production state with the canonical bounded SAM observation source.
    pub fn new_production_with_sam_observation(
        password: String,
        controls: ProductionControls,
        sam_session_observation: Option<SamSessionObservationHandle>,
    ) -> Self {
        let log_ring = Arc::new(super::observability::LogRing::default());
        Self {
            token_service: TokenService::new(),
            auth_throttle: AuthThrottle::new(),
            password,
            control_plane: controls.control_plane,
            address_book_control: controls.address_books,
            tunnel_manager: controls.tunnels,
            router_info: controls.router_info,
            semaphore: Semaphore::new(MAX_CONCURRENT_REQUESTS),
            router_id: String::new(),
            router_info_bytes: Vec::new(),
            router_info_b64: String::new(),
            startup_time: std::time::Instant::now(),
            log_ring,
            service_registry: controls.service_registry,
            sam_session_observation,
        }
    }

    /// Create test state with fake adapters.
    ///
    /// This is the only constructor that installs fake implementations.
    /// Available only in test builds.
    #[cfg(test)]
    pub fn new_test(password: String) -> Self {
        use super::{
            control_plane::{FakeAddressBookControl, FakeControlPlane, FakeTunnelManagerControl},
            router_info::FakeRouterInfoControl,
        };
        let log_ring = Arc::new(super::observability::LogRing::default());
        Self {
            token_service: TokenService::new(),
            auth_throttle: AuthThrottle::new(),
            password,
            control_plane: Arc::new(FakeControlPlane::new()),
            address_book_control: Arc::new(FakeAddressBookControl::new()),
            tunnel_manager: Arc::new(FakeTunnelManagerControl::new()),
            router_info: Arc::new(FakeRouterInfoControl::new()),
            semaphore: Semaphore::new(MAX_CONCURRENT_REQUESTS),
            router_id: String::new(),
            router_info_bytes: Vec::new(),
            router_info_b64: String::new(),
            startup_time: std::time::Instant::now(),
            log_ring,
            service_registry: ServiceRegistry::new(),
            sam_session_observation: Some(SamSessionObservationHandle::empty_for_test()),
        }
    }

    /// Create test state with fake adapters (public for integration tests).
    ///
    /// This constructor is identical to `new_test` but available outside
    /// `cfg(test)` so integration tests can exercise the handler.
    #[allow(dead_code)]
    pub fn new_for_test(password: String) -> Self {
        use super::{
            control_plane::{FakeAddressBookControl, FakeControlPlane, FakeTunnelManagerControl},
            router_info::FakeRouterInfoControl,
        };
        let log_ring = Arc::new(super::observability::LogRing::default());
        Self {
            token_service: TokenService::new(),
            auth_throttle: AuthThrottle::new(),
            password,
            control_plane: Arc::new(FakeControlPlane::new()),
            address_book_control: Arc::new(FakeAddressBookControl::new()),
            tunnel_manager: Arc::new(FakeTunnelManagerControl::new()),
            router_info: Arc::new(FakeRouterInfoControl::new()),
            semaphore: Semaphore::new(MAX_CONCURRENT_REQUESTS),
            router_id: String::new(),
            router_info_bytes: Vec::new(),
            router_info_b64: String::new(),
            startup_time: std::time::Instant::now(),
            log_ring,
            service_registry: ServiceRegistry::new(),
            sam_session_observation: Some(SamSessionObservationHandle::empty_for_test()),
        }
    }

    /// Get a clone of the shared log ring (used by the production router
    /// info adapter for I2PControl log snapshot/clear).
    #[allow(dead_code)]
    pub fn log_ring_arc(&self) -> Arc<super::observability::LogRing> {
        Arc::clone(&self.log_ring)
    }

    /// Get a reference to the service registry.
    #[allow(dead_code)]
    pub fn service_registry(&self) -> &ServiceRegistry {
        &self.service_registry
    }

    /// Get a clone of the service registry (cheap Arc clone). Used by the
    /// application composition root to share the registry with producers
    /// (proxy tasks, listener snapshot readouts, tunnel query tasks).
    #[allow(dead_code)]
    pub fn service_registry_clone(&self) -> ServiceRegistry {
        self.service_registry.clone()
    }

    /// Take a snapshot from the service registry.
    #[allow(dead_code)]
    pub fn service_snapshot(&self) -> ServiceSnapshot {
        self.service_registry.snapshot()
    }

    /// Replace the service registry (for testing or composition).
    ///
    /// Producers in the composition root (proxy tasks, listener snapshot
    /// readouts) should allocate their handles from the registry they
    /// already hold a clone of — only the I2PControl-facing half is
    /// replaced here.
    #[allow(dead_code)]
    pub fn set_service_registry(&mut self, registry: ServiceRegistry) {
        self.service_registry = registry;
    }

    /// Get a reference to the token service.
    pub fn token_service(&self) -> &TokenService {
        &self.token_service
    }

    pub(crate) fn auth_throttle(&self) -> &AuthThrottle {
        &self.auth_throttle
    }

    /// Get a reference to the router info control.
    pub fn router_info(&self) -> &dyn RouterInfoControl {
        &*self.router_info
    }

    /// Get a reference to the address book control.
    pub fn address_book_control(&self) -> &dyn AddressBookControl {
        &*self.address_book_control
    }

    /// Get a reference to the tunnel manager control.
    ///
    /// Used by ClientServicesInfo to query live I2PTunnel inventory
    /// at request time rather than relying on a startup-only snapshot.
    pub fn tunnel_manager(&self) -> &dyn TunnelManagerControl {
        &*self.tunnel_manager
    }

    /// Get the canonical bounded SAM observation source, if SAM is enabled.
    pub fn sam_session_observation(&self) -> Option<&SamSessionObservationHandle> {
        self.sam_session_observation.as_ref()
    }

    /// Replace the address book control plane (for testing).
    #[allow(dead_code)]
    pub fn set_address_book_control(&mut self, control: Box<dyn AddressBookControl>) {
        self.address_book_control = control.into();
    }

    /// Replace the tunnel manager control plane (for testing).
    #[allow(dead_code)]
    pub fn set_tunnel_manager(&mut self, control: Box<dyn TunnelManagerControl>) {
        self.tunnel_manager = control.into();
    }

    /// Replace the router info inspection control plane (for testing).
    #[allow(dead_code)]
    pub fn set_router_info(&mut self, control: Box<dyn RouterInfoControl>) {
        self.router_info = control.into();
    }

    /// Set startup-retained values (router identity, serialized RI).
    ///
    /// These are retained once at startup and never re-read from disk.
    pub fn set_startup_values(
        &mut self,
        router_id: String,
        router_info_bytes: Vec<u8>,
        router_info_b64: String,
    ) {
        self.router_id = router_id;
        self.router_info_bytes = router_info_bytes;
        self.router_info_b64 = router_info_b64;
    }

    /// Get the local router identity (Base64).
    #[allow(dead_code)]
    pub fn router_id(&self) -> &str {
        &self.router_id
    }

    /// Get the serialized RouterInfo bytes.
    #[allow(dead_code)]
    pub fn router_info_bytes(&self) -> &[u8] {
        &self.router_info_bytes
    }

    /// Get the Base64-encoded serialized RouterInfo.
    #[allow(dead_code)]
    pub fn router_info_b64(&self) -> &str {
        &self.router_info_b64
    }

    /// Get the uptime since server startup.
    #[allow(dead_code)]
    pub fn uptime(&self) -> std::time::Duration {
        self.startup_time.elapsed()
    }

    /// List all tunnel definitions.
    pub async fn tunnel_list(
        &self,
    ) -> Result<Vec<crate::i2pcontrol::domain::tunnel::TunnelDefinition>, String> {
        self.tunnel_manager.list().await
    }

    /// Get a tunnel definition by name.
    pub async fn tunnel_get(
        &self,
        name: &str,
    ) -> Result<Option<crate::i2pcontrol::domain::tunnel::TunnelDefinition>, String> {
        self.tunnel_manager.get(name).await
    }

    /// Create a new tunnel definition.
    pub async fn tunnel_create(
        &self,
        definition: crate::i2pcontrol::domain::tunnel::TunnelDefinition,
    ) -> Result<(), String> {
        self.tunnel_manager.create(definition).await
    }

    /// Update an existing tunnel definition.
    pub async fn tunnel_update(
        &self,
        name: &str,
        definition: crate::i2pcontrol::domain::tunnel::TunnelDefinition,
        new_name: Option<crate::i2pcontrol::domain::tunnel::TunnelName>,
    ) -> Result<bool, String> {
        self.tunnel_manager.update(name, definition, new_name).await
    }

    /// Delete a tunnel definition.
    pub async fn tunnel_delete(&self, name: &str) -> Result<bool, String> {
        self.tunnel_manager.delete(name).await
    }

    /// Start a tunnel.
    pub async fn tunnel_start(&self, name: &str) -> Result<String, String> {
        self.tunnel_manager.start(name).await
    }

    /// Stop a tunnel.
    pub async fn tunnel_stop(&self, name: &str) -> Result<String, String> {
        self.tunnel_manager.stop(name).await
    }

    /// Restart a tunnel.
    pub async fn tunnel_restart(&self, name: &str) -> Result<String, String> {
        self.tunnel_manager.restart(name).await
    }

    /// List entries in the specified address book.
    pub async fn address_book_list(
        &self,
        book_type: crate::i2pcontrol::domain::address_book::AdministrativeAddressBookType,
    ) -> Result<Vec<crate::i2pcontrol::domain::address_book::AddressBookEntry>, String> {
        self.address_book_control.list(book_type).await
    }

    /// Look up an entry in the specified address book.
    pub async fn address_book_lookup(
        &self,
        book_type: crate::i2pcontrol::domain::address_book::AdministrativeAddressBookType,
        hostname: &str,
    ) -> Result<Option<crate::i2pcontrol::domain::address_book::AddressBookEntry>, String> {
        self.address_book_control.lookup(book_type, hostname).await
    }

    /// Add an entry to the specified address book.
    pub async fn address_book_add(
        &self,
        book_type: crate::i2pcontrol::domain::address_book::AdministrativeAddressBookType,
        entry: crate::i2pcontrol::domain::address_book::AddressBookEntry,
    ) -> Result<(), String> {
        self.address_book_control.add(book_type, entry).await
    }

    /// Update an entry in the specified address book.
    pub async fn address_book_update(
        &self,
        book_type: crate::i2pcontrol::domain::address_book::AdministrativeAddressBookType,
        entry: crate::i2pcontrol::domain::address_book::AddressBookEntry,
    ) -> Result<bool, String> {
        self.address_book_control.update(book_type, entry).await
    }

    /// Delete an entry from the specified address book.
    pub async fn address_book_delete(
        &self,
        book_type: crate::i2pcontrol::domain::address_book::AdministrativeAddressBookType,
        hostname: &str,
    ) -> Result<bool, String> {
        self.address_book_control.delete(book_type, hostname).await
    }

    /// Delete all entries from the specified address book.
    pub async fn address_book_delete_all(
        &self,
        book_type: crate::i2pcontrol::domain::address_book::AdministrativeAddressBookType,
    ) -> Result<bool, String> {
        self.address_book_control.delete_all(book_type).await
    }

    /// Get the current subscription set.
    #[allow(dead_code)]
    pub async fn address_book_subscriptions(
        &self,
    ) -> Result<crate::i2pcontrol::domain::address_book::SubscriptionSet, String> {
        self.address_book_control.subscriptions().await
    }

    /// Set the subscription set atomically.
    pub async fn address_book_set_subscriptions(
        &self,
        subscriptions: crate::i2pcontrol::domain::address_book::SubscriptionSet,
    ) -> Result<(), String> {
        self.address_book_control.set_subscriptions(subscriptions).await
    }

    /// Get the address book configuration.
    #[allow(dead_code)]
    pub async fn address_book_configuration(
        &self,
    ) -> Result<crate::i2pcontrol::domain::address_book::AddressBookConfiguration, String> {
        self.address_book_control.configuration().await
    }

    /// Set the address book configuration atomically.
    pub async fn address_book_set_configuration(
        &self,
        configuration: crate::i2pcontrol::domain::address_book::AddressBookConfiguration,
    ) -> Result<(), String> {
        self.address_book_control.set_configuration(configuration).await
    }
}

/// Production dependencies for I2PControl state construction.
///
/// All fields are required. The caller must construct and load each adapter
/// before passing it here. This ensures that the production composition root
/// cannot silently substitute fake, empty, zeroed, or separately initialized
/// state.
pub struct ProductionControls {
    pub address_books: Arc<dyn AddressBookControl>,
    pub tunnels: Arc<dyn TunnelManagerControl>,
    pub router_info: Arc<dyn RouterInfoControl>,
    pub control_plane: Arc<dyn ControlPlane>,
    pub service_registry: ServiceRegistry,
}

/// A bound and initialized I2PControl server, ready to serve requests.
///
/// Created by `init_server` which performs validation, TLS setup, and port binding.
/// Passed to `serve` which runs the TLS accept loop under structured cancellation.
pub struct ServerInstance {
    listener: TcpListener,
    tls_acceptor: TlsAcceptor,
    state: Arc<I2pControlState>,
    bind: SocketAddr,
    connection_semaphore: Arc<Semaphore>,
}

impl ServerInstance {
    /// Get a clone of the shared [`I2pControlState`].
    ///
    /// Used by the application composition root to access the service
    /// registry handle so additional producers (e.g. SAM session snapshot
    /// tasks) can be wired after [`init_server`] returns.
    #[allow(dead_code)]
    pub(crate) fn state_clone(&self) -> Arc<I2pControlState> {
        Arc::clone(&self.state)
    }

    /// Get the bound listener address.
    #[allow(dead_code)]
    pub fn bind(&self) -> SocketAddr {
        self.bind
    }

    /// Create a ServerInstance directly for integration testing.
    ///
    /// Bypasses `init_server` to allow tests to supply pre-built state,
    /// ephemeral listeners, and generated TLS material.
    #[allow(dead_code)]
    pub fn new_for_test(
        listener: TcpListener,
        tls_acceptor: TlsAcceptor,
        state: Arc<I2pControlState>,
        bind: SocketAddr,
    ) -> Self {
        Self {
            listener,
            tls_acceptor,
            state,
            bind,
            connection_semaphore: Arc::new(Semaphore::new(MAX_CONNECTION_TASKS)),
        }
    }

    /// Create a test server with an explicit pre-spawn connection bound.
    ///
    /// This keeps the production limit fixed while allowing integration tests
    /// to deterministically exercise saturation and permit restoration.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn new_for_test_with_connection_limit(
        listener: TcpListener,
        tls_acceptor: TlsAcceptor,
        state: Arc<I2pControlState>,
        bind: SocketAddr,
        connection_limit: usize,
    ) -> Self {
        assert!(connection_limit > 0, "connection limit must be positive");
        Self {
            listener,
            tls_acceptor,
            state,
            bind,
            connection_semaphore: Arc::new(Semaphore::new(connection_limit)),
        }
    }
}

/// Bundle of dependencies used to construct the production I2PControl server.
///
/// When production adapters are supplied, they replace the corresponding
/// fakes. The fakes are retained as defaults so that headless test
/// environments and unit tests can build a server without supplying real
/// router state.
pub struct ServerInitContext {
    /// Local router identity in Base64.
    pub router_id: String,
    /// Serialized local RouterInfo bytes.
    pub router_info_bytes: Vec<u8>,
    /// Event metrics source for bandwidth and tunnel build counters.
    ///
    /// When `None`, a default zeroed source is used.
    pub event_metrics: Option<Arc<dyn EventMetrics>>,
    /// Share ratio from the active configuration.
    pub share_ratio: f64,
    /// Configured inbound bandwidth limit in bytes/second.
    pub configured_bandwidth_in: u64,
    /// Configured outbound bandwidth limit in bytes/second.
    pub configured_bandwidth_out: u64,
    /// Pre-built service registry from the application composition root.
    ///
    /// When provided, `init_server` uses this registry instead of creating
    /// a new one. The composition root shares its clone of the same
    /// registry with proxy tasks and listener snapshot readouts.
    pub service_registry: Option<ServiceRegistry>,
    /// Shared log ring from the tracing-backed application logger.
    ///
    /// When `None`, a fresh default ring is created (suitable for tests).
    pub log_ring: Option<Arc<super::observability::LogRing>>,
    /// Canonical bounded SAM observation source, supplied by the core router.
    pub sam_session_observation: Option<SamSessionObservationHandle>,
    /// Whether the core router actually bound the SAM listener.
    pub sam_listener_enabled: bool,
    /// The dedicated Proposal 170 address-book control handle.
    pub address_book_handle: Option<Arc<RuntimeAddressBookHandle>>,
    /// Startup-configured generic tunnel definitions shared with production
    /// TunnelManager and the existing tunnel managers.
    pub startup_tunnel_inventory: Option<StartupTunnelInventory>,
    /// Existing router SAM TCP port used by the control-plane client backend.
    pub sam_tcp_port: Option<u16>,
    /// Canonical bounded public peer directory source.
    pub peer_directory: Option<Arc<dyn PeerDirectorySource>>,
    /// Canonical bounded current transport source.
    pub active_peer_source: Option<Arc<dyn ActivePeerSource>>,
    /// Canonical bounded current tunnel source.
    pub tunnel_source: Option<Arc<dyn TunnelSource>>,
}

impl ServerInitContext {
    /// Create a new init context with the required startup values and
    /// sensible defaults for optional dependencies.
    pub fn new(router_id: String, router_info_bytes: Vec<u8>) -> Self {
        Self {
            router_id,
            router_info_bytes,
            event_metrics: None,
            share_ratio: 0.0,
            configured_bandwidth_in: 0,
            configured_bandwidth_out: 0,
            service_registry: None,
            log_ring: None,
            sam_session_observation: None,
            sam_listener_enabled: false,
            address_book_handle: None,
            startup_tunnel_inventory: None,
            sam_tcp_port: None,
            peer_directory: None,
            active_peer_source: None,
            tunnel_source: None,
        }
    }

    /// Set the event metrics source.
    pub fn with_event_metrics(mut self, metrics: Arc<dyn EventMetrics>) -> Self {
        self.event_metrics = Some(metrics);
        self
    }

    /// Set the share ratio.
    pub fn with_share_ratio(mut self, ratio: f64) -> Self {
        self.share_ratio = ratio;
        self
    }

    /// Set the configured bandwidth limits.
    pub fn with_configured_bandwidth(mut self, inbound: u64, outbound: u64) -> Self {
        self.configured_bandwidth_in = inbound;
        self.configured_bandwidth_out = outbound;
        self
    }

    /// Inject a pre-built service registry from the application composition
    /// root. Producers in the composition root (proxy tasks, listener
    /// snapshot readouts, tunnel query tasks) share clones of this same
    /// registry.
    pub fn with_service_registry(mut self, registry: ServiceRegistry) -> Self {
        self.service_registry = Some(registry);
        self
    }

    /// Inject the tracing-backed application log ring.
    ///
    /// The ring is shared with the tracing subscriber so events recorded
    /// through the application logger appear in I2PControl log retrieval.
    pub fn with_log_ring(mut self, ring: Arc<super::observability::LogRing>) -> Self {
        self.log_ring = Some(ring);
        self
    }

    /// Inject the canonical bounded SAM observation source.
    pub fn with_sam_session_observation(mut self, handle: SamSessionObservationHandle) -> Self {
        self.sam_session_observation = Some(handle);
        self
    }

    /// Record whether the core router bound the SAM listener.
    pub fn with_sam_listener_enabled(mut self, enabled: bool) -> Self {
        self.sam_listener_enabled = enabled;
        self
    }

    /// Inject the dedicated Proposal 170 address-book control handle.
    pub fn with_address_book_handle(mut self, handle: Arc<RuntimeAddressBookHandle>) -> Self {
        self.address_book_handle = Some(handle);
        self
    }

    /// Inject the startup tunnel inventory built by the application
    /// composition root.
    pub fn with_startup_tunnel_inventory(mut self, inventory: StartupTunnelInventory) -> Self {
        self.startup_tunnel_inventory = Some(inventory);
        self
    }

    /// Inject the already-bound router SAM TCP port for generic client
    /// tunnel composition.
    pub fn with_sam_tcp_port(mut self, port: u16) -> Self {
        self.sam_tcp_port = Some(port);
        self
    }

    /// Inject the canonical live public peer directory source.
    pub fn with_peer_directory_source(mut self, source: Arc<dyn PeerDirectorySource>) -> Self {
        self.peer_directory = Some(source);
        self
    }

    /// Inject the canonical current transport source.
    pub fn with_active_peer_source(mut self, source: Arc<dyn ActivePeerSource>) -> Self {
        self.active_peer_source = Some(source);
        self
    }

    /// Inject the canonical current tunnel source.
    pub fn with_tunnel_source(mut self, source: Arc<dyn TunnelSource>) -> Self {
        self.tunnel_source = Some(source);
        self
    }
}

/// Initialize the I2PControl server: validate config, set up TLS, bind the port.
///
/// Returns a `ServerInstance` ready to serve, or an error if startup fails.
/// This function is synchronous-safe for calling from `setup_router` so that
/// bind/TLS/startup failures are surfaced as application errors.
///
/// # Fail-closed behavior
///
/// Directory creation, adapter construction, or store load failure aborts
/// I2PControl initialization. No partially constructed server state is
/// returned. No fake adapters are substituted on failure.
pub async fn init_server(
    config: &I2pControlConfig,
    base_path: &std::path::Path,
    ctx: ServerInitContext,
) -> Result<ServerInstance, I2pControlError> {
    config.validate()?;

    if ctx.sam_listener_enabled && ctx.sam_session_observation.is_none() {
        return Err(I2pControlError::Config(
            "I2PControl requires the core SAM observation handle when SAM is enabled".into(),
        ));
    }

    // Build TLS config (validates cert/key material) and retain the acceptor
    let tls_config = super::tls::build_tls_config(&config.tls, base_path)?;
    let tls_acceptor = TlsAcceptor::from(tls_config);

    let router_info_bytes = ctx.router_info_bytes.clone();
    let router_info_b64 = base64_encode(&router_info_bytes);

    // --- Build and load production address book adapter ---
    let ab_dir = base_path.join("addressbooks");
    std::fs::create_dir_all(&ab_dir).map_err(|e| {
        I2pControlError::Persistence(format!(
            "failed to create address book directory {}: {e}",
            ab_dir.display()
        ))
    })?;
    let address_book_handle = ctx.address_book_handle.ok_or_else(|| {
        I2pControlError::Config("I2PControl requires the runtime address-book owner".into())
    })?;
    let address_books = Arc::new(ProductionAddressBookControl::new(
        Arc::clone(&address_book_handle),
        ab_dir,
    ));
    address_books.load().await.map_err(|e| {
        I2pControlError::Persistence(format!("failed to load address book store: {e}"))
    })?;

    // --- Build and load production tunnel manager adapter ---
    let tm_dir = base_path.join("tunnels");
    std::fs::create_dir_all(&tm_dir).map_err(|e| {
        I2pControlError::Persistence(format!(
            "failed to create tunnel store directory {}: {e}",
            tm_dir.display()
        ))
    })?;
    let startup_tunnel_inventory = ctx.startup_tunnel_inventory.unwrap_or_default();
    let tunnels: Arc<ProductionTunnelManagerControl> = Arc::new(
        ProductionTunnelManagerControl::new_with_startup_inventory_and_sam_port_and_address_book(
            tm_dir.clone(),
            startup_tunnel_inventory,
            ctx.sam_tcp_port,
            Some(address_book_handle),
        )
        .map_err(|e| {
            I2pControlError::Persistence(format!("failed to create tunnel manager: {e}"))
        })?,
    );
    tunnels
        .load()
        .await
        .map_err(|e| I2pControlError::Persistence(format!("failed to load tunnel store: {e}")))?;

    // --- Build the shared tunnel service reference for all consumers ---
    let tunnels_shared: Arc<dyn TunnelManagerControl> = tunnels.clone();

    // --- Build production control plane (identity/version/uptime only) ---
    let metrics = ctx.event_metrics.clone().unwrap_or_else(|| Arc::new(NoopEventMetrics));
    let control_plane: Arc<dyn ControlPlane> = Arc::new(ProductionControlPlane::new(
        ctx.router_id.clone(),
        env!("CARGO_PKG_VERSION").to_string(),
        Arc::clone(&metrics),
    ));

    // --- Build production router info adapter using the shared tunnel service ---
    let log_ring = ctx.log_ring.unwrap_or_default();
    let mut router_info_control = ProductionRouterInfoControl::new(
        ctx.router_id.clone(),
        env!("CARGO_PKG_VERSION").to_string(),
        ctx.share_ratio,
        ctx.configured_bandwidth_in,
        ctx.configured_bandwidth_out,
        metrics,
        log_ring,
        tunnels_shared,
    );
    if let Some(source) = ctx.peer_directory {
        router_info_control = router_info_control.with_peer_directory_source(source);
    }
    if let Some(source) = ctx.active_peer_source {
        router_info_control = router_info_control.with_active_peer_source(source);
    }
    if let Some(source) = ctx.tunnel_source {
        router_info_control = router_info_control.with_tunnel_source(source);
    }
    let router_info: Arc<dyn RouterInfoControl> = Arc::new(router_info_control);

    // --- Install the pre-built service registry from the composition root ---
    let service_registry = ctx.service_registry.unwrap_or_default();

    // --- Construct production state with all required dependencies ---
    let mut state = I2pControlState::new_production_with_sam_observation(
        config.password.clone(),
        ProductionControls {
            address_books,
            tunnels: tunnels.clone(),
            router_info,
            control_plane,
            service_registry,
        },
        ctx.sam_session_observation,
    );
    state.set_startup_values(ctx.router_id, router_info_bytes, router_info_b64);

    let state = Arc::new(state);

    // Bind listener — this verifies the port is available
    let listener = TcpListener::bind(config.bind)
        .await
        .map_err(|e| I2pControlError::Bind(format!("Failed to bind to {}: {e}", config.bind)))?;

    tracing::info!(
        target: LOG_TARGET,
        bind = %config.bind,
        "I2PControl HTTPS listener bound",
    );

    Ok(ServerInstance {
        listener,
        tls_acceptor,
        state,
        bind: config.bind,
        connection_semaphore: Arc::new(Semaphore::new(MAX_CONNECTION_TASKS)),
    })
}

/// Zero-cost event metrics stub for production startup when no real metrics
/// source is provided. All counters return zero.
struct NoopEventMetrics;

impl EventMetrics for NoopEventMetrics {
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

/// Run the I2PControl server loop with structured shutdown.
///
/// This function is called from a spawned task after `init_server` has
/// validated configuration, set up TLS, and bound the port. The TLS
/// acceptor is used for every connection; plaintext HTTP never reaches
/// JSON-RPC dispatch.
///
/// Connection and request phases are bounded by resource permits:
///
/// ```text
/// TCP accept permit
///     -> TLS handshake timeout
///     -> HTTP connection (body limit enforced)
///     -> JSON-RPC in-flight permit
///     -> parse/validate
///     -> authenticate/version gate
///     -> bounded dispatch deadline
/// ```
///
/// Every permit is released on success, failure, timeout, disconnect,
/// cancellation, or shutdown.
pub async fn serve(
    instance: ServerInstance,
    shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<(), I2pControlError> {
    let ServerInstance {
        listener,
        tls_acceptor,
        state,
        bind,
        connection_semaphore,
    } = instance;

    let app = Router::new()
        .route("/", post(handle_jsonrpc))
        .with_state(state.clone())
        .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE));

    tracing::info!(
        target: LOG_TARGET,
        %bind,
        max_body_size = MAX_BODY_SIZE,
        max_connection_tasks = MAX_CONNECTION_TASKS,
        tls_handshake_timeout_s = TLS_HANDSHAKE_TIMEOUT.as_secs(),
        request_deadline_s = REQUEST_DEADLINE.as_secs(),
        "I2PControl HTTPS server accepting requests",
    );

    let mut shutdown_rx = shutdown_rx;

    // Pre-spawn connection semaphore bounds the number of TLS/connection
    // tasks. Each accepted connection acquires one permit before spawning
    // and releases it on every exit path (success, failure, timeout,
    // disconnect, or cancellation).
    let result = loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((tcp_stream, peer_addr)) => {
                        // Try to acquire a connection permit. If saturated,
                        // drop the accepted socket immediately.
                        let permit = match connection_semaphore.clone().try_acquire_owned() {
                            Ok(permit) => permit,
                            Err(_) => {
                                tracing::debug!(
                                    target: LOG_TARGET,
                                    "I2PControl connection limit reached; dropping accepted socket",
                                );
                                drop(tcp_stream);
                                continue;
                            }
                        };

                        let acceptor = tls_acceptor.clone();
                        let app = app.clone();

                        // Spawn TLS handshake + HTTP in a separate task
                        // to keep the accept loop unblocked.
                        tokio::spawn(async move {
                            // Move permit into the task so every exit path
                            // releases it when the task completes.
                            let _permit = permit;

                            // TLS handshake with timeout
                            let tls_stream = match tokio::time::timeout(
                                TLS_HANDSHAKE_TIMEOUT,
                                acceptor.accept(tcp_stream),
                            )
                            .await
                            {
                                Ok(Ok(tls)) => tls,
                                Ok(Err(e)) => {
                                    tracing::debug!(
                                        target: LOG_TARGET,
                                        error = %e,
                                        "I2PControl TLS handshake failed",
                                    );
                                    return;
                                }
                                Err(_elapsed) => {
                                    tracing::debug!(
                                        target: LOG_TARGET,
                                        "I2PControl TLS handshake timed out",
                                    );
                                    return;
                                }
                            };

                            // Build hyper service from the cloned Router
                            let io = TokioIo::new(tls_stream);


                            let svc = app
                                .map_request(move |mut req: http::Request<hyper::body::Incoming>| {
                                    req.extensions_mut().insert(peer_addr);
                                    req.map(axum::body::Body::new)
                                });

                            let hyper_svc = TowerToHyperService::new(svc);
                            let builder = HyperBuilder::new(TokioExecutor::new());
                            let conn = builder.serve_connection(io, hyper_svc);
                            let _ = tokio::time::timeout(REQUEST_DEADLINE, conn).await;
                        });
                    }
                    Err(e) => {
                        tracing::error!(
                            target: LOG_TARGET,
                            ?e,
                            "I2PControl accept error",
                        );
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                tracing::info!(
                    target: LOG_TARGET,
                    "I2PControl server received shutdown signal",
                );
                break Ok(());
            }
        }
    };

    state.token_service().clear();
    tracing::info!(target: LOG_TARGET, "I2PControl server stopped");
    result
}

/// Resolve an optional request ID, defaulting to Null.
fn resolve_id(id: &Option<RequestId>) -> RequestId {
    id.clone().unwrap_or(RequestId::Null)
}

/// Handle a JSON-RPC request.
pub(crate) async fn handle_jsonrpc(
    State(state): State<Arc<I2pControlState>>,
    headers: HeaderMap,
    Extension(peer_addr): Extension<SocketAddr>,
    body: String,
) -> Response {
    // Acquire concurrency permit
    let _permit =
        match tokio::time::timeout(Duration::from_secs(5), state.semaphore.acquire()).await {
            Ok(Ok(permit)) => permit,
            _ => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32603,
                            "message": "Server is busy"
                        }
                    })),
                )
                    .into_response();
            }
        };

    // Check body size
    if body.len() > MAX_BODY_SIZE {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32700,
                    "message": "Request body too large"
                }
            })),
        )
            .into_response();
    }

    // Parse JSON-RPC request
    let request = match rpc::parse_request(&body) {
        Ok(req) => req,
        Err(err) => return Json(serde_json::to_value(&err).unwrap()).into_response(),
    };

    // Authenticate before method-specific selector/config parsing. The
    // protected dispatcher receives a sanitized request with Token removed.
    let response = if request.method == rpc::methods::AUTHENTICATE {
        handle_authenticate_with_source(&state, &request, Some(peer_addr)).await
    } else {
        match authenticate_protected_request(&state, &headers, &request) {
            Ok(request) => dispatch_protected(&state, &request).await,
            Err(error) => serde_json::to_value(error).unwrap(),
        }
    };

    // Execute notifications through the same path as ordinary requests, then
    // suppress the response. Explicit JSON null remains a response ID and is
    // therefore not treated as a notification.
    let dispatch = DispatchResult {
        response,
        emit_response: !request.is_notification(),
    };
    if dispatch.emit_response {
        Json(dispatch.response).into_response()
    } else {
        StatusCode::NO_CONTENT.into_response()
    }
}

/// Result of the common dispatch path. Keeping response emission separate
/// ensures notification execution cannot bypass authentication, validation,
/// resource limits, or handler side effects.
struct DispatchResult {
    response: serde_json::Value,
    emit_response: bool,
}

async fn dispatch_protected(
    state: &I2pControlState,
    request: &JsonRpcRequest,
) -> serde_json::Value {
    match request.method.as_str() {
        rpc::methods::ADDRESS_BOOK => {
            super::address_book::handle_address_book(state, request).await
        }
        rpc::methods::SET_SUBSCRIPTIONS => {
            super::address_book::handle_set_subscriptions(state, request).await
        }
        rpc::methods::SET_CONFIG => super::address_book::handle_set_config(state, request).await,
        rpc::methods::TUNNEL_MANAGER => {
            super::tunnel_manager::handle_tunnel_manager(state, request).await
        }
        rpc::methods::ROUTER_INFO => {
            super::router_info_handler::handle_router_info(state, request).await
        }
        rpc::methods::CLIENT_SERVICES_INFO => {
            super::client_services::handle_client_services_info(state, request).await
        }
        _ => serde_json::to_value(JsonRpcErrorResponse::new(
            resolve_id(&request.id),
            rpc::error_codes::METHOD_NOT_FOUND,
            format!("Method '{}' not found", request.method),
        ))
        .unwrap(),
    }
}

/// Handle the Authenticate method.
///
/// Returns a `serde_json::Value` that is either a success or error JSON-RPC response.
#[cfg(test)]
async fn handle_authenticate(
    state: &I2pControlState,
    request: &JsonRpcRequest,
) -> serde_json::Value {
    handle_authenticate_with_source(state, request, None).await
}

async fn handle_authenticate_with_source(
    state: &I2pControlState,
    request: &JsonRpcRequest,
    source: Option<SocketAddr>,
) -> serde_json::Value {
    let id = resolve_id(&request.id);

    // Parse authenticate params
    let params = match &request.params {
        Some(params) => {
            match serde_json::from_value::<AuthenticateParams>(serde_json::Value::Object(
                params.clone(),
            )) {
                Ok(p) => p,
                Err(_) => {
                    return serde_json::to_value(JsonRpcErrorResponse::new(
                        id,
                        rpc::error_codes::INVALID_PARAMS,
                        "Invalid Authenticate parameters",
                    ))
                    .unwrap();
                }
            }
        }
        None => {
            return serde_json::to_value(JsonRpcErrorResponse::new(
                id,
                rpc::error_codes::INVALID_PARAMS,
                "Missing parameters",
            ))
            .unwrap();
        }
    };

    // Missing and unsupported API versions are distinct I2PControl errors.
    let api_version = match params.api {
        None => {
            return serde_json::to_value(JsonRpcErrorResponse::new(
                id,
                rpc::error_codes::UNSPECIFIED_API_VERSION,
                rpc::error_codes::UNSPECIFIED_API_VERSION_MESSAGE,
            ))
            .unwrap();
        }
        Some(v) if auth::validate_api_version(v) => v,
        Some(_) => {
            return serde_json::to_value(JsonRpcErrorResponse::new(
                id,
                rpc::error_codes::UNSUPPORTED_API_VERSION,
                rpc::error_codes::UNSUPPORTED_API_VERSION_MESSAGE,
            ))
            .unwrap();
        }
    };

    // Validate password
    let password = match params.password.as_deref() {
        Some(p) => p,
        None => {
            return invalid_password_response(state, source, id).await;
        }
    };

    if !auth::compare_passwords(password, &state.password) {
        return invalid_password_response(state, source, id).await;
    }

    state.auth_throttle().clear(source);

    // Issue token
    let token = state.token_service.issue();

    tracing::debug!(
        target: LOG_TARGET,
        "Authenticate successful",
    );

    serde_json::to_value(JsonRpcSuccess::new(
        id,
        serde_json::to_value(AuthenticateResult {
            Token: token,
            API: api_version,
        })
        .unwrap(),
    ))
    .unwrap()
}

async fn invalid_password_response(
    state: &I2pControlState,
    source: Option<SocketAddr>,
    id: RequestId,
) -> serde_json::Value {
    let delay = state.auth_throttle().reserve_failure(source);
    // The throttle lock is released before this await. Reservation is deliberately conservative:
    // cancellation during the delay does not erase the recorded failed attempt.
    if !delay.is_zero() {
        tokio::time::sleep(delay).await;
    }
    serde_json::to_value(JsonRpcErrorResponse::new(
        id,
        rpc::error_codes::INVALID_PASSWORD,
        rpc::error_codes::INVALID_PASSWORD_MESSAGE,
    ))
    .unwrap()
}

/// Authenticate a protected request and remove authentication metadata before
/// method-specific validation. The header remains a compatibility extension;
/// an explicit params.Token always takes precedence and conflicts fail closed.
fn authenticate_protected_request(
    state: &I2pControlState,
    headers: &HeaderMap,
    request: &JsonRpcRequest,
) -> Result<JsonRpcRequest, JsonRpcErrorResponse> {
    let id = resolve_id(&request.id);
    let parameter_token = request
        .params
        .as_ref()
        .and_then(|params| params.get("Token"))
        .map(|value| value.as_str().map(str::to_owned));
    let header_token = headers
        .get("X-I2PControl-Token")
        .map(|value| value.to_str().ok().map(str::to_owned));

    let token = match (parameter_token, header_token) {
        (None, None) => {
            return Err(JsonRpcErrorResponse::new(
                id,
                rpc::error_codes::NO_TOKEN,
                rpc::error_codes::NO_TOKEN_MESSAGE,
            ));
        }
        (Some(Some(token)), None) | (None, Some(Some(token))) => token,
        (Some(Some(parameter)), Some(Some(header))) if parameter == header => parameter,
        _ => {
            return Err(JsonRpcErrorResponse::new(
                id,
                rpc::error_codes::INVALID_TOKEN,
                rpc::error_codes::INVALID_TOKEN_MESSAGE,
            ));
        }
    };

    if !state.token_service.validate(&token) {
        return Err(JsonRpcErrorResponse::new(
            id,
            rpc::error_codes::INVALID_TOKEN,
            rpc::error_codes::INVALID_TOKEN_MESSAGE,
        ));
    }

    let mut sanitized = request.clone();
    if let Some(params) = sanitized.params.as_mut() {
        params.remove("Token");
    }
    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2pcontrol::{
        control_plane::{FakeAddressBookControl, FakeControlPlane, FakeTunnelManagerControl},
        router_info::FakeRouterInfoControl,
    };

    #[test]
    fn config_validation_empty_password() {
        let config = I2pControlConfig {
            enabled: true,
            bind: "127.0.0.1:7650".parse().unwrap(),
            password: String::new(),
            tls: TlsConfig {
                certificate: None,
                private_key: None,
            },
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_validation_disabled_no_password() {
        let config = I2pControlConfig {
            enabled: false,
            bind: "127.0.0.1:7650".parse().unwrap(),
            password: String::new(),
            tls: TlsConfig {
                certificate: None,
                private_key: None,
            },
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn resolve_id_returns_id() {
        assert_eq!(
            resolve_id(&Some(RequestId::Number(1))),
            RequestId::Number(1)
        );
    }

    #[test]
    fn resolve_id_defaults_to_null() {
        assert_eq!(resolve_id(&None), RequestId::Null);
    }

    fn request(method: &str, params: serde_json::Value, id: Option<RequestId>) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: params.as_object().cloned(),
            id,
        }
    }

    #[tokio::test]
    async fn authenticate_uses_standard_params_and_numeric_api() {
        let state = I2pControlState::new_test("testpass".to_string());
        let response = handle_authenticate(
            &state,
            &request(
                rpc::methods::AUTHENTICATE,
                serde_json::json!({"API": 2, "Password": "testpass"}),
                Some(RequestId::Number(7)),
            ),
        )
        .await;

        assert_eq!(response["result"]["API"], 2);
        assert!(response["result"]["API"].is_number());
        assert!(response["result"]["Token"].is_string());
        assert!(!response["result"]["Token"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn authenticate_distinguishes_password_and_api_errors() {
        let state = I2pControlState::new_test("testpass".to_string());

        let missing_password = handle_authenticate(
            &state,
            &request(
                rpc::methods::AUTHENTICATE,
                serde_json::json!({"API": 2}),
                Some(RequestId::Number(1)),
            ),
        )
        .await;
        assert_eq!(
            missing_password["error"]["code"],
            rpc::error_codes::INVALID_PASSWORD
        );
        assert_eq!(
            missing_password["error"]["message"],
            rpc::error_codes::INVALID_PASSWORD_MESSAGE
        );

        let missing_api = handle_authenticate(
            &state,
            &request(
                rpc::methods::AUTHENTICATE,
                serde_json::json!({"Password": "testpass"}),
                Some(RequestId::Number(2)),
            ),
        )
        .await;
        assert_eq!(
            missing_api["error"]["code"],
            rpc::error_codes::UNSPECIFIED_API_VERSION
        );

        let unsupported_api = handle_authenticate(
            &state,
            &request(
                rpc::methods::AUTHENTICATE,
                serde_json::json!({"API": 3, "Password": "testpass"}),
                Some(RequestId::Number(3)),
            ),
        )
        .await;
        assert_eq!(
            unsupported_api["error"]["code"],
            rpc::error_codes::UNSUPPORTED_API_VERSION
        );

        let wrong_password = handle_authenticate(
            &state,
            &request(
                rpc::methods::AUTHENTICATE,
                serde_json::json!({"API": 2, "Password": "wrong-secret"}),
                Some(RequestId::Number(4)),
            ),
        )
        .await;
        assert_eq!(
            wrong_password["error"]["code"],
            rpc::error_codes::INVALID_PASSWORD
        );
        assert!(!wrong_password.to_string().contains("wrong-secret"));
    }

    #[tokio::test]
    async fn failed_authentication_is_bounded_and_throttled() {
        let state = I2pControlState::new_test("testpass".to_string());
        let source = Some("127.0.0.1:7650".parse().unwrap());
        let wrong = request(
            rpc::methods::AUTHENTICATE,
            serde_json::json!({"API": 2, "Password": "wrong"}),
            Some(RequestId::Number(1)),
        );

        let first = handle_authenticate_with_source(&state, &wrong, source).await;
        assert_eq!(first["error"]["code"], rpc::error_codes::INVALID_PASSWORD);
        assert_eq!(state.auth_throttle().count(), 1);
        let delay = state.auth_throttle().reserve_failure(source);
        assert!(delay <= auth::THROTTLE_MAX_DELAY);

        let second = handle_authenticate_with_source(&state, &wrong, source).await;
        assert_eq!(second["error"]["code"], rpc::error_codes::INVALID_PASSWORD);
        assert_eq!(state.auth_throttle().count(), 1);
    }

    #[tokio::test]
    async fn successful_authentication_resets_failure_state() {
        let state = I2pControlState::new_test("testpass".to_string());
        let source = Some("127.0.0.1:7651".parse().unwrap());
        let wrong = request(
            rpc::methods::AUTHENTICATE,
            serde_json::json!({"API": 2, "Password": "wrong"}),
            Some(RequestId::Number(1)),
        );
        let correct = request(
            rpc::methods::AUTHENTICATE,
            serde_json::json!({"API": 2, "Password": "testpass"}),
            Some(RequestId::Number(2)),
        );
        let _ = handle_authenticate_with_source(&state, &wrong, source).await;
        let response = handle_authenticate_with_source(&state, &correct, source).await;
        assert!(response["result"]["Token"].is_string());
        assert_eq!(state.auth_throttle().count(), 0);
    }

    #[tokio::test]
    async fn authentication_throttle_is_shared_across_reconnect_ports() {
        let state = I2pControlState::new_test("testpass".to_string());
        let wrong = request(
            rpc::methods::AUTHENTICATE,
            serde_json::json!({"API": 2, "Password": "wrong"}),
            Some(RequestId::Number(1)),
        );
        let correct = request(
            rpc::methods::AUTHENTICATE,
            serde_json::json!({"API": 2, "Password": "testpass"}),
            Some(RequestId::Number(2)),
        );
        let first_port = Some("127.0.0.1:10001".parse().unwrap());
        let second_port = Some("127.0.0.1:50000".parse().unwrap());

        let first = handle_authenticate_with_source(&state, &wrong, first_port).await;
        let second = handle_authenticate_with_source(&state, &wrong, second_port).await;
        assert_eq!(first["error"]["code"], rpc::error_codes::INVALID_PASSWORD);
        assert_eq!(second["error"]["code"], rpc::error_codes::INVALID_PASSWORD);
        assert_eq!(state.auth_throttle().count(), 1);

        let response = handle_authenticate_with_source(&state, &correct, second_port).await;
        assert!(response["result"]["Token"].is_string());
        assert_eq!(state.auth_throttle().count(), 0);
    }

    #[tokio::test]
    async fn protected_authentication_sanitizes_params_and_supports_base_router_info() {
        let router_info = FakeRouterInfoControl::new();
        router_info.set_version("Emissary test".to_string());
        let mut state = I2pControlState::new_test("testpass".to_string());
        state.set_router_info(Box::new(router_info));
        let token = state.token_service().issue();
        let original = request(
            rpc::methods::ROUTER_INFO,
            serde_json::json!({"Token": token, "i2p.router.version": false}),
            Some(RequestId::Number(1)),
        );
        let sanitized = authenticate_protected_request(&state, &HeaderMap::new(), &original)
            .expect("params.Token should authenticate");
        assert!(!sanitized.params.as_ref().unwrap().contains_key("Token"));
        assert!(sanitized.params.as_ref().unwrap().contains_key("i2p.router.version"));

        let response = dispatch_protected(&state, &sanitized).await;
        assert_eq!(response["result"]["i2p.router.version"], "Emissary test");
    }

    #[tokio::test]
    async fn unsupported_base_methods_return_method_not_found() {
        let state = I2pControlState::new_test("testpass".to_string());
        for method in rpc::methods::UNSUPPORTED_BASE {
            let response = dispatch_protected(
                &state,
                &request(method, serde_json::json!({}), Some(RequestId::Number(1))),
            )
            .await;
            assert_eq!(
                response["error"]["code"],
                rpc::error_codes::METHOD_NOT_FOUND
            );
            assert!(response["result"].is_null(), "method: {method}");
        }
    }

    #[tokio::test]
    async fn protected_authentication_distinguishes_missing_unknown_and_conflicting_tokens() {
        let state = I2pControlState::new_test("testpass".to_string());
        let base_request = request(
            rpc::methods::ROUTER_INFO,
            serde_json::json!({"i2p.router.version": true}),
            Some(RequestId::Number(1)),
        );
        let missing =
            authenticate_protected_request(&state, &HeaderMap::new(), &base_request).unwrap_err();
        assert_eq!(missing.error.code, rpc::error_codes::NO_TOKEN);

        let unknown = request(
            rpc::methods::ROUTER_INFO,
            serde_json::json!({"Token": "unknown", "i2p.router.version": true}),
            Some(RequestId::Number(2)),
        );
        let unknown =
            authenticate_protected_request(&state, &HeaderMap::new(), &unknown).unwrap_err();
        assert_eq!(unknown.error.code, rpc::error_codes::INVALID_TOKEN);
        assert!(!unknown.error.message.contains("unknown"));

        let token = state.token_service().issue();
        let mut headers = HeaderMap::new();
        headers.insert("X-I2PControl-Token", "different".parse().unwrap());
        let conflict = request(
            rpc::methods::ROUTER_INFO,
            serde_json::json!({"Token": token, "i2p.router.version": true}),
            Some(RequestId::Number(3)),
        );
        let conflict = authenticate_protected_request(&state, &headers, &conflict).unwrap_err();
        assert_eq!(conflict.error.code, rpc::error_codes::INVALID_TOKEN);

        let header_only = request(
            rpc::methods::ROUTER_INFO,
            serde_json::json!({"i2p.router.version": true}),
            Some(RequestId::Number(4)),
        );
        headers.insert("X-I2PControl-Token", token.parse().unwrap());
        assert!(authenticate_protected_request(&state, &headers, &header_only).is_ok());
    }

    #[tokio::test]
    async fn notifications_execute_then_suppress_success_and_error_responses() {
        let state = Arc::new(I2pControlState::new_for_test("testpass".to_string()));
        let success = handle_jsonrpc(
            State(Arc::clone(&state)),
            HeaderMap::new(),
            Extension("127.0.0.1:7650".parse().unwrap()),
            r#"{"jsonrpc":"2.0","method":"Authenticate","params":{"API":2,"Password":"testpass"}}"#
                .to_string(),
        )
        .await;
        assert_eq!(success.status(), StatusCode::NO_CONTENT);
        assert_eq!(state.token_service().count(), 1);

        let error = handle_jsonrpc(
            State(Arc::clone(&state)),
            HeaderMap::new(),
            Extension("127.0.0.1:7650".parse().unwrap()),
            r#"{"jsonrpc":"2.0","method":"Authenticate","params":{"API":2,"Password":"wrong"}}"#
                .to_string(),
        )
        .await;
        assert_eq!(error.status(), StatusCode::NO_CONTENT);
        assert_eq!(state.token_service().count(), 1);
    }

    // --- M008 composition and provenance tests ---

    #[test]
    fn production_requires_all_dependencies() {
        let tm: Arc<dyn TunnelManagerControl> = Arc::new(FakeTunnelManagerControl::new());
        let ab: Arc<dyn AddressBookControl> = Arc::new(FakeAddressBookControl::new());
        let ri: Arc<dyn RouterInfoControl> = Arc::new(FakeRouterInfoControl::new());
        let cp: Arc<dyn ControlPlane> = Arc::new(FakeControlPlane::new());

        let state = I2pControlState::new_production_with_sam_observation(
            "testpass".to_string(),
            ProductionControls {
                address_books: ab,
                tunnels: tm,
                router_info: ri,
                control_plane: cp,
                service_registry: ServiceRegistry::new(),
            },
            Some(SamSessionObservationHandle::empty_for_test()),
        );

        assert_eq!(state.router_id(), "");
    }

    #[test]
    fn test_state_is_only_path_for_fakes() {
        let state = I2pControlState::new_test("testpass".to_string());
        let _ = state.router_info();
    }

    #[tokio::test]
    async fn shared_tunnel_object_identity() {
        use crate::i2pcontrol::domain::tunnel::{
            StartIntent, TunnelDefinition, TunnelName, TunnelOwnership, TunnelRuntimeState,
            TunnelType,
        };
        use std::sync::atomic::AtomicUsize;

        struct SentinelTunnelControl {
            generation: AtomicUsize,
        }
        impl SentinelTunnelControl {
            fn new() -> Self {
                Self {
                    generation: AtomicUsize::new(0),
                }
            }
            fn generation(&self) -> usize {
                self.generation.load(std::sync::atomic::Ordering::Acquire)
            }
        }

        #[async_trait::async_trait]
        impl TunnelManagerControl for SentinelTunnelControl {
            async fn list(&self) -> Result<Vec<TunnelDefinition>, String> {
                Ok(Vec::new())
            }
            async fn get(&self, _: &str) -> Result<Option<TunnelDefinition>, String> {
                Ok(None)
            }
            async fn create(&self, _: TunnelDefinition) -> Result<(), String> {
                self.generation.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                Ok(())
            }
            async fn update(
                &self,
                _: &str,
                _: TunnelDefinition,
                _: Option<TunnelName>,
            ) -> Result<bool, String> {
                Ok(false)
            }
            async fn delete(&self, _: &str) -> Result<bool, String> {
                Ok(false)
            }
            async fn start(&self, _: &str) -> Result<String, String> {
                Ok("ok".into())
            }
            async fn stop(&self, _: &str) -> Result<String, String> {
                Ok("ok".into())
            }
            async fn restart(&self, _: &str) -> Result<String, String> {
                Ok("ok".into())
            }
            fn get_backend(
                &self,
                _: TunnelType,
            ) -> Option<Arc<dyn crate::i2pcontrol::backends::TunnelBackend>> {
                None
            }
            fn registry(&self) -> &crate::i2pcontrol::backends::registry::TunnelBackendRegistry {
                use std::sync::OnceLock;
                static REGISTRY: OnceLock<
                    crate::i2pcontrol::backends::registry::TunnelBackendRegistry,
                > = OnceLock::new();
                REGISTRY.get_or_init(|| {
                    crate::i2pcontrol::backends::registry::create_default_registry()
                        .expect("default registry is exhaustive")
                })
            }
        }

        let sentinel = Arc::new(SentinelTunnelControl::new());

        sentinel
            .create(TunnelDefinition {
                name: TunnelName::new("t").unwrap(),
                tunnel_type: TunnelType::Client,
                ownership: TunnelOwnership::ControlPlane,
                runtime_state: TunnelRuntimeState::Stopped,
                start_intent: StartIntent::DoNotStart,
                options: Default::default(),
                raw_config: Default::default(),
            })
            .await
            .unwrap();
        assert_eq!(sentinel.generation(), 1);

        let ab: Arc<dyn AddressBookControl> = Arc::new(FakeAddressBookControl::new());
        let ri: Arc<dyn RouterInfoControl> = Arc::new(FakeRouterInfoControl::new());
        let cp: Arc<dyn ControlPlane> = Arc::new(FakeControlPlane::new());

        let state = I2pControlState::new_production_with_sam_observation(
            "testpass".to_string(),
            ProductionControls {
                address_books: ab,
                tunnels: sentinel.clone() as Arc<dyn TunnelManagerControl>,
                router_info: ri,
                control_plane: cp,
                service_registry: ServiceRegistry::new(),
            },
            Some(SamSessionObservationHandle::empty_for_test()),
        );

        let _ = state.tunnel_list().await.unwrap();
        assert_eq!(
            sentinel.generation(),
            1,
            "state and sentinel share the same object"
        );
    }

    #[tokio::test]
    async fn tunnel_list_failure_returns_error() {
        struct FailingTunnelControl;
        #[async_trait::async_trait]
        impl TunnelManagerControl for FailingTunnelControl {
            async fn list(
                &self,
            ) -> Result<Vec<crate::i2pcontrol::domain::tunnel::TunnelDefinition>, String>
            {
                Err("store read failed".into())
            }
            async fn get(
                &self,
                _: &str,
            ) -> Result<Option<crate::i2pcontrol::domain::tunnel::TunnelDefinition>, String>
            {
                Err("store read failed".into())
            }
            async fn create(
                &self,
                _: crate::i2pcontrol::domain::tunnel::TunnelDefinition,
            ) -> Result<(), String> {
                unimplemented!()
            }
            async fn update(
                &self,
                _: &str,
                _: crate::i2pcontrol::domain::tunnel::TunnelDefinition,
                _: Option<crate::i2pcontrol::domain::tunnel::TunnelName>,
            ) -> Result<bool, String> {
                unimplemented!()
            }
            async fn delete(&self, _: &str) -> Result<bool, String> {
                unimplemented!()
            }
            async fn start(&self, _: &str) -> Result<String, String> {
                unimplemented!()
            }
            async fn stop(&self, _: &str) -> Result<String, String> {
                unimplemented!()
            }
            async fn restart(&self, _: &str) -> Result<String, String> {
                unimplemented!()
            }
            fn get_backend(
                &self,
                _: crate::i2pcontrol::domain::tunnel::TunnelType,
            ) -> Option<Arc<dyn crate::i2pcontrol::backends::TunnelBackend>> {
                None
            }
            fn registry(&self) -> &crate::i2pcontrol::backends::registry::TunnelBackendRegistry {
                use std::sync::OnceLock;
                static REGISTRY: OnceLock<
                    crate::i2pcontrol::backends::registry::TunnelBackendRegistry,
                > = OnceLock::new();
                REGISTRY.get_or_init(|| {
                    crate::i2pcontrol::backends::registry::create_default_registry()
                        .expect("default registry is exhaustive")
                })
            }
        }

        let tm: Arc<dyn TunnelManagerControl> = Arc::new(FailingTunnelControl);
        let ab: Arc<dyn AddressBookControl> = Arc::new(FakeAddressBookControl::new());
        let ri: Arc<dyn RouterInfoControl> = Arc::new(FakeRouterInfoControl::new());
        let cp: Arc<dyn ControlPlane> = Arc::new(FakeControlPlane::new());

        let state = I2pControlState::new_production_with_sam_observation(
            "testpass".to_string(),
            ProductionControls {
                address_books: ab,
                tunnels: tm,
                router_info: ri,
                control_plane: cp,
                service_registry: ServiceRegistry::new(),
            },
            Some(SamSessionObservationHandle::empty_for_test()),
        );

        let result = state.tunnel_list().await;
        assert!(
            result.is_err(),
            "tunnel_list should propagate the error, not return empty Vec"
        );

        let result = state.tunnel_get("test").await;
        assert!(
            result.is_err(),
            "tunnel_get should propagate the error, not return None"
        );
    }

    #[tokio::test]
    async fn address_book_failure_returns_error() {
        use crate::i2pcontrol::domain::address_book::AdministrativeAddressBookType;

        struct FailingAddressBookControl;
        #[async_trait::async_trait]
        impl crate::i2pcontrol::control_plane::AddressBookControl for FailingAddressBookControl {
            async fn list(
                &self,
                _: AdministrativeAddressBookType,
            ) -> Result<Vec<crate::i2pcontrol::domain::address_book::AddressBookEntry>, String>
            {
                Err("store read failed".into())
            }
            async fn lookup(
                &self,
                _: AdministrativeAddressBookType,
                _: &str,
            ) -> Result<Option<crate::i2pcontrol::domain::address_book::AddressBookEntry>, String>
            {
                Err("store read failed".into())
            }
            async fn add(
                &self,
                _: AdministrativeAddressBookType,
                _: crate::i2pcontrol::domain::address_book::AddressBookEntry,
            ) -> Result<(), String> {
                unimplemented!()
            }
            async fn update(
                &self,
                _: AdministrativeAddressBookType,
                _: crate::i2pcontrol::domain::address_book::AddressBookEntry,
            ) -> Result<bool, String> {
                unimplemented!()
            }
            async fn delete(
                &self,
                _: AdministrativeAddressBookType,
                _: &str,
            ) -> Result<bool, String> {
                unimplemented!()
            }
            async fn delete_all(&self, _: AdministrativeAddressBookType) -> Result<bool, String> {
                unimplemented!()
            }
            async fn subscriptions(
                &self,
            ) -> Result<crate::i2pcontrol::domain::address_book::SubscriptionSet, String>
            {
                Err("store read failed".into())
            }
            async fn set_subscriptions(
                &self,
                _: crate::i2pcontrol::domain::address_book::SubscriptionSet,
            ) -> Result<(), String> {
                unimplemented!()
            }
            async fn configuration(
                &self,
            ) -> Result<crate::i2pcontrol::domain::address_book::AddressBookConfiguration, String>
            {
                Err("store read failed".into())
            }
            async fn set_configuration(
                &self,
                _: crate::i2pcontrol::domain::address_book::AddressBookConfiguration,
            ) -> Result<(), String> {
                unimplemented!()
            }
        }

        let tm: Arc<dyn TunnelManagerControl> = Arc::new(FakeTunnelManagerControl::new());
        let ab: Arc<dyn AddressBookControl> = Arc::new(FailingAddressBookControl);
        let ri: Arc<dyn RouterInfoControl> = Arc::new(FakeRouterInfoControl::new());
        let cp: Arc<dyn ControlPlane> = Arc::new(FakeControlPlane::new());

        let state = I2pControlState::new_production_with_sam_observation(
            "testpass".to_string(),
            ProductionControls {
                address_books: ab,
                tunnels: tm,
                router_info: ri,
                control_plane: cp,
                service_registry: ServiceRegistry::new(),
            },
            Some(SamSessionObservationHandle::empty_for_test()),
        );

        let result = state.address_book_list(AdministrativeAddressBookType::Private).await;
        assert!(
            result.is_err(),
            "address_book_list should propagate error, not return empty Vec"
        );

        let result = state
            .address_book_lookup(AdministrativeAddressBookType::Private, "test.i2p")
            .await;
        assert!(
            result.is_err(),
            "address_book_lookup should propagate error, not return None"
        );

        let result = state.address_book_subscriptions().await;
        assert!(
            result.is_err(),
            "address_book_subscriptions should propagate error"
        );
    }

    #[tokio::test]
    async fn fail_closed_startup_dir_creation_failure() {
        let tmp = tempfile::tempdir().unwrap();
        // Block addressbooks directory creation by placing a file in its path
        let blocker = tmp.path().join("addressbooks");
        std::fs::write(&blocker, "x").unwrap();

        let config = I2pControlConfig {
            enabled: true,
            bind: "127.0.0.1:0".parse().unwrap(),
            password: "testpass".to_string(),
            tls: TlsConfig {
                certificate: None,
                private_key: None,
            },
        };
        let ctx = ServerInitContext::new("id".into(), vec![]);

        let result = init_server(&config, tmp.path(), ctx).await;
        assert!(result.is_err());
        if let Err(I2pControlError::Persistence(msg)) = result {
            assert!(
                msg.contains("address book"),
                "error should mention address book: {msg}"
            );
        } else {
            panic!("expected Persistence error");
        }
    }

    #[tokio::test]
    async fn fail_closed_startup_no_temp_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let config = I2pControlConfig {
            enabled: true,
            bind: "127.0.0.1:0".parse().unwrap(),
            password: "testpass".to_string(),
            tls: TlsConfig {
                certificate: None,
                private_key: None,
            },
        };
        let (_manager, control) = crate::i2pcontrol::address_book_runtime::new_controlled_manager(
            tmp.path().to_owned(),
            crate::config::AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let ctx = ServerInitContext::new("id".into(), vec![]).with_address_book_handle(control);

        let _ = init_server(&config, tmp.path(), ctx).await.unwrap();

        // No temp fallback directories should have been created
        let temp = std::env::temp_dir();
        let entries: Vec<_> = std::fs::read_dir(&temp)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("emissary-i2pcontrol"))
            .collect();
        assert!(
            entries.is_empty(),
            "no fallback directories should exist in temp: {:?}",
            entries
        );
    }
}
