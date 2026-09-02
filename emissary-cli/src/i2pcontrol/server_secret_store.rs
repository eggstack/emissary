//! Persistent identities for control-plane-owned generic server tunnels.
//!
//! This store is deliberately narrower than a general secret manager. It has
//! one fixed location below the I2PControl state root, one bounded map keyed by
//! an internal identity, and no request-selected path or filename.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::{Component, Path, PathBuf},
};

use emissary_core::crypto::base64_decode;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::i2pcontrol::stores::publication::publish_with_backup;

const STORE_DIRECTORY: &str = "server-destinations";
const CURRENT_FILE: &str = "current.json";
const BACKUP_FILE: &str = "backup.json";
const MAX_STORE_SIZE: usize = 1024 * 1024;
const MAX_ENTRIES: usize = 1000;
const MAX_ID_LENGTH: usize = 128;
const MAX_SECRET_LENGTH: usize = 64 * 1024;
const IMPORT_DIRECTORY: &str = "server-key-imports";
const MAX_REFERENCE_LENGTH: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecretEnvelope {
    version: u32,
    entries: BTreeMap<String, String>,
}

/// A private destination returned only to the runtime adapter.
///
/// The value cannot be displayed or debug-printed accidentally.
#[derive(Clone, PartialEq, Eq)]
pub struct StoredDestination(String);

impl StoredDestination {
    /// Wrap freshly generated private material without exposing it to the
    /// public domain model.
    pub(crate) fn from_private(value: String) -> Self {
        Self(value)
    }

    /// Borrow the private destination for immediate session construction.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for StoredDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StoredDestination(***)")
    }
}

impl fmt::Display for StoredDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

#[derive(Debug)]
struct StoreState {
    entries: BTreeMap<String, String>,
}

/// Backend-owned persistent destination identity store.
#[derive(Clone)]
pub struct ServerDestinationStore {
    root: PathBuf,
    state: std::sync::Arc<tokio::sync::Mutex<StoreState>>,
    mutation: std::sync::Arc<tokio::sync::Mutex<()>>,
}

impl fmt::Debug for ServerDestinationStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.state.try_lock().map(|state| state.entries.len()).unwrap_or(0);
        f.debug_struct("ServerDestinationStore")
            .field("root", &self.root)
            .field("entry_count", &count)
            .finish()
    }
}

