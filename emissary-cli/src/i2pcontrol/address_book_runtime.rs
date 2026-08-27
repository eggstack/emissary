//! Runtime AddressBook adapter owned by I2PControl.
//!
//! This module owns Proposal 170 administrative persistence, validation, migration, and
//! publication policy. The parent AddressBook module retains only the legacy downloader and its
//! narrow runtime publication seam.
#![cfg(feature = "i2pcontrol")]

use std::{
    collections::{BTreeMap, HashMap},
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::address_book::{
    AddressBookManager, AddressBookRuntimeContext, AddressBookRuntimeHook, RuntimeRefreshSettings,
    RuntimeSubscriptionCommand, RuntimeSubscriptionControl,
};
use crate::config::AddressBookConfig;
use crate::i2pcontrol::stores::publication::{publish_with_backup, publish_with_backup_sync};
use emissary_core::{
    crypto::{base32_encode, base64_decode},
    primitives::Destination,
};
use parking_lot::RwLock;

pub(crate) const MAX_LEGACY_DESTINATION_ENTRIES: usize = 10_000;
pub(crate) const MAX_LEGACY_DESTINATION_FILE_BYTES: usize = 64 * 1024;
pub(crate) const MAX_LEGACY_DESTINATION_BYTES: usize = 1024 * 1024;
const CONFIG_SCHEMA_VERSION: u32 = 2;
const DEFAULT_UPDATE_DELAY_HOURS: u64 = 24;
const MIN_UPDATE_DELAY_HOURS: u64 = 1;
const MAX_UPDATE_DELAY_HOURS: u64 = 24 * 30;
const MAX_CONFIGURATION_PATH_LENGTH: usize = 1024;
const MAX_THEME_LENGTH: usize = 128;
const MAX_ADMIN_FILE_BYTES: usize = 4 * 1024 * 1024;

const CONFIG_KEYS: &[&str] = &[
    "subscriptions",
    "published_addressbook",
    "router_addressbook",
    "local_addressbook",
    "private_addressbook",
    "etags",
    "last_modified",
    "log",
    "update_delay",
    "proxy_port",
    "proxy_host",
    "should_publish",
    "theme",
];

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RuntimeAddressBookConfiguration {
    pub version: u32,
    #[serde(default)]
    pub paths: BTreeMap<String, String>,
    pub update_delay_hours: u64,
    pub proxy_port: Option<u16>,
    pub proxy_host: Option<String>,
    pub should_publish: bool,
    pub theme: Option<String>,
    #[serde(default)]
    pub explicit_keys: std::collections::BTreeSet<String>,
}

impl Default for RuntimeAddressBookConfiguration {
    fn default() -> Self {
        Self {
            version: CONFIG_SCHEMA_VERSION,
            paths: BTreeMap::new(),
            update_delay_hours: DEFAULT_UPDATE_DELAY_HOURS,
            proxy_port: None,
            proxy_host: None,
            should_publish: true,
            theme: None,
            explicit_keys: std::collections::BTreeSet::new(),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RuntimeAddressBookConfiguration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        if !value.is_object() {
            return Err(serde::de::Error::custom(
                "address book configuration must be an object",
            ));
        }
        if value.get("version").is_none() && value.get("paths").is_none() {
            // M034 stored rejected values as a flat string map. Those values
            // were never active, so loading them as defaults preserves the
            // durable authority without reviving inert behavior.
            return Ok(Self::default());
        }
        #[derive(serde::Deserialize)]
        struct Stored {
            #[serde(default = "default_config_version")]
            version: u32,
            #[serde(default)]
            paths: BTreeMap<String, String>,
            #[serde(default = "default_update_delay")]
            update_delay_hours: u64,
            #[serde(default)]
            proxy_port: Option<u16>,
            #[serde(default)]
            proxy_host: Option<String>,
            #[serde(default = "default_should_publish")]
            should_publish: bool,
            #[serde(default)]
            theme: Option<String>,
            #[serde(default)]
            explicit_keys: std::collections::BTreeSet<String>,
        }
        fn default_config_version() -> u32 {
            CONFIG_SCHEMA_VERSION
        }
        fn default_update_delay() -> u64 {
            DEFAULT_UPDATE_DELAY_HOURS
        }
        fn default_should_publish() -> bool {
            true
        }
        let stored: Stored = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            version: stored.version,
            paths: stored.paths,
            update_delay_hours: stored.update_delay_hours,
            proxy_port: stored.proxy_port,
            proxy_host: stored.proxy_host,
            should_publish: stored.should_publish,
            theme: stored.theme,
            explicit_keys: stored.explicit_keys,
        })
    }
}

