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

//! Production adapters for I2PControl control plane traits.
//!
//! These adapters wrap the real emissary-core subsystems, the runtime
//! address-book owner, and bounded persistent I2PControl stores. They expose
//! only purpose-specific handles and snapshot DTOs.
//!
//! All adapters are `Send + Sync` and document no mutation, no event
//! subscriber consumption, and no private key exposure.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;

use crate::i2pcontrol::{
    address_book_runtime::{
        RuntimeAddressBookEntry, RuntimeAddressBookHandle, RuntimeAddressBookSnapshot,
        RuntimeAddressBookType,
    },
    backends::{registry::TunnelBackendRegistry, BackendError, TunnelBackend},
    backends::options::validate_common_options,
    client_secret_store::ClientDestinationStore,
    control_plane::{AddressBookControl, ControlPlane, TunnelManagerControl},
    domain::{
        address_book::{
            AddressBookConfiguration, AddressBookEntry, AdministrativeAddressBookType,
            SubscriptionSet,
        },
        tunnel::{TunnelDefinition, TunnelName, TunnelType},
    },
    news::RouterNewsSource,
    observability::LogRing,
    router_info::{
        ActivePeerSnapshot, ActivePeerSource, ActivePeerStats, BannedPeer, ClockSkew,
        I2PTunnelStats, InspectionError, InspectionGroup, LogEntry, LogSnapshot, NetworkSnapshot,
        NetworkStatus, PeerDirectorySnapshot, PeerDirectorySource, PeerIdentity, PeerLimits,
        RecentTransitTraffic, RouterInfoControl, TransitBytes, TransportBytes, TransportLimits,
        TunnelBuildStats, TunnelDetail, TunnelDetails, TunnelSource, TunnelSummary,
        BANNED_PEER_SOURCE,
    },
    server_secret_store::{ServerDestinationStore, StoredDestination},
    stores::{address_book_store::AddressBookStore, tunnel_store::TunnelStore},
    transit_sampler::TransitBandwidthSampler,
};

use emissary_core::{
    crypto::base64_encode,
    events::EventHandle,
    inspection::{
        NetworkState, PeerDirectoryInspection, PeerDirectoryInspectionError, TransportInspection,
        TransportInspectionError, TunnelInspection, TunnelInspectionError, TunnelPoolKind,
    },
    runtime::Runtime,
    FirewallStatus,
};

use crate::tunnel_client::{StartupTunnelAction, StartupTunnelLifecycleHandle, StartupTunnelState};

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum CommitBoundary {
    BeforeSecretCommit,
    AfterFreshSecretCommit,
    AfterReplacementSecretCommit,
    BeforeExistingDefinitionPersist,
}

#[cfg(test)]
struct CommitPhaseTestHook {
    boundary: std::sync::Mutex<Option<CommitBoundary>>,
    entered: std::sync::atomic::AtomicBool,
    entered_notify: tokio::sync::Notify,
    release: tokio::sync::Notify,
    terminalized: std::sync::atomic::AtomicBool,
    terminalized_notify: tokio::sync::Notify,
}

#[cfg(test)]
impl CommitPhaseTestHook {
    fn new() -> Self {
        Self {
            boundary: std::sync::Mutex::new(None),
            entered: std::sync::atomic::AtomicBool::new(false),
            entered_notify: tokio::sync::Notify::new(),
            release: tokio::sync::Notify::new(),
            terminalized: std::sync::atomic::AtomicBool::new(false),
            terminalized_notify: tokio::sync::Notify::new(),
        }
    }

    fn arm(&self, boundary: CommitBoundary) {
        *self.boundary.lock().expect("commit hook lock poisoned") = Some(boundary);
        self.entered.store(false, std::sync::atomic::Ordering::SeqCst);
        self.terminalized.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    async fn pause(&self, boundary: CommitBoundary) {
        let armed = {
            let mut configured = self.boundary.lock().expect("commit hook lock poisoned");
            let armed = configured.as_ref() == Some(&boundary);
            if armed {
                configured.take();
            }
            armed
        };
        if !armed {
            return;
        }
        self.entered.store(true, std::sync::atomic::Ordering::SeqCst);
        self.entered_notify.notify_waiters();
        self.release.notified().await;
    }

    async fn wait_entered(&self) {
        while !self.entered.load(std::sync::atomic::Ordering::SeqCst) {
            self.entered_notify.notified().await;
        }
    }

    fn release(&self) {
        self.release.notify_one();
    }

    fn mark_terminalized(&self) {
        self.terminalized.store(true, std::sync::atomic::Ordering::SeqCst);
        self.terminalized_notify.notify_waiters();
    }

    async fn wait_terminalized(&self) {
        while !self.terminalized.load(std::sync::atomic::Ordering::SeqCst) {
            self.terminalized_notify.notified().await;
        }
    }
}

/// Maximum number of startup and control-plane tunnel definitions exposed by
/// one logical inventory.
pub const MAX_TUNNEL_INVENTORY: usize = 1000;

/// Public peer directory backed by the canonical, live core profile storage.
pub struct LivePeerDirectorySource<R: Runtime> {
    inspection: PeerDirectoryInspection<R>,
    max_items: usize,
}

impl<R: Runtime> LivePeerDirectorySource<R> {
    /// Create a bounded request-time public peer directory source.
    pub fn new(inspection: PeerDirectoryInspection<R>, max_items: usize) -> Self {
        Self {
            inspection,
            max_items,
        }
    }
}

impl<R: Runtime + Sync> PeerDirectorySource for LivePeerDirectorySource<R> {
    fn snapshot(&self) -> Result<PeerDirectorySnapshot, InspectionError> {
        let snapshot = self.inspection.snapshot(self.max_items).map_err(|error| match error {
            PeerDirectoryInspectionError::ItemLimitExceeded { limit } =>
                InspectionError::ResultTooLarge {
                    group: InspectionGroup::PeerList,
                    limit,
                },
            PeerDirectoryInspectionError::IncompleteEntry =>
                InspectionError::TemporarilyUnavailable {
                    group: InspectionGroup::PeerLookup,
                },
        })?;

        let mut peer_ids = Vec::with_capacity(snapshot.entries.len());
        let mut router_infos = BTreeMap::new();
        for entry in snapshot.entries {
            let peer_id = entry.router_id.to_base64().to_owned();
            peer_ids.push(peer_id.clone());
            router_infos.insert(peer_id, entry.router_info);
        }
        peer_ids.sort_unstable();
        peer_ids.dedup();

        Ok(PeerDirectorySnapshot {
            peer_ids,
            router_infos,
        })
    }
}

/// Current transport source backed by the canonical transport manager.
pub struct LiveActivePeerSource {
    inspection: TransportInspection,
    max_items: usize,
}

impl LiveActivePeerSource {
    /// Create a bounded request-time transport source.
    pub fn new(inspection: TransportInspection, max_items: usize) -> Self {
        Self {
            inspection,
            max_items,
        }
    }
}

impl ActivePeerSource for LiveActivePeerSource {
    fn snapshot(&self) -> Result<ActivePeerSnapshot, InspectionError> {
        let snapshot = self.inspection.snapshot(self.max_items).map_err(|error| match error {
            TransportInspectionError::ItemLimitExceeded { limit } =>
                InspectionError::ResultTooLarge {
                    group: InspectionGroup::PeerList,
                    limit,
                },
        })?;
        Ok(ActivePeerSnapshot {
            peer_ids: snapshot.connected_peer_ids,
            ntcp_limit: snapshot.ntcp2_limit,
            ssu_limit: snapshot.ssu2_limit,
            stats: snapshot
                .peer_stats
                .into_iter()
                .map(|peer| ActivePeerStats {
                    peer_id: peer.peer_id,
                    direction: if peer.inbound {
                        "inbound".to_owned()
                    } else {
                        "outbound".to_owned()
                    },
                    state: if peer.connected {
                        "connected".to_owned()
                    } else {
                        "disconnected".to_owned()
                    },
                    bytes_received: peer.bytes_received,
                    bytes_sent: peer.bytes_sent,
                    avg_latency_ms: None,
                })
                .collect(),
        })
    }
}

/// Current tunnel source backed by the canonical core tunnel owners.
pub struct LiveTunnelSource {
    inspection: TunnelInspection,
    max_items: usize,
}

impl LiveTunnelSource {
    /// Create a bounded request-time tunnel source.
    pub fn new(inspection: TunnelInspection, max_items: usize) -> Self {
        Self {
            inspection,
            max_items,
        }
    }
}

impl TunnelSource for LiveTunnelSource {
    fn snapshot(&self) -> Result<TunnelDetails, InspectionError> {
        let snapshot = self.inspection.snapshot(self.max_items).map_err(|error| match error {
            TunnelInspectionError::Incomplete => InspectionError::TemporarilyUnavailable {
                group: InspectionGroup::TunnelSummary,
            },
            TunnelInspectionError::ItemLimitExceeded { limit } => InspectionError::ResultTooLarge {
                group: InspectionGroup::TunnelSummary,
                limit,
            },
        })?;

        let mut details = TunnelDetails {
            queue_depth: snapshot.queue_depth,
            tbm_queue_depth: snapshot.tbm_queue_depth,
            ..Default::default()
        };
        for entry in snapshot.entries {
            let detail = TunnelDetail {
                tunnel_id: entry.tunnel_id,
                pool_id: (entry.pool_kind != TunnelPoolKind::Participating)
                    .then_some(entry.pool_id),
                direction: entry.direction.map(|direction| match direction {
                    emissary_core::inspection::TunnelDirection::Inbound => "inbound".to_owned(),
                    emissary_core::inspection::TunnelDirection::Outbound => "outbound".to_owned(),
                }),
            };
            match entry.pool_kind {
                TunnelPoolKind::Participating => details.participating.push(detail),
                TunnelPoolKind::Exploratory => details.exploratory.push(detail),
                TunnelPoolKind::Client => details.client.push(detail),
            }
        }
        Ok(details)
    }
}

/// Parsed startup client tunnel values needed by the I2PControl composition
/// seam. This deliberately avoids making the library-side I2PControl code
/// depend on the binary's full configuration module.
#[derive(Debug, Clone)]
pub struct StartupClientConfig {
    pub name: String,
    pub address: Option<String>,
    pub port: u16,
    pub destination: String,
    pub destination_port: Option<u16>,
}

/// Parsed startup server tunnel values needed by the I2PControl composition
/// seam. The private destination path is intentionally not copied into the
/// administrative inventory.
#[derive(Debug, Clone)]
pub struct StartupServerConfig {
    pub name: String,
    pub port: u16,
}

/// Read-only startup tunnel definitions shared by I2PControl and the existing
/// generic tunnel managers.
///
/// The control plane may observe this source, but it never persists or mutates
/// these definitions. The only runtime update is the actual server I2P
/// destination reported by the existing Yosemite session after it starts.
#[derive(Clone, Default)]
pub struct StartupTunnelInventory {
    definitions: Arc<RwLock<BTreeMap<String, TunnelDefinition>>>,
    lifecycle: Option<StartupTunnelLifecycleHandle>,
}

impl StartupTunnelInventory {
    /// Map the already-parsed startup configuration into bounded domain DTOs.
    pub fn from_configs(
        clients: &[StartupClientConfig],
        servers: &[StartupServerConfig],
    ) -> Result<Self, String> {
        let mut definitions = BTreeMap::new();
        if clients.len().saturating_add(servers.len()) > MAX_TUNNEL_INVENTORY {
            return Err(format!(
                "startup tunnel inventory exceeds maximum of {MAX_TUNNEL_INVENTORY} entries"
            ));
        }

        for config in clients {
            let definition = startup_client_definition(config)?;
            insert_startup_definition(&mut definitions, definition)?;
        }
        for config in servers {
            let definition = startup_server_definition(config)?;
            insert_startup_definition(&mut definitions, definition)?;
        }

        Ok(Self {
            definitions: Arc::new(RwLock::new(definitions)),
            lifecycle: None,
        })
    }

    /// Attach the neutral runtime owner used by the application composition.
    pub fn with_lifecycle(mut self, lifecycle: StartupTunnelLifecycleHandle) -> Self {
        self.lifecycle = Some(lifecycle);
        self
    }

    fn runtime_state(
        &self,
        name: &str,
    ) -> Option<crate::i2pcontrol::domain::tunnel::TunnelRuntimeState> {
        match self.lifecycle.as_ref()?.state(name).ok().flatten()? {
            StartupTunnelState::Starting =>
                Some(crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::Starting),
            StartupTunnelState::Running =>
                Some(crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::Running),
            StartupTunnelState::Stopping =>
                Some(crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::Stopping),
            StartupTunnelState::Stopped =>
                Some(crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::Stopped),
            StartupTunnelState::Failed =>
                Some(crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::Failed),
        }
    }

    /// Return startup definitions in deterministic name order.
    pub fn list(&self) -> Result<Vec<TunnelDefinition>, String> {
        let definitions = self
            .definitions
            .read()
            .map_err(|_| "startup tunnel inventory lock poisoned".to_string())?;
        Ok(definitions
            .values()
            .cloned()
            .map(|mut definition| {
                if let Some(state) = self.runtime_state(definition.name.as_str()) {
                    definition.runtime_state = state;
                }
                definition
            })
            .collect())
    }

    /// Return one startup definition by its exact configured name.
    pub fn get(&self, name: &str) -> Result<Option<TunnelDefinition>, String> {
        let definitions = self
            .definitions
            .read()
            .map_err(|_| "startup tunnel inventory lock poisoned".to_string())?;
        Ok(definitions.get(name).cloned().map(|mut definition| {
            if let Some(state) = self.runtime_state(name) {
                definition.runtime_state = state;
            }
            definition
        }))
    }