impl ServerDestinationStore {
    /// Create a store below a fixed, purpose-specific directory.
    pub fn new(state_root: impl Into<PathBuf>) -> Self {
        Self {
            root: state_root.into().join(STORE_DIRECTORY),
            state: std::sync::Arc::new(tokio::sync::Mutex::new(StoreState {
                entries: BTreeMap::new(),
            })),
            mutation: std::sync::Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    /// Return the fixed store directory, for diagnostics and tests only.
    #[allow(dead_code)]
    pub fn directory(&self) -> &Path {
        &self.root
    }

    /// Load the newest valid current/backup state.
    pub async fn load(&self) -> Result<(), String> {
        let _guard = self.mutation.lock().await;
        self.validate_root().await?;
        let current = self.read_file(CURRENT_FILE).await?;
        let backup = self.read_file(BACKUP_FILE).await?;
        let entries = match (current, backup) {
            (Some(Ok(entries)), _) => entries,
            (_, Some(Ok(entries))) => entries,
            (Some(Err(_)), Some(Err(error))) => return Err(error),
            (Some(Err(error)), None) => return Err(error),
            (None, Some(Err(error))) => return Err(error),
            (None, None) => BTreeMap::new(),
        };
        let mut state = self.state.lock().await;
        state.entries = entries;
        Ok(())
    }

    /// Look up private material by the stable internal identity.
    pub async fn get(&self, identity: &str) -> Result<Option<StoredDestination>, String> {
        validate_identity(identity)?;
        let state = self.state.lock().await;
        Ok(state.entries.get(identity).cloned().map(StoredDestination))
    }

    /// Publish or replace one validated identity.
    pub async fn put(&self, identity: &str, destination: StoredDestination) -> Result<(), String> {
        validate_identity(identity)?;
        validate_destination(destination.as_str())?;
        let _guard = self.mutation.lock().await;
        let entries = {
            let state = self.state.lock().await;
            let mut entries = state.entries.clone();
            if !entries.contains_key(identity) && entries.len() >= MAX_ENTRIES {
                return Err("server destination store capacity exhausted".to_string());
            }
            entries.insert(identity.to_string(), destination.0);
            entries
        };
        self.publish(&entries).await?;
        self.state.lock().await.entries = entries;
        Ok(())
    }

    /// Import private destination material from the confined administrative
    /// import root. The returned value is only intended for an immediate
    /// subsequent `put`; runtime use never follows the external file.
    pub async fn import_reference(&self, reference: &str) -> Result<StoredDestination, String> {
        validate_reference(reference)?;
        let import_root = self
            .root
            .parent()
            .ok_or_else(|| "server destination store has no administrative root".to_string())?
            .join(IMPORT_DIRECTORY);
        tokio::fs::create_dir_all(&import_root)
            .await
            .map_err(|_| "server private-key import root is unavailable".to_string())?;
        let metadata = tokio::fs::symlink_metadata(&import_root)
            .await
            .map_err(|_| "server private-key import root is unavailable".to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("server private-key import root is not a directory".to_string());
        }
        let path = import_root.join(reference);
        reject_symlink_components(&import_root, &path).await?;
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|_| "server private-key import is unavailable".to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("server private-key import is not a regular file".to_string());
        }
        if metadata.len() as usize > MAX_SECRET_LENGTH {
            return Err("server private-key import is oversized".to_string());
        }
        let value = String::from_utf8(
            tokio::fs::read(path)
                .await
                .map_err(|_| "server private-key import could not be read".to_string())?,
        )
        .map_err(|_| "server private-key import is not valid text".to_string())?
        .trim()
        .to_owned();
        validate_destination(&value)?;
        Ok(StoredDestination::from_private(value))
    }

    /// Remove one identity after its owning definition has been removed.
    pub async fn remove(&self, identity: &str) -> Result<bool, String> {
        validate_identity(identity)?;
        let _guard = self.mutation.lock().await;
        let entries = {
            let state = self.state.lock().await;
            if !state.entries.contains_key(identity) {
                return Ok(false);
            }
            let mut entries = state.entries.clone();
            entries.remove(identity);
            entries
        };
        self.publish(&entries).await?;
        self.state.lock().await.entries = entries;
        Ok(true)
    }

    /// Remove crash leftovers that are no longer claimed by a definition.
    pub async fn prune_unreferenced(&self, referenced: &BTreeSet<String>) -> Result<(), String> {
        let _guard = self.mutation.lock().await;
        let entries = {
            let state = self.state.lock().await;
            state
                .entries
                .iter()
                .filter(|(identity, _)| referenced.contains(*identity))
                .map(|(identity, destination)| (identity.clone(), destination.clone()))
                .collect::<BTreeMap<_, _>>()
        };
        if entries.len() == self.state.lock().await.entries.len() {
            return Ok(());
        }
        self.publish(&entries).await?;
        self.state.lock().await.entries = entries;
        Ok(())
    }

    /// Generate a safe, non-secret stable identity.
    pub fn new_identity() -> String {
        let mut bytes = [0u8; 24];
        rand::rng().fill(&mut bytes);
        let mut identity = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            use std::fmt::Write;
            let _ = write!(identity, "{byte:02x}");
        }
        identity
    }

    async fn validate_root(&self) -> Result<(), String> {
        if tokio::fs::symlink_metadata(&self.root)
            .await
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err("server destination store directory is a symlink".to_string());
        }
        if let Some(parent) = self.root.parent() {
            if tokio::fs::symlink_metadata(parent)
                .await
                .is_ok_and(|m| m.file_type().is_symlink())
            {
                return Err("server destination store parent is a symlink".to_string());
            }
        }
        tokio::fs::create_dir_all(&self.root)
            .await
            .map_err(|_| "failed to create server destination store".to_string())?;
        let metadata = tokio::fs::metadata(&self.root)
            .await
            .map_err(|_| "failed to inspect server destination store".to_string())?;
        if !metadata.is_dir() {
            return Err("server destination store is not a directory".to_string());
        }
        Ok(())
    }

    async fn read_file(
        &self,
        name: &str,
    ) -> Result<Option<Result<BTreeMap<String, String>, String>>, String> {
        let path = self.root.join(name);
        let metadata = match tokio::fs::symlink_metadata(&path).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => {
                return Ok(Some(Err(
                    "failed to inspect server destination state".into()
                )))
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Ok(Some(Err(
                "server destination state file is not regular".into()
            )));
        }
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|_| "failed to read server destination state".to_string())?;
        if bytes.len() > MAX_STORE_SIZE {
            return Ok(Some(Err("server destination state is oversized".into())));
        }
        let envelope: SecretEnvelope = match serde_json::from_slice(&bytes) {
            Ok(envelope) => envelope,
            Err(_) => return Ok(Some(Err("server destination state is corrupt".into()))),
        };
        if envelope.version != 1 {
            return Ok(Some(Err(
                "unsupported server destination state version".into()
            )));
        }
        match validate_entries(&envelope.entries) {
            Ok(()) => Ok(Some(Ok(envelope.entries))),
            Err(error) => Ok(Some(Err(error))),
        }
    }

    async fn publish(&self, entries: &BTreeMap<String, String>) -> Result<(), String> {
        validate_entries(entries)?;
        self.validate_root().await?;
        let envelope = SecretEnvelope {
            version: 1,
            entries: entries.clone(),
        };
        let bytes = serde_json::to_vec(&envelope)
            .map_err(|_| "failed to serialize server destination state".to_string())?;
        if bytes.len() > MAX_STORE_SIZE {
            return Err("server destination state is oversized".to_string());
        }

        publish_with_backup(
            &self.root,
            CURRENT_FILE,
            BACKUP_FILE,
            ".tmp-current.json",
            &bytes,
            MAX_STORE_SIZE,
        )
        .await
    }
}

