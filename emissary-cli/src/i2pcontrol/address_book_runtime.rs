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

/// One bounded command sent to the live address-book manager.
pub(crate) struct RuntimeSubscriptionCommand {
    pub(crate) subscriptions: Vec<String>,
    pub(crate) response: futures::channel::oneshot::Sender<Result<(), String>>,
}

/// The only live control seam for changing the downloader's subscription set.
pub(crate) struct RuntimeSubscriptionControl {
    pub(crate) sender: tokio::sync::mpsc::Sender<RuntimeSubscriptionCommand>,
    pub(crate) active: RwLock<Vec<String>>,
    pub(crate) started: AtomicBool,
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
}

#[cfg(feature = "i2pcontrol")]
impl RuntimeAddressBookOwner {
    pub(crate) async fn new(
        path: PathBuf,
        addresses: Arc<RwLock<HashMap<String, String>>>,
        serialized: Arc<RwLock<String>>,
        initial_subscriptions: Vec<String>,
    ) -> Arc<Self> {
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
        let owner = Arc::new(Self {
            path,
            addresses,
            serialized,
            state: RwLock::new(state),
            mutation: tokio::sync::Mutex::new(()),
            authority_present: AtomicBool::new(authority_present),
            initialization_error,
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

    pub(crate) async fn merge_downloaded(&self, addresses: HashMap<String, (String, String)>) {
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
}
