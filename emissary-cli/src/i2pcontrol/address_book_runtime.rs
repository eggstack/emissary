//! Runtime AddressBook adapter owned by I2PControl.
//!
//! This module owns Proposal 170 administrative persistence, validation, migration, and
//! publication policy. The parent AddressBook module retains only the legacy downloader and its
//! narrow runtime publication seam.
#![cfg(feature = "i2pcontrol")]

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::i2pcontrol::stores::publication::{publish_with_backup, publish_with_backup_sync};
use crate::address_book::{
    AddressBookManager, AddressBookRuntimeContext, AddressBookRuntimeHook,
    RuntimeSubscriptionCommand, RuntimeSubscriptionControl,
};
use crate::config::AddressBookConfig;
use emissary_core::{
    crypto::{base32_encode, base64_decode},
    primitives::Destination,
};
use parking_lot::RwLock;

pub(crate) const MAX_LEGACY_DESTINATION_ENTRIES: usize = 10_000;
pub(crate) const MAX_LEGACY_DESTINATION_FILE_BYTES: usize = 64 * 1024;
pub(crate) const MAX_LEGACY_DESTINATION_BYTES: usize = 1024 * 1024;

pub(crate) fn base32_for_destination(destination: &str) -> String {
    base64_decode(destination)
        .and_then(|decoded| Destination::parse(&decoded).ok())
        .map(|destination| base32_encode(destination.id().to_vec()))
        .unwrap_or_else(|| destination.to_string())
}

pub(crate) fn is_valid_full_destination(destination: &str) -> bool {
    let Some(decoded) = base64_decode(destination) else {
        return false;
    };
    Destination::parse(&decoded).is_ok()
}

fn is_base32_seed(destination: &str) -> bool {
    destination.len() == 52
        && destination
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || matches!(byte, b'2'..=b'7'))
}

fn repair_published_entries(
    state: &mut RuntimeAddressBookSnapshot,
    destinations: &BTreeMap<String, RuntimeAddressBookEntry>,
) -> Result<bool, String> {
    let mut repaired = false;
    for (hostname, entry) in &mut state.published {
        if is_valid_full_destination(&entry.destination) {
            continue;
        }
        let Some(legacy) = destinations.get(hostname) else {
            return Err("address book state contains an unrepairable destination".to_string());
        };
        if !is_base32_seed(&entry.destination)
            || base32_for_destination(&legacy.destination) != entry.destination
        {
            return Err("address book state contains an unrepairable destination".to_string());
        }
        entry.destination = legacy.destination.clone();
        repaired = true;
    }
    Ok(repaired)
}

pub(crate) fn validate_runtime_entry(
    key: &str,
    entry: &RuntimeAddressBookEntry,
) -> Result<(), String> {
    if key != entry.hostname || entry.hostname.is_empty() || entry.hostname.len() > 254 {
        return Err("address book state contains an invalid hostname".to_string());
    }
    if entry.hostname.contains('/')
        || entry.hostname.contains('\\')
        || entry.hostname.chars().any(|character| character.is_control())
    {
        return Err("address book state contains an invalid hostname".to_string());
    }
    if !is_valid_full_destination(&entry.destination) {
        return Err("address book state contains an invalid destination".to_string());
    }
    Ok(())
}

pub(crate) fn validate_runtime_snapshot(state: &RuntimeAddressBookSnapshot) -> Result<(), String> {
    let books = [
        &state.private,
        &state.local,
        &state.router,
        &state.published,
    ];
    let total_entries = books.iter().map(|book| book.len()).sum::<usize>();
    if total_entries > MAX_LEGACY_DESTINATION_ENTRIES {
        return Err("address book state exceeds its entry limit".to_string());
    }
    let mut hostnames = std::collections::BTreeSet::new();
    for book in books {
        for (hostname, entry) in book {
            validate_runtime_entry(hostname, entry)?;
            if !hostnames.insert(hostname) {
                return Err("address book state contains a hostname collision".to_string());
            }
        }
    }
    Ok(())
}