fn validate_identity(identity: &str) -> Result<(), String> {
    if identity.is_empty()
        || identity.len() > MAX_ID_LENGTH
        || identity.contains('/')
        || identity.contains('\\')
        || identity.chars().any(|character| character.is_control())
    {
        return Err("invalid server destination identity".to_string());
    }
    Ok(())
}

fn validate_destination(destination: &str) -> Result<(), String> {
    if destination.is_empty() || destination.len() > MAX_SECRET_LENGTH {
        return Err("invalid server destination".to_string());
    }
    let decoded =
        base64_decode(destination).ok_or_else(|| "invalid server destination".to_string())?;
    if decoded.is_empty() {
        return Err("invalid server destination".to_string());
    }
    Ok(())
}

fn validate_entries(entries: &BTreeMap<String, String>) -> Result<(), String> {
    if entries.len() > MAX_ENTRIES {
        return Err("server destination store capacity exceeded".to_string());
    }
    for (identity, destination) in entries {
        validate_identity(identity)?;
        validate_destination(destination)?;
    }
    Ok(())
}

fn validate_reference(reference: &str) -> Result<(), String> {
    if reference.is_empty()
        || reference.len() > MAX_REFERENCE_LENGTH
        || reference.starts_with('/')
        || reference.contains('\\')
        || reference.chars().any(char::is_control)
    {
        return Err("PrivKeyFile must be a safe relative import reference".to_string());
    }
    if Path::new(reference).components().any(|component| {
        matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
    }) {
        return Err("PrivKeyFile must not escape the server import root".to_string());
    }
    Ok(())
}

async fn reject_symlink_components(root: &Path, path: &Path) -> Result<(), String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "server private-key import escapes its root".to_string())?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err("server private-key import path is invalid".to_string());
        };
        current.push(component);
        if tokio::fs::symlink_metadata(&current)
            .await
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err("server private-key import contains a symlink".to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use emissary_core::crypto::base64_encode;

    fn secret() -> StoredDestination {
        StoredDestination(base64_encode([7u8; 128]))
    }

    #[tokio::test]
    async fn current_corruption_recovers_valid_backup() {
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        store.load().await.unwrap();
        let identity = ServerDestinationStore::new_identity();
        store.put(&identity, secret()).await.unwrap();
        store.put(&identity, secret()).await.unwrap();
        tokio::fs::write(store.directory().join(CURRENT_FILE), b"broken").await.unwrap();

        let restarted = ServerDestinationStore::new(root.path());
        restarted.load().await.unwrap();
        assert!(restarted.get(&identity).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn identity_and_secret_are_not_debuggable() {
        let secret = secret();
        assert!(!format!("{secret:?}").contains(secret.as_str()));
        assert!(!format!("{secret}").contains(secret.as_str()));
    }

    #[tokio::test]
    async fn symlink_state_file_is_rejected() {
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        store.load().await.unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/etc/passwd", store.directory().join(CURRENT_FILE))
                .unwrap();
            let restarted = ServerDestinationStore::new(root.path());
            assert!(restarted.load().await.is_err());
        }
    }

    #[tokio::test]
    async fn stale_temp_does_not_override_current() {
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        store.load().await.unwrap();
        let identity = ServerDestinationStore::new_identity();
        store.put(&identity, secret()).await.unwrap();
        tokio::fs::write(store.directory().join(".tmp-current.json"), b"broken")
            .await
            .unwrap();

        let restarted = ServerDestinationStore::new(root.path());
        restarted.load().await.unwrap();
        assert!(restarted.get(&identity).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn concurrent_publications_retain_complete_generations() {
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        store.load().await.unwrap();
        let first = ServerDestinationStore::new_identity();
        let second = ServerDestinationStore::new_identity();
        let (left, right) = tokio::join!(store.put(&first, secret()), store.put(&second, secret()));
        left.unwrap();
        right.unwrap();

        assert!(store.get(&first).await.unwrap().is_some());
        assert!(store.get(&second).await.unwrap().is_some());
        let restarted = ServerDestinationStore::new(root.path());
        restarted.load().await.unwrap();
        assert!(restarted.get(&first).await.unwrap().is_some());
        assert!(restarted.get(&second).await.unwrap().is_some());
    }
}