impl RuntimeAddressBookConfiguration {
    fn validate_stored(&self) -> Result<(), String> {
        if self.version != CONFIG_SCHEMA_VERSION {
            return Err("address book configuration schema version is unsupported".to_string());
        }
        if self.explicit_keys.iter().any(|key| !CONFIG_KEYS.contains(&key.as_str())) {
            return Err("address book configuration contains an unknown key".to_string());
        }
        for key in self.paths.keys() {
            if !matches!(
                key.as_str(),
                "subscriptions"
                    | "published_addressbook"
                    | "router_addressbook"
                    | "local_addressbook"
                    | "private_addressbook"
                    | "etags"
                    | "last_modified"
                    | "log"
            ) || !self.explicit_keys.contains(key)
            {
                return Err("address book configuration contains an invalid path".to_string());
            }
            let value = self.paths.get(key).expect("path key was just checked");
            if value.is_empty() || value.len() > MAX_CONFIGURATION_PATH_LENGTH {
                return Err("address book configuration path is invalid".to_string());
            }
        }
        for key in &self.explicit_keys {
            if matches!(
                key.as_str(),
                "subscriptions"
                    | "published_addressbook"
                    | "router_addressbook"
                    | "local_addressbook"
                    | "private_addressbook"
                    | "etags"
                    | "last_modified"
                    | "log"
            ) && !self.paths.contains_key(key)
            {
                return Err("address book configuration is incomplete".to_string());
            }
        }
        if self.update_delay_hours < MIN_UPDATE_DELAY_HOURS
            || self.update_delay_hours > MAX_UPDATE_DELAY_HOURS
        {
            return Err("update_delay is outside its supported range".to_string());
        }
        if self.explicit_keys.contains("proxy_port") && self.proxy_port == Some(0) {
            return Err("proxy_port must be non-zero".to_string());
        }
        if let Some(host) = &self.proxy_host {
            validate_proxy_host(host)?;
        }
        if let Some(theme) = &self.theme {
            if theme.is_empty() || theme.len() > MAX_THEME_LENGTH {
                return Err("theme is outside its supported size".to_string());
            }
        }
        for key in ["proxy_port", "proxy_host", "should_publish", "theme"] {
            if self.explicit_keys.contains(key)
                && match key {
                    "proxy_port" => self.proxy_port.is_none(),
                    "proxy_host" => self.proxy_host.is_none(),
                    "theme" => self.theme.is_none(),
                    _ => false,
                }
            {
                return Err("address book configuration is incomplete".to_string());
            }
        }
        Ok(())
    }

    fn from_external(values: &BTreeMap<String, String>) -> Result<Self, String> {
        let mut configuration = Self::default();
        for (key, value) in values {
            if !CONFIG_KEYS.contains(&key.as_str()) {
                return Err("unknown address book configuration key".to_string());
            }
            configuration.explicit_keys.insert(key.clone());
            match key.as_str() {
                "subscriptions"
                | "published_addressbook"
                | "router_addressbook"
                | "local_addressbook"
                | "private_addressbook"
                | "etags"
                | "last_modified"
                | "log" => {
                    if value.is_empty() || value.len() > MAX_CONFIGURATION_PATH_LENGTH {
                        return Err("address book configuration path is invalid".to_string());
                    }
                    configuration.paths.insert(key.clone(), value.clone());
                }
                "update_delay" => {
                    let hours = value.parse::<u64>().map_err(|_| {
                        "update_delay must be an integer number of hours".to_string()
                    })?;
                    if !(MIN_UPDATE_DELAY_HOURS..=MAX_UPDATE_DELAY_HOURS).contains(&hours) {
                        return Err("update_delay is outside its supported range".to_string());
                    }
                    configuration.update_delay_hours = hours;
                }
                "proxy_port" => {
                    let port = value
                        .parse::<u16>()
                        .map_err(|_| "proxy_port must be a valid port".to_string())?;
                    if port == 0 {
                        return Err("proxy_port must be non-zero".to_string());
                    }
                    configuration.proxy_port = Some(port);
                }
                "proxy_host" => {
                    validate_proxy_host(value)?;
                    configuration.proxy_host = Some(value.clone());
                }
                "should_publish" => {
                    configuration.should_publish = match value.as_str() {
                        "true" => true,
                        "false" => false,
                        _ => return Err("should_publish must be true or false".to_string()),
                    };
                }
                "theme" => {
                    if value.is_empty() || value.len() > MAX_THEME_LENGTH {
                        return Err("theme is outside its supported size".to_string());
                    }
                    configuration.theme = Some(value.clone());
                }
                _ => unreachable!(),
            }
        }
        Ok(configuration)
    }