    /// Publish the actual destination exposed by a running server tunnel.
    pub fn publish_server_destination(&self, name: &str, destination: &str) -> Result<(), String> {
        let mut definitions = self
            .definitions
            .write()
            .map_err(|_| "startup tunnel inventory lock poisoned".to_string())?;
        let Some(definition) = definitions.get_mut(name) else {
            return Err("startup tunnel name is not configured".to_string());
        };
        if !matches!(
            definition.tunnel_type,
            TunnelType::Server | TunnelType::IrcServer
        ) {
            return Err("startup tunnel is not a server definition".to_string());
        }
        if destination.is_empty() {
            return Err("server tunnel destination is empty".to_string());
        }
        definition.options.hosting_destination = Some(destination.to_string());
        Ok(())
    }
}

fn insert_startup_definition(
    definitions: &mut BTreeMap<String, TunnelDefinition>,
    definition: TunnelDefinition,
) -> Result<(), String> {
    let name = definition.name.as_str().to_string();
    if definitions.insert(name, definition).is_some() {
        return Err("duplicate startup tunnel name across client and server configuration".into());
    }
    Ok(())
}

fn startup_client_definition(config: &StartupClientConfig) -> Result<TunnelDefinition, String> {
    let name = TunnelName::new(config.name.clone()).map_err(|error| error.to_string())?;
    Ok(TunnelDefinition {
        name,
        tunnel_type: TunnelType::Client,
        ownership: crate::i2pcontrol::domain::tunnel::TunnelOwnership::StartupManaged,
        runtime_state: crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::ExternallyManaged,
        start_intent: crate::i2pcontrol::domain::tunnel::StartIntent::StartOnLoad,
        options: crate::i2pcontrol::domain::tunnel::TunnelOptions {
            target_destination: Some(config.destination.clone()),
            target_port: config.destination_port,
            listen_interface: config.address.clone(),
            listen_port: Some(config.port),
            ..Default::default()
        },
        raw_config: BTreeMap::from([
            ("name".into(), serde_json::json!(config.name)),
            ("type".into(), serde_json::json!("client")),
        ]),
    })
}

fn startup_server_definition(config: &StartupServerConfig) -> Result<TunnelDefinition, String> {
    let name = TunnelName::new(config.name.clone()).map_err(|error| error.to_string())?;
    Ok(TunnelDefinition {
        name,
        tunnel_type: TunnelType::Server,
        ownership: crate::i2pcontrol::domain::tunnel::TunnelOwnership::StartupManaged,
        runtime_state: crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::ExternallyManaged,
        start_intent: crate::i2pcontrol::domain::tunnel::StartIntent::StartOnLoad,
        options: crate::i2pcontrol::domain::tunnel::TunnelOptions {
            listen_port: Some(config.port),
            // The destination is filled only after the existing server
            // manager obtains it from Yosemite Session::destination().
            hosting_destination: None,
            ..Default::default()
        },
        raw_config: BTreeMap::from([
            ("name".into(), serde_json::json!(config.name)),
            ("type".into(), serde_json::json!("server")),
        ]),
    })
}

// `Runtime` is used by `EventHandleMetrics<R>` below.

// --- EventMetrics abstraction -----------------------------------------------

/// Read-only metric snapshot interface used by the production adapter.
///
/// This trait decouples the production router-info adapter from the
/// concrete `Runtime` implementation of `EventHandle`, so the adapter can
/// be constructed and tested without requiring the runtime-specific
/// tokio/smol dependencies that ship with the core mock runtime.
#[allow(dead_code)]
pub trait EventMetrics: Send + Sync {
    /// Cumulative inbound transport bytes.
    fn transport_inbound_bytes(&self) -> u64;
    /// Cumulative outbound transport bytes.
    fn transport_outbound_bytes(&self) -> u64;
    /// Cumulative inbound transit bytes.
    fn transit_inbound_bytes(&self) -> u64;
    /// Cumulative outbound transit bytes.
    fn transit_outbound_bytes(&self) -> u64;
    /// Read the authoritative cumulative transit counter used by the
    /// request-independent RouterInfo sampler.
    ///
    /// Implementations without a real transit counter return `None`. The
    /// default preserves the existing metrics boundary for adapters that do
    /// not explicitly opt into the sampler.
    fn transit_bytes_snapshot(&self) -> Option<u64> {
        None
    }
    /// Number of currently connected routers.
    fn connected_routers(&self) -> usize;
    /// Number of active transit tunnels.
    fn transit_tunnel_count(&self) -> usize;
    /// Cumulative tunnel build successes.
    fn tunnel_build_successes(&self) -> u64;
    /// Cumulative tunnel build failures.
    fn tunnel_build_failures(&self) -> u64;
    /// Recent tunnel build success rate in reference-rounded percentage points.
    fn tunnel_build_success_rate(&self) -> f64 {
        0.0
    }
    /// Latest IPv4 firewall status.
    fn ipv4_firewall_status(&self) -> FirewallStatus;
    /// Latest IPv6 firewall status.
    fn ipv6_firewall_status(&self) -> FirewallStatus;
    /// Latest independently tracked IPv4 network state.
    fn ipv4_network_state(&self) -> NetworkState {
        NetworkState {
            status: self.ipv4_firewall_status(),
            ..NetworkState::default()
        }
    }
    /// Latest independently tracked IPv6 network state.
    fn ipv6_network_state(&self) -> NetworkState {
        NetworkState {
            status: self.ipv6_firewall_status(),
            ..NetworkState::default()
        }
    }
}

/// Adapter that wraps a concrete `EventHandle<R>` and implements
/// [`EventMetrics`].
pub struct EventHandleMetrics<R: Runtime> {
    handle: EventHandle<R>,
}

impl<R: Runtime> EventHandleMetrics<R> {
    /// Create a new metric adapter for the given event handle.
    pub fn new(handle: EventHandle<R>) -> Self {
        Self { handle }
    }
}

impl<R: Runtime> EventMetrics for EventHandleMetrics<R> {
    fn transport_inbound_bytes(&self) -> u64 {
        self.handle.transport_inbound_bytes()
    }
    fn transport_outbound_bytes(&self) -> u64 {
        self.handle.transport_outbound_bytes()
    }
    fn transit_inbound_bytes(&self) -> u64 {
        self.handle.transit_inbound_bytes()
    }
    fn transit_outbound_bytes(&self) -> u64 {
        self.handle.transit_outbound_bytes()
    }
    fn transit_bytes_snapshot(&self) -> Option<u64> {
        Some(self.handle.transit_outbound_bytes())
    }
    fn connected_routers(&self) -> usize {
        self.handle.connected_routers()
    }
    fn transit_tunnel_count(&self) -> usize {
        self.handle.transit_tunnel_count()
    }
    fn tunnel_build_successes(&self) -> u64 {
        self.handle.tunnel_build_successes()
    }
    fn tunnel_build_failures(&self) -> u64 {
        self.handle.tunnel_build_failures()
    }
    fn tunnel_build_success_rate(&self) -> f64 {
        self.handle.tunnel_build_success_rate()
    }
    fn ipv4_firewall_status(&self) -> FirewallStatus {
        self.handle.ipv4_firewall_status()
    }
    fn ipv6_firewall_status(&self) -> FirewallStatus {
        self.handle.ipv6_firewall_status()
    }

    fn ipv4_network_state(&self) -> NetworkState {
        self.handle.ipv4_network_state()
    }

    fn ipv6_network_state(&self) -> NetworkState {
        self.handle.ipv6_network_state()
    }
}

// --- Production ControlPlane -------------------------------------------------

/// Production control plane backed by real router state.
///
/// Provides identity, version, and uptime from the running router. All methods
/// are non-mutating and do not consume the EventSubscriber.
#[allow(dead_code)]
pub struct ProductionControlPlane {
    router_id_b64: String,
    version: String,
    startup: std::time::Instant,
    metrics: Arc<dyn EventMetrics>,
}

impl ProductionControlPlane {
    /// Create a new production control plane.
    pub fn new(router_id_b64: String, version: String, metrics: Arc<dyn EventMetrics>) -> Self {
        Self {
            router_id_b64,
            version,
            startup: std::time::Instant::now(),
            metrics,
        }
    }

    /// Access the underlying metrics source.
    #[allow(dead_code)]
    pub fn metrics(&self) -> &Arc<dyn EventMetrics> {
        &self.metrics
    }
}

impl ControlPlane for ProductionControlPlane {
    fn router_identity(&self) -> Result<String, String> {
        Ok(self.router_id_b64.clone())
    }

    fn router_uptime_ms(&self) -> u64 {
        self.startup.elapsed().as_millis() as u64
    }

    fn router_version(&self) -> String {
        self.version.clone()
    }
}

// --- Production AddressBookControl ------------------------------------------

/// Production address book control plane backed by the running router's
/// [`AddressBookHandle`]. The old I2PControl generation store is accepted only
/// as one-time migration input and is never retained as a second authority.
pub struct ProductionAddressBookControl {
    runtime: Arc<RuntimeAddressBookHandle>,
    legacy_dir: PathBuf,
}

impl ProductionAddressBookControl {
    /// Create a production adapter for the already-composed runtime owner.
    pub fn new(runtime: Arc<RuntimeAddressBookHandle>, legacy_dir: PathBuf) -> Self {
        Self {
            runtime,
            legacy_dir,
        }
    }

    /// Validate the runtime source and migrate the legacy administrative store
    /// before it can become a second authority.
    pub async fn load(&self) -> Result<(), String> {
        if let Some(error) = self.runtime.runtime_initialization_error() {
            return Err(error);
        }

        let destinations = self.runtime.legacy_destinations().await?;
        if self.runtime.runtime_authority_present() {
            self.runtime.runtime_clear_unsupported_configuration().await?;
            return self.runtime.repair_published_runtime_state(destinations).await;
        }

        let mut store = AddressBookStore::new(self.legacy_dir.clone(), 1024 * 1024);
        store.load().await.map_err(|e| format!("legacy store load: {e}"))?;

        let mut snapshot = RuntimeAddressBookSnapshot::default();
        for (source, target) in [
            (
                AdministrativeAddressBookType::Private,
                &mut snapshot.private,
            ),
            (AdministrativeAddressBookType::Local, &mut snapshot.local),
            (AdministrativeAddressBookType::Router, &mut snapshot.router),
            (
                AdministrativeAddressBookType::Published,
                &mut snapshot.published,
            ),
        ] {
            for entry in store.list(source) {
                target.insert(
                    entry.hostname.clone(),
                    RuntimeAddressBookEntry {
                        hostname: entry.hostname.clone(),
                        destination: entry.destination.clone(),
                    },
                );
            }
        }
        snapshot.subscriptions = store.subscriptions().as_slice().to_vec();
        self.runtime.import_legacy_runtime_state(snapshot, destinations).await
    }
}

impl Clone for ProductionAddressBookControl {
    fn clone(&self) -> Self {
        Self {
            runtime: Arc::clone(&self.runtime),
            legacy_dir: self.legacy_dir.clone(),
        }
    }
}

#[async_trait]
impl AddressBookControl for ProductionAddressBookControl {
    async fn list(
        &self,
        book_type: AdministrativeAddressBookType,
    ) -> Result<Vec<AddressBookEntry>, String> {
        Ok(self
            .runtime
            .runtime_list(runtime_book_type(book_type))
            .await?
            .into_iter()
            .map(runtime_entry)
            .collect())
    }

    async fn lookup(
        &self,
        book_type: AdministrativeAddressBookType,
        hostname: &str,
    ) -> Result<Option<AddressBookEntry>, String> {
        Ok(self
            .runtime
            .runtime_lookup(runtime_book_type(book_type), hostname)
            .await?
            .map(runtime_entry))
    }

    async fn add(
        &self,
        book_type: AdministrativeAddressBookType,
        entry: AddressBookEntry,
    ) -> Result<(), String> {
        self.runtime
            .runtime_add(runtime_book_type(book_type), runtime_entry_from(entry))
            .await
    }

    async fn update(
        &self,
        book_type: AdministrativeAddressBookType,
        entry: AddressBookEntry,
    ) -> Result<bool, String> {
        self.runtime
            .runtime_update(runtime_book_type(book_type), runtime_entry_from(entry))
            .await
    }

    async fn delete(
        &self,
        book_type: AdministrativeAddressBookType,
        hostname: &str,
    ) -> Result<bool, String> {
        self.runtime.runtime_delete(runtime_book_type(book_type), hostname).await
    }

    async fn delete_all(&self, book_type: AdministrativeAddressBookType) -> Result<bool, String> {
        self.runtime.runtime_delete_all(runtime_book_type(book_type)).await
    }

    async fn subscriptions(&self) -> Result<SubscriptionSet, String> {
        Ok(SubscriptionSet::from_vec(
            self.runtime.runtime_subscriptions().await?,
        ))
    }

    async fn set_subscriptions(&self, subscriptions: SubscriptionSet) -> Result<(), String> {
        self.runtime.runtime_set_subscriptions(subscriptions.as_slice().to_vec()).await
    }

    async fn configuration(&self) -> Result<AddressBookConfiguration, String> {
        Ok(AddressBookConfiguration::from_map(
            self.runtime.runtime_configuration().await?,
        ))
    }

    async fn set_configuration(
        &self,
        configuration: AddressBookConfiguration,
    ) -> Result<(), String> {
        self.runtime.runtime_set_configuration(configuration.as_map().clone()).await
    }
}

fn runtime_book_type(book_type: AdministrativeAddressBookType) -> RuntimeAddressBookType {
    match book_type {
        AdministrativeAddressBookType::Private => RuntimeAddressBookType::Private,
        AdministrativeAddressBookType::Local => RuntimeAddressBookType::Local,
        AdministrativeAddressBookType::Router => RuntimeAddressBookType::Router,
        AdministrativeAddressBookType::Published => RuntimeAddressBookType::Published,
    }
}

fn runtime_entry(entry: RuntimeAddressBookEntry) -> AddressBookEntry {
    AddressBookEntry::new(entry.hostname, entry.destination)
}

fn runtime_entry_from(entry: AddressBookEntry) -> RuntimeAddressBookEntry {
    RuntimeAddressBookEntry {
        hostname: entry.hostname,
        destination: entry.destination,
    }
}

// --- Production TunnelManagerControl ----------------------------------------

/// Production tunnel manager control plane backed by the persistent
/// [`TunnelStore`].
pub struct ProductionTunnelManagerControl {
    inner: Arc<tokio::sync::Mutex<TunnelStore>>,
    registry: TunnelBackendRegistry,
    startup: StartupTunnelInventory,
    server_destinations: ServerDestinationStore,
    client_destinations: ClientDestinationStore,
    sam_tcp_port: Option<u16>,
    lifecycle: Arc<tokio::sync::Mutex<BTreeMap<String, Arc<tokio::sync::Mutex<()>>>>>,
    #[cfg(test)]
    commit_hook: Arc<CommitPhaseTestHook>,
}

/// Bounded server-start transaction state.
///
/// `Fresh` carries a staged (not yet committed) identity whose durable
/// definition was never persisted. `Replacement` carries a staged candidate
/// shadowing the durable secret plus the retained previous secret bytes for
/// commit-phase restore. Both are bounded to one `start_locked` call.
enum ServerStartKind {
    NotServer,
    ExistingUnchanged,
    Fresh { identity: String },
    Replacement { identity: String, previous_private: String },
}

/// In-memory prepared server definition with its transaction evidence.
struct PreparedServerStart {
    definition: TunnelDefinition,
    kind: ServerStartKind,
    _guard: ServerStartGuard,
}

/// Cancellation guard for staged server secrets.
///
/// Held from staging through commit/rollback so cancellation, panic-unwind
/// cleanup, or an early explicit return still drops the staged candidate.
/// The synchronous `discard_sync` never blocks on network or file I/O; state
/// locks are held only for short in-memory updates.
struct ServerStartGuard {
    store: Option<(ServerDestinationStore, String)>,
}

impl Drop for ServerStartGuard {
    fn drop(&mut self) {
        if let Some((store, identity)) = self.store.take() {
            store.discard_sync(&identity);
        }
    }
}

impl ProductionTunnelManagerControl {
    /// Create a new production tunnel manager control plane.
    #[allow(dead_code)]
    pub fn new(dir: PathBuf) -> Result<Self, String> {
        Self::new_with_startup_inventory(dir, StartupTunnelInventory::default())
    }