/// Administrative address-book source selected by Proposal 170.
#[cfg(feature = "i2pcontrol")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAddressBookType {
    /// Private entries.
    Private,
    /// Local entries.
    Local,
    /// Router entries.
    Router,
    /// Published entries.
    Published,
}

/// A bounded entry owned by the runtime address-book authority.
#[cfg(feature = "i2pcontrol")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeAddressBookEntry {
    /// Hostname used for lookup.
    pub hostname: String,
    /// Full, structurally validated Base64 destination.
    pub destination: String,
}

/// Complete runtime address-book state exchanged with the I2PControl adapter.
#[cfg(feature = "i2pcontrol")]
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeAddressBookSnapshot {
    /// Private entries.
    pub private: BTreeMap<String, RuntimeAddressBookEntry>,
    /// Local entries.
    pub local: BTreeMap<String, RuntimeAddressBookEntry>,
    /// Router entries.
    pub router: BTreeMap<String, RuntimeAddressBookEntry>,
    /// Published entries, including entries loaded by the existing downloader.
    pub published: BTreeMap<String, RuntimeAddressBookEntry>,
    /// Stored subscription metadata.
    pub subscriptions: Vec<String>,
    /// Stored non-operative configuration metadata.
    pub configuration: BTreeMap<String, String>,
}

#[cfg(feature = "i2pcontrol")]
impl RuntimeAddressBookSnapshot {
    pub(crate) fn book(
        &self,
        book_type: RuntimeAddressBookType,
    ) -> &BTreeMap<String, RuntimeAddressBookEntry> {
        match book_type {
            RuntimeAddressBookType::Private => &self.private,
            RuntimeAddressBookType::Local => &self.local,
            RuntimeAddressBookType::Router => &self.router,
            RuntimeAddressBookType::Published => &self.published,
        }
    }

    pub(crate) fn book_mut(
        &mut self,
        book_type: RuntimeAddressBookType,
    ) -> &mut BTreeMap<String, RuntimeAddressBookEntry> {
        match book_type {
            RuntimeAddressBookType::Private => &mut self.private,
            RuntimeAddressBookType::Local => &mut self.local,
            RuntimeAddressBookType::Router => &mut self.router,
            RuntimeAddressBookType::Published => &mut self.published,
        }
    }
}

/// Runtime address-book owner shared by the router and I2PControl.
#[cfg(feature = "i2pcontrol")]
pub(crate) struct RuntimeAddressBookOwner {
    pub(crate) path: PathBuf,
    addresses: Arc<RwLock<HashMap<String, String>>>,
    serialized: Arc<RwLock<String>>,
    pub(crate) state: RwLock<RuntimeAddressBookSnapshot>,
    mutation: tokio::sync::Mutex<()>,
    authority_present: AtomicBool,
    initialization_error: Option<String>,
    subscription_control: Arc<RuntimeSubscriptionControl>,
}

#[cfg(feature = "i2pcontrol")]
impl RuntimeAddressBookOwner {
    pub(crate) async fn new(
        context: AddressBookRuntimeContext,
        initial_subscriptions: Vec<String>,
    ) -> Arc<Self> {
        let path = context.path;
        let addresses = context.addresses;
        let serialized = context.serialized;
        let state_path = path.join("control-state.json");
        let backup_path = path.join("control-state.json.bak");
        let current_exists = state_path.exists();
        let backup_exists = backup_path.exists();
        let mut initialization_error = None;

        let loaded = match tokio::fs::read_to_string(&state_path).await {
            Ok(raw) => match serde_json::from_str::<RuntimeAddressBookSnapshot>(&raw) {
                Ok(state) => Some(state),
                Err(_) => match tokio::fs::read_to_string(&backup_path).await {
                    Ok(raw) => serde_json::from_str(&raw).ok(),
                    Err(_) => None,
                },
            },
            Err(_) => match tokio::fs::read_to_string(&backup_path).await {
                Ok(raw) => serde_json::from_str(&raw).ok(),
                Err(_) => None,
            },
        };

        let authority_present = current_exists || backup_exists;
        let initial_subscriptions_for_fallback = initial_subscriptions.clone();
        let state = loaded.unwrap_or_else(|| {
            if authority_present {
                initialization_error = Some("address book state is corrupt".to_string());
            }
            RuntimeAddressBookSnapshot {
                subscriptions: initial_subscriptions_for_fallback,
                ..RuntimeAddressBookSnapshot::default()
            }
        });
        let active_subscriptions = state.subscriptions.clone();
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        let owner = Arc::new(Self {
            path,
            addresses,
            serialized,
            state: RwLock::new(state),
            mutation: tokio::sync::Mutex::new(()),
            authority_present: AtomicBool::new(authority_present),
            initialization_error,
            subscription_control: Arc::new(RuntimeSubscriptionControl {
                sender,
                active: RwLock::new(active_subscriptions),
                started: AtomicBool::new(false),
                receiver: std::sync::Mutex::new(Some(receiver)),
            }),
        });
        owner.rebuild_runtime_indexes();
        owner
    }