    fn external_map(&self) -> BTreeMap<String, String> {
        self.explicit_keys
            .iter()
            .filter_map(|key| {
                let value = match key.as_str() {
                    "update_delay" => self.update_delay_hours.to_string(),
                    "proxy_port" => self.proxy_port?.to_string(),
                    "proxy_host" => self.proxy_host.clone()?,
                    "should_publish" => self.should_publish.to_string(),
                    "theme" => self.theme.clone()?,
                    _ => self.paths.get(key)?.clone(),
                };
                Some((key.clone(), value))
            })
            .collect()
    }

    fn path_value(&self, key: &str, default: &str) -> String {
        self.paths.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
}

fn validate_proxy_host(host: &str) -> Result<(), String> {
    if host.is_empty()
        || host.len() > 254
        || host
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || host.chars().any(|character| matches!(character, '/' | '\\' | '@'))
    {
        return Err("proxy_host is invalid".to_string());
    }
    if host.starts_with('[') || host.ends_with(']') {
        let Some(inner) = host.strip_prefix('[').and_then(|host| host.strip_suffix(']')) else {
            return Err("proxy_host is invalid".to_string());
        };
        if inner.parse::<std::net::Ipv6Addr>().is_err() {
            return Err("proxy_host is invalid".to_string());
        }
    }
    if host.parse::<std::net::IpAddr>().is_err()
        && !host.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'[' | b']')
        })
    {
        return Err("proxy_host is invalid".to_string());
    }
    Ok(())
}

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