    /// Create a production tunnel manager with the composed startup source.
    pub fn new_with_startup_inventory(
        dir: PathBuf,
        startup: StartupTunnelInventory,
    ) -> Result<Self, String> {
        Self::new_with_startup_inventory_and_sam_port(dir, startup, None)
    }

    /// Create a production tunnel manager with the existing router SAM
    /// endpoint. `None` retains the dependency-light test construction path.
    pub fn new_with_startup_inventory_and_sam_port(
        dir: PathBuf,
        startup: StartupTunnelInventory,
        sam_tcp_port: Option<u16>,
    ) -> Result<Self, String> {
        Self::new_with_startup_inventory_and_sam_port_and_address_book(
            dir,
            startup,
            sam_tcp_port,
            None,
        )
    }

    /// Create the production tunnel manager with the runtime address-book
    /// resolver used by dynamic HTTP/CONNECT client targets.
    pub fn new_with_startup_inventory_and_sam_port_and_address_book(
        dir: PathBuf,
        startup: StartupTunnelInventory,
        sam_tcp_port: Option<u16>,
        address_book: Option<Arc<RuntimeAddressBookHandle>>,
    ) -> Result<Self, String> {
        let state_root = dir.parent().unwrap_or(dir.as_path()).to_path_buf();
        let client_destinations = ClientDestinationStore::new(state_root.clone());
        let server_destinations = ServerDestinationStore::new(state_root);
        let shared_sessions = Arc::new(
            crate::i2pcontrol::backends::runtime::session::SharedClientSessionRegistry::new(),
        );
        let registry = match sam_tcp_port {
            Some(port) => {
                crate::i2pcontrol::backends::registry::create_production_registry_with_runtime(
                    port,
                    server_destinations.clone(),
                    address_book,
                    Some(shared_sessions),
                    Some(client_destinations.clone()),
                )
            }
            None => crate::i2pcontrol::backends::registry::create_default_registry(),
        }
        .map_err(|e| format!("failed to create registry: {e}"))?;
        Ok(Self {
            inner: Arc::new(tokio::sync::Mutex::new(TunnelStore::new(dir, 1024 * 1024))),
            registry,
            startup,
            server_destinations,
            client_destinations,
            sam_tcp_port,
            lifecycle: Arc::new(tokio::sync::Mutex::new(BTreeMap::new())),
            #[cfg(test)]
            commit_hook: Arc::new(CommitPhaseTestHook::new()),
        })
    }

    /// Load existing state from disk.
    pub async fn load(&self) -> Result<(), String> {
        self.server_destinations.load().await?;
        self.client_destinations.load().await?;
        let mut store = self.inner.lock().await;
        store.load().await.map_err(|e| format!("store load: {e}"))?;
        let referenced_server_identities = store
            .list()
            .iter()
            .filter(|definition| {
                matches!(
                    definition.tunnel_type,
                    TunnelType::Server
                        | TunnelType::HttpServer
                        | TunnelType::HttpBidirServer
                        | TunnelType::IrcServer
                        | TunnelType::StreamrServer
                )
            })
            .filter_map(|definition| {
                definition
                    .raw_config
                    .get(crate::i2pcontrol::backends::server::SERVER_IDENTITY_KEY)
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
            })
            .collect::<std::collections::BTreeSet<_>>();
        let startup_names = self
            .startup
            .list()?
            .into_iter()
            .map(|definition| definition.name.as_str().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        if store
            .list()
            .iter()
            .any(|definition| startup_names.contains(definition.name.as_str()))
        {
            return Err(
                "startup and persisted tunnel definitions contain a colliding name".to_string(),
            );
        }
        if startup_names.len().saturating_add(store.len()) > MAX_TUNNEL_INVENTORY {
            return Err(format!(
                "combined tunnel inventory exceeds maximum of {MAX_TUNNEL_INVENTORY} entries"
            ));
        }
        let referenced_client_names = store
            .list()
            .iter()
            .filter(|definition| definition.tunnel_type.is_client())
            .filter(|definition| {
                definition.options.persistent_client_key.unwrap_or(false)
                    || definition.options.priv_key_file.is_some()
            })
            .map(|definition| definition.name.as_str().to_owned())
            .collect::<std::collections::BTreeSet<_>>();
        drop(store);
        self.server_destinations
            .prune_unreferenced(&referenced_server_identities)
            .await?;
        self.client_destinations
            .prune_unreferenced(&referenced_client_names)
            .await?;
        self.reconcile_start_on_load().await;
        Ok(())
    }

    /// Start eligible persisted definitions after durable state and secrets
    /// have loaded. Each definition is isolated so one invalid runtime or
    /// unavailable SAM endpoint cannot prevent the administrative service from
    /// coming up for the remaining definitions.
    async fn reconcile_start_on_load(&self) {
        let definitions = {
            let store = self.inner.lock().await;
            store.list().into_iter().cloned().collect::<Vec<_>>()
        };

        for definition in definitions {
            if definition.ownership
                != crate::i2pcontrol::domain::tunnel::TunnelOwnership::ControlPlane
                || definition.start_intent
                    != crate::i2pcontrol::domain::tunnel::StartIntent::StartOnLoad
                || !matches!(
                    definition.tunnel_type,
                    TunnelType::Client
                        | TunnelType::HttpClient
                        | TunnelType::IrcClient
                        | TunnelType::Socks
                        | TunnelType::SocksIrc
                        | TunnelType::ConnectClient
                        | TunnelType::Server
                        | TunnelType::HttpServer
                        | TunnelType::HttpBidirServer
                        | TunnelType::IrcServer
                        | TunnelType::StreamrClient
                        | TunnelType::StreamrServer
                )
            {
                continue;
            }

            let name = definition.name.as_str().to_string();
            match self.start(&name).await {
                Ok(result) if result.starts_with("error") => tracing::warn!(
                    target: "emissary::i2pcontrol::tunnel_manager",
                    tunnel = %name,
                    "StartOnLoad tunnel did not start",
                ),
                Err(_) => tracing::warn!(
                    target: "emissary::i2pcontrol::tunnel_manager",
                    tunnel = %name,
                    "StartOnLoad tunnel could not be reconciled",
                ),
                _ => {}
            }
        }
    }

    /// Acquire all per-name lifecycle locks in deterministic order.
    ///
    /// The lock entries are deliberately retained for the bounded lifetime of
    /// this manager. This keeps rename and delete races serializable without
    /// holding the store lock across runtime awaits or introducing a lock
    /// reclamation race.
    async fn lifecycle_locks(&self, names: &[&str]) -> Vec<tokio::sync::OwnedMutexGuard<()>> {
        let mut names = names.iter().map(|name| (*name).to_string()).collect::<Vec<_>>();
        names.sort();
        names.dedup();
        let locks = {
            let mut lifecycle = self.lifecycle.lock().await;
            names
                .iter()
                .map(|name| {
                    lifecycle
                        .entry(name.clone())
                        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                        .clone()
                })
                .collect::<Vec<_>>()
        };
        let mut guards = Vec::with_capacity(locks.len());
        for lock in locks {
            guards.push(lock.lock_owned().await);
        }
        guards
    }

    async fn lifecycle_lock(&self, name: &str) -> tokio::sync::OwnedMutexGuard<()> {
        self.lifecycle_locks(&[name]).await.remove(0)
    }

    fn runtime_is_active(&self, definition: &TunnelDefinition) -> bool {
        if definition.ownership != crate::i2pcontrol::domain::tunnel::TunnelOwnership::ControlPlane
        {
            return false;
        }
        matches!(
            self.registry.get(definition.tunnel_type).inspect(definition).runtime_state,
            crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::Starting
                | crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::Running
                | crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::Stopping
        )
    }

    async fn start_locked(
        &self,
        name: &str,
        lifecycle: tokio::sync::OwnedMutexGuard<()>,
    ) -> Result<String, String> {
        if self.startup.get(name)?.is_some() {
            return self.startup_action(name, StartupTunnelAction::Start).await;
        }
        let definition = {
            let store = self.inner.lock().await;
            store
                .get(name)
                .ok_or_else(|| format!("error - tunnel '{name}' not found"))?
                .clone()
        };
        // Deterministic option validation precedes any secret allocation.
        if let Err(error) = validate_common_options(definition.tunnel_type, &definition.options) {
            return Ok(format!("error - {error}"));
        }
        // Pure backend preflight precedes generation/import/persistence.
        // No listener/session/task allocation, no network I/O, no secret or
        // runtime-map mutation happens inside `validate_start`.
        let backend = self.registry.get(definition.tunnel_type);
        match backend.validate_start(&definition) {
            Ok(()) => {}
            Err(BackendError::NotImplemented { tunnel_type }) => {
                return Ok(format!("error - {} not implemented", tunnel_type.as_str()));
            }
            Err(error) => return Ok(format!("error - {error}")),
        }
        if definition.tunnel_type.is_client() {
            self.client_destinations
                .stage(&definition, self.sam_tcp_port.ok_or_else(|| {
                    "client backend requires the router SAM listener".to_string()
                })?)
                .await?;
        }
        // Stage the server identity mutation without durable effects. The
        // returned guard clears any staged candidate if this future is
        // cancelled or dropped before commit, so a failed/uncommitted
        // replacement never becomes authoritative.
        let prepared = self.prepare_server_start(definition).await?;
        let backend = self.registry.get(prepared.definition.tunnel_type);
        match backend.start(&prepared.definition).await {
            Ok(()) => {
                if prepared.definition.tunnel_type.is_client() {
                    if let Err(error) = self
                        .client_destinations
                        .commit(prepared.definition.name.as_str())
                        .await
                    {
                        let _ = backend.stop(&prepared.definition).await;
                        self.client_destinations
                            .discard(prepared.definition.name.as_str())
                            .await;
                        return Ok(format!("error - {error}"));
                    }
                }
                if prepared.definition.tunnel_type.is_server() {
                    let status = backend.inspect(&prepared.definition);
                    let Some(destination) = status.destination else {
                        let _ = backend.stop(&prepared.definition).await;
                        self.rollback_server_start(prepared).await;
                        return Ok("error - server runtime did not publish a destination".into());
                    };
                    return self
                        .terminalize_server_start(prepared, destination, lifecycle)
                        .await;
                }
                Ok("ok".to_string())
            }
            Err(BackendError::NotImplemented { tunnel_type }) => {
                if prepared.definition.tunnel_type.is_client() {
                    self.client_destinations
                        .discard(prepared.definition.name.as_str())
                        .await;
                }
                if prepared.definition.tunnel_type.is_server() {
                    self.rollback_server_start(prepared).await;
                }
                Ok(format!("error - {} not implemented", tunnel_type.as_str()))
            }
            Err(error) => {
                if prepared.definition.tunnel_type.is_client() {
                    self.client_destinations
                        .discard(prepared.definition.name.as_str())
                        .await;
                }
                if prepared.definition.tunnel_type.is_server() {
                    self.rollback_server_start(prepared).await;
                }
                Ok(format!("error - {error}"))
            }
        }
    }

    async fn startup_action(
        &self,
        name: &str,
        action: StartupTunnelAction,
    ) -> Result<String, String> {
        let Some(lifecycle) = self.startup.lifecycle.as_ref() else {
            return Err("startup-managed tunnel lifecycle is externally managed".into());
        };
        lifecycle.apply(name, action).await.map(|()| "ok".to_string())
    }

    async fn with_runtime_state(&self, mut definition: TunnelDefinition) -> TunnelDefinition {
        if definition.ownership == crate::i2pcontrol::domain::tunnel::TunnelOwnership::ControlPlane
        {
            let status = self.registry.get(definition.tunnel_type).inspect(&definition);
            definition.runtime_state = status.runtime_state;
            if matches!(
                definition.tunnel_type,
                TunnelType::Server
                    | TunnelType::HttpServer
                    | TunnelType::HttpBidirServer
                    | TunnelType::IrcServer
                    | TunnelType::StreamrServer
            ) {
                let public_destination = if status.destination.is_some() {
                    status.destination
                } else {
                    let identity = definition
                        .raw_config
                        .get(crate::i2pcontrol::backends::server::SERVER_IDENTITY_KEY)
                        .and_then(|value| value.as_str());
                    let public = definition
                        .raw_config
                        .get(crate::i2pcontrol::backends::server::SERVER_PUBLIC_DESTINATION_KEY)
                        .and_then(|value| value.as_str());
                    match (identity, public) {
                        (Some(identity), Some(public))
                            if self
                                .server_destinations
                                .get(identity)
                                .await
                                .ok()
                                .flatten()
                                .is_some() =>
                            Some(public.to_string()),
                        _ => None,
                    }
                };
                definition.options.hosting_destination = public_destination;
            }
        }
        definition
    }

    /// Stage a server start without durable effects.
    ///
    /// Fresh identities are generated/imported into the secret-store staging
    /// area and given an in-memory identity key; the durable `TunnelStore`
    /// copy is left untouched until commit. Existing identities without a
    /// `PrivKeyFile` replacement need no staging. A `PrivKeyFile`
    /// replacement imports the candidate and stages it over the durable
    /// secret without overwriting durability until commit. No `TunnelStore`
    /// or secret-store durable mutation happens here.
    async fn prepare_server_start(
        &self,
        definition: TunnelDefinition,
    ) -> Result<PreparedServerStart, String> {
        if !definition.tunnel_type.is_server() {
            return Ok(PreparedServerStart {
                definition,
                kind: ServerStartKind::NotServer,
                _guard: ServerStartGuard { store: None },
            });
        }
        let identity = definition
            .raw_config
            .get(crate::i2pcontrol::backends::server::SERVER_IDENTITY_KEY)
            .and_then(|value| value.as_str())
            .map(str::to_string);
        if let Some(identity) = identity {
            if self.server_destinations.get(&identity).await?.is_none() {
                return Err("server destination identity is unavailable".to_string());
            }
            if let Some(reference) = definition.options.priv_key_file.as_deref() {
                let candidate = self.server_destinations.import_reference(reference).await?;
                let previous = self
                    .server_destinations
                    .get(&identity)
                    .await?
                    .ok_or_else(|| "server destination identity is unavailable".to_string())?;
                // Retain the previous secret bytes only for commit-phase
                // restore; durable state still holds the previous secret
                // until `commit` and no secret is logged or exposed.
                let previous_private = previous.as_str().to_owned();
                self.server_destinations.stage(&identity, candidate).await?;
                let guard = ServerStartGuard {
                    store: Some((self.server_destinations.clone(), identity.clone())),
                };
                return Ok(PreparedServerStart {
                    definition,
                    kind: ServerStartKind::Replacement {
                        identity,
                        previous_private,
                    },
                    _guard: guard,
                });
            }
            return Ok(PreparedServerStart {
                definition,
                kind: ServerStartKind::ExistingUnchanged,
                _guard: ServerStartGuard { store: None },
            });
        }

        let sam_tcp_port = self
            .sam_tcp_port
            .ok_or_else(|| "server backend requires the router SAM listener".to_string())?;
        let candidate = if let Some(reference) = definition.options.priv_key_file.as_deref() {
            self.server_destinations.import_reference(reference).await?
        } else {
            crate::tunnel_server::generate_persistent_destination(sam_tcp_port)
                .await
                .map_err(|_| "server destination generation failed".to_string())
                .map(StoredDestination::from_private)?
        };
        let identity = ServerDestinationStore::new_identity();
        self.server_destinations.stage(&identity, candidate).await?;
        let guard = ServerStartGuard {
            store: Some((self.server_destinations.clone(), identity.clone())),
        };
        let mut prepared = definition;
        prepared.raw_config.insert(
            crate::i2pcontrol::backends::server::SERVER_IDENTITY_KEY.to_string(),
            serde_json::json!(identity.clone()),
        );
        Ok(PreparedServerStart {
            definition: prepared,
            kind: ServerStartKind::Fresh { identity },
            _guard: guard,
        })
    }

    /// Drop staged state after a failed start before any commit.
    ///
    /// Durable secret and definition state are untouched for fresh starts
    /// (the identity was never persisted) and for replacements (the staged
    /// candidate shadowed durability without overwriting it).
    async fn rollback_server_start(&self, prepared: PreparedServerStart) {
        match &prepared.kind {
            ServerStartKind::Fresh { identity }
            | ServerStartKind::Replacement { identity, .. } => {
                self.server_destinations.discard(identity).await;
            }
            ServerStartKind::NotServer | ServerStartKind::ExistingUnchanged => {}
        }
        drop(prepared);
    }

    /// Transfer the prepared transaction and the per-name lifecycle owner to
    /// a bounded task before the first commit-phase await. The caller only
    /// awaits the result channel; cancelling that await cannot cancel the
    /// transaction or release lifecycle exclusion.
    async fn terminalize_server_start(
        &self,
        prepared: PreparedServerStart,
        destination: String,
        lifecycle: tokio::sync::OwnedMutexGuard<()>,
    ) -> Result<String, String> {
        let manager = self.clone();
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = manager.commit_server_start(prepared, destination).await;
            #[cfg(test)]
            manager.commit_hook.mark_terminalized();
            drop(lifecycle);
            let _ = result_tx.send(result);
        });
        match result_rx.await {
            Ok(Ok(())) => Ok("ok".to_string()),
            Ok(Err(error)) => Ok(format!("error - {error}")),
            Err(_) => Err("server start terminalization task exited".to_string()),
        }
    }