    pub(crate) fn rebuild_runtime_indexes(&self) {
        let state = self.state.read();
        let mut effective = BTreeMap::new();
        for book in [
            &state.private,
            &state.local,
            &state.router,
            &state.published,
        ] {
            for (hostname, entry) in book {
                effective
                    .entry(hostname.clone())
                    .or_insert_with(|| base32_for_destination(&entry.destination));
            }
        }
        let mut addresses = self.addresses.write();
        addresses.clear();
        addresses.extend(effective);
        let mut serialized = self.serialized.write();
        *serialized = addresses
            .iter()
            .map(|(hostname, address)| format!("{hostname}={address}"))
            .collect::<Vec<_>>()
            .join("\n");
    }

    pub(crate) fn initialization_error(&self) -> Option<String> {
        self.initialization_error.clone()
    }

    pub(crate) fn authority_present(&self) -> bool {
        self.authority_present.load(Ordering::Acquire)
    }

    pub(crate) fn snapshot(&self) -> RuntimeAddressBookSnapshot {
        self.state.read().clone()
    }

    pub(crate) fn entries(
        &self,
        book_type: RuntimeAddressBookType,
    ) -> Vec<RuntimeAddressBookEntry> {
        self.state.read().book(book_type).values().cloned().collect()
    }

    pub(crate) fn resolve_base32(&self, hostname: &str) -> Option<String> {
        let state = self.state.read();
        for book in [
            &state.private,
            &state.local,
            &state.router,
            &state.published,
        ] {
            if let Some(entry) = book.get(hostname) {
                return Some(base32_for_destination(&entry.destination));
            }
        }
        None
    }

    pub(crate) fn resolve_base64(&self, hostname: &str) -> Option<String> {
        let state = self.state.read();
        for book in [
            &state.private,
            &state.local,
            &state.router,
            &state.published,
        ] {
            if let Some(entry) = book.get(hostname) {
                return Some(entry.destination.clone());
            }
        }
        None
    }

    async fn persist(&self, state: &RuntimeAddressBookSnapshot) -> Result<(), String> {
        let raw = serde_json::to_vec(state)
            .map_err(|_| "address book serialization failed".to_string())?;
        if raw.len() > 1024 * 1024 {
            return Err("address book state exceeds its size limit".to_string());
        }
        publish_with_backup(
            &self.path,
            "control-state.json",
            "control-state.json.bak",
            ".control-state.json.tmp",
            &raw,
            1024 * 1024,
        )
        .await
        .map_err(|_| "address book persistence failed".to_string())
    }

    fn persist_sync(&self, state: &RuntimeAddressBookSnapshot) {
        let Ok(raw) = serde_json::to_vec(state) else {
            return;
        };
        let _ = publish_with_backup_sync(
            &self.path,
            "control-state.json",
            "control-state.json.bak",
            ".control-state.json.tmp",
            &raw,
            1024 * 1024,
        );
    }