fn validate_loadable_snapshot(state: &RuntimeAddressBookSnapshot) -> Result<(), String> {
    let books = [
        (&state.private, false),
        (&state.local, false),
        (&state.router, false),
        (&state.published, true),
    ];
    let total_entries = books.iter().map(|(book, _)| book.len()).sum::<usize>();
    if total_entries > MAX_LEGACY_DESTINATION_ENTRIES {
        return Err("address book state exceeds its entry limit".to_string());
    }
    let mut hostnames = std::collections::BTreeSet::new();
    for (book, published) in books {
        for (hostname, entry) in book {
            if published && !is_valid_full_destination(&entry.destination) {
                if hostname != &entry.hostname
                    || hostname.is_empty()
                    || hostname.len() > 254
                    || hostname.contains('/')
                    || hostname.contains('\\')
                    || hostname.chars().any(|character| character.is_control())
                {
                    return Err("address book state contains an invalid hostname".to_string());
                }
            } else {
                validate_runtime_entry(hostname, entry)?;
            }
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
    /// Versioned, validated Proposal 170 configuration.
    pub configuration: RuntimeAddressBookConfiguration,
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
        if let Err(error) = ensure_admin_root(&path).await {
            initialization_error = Some(error);
        }

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
        let mut state = loaded.unwrap_or_else(|| {
            if authority_present {
                initialization_error = Some("address book state is corrupt".to_string());
            }
            RuntimeAddressBookSnapshot {
                subscriptions: initial_subscriptions_for_fallback.clone(),
                ..RuntimeAddressBookSnapshot::default()
            }
        });
        if state.configuration.validate_stored().is_err()
            || validate_loadable_snapshot(&state).is_err()
        {
            initialization_error = Some("address book state is corrupt".to_string());
            state = RuntimeAddressBookSnapshot {
                subscriptions: initial_subscriptions_for_fallback,
                ..RuntimeAddressBookSnapshot::default()
            };
        }
        let active_subscriptions = state.subscriptions.clone();
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        let settings = RuntimeRefreshSettings::defaults(&path, "127.0.0.1", 4444);
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
                settings: RwLock::new(settings),
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

    fn artifact_paths(
        &self,
        configuration: &RuntimeAddressBookConfiguration,
    ) -> Result<[PathBuf; 5], String> {
        let defaults = [
            "private_addressbook",
            "local_addressbook",
            "router_addressbook",
            "published_addressbook",
            "subscriptions",
        ];
        let keys = [
            "private_addressbook",
            "local_addressbook",
            "router_addressbook",
            "published_addressbook",
            "subscriptions",
        ];
        let mut paths = [
            PathBuf::new(),
            PathBuf::new(),
            PathBuf::new(),
            PathBuf::new(),
            PathBuf::new(),
        ];
        for ((path, key), default) in paths.iter_mut().zip(keys).zip(defaults) {
            *path = resolve_confined_path(&self.path, &configuration.path_value(key, default))?;
        }
        for (index, path) in paths.iter().enumerate() {
            if paths[..index].contains(path) {
                return Err("address book configuration paths must be distinct".to_string());
            }
        }
        Ok(paths)
    }

    async fn publish_configured_artifacts(
        &self,
        state: &RuntimeAddressBookSnapshot,
    ) -> Result<(), String> {
        let paths = self.artifact_paths(&state.configuration)?;
        for (path, book) in paths[..3].iter().zip([&state.private, &state.local, &state.router]) {
            atomic_write(path, &serialize_book(book)?).await?;
        }
        if state.configuration.should_publish {
            atomic_write(&paths[3], &serialize_book(&state.published)?).await?;
        }
        atomic_write(&paths[4], state.subscriptions.join("\n").as_bytes()).await?;
        if let Some(log_path) = state.configuration.paths.get("log") {
            let log_path = resolve_confined_path(&self.path, log_path)?;
            atomic_write(
                &log_path,
                b"AddressBook administrative generation committed\n",
            )
            .await?;
        }
        Ok(())
    }

    async fn load_configured_generation(
        &self,
        configuration: &RuntimeAddressBookConfiguration,
        current: &RuntimeAddressBookSnapshot,
    ) -> Result<RuntimeAddressBookSnapshot, String> {
        let paths = self.artifact_paths(configuration)?;
        let mut next = current.clone();
        for (slot, path) in [
            &mut next.private,
            &mut next.local,
            &mut next.router,
            &mut next.published,
        ]
        .into_iter()
        .zip(paths[..4].iter())
        {
            if let Some(book) = load_book(path).await? {
                *slot = book;
            }
        }
        if let Some(bytes) = read_bounded(&paths[4]).await? {
            let raw = String::from_utf8(bytes)
                .map_err(|_| "configured subscription file is invalid".to_string())?;
            let subscriptions = raw
                .lines()
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>();
            validate_runtime_subscriptions(&subscriptions)?;
            next.subscriptions = subscriptions;
        }
        next.configuration = configuration.clone();
        validate_runtime_snapshot(&next)?;
        Ok(next)
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
        self.publish_configured_artifacts(&state).await?;
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
        self.publish_configured_artifacts(&state).await?;
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
        // Rejected M034 configuration values were inert metadata. Do not
        // promote them into the operational authority during migration.
        merged.configuration = RuntimeAddressBookConfiguration::default();
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
        if validate_runtime_snapshot(&state).is_ok()
            && self.publish_configured_artifacts(&state).await.is_ok()
            && self.persist(&state).await.is_ok()
        {
            *self.state.write() = state;
            self.authority_present.store(true, Ordering::Release);
            self.rebuild_runtime_indexes();
        }
    }

    async fn set_configuration(
        &self,
        configuration: RuntimeAddressBookConfiguration,
    ) -> Result<(), String> {
        let defaults = self.subscription_control.settings.read().clone();
        let settings = resolved_settings(&self.path, &configuration, &defaults)?;
        let mut all_paths = self.artifact_paths(&configuration)?.to_vec();
        all_paths.push(settings.etags_path.clone());
        all_paths.push(settings.last_modified_path.clone());
        if let Some(log_path) = configuration.paths.get("log") {
            all_paths.push(resolve_confined_path(&self.path, log_path)?);
        }
        all_paths.sort();
        if all_paths.windows(2).any(|paths| paths[0] == paths[1]) {
            return Err("address book configuration paths must be distinct".to_string());
        }
        let _guard = self.mutation.lock().await;
        let current = self.snapshot();
        let mut next = self.load_configured_generation(&configuration, &current).await?;
        next.configuration = configuration;
        self.publish_configured_artifacts(&next).await?;
        self.persist(&next).await?;
        *self.state.write() = next;
        *self.subscription_control.active.write() = self.state.read().subscriptions.clone();
        *self.subscription_control.settings.write() = settings;
        self.authority_present.store(true, Ordering::Release);
        self.rebuild_runtime_indexes();
        Ok(())
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

fn resolve_confined_path(root: &Path, raw: &str) -> Result<PathBuf, String> {
    if raw.is_empty()
        || raw.len() > MAX_CONFIGURATION_PATH_LENGTH
        || raw.contains('\0')
        || raw.chars().any(|character| character.is_control())
        || raw.contains('\\')
    {
        return Err("address book path is invalid".to_string());
    }
    let candidate = Path::new(raw);
    if candidate.is_absolute() {
        return Err("address book path must be relative to its administrative root".to_string());
    }

    let mut normalized = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err("address book path escapes its administrative root".to_string());
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("address book path is invalid".to_string());
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err("address book path must name a file".to_string());
    }
    if matches!(
        normalized.file_name().and_then(|name| name.to_str()),
        Some(
            "control-state.json"
                | "control-state.json.bak"
                | ".control-state.json.tmp"
                | "addresses"
                | "host_modified_times"
                | "destinations"
        )
    ) {
        return Err("address book path names a reserved runtime artifact".to_string());
    }

    let root = std::fs::canonicalize(root)
        .map_err(|_| "address book administrative root is unavailable".to_string())?;
    let path = root.join(normalized);
    let mut current = root.clone();
    let relative = path
        .strip_prefix(&root)
        .map_err(|_| "address book path escapes its administrative root".to_string())?;
    let components = relative.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink()
                    || (index + 1 < components.len() && !metadata.is_dir())
                    || (index + 1 == components.len() && !metadata.is_file())
                {
                    return Err("address book path is not a regular confined file".to_string());
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                break;
            }
            Err(_) => return Err("address book path is unavailable".to_string()),
        }
    }
    Ok(path)
}

async fn ensure_admin_root(root: &Path) -> Result<(), String> {
    tokio::fs::create_dir_all(root)
        .await
        .map_err(|_| "address book administrative root is unavailable".to_string())?;
    let metadata = tokio::fs::symlink_metadata(root)
        .await
        .map_err(|_| "address book administrative root is unavailable".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("address book administrative root is invalid".to_string());
    }
    Ok(())
}

fn resolved_settings(
    root: &Path,
    configuration: &RuntimeAddressBookConfiguration,
    defaults: &RuntimeRefreshSettings,
) -> Result<RuntimeRefreshSettings, String> {
    let mut settings = defaults.clone();
    settings.interval = Duration::from_secs(
        configuration
            .update_delay_hours
            .checked_mul(60 * 60)
            .ok_or_else(|| "update_delay is outside its supported range".to_string())?,
    );
    settings.proxy_host =
        configuration.proxy_host.clone().unwrap_or_else(|| defaults.proxy_host.clone());
    settings.proxy_port = configuration.proxy_port.unwrap_or(defaults.proxy_port);
    settings.should_publish = configuration.should_publish;
    settings.subscriptions_path = resolve_confined_path(
        root,
        &configuration.path_value("subscriptions", "subscriptions"),
    )?;
    settings.etags_path = resolve_confined_path(root, &configuration.path_value("etags", "etags"))?;
    settings.last_modified_path = resolve_confined_path(
        root,
        &configuration.path_value("last_modified", "last_modified"),
    )?;
    Ok(settings)
}

async fn read_bounded(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("address book configured file is unavailable".to_string()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("address book configured file is not a regular file".to_string());
    }
    if metadata.len() > MAX_ADMIN_FILE_BYTES as u64 {
        return Err("address book configured file exceeds its size limit".to_string());
    }
    tokio::fs::read(path)
        .await
        .map(Some)
        .map_err(|_| "address book configured file is unavailable".to_string())
}

async fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_ADMIN_FILE_BYTES {
        return Err("address book configured file exceeds its size limit".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "address book configured file has no parent".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|_| "address book configured directory is unavailable".to_string())?;
    let metadata = tokio::fs::symlink_metadata(path).await;
    if let Ok(metadata) = metadata {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("address book configured file is not a regular file".to_string());
        }
    }
    let temporary = path.with_extension("m096-tmp");
    atomic_write_inner(&temporary, path, bytes).await
}

