//! Confined, I2PControl-owned client destination identity storage.
//!
//! Client identities are deliberately kept out of `TunnelDefinition`.  The
//! definition stores only the contract option and (for imports) the safe
//! administrative reference; this store owns the private destination bundle
//! consumed by Yosemite.

use std::{
    collections::BTreeMap,
    fmt,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use emissary_core::crypto::base64_decode;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::i2pcontrol::{
    domain::tunnel::TunnelDefinition,
    idle_resume::{is_shared_synthetic_name, shared_synthetic_name},
    stores::publication::publish_with_backup,
};

const STORE_DIRECTORY: &str = "client-destinations";
const IMPORT_DIRECTORY: &str = "client-key-imports";
const CURRENT_FILE: &str = "current.json";
const BACKUP_FILE: &str = "backup.json";
const TEMP_FILE: &str = ".tmp-current.json";
const MAX_STORE_SIZE: usize = 1024 * 1024;
const MAX_SECRET_SIZE: usize = 64 * 1024;
const MAX_REFERENCE_LENGTH: usize = 256;
const MAX_ENTRIES: usize = 1000;

#[derive(Clone, Serialize, Deserialize)]
struct StoredEntry {
    private_key: String,
    import_reference: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Envelope {
    version: u32,
    entries: BTreeMap<String, StoredEntry>,
}

#[derive(Clone)]
struct PendingEntry {
    private_key: Option<String>,
    import_reference: Option<String>,
    persist: bool,
}

#[derive(Default)]
struct State {
    entries: BTreeMap<String, StoredEntry>,
    pending: BTreeMap<String, PendingEntry>,
}

/// A client private destination bundle that is safe to pass to a runtime
/// owner but cannot be printed accidentally.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct StoredClientDestination(String);

impl StoredClientDestination {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for StoredClientDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StoredClientDestination(***)")
    }
}

impl fmt::Display for StoredClientDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

/// Persistent owner for client destination/key bundles.
#[derive(Clone)]
pub(crate) struct ClientDestinationStore {
    root: PathBuf,
    state: Arc<Mutex<State>>,
    mutation: Arc<Mutex<()>>,
}

impl fmt::Debug for ClientDestinationStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.state.try_lock().map(|state| state.entries.len()).unwrap_or(0);
        f.debug_struct("ClientDestinationStore")
            .field("root", &self.root)
            .field("entry_count", &count)
            .finish()
    }
}

impl ClientDestinationStore {
    pub(crate) fn new(state_root: impl Into<PathBuf>) -> Self {
        let state_root = state_root.into();
        Self {
            root: state_root.join(STORE_DIRECTORY),
            state: Arc::new(Mutex::new(State::default())),
            mutation: Arc::new(Mutex::new(())),
        }
    }

    pub(crate) async fn load(&self) -> Result<(), String> {
        let _guard = self.mutation.lock().await;
        self.ensure_root().await?;
        let current = self.read_state(CURRENT_FILE).await?;
        let backup = self.read_state(BACKUP_FILE).await?;
        let entries = match (current, backup) {
            (Some(Ok(entries)), _) => entries,
            (_, Some(Ok(entries))) => entries,
            (Some(Err(error)), Some(Err(_))) | (Some(Err(error)), None) => return Err(error),
            (None, Some(Err(error))) => return Err(error),
            (None, None) => BTreeMap::new(),
        };
        self.state.lock().await.entries = entries;
        Ok(())
    }

    /// Prepare the identity for a generation.  This performs generation or
    /// import, but does not publish a replacement identity until `commit`.
    #[allow(dead_code)]
    pub(crate) async fn stage(
        &self,
        definition: &TunnelDefinition,
        sam_tcp_port: u16,
    ) -> Result<(), String> {
        self.stage_with_resume(definition, sam_tcp_port, false).await
    }