    pub(crate) fn legacy_publish_sync(&self, entry: RuntimeAddressBookEntry) {
        let mut state = self.state.write();
        state.published.insert(entry.hostname.clone(), entry);
        let committed = state.clone();
        drop(state);
        self.persist_sync(&committed);
        self.authority_present.store(true, Ordering::Release);
        self.rebuild_runtime_indexes();
    }

    pub(crate) fn legacy_remove_sync(&self, hostname: &str) {
        let mut state = self.state.write();
        state.published.remove(hostname);
        let committed = state.clone();
        drop(state);
        self.persist_sync(&committed);
        self.authority_present.store(true, Ordering::Release);
        self.rebuild_runtime_indexes();
    }

    async fn commit(&self, state: RuntimeAddressBookSnapshot) -> Result<(), String> {
        let _guard = self.mutation.lock().await;
        self.persist(&state).await?;
        *self.state.write() = state;
        self.authority_present.store(true, Ordering::Release);
        self.rebuild_runtime_indexes();
        Ok(())
    }

    pub(crate) async fn mutate<T, F>(&self, update: F) -> Result<T, String>
    where
        F: FnOnce(&mut RuntimeAddressBookSnapshot) -> Result<T, String>,
    {
        let _guard = self.mutation.lock().await;
        let mut state = self.snapshot();
        let result = update(&mut state)?;
        self.persist(&state).await?;
        *self.state.write() = state;
        self.authority_present.store(true, Ordering::Release);
        self.rebuild_runtime_indexes();
        Ok(result)
    }

    pub(crate) async fn import_legacy(
        &self,
        legacy: RuntimeAddressBookSnapshot,
        destinations: BTreeMap<String, RuntimeAddressBookEntry>,
    ) -> Result<(), String> {
        if self.authority_present() {
            return Ok(());
        }
        let current = self.snapshot();
        let mut merged = current;
        merged.private = legacy.private;
        merged.local = legacy.local;
        merged.router = legacy.router;
        merged.published.extend(legacy.published);
        if !legacy.subscriptions.is_empty() {
            merged.subscriptions = legacy.subscriptions;
        }
        // Proposal 170 configuration has no live Emissary owner. Do not
        // promote historical inert metadata into the runtime authority.
        merged.configuration.clear();
        for (hostname, entry) in &destinations {
            merged.published.entry(hostname.clone()).or_insert_with(|| entry.clone());
        }
        repair_published_entries(&mut merged, &destinations)?;
        validate_runtime_snapshot(&merged)?;
        self.commit(merged).await
    }

    pub(crate) async fn repair_published(
        &self,
        destinations: BTreeMap<String, RuntimeAddressBookEntry>,
    ) -> Result<(), String> {
        let mut state = self.snapshot();
        let repaired = repair_published_entries(&mut state, &destinations)?;
        validate_runtime_snapshot(&state)?;
        if repaired {
            self.commit(state).await
        } else {
            Ok(())
        }
    }

    async fn merge_downloaded_impl(&self, addresses: HashMap<String, (String, String)>) {
        let _guard = self.mutation.lock().await;
        let mut state = self.snapshot();
        for (hostname, (base32, destination)) in addresses {
            match state.published.get_mut(&hostname) {
                None => {
                    state.published.insert(
                        hostname.clone(),
                        RuntimeAddressBookEntry {
                            hostname,
                            destination,
                        },
                    );
                }
                Some(existing)
                    if !is_valid_full_destination(&existing.destination)
                        && existing.destination == base32 =>
                {
                    existing.destination = destination;
                }
                Some(_) => {}
            }
        }
        if validate_runtime_snapshot(&state).is_ok() && self.persist(&state).await.is_ok() {
            *self.state.write() = state;
            self.authority_present.store(true, Ordering::Release);
            self.rebuild_runtime_indexes();
        }
    }

    pub(crate) fn subscription_control(&self) -> Arc<RuntimeSubscriptionControl> {
        Arc::clone(&self.subscription_control)
    }
}

const MAX_RUNTIME_SUBSCRIPTIONS: usize = 1000;
const MAX_RUNTIME_SUBSCRIPTION_LENGTH: usize = 2048;
const MAX_RUNTIME_SUBSCRIPTION_BYTES: usize = 4 * 1024 * 1024;