async fn atomic_write_inner(temporary: &Path, target: &Path, bytes: &[u8]) -> Result<(), String> {
    tokio::fs::write(temporary, bytes)
        .await
        .map_err(|_| "address book configured file publication failed".to_string())?;
    let file = tokio::fs::File::open(temporary)
        .await
        .map_err(|_| "address book configured file publication failed".to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(temporary, std::fs::Permissions::from_mode(0o600))
            .await
            .map_err(|_| "address book configured file publication failed".to_string())?;
    }
    file.sync_all()
        .await
        .map_err(|_| "address book configured file publication failed".to_string())?;
    tokio::fs::rename(temporary, target)
        .await
        .map_err(|_| "address book configured file publication failed".to_string())
}

fn serialize_book(book: &BTreeMap<String, RuntimeAddressBookEntry>) -> Result<Vec<u8>, String> {
    serde_json::to_vec(book).map_err(|_| "address book serialization failed".to_string())
}

async fn load_book(
    path: &Path,
) -> Result<Option<BTreeMap<String, RuntimeAddressBookEntry>>, String> {
    let Some(bytes) = read_bounded(path).await? else {
        return Ok(None);
    };
    let book = serde_json::from_slice(&bytes)
        .map_err(|_| "configured address book file is invalid".to_string())?;
    validate_runtime_book(&book)?;
    Ok(Some(book))
}