    /// Commit a staged server start after the runtime published a destination.
    ///
    /// Secret commit precedes durable definition persistence. Any failure
    /// stops the runtime and restores exact previous state: fresh commits
    /// remove a just-committed secret, replacement commits restore the
    /// retained previous secret, and the durable definition is only updated
    /// after the secret is safely committed.
    async fn commit_server_start(
        &self,
        prepared: PreparedServerStart,
        destination: String,
    ) -> Result<(), String> {
        let backend = self.registry.get(prepared.definition.tunnel_type);
        match &prepared.kind {
            ServerStartKind::NotServer | ServerStartKind::ExistingUnchanged => {
                let definition = prepared.definition.clone();
                #[cfg(test)]
                self.commit_hook
                    .pause(CommitBoundary::BeforeExistingDefinitionPersist)
                    .await;
                if let Err(error) = self
                    .persist_server_public_destination(definition.clone(), destination)
                    .await
                {
                    let _ = backend.stop(&definition).await;
                    return Err(error);
                }
                Ok(())
            }
            ServerStartKind::Fresh { identity } => {
                let identity = identity.clone();
                let definition = prepared.definition.clone();
                #[cfg(test)]
                self.commit_hook.pause(CommitBoundary::BeforeSecretCommit).await;
                if let Err(error) = self.server_destinations.commit(&identity).await {
                    let _ = backend.stop(&definition).await;
                    self.server_destinations.discard(&identity).await;
                    return Err(error);
                }
                #[cfg(test)]
                self.commit_hook
                    .pause(CommitBoundary::AfterFreshSecretCommit)
                    .await;
                if let Err(error) = self
                    .persist_server_public_destination(definition.clone(), destination)
                    .await
                {
                    let _ = backend.stop(&definition).await;
                    let _ = self.server_destinations.remove(&identity).await;
                    return Err(error);
                }
                Ok(())
            }
            ServerStartKind::Replacement {
                identity,
                previous_private,
            } => {
                let identity = identity.clone();
                let previous_private = previous_private.clone();
                let definition = prepared.definition.clone();
                #[cfg(test)]
                self.commit_hook.pause(CommitBoundary::BeforeSecretCommit).await;
                if let Err(error) = self.server_destinations.commit(&identity).await {
                    let _ = backend.stop(&definition).await;
                    self.server_destinations.discard(&identity).await;
                    return Err(error);
                }
                #[cfg(test)]
                self.commit_hook
                    .pause(CommitBoundary::AfterReplacementSecretCommit)
                    .await;
                if let Err(error) = self
                    .persist_server_public_destination(definition.clone(), destination)
                    .await
                {
                    let _ = backend.stop(&definition).await;
                    let _ = self
                        .server_destinations
                        .put(&identity, StoredDestination::from_private(previous_private))
                        .await;
                    return Err(error);
                }
                Ok(())
            }
        }
    }

    async fn persist_server_public_destination(
        &self,
        mut definition: TunnelDefinition,
        destination: String,
    ) -> Result<(), String> {
        definition.options.hosting_destination = Some(destination.clone());
        definition.raw_config.insert(
            crate::i2pcontrol::backends::server::SERVER_PUBLIC_DESTINATION_KEY.to_string(),
            serde_json::json!(destination),
        );
        let mut store = self.inner.lock().await;
        store
            .upsert(definition)
            .await
            .map_err(|error| format!("store upsert: {error}"))?;
        Ok(())
    }
}

impl Clone for ProductionTunnelManagerControl {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            registry: self.registry.clone(),
            startup: self.startup.clone(),
            server_destinations: self.server_destinations.clone(),
            client_destinations: self.client_destinations.clone(),
            sam_tcp_port: self.sam_tcp_port,
            lifecycle: Arc::clone(&self.lifecycle),
            #[cfg(test)]
            commit_hook: Arc::clone(&self.commit_hook),
        }
    }
}

#[async_trait]
impl TunnelManagerControl for ProductionTunnelManagerControl {
    async fn list(&self) -> Result<Vec<TunnelDefinition>, String> {
        let persisted = {
            let store = self.inner.lock().await;
            store.list().into_iter().cloned().collect::<Vec<_>>()
        };
        let startup = self.startup.list()?;
        let mut definitions = BTreeMap::new();
        for definition in startup {
            definitions.insert(definition.name.as_str().to_string(), definition);
        }
        for definition in persisted {
            let definition = self.with_runtime_state(definition).await;
            if definitions.insert(definition.name.as_str().to_string(), definition).is_some() {
                return Err(
                    "startup and persisted tunnel definitions contain a colliding name".into(),
                );
            }
        }
        if definitions.len() > MAX_TUNNEL_INVENTORY {
            return Err(format!(
                "combined tunnel inventory exceeds maximum of {MAX_TUNNEL_INVENTORY} entries"
            ));
        }
        Ok(definitions.into_values().collect())
    }

    async fn get(&self, name: &str) -> Result<Option<TunnelDefinition>, String> {
        if let Some(definition) = self.startup.get(name)? {
            return Ok(Some(definition));
        }
        let definition = {
            let store = self.inner.lock().await;
            store.get(name).cloned()
        };
        Ok(match definition {
            Some(definition) => Some(self.with_runtime_state(definition).await),
            None => None,
        })
    }

    async fn create(&self, definition: TunnelDefinition) -> Result<(), String> {
        if self.startup.get(definition.name.as_str())?.is_some() {
            return Err("error - tunnel name is owned by startup configuration".into());
        }
        let mut store = self.inner.lock().await;
        if store.len().saturating_add(self.startup.list()?.len()) >= MAX_TUNNEL_INVENTORY {
            return Err(format!(
                "combined tunnel inventory exceeds maximum of {MAX_TUNNEL_INVENTORY} entries"
            ));
        }
        if store.contains(definition.name.as_str()) {
            return Err(format!(
                "error - tunnel '{}' already exists",
                definition.name.as_str()
            ));
        }
        store.upsert(definition).await.map_err(|e| format!("store upsert: {e}"))?;
        Ok(())
    }

    async fn update(
        &self,
        name: &str,
        definition: TunnelDefinition,
        new_name: Option<TunnelName>,
    ) -> Result<bool, String> {
        let new_name_str = new_name.as_ref().map(TunnelName::as_str);
        let names = new_name_str.map_or_else(|| vec![name], |new_name| vec![name, new_name]);
        let _lifecycle = self.lifecycle_locks(&names).await;
        if self.startup.get(name)?.is_some() {
            return Err("error - tunnel name is owned by startup configuration".into());
        }
        if let Some(candidate) = new_name.as_ref() {
            if self.startup.get(candidate.as_str())?.is_some() {
                return Err("error - tunnel name is owned by startup configuration".into());
            }
        }
        let current = {
            let store = self.inner.lock().await;
            store.get(name).cloned()
        };
        let Some(current) = current else {
            return Ok(false);
        };
        if self.runtime_is_active(&current) {
            return Err("running tunnel edit is not supported; stop the tunnel first".into());
        }
        if let Some(ref nn) = new_name {
            let store = self.inner.lock().await;
            if nn.as_str() != name && store.contains(nn.as_str()) {
                return Err(format!(
                    "error - tunnel name '{}' already exists",
                    nn.as_str()
                ));
            }
        }
        let mut definition = definition;
        definition.runtime_state =
            self.registry.get(current.tunnel_type).inspect(&current).runtime_state;
        let renamed_client = if current.tunnel_type.is_client() {
            new_name
                .as_ref()
                .filter(|candidate| candidate.as_str() != name)
                .map(|candidate| candidate.as_str().to_owned())
        } else {
            None
        };
        if let Some(new_name) = renamed_client.as_deref() {
            self.client_destinations.rename(name, new_name).await?;
        }
        let mut store = self.inner.lock().await;
        let result = store
            .update(name, definition, new_name.as_ref().map(TunnelName::as_str))
            .await
            .map_err(|e| format!("store update: {e}"));
        if !matches!(&result, Ok(true)) {
            if let Some(new_name) = renamed_client.as_deref() {
                let _ = self.client_destinations.rename(new_name, name).await;
            }
        }
        result
    }

    async fn delete(&self, name: &str) -> Result<bool, String> {
        let _lifecycle = self.lifecycle_lock(name).await;
        if self.startup.get(name)?.is_some() {
            return Err("error - tunnel is managed by startup configuration".into());
        }
        let definition = {
            let store = self.inner.lock().await;
            store.get(name).cloned()
        };
        let Some(definition) = definition else {
            return Ok(false);
        };
        if self.runtime_is_active(&definition) {
            self.registry
                .get(definition.tunnel_type)
                .stop(&definition)
                .await
                .map_err(|error| error.to_string())?;
        }
        let identity = definition
            .raw_config
            .get(crate::i2pcontrol::backends::server::SERVER_IDENTITY_KEY)
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let removed = {
            let mut store = self.inner.lock().await;
            store.remove(name).await.map_err(|e| format!("store remove: {e}"))?
        };
        if removed.is_none() {
            return Ok(false);
        }
        if let Some(identity) = identity {
            if let Err(error) = self.server_destinations.remove(&identity).await {
                let mut store = self.inner.lock().await;
                let _ = store.upsert(definition).await;
                return Err(error);
            }
        }
        if definition.tunnel_type.is_client() {
            if let Err(error) = self.client_destinations.remove(name).await {
                let mut store = self.inner.lock().await;
                let _ = store.upsert(definition).await;
                return Err(error);
            }
        }
        Ok(true)
    }

    async fn start(&self, name: &str) -> Result<String, String> {
        let lifecycle = self.lifecycle_lock(name).await;
        if self.startup.get(name)?.is_some() {
            return self.startup_action(name, StartupTunnelAction::Start).await;
        }
        self.start_locked(name, lifecycle).await
    }

    async fn stop(&self, name: &str) -> Result<String, String> {
        let _lifecycle = self.lifecycle_lock(name).await;
        if self.startup.get(name)?.is_some() {
            return self.startup_action(name, StartupTunnelAction::Stop).await;
        }
        let def = {
            let store = self.inner.lock().await;
            store
                .get(name)
                .ok_or_else(|| format!("error - tunnel '{}' not found", name))?
                .clone()
        };
        let backend = self.registry.get(def.tunnel_type);
        match backend.stop(&def).await {
            Ok(()) => Ok("ok".to_string()),
            Err(e) => Ok(format!("error - {e}")),
        }
    }