fn validate_runtime_subscriptions(subscriptions: &[String]) -> Result<(), String> {
    if subscriptions.len() > MAX_RUNTIME_SUBSCRIPTIONS {
        return Err(format!(
            "too many subscriptions; maximum is {MAX_RUNTIME_SUBSCRIPTIONS}"
        ));
    }
    let mut total_bytes = 0usize;
    for subscription in subscriptions {
        if subscription.len() > MAX_RUNTIME_SUBSCRIPTION_LENGTH {
            return Err("subscription exceeds its length limit".to_string());
        }
        if subscription.chars().any(|character| character.is_control()) {
            return Err("subscription contains control characters".to_string());
        }
        let parsed = url::Url::parse(subscription)
            .map_err(|_| "subscription is not a valid URL".to_string())?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err("subscription must be an HTTP or HTTPS URL with a host".to_string());
        }
        total_bytes = total_bytes
            .checked_add(subscription.len())
            .ok_or_else(|| "subscription set exceeds its size limit".to_string())?;
    }
    if total_bytes > MAX_RUNTIME_SUBSCRIPTION_BYTES {
        return Err("subscription set exceeds its size limit".to_string());
    }
    Ok(())
}

impl AddressBookRuntimeHook for Arc<RuntimeAddressBookOwner> {
    fn initial_subscriptions(&self, configured: &[String]) -> Vec<String> {
        let stored = self.state.read().subscriptions.clone();
        if self.authority_present() {
            stored
        } else {
            configured.to_vec()
        }
    }

    fn subscription_control(&self) -> Arc<RuntimeSubscriptionControl> {
        RuntimeAddressBookOwner::subscription_control(self.as_ref())
    }

    fn commit_subscriptions(
        &self,
        subscriptions: Vec<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let owner = Arc::clone(self);
        Box::pin(async move {
            validate_runtime_subscriptions(&subscriptions)?;
            owner
                .mutate(|state| {
                    state.subscriptions = subscriptions.clone();
                    Ok(())
                })
                .await?;
            *owner.subscription_control.active.write() = subscriptions;
            Ok(())
        })
    }

    fn merge_downloaded(
        &self,
        addresses: HashMap<String, (String, String)>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        let owner = Arc::clone(self);
        Box::pin(async move { owner.merge_downloaded_impl(addresses).await })
    }

    fn resolve_base32(&self, hostname: &str) -> Option<String> {
        RuntimeAddressBookOwner::resolve_base32(self, hostname)
    }

    fn resolve_base64(&self, hostname: &str) -> Option<String> {
        RuntimeAddressBookOwner::resolve_base64(self, hostname)
    }

    fn legacy_publish(&self, hostname: String, destination: String) {
        self.legacy_publish_sync(RuntimeAddressBookEntry { hostname, destination });
    }

    fn legacy_remove(&self, hostname: &str) {
        self.legacy_remove_sync(hostname);
    }
}

/// Dedicated administrative handle. All Proposal 170 state and mutation
/// semantics stay in this I2PControl-owned module.
#[derive(Clone)]
pub struct RuntimeAddressBookHandle {
    pub(crate) owner: Arc<RuntimeAddressBookOwner>,
}

/// Compose the ordinary downloader with the I2PControl-owned runtime hook.
/// The downloader remains the only legacy resolver/publication owner; this
/// function only wires the administrative adapter into its neutral seam.
pub async fn new_controlled_manager(
    base_path: PathBuf,
    config: AddressBookConfig,
) -> (AddressBookManager, Arc<RuntimeAddressBookHandle>) {
    let configured_subscriptions = config.subscriptions.clone().unwrap_or_default();
    let manager = AddressBookManager::new(base_path, config).await;
    let owner = RuntimeAddressBookOwner::new(manager.runtime_context(), configured_subscriptions).await;
    let hook: Arc<dyn AddressBookRuntimeHook> = Arc::new(Arc::clone(&owner));
    let handle = RuntimeAddressBookHandle::new(owner);
    (manager.with_runtime_hook(hook), handle)
}