fn validate_runtime_book(book: &BTreeMap<String, RuntimeAddressBookEntry>) -> Result<(), String> {
    if book.len() > MAX_LEGACY_DESTINATION_ENTRIES {
        return Err("configured address book exceeds its entry limit".to_string());
    }
    for (hostname, entry) in book {
        validate_runtime_entry(hostname, entry)?;
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

    fn refresh_settings(&self, defaults: RuntimeRefreshSettings) -> RuntimeRefreshSettings {
        resolved_settings(&self.path, &self.state.read().configuration, &defaults)
            .unwrap_or(defaults)
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

    fn publish_artifacts(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        let owner = Arc::clone(self);
        Box::pin(async move {
            let state = owner.snapshot();
            if let Err(error) = owner.publish_configured_artifacts(&state).await {
                tracing::warn!(target: "emissary::i2pcontrol::address_book", ?error, "address book artifact publication failed");
            }
        })
    }

    fn resolve_base32(&self, hostname: &str) -> Option<String> {
        RuntimeAddressBookOwner::resolve_base32(self, hostname)
    }

    fn resolve_base64(&self, hostname: &str) -> Option<String> {
        RuntimeAddressBookOwner::resolve_base64(self, hostname)
    }

    fn legacy_publish(&self, hostname: String, destination: String) {
        self.legacy_publish_sync(RuntimeAddressBookEntry {
            hostname,
            destination,
        });
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
    let owner =
        RuntimeAddressBookOwner::new(manager.runtime_context(), configured_subscriptions).await;
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

    pub async fn runtime_set_subscriptions(
        &self,
        subscriptions: Vec<String>,
    ) -> Result<(), String> {
        validate_runtime_subscriptions(&subscriptions)?;
        if !self.owner.subscription_control.started.load(Ordering::Acquire) {
            return Err("address book downloader is unavailable".to_string());
        }
        let (response, result) = futures::channel::oneshot::channel();
        self.owner
            .subscription_control
            .sender
            .send(RuntimeSubscriptionCommand {
                subscriptions,
                response,
            })
            .await
            .map_err(|_| "address book subscription command was unavailable".to_string())?;
        result
            .await
            .map_err(|_| "address book subscription command was cancelled".to_string())?
    }

    pub async fn runtime_configuration(&self) -> Result<BTreeMap<String, String>, String> {
        Ok(self.owner.state.read().configuration.external_map())
    }

    #[allow(dead_code)]
    pub(crate) fn runtime_refresh_settings(&self) -> RuntimeRefreshSettings {
        let defaults = self.owner.subscription_control.settings.read().clone();
        resolved_settings(
            &self.owner.path,
            &self.owner.state.read().configuration,
            &defaults,
        )
        .unwrap_or(defaults)
    }

    pub async fn runtime_set_configuration(
        &self,
        configuration: BTreeMap<String, String>,
    ) -> Result<(), String> {
        if configuration.is_empty() {
            return Ok(());
        }
        let parsed = RuntimeAddressBookConfiguration::from_external(&configuration)
            .map_err(|error| format!("configuration validation failed: {error}"))?;
        ensure_admin_root(&self.owner.path).await?;
        self.owner.set_configuration(parsed).await?;

        // The existing downloader worker is the sole refresh owner. A
        // configuration commit updates its shared settings and queues one
        // bounded refresh; no second worker is created.
        if self.owner.subscription_control.started.load(Ordering::Acquire) {
            let (response, result) = futures::channel::oneshot::channel();
            let subscriptions = self.owner.subscription_control.active.read().clone();
            if self
                .owner
                .subscription_control
                .sender
                .send(RuntimeSubscriptionCommand {
                    subscriptions,
                    response,
                })
                .await
                .is_ok()
            {
                let _ = result.await;
            } else {
                tracing::warn!(target: "emissary::i2pcontrol::address_book", "address book refresh worker was unavailable after configuration commit");
            }
        }
        Ok(())
    }

    pub async fn runtime_clear_unsupported_configuration(&self) -> Result<(), String> {
        if self.owner.state.read().configuration.explicit_keys.is_empty() {
            return Ok(());
        }
        self.owner
            .mutate(|state| {
                state.configuration = RuntimeAddressBookConfiguration::default();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn config(values: &[(&str, &str)]) -> BTreeMap<String, String> {
        values
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn confined_paths_normalize_without_escaping() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            resolve_confined_path(root.path(), "nested/../chosen.json").unwrap(),
            root.path().join("chosen.json")
        );
        assert!(resolve_confined_path(root.path(), "../../outside.json").is_err());
        assert!(resolve_confined_path(root.path(), "/tmp/outside.json").is_err());
        assert!(resolve_confined_path(root.path(), "nested\\outside.json").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn confined_paths_reject_symlink_escape_and_special_files() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), root.path().join("escape")).unwrap();
        assert!(resolve_confined_path(root.path(), "escape/file.json").is_err());

        std::fs::write(root.path().join("file.json"), b"{}").unwrap();
        assert!(resolve_confined_path(root.path(), "file.json").is_ok());
        assert!(resolve_confined_path(root.path(), "control-state.json").is_err());
    }

    #[tokio::test]
    async fn configuration_is_typed_confined_and_restart_safe() {
        let base = tempfile::tempdir().unwrap().keep();
        let (_manager, handle) = new_controlled_manager(
            base.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let values = config(&[
            ("private_addressbook", "books/private.json"),
            ("local_addressbook", "books/local.json"),
            ("router_addressbook", "books/router.json"),
            ("published_addressbook", "books/published.json"),
            ("subscriptions", "meta/subscriptions"),
            ("etags", "meta/etags"),
            ("last_modified", "meta/last-modified"),
            ("log", "meta/addressbook.log"),
            ("update_delay", "2"),
            ("proxy_host", "127.0.0.1"),
            ("proxy_port", "4445"),
            ("should_publish", "false"),
            ("theme", "dark"),
        ]);
        handle.runtime_set_configuration(values.clone()).await.unwrap();
        assert_eq!(handle.runtime_configuration().await.unwrap(), values);
        assert_eq!(
            handle.runtime_refresh_settings().interval,
            Duration::from_secs(7200)
        );
        assert_eq!(handle.runtime_refresh_settings().proxy_port, 4445);
        assert!(!handle.runtime_refresh_settings().should_publish);
        assert!(base.join("addressbook/meta/addressbook.log").exists());

        drop(handle);
        let (_manager, restarted) = new_controlled_manager(
            base,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        assert_eq!(restarted.runtime_configuration().await.unwrap(), values);
        assert_eq!(restarted.runtime_refresh_settings().proxy_port, 4445);
    }

    #[tokio::test]
    async fn invalid_path_and_target_failure_preserve_prior_generation() {
        let base = tempfile::tempdir().unwrap().keep();
        let (_manager, handle) = new_controlled_manager(
            base,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let initial = config(&[("theme", "light")]);
        handle.runtime_set_configuration(initial.clone()).await.unwrap();
        assert!(handle
            .runtime_set_configuration(config(&[("private_addressbook", "../../escape.json")]))
            .await
            .is_err());
        assert_eq!(handle.runtime_configuration().await.unwrap(), initial);

        let root = handle.owner.path.clone();
        tokio::fs::write(root.join("bad.json"), b"not-json").await.unwrap();
        assert!(handle
            .runtime_set_configuration(config(&[("private_addressbook", "bad.json")]))
            .await
            .is_err());
        assert_eq!(handle.runtime_configuration().await.unwrap(), initial);
    }
}