    async fn restart(&self, name: &str) -> Result<String, String> {
        let lifecycle = self.lifecycle_lock(name).await;
        if self.startup.get(name)?.is_some() {
            return self.startup_action(name, StartupTunnelAction::Restart).await;
        }
        let definition = {
            let store = self.inner.lock().await;
            store
                .get(name)
                .ok_or_else(|| format!("error - tunnel '{}' not found", name))?
                .clone()
        };
        let backend = self.registry.get(definition.tunnel_type);
        if let Err(error) = backend.stop(&definition).await {
            return Ok(format!("error - {error}"));
        }
        // Reload after the exact stop. This prevents a restart from using a
        // stale pre-edit definition and keeps the old and new generations
        // strictly non-overlapping.
        self.start_locked(name, lifecycle).await
    }

    fn get_backend(&self, tunnel_type: TunnelType) -> Option<Arc<dyn TunnelBackend>> {
        if self.registry.contains(tunnel_type) {
            Some(self.registry.get(tunnel_type))
        } else {
            None
        }
    }

    fn registry(&self) -> &TunnelBackendRegistry {
        &self.registry
    }
}

// --- Production RouterInfoControl -------------------------------------------

/// Production router info inspection adapter backed by real router state.
///
/// Reads:
/// - Identity, version, and startup time from retained startup values
/// - Bandwidth counters and rolling-window traffic from event metrics
/// - Tunnel build success/failure counters from event metrics
/// - Network status from cached firewall status on event metrics
/// - I2PTunnel quick stats from the configured tunnel store
/// - Log entries from the shared `LogRing`
///
/// This adapter never mutates router state, never consumes the EventSubscriber,
/// and never exposes private key material.
pub struct ProductionRouterInfoControl {
    router_id_b64: String,
    version: String,
    startup: std::time::Instant,
    share_ratio: f64,
    configured_bandwidth_in: u64,
    configured_bandwidth_out: u64,
    metrics: Arc<dyn EventMetrics>,
    transit_bandwidth_sampler: Option<Arc<TransitBandwidthSampler>>,
    log_ring: Arc<LogRing>,
    tunnel_manager: Arc<dyn TunnelManagerControl>,
    peer_directory: Option<Arc<dyn PeerDirectorySource>>,
    active_peer_source: Option<Arc<dyn ActivePeerSource>>,
    tunnel_source: Option<Arc<dyn TunnelSource>>,
    router_news: Option<Arc<RouterNewsSource>>,
}

impl ProductionRouterInfoControl {
    /// Create a new production router info control adapter.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        router_id_b64: String,
        version: String,
        share_ratio: f64,
        configured_bandwidth_in: u64,
        configured_bandwidth_out: u64,
        metrics: Arc<dyn EventMetrics>,
        log_ring: Arc<LogRing>,
        tunnel_manager: Arc<dyn TunnelManagerControl>,
    ) -> Self {
        let transit_bandwidth_sampler = TransitBandwidthSampler::start(Arc::clone(&metrics));
        Self {
            router_id_b64,
            version,
            startup: std::time::Instant::now(),
            share_ratio,
            configured_bandwidth_in,
            configured_bandwidth_out,
            metrics,
            transit_bandwidth_sampler,
            log_ring,
            tunnel_manager,
            peer_directory: None,
            active_peer_source: None,
            tunnel_source: None,
            router_news: None,
        }
    }

    /// Attach the canonical live public peer directory source.
    pub fn with_peer_directory_source(mut self, source: Arc<dyn PeerDirectorySource>) -> Self {
        self.peer_directory = Some(source);
        self
    }

    /// Attach the canonical live transport source.
    pub fn with_active_peer_source(mut self, source: Arc<dyn ActivePeerSource>) -> Self {
        self.active_peer_source = Some(source);
        self
    }

    /// Attach the canonical live tunnel source.
    pub fn with_tunnel_source(mut self, source: Arc<dyn TunnelSource>) -> Self {
        self.tunnel_source = Some(source);
        self
    }

    /// Attach the optional, bounded signed news source owned by I2PControl.
    pub(crate) fn with_router_news_source(mut self, source: Arc<RouterNewsSource>) -> Self {
        self.router_news = Some(source);
        self
    }

    fn firewall_status_to_network(status: FirewallStatus) -> NetworkStatus {
        match status {
            FirewallStatus::Ok => NetworkStatus::Ok,
            FirewallStatus::Firewalled => NetworkStatus::Firewalled,
            FirewallStatus::SymmetricNat => NetworkStatus::SymmetricNat,
            FirewallStatus::Unknown => NetworkStatus::Unknown,
        }
    }
}

#[async_trait]
impl RouterInfoControl for ProductionRouterInfoControl {
    fn router_identity(&self) -> Result<String, InspectionError> {
        Ok(self.router_id_b64.clone())
    }

    fn router_version(&self) -> Result<String, InspectionError> {
        Ok(self.version.clone())
    }

    fn router_uptime_ms(&self) -> Result<u64, InspectionError> {
        Ok(self.startup.elapsed().as_millis() as u64)
    }

    async fn network_snapshot(&self) -> Result<NetworkSnapshot, InspectionError> {
        let ipv4_state = self.metrics.ipv4_network_state();
        let ipv6_state = self.metrics.ipv6_network_state();
        let ipv4 = Self::firewall_status_to_network(ipv4_state.status);
        let ipv6 = Self::firewall_status_to_network(ipv6_state.status);
        let firewalled = ipv4 == NetworkStatus::Firewalled || ipv6 == NetworkStatus::Firewalled;
        let hidden = ipv4 == NetworkStatus::Hidden || ipv6 == NetworkStatus::Hidden;
        Ok(NetworkSnapshot {
            ipv4_status: ipv4,
            ipv6_status: ipv6,
            ipv4_error: ipv4_state.error,
            ipv6_error: ipv6_state.error,
            ipv4_testing: ipv4_state.testing,
            ipv6_testing: ipv6_state.testing,
            firewalled,
            hidden,
            reachability_disabled: false,
        })
    }

    async fn clock_skew(&self) -> Result<ClockSkew, InspectionError> {
        Ok(ClockSkew::default())
    }

    async fn transport_bytes(&self) -> Result<TransportBytes, InspectionError> {
        Ok(TransportBytes {
            received: self.metrics.transport_inbound_bytes(),
            sent: self.metrics.transport_outbound_bytes(),
        })
    }

    async fn recent_transit_traffic(&self) -> Result<RecentTransitTraffic, InspectionError> {
        Err(InspectionError::Unavailable {
            group: InspectionGroup::TrafficMetrics,
        })
    }

    async fn transit_bandwidth_15s(&self) -> Result<u64, InspectionError> {
        self.transit_bandwidth_sampler
            .as_ref()
            .ok_or(InspectionError::UnavailableReason {
                group: InspectionGroup::TrafficMetrics,
                reason: "no authoritative cumulative transit source",
            })?
            .snapshot()
            .map_err(|reason| InspectionError::UnavailableReason {
                group: InspectionGroup::TrafficMetrics,
                reason,
            })
    }

    async fn transit_bytes(&self) -> Result<TransitBytes, InspectionError> {
        Ok(TransitBytes {
            received: self.metrics.transit_inbound_bytes(),
            sent: self.metrics.transit_outbound_bytes(),
        })
    }

    async fn tunnel_build_stats(&self) -> Result<TunnelBuildStats, InspectionError> {
        Ok(TunnelBuildStats {
            successes: self.metrics.tunnel_build_successes(),
            failures: self.metrics.tunnel_build_failures(),
        })
    }

    async fn recent_tunnel_success_rate(&self) -> Result<f64, InspectionError> {
        Ok(self.metrics.tunnel_build_success_rate())
    }

    async fn tunnel_summary(&self) -> Result<TunnelSummary, InspectionError> {
        let configured = self.tunnel_manager.list().await.map(|l| l.len()).map_err(|_| {
            InspectionError::QueryFailed {
                group: InspectionGroup::TunnelSummary,
            }
        })?;
        // active_participating comes from the live event-metrics transit tunnel
        // count. The handler rejects unsupported exploratory/client/queue
        // selectors before calling this method, so these fields are never
        // serialized as fabricated success values.
        let active_participating = self.metrics.transit_tunnel_count();
        Ok(TunnelSummary {
            active_participating,
            configured,
            exploratory_inbound: 0,
            exploratory_outbound: 0,
            client_inbound: 0,
            client_outbound: 0,
            queue_depth: 0,
        })
    }

    async fn tunnel_details(&self) -> Result<TunnelDetails, InspectionError> {
        let source = self.tunnel_source.as_ref().ok_or(InspectionError::Unavailable {
            group: InspectionGroup::TunnelSummary,
        })?;
        source.snapshot()
    }

    async fn netdb_snapshot(
        &self,
    ) -> Result<crate::i2pcontrol::router_info::NetDbSnapshot, InspectionError> {
        Err(InspectionError::Unavailable {
            group: InspectionGroup::NetDb,
        })
    }

    async fn udp_snapshot(
        &self,
    ) -> Result<crate::i2pcontrol::router_info::UdpSnapshot, InspectionError> {
        // UDP-specific active state requires a transport-specific canonical
        // source. The aggregate connected_routers count from EventMetrics does
        // not distinguish UDP from TCP connections. Return unavailable rather
        // than inferring cross-transport truth from an aggregate.
        Err(InspectionError::Unavailable {
            group: InspectionGroup::UdpTransport,
        })
    }

    async fn tcp_snapshot(
        &self,
    ) -> Result<crate::i2pcontrol::router_info::TcpSnapshot, InspectionError> {
        // TCP-specific active state requires a transport-specific canonical
        // source. No existing event-metric handle exposes TCP-only connected
        // peer state. Return unavailable rather than fabricate a value.
        Err(InspectionError::Unavailable {
            group: InspectionGroup::TcpTransport,
        })
    }

    async fn known_peers(&self) -> Result<Vec<PeerIdentity>, InspectionError> {
        let source = self.peer_directory.as_ref().ok_or(InspectionError::Unavailable {
            group: InspectionGroup::PeerList,
        })?;
        Ok(source
            .snapshot()?
            .peer_ids
            .into_iter()
            .map(|id| PeerIdentity {
                id,
                is_active: false,
            })
            .collect())
    }

    async fn active_peers(&self) -> Result<Vec<PeerIdentity>, InspectionError> {
        let source = self.active_peer_source.as_ref().ok_or(InspectionError::Unavailable {
            group: InspectionGroup::PeerList,
        })?;
        Ok(source
            .snapshot()?
            .peer_ids
            .into_iter()
            .map(|id| PeerIdentity {
                id,
                is_active: true,
            })
            .collect())
    }

    async fn peer_router_info(&self, peer_id: &str) -> Result<Option<String>, InspectionError> {
        let source = self.peer_directory.as_ref().ok_or(InspectionError::Unavailable {
            group: InspectionGroup::PeerLookup,
        })?;
        let snapshot = source.snapshot()?;
        Ok(snapshot.router_infos.get(peer_id).map(|bytes| base64_encode(bytes.clone())))
    }

    async fn peer_directory(&self) -> Result<PeerDirectorySnapshot, InspectionError> {
        let source = self.peer_directory.as_ref().ok_or(InspectionError::Unavailable {
            group: InspectionGroup::PeerList,
        })?;
        source.snapshot()
    }

    async fn banned_peers(&self) -> Result<Vec<BannedPeer>, InspectionError> {
        Ok(BANNED_PEER_SOURCE.snapshot())
    }

    async fn peer_limits(&self) -> Result<PeerLimits, InspectionError> {
        Err(InspectionError::Unavailable {
            group: InspectionGroup::PeerStats,
        })
    }

    async fn transport_limits(&self) -> Result<TransportLimits, InspectionError> {
        let source = self.active_peer_source.as_ref().ok_or(InspectionError::Unavailable {
            group: InspectionGroup::PeerStats,
        })?;
        let snapshot = source.snapshot()?;
        Ok(TransportLimits {
            ntcp_limit: snapshot.ntcp_limit,
            ssu_limit: snapshot.ssu_limit,
        })
    }

    async fn active_peer_stats(&self) -> Result<Vec<ActivePeerStats>, InspectionError> {
        let source = self.active_peer_source.as_ref().ok_or(InspectionError::Unavailable {
            group: InspectionGroup::PeerStats,
        })?;
        let mut stats = source.snapshot()?.stats;
        stats.sort_unstable_by(|left, right| left.peer_id.cmp(&right.peer_id));
        Ok(stats)
    }

    async fn i2ptunnel_stats(&self) -> Result<I2PTunnelStats, InspectionError> {
        let configured = self.tunnel_manager.list().await.map(|l| l.len()).map_err(|_| {
            InspectionError::QueryFailed {
                group: InspectionGroup::I2PTunnel,
            }
        })?;
        Ok(I2PTunnelStats {
            configured_count: configured,
        })
    }

    async fn log_snapshot(&self) -> Result<LogSnapshot, InspectionError> {
        let (entries, generation) = self.log_ring.snapshot();
        let owned: Vec<LogEntry> = entries
            .into_iter()
            .map(|e| LogEntry {
                timestamp_ms: e.timestamp_ms,
                level: e.level,
                target: e.target,
                message: e.message,
            })
            .collect();
        Ok(LogSnapshot {
            entries: owned,
            generation,
        })
    }

    async fn log_clear(&self) -> Result<(), InspectionError> {
        self.log_ring.clear();
        Ok(())
    }

    fn router_news(&self) -> Result<String, InspectionError> {
        self.router_news
            .as_ref()
            .ok_or(InspectionError::UnavailableReason {
                group: InspectionGroup::Retained,
                reason: "no router news owner",
            })?
            .snapshot()
            .map_err(Into::into)
    }

    async fn share_ratio(&self) -> Result<f64, InspectionError> {
        Ok(self.share_ratio)
    }

