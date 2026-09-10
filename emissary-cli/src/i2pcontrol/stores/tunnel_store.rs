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

use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::generation_store::{GenerationStore, StoreResult};
use crate::i2pcontrol::domain::{revision::StateRevision, tunnel::TunnelDefinition};

const BLOCKED_LEASESET_RAW_KEYS: [&str; 3] =
    ["EncryptLeaseSet", "OptionalLookup", "LeaseSetClientAuths"];

const SCRUB_FALLBACK_GENERATIONS: usize = 2;

/// Internal migration state for removal of M162-era blocked LeaseSet fields.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TunnelStoreScrubState {
    #[default]
    LegacyUnstarted,
    SanitizedPendingHistoryPurge,
    ScrubComplete,
}

/// Persistent tunnel definitions envelope payload.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TunnelStorePayload {
    /// Tunnel definitions keyed by tunnel name.
    pub tunnels: BTreeMap<String, TunnelDefinition>,

    /// Crash-resumable state for the blocked LeaseSet history scrub.
    #[serde(default)]
    scrub_state: TunnelStoreScrubState,
}

impl TunnelStorePayload {
    /// Create an empty payload.
    pub fn empty() -> Self {
        Self {
            tunnels: BTreeMap::new(),
            scrub_state: TunnelStoreScrubState::ScrubComplete,
        }
    }
}

/// Persistent tunnel definition store.
///
/// Stores all Proposal 170 tunnel definitions using versioned generation
/// persistence. Thread-safe via async mutation serialization.
#[allow(dead_code)]
pub struct TunnelStore {
    inner: GenerationStore<TunnelStorePayload>,
}

#[allow(dead_code)]
impl TunnelStore {
    /// Create a new tunnel store.
    pub fn new(dir: PathBuf, max_size: usize) -> Self {
        Self {
            inner: GenerationStore::new(dir, max_size),
        }
    }

    /// Load existing state from disk.
    pub async fn load(&mut self) -> StoreResult<Option<StateRevision>> {
        let revision = self.inner.load().await?;
        let Some(revision) = revision else {
            return Ok(None);
        };

        let mut migrated = self.inner.current().cloned().expect("loaded payload");
        let mut changed = false;
        for definition in migrated.tunnels.values_mut() {
            changed |= migrate_typed_secret(
                &mut definition.raw_config,
                "ProxyPassword",
                &mut definition.options.proxy_password,
            );
            changed |= migrate_typed_secret(
                &mut definition.raw_config,
                "i2p.tunnel.proxyPassword",
                &mut definition.options.proxy_password,
            );
            changed |= migrate_typed_secret(
                &mut definition.raw_config,
                "i2p.tunnel.sslKey",
                &mut definition.options.ssl_key,
            );
            changed |= migrate_typed_secret(
                &mut definition.raw_config,
                "i2p.tunnel.ircPassword",
                &mut definition.options.irc_password,
            );
        }

        let scrub_required = migrated.scrub_state != TunnelStoreScrubState::ScrubComplete
            || migrated.tunnels.values().any(contains_blocked_leaseset_state);
        if scrub_required {
            for definition in migrated.tunnels.values_mut() {
                sanitize_blocked_leaseset_state(definition);
            }
            migrated.scrub_state = TunnelStoreScrubState::SanitizedPendingHistoryPurge;

            // Two complete pending generations must exist before any
            // contaminated history is removed, preserving clean fallback.
            self.inner.publish(migrated.clone(), |_| Ok(())).await?;
            self.inner.publish(migrated.clone(), |_| Ok(())).await?;
            self.inner.purge_history_keep_newest(SCRUB_FALLBACK_GENERATIONS).await?;

            migrated.scrub_state = TunnelStoreScrubState::ScrubComplete;
            return Ok(Some(self.inner.publish(migrated, |_| Ok(())).await?));
        }

        if changed {
            Ok(Some(self.inner.publish(migrated, |_| Ok(())).await?))
        } else {
            Ok(Some(revision))
        }
    }

    /// Return the current revision.
    pub fn revision(&self) -> StateRevision {
        self.inner.revision()
    }

    /// List all tunnel definitions.
    pub fn list(&self) -> Vec<&TunnelDefinition> {
        self.inner.current().map(|p| p.tunnels.values().collect()).unwrap_or_default()
    }