    /// Prepare the identity for a generation with an explicit proven
    /// idle-resume decision (M134).
    ///
    /// `is_qualifying_resume` must be true only when the immediately preceding
    /// owning generation was authoritatively closed by the configured
    /// idle-close policy (`SamTerminationReason::IdlePolicy` via the
    /// I2PControl-owned `IdleResumeTracker`). A fresh successor is staged only
    /// in that case; all other starts reuse the committed identity (or stage
    /// the initial identity when none exists). Failed or cancelled starts must
    /// call `discard` so the staged successor never becomes authoritative and
    /// eligibility stays retryable.
    pub(crate) async fn stage_with_resume(
        &self,
        definition: &TunnelDefinition,
        sam_tcp_port: u16,
        is_qualifying_resume: bool,
    ) -> Result<(), String> {
        if !definition.tunnel_type.is_client() {
            return Ok(());
        }
        let options = &definition.options;
        let name = definition.name.as_str().to_owned();
        // Synthetic shared entries are owned by the shared resume path below,
        // never by per-name staging. A user tunnel can never carry a synthetic
        // name (rejected at the administrative boundary).
        if is_shared_synthetic_name(&name) {
            return Err("client destination name is reserved".to_string());
        }
        let persistent = options.persistent_client_key.unwrap_or(false);
        let new_dest = options.new_dest.unwrap_or(false);
        let import_reference = options.priv_key_file.as_deref();
        let existing = self.state.lock().await.entries.get(&name).cloned();

        let (private_key, imported_reference) = if let Some(reference) = import_reference {
            validate_reference(reference)?;
            if existing.as_ref().and_then(|entry| entry.import_reference.as_deref())
                == Some(reference)
            {
                let entry = existing.as_ref().expect("reference comparison implies entry");
                (entry.private_key.clone(), Some(reference.to_owned()))
            } else {
                (
                    self.import_reference(reference).await?,
                    Some(reference.to_owned()),
                )
            }
        } else if new_dest {
            // M134: `NewDest=true` behaves like a stable stored identity
            // except that a qualifying idle resume replaces the committed
            // successor exactly once. Without a qualifying fact the committed
            // identity is reused (or initially generated when none exists);
            // no rotation occurs on manual Stop/Start, Restart, failure or
            // unrelated edits.
            if is_qualifying_resume {
                generate_private_key(sam_tcp_port, options.sig_type.as_deref()).await?
            } else {
                match existing {
                    Some(entry) => (entry.private_key, entry.import_reference),
                    None => generate_private_key(sam_tcp_port, options.sig_type.as_deref()).await?,
                }
            }
        } else if persistent {
            match existing {
                Some(entry) => (entry.private_key, entry.import_reference),
                None => generate_private_key(sam_tcp_port, options.sig_type.as_deref()).await?,
            }
        } else {
            let mut state = self.state.lock().await;
            state.pending.insert(
                name,
                PendingEntry {
                    private_key: None,
                    import_reference: None,
                    persist: false,
                },
            );
            return Ok(());
        };

        validate_private_key(&private_key)?;
        // M134: `NewDest=true` successors are durably committed (like
        // persistent) so ordinary starts reuse them without rotation; only a
        // qualifying resume replaces them.
        let persist = persistent || import_reference.is_some() || new_dest;
        self.state.lock().await.pending.insert(
            name,
            PendingEntry {
                private_key: Some(private_key),
                import_reference: imported_reference,
                persist,
            },
        );
        Ok(())
    }

    /// Stage a shared successor for one stable shared policy key (M134).
    ///
    /// Shared identities live under synthetic names in the same atomic
    /// envelope so one qualifying idle close yields one shared successor, not
    /// one per member. `is_qualifying_resume` follows the same proven-resume
    /// rule as the dedicated path. All shared stages persist so ordinary
    /// member starts reuse the current shared identity without rotation.
    pub(crate) async fn shared_stage(
        &self,
        policy_key: &str,
        sam_tcp_port: u16,
        signature_type: Option<&str>,
        is_qualifying_resume: bool,
    ) -> Result<(), String> {
        let name = shared_synthetic_name(policy_key);
        let existing = self.state.lock().await.entries.get(&name).cloned();
        let (private_key, imported_reference) = if is_qualifying_resume {
            generate_private_key(sam_tcp_port, signature_type).await?
        } else {
            match existing {
                Some(entry) => (entry.private_key, entry.import_reference),
                None => generate_private_key(sam_tcp_port, signature_type).await?,
            }
        };
        validate_private_key(&private_key)?;
        self.state.lock().await.pending.insert(
            name,
            PendingEntry {
                private_key: Some(private_key),
                import_reference: imported_reference,
                persist: true,
            },
        );
        Ok(())
    }