    async fn configured_bw_limits(&self) -> Result<(u64, u64), InspectionError> {
        Ok((self.configured_bandwidth_in, self.configured_bandwidth_out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::AddressBookConfig,
        i2pcontrol::address_book_runtime::{new_controlled_manager, RuntimeAddressBookType},
    };

    fn valid_destination(seed: u8) -> String {
        use emissary_core::crypto::{base64_encode, SigningPrivateKey};
        use emissary_util::runtime::tokio::Runtime as TokioRuntime;

        let key = SigningPrivateKey::from_bytes(&[seed; 32]).unwrap();
        base64_encode(
            emissary_core::primitives::Destination::new::<TokioRuntime>(key.public()).serialize(),
        )
    }

    #[tokio::test]
    async fn legacy_address_book_state_migrates_into_runtime_owner_once() {
        let base = tempfile::tempdir().unwrap().keep();
        let legacy_dir = base.join("addressbooks");
        let mut legacy = AddressBookStore::new(legacy_dir.clone(), 1024 * 1024);
        legacy
            .add(
                AdministrativeAddressBookType::Private,
                AddressBookEntry::new("legacy.i2p", valid_destination(1)),
            )
            .await
            .unwrap();

        let (manager, control) = new_controlled_manager(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let adapter = ProductionAddressBookControl::new(control.clone(), legacy_dir.clone());
        adapter.load().await.unwrap();
        assert_eq!(
            control.runtime_list(RuntimeAddressBookType::Private).await.unwrap()[0].hostname,
            "legacy.i2p"
        );
        assert!(control.runtime_authority_present());

        drop(manager);
        let (_manager, control) = new_controlled_manager(
            base,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let adapter = ProductionAddressBookControl::new(control.clone(), legacy_dir);
        adapter.load().await.unwrap();
        assert_eq!(
            control.runtime_list(RuntimeAddressBookType::Private).await.unwrap().len(),
            1
        );
    }

    #[tokio::test]
    async fn production_setters_apply_operational_configuration() {
        use crate::i2pcontrol::control_plane::AddressBookControl;

        let base = tempfile::tempdir().unwrap().keep();
        let (_manager, control) = new_controlled_manager(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;

        let adapter = ProductionAddressBookControl::new(control.clone(), base.join("addressbooks"));

        let mut config = AddressBookConfiguration::new();
        config.insert(
            "private_addressbook".to_string(),
            "chosen-by-request".to_string(),
        );
        adapter.set_configuration(config).await.unwrap();
        assert_eq!(
            control.runtime_configuration().await.unwrap().get("private_addressbook"),
            Some(&"chosen-by-request".to_string())
        );

        let subscriptions =
            SubscriptionSet::from_vec(vec!["https://example.i2p/hosts.txt".to_string()]);
        assert!(adapter.set_subscriptions(subscriptions).await.is_err());
        assert!(control.runtime_subscriptions().await.unwrap().is_empty());
        assert!(control.runtime_authority_present());
    }

    #[tokio::test]
    async fn legacy_configuration_is_not_promoted_into_runtime_owner() {
        let base = tempfile::tempdir().unwrap().keep();
        let legacy_dir = base.join("addressbooks");
        let mut legacy = AddressBookStore::new(legacy_dir.clone(), 1024 * 1024);
        let mut configuration = AddressBookConfiguration::new();
        configuration.insert("theme".to_string(), "light".to_string());
        legacy.set_configuration(configuration).await.unwrap();

        let (_manager, control) = new_controlled_manager(
            base,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let adapter = ProductionAddressBookControl::new(control.clone(), legacy_dir);
        adapter.load().await.unwrap();
        assert!(control.runtime_configuration().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn legacy_address_book_migration_preserves_cross_book_shadowing() {
        let base = tempfile::tempdir().unwrap().keep();
        let legacy_dir = base.join("addressbooks");
        let published_destination = valid_destination(3);
        let private_destination = valid_destination(2);
        tokio::fs::create_dir_all(base.join("addressbook")).await.unwrap();
        tokio::fs::write(
            base.join("addressbook/addresses"),
            "collision.i2p=collision-base32\n",
        )
        .await
        .unwrap();
        tokio::fs::create_dir_all(base.join("addressbook/destinations")).await.unwrap();
        tokio::fs::write(
            base.join("addressbook/destinations/collision.i2p.txt"),
            &published_destination,
        )
        .await
        .unwrap();

        let mut legacy = AddressBookStore::new(legacy_dir.clone(), 1024 * 1024);
        legacy
            .add(
                AdministrativeAddressBookType::Private,
                AddressBookEntry::new("collision.i2p", private_destination.clone()),
            )
            .await
            .unwrap();

        let (_manager, control) = new_controlled_manager(
            base,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let adapter = ProductionAddressBookControl::new(control.clone(), legacy_dir);
        adapter.load().await.unwrap();
        assert_eq!(
            control
                .runtime_lookup(RuntimeAddressBookType::Private, "collision.i2p")
                .await
                .unwrap()
                .unwrap()
                .destination,
            private_destination
        );
        assert_eq!(
            control
                .runtime_lookup(RuntimeAddressBookType::Published, "collision.i2p")
                .await
                .unwrap()
                .unwrap()
                .destination,
            published_destination
        );
        assert_eq!(
            control.owner.resolve_base64("collision.i2p"),
            Some(private_destination)
        );
        let selectors = crate::i2pcontrol::address_book::resolve_address_book_selectors(
            &adapter,
            &[
                crate::i2pcontrol::rpc::router_info_keys::ADDRESS_BOOK_PRIVATE,
                crate::i2pcontrol::rpc::router_info_keys::ADDRESS_BOOK_PUBLISHED,
            ],
        )
        .await
        .unwrap();
        assert_eq!(
            selectors[crate::i2pcontrol::rpc::router_info_keys::ADDRESS_BOOK_PRIVATE]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            selectors[crate::i2pcontrol::rpc::router_info_keys::ADDRESS_BOOK_PUBLISHED]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn first_activation_imports_full_destinations_for_api_and_router_info() {
        use crate::i2pcontrol::{
            address_book::resolve_address_book_selectors, control_plane::AddressBookControl,
        };
        use emissary_core::crypto::{base32_encode, base64_decode};

        let base = tempfile::tempdir().unwrap().keep();
        let destination = valid_destination(10);
        let base32 = base32_encode(
            emissary_core::primitives::Destination::parse(base64_decode(&destination).unwrap())
                .unwrap()
                .id()
                .to_vec(),
        );
        tokio::fs::create_dir_all(base.join("addressbook/destinations")).await.unwrap();
        tokio::fs::write(
            base.join("addressbook/addresses"),
            format!("first.i2p={base32}\n"),
        )
        .await
        .unwrap();
        tokio::fs::write(
            base.join("addressbook/destinations/first.i2p.txt"),
            &destination,
        )
        .await
        .unwrap();

        let (_manager, control) = new_controlled_manager(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let adapter = ProductionAddressBookControl::new(control.clone(), base.join("addressbooks"));
        adapter.load().await.unwrap();

        assert_eq!(
            adapter
                .lookup(AdministrativeAddressBookType::Published, "first.i2p")
                .await
                .unwrap()
                .unwrap()
                .destination,
            destination
        );
        let selectors = resolve_address_book_selectors(
            &adapter,
            &[crate::i2pcontrol::rpc::router_info_keys::ADDRESS_BOOK_PUBLISHED],
        )
        .await
        .unwrap();
        assert_eq!(
            selectors[crate::i2pcontrol::rpc::router_info_keys::ADDRESS_BOOK_PUBLISHED][0]["value"],
            destination
        );
    }

    #[tokio::test]
    async fn persisted_base32_seed_is_repaired_from_matching_destination_file() {
        use emissary_core::crypto::{base32_encode, base64_decode};

        let base = tempfile::tempdir().unwrap().keep();
        let destination = valid_destination(11);
        let base32 = base32_encode(
            emissary_core::primitives::Destination::parse(base64_decode(&destination).unwrap())
                .unwrap()
                .id()
                .to_vec(),
        );
        let state = RuntimeAddressBookSnapshot {
            published: BTreeMap::from([(
                "repair.i2p".to_string(),
                RuntimeAddressBookEntry {
                    hostname: "repair.i2p".to_string(),
                    destination: base32,
                },
            )]),
            ..RuntimeAddressBookSnapshot::default()
        };
        tokio::fs::create_dir_all(base.join("addressbook/destinations")).await.unwrap();
        tokio::fs::write(
            base.join("addressbook/control-state.json"),
            serde_json::to_vec(&state).unwrap(),
        )
        .await
        .unwrap();
        tokio::fs::write(
            base.join("addressbook/destinations/repair.i2p.txt"),
            &destination,
        )
        .await
        .unwrap();

        let (_manager, control) = new_controlled_manager(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        ProductionAddressBookControl::new(control.clone(), base.join("addressbooks"))
            .load()
            .await
            .unwrap();
        assert_eq!(
            control
                .runtime_lookup(RuntimeAddressBookType::Published, "repair.i2p")
                .await
                .unwrap()
                .unwrap()
                .destination,
            destination
        );
    }

    #[tokio::test]
    async fn unrepairable_published_seed_fails_without_mutating_state() {
        let base = tempfile::tempdir().unwrap().keep();
        let state = serde_json::json!({
            "private": {},
            "local": {},
            "router": {},
            "published": {
                "broken.i2p": {
                    "hostname": "broken.i2p",
                    "destination": "not-a-destination"
                }
            },
            "subscriptions": [],
            "configuration": {}
        });
        tokio::fs::create_dir_all(base.join("addressbook")).await.unwrap();
        let state_bytes = serde_json::to_vec(&state).unwrap();
        tokio::fs::write(base.join("addressbook/control-state.json"), &state_bytes)
            .await
            .unwrap();

        let (_manager, control) = new_controlled_manager(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let error = ProductionAddressBookControl::new(control, base.join("addressbooks"))
            .load()
            .await
            .unwrap_err();
        assert!(error.contains("unrepairable"));
        assert_eq!(
            tokio::fs::read(base.join("addressbook/control-state.json")).await.unwrap(),
            state_bytes
        );
    }

    #[tokio::test]
    async fn reenable_does_not_resurrect_deleted_published_entry() {
        use emissary_core::crypto::base32_encode;

        let base = tempfile::tempdir().unwrap().keep();
        let destination = valid_destination(12);
        let base32 = base32_encode(
            emissary_core::primitives::Destination::parse(
                emissary_core::crypto::base64_decode(&destination).unwrap(),
            )
            .unwrap()
            .id()
            .to_vec(),
        );
        tokio::fs::create_dir_all(base.join("addressbook/destinations")).await.unwrap();
        tokio::fs::write(
            base.join("addressbook/addresses"),
            format!("deleted.i2p={base32}\n"),
        )
        .await
        .unwrap();
        tokio::fs::write(
            base.join("addressbook/destinations/deleted.i2p.txt"),
            &destination,
        )
        .await
        .unwrap();

        let (manager, control) = new_controlled_manager(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let adapter = ProductionAddressBookControl::new(control.clone(), base.join("addressbooks"));
        adapter.load().await.unwrap();
        assert!(control
            .runtime_delete(RuntimeAddressBookType::Published, "deleted.i2p")
            .await
            .unwrap());
        drop(manager);

        let (_manager, control) = new_controlled_manager(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        ProductionAddressBookControl::new(control.clone(), base.join("addressbooks"))
            .load()
            .await
            .unwrap();
        assert!(control
            .runtime_lookup(RuntimeAddressBookType::Published, "deleted.i2p")
            .await
            .unwrap()
            .is_none());
    }

    // --- M120 server preallocation + secret transactionality regressions ---

    fn m120_server_definition(name: &str, tunnel_type: TunnelType) -> TunnelDefinition {
        use crate::i2pcontrol::domain::tunnel::{
            StartIntent, TunnelName, TunnelOptions, TunnelOwnership, TunnelRuntimeState,
        };
        let mut options = TunnelOptions::default();
        match tunnel_type {
            TunnelType::Server | TunnelType::IrcServer => {
                options.target_port = Some(6667);
            }
            TunnelType::HttpServer => {
                options.target_port = Some(8080);
            }
            TunnelType::HttpBidirServer => {
                options.target_port = Some(8080);
                options.listen_port = Some(0);
            }
            TunnelType::StreamrServer => {
                options.listen_port = Some(0);
            }
            _ => panic!("m120 helper only builds server families"),
        }
        TunnelDefinition {
            name: TunnelName::new(name).unwrap(),
            tunnel_type,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options,
            raw_config: Default::default(),
        }
    }

    fn m120_secret(seed: u8) -> String {
        emissary_core::crypto::base64_encode([seed; 128])
    }

    fn m120_write_import(state_root: &std::path::Path, filename: &str, secret_b64: &str) {
        std::fs::create_dir_all(state_root.join("server-key-imports")).unwrap();
        std::fs::write(state_root.join("server-key-imports").join(filename), secret_b64).unwrap();
    }

    async fn m120_counting_sam() -> (
        u16,
        std::sync::Arc<std::sync::atomic::AtomicUsize>,
        tokio::task::JoinHandle<()>,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = count.clone();
        let task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                drop(stream);
            }
        });
        (port, count, task)
    }

    async fn m120_succeeding_sam() -> (u16, tokio::task::JoinHandle<()>) {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            break;
                        }
                        if line.starts_with("HELLO") {
                            if write_half
                                .write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n")
                                .await
                                .is_err()
                            {
                                break;
                            }
                        } else if line.starts_with("DEST GENERATE") {
                            let private = emissary_core::crypto::base64_encode([9u8; 128]);
                            let reply = format!("DEST REPLY PUB=destination PRIV={private}\n");
                            if write_half.write_all(reply.as_bytes()).await.is_err() {
                                break;
                            }
                        } else if line.starts_with("SESSION CREATE") {
                            if write_half
                                .write_all(
                                    b"SESSION STATUS RESULT=OK DESTINATION=server-destination\n",
                                )
                                .await
                                .is_err()
                            {
                                break;
                            }
                        } else if write_half.write_all(b"STREAM STATUS RESULT=OK\n").await.is_err() {
                            break;
                        }
                    }
                });
            }
        });
        (port, task)
    }

    async fn m120_failing_sam() -> (u16, tokio::task::JoinHandle<()>) {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            break;
                        }
                        if line.starts_with("HELLO") {
                            if write_half
                                .write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n")
                                .await
                                .is_err()
                            {
                                break;
                            }
                        } else if line.starts_with("DEST GENERATE") {
                            let private = emissary_core::crypto::base64_encode([9u8; 128]);
                            let reply = format!("DEST REPLY PUB=destination PRIV={private}\n");
                            if write_half.write_all(reply.as_bytes()).await.is_err() {
                                break;
                            }
                        } else {
                            // Fail session establishment fast: close without a
                            // response so `start` returns without timeout.
                            break;
                        }
                    }
                });
            }
        });
        (port, task)
    }

    async fn m120_manager_with_sam(
        tmp: &tempfile::TempDir,
        sam_port: u16,
    ) -> ProductionTunnelManagerControl {
        let manager = ProductionTunnelManagerControl::new_with_startup_inventory_and_sam_port(
            tmp.path().join("tunnels"),
            StartupTunnelInventory::default(),
            Some(sam_port),
        )
        .unwrap();
        manager.load().await.unwrap();
        manager
    }

    fn m120_identity_of(definition: &TunnelDefinition) -> Option<String> {
        definition
            .raw_config
            .get(crate::i2pcontrol::backends::server::SERVER_IDENTITY_KEY)
            .and_then(|value| value.as_str())
            .map(str::to_string)
    }

    #[tokio::test]
    async fn m120_common_option_fails_before_secret_allocation_for_all_server_families() {
        use crate::i2pcontrol::domain::tunnel::TunnelType;
        for tunnel_type in [
            TunnelType::Server,
            TunnelType::HttpServer,
            TunnelType::HttpBidirServer,
            TunnelType::IrcServer,
            TunnelType::StreamrServer,
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let (sam_port, sam_count, sam_task) = m120_counting_sam().await;
            let manager = m120_manager_with_sam(&tmp, sam_port).await;
            let mut definition = m120_server_definition("fresh-common", tunnel_type);
            definition.options.use_ssl = Some(true);
            manager.create(definition).await.unwrap();

            let result = manager.start("fresh-common").await.unwrap();
            assert!(
                result.contains("UseSSL"),
                "{tunnel_type} common rejection must name UseSSL, got: {result}"
            );
            let stored = manager.get("fresh-common").await.unwrap().unwrap();
            assert!(
                m120_identity_of(&stored).is_none(),
                "{tunnel_type} must not allocate an identity key"
            );
            assert!(
                !stored.raw_config.contains_key(
                    crate::i2pcontrol::backends::server::SERVER_PUBLIC_DESTINATION_KEY,
                ),
                "{tunnel_type} must not publish a destination"
            );
            assert_eq!(
                manager.server_destinations.staged_count().await,
                0,
                "{tunnel_type} must leave no staged secret"
            );
            assert_eq!(
                sam_count.load(std::sync::atomic::Ordering::SeqCst),
                0,
                "{tunnel_type} must not contact SAM before validation"
            );
            assert!(
                !result.contains("true"),
                "{tunnel_type} error must not echo option values"
            );
            sam_task.abort();
        }
    }

    #[tokio::test]
    async fn m120_raw_option_fails_before_generation_for_each_server_shape() {
        use crate::i2pcontrol::domain::tunnel::TunnelType;
        let cases = [
            (TunnelType::Server, "SignatureType"),
            (TunnelType::HttpServer, "TargetDestination"),
            (TunnelType::HttpBidirServer, "SignatureType"),
            (TunnelType::IrcServer, "SignatureType"),
            (TunnelType::StreamrServer, "SigType"),
        ];
        for (tunnel_type, bad_key) in cases {
            let tmp = tempfile::tempdir().unwrap();
            let (sam_port, sam_count, sam_task) = m120_counting_sam().await;
            let manager = m120_manager_with_sam(&tmp, sam_port).await;
            let mut definition = m120_server_definition("fresh-raw", tunnel_type);
            definition.raw_config.insert(bad_key.to_owned(), serde_json::json!("bogus"));
            manager.create(definition).await.unwrap();

            let result = manager.start("fresh-raw").await.unwrap();
            assert!(
                result.contains(bad_key),
                "{tunnel_type} raw rejection must name {bad_key}, got: {result}"
            );
            let stored = manager.get("fresh-raw").await.unwrap().unwrap();
            assert!(m120_identity_of(&stored).is_none());
            assert_eq!(manager.server_destinations.staged_count().await, 0);
            assert_eq!(sam_count.load(std::sync::atomic::Ordering::SeqCst), 0);
            sam_task.abort();
        }
    }

    #[tokio::test]
    async fn m120_i2cp_and_import_order_fail_before_allocation() {
        // Deterministic I2CP rejection precedes generation.
        let tmp = tempfile::tempdir().unwrap();
        let (sam_port, sam_count, sam_task) = m120_counting_sam().await;
        let manager = m120_manager_with_sam(&tmp, sam_port).await;
        let mut definition =
            m120_server_definition("fresh-i2cp", crate::i2pcontrol::domain::tunnel::TunnelType::Server);
        definition.options.i2cp_options.insert("bogus".to_owned(), "1".to_owned());
        manager.create(definition).await.unwrap();
        let result = manager.start("fresh-i2cp").await.unwrap();
        assert!(result.contains("I2CPOptions"), "I2CP rejection, got: {result}");
        assert!(m120_identity_of(&manager.get("fresh-i2cp").await.unwrap().unwrap()).is_none());
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        assert_eq!(sam_count.load(std::sync::atomic::Ordering::SeqCst), 0);
        sam_task.abort();

        // Validation precedes import: a missing import file plus an invalid
        // option must report the option, not the import failure.
        let tmp = tempfile::tempdir().unwrap();
        let (sam_port, _, sam_task) = m120_counting_sam().await;
        let manager = m120_manager_with_sam(&tmp, sam_port).await;
        let mut definition = m120_server_definition(
            "fresh-import-order",
            crate::i2pcontrol::domain::tunnel::TunnelType::Server,
        );
        definition.options.use_ssl = Some(true);
        definition.options.priv_key_file = Some("missing.key".to_owned());
        manager.create(definition).await.unwrap();
        let result = manager.start("fresh-import-order").await.unwrap();
        assert!(
            result.contains("UseSSL"),
            "validation must precede import, got: {result}"
        );
        assert!(!result.contains("missing.key"));
        assert!(
            m120_identity_of(
                &manager.get("fresh-import-order").await.unwrap().unwrap()
            )
            .is_none()
        );
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        sam_task.abort();
    }

    #[tokio::test]
    async fn m120_failed_start_never_persists_identity_keys() {
        let tmp = tempfile::tempdir().unwrap();
        let (sam_port, _, sam_task) = m120_counting_sam().await;
        let manager = m120_manager_with_sam(&tmp, sam_port).await;
        let mut definition = m120_server_definition(
            "no-identity",
            crate::i2pcontrol::domain::tunnel::TunnelType::Server,
        );
        definition.raw_config.insert("SignatureType".to_owned(), serde_json::json!("x"));
        manager.create(definition).await.unwrap();
        let result = manager.start("no-identity").await.unwrap();
        assert!(result.starts_with("error"));
        let stored = manager.get("no-identity").await.unwrap().unwrap();
        assert!(m120_identity_of(&stored).is_none());
        assert!(
            !stored.raw_config.contains_key(
                crate::i2pcontrol::backends::server::SERVER_PUBLIC_DESTINATION_KEY,
            )
        );
        // Reload proves nothing durable was written.
        drop(manager);
        let reloaded = ProductionTunnelManagerControl::new_with_startup_inventory_and_sam_port(
            tmp.path().join("tunnels"),
            StartupTunnelInventory::default(),
            Some(sam_port),
        )
        .unwrap();
        reloaded.load().await.unwrap();
        let stored = reloaded.get("no-identity").await.unwrap().unwrap();
        assert!(m120_identity_of(&stored).is_none());
        sam_task.abort();
    }

    #[tokio::test]
    async fn m120_existing_replacement_failure_restores_previous_secret() {
        let secret_old = m120_secret(0xA1);
        let secret_new = m120_secret(0xB2);
        assert_ne!(secret_old, secret_new);

        // Commit the original identity with a succeeding SAM.
        let tmp = tempfile::tempdir().unwrap();
        let (good_port, good_task) = m120_succeeding_sam().await;
        let manager = m120_manager_with_sam(&tmp, good_port).await;
        m120_write_import(tmp.path(), "old.key", &secret_old);
        let mut definition = m120_server_definition(
            "replace-server",
            crate::i2pcontrol::domain::tunnel::TunnelType::Server,
        );
        definition.options.priv_key_file = Some("old.key".to_owned());
        manager.create(definition).await.unwrap();
        assert_eq!(manager.start("replace-server").await.unwrap(), "ok");
        let committed = manager.get("replace-server").await.unwrap().unwrap();
        let identity = m120_identity_of(&committed).expect("identity committed");
        assert_eq!(
            manager.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            secret_old
        );
        manager.stop("replace-server").await.unwrap();
        drop(manager);
        good_task.abort();

        // Fail the replacement with a SAM that closes session setup.
        let (bad_port, bad_task) = m120_failing_sam().await;
        let manager = m120_manager_with_sam(&tmp, bad_port).await;
        m120_write_import(tmp.path(), "new.key", &secret_new);
        {
            let mut store = manager.inner.lock().await;
            let mut definition = store.get("replace-server").cloned().unwrap();
            definition.options.priv_key_file = Some("new.key".to_owned());
            store.upsert(definition).await.unwrap();
        }
        let result = manager.start("replace-server").await.unwrap();
        assert!(result.starts_with("error"), "replacement must fail, got: {result}");
        assert!(!result.contains(&secret_old));
        assert!(!result.contains(&secret_new));
        assert!(!result.contains("new.key"));
        // Exact previous secret and durable definition are restored.
        assert_eq!(
            manager.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            secret_old
        );
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        let stored = manager.get("replace-server").await.unwrap().unwrap();
        assert_eq!(m120_identity_of(&stored).as_deref(), Some(identity.as_str()));
        bad_task.abort();
    }

    #[tokio::test]
    async fn m120_fresh_import_failure_leaves_no_secret_or_definition() {
        let secret_new = m120_secret(0xC3);
        let tmp = tempfile::tempdir().unwrap();
        let (bad_port, bad_task) = m120_failing_sam().await;
        let manager = m120_manager_with_sam(&tmp, bad_port).await;
        m120_write_import(tmp.path(), "fresh.key", &secret_new);
        let mut definition = m120_server_definition(
            "fresh-failure",
            crate::i2pcontrol::domain::tunnel::TunnelType::Server,
        );
        definition.options.priv_key_file = Some("fresh.key".to_owned());
        manager.create(definition).await.unwrap();

        let result = manager.start("fresh-failure").await.unwrap();
        assert!(result.starts_with("error"), "fresh start must fail, got: {result}");
        assert!(!result.contains(&secret_new));
        let stored = manager.get("fresh-failure").await.unwrap().unwrap();
        assert!(m120_identity_of(&stored).is_none());
        assert!(
            !stored.raw_config.contains_key(
                crate::i2pcontrol::backends::server::SERVER_PUBLIC_DESTINATION_KEY,
            )
        );
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        assert!(
            !tmp.path().join("server-destinations").join("current.json").exists(),
            "no durable secret may be written before commit"
        );
        bad_task.abort();
    }

    #[tokio::test]
    async fn m120_fresh_generated_failure_removes_secret_and_restores_definition() {
        let tmp = tempfile::tempdir().unwrap();
        // Generation succeeds (DEST GENERATE) but session setup fails.
        let (bad_port, bad_task) = m120_failing_sam().await;
        let manager = m120_manager_with_sam(&tmp, bad_port).await;
        let definition = m120_server_definition(
            "fresh-generated",
            crate::i2pcontrol::domain::tunnel::TunnelType::Server,
        );
        manager.create(definition).await.unwrap();

        let result = manager.start("fresh-generated").await.unwrap();
        assert!(result.starts_with("error"), "generated start must fail, got: {result}");
        let stored = manager.get("fresh-generated").await.unwrap().unwrap();
        assert!(m120_identity_of(&stored).is_none());
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        assert!(
            !tmp.path().join("server-destinations").join("current.json").exists(),
            "failed generation must not leave a durable secret"
        );
        bad_task.abort();
    }

    #[tokio::test]
    async fn m120_public_destination_persistence_failure_rolls_back() {
        let secret = m120_secret(0xD4);
        let tmp = tempfile::tempdir().unwrap();
        let (good_port, good_task) = m120_succeeding_sam().await;
        let manager = m120_manager_with_sam(&tmp, good_port).await;
        m120_write_import(tmp.path(), "persist.key", &secret);
        let mut definition = m120_server_definition(
            "persist-failure",
            crate::i2pcontrol::domain::tunnel::TunnelType::Server,
        );
        definition.options.priv_key_file = Some("persist.key".to_owned());
        manager.create(definition).await.unwrap();

        // Break durable definition persistence after the backend succeeds.
        let tunnels_dir = tmp.path().join("tunnels");
        std::fs::remove_dir_all(&tunnels_dir).unwrap();
        std::fs::write(&tunnels_dir, b"blocker").unwrap();

        let result = manager.start("persist-failure").await.unwrap();
        assert!(result.starts_with("error"), "persist must fail, got: {result}");
        assert!(!result.contains(&secret));
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        // The just-committed secret is removed again; the store returns to empty.
        let store_file = tmp.path().join("server-destinations").join("current.json");
        if store_file.exists() {
            let bytes = std::fs::read(&store_file).unwrap();
            let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(
                envelope["entries"],
                serde_json::json!({}),
                "committed secret must be rolled back"
            );
        }
        // Runtime was stopped again.
        let stored = manager.get("persist-failure").await.unwrap().unwrap();
        assert_eq!(
            stored.runtime_state,
            crate::i2pcontrol::domain::tunnel::TunnelRuntimeState::Stopped
        );
        assert!(m120_identity_of(&stored).is_none());
        good_task.abort();
    }

    #[tokio::test]
    async fn m120_success_commits_once_and_survives_stop_restart_reload() {
        let tmp = tempfile::tempdir().unwrap();
        let (good_port, good_task) = m120_succeeding_sam().await;
        let manager = m120_manager_with_sam(&tmp, good_port).await;
        let definition = m120_server_definition(
            "stable-server",
            crate::i2pcontrol::domain::tunnel::TunnelType::Server,
        );
        manager.create(definition).await.unwrap();
        assert_eq!(manager.start("stable-server").await.unwrap(), "ok");

        let committed = manager.get("stable-server").await.unwrap().unwrap();
        let identity = m120_identity_of(&committed).expect("identity committed");
        let public = committed
            .raw_config
            .get(crate::i2pcontrol::backends::server::SERVER_PUBLIC_DESTINATION_KEY)
            .and_then(|value| value.as_str())
            .expect("public destination committed")
            .to_owned();
        assert_eq!(public, "server-destination");
        let secret = manager.server_destinations.get(&identity).await.unwrap().unwrap();
        assert_eq!(manager.server_destinations.staged_count().await, 0);

        manager.stop("stable-server").await.unwrap();
        let stopped = manager.get("stable-server").await.unwrap().unwrap();
        assert_eq!(m120_identity_of(&stopped).as_deref(), Some(identity.as_str()));

        assert_eq!(manager.start("stable-server").await.unwrap(), "ok");
        let restarted = manager.get("stable-server").await.unwrap().unwrap();
        assert_eq!(m120_identity_of(&restarted).as_deref(), Some(identity.as_str()));
        assert_eq!(
            manager.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            secret.as_str()
        );
        manager.stop("stable-server").await.unwrap();
        drop(manager);

        let reloaded = m120_manager_with_sam(&tmp, good_port).await;
        let stored = reloaded.get("stable-server").await.unwrap().unwrap();
        assert_eq!(m120_identity_of(&stored).as_deref(), Some(identity.as_str()));
        assert_eq!(
            reloaded.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            secret.as_str()
        );
        good_task.abort();
    }

    #[tokio::test]
    async fn m120_concurrent_same_name_starts_commit_once() {
        let secret = m120_secret(0xE5);
        let tmp = tempfile::tempdir().unwrap();
        let (good_port, good_task) = m120_succeeding_sam().await;
        let manager = m120_manager_with_sam(&tmp, good_port).await;
        m120_write_import(tmp.path(), "race.key", &secret);
        let mut definition = m120_server_definition(
            "race-server",
            crate::i2pcontrol::domain::tunnel::TunnelType::Server,
        );
        definition.options.priv_key_file = Some("race.key".to_owned());
        manager.create(definition).await.unwrap();

        let first = manager.clone();
        let second = manager.clone();
        let (left, right) = tokio::join!(first.start("race-server"), second.start("race-server"));
        let outcomes = [left.unwrap(), right.unwrap()];
        assert_eq!(
            outcomes.iter().filter(|result| *result == "ok").count(),
            1,
            "exactly one concurrent start must win, got: {outcomes:?}"
        );
        assert!(outcomes.iter().any(|result| result.starts_with("error")));
        for outcome in &outcomes {
            assert!(!outcome.contains(&secret));
        }
        let stored = manager.get("race-server").await.unwrap().unwrap();
        let identity = m120_identity_of(&stored).expect("winner committed");
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        assert!(
            manager.server_destinations.get(&identity).await.unwrap().is_some(),
            "winner secret must be committed"
        );
        let store_file = tmp.path().join("server-destinations").join("current.json");
        let bytes = std::fs::read(&store_file).unwrap();
        let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(envelope["entries"].as_object().unwrap().len(), 1);
        manager.stop("race-server").await.unwrap();
        good_task.abort();
    }

    #[tokio::test]
    async fn m120_cancellation_after_staging_leaves_no_pending_secret() {
        use std::time::Duration;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        // SAM answers HELLO then hangs before session setup.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let hanging_port = listener.local_addr().unwrap().port();
        let hanging = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                        return;
                    }
                    let _ = write_half.write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n").await;
                    tokio::time::sleep(Duration::from_secs(60)).await;
                });
            }
        });
        let secret = m120_secret(0xF6);
        let tmp = tempfile::tempdir().unwrap();
        let manager = m120_manager_with_sam(&tmp, hanging_port).await;
        m120_write_import(tmp.path(), "cancel.key", &secret);
        let mut definition = m120_server_definition(
            "cancel-server",
            crate::i2pcontrol::domain::tunnel::TunnelType::Server,
        );
        definition.options.priv_key_file = Some("cancel.key".to_owned());
        manager.create(definition).await.unwrap();

        let worker = tokio::spawn(async move { manager.start("cancel-server").await });
        tokio::time::sleep(Duration::from_millis(300)).await;
        worker.abort();
        let _ = worker.await;
        hanging.abort();

        let checker = m120_manager_with_sam(&tmp, hanging_port).await;
        assert_eq!(checker.server_destinations.staged_count().await, 0);
        let stored = checker.get("cancel-server").await.unwrap().unwrap();
        assert!(m120_identity_of(&stored).is_none());
        assert!(
            !tmp.path().join("server-destinations").join("current.json").exists(),
            "cancelled staging must not reach durability"
        );
    }

    // --- M123 commit-phase cancellation atomicity regressions ---

    #[tokio::test]
    async fn m123_abort_before_fresh_secret_commit_terminalizes_and_holds_lifecycle() {
        let secret = m120_secret(0x11);
        let tmp = tempfile::tempdir().unwrap();
        let (sam_port, sam_task) = m120_succeeding_sam().await;
        let manager = m120_manager_with_sam(&tmp, sam_port).await;
        m120_write_import(tmp.path(), "fresh-before-commit.key", &secret);
        let mut definition = m120_server_definition("m123-fresh-before", TunnelType::Server);
        definition.options.priv_key_file = Some("fresh-before-commit.key".to_owned());
        manager.create(definition).await.unwrap();

        let hook = Arc::clone(&manager.commit_hook);
        hook.arm(CommitBoundary::BeforeSecretCommit);
        let worker_manager = manager.clone();
        let worker = tokio::spawn(async move { worker_manager.start("m123-fresh-before").await });
        hook.wait_entered().await;
        assert_eq!(manager.server_destinations.staged_count().await, 1);
        assert!(!tmp.path().join("server-destinations/current.json").exists());

        let competing_manager = manager.clone();
        let mut competing = tokio::spawn(async move {
            competing_manager.stop("m123-fresh-before").await
        });
        assert!(tokio::time::timeout(std::time::Duration::from_millis(50), &mut competing)
            .await
            .is_err());

        worker.abort();
        let _ = worker.await;
        hook.release();
        hook.wait_terminalized().await;
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        assert_eq!(competing.await.unwrap().unwrap(), "ok");

        drop(manager);
        let reloaded = m120_manager_with_sam(&tmp, sam_port).await;
        let stored = reloaded.get("m123-fresh-before").await.unwrap().unwrap();
        let identity = m120_identity_of(&stored).expect("fresh identity committed");
        assert_eq!(stored.options.hosting_destination.as_deref(), Some("server-destination"));
        assert_eq!(
            reloaded.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            secret
        );
        sam_task.abort();
    }

    #[tokio::test]
    async fn m123_abort_after_fresh_secret_commit_finishes_definition_persistence() {
        let secret = m120_secret(0x22);
        let tmp = tempfile::tempdir().unwrap();
        let (sam_port, sam_task) = m120_succeeding_sam().await;
        let manager = m120_manager_with_sam(&tmp, sam_port).await;
        m120_write_import(tmp.path(), "fresh-after-commit.key", &secret);
        let mut definition = m120_server_definition("m123-fresh-after", TunnelType::Server);
        definition.options.priv_key_file = Some("fresh-after-commit.key".to_owned());
        manager.create(definition).await.unwrap();

        let hook = Arc::clone(&manager.commit_hook);
        hook.arm(CommitBoundary::AfterFreshSecretCommit);
        let worker_manager = manager.clone();
        let worker = tokio::spawn(async move { worker_manager.start("m123-fresh-after").await });
        hook.wait_entered().await;
        let bytes = tokio::fs::read(tmp.path().join("server-destinations/current.json"))
            .await
            .unwrap();
        let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(envelope["entries"].as_object().unwrap().len(), 1);
        assert!(
            m120_identity_of(&manager.get("m123-fresh-after").await.unwrap().unwrap()).is_none()
        );

        worker.abort();
        let _ = worker.await;
        hook.release();
        hook.wait_terminalized().await;
        let stored = manager.get("m123-fresh-after").await.unwrap().unwrap();
        let identity = m120_identity_of(&stored).expect("fresh identity committed");
        assert_eq!(stored.options.hosting_destination.as_deref(), Some("server-destination"));
        assert_eq!(manager.server_destinations.staged_count().await, 0);

        drop(manager);
        let reloaded = m120_manager_with_sam(&tmp, sam_port).await;
        let stored = reloaded.get("m123-fresh-after").await.unwrap().unwrap();
        assert_eq!(m120_identity_of(&stored).as_deref(), Some(identity.as_str()));
        assert_eq!(
            reloaded.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            secret
        );
        sam_task.abort();
    }

    #[tokio::test]
    async fn m123_abort_after_replacement_secret_commit_finishes_matching_definition() {
        let old_secret = m120_secret(0x33);
        let new_secret = m120_secret(0x44);
        let tmp = tempfile::tempdir().unwrap();
        let (sam_port, sam_task) = m120_succeeding_sam().await;
        let manager = m120_manager_with_sam(&tmp, sam_port).await;
        m120_write_import(tmp.path(), "m123-old.key", &old_secret);
        m120_write_import(tmp.path(), "m123-new.key", &new_secret);
        let mut definition = m120_server_definition("m123-replacement", TunnelType::Server);
        definition.options.priv_key_file = Some("m123-old.key".to_owned());
        manager.create(definition).await.unwrap();
        assert_eq!(manager.start("m123-replacement").await.unwrap(), "ok");
        manager.stop("m123-replacement").await.unwrap();

        let identity = m120_identity_of(&manager.get("m123-replacement").await.unwrap().unwrap())
            .expect("identity committed");
        let mut replacement = manager.get("m123-replacement").await.unwrap().unwrap();
        replacement.options.priv_key_file = Some("m123-new.key".to_owned());
        manager.update("m123-replacement", replacement, None).await.unwrap();

        let hook = Arc::clone(&manager.commit_hook);
        hook.arm(CommitBoundary::AfterReplacementSecretCommit);
        let worker_manager = manager.clone();
        let worker = tokio::spawn(async move { worker_manager.start("m123-replacement").await });
        hook.wait_entered().await;
        assert_eq!(
            manager.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            new_secret
        );
        let paused = manager.get("m123-replacement").await.unwrap().unwrap();
        assert_eq!(
            paused
                .raw_config
                .get(crate::i2pcontrol::backends::server::SERVER_PUBLIC_DESTINATION_KEY)
                .and_then(|value| value.as_str()),
            Some("server-destination")
        );

        worker.abort();
        let _ = worker.await;
        hook.release();
        hook.wait_terminalized().await;
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        assert_eq!(
            manager.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            new_secret
        );
        manager.stop("m123-replacement").await.unwrap();

        drop(manager);
        let reloaded = m120_manager_with_sam(&tmp, sam_port).await;
        assert_eq!(
            reloaded.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            new_secret
        );
        let stored = reloaded.get("m123-replacement").await.unwrap().unwrap();
        assert_eq!(m120_identity_of(&stored).as_deref(), Some(identity.as_str()));
        sam_task.abort();
    }

    #[tokio::test]
    async fn m123_abort_existing_unchanged_start_finishes_public_persistence() {
        let secret = m120_secret(0x55);
        let tmp = tempfile::tempdir().unwrap();
        let (sam_port, sam_task) = m120_succeeding_sam().await;
        let manager = m120_manager_with_sam(&tmp, sam_port).await;
        m120_write_import(tmp.path(), "m123-existing.key", &secret);
        let mut definition = m120_server_definition("m123-existing", TunnelType::Server);
        definition.options.priv_key_file = Some("m123-existing.key".to_owned());
        manager.create(definition).await.unwrap();
        assert_eq!(manager.start("m123-existing").await.unwrap(), "ok");
        manager.stop("m123-existing").await.unwrap();

        let mut unchanged = manager.get("m123-existing").await.unwrap().unwrap();
        unchanged.options.priv_key_file = None;
        manager.update("m123-existing", unchanged, None).await.unwrap();
        let identity = m120_identity_of(&manager.get("m123-existing").await.unwrap().unwrap())
            .expect("identity committed");
        let hook = Arc::clone(&manager.commit_hook);
        hook.arm(CommitBoundary::BeforeExistingDefinitionPersist);
        let worker_manager = manager.clone();
        let worker = tokio::spawn(async move { worker_manager.start("m123-existing").await });
        hook.wait_entered().await;
        worker.abort();
        let _ = worker.await;
        hook.release();
        hook.wait_terminalized().await;

        let stored = manager.get("m123-existing").await.unwrap().unwrap();
        assert_eq!(m120_identity_of(&stored).as_deref(), Some(identity.as_str()));
        assert_eq!(stored.options.hosting_destination.as_deref(), Some("server-destination"));
        assert_eq!(
            manager.server_destinations.get(&identity).await.unwrap().unwrap().as_str(),
            secret
        );
        manager.stop("m123-existing").await.unwrap();
        sam_task.abort();
    }

    #[tokio::test]
    async fn m123_abort_restart_start_phase_terminalizes_before_competing_stop() {
        let secret = m120_secret(0x66);
        let tmp = tempfile::tempdir().unwrap();
        let (sam_port, sam_task) = m120_succeeding_sam().await;
        let manager = m120_manager_with_sam(&tmp, sam_port).await;
        m120_write_import(tmp.path(), "m123-restart.key", &secret);
        let mut definition = m120_server_definition("m123-restart", TunnelType::Server);
        definition.options.priv_key_file = Some("m123-restart.key".to_owned());
        manager.create(definition).await.unwrap();
        assert_eq!(manager.start("m123-restart").await.unwrap(), "ok");
        manager.stop("m123-restart").await.unwrap();

        let hook = Arc::clone(&manager.commit_hook);
        hook.arm(CommitBoundary::BeforeSecretCommit);
        let worker_manager = manager.clone();
        let worker = tokio::spawn(async move { worker_manager.restart("m123-restart").await });
        hook.wait_entered().await;
        let competing_manager = manager.clone();
        let mut competing = tokio::spawn(async move {
            competing_manager.stop("m123-restart").await
        });
        assert!(tokio::time::timeout(std::time::Duration::from_millis(50), &mut competing)
            .await
            .is_err());
        worker.abort();
        let _ = worker.await;
        hook.release();
        hook.wait_terminalized().await;
        assert_eq!(competing.await.unwrap().unwrap(), "ok");
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        sam_task.abort();
    }

    #[tokio::test]
    async fn m120_startup_managed_server_path_is_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let startup = StartupTunnelInventory::from_configs(
            &[],
            &[crate::i2pcontrol::production::StartupServerConfig {
                name: "startup-srv".to_owned(),
                port: 8080,
            }],
        )
        .unwrap();
        let manager = ProductionTunnelManagerControl::new_with_startup_inventory(
            tmp.path().join("tunnels"),
            startup,
        )
        .unwrap();
        manager.load().await.unwrap();
        let result = manager.start("startup-srv").await;
        assert!(result.is_err(), "startup path stays externally managed");
        assert_eq!(manager.server_destinations.staged_count().await, 0);
        assert!(manager.startup.get("startup-srv").unwrap().is_some());
    }
}
