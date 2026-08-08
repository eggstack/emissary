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

use crate::{
    address_book::{
        RuntimeAddressBookEntry, RuntimeAddressBookHandle, RuntimeAddressBookSnapshot,
        RuntimeAddressBookType,
    },
    i2pcontrol::{
        backends::{registry::TunnelBackendRegistry, BackendError, TunnelBackend},
        control_plane::{AddressBookControl, ControlPlane, TunnelManagerControl},
        domain::{
            address_book::{
                AddressBookConfiguration, AddressBookEntry, AdministrativeAddressBookType,
                SubscriptionSet,
            },
            tunnel::{TunnelDefinition, TunnelName, TunnelType},
        },
        observability::LogRing,
        router_info::{
            ActivePeerStats, BannedPeer, ClockSkew, I2PTunnelStats, InspectionError,
            InspectionGroup, LogEntry, LogSnapshot, NetworkSnapshot, NetworkStatus,
            PeerDirectorySnapshot, PeerDirectorySource, PeerIdentity, PeerLimits,
            RecentTransitTraffic, RouterInfoControl, TransitBytes, TransportBytes,
            TunnelBuildStats, TunnelSummary,
        },
        server_secret_store::{ServerDestinationStore, StoredDestination},
        stores::{address_book_store::AddressBookStore, tunnel_store::TunnelStore},
    },
};

use emissary_core::{
    crypto::base64_encode, events::EventHandle, inspection::CoreSnapshot, runtime::Runtime,
    FirewallStatus,
};

/// Maximum number of startup and control-plane tunnel definitions exposed by
/// one logical inventory.
pub const MAX_TUNNEL_INVENTORY: usize = 1000;

/// Public peer directory backed by the existing bounded core inspection
/// snapshot. The clone contains only owned public RouterInfo bytes; it is not a
/// router or NetDB control handle.
pub struct CoreSnapshotPeerDirectory {
    snapshot: CoreSnapshot,
    max_items: usize,
}

impl CoreSnapshotPeerDirectory {
    /// Create a bounded public peer directory source.
    pub fn new(snapshot: CoreSnapshot, max_items: usize) -> Self {
        Self {
            snapshot,
            max_items,
        }
    }
}