    /// Return the staged-or-committed shared identity for a policy key.
    pub(crate) async fn shared_active(
        &self,
        policy_key: &str,
    ) -> Result<Option<StoredClientDestination>, String> {
        self.active(&shared_synthetic_name(policy_key)).await
    }

    /// Commit a staged shared successor after the shared session is
    /// irreversibly accepted. A failed start must call `shared_discard`.
    pub(crate) async fn shared_commit(&self, policy_key: &str) -> Result<(), String> {
        self.commit(&shared_synthetic_name(policy_key)).await
    }

    /// Discard a staged shared successor after a failed or cancelled resume.
    /// Eligibility stays retryable; no durable state changes.
    pub(crate) async fn shared_discard(&self, policy_key: &str) {
        self.discard(&shared_synthetic_name(policy_key)).await;
    }

    /// Remove a shared synthetic entry once no member references its policy.
    pub(crate) async fn shared_remove(&self, policy_key: &str) -> Result<bool, String> {
        self.remove(&shared_synthetic_name(policy_key)).await
    }

    /// Return the staged identity, or the committed identity when no stage is
    /// active.  Private material never crosses the public domain boundary.
    pub(crate) async fn active(
        &self,
        name: &str,
    ) -> Result<Option<StoredClientDestination>, String> {
        let state = self.state.lock().await;
        if let Some(pending) = state.pending.get(name) {
            return Ok(pending.private_key.clone().map(StoredClientDestination));
        }
        Ok(state
            .entries
            .get(name)
            .map(|entry| StoredClientDestination(entry.private_key.clone())))
    }

    /// Commit the staged identity after the runtime has reached its promised
    /// active point.  A failed start calls `discard` instead.
    pub(crate) async fn commit(&self, name: &str) -> Result<(), String> {
        let _guard = self.mutation.lock().await;
        let pending = self.state.lock().await.pending.remove(name);
        let Some(pending) = pending else {
            return Ok(());
        };
        let mut entries = self.state.lock().await.entries.clone();
        if pending.persist {
            let private_key = pending
                .private_key
                .ok_or_else(|| "client identity transaction is incomplete".to_string())?;
            if !entries.contains_key(name) && entries.len() >= MAX_ENTRIES {
                return Err("client destination store capacity exhausted".to_string());
            }
            entries.insert(
                name.to_owned(),
                StoredEntry {
                    private_key,
                    import_reference: pending.import_reference,
                },
            );
        } else {
            entries.remove(name);
        }
        self.publish(&entries).await?;
        self.state.lock().await.entries = entries;
        Ok(())
    }

    pub(crate) async fn discard(&self, name: &str) {
        self.state.lock().await.pending.remove(name);
    }

    pub(crate) async fn remove(&self, name: &str) -> Result<bool, String> {
        let _guard = self.mutation.lock().await;
        let mut entries = self.state.lock().await.entries.clone();
        let removed = entries.remove(name).is_some();
        self.state.lock().await.pending.remove(name);
        if removed {
            self.publish(&entries).await?;
            self.state.lock().await.entries = entries;
        }
        Ok(removed)
    }