impl RuntimeAddressBookHandle {
    pub(crate) fn new(owner: Arc<RuntimeAddressBookOwner>) -> Arc<Self> {
        Arc::new(Self { owner })
    }

    pub fn runtime_initialization_error(&self) -> Option<String> {
        self.owner.initialization_error()
    }

    pub fn runtime_authority_present(&self) -> bool {
        self.owner.authority_present()
    }

    pub async fn runtime_list(
        &self,
        book_type: RuntimeAddressBookType,
    ) -> Result<Vec<RuntimeAddressBookEntry>, String> {
        Ok(self.owner.entries(book_type))
    }

    pub async fn runtime_lookup(
        &self,
        book_type: RuntimeAddressBookType,
        hostname: &str,
    ) -> Result<Option<RuntimeAddressBookEntry>, String> {
        Ok(self.owner.state.read().book(book_type).get(hostname).cloned())
    }

    pub async fn runtime_add(
        &self,
        book_type: RuntimeAddressBookType,
        entry: RuntimeAddressBookEntry,
    ) -> Result<(), String> {
        validate_runtime_entry(&entry.hostname, &entry)?;
        self.owner
            .mutate(|state| {
                if state.book(book_type).contains_key(&entry.hostname) {
                    return Err("address book entry already exists".to_string());
                }
                let occupied_elsewhere = [
                    RuntimeAddressBookType::Private,
                    RuntimeAddressBookType::Local,
                    RuntimeAddressBookType::Router,
                    RuntimeAddressBookType::Published,
                ]
                .into_iter()
                .filter(|book| *book != book_type)
                .any(|book| state.book(book).contains_key(&entry.hostname));
                if occupied_elsewhere {
                    return Err("address book hostname collision".to_string());
                }
                state.book_mut(book_type).insert(entry.hostname.clone(), entry);
                Ok(())
            })
            .await
    }

    pub async fn runtime_update(
        &self,
        book_type: RuntimeAddressBookType,
        entry: RuntimeAddressBookEntry,
    ) -> Result<bool, String> {
        validate_runtime_entry(&entry.hostname, &entry)?;
        self.owner
            .mutate(|state| {
                if !state.book(book_type).contains_key(&entry.hostname) {
                    return Ok(false);
                }
                state.book_mut(book_type).insert(entry.hostname.clone(), entry);
                Ok(true)
            })
            .await
    }

    pub async fn runtime_delete(
        &self,
        book_type: RuntimeAddressBookType,
        hostname: &str,
    ) -> Result<bool, String> {
        self.owner
            .mutate(|state| Ok(state.book_mut(book_type).remove(hostname).is_some()))
            .await
    }

    pub async fn runtime_delete_all(
        &self,
        book_type: RuntimeAddressBookType,
    ) -> Result<bool, String> {
        self.owner
            .mutate(|state| {
                if state.book(book_type).is_empty() {
                    return Ok(false);
                }
                state.book_mut(book_type).clear();
                Ok(true)
            })
            .await
    }

    pub async fn runtime_subscriptions(&self) -> Result<Vec<String>, String> {
        Ok(self.owner.state.read().subscriptions.clone())
    }

    #[allow(dead_code)]
    pub fn runtime_active_subscriptions(&self) -> Vec<String> {
        self.owner.subscription_control.active.read().clone()
    }

    pub async fn runtime_set_subscriptions(&self, subscriptions: Vec<String>) -> Result<(), String> {
        validate_runtime_subscriptions(&subscriptions)?;
        if !self
            .owner
            .subscription_control
            .started
            .load(Ordering::Acquire)
        {
            return Err("address book downloader is unavailable".to_string());
        }
        let (response, result) = futures::channel::oneshot::channel();
        self.owner
            .subscription_control
            .sender
            .send(RuntimeSubscriptionCommand { subscriptions, response })
            .await
            .map_err(|_| "address book subscription command was unavailable".to_string())?;
        result
            .await
            .map_err(|_| "address book subscription command was cancelled".to_string())?
    }