impl PeerDirectorySource for CoreSnapshotPeerDirectory {
    fn snapshot(&self) -> Result<PeerDirectorySnapshot, InspectionError> {
        if self.snapshot.netdb.known_router_ids.len() > self.max_items {
            return Err(InspectionError::ResultTooLarge {
                group: InspectionGroup::PeerList,
                limit: self.max_items,
            });
        }

        let mut peer_ids = self.snapshot.netdb.known_router_ids.clone();
        peer_ids.sort_unstable();

        Ok(PeerDirectorySnapshot {
            peer_ids,
            router_infos: self.snapshot.netdb.peer_router_infos.clone(),
        })
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
        })
    }

    /// Return startup definitions in deterministic name order.
    pub fn list(&self) -> Result<Vec<TunnelDefinition>, String> {
        let definitions = self
            .definitions
            .read()
            .map_err(|_| "startup tunnel inventory lock poisoned".to_string())?;
        Ok(definitions.values().cloned().collect())
    }

    /// Return one startup definition by its exact configured name.
    pub fn get(&self, name: &str) -> Result<Option<TunnelDefinition>, String> {
        let definitions = self
            .definitions
            .read()
            .map_err(|_| "startup tunnel inventory lock poisoned".to_string())?;
        Ok(definitions.get(name).cloned())
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
        if definition.tunnel_type != TunnelType::Server {
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
    /// Number of currently connected routers.
    fn connected_routers(&self) -> usize;
    /// Number of active transit tunnels.
    fn transit_tunnel_count(&self) -> usize;
    /// Cumulative tunnel build successes.
    fn tunnel_build_successes(&self) -> u64;
    /// Cumulative tunnel build failures.
    fn tunnel_build_failures(&self) -> u64;
    /// Latest IPv4 firewall status.
    fn ipv4_firewall_status(&self) -> FirewallStatus;
    /// Latest IPv6 firewall status.
    fn ipv6_firewall_status(&self) -> FirewallStatus;
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
    fn ipv4_firewall_status(&self) -> FirewallStatus {
        self.handle.ipv4_firewall_status()
    }
    fn ipv6_firewall_status(&self) -> FirewallStatus {
        self.handle.ipv6_firewall_status()
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
    sam_tcp_port: Option<u16>,
    lifecycle: Arc<tokio::sync::Mutex<BTreeMap<String, Arc<tokio::sync::Mutex<()>>>>>,
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
        let state_root = dir.parent().unwrap_or(dir.as_path()).to_path_buf();
        let server_destinations = ServerDestinationStore::new(state_root);
        let registry = match sam_tcp_port {
            Some(port) => {
                crate::i2pcontrol::backends::registry::create_production_registry_with_server_store(
                    port,
                    server_destinations.clone(),
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
            sam_tcp_port,
            lifecycle: Arc::new(tokio::sync::Mutex::new(BTreeMap::new())),
        })
    }

    /// Load existing state from disk.
    pub async fn load(&self) -> Result<(), String> {
        self.server_destinations.load().await?;
        let mut store = self.inner.lock().await;
        store.load().await.map_err(|e| format!("store load: {e}"))?;
        let referenced_server_identities = store
            .list()
            .iter()
            .filter(|definition| definition.tunnel_type == TunnelType::Server)
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
        drop(store);
        self.server_destinations
            .prune_unreferenced(&referenced_server_identities)
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
                    TunnelType::Client | TunnelType::Server
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

    async fn start_locked(&self, name: &str) -> Result<String, String> {
        if self.startup.get(name)?.is_some() {
            return Err("startup-managed tunnel lifecycle is externally managed".into());
        }
        let definition = {
            let store = self.inner.lock().await;
            store
                .get(name)
                .ok_or_else(|| format!("error - tunnel '{}' not found", name))?
                .clone()
        };
        let definition = self.prepare_server_definition(definition).await?;
        let backend = self.registry.get(definition.tunnel_type);
        match backend.start(&definition).await {
            Ok(()) => {
                if definition.tunnel_type == TunnelType::Server {
                    let status = backend.inspect(&definition);
                    let Some(destination) = status.destination else {
                        let _ = backend.stop(&definition).await;
                        return Ok("error - server runtime did not publish a destination".into());
                    };
                    if let Err(error) = self
                        .persist_server_public_destination(definition.clone(), destination)
                        .await
                    {
                        let _ = backend.stop(&definition).await;
                        return Ok(format!("error - {error}"));
                    }
                }
                Ok("ok".to_string())
            }
            Err(BackendError::NotImplemented { tunnel_type }) => {
                Ok(format!("error - {} not implemented", tunnel_type.as_str()))
            }
            Err(error) => Ok(format!("error - {error}")),
        }
    }

    async fn with_runtime_state(&self, mut definition: TunnelDefinition) -> TunnelDefinition {
        if definition.ownership == crate::i2pcontrol::domain::tunnel::TunnelOwnership::ControlPlane
        {
            let status = self.registry.get(definition.tunnel_type).inspect(&definition);
            definition.runtime_state = status.runtime_state;
            if definition.tunnel_type == TunnelType::Server {
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
                        {
                            Some(public.to_string())
                        }
                        _ => None,
                    }
                };
                definition.options.hosting_destination = public_destination;
            }
        }
        definition
    }

    async fn prepare_server_definition(
        &self,
        mut definition: TunnelDefinition,
    ) -> Result<TunnelDefinition, String> {
        if definition.tunnel_type != TunnelType::Server {
            return Ok(definition);
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
            return Ok(definition);
        }

        let sam_tcp_port = self
            .sam_tcp_port
            .ok_or_else(|| "server backend requires the router SAM listener".to_string())?;
        let private = crate::tunnel_server::generate_persistent_destination(sam_tcp_port)
            .await
            .map_err(|_| "server destination generation failed".to_string())?;
        let identity = ServerDestinationStore::new_identity();
        self.server_destinations
            .put(&identity, StoredDestination::from_private(private))
            .await?;
        definition.raw_config.insert(
            crate::i2pcontrol::backends::server::SERVER_IDENTITY_KEY.to_string(),
            serde_json::json!(identity),
        );
        let result = {
            let mut store = self.inner.lock().await;
            store
                .upsert(definition.clone())
                .await
                .map_err(|error| format!("store upsert: {error}"))
        };
        if let Err(error) = result {
            let _ = self.server_destinations.remove(&identity).await;
            return Err(error);
        }
        Ok(definition)
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
            sam_tcp_port: self.sam_tcp_port,
            lifecycle: Arc::clone(&self.lifecycle),
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
        let mut store = self.inner.lock().await;
        store
            .update(name, definition, new_name.as_ref().map(TunnelName::as_str))
            .await
            .map_err(|e| format!("store update: {e}"))
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
        Ok(true)
    }

    async fn start(&self, name: &str) -> Result<String, String> {
        let _lifecycle = self.lifecycle_lock(name).await;
        self.start_locked(name).await
    }

    async fn stop(&self, name: &str) -> Result<String, String> {
        let _lifecycle = self.lifecycle_lock(name).await;
        if self.startup.get(name)?.is_some() {
            return Err("startup-managed tunnel lifecycle is externally managed".into());
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
        let _lifecycle = self.lifecycle_lock(name).await;
        if self.startup.get(name)?.is_some() {
            return Err("startup-managed tunnel lifecycle is externally managed".into());
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
        self.start_locked(name).await
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
    log_ring: Arc<LogRing>,
    tunnel_manager: Arc<dyn TunnelManagerControl>,
    peer_directory: Option<Arc<dyn PeerDirectorySource>>,
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
        Self {
            router_id_b64,
            version,
            startup: std::time::Instant::now(),
            share_ratio,
            configured_bandwidth_in,
            configured_bandwidth_out,
            metrics,
            log_ring,
            tunnel_manager,
            peer_directory: None,
        }
    }

    /// Attach the canonical, read-only public peer directory source.
    pub fn with_peer_directory_source(mut self, source: Arc<dyn PeerDirectorySource>) -> Self {
        self.peer_directory = Some(source);
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
        let ipv4 = Self::firewall_status_to_network(self.metrics.ipv4_firewall_status());
        let ipv6 = Self::firewall_status_to_network(self.metrics.ipv6_firewall_status());
        let firewalled = ipv4 == NetworkStatus::Firewalled || ipv6 == NetworkStatus::Firewalled;
        let hidden = ipv4 == NetworkStatus::Hidden || ipv6 == NetworkStatus::Hidden;
        Ok(NetworkSnapshot {
            ipv4_status: ipv4,
            ipv6_status: ipv6,
            error: None,
            testing: false,
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
        // Active peer IDs require a bounded current snapshot from the canonical
        // core transport owner. No existing event-metric handle exposes this
        // list. Return unavailable rather than a stale snapshot.
        Err(InspectionError::Unavailable {
            group: InspectionGroup::PeerList,
        })
    }

    async fn peer_router_info(&self, _peer_id: &str) -> Result<Option<String>, InspectionError> {
        let source = self.peer_directory.as_ref().ok_or(InspectionError::Unavailable {
            group: InspectionGroup::PeerLookup,
        })?;
        let snapshot = source.snapshot()?;
        let Some(bytes) = snapshot.router_infos.get(_peer_id) else {
            return Ok(None);
        };
        Ok(Some(base64_encode(bytes.clone())))
    }

    async fn banned_peers(&self) -> Result<Vec<BannedPeer>, InspectionError> {
        Err(InspectionError::Unavailable {
            group: InspectionGroup::PeerStats,
        })
    }

    async fn peer_limits(&self) -> Result<PeerLimits, InspectionError> {
        Err(InspectionError::Unavailable {
            group: InspectionGroup::PeerStats,
        })
    }

    async fn active_peer_stats(&self) -> Result<Vec<ActivePeerStats>, InspectionError> {
        Err(InspectionError::Unavailable {
            group: InspectionGroup::PeerStats,
        })
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
        Err(InspectionError::UnavailableReason {
            group: InspectionGroup::Retained,
            reason: "no router news owner",
        })
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
        address_book::{AddressBookManager, RuntimeAddressBookType},
        config::AddressBookConfig,
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

        let manager = AddressBookManager::new_with_control_owner(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
        let adapter = ProductionAddressBookControl::new(control.clone(), legacy_dir.clone());
        adapter.load().await.unwrap();
        assert_eq!(
            control.runtime_list(RuntimeAddressBookType::Private).await.unwrap()[0].hostname,
            "legacy.i2p"
        );
        assert!(control.runtime_authority_present());

        drop(manager);
        let manager = AddressBookManager::new_with_control_owner(
            base,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
        ProductionAddressBookControl::new(control.clone(), legacy_dir)
            .load()
            .await
            .unwrap();
        assert_eq!(
            control.runtime_list(RuntimeAddressBookType::Private).await.unwrap().len(),
            1
        );
    }

    #[tokio::test]
    async fn production_setters_do_not_report_inert_success() {
        use crate::i2pcontrol::control_plane::AddressBookControl;

        let base = tempfile::tempdir().unwrap().keep();
        let manager = AddressBookManager::new_with_control_owner(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
        let adapter = ProductionAddressBookControl::new(control.clone(), base.join("addressbooks"));

        let mut config = AddressBookConfiguration::new();
        config.insert(
            "private_addressbook".to_string(),
            "chosen-by-request".to_string(),
        );
        assert!(adapter.set_configuration(config).await.is_err());
        assert!(control.runtime_configuration().await.unwrap().is_empty());

        let subscriptions =
            SubscriptionSet::from_vec(vec!["https://example.i2p/hosts.txt".to_string()]);
        assert!(adapter.set_subscriptions(subscriptions).await.is_err());
        assert!(control.runtime_subscriptions().await.unwrap().is_empty());
        assert!(!control.runtime_authority_present());
    }

    #[tokio::test]
    async fn legacy_configuration_is_not_promoted_into_runtime_owner() {
        let base = tempfile::tempdir().unwrap().keep();
        let legacy_dir = base.join("addressbooks");
        let mut legacy = AddressBookStore::new(legacy_dir.clone(), 1024 * 1024);
        let mut configuration = AddressBookConfiguration::new();
        configuration.insert("theme".to_string(), "light".to_string());
        legacy.set_configuration(configuration).await.unwrap();

        let manager = AddressBookManager::new_with_control_owner(
            base,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
        ProductionAddressBookControl::new(control.clone(), legacy_dir)
            .load()
            .await
            .unwrap();
        assert!(control.runtime_configuration().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn legacy_address_book_migration_rejects_hostname_collisions() {
        let base = tempfile::tempdir().unwrap().keep();
        let legacy_dir = base.join("addressbooks");
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
            valid_destination(3),
        )
        .await
        .unwrap();

        let mut legacy = AddressBookStore::new(legacy_dir.clone(), 1024 * 1024);
        legacy
            .add(
                AdministrativeAddressBookType::Private,
                AddressBookEntry::new("collision.i2p", valid_destination(2)),
            )
            .await
            .unwrap();

        let manager = AddressBookManager::new_with_control_owner(
            base,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
        let error = ProductionAddressBookControl::new(control.clone(), legacy_dir)
            .load()
            .await
            .unwrap_err();
        assert!(error.contains("collision"));
        assert!(control.runtime_list(RuntimeAddressBookType::Private).await.unwrap().is_empty());
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

        let manager = AddressBookManager::new_with_control_owner(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
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

        let manager = AddressBookManager::new_with_control_owner(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
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

        let manager = AddressBookManager::new_with_control_owner(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
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

        let manager = AddressBookManager::new_with_control_owner(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
        let adapter = ProductionAddressBookControl::new(control.clone(), base.join("addressbooks"));
        adapter.load().await.unwrap();
        assert!(control
            .runtime_delete(RuntimeAddressBookType::Published, "deleted.i2p")
            .await
            .unwrap());
        drop(manager);

        let manager = AddressBookManager::new_with_control_owner(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let control = manager.control_handle().unwrap();
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
}