    pub(crate) async fn rename(&self, old_name: &str, new_name: &str) -> Result<(), String> {
        if old_name == new_name {
            return Ok(());
        }
        let _guard = self.mutation.lock().await;
        let mut entries = self.state.lock().await.entries.clone();
        let Some(entry) = entries.remove(old_name) else {
            return Ok(());
        };
        if entries.contains_key(new_name) {
            return Err("client destination identity name already exists".to_string());
        }
        entries.insert(new_name.to_owned(), entry);
        self.publish(&entries).await?;
        let mut state = self.state.lock().await;
        state.entries = entries;
        if let Some(pending) = state.pending.remove(old_name) {
            state.pending.insert(new_name.to_owned(), pending);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) async fn prune_unreferenced(
        &self,
        referenced: &std::collections::BTreeSet<String>,
    ) -> Result<(), String> {
        self.prune_unreferenced_with_shared(referenced, &std::collections::BTreeSet::new())
            .await
    }

    /// Prune durable entries that are referenced by neither a per-name tunnel
    /// nor a live shared policy. Synthetic shared entries are retained only
    /// while their policy remains referenced.
    pub(crate) async fn prune_unreferenced_with_shared(
        &self,
        referenced: &std::collections::BTreeSet<String>,
        referenced_shared: &std::collections::BTreeSet<String>,
    ) -> Result<(), String> {
        let _guard = self.mutation.lock().await;
        let entries = self
            .state
            .lock()
            .await
            .entries
            .iter()
            .filter(|(name, _)| {
                if is_shared_synthetic_name(name) {
                    referenced_shared.contains(*name)
                } else {
                    referenced.contains(*name)
                }
            })
            .map(|(name, entry)| (name.clone(), entry.clone()))
            .collect::<BTreeMap<_, _>>();
        let changed = entries.len() != self.state.lock().await.entries.len();
        if changed {
            self.publish(&entries).await?;
            self.state.lock().await.entries = entries;
        }
        Ok(())
    }

    async fn import_reference(&self, reference: &str) -> Result<String, String> {
        validate_reference(reference)?;
        let import_root = self
            .root
            .parent()
            .ok_or_else(|| "client destination store has no administrative root".to_string())?
            .join(IMPORT_DIRECTORY);
        ensure_directory(&import_root).await?;
        let path = import_root.join(reference);
        reject_symlink_components(&import_root, &path).await?;
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|_| "client private-key import is unavailable".to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("client private-key import is not a regular file".to_string());
        }
        if metadata.len() as usize > MAX_SECRET_SIZE {
            return Err("client private-key import is oversized".to_string());
        }
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|_| "client private-key import could not be read".to_string())?;
        let value = String::from_utf8(bytes)
            .map_err(|_| "client private-key import is not valid text".to_string())?;
        let value = value.trim().to_owned();
        validate_private_key(&value)?;
        Ok(value)
    }

    async fn ensure_root(&self) -> Result<(), String> {
        ensure_directory(&self.root).await
    }

    async fn read_state(
        &self,
        file_name: &str,
    ) -> Result<Option<Result<BTreeMap<String, StoredEntry>, String>>, String> {
        let path = self.root.join(file_name);
        let metadata = match tokio::fs::symlink_metadata(&path).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => {
                return Ok(Some(Err(
                    "client destination state is unavailable".to_string()
                )))
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Ok(Some(Err(
                "client destination state is not regular".to_string()
            )));
        }
        if metadata.len() as usize > MAX_STORE_SIZE {
            return Ok(Some(Err(
                "client destination state is oversized".to_string()
            )));
        }
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|_| "client destination state could not be read".to_string())?;
        let envelope: Envelope = serde_json::from_slice(&bytes)
            .map_err(|_| "client destination state is corrupt".to_string())?;
        if envelope.version != 1 || envelope.entries.len() > MAX_ENTRIES {
            return Ok(Some(
                Err("unsupported client destination state".to_string()),
            ));
        }
        for entry in envelope.entries.values() {
            validate_private_key(&entry.private_key)?;
            if let Some(reference) = &entry.import_reference {
                validate_reference(reference)?;
            }
        }
        Ok(Some(Ok(envelope.entries)))
    }

    async fn publish(&self, entries: &BTreeMap<String, StoredEntry>) -> Result<(), String> {
        if entries.len() > MAX_ENTRIES {
            return Err("client destination store capacity exhausted".to_string());
        }
        for entry in entries.values() {
            validate_private_key(&entry.private_key)?;
        }
        self.ensure_root().await?;
        let bytes = serde_json::to_vec(&Envelope {
            version: 1,
            entries: entries.clone(),
        })
        .map_err(|_| "client destination state could not be serialized".to_string())?;
        publish_with_backup(
            &self.root,
            CURRENT_FILE,
            BACKUP_FILE,
            TEMP_FILE,
            &bytes,
            MAX_STORE_SIZE,
        )
        .await
    }
}

async fn generate_private_key(
    sam_tcp_port: u16,
    signature_type: Option<&str>,
) -> Result<(String, Option<String>), String> {
    let router = yosemite_i2pcontrol::RouterApi::new(sam_tcp_port);
    let generated = match signature_type {
        Some(value) => {
            router
                .generate_destination_with_signature_type(parse_signature_type(value)?)
                .await
        }
        None => router.generate_destination().await,
    };
    let (_, private_key) =
        generated.map_err(|_| "client destination generation failed".to_string())?;
    Ok((private_key, None))
}