    /// Get a tunnel definition by name.
    pub fn get(&self, name: &str) -> Option<&TunnelDefinition> {
        self.inner.current().and_then(|p| p.tunnels.get(name))
    }

    /// Add or replace a tunnel definition.
    pub async fn upsert(&mut self, definition: TunnelDefinition) -> StoreResult<StateRevision> {
        validate_no_blocked_leaseset_state(&definition)?;
        let name = definition.name.as_str().to_string();
        let current = self.inner.current().cloned().unwrap_or_else(TunnelStorePayload::empty);
        let mut tunnels = current.tunnels;
        tunnels.insert(name, definition);
        let payload = TunnelStorePayload {
            tunnels,
            scrub_state: current.scrub_state,
        };
        self.inner.publish(payload, |_| Ok(())).await
    }

    /// Atomically update a definition, optionally changing its name.
    ///
    /// The complete before-state is cloned, checked, and replaced in one
    /// generation publication. A failed publication leaves both the map and
    /// the durable generation untouched.
    pub async fn update(
        &mut self,
        name: &str,
        mut definition: TunnelDefinition,
        new_name: Option<&str>,
    ) -> StoreResult<bool> {
        validate_no_blocked_leaseset_state(&definition)?;
        let current = match self.inner.current() {
            Some(payload) => payload.clone(),
            None => return Ok(false),
        };
        if !current.tunnels.contains_key(name) {
            return Ok(false);
        }

        let target_name = new_name.unwrap_or(name);
        if target_name != name && current.tunnels.contains_key(target_name) {
            return Err(super::generation_store::StoreError::InvalidState(format!(
                "tunnel name '{}' already exists",
                target_name
            )));
        }

        definition.name = crate::i2pcontrol::domain::tunnel::TunnelName::new(target_name)
            .map_err(|e| super::generation_store::StoreError::InvalidState(e.to_string()))?;

        let mut tunnels = current.tunnels;
        tunnels.remove(name);
        tunnels.insert(target_name.to_string(), definition);
        self.inner
            .publish(
                TunnelStorePayload {
                    tunnels,
                    scrub_state: current.scrub_state,
                },
                |_| Ok(()),
            )
            .await?;
        Ok(true)
    }

    /// Remove a tunnel definition by name.
    pub async fn remove(&mut self, name: &str) -> StoreResult<Option<StateRevision>> {
        let current = match self.inner.current() {
            Some(p) => p.clone(),
            None => return Ok(None),
        };
        let mut tunnels = current.tunnels;
        let removed = tunnels.remove(name);
        if removed.is_none() {
            return Ok(None);
        }
        let payload = TunnelStorePayload {
            tunnels,
            scrub_state: current.scrub_state,
        };
        let rev = self.inner.publish(payload, |_| Ok(())).await?;
        Ok(Some(rev))
    }

    /// Return the number of stored tunnel definitions.
    pub fn len(&self) -> usize {
        self.inner.current().map(|p| p.tunnels.len()).unwrap_or(0)
    }

    /// Return true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return true if a tunnel with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.inner.current().map(|p| p.tunnels.contains_key(name)).unwrap_or(false)
    }

    /// Inject a publication failure for the next mutation in unit tests.
    #[cfg(test)]
    pub fn fail_next_publication(&mut self) {
        self.inner.fail_next_publication();
    }

    /// Inject a permission-setting failure for the next mutation in unit tests.
    #[cfg(test)]
    pub fn fail_next_permission_change(&mut self) {
        self.inner.fail_next_permission_change();
    }

    /// Inject a history-purge deletion failure for migration tests.
    #[cfg(test)]
    pub fn fail_next_history_purge(&mut self) {
        self.inner.fail_next_history_purge();
    }

    /// Inject a history-purge directory-sync failure for migration tests.
    #[cfg(test)]
    pub fn fail_next_history_purge_directory_sync(&mut self) {
        self.inner.fail_next_history_purge_directory_sync();
    }
}

fn contains_blocked_leaseset_state(definition: &TunnelDefinition) -> bool {
    definition.options.encrypt_lease_set.is_some()
        || definition.options.optional_lookup.is_some()
        || !definition.options.lease_set_client_auths.is_empty()
        || BLOCKED_LEASESET_RAW_KEYS
            .iter()
            .any(|key| definition.raw_config.contains_key(*key))
}