    pub async fn runtime_configuration(&self) -> Result<BTreeMap<String, String>, String> {
        Ok(self.owner.state.read().configuration.clone())
    }

    pub async fn runtime_set_configuration(
        &self,
        configuration: BTreeMap<String, String>,
    ) -> Result<(), String> {
        if configuration.is_empty() {
            return Ok(());
        }
        Err("address book configuration is unsupported".to_string())
    }

    pub async fn runtime_clear_unsupported_configuration(&self) -> Result<(), String> {
        if self.owner.state.read().configuration.is_empty() {
            return Ok(());
        }
        self.owner
            .mutate(|state| {
                state.configuration.clear();
                Ok(())
            })
            .await
    }

    pub async fn import_legacy_runtime_state(
        &self,
        snapshot: RuntimeAddressBookSnapshot,
        destinations: BTreeMap<String, RuntimeAddressBookEntry>,
    ) -> Result<(), String> {
        self.owner.import_legacy(snapshot, destinations).await
    }

    pub async fn repair_published_runtime_state(
        &self,
        destinations: BTreeMap<String, RuntimeAddressBookEntry>,
    ) -> Result<(), String> {
        self.owner.repair_published(destinations).await
    }

    pub async fn legacy_destinations(
        &self,
    ) -> Result<BTreeMap<String, RuntimeAddressBookEntry>, String> {
        load_legacy_destinations(&self.owner.path.join("destinations")).await
    }
}

fn validate_legacy_hostname(hostname: &str) -> bool {
    !hostname.is_empty()
        && hostname.len() <= 254
        && hostname != "."
        && hostname != ".."
        && !hostname.contains('/')
        && !hostname.contains('\\')
        && !hostname.chars().any(|character| character.is_control())
}

async fn load_legacy_destinations(
    path: &std::path::Path,
) -> Result<BTreeMap<String, RuntimeAddressBookEntry>, String> {
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(_) => return Err("legacy destination source is unavailable".to_string()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("legacy destination source is unavailable".to_string());
    }
    let mut entries = tokio::fs::read_dir(path)
        .await
        .map_err(|_| "legacy destination source is unavailable".to_string())?;
    let mut snapshot = BTreeMap::new();
    let mut total_bytes = 0usize;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| "legacy destination source is unavailable".to_string())?
    {
        let file_type = entry
            .file_type()
            .await
            .map_err(|_| "legacy destination source is unavailable".to_string())?;
        if !file_type.is_file() {
            continue;
        }
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| "legacy destination source contains an invalid filename".to_string())?;
        let Some(hostname) = file_name.strip_suffix(".txt") else {
            continue;
        };
        if !validate_legacy_hostname(hostname) {
            return Err("legacy destination source contains an invalid filename".to_string());
        }
        let metadata = entry
            .metadata()
            .await
            .map_err(|_| "legacy destination source is unavailable".to_string())?;
        let file_bytes = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
        if file_bytes > MAX_LEGACY_DESTINATION_FILE_BYTES
            || snapshot.len() >= MAX_LEGACY_DESTINATION_ENTRIES
            || total_bytes.saturating_add(file_bytes) > MAX_LEGACY_DESTINATION_BYTES
        {
            return Err("legacy destination source exceeds its limit".to_string());
        }
        let destination = tokio::fs::read_to_string(entry.path())
            .await
            .map_err(|_| "legacy destination source contains an invalid destination".to_string())?;
        let destination = destination.trim().to_string();
        if destination.len() > MAX_LEGACY_DESTINATION_FILE_BYTES
            || !is_valid_full_destination(&destination)
        {
            return Err("legacy destination source contains an invalid destination".to_string());
        }
        total_bytes = total_bytes.saturating_add(destination.len());
        snapshot.insert(
            hostname.to_string(),
            RuntimeAddressBookEntry {
                hostname: hostname.to_string(),
                destination,
            },
        );
    }
    Ok(snapshot)
}