fn parse_signature_type(value: &str) -> Result<u16, String> {
    match value.parse::<u16>() {
        Ok(7) if value == "7" => Ok(7),
        Ok(_) => Err("SigType is unsupported by the Emissary SAM router".to_owned()),
        Err(_) => Err("SigType must be the supported unsigned value 7".to_owned()),
    }
}

fn validate_private_key(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > MAX_SECRET_SIZE || value.chars().any(char::is_control) {
        return Err("client private destination is invalid".to_string());
    }
    let decoded =
        base64_decode(value).ok_or_else(|| "client private destination is invalid".to_string())?;
    if decoded.is_empty() {
        return Err("client private destination is invalid".to_string());
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
    let path = Path::new(reference);
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err("PrivKeyFile must not escape the client import root".to_string());
    }
    Ok(())
}

async fn reject_symlink_components(root: &Path, path: &Path) -> Result<(), String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "client private-key import escapes its root".to_string())?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err("client private-key import path is invalid".to_string());
        };
        current.push(component);
        if let Ok(metadata) = tokio::fs::symlink_metadata(&current).await {
            if metadata.file_type().is_symlink() {
                return Err("client private-key import contains a symlink".to_string());
            }
        }
    }
    Ok(())
}

async fn ensure_directory(path: &Path) -> Result<(), String> {
    if tokio::fs::symlink_metadata(path)
        .await
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err("client secret directory is a symlink".to_string());
    }
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|_| "client secret directory could not be created".to_string())?;
    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|_| "client secret directory could not be inspected".to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("client secret path is not a directory".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2pcontrol::domain::tunnel::{
        StartIntent, TunnelName, TunnelOptions, TunnelOwnership, TunnelRuntimeState, TunnelType,
    };
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        sync::oneshot,
    };

    fn definition(name: &str) -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new(name).unwrap(),
            tunnel_type: TunnelType::Client,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions::default(),
            raw_config: BTreeMap::new(),
        }
    }

    #[tokio::test]
    async fn imported_identity_is_confined_atomic_and_restart_safe() {
        let directory = tempfile::tempdir().unwrap();
        let store = ClientDestinationStore::new(directory.path());
        let import_root = directory.path().join(IMPORT_DIRECTORY);
        tokio::fs::create_dir_all(&import_root).await.unwrap();
        tokio::fs::write(import_root.join("alice.key"), b"aGVsbG8=").await.unwrap();

        let mut definition = definition("alice");
        definition.options.priv_key_file = Some("alice.key".to_owned());
        store.stage(&definition, 7656).await.unwrap();
        let active = store.active("alice").await.unwrap().unwrap();
        assert_eq!(active.as_str(), "aGVsbG8=");
        assert_eq!(format!("{}", active), "***");
        store.commit("alice").await.unwrap();

        let reloaded = ClientDestinationStore::new(directory.path());
        reloaded.load().await.unwrap();
        assert_eq!(
            reloaded.active("alice").await.unwrap().unwrap().as_str(),
            "aGVsbG8="
        );
        let metadata =
            tokio::fs::metadata(directory.path().join(STORE_DIRECTORY).join(CURRENT_FILE))
                .await
                .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        }
    }

    #[tokio::test]
    async fn import_rejects_escape_and_unsafe_file_types() {
        let directory = tempfile::tempdir().unwrap();
        let store = ClientDestinationStore::new(directory.path());
        let mut definition = definition("unsafe");
        for reference in ["../outside.key", "/tmp/key", "nested\\key"] {
            definition.options.priv_key_file = Some(reference.to_owned());
            assert!(store.stage(&definition, 7656).await.is_err());
        }
        let import_root = directory.path().join(IMPORT_DIRECTORY);
        tokio::fs::create_dir_all(&import_root).await.unwrap();
        tokio::fs::create_dir(import_root.join("directory.key")).await.unwrap();
        definition.options.priv_key_file = Some("directory.key".to_owned());
        assert!(store.stage(&definition, 7656).await.is_err());
    }

    #[tokio::test]
    async fn generated_identity_uses_selected_signature_type_without_fallback() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let (command_tx, command_rx) = oneshot::channel();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            assert_eq!(line, "HELLO VERSION\n");
            write_half.write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n").await.unwrap();
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            command_tx.send(line).unwrap();
            write_half
                .write_all(b"DEST REPLY PUB=destination PRIV=cHJpdmF0ZQ==\n")
                .await
                .unwrap();
        });

        let directory = tempfile::tempdir().unwrap();
        let store = ClientDestinationStore::new(directory.path());
        let mut definition = definition("selected-signature");
        definition.options.persistent_client_key = Some(true);
        definition.options.sig_type = Some("7".to_owned());
        store.stage(&definition, port).await.unwrap();

        assert_eq!(
            command_rx.await.unwrap(),
            "DEST GENERATE SIGNATURE_TYPE=7\n"
        );
        assert_eq!(
            store.active("selected-signature").await.unwrap().unwrap().as_str(),
            "cHJpdmF0ZQ=="
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn generated_identity_rejects_invalid_signature_without_defaulting() {
        let directory = tempfile::tempdir().unwrap();
        let store = ClientDestinationStore::new(directory.path());
        let mut definition = definition("invalid-signature");
        definition.options.persistent_client_key = Some(true);
        definition.options.sig_type = Some("not-a-number".to_owned());

        assert!(store.stage(&definition, 7656).await.is_err());
    }

    /// Fake SAM that serves `DEST GENERATE` with a programmed sequence of
    /// distinct private keys and counts generations. Each `DEST GENERATE`
    /// consumes the next key; `SESSION CREATE` is rejected (secret tests never
    /// construct Yosemite sessions).
    async fn counting_dest_generate(
        keys: Vec<&'static str>,
    ) -> (
        u16,
        tokio::task::JoinHandle<()>,
        Arc<std::sync::atomic::AtomicUsize>,
    ) {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let count = Arc::new(AtomicUsize::new(0));
        let count_for_task = Arc::clone(&count);
        let task = tokio::spawn(async move {
            // Serve one connection per generation (Yosemite opens a fresh SAM
            // connection per `generate_destination` call).
            for key in keys {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                count_for_task.fetch_add(1, Ordering::AcqRel);
                let (read_half, mut write_half) = stream.into_split();
                let mut reader = BufReader::new(read_half);
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
                assert_eq!(line, "HELLO VERSION\n");
                write_half.write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n").await.unwrap();
                line.clear();
                reader.read_line(&mut line).await.unwrap();
                assert!(line.starts_with("DEST GENERATE"));
                write_half
                    .write_all(format!("DEST REPLY PUB=destination PRIV={key}\n").as_bytes())
                    .await
                    .unwrap();
            }
        });
        (port, task, count)
    }

    #[tokio::test]
    async fn m134_newdest_reuses_without_qualifying_resume() {
        use std::sync::atomic::Ordering;
        let (port, task, count) = counting_dest_generate(vec!["aGVsbG8=", "d29ybGQ="]).await;
        let directory = tempfile::tempdir().unwrap();
        let store = ClientDestinationStore::new(directory.path());
        let mut definition = definition("newdest-reuse");
        definition.options.new_dest = Some(true);

        // Initial generation (no qualifying fact, no committed): generates once.
        store.stage_with_resume(&definition, port, false).await.unwrap();
        let initial = store.active("newdest-reuse").await.unwrap().unwrap();
        assert_eq!(initial.as_str(), "aGVsbG8=");
        store.commit("newdest-reuse").await.unwrap();
        assert_eq!(count.load(Ordering::Acquire), 1);

        // Ordinary start without a qualifying fact reuses: no new generation.
        store.stage_with_resume(&definition, port, false).await.unwrap();
        let reused = store.active("newdest-reuse").await.unwrap().unwrap();
        assert_eq!(reused.as_str(), "aGVsbG8=");
        store.commit("newdest-reuse").await.unwrap();
        assert_eq!(count.load(Ordering::Acquire), 1);
        task.abort();
    }

    #[tokio::test]
    async fn m134_qualifying_resume_rotates_once_then_stabilizes() {
        use std::sync::atomic::Ordering;
        let (port, task, count) =
            counting_dest_generate(vec!["aGVsbG8=", "d29ybGQ=", "c2VjcmV0"]).await;
        let directory = tempfile::tempdir().unwrap();
        let store = ClientDestinationStore::new(directory.path());
        let mut definition = definition("newdest-rotate");
        definition.options.new_dest = Some(true);

        store.stage_with_resume(&definition, port, false).await.unwrap();
        store.commit("newdest-rotate").await.unwrap();
        assert_eq!(count.load(Ordering::Acquire), 1);

        // Qualifying resume stages exactly one fresh successor.
        store.stage_with_resume(&definition, port, true).await.unwrap();
        let successor = store.active("newdest-rotate").await.unwrap().unwrap();
        assert_eq!(successor.as_str(), "d29ybGQ=");
        assert_eq!(count.load(Ordering::Acquire), 2);
        store.commit("newdest-rotate").await.unwrap();

        // Second ordinary start after the resume does not rotate again.
        store.stage_with_resume(&definition, port, false).await.unwrap();
        let stable = store.active("newdest-rotate").await.unwrap().unwrap();
        assert_eq!(stable.as_str(), "d29ybGQ=");
        store.commit("newdest-rotate").await.unwrap();
        assert_eq!(count.load(Ordering::Acquire), 2);
        task.abort();
    }

    #[tokio::test]
    async fn m134_failed_resume_discards_staged_successor() {
        let (port, task, _) =
            counting_dest_generate(vec!["aGVsbG8=", "d29ybGQ=", "c2VjcmV0"]).await;
        let directory = tempfile::tempdir().unwrap();
        let store = ClientDestinationStore::new(directory.path());
        let mut definition = definition("newdest-failure");
        definition.options.new_dest = Some(true);

        store.stage_with_resume(&definition, port, false).await.unwrap();
        store.commit("newdest-failure").await.unwrap();

        // Qualifying stage succeeds, then the start fails before commit:
        // discard must restore the committed identity with no leak.
        store.stage_with_resume(&definition, port, true).await.unwrap();
        assert_eq!(
            store.active("newdest-failure").await.unwrap().unwrap().as_str(),
            "d29ybGQ="
        );
        store.discard("newdest-failure").await;
        assert_eq!(
            store.active("newdest-failure").await.unwrap().unwrap().as_str(),
            "aGVsbG8="
        );
        // Retry of the same logical resume may stage again (eligibility is
        // tracker-owned and stays retryable; the store itself never consumes).
        store.stage_with_resume(&definition, port, true).await.unwrap();
        store.discard("newdest-failure").await;
        task.abort();
    }

    #[tokio::test]
    async fn m134_shared_policy_yields_one_successor_for_all_members() {
        let (port, task, _) =
            counting_dest_generate(vec!["aGVsbG8=", "d29ybGQ=", "c2VjcmV0"]).await;
        let directory = tempfile::tempdir().unwrap();
        let store = ClientDestinationStore::new(directory.path());
        let policy = "shared-policy-m134";

        // Initial shared generation: one successor, visible to every member
        // through the same synthetic name.
        store.shared_stage(policy, port, None, false).await.unwrap();
        store.shared_commit(policy).await.unwrap();
        let first = store.shared_active(policy).await.unwrap().unwrap();
        assert_eq!(first.as_str(), "aGVsbG8=");

        // Qualifying shared resume: one fresh successor, not one per member.
        store.shared_stage(policy, port, None, true).await.unwrap();
        let successor = store.shared_active(policy).await.unwrap().unwrap();
        assert_eq!(successor.as_str(), "d29ybGQ=");
        store.shared_commit(policy).await.unwrap();
        assert_eq!(
            store.shared_active(policy).await.unwrap().unwrap().as_str(),
            "d29ybGQ="
        );

        // Failed shared resume discards without touching the committed.
        store.shared_stage(policy, port, None, true).await.unwrap();
        store.shared_discard(policy).await;
        assert_eq!(
            store.shared_active(policy).await.unwrap().unwrap().as_str(),
            "d29ybGQ="
        );
        task.abort();
    }

    #[tokio::test]
    async fn m134_no_secret_material_in_debug_or_display() {
        let directory = tempfile::tempdir().unwrap();
        let store = ClientDestinationStore::new(directory.path());
        let debug = format!("{store:?}");
        assert!(!debug.contains("aGVsbG8="));
        let (port, task, _) = counting_dest_generate(vec!["aGVsbG8="]).await;
        let mut definition = definition("redaction-check");
        definition.options.new_dest = Some(true);
        store.stage_with_resume(&definition, port, false).await.unwrap();
        let active = store.active("redaction-check").await.unwrap().unwrap();
        assert_eq!(format!("{active:?}"), "StoredClientDestination(***)");
        assert_eq!(format!("{active}"), "***");
        task.abort();
    }
}