fn validate_no_blocked_leaseset_state(definition: &TunnelDefinition) -> StoreResult<()> {
    if contains_blocked_leaseset_state(definition) {
        return Err(super::generation_store::StoreError::InvalidState(
            "blocked LeaseSet-security option is not supported".to_string(),
        ));
    }
    Ok(())
}

fn sanitize_blocked_leaseset_state(definition: &mut TunnelDefinition) -> bool {
    let had_blocked = contains_blocked_leaseset_state(definition);
    definition.options.encrypt_lease_set = None;
    definition.options.optional_lookup = crate::i2pcontrol::domain::tunnel::OptionRedacted::none();
    definition.options.lease_set_client_auths.clear();
    for key in BLOCKED_LEASESET_RAW_KEYS {
        definition.raw_config.remove(key);
    }
    had_blocked
}

fn migrate_typed_secret(
    raw_config: &mut BTreeMap<String, serde_json::Value>,
    key: &str,
    target: &mut crate::i2pcontrol::domain::tunnel::OptionRedacted,
) -> bool {
    let Some(value) = raw_config.remove(key) else {
        return false;
    };
    if target.is_none() {
        if let Some(secret) = value.as_str() {
            *target = crate::i2pcontrol::domain::tunnel::OptionRedacted::new(secret);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2pcontrol::domain::tunnel::{
        EncryptLeaseSetMode, LeaseSetClientAuthEntry, OptionRedacted, StartIntent, TunnelName,
        TunnelOptions, TunnelOwnership, TunnelRuntimeState, TunnelType,
    };

    fn test_dir() -> PathBuf {
        tempfile::tempdir().unwrap().keep()
    }

    fn test_definition(name: &str, tunnel_type: TunnelType) -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new(name).unwrap(),
            tunnel_type,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions::default(),
            raw_config: std::collections::BTreeMap::new(),
        }
    }

    fn contaminated_definition() -> TunnelDefinition {
        let mut definition = test_definition("legacy", TunnelType::Server);
        definition.start_intent = StartIntent::StartOnLoad;
        definition.options.description = Some("preserve me".to_string());
        definition.options.encrypt_lease_set = Some(EncryptLeaseSetMode::Blinded);
        definition.options.optional_lookup = OptionRedacted::new("lookup-secret");
        definition.options.lease_set_client_auths = vec![
            LeaseSetClientAuthEntry::new(
                "alice",
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            )
            .unwrap(),
        ];
        definition
            .raw_config
            .insert("SafeOption".to_string(), serde_json::json!("preserve"));
        definition
            .raw_config
            .insert("OptionalLookup".to_string(), serde_json::json!("lookup-secret"));
        definition
    }

    async fn write_generation(
        dir: &std::path::Path,
        revision: u64,
        payload: TunnelStorePayload,
        omit_scrub_state: bool,
    ) {
        let envelope = super::super::generation_store::Envelope::new(
            StateRevision::new(revision),
            payload,
        );
        let mut value = serde_json::to_value(envelope).unwrap();
        if omit_scrub_state {
            value["payload"].as_object_mut().unwrap().remove("scrub_state");
        }
        tokio::fs::write(
            dir.join(format!("gen-{revision:020}.json")),
            serde_json::to_vec(&value).unwrap(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn empty_store() {
        let dir = test_dir();
        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
        let loaded = store.load().await.unwrap();
        assert_eq!(loaded, None);
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(store.list().is_empty());
        assert!(store.get("test").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn upsert_and_get() {
        let dir = test_dir();
        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);

        let def = test_definition("my-tunnel", TunnelType::Socks);
        store.upsert(def.clone()).await.unwrap();

        let found = store.get("my-tunnel").unwrap();
        assert_eq!(found.name.as_str(), "my-tunnel");
        assert_eq!(found.tunnel_type, TunnelType::Socks);
        assert_eq!(store.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn remove_tunnel() {
        let dir = test_dir();
        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);

        store.upsert(test_definition("t1", TunnelType::Client)).await.unwrap();
        store.upsert(test_definition("t2", TunnelType::Server)).await.unwrap();
        assert_eq!(store.len(), 2);

        let removed = store.remove("t1").await.unwrap();
        assert!(removed.is_some());
        assert_eq!(store.len(), 1);
        assert!(store.get("t1").is_none());
        assert!(store.get("t2").is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn remove_nonexistent_returns_none() {
        let dir = test_dir();
        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
        let removed = store.remove("nonexistent").await.unwrap();
        assert!(removed.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn rename_is_one_generation_and_failure_atomic() {
        let dir = test_dir();
        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
        store.upsert(test_definition("before", TunnelType::Client)).await.unwrap();
        let first_revision = store.revision();

        store.fail_next_publication();
        let result = store
            .update(
                "before",
                test_definition("after", TunnelType::Client),
                Some("after"),
            )
            .await;
        assert!(result.is_err());
        assert_eq!(store.revision(), first_revision);
        assert!(store.get("before").is_some());
        assert!(store.get("after").is_none());

        assert!(store
            .update(
                "before",
                test_definition("after", TunnelType::Client),
                Some("after"),
            )
            .await
            .unwrap());
        assert_eq!(
            store.revision(),
            StateRevision::new(first_revision.value() + 1)
        );
        assert!(store.get("before").is_none());
        assert!(store.get("after").is_some());

        let mut restarted = TunnelStore::new(dir.clone(), 1024 * 1024);
        restarted.load().await.unwrap();
        assert!(restarted.get("before").is_none());
        assert!(restarted.get("after").is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn permission_failure_does_not_publish() {
        let dir = test_dir();
        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
        store.upsert(test_definition("before", TunnelType::Client)).await.unwrap();
        let revision = store.revision();
        store.fail_next_permission_change();
        assert!(store.upsert(test_definition("after", TunnelType::Client)).await.is_err());
        assert_eq!(store.revision(), revision);
        assert!(store.get("before").is_some());
        assert!(store.get("after").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn round_trip_persistence() {
        let dir = test_dir();
        {
            let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
            store.upsert(test_definition("t1", TunnelType::HttpClient)).await.unwrap();
            store.upsert(test_definition("t2", TunnelType::IrcServer)).await.unwrap();
        }

        // Reload from disk
        {
            let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
            let loaded = store.load().await.unwrap();
            assert!(loaded.is_some());
            assert_eq!(store.len(), 2);
            assert!(store.get("t1").is_some());
            assert!(store.get("t2").is_some());
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn load_migrates_duplicate_typed_secrets_once() {
        let dir = test_dir();
        {
            let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
            let mut definition = test_definition("legacy", TunnelType::Client);
            definition.raw_config.insert(
                "ProxyPassword".to_string(),
                serde_json::json!("legacy-secret"),
            );
            store.upsert(definition).await.unwrap();
        }

        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
        let revision = store.load().await.unwrap().unwrap();
        let definition = store.get("legacy").unwrap();
        assert_eq!(
            definition.options.proxy_password.as_deref(),
            Some("legacy-secret")
        );
        assert!(!definition.raw_config.contains_key("ProxyPassword"));
        assert_eq!(revision, StateRevision::new(2));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn m163_scrubs_legacy_fields_and_all_retained_history() {
        let dir = test_dir();
        let definition = contaminated_definition();
        let payload = TunnelStorePayload {
            tunnels: [(definition.name.as_str().to_string(), definition)].into_iter().collect(),
            scrub_state: TunnelStoreScrubState::LegacyUnstarted,
        };
        write_generation(&dir, 1, payload.clone(), true).await;
        write_generation(&dir, 2, payload, false).await;

        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
        let revision = store.load().await.unwrap().unwrap();
        let definition = store.get("legacy").unwrap();
        assert_eq!(revision, StateRevision::new(5));
        assert!(definition.options.encrypt_lease_set.is_none());
        assert!(definition.options.optional_lookup.is_none());
        assert!(definition.options.lease_set_client_auths.is_empty());
        assert_eq!(definition.options.description.as_deref(), Some("preserve me"));
        assert_eq!(definition.start_intent, StartIntent::StartOnLoad);
        assert_eq!(definition.raw_config.get("SafeOption"), Some(&serde_json::json!("preserve")));
        assert!(!definition.raw_config.contains_key("OptionalLookup"));

        let generations: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
            .collect();
        assert_eq!(generations.len(), 3, "two clean fallbacks plus completion");
        for path in generations {
            let bytes = std::fs::read(path).unwrap();
            assert!(!bytes.windows(b"lookup-secret".len()).any(|window| window == b"lookup-secret"));
            assert!(!bytes.windows(b"OptionalLookup".len()).any(|window| window == b"OptionalLookup"));
            assert!(!bytes.windows(b"alice".len()).any(|window| window == b"alice"));
        }
        let loaded: serde_json::Value = serde_json::from_slice(
            &std::fs::read(dir.join("gen-00000000000000000005.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(loaded["payload"]["scrub_state"], "scrub_complete");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn m163_pending_scrub_resumes_after_purge_failure() {
        let dir = test_dir();
        let definition = contaminated_definition();
        let payload = TunnelStorePayload {
            tunnels: [(definition.name.as_str().to_string(), definition)].into_iter().collect(),
            scrub_state: TunnelStoreScrubState::LegacyUnstarted,
        };
        write_generation(&dir, 1, payload, true).await;

        let mut interrupted = TunnelStore::new(dir.clone(), 1024 * 1024);
        interrupted.fail_next_history_purge();
        assert!(interrupted.load().await.is_err());
        drop(interrupted);

        let mut resumed = TunnelStore::new(dir.clone(), 1024 * 1024);
        assert!(resumed.load().await.is_ok());
        assert!(resumed.get("legacy").unwrap().options.optional_lookup.is_none());
        let bytes = std::fs::read_dir(&dir)
            .unwrap()
            .map(|entry| std::fs::read(entry.unwrap().path()).unwrap())
            .collect::<Vec<_>>();
        assert!(bytes.iter().all(|bytes| !bytes.windows(b"lookup-secret".len()).any(
            |window| window == b"lookup-secret"
        )));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn m163_directory_sync_failure_is_fail_closed_and_retryable() {
        let dir = test_dir();
        let definition = contaminated_definition();
        let payload = TunnelStorePayload {
            tunnels: [(definition.name.as_str().to_string(), definition)].into_iter().collect(),
            scrub_state: TunnelStoreScrubState::LegacyUnstarted,
        };
        write_generation(&dir, 1, payload, true).await;

        let mut interrupted = TunnelStore::new(dir.clone(), 1024 * 1024);
        interrupted.fail_next_history_purge_directory_sync();
        assert!(interrupted.load().await.is_err());
        drop(interrupted);

        let mut resumed = TunnelStore::new(dir.clone(), 1024 * 1024);
        assert!(resumed.load().await.is_ok());
        assert!(resumed.get("legacy").unwrap().options.lease_set_client_auths.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn m163_store_mutations_reject_blocked_state_before_publication() {
        let dir = test_dir();
        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
        let contaminated = contaminated_definition();
        assert!(store.upsert(contaminated.clone()).await.is_err());
        assert!(store.is_empty());
        assert!(std::fs::read_dir(&dir).unwrap().next().is_none());

        let ordinary = test_definition("ordinary", TunnelType::Server);
        store.upsert(ordinary.clone()).await.unwrap();
        let revision = store.revision();
        assert!(store.update("ordinary", contaminated, None).await.is_err());
        assert_eq!(store.revision(), revision);
        assert_eq!(store.get("ordinary"), Some(&ordinary));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn m163_scrub_complete_contamination_reenters_pending_scrub() {
        let dir = test_dir();
        let definition = contaminated_definition();
        let payload = TunnelStorePayload {
            tunnels: [(definition.name.as_str().to_string(), definition)].into_iter().collect(),
            scrub_state: TunnelStoreScrubState::ScrubComplete,
        };
        write_generation(&dir, 1, payload, false).await;

        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
        store.load().await.unwrap();
        let definition = store.get("legacy").unwrap();
        assert!(definition.options.encrypt_lease_set.is_none());
        assert!(definition.options.optional_lookup.is_none());
        assert!(definition.options.lease_set_client_auths.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn unsupported_tunnel_persistence() {
        let dir = test_dir();
        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);

        let def = TunnelDefinition {
            name: TunnelName::new("unsupported-test").unwrap(),
            tunnel_type: TunnelType::StreamrServer,
            ownership: TunnelOwnership::Unsupported,
            runtime_state: TunnelRuntimeState::Unsupported,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions::default(),
            raw_config: std::collections::BTreeMap::new(),
        };
        store.upsert(def.clone()).await.unwrap();

        let found = store.get("unsupported-test").unwrap();
        assert_eq!(found.ownership, TunnelOwnership::Unsupported);
        assert_eq!(found.runtime_state, TunnelRuntimeState::Unsupported);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn contains_method() {
        let dir = test_dir();
        let mut store = TunnelStore::new(dir.clone(), 1024 * 1024);
        assert!(!store.contains("test"));

        store.upsert(test_definition("test", TunnelType::Client)).await.unwrap();
        assert!(store.contains("test"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
