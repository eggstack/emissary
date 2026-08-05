//! Control-plane-owned generic server tunnel runtime.

use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::task::JoinHandle;

use super::{BackendError, BackendResult, BackendStatus, TunnelBackend};
use crate::{
    i2pcontrol::{
        domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
        server_secret_store::ServerDestinationStore,
    },
    tunnel_server::{
        run_single_server, ServerRuntimeError, ServerTunnelRuntimeConfig, DestinationObserver,
    },
};

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RUNTIME_TASKS: usize = 1000;
pub(crate) const SERVER_IDENTITY_KEY: &str = "__emissary_server_destination_identity";
pub(crate) const SERVER_PUBLIC_DESTINATION_KEY: &str = "__emissary_server_public_destination";

#[derive(Debug)]
struct RuntimeEntry {
    generation: u64,
    state: TunnelRuntimeState,
    cancellation: tokio::sync::watch::Sender<bool>,
    task: Option<JoinHandle<()>>,
    failure: Option<&'static str>,
    destination: Option<String>,
}

#[derive(Debug, Default)]
struct RuntimeMap {
    next_generation: u64,
    entries: HashMap<String, RuntimeEntry>,
}

/// Bounded, per-name runtime supervisor for control-plane server tunnels.
#[derive(Clone, Debug)]
pub struct ServerRuntimeSupervisor {
    inner: Arc<Mutex<RuntimeMap>>,
    sam_tcp_port: u16,
}

impl ServerRuntimeSupervisor {
    /// Create a supervisor using the router's already-bound SAM endpoint.
    pub fn new(sam_tcp_port: u16) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeMap::default())),
            sam_tcp_port,
        }
    }

    fn reserve(&self, name: &str) -> BackendResult<(u64, tokio::sync::watch::Receiver<bool>)> {
        let mut runtime = self.inner.lock();
        if let Some(entry) = runtime.entries.get(name) {
            if entry.task.is_some()
                && matches!(
                    entry.state,
                    TunnelRuntimeState::Starting
                        | TunnelRuntimeState::Running
                        | TunnelRuntimeState::Stopping
                )
            {
                return Err(BackendError::InvalidState {
                    tunnel_type: TunnelType::Server,
                    current_state: entry.state,
                    attempted_action: "start",
                });
            }
        }
        let active_tasks = runtime.entries.values().filter(|entry| entry.task.is_some()).count();
        if active_tasks >= MAX_RUNTIME_TASKS {
            return Err(BackendError::Internal {
                message: "server runtime capacity exhausted".to_string(),
            });
        }
        runtime.next_generation = runtime.next_generation.wrapping_add(1);
        let generation = runtime.next_generation;
        let (cancellation, receiver) = tokio::sync::watch::channel(false);
        runtime.entries.insert(
            name.to_string(),
            RuntimeEntry {
                generation,
                state: TunnelRuntimeState::Starting,
                cancellation,
                task: None,
                failure: None,
                destination: None,
            },
        );
        Ok((generation, receiver))
    }

    fn set_task(&self, name: &str, generation: u64, task: JoinHandle<()>) {
        let mut runtime = self.inner.lock();
        if let Some(entry) = runtime.entries.get_mut(name) {
            if entry.generation == generation {
                entry.task = Some(task);
            }
        }
    }

    fn publish_destination(&self, name: &str, generation: u64, destination: &str) {
        let mut runtime = self.inner.lock();
        if let Some(entry) = runtime.entries.get_mut(name) {
            if entry.generation == generation && !destination.is_empty() {
                entry.destination = Some(destination.to_string());
            }
        }
    }

    fn mark_running(&self, name: &str, generation: u64) -> bool {
        let mut runtime = self.inner.lock();
        let Some(entry) = runtime.entries.get_mut(name) else {
            return false;
        };
        if entry.generation != generation || entry.task.is_none() || entry.destination.is_none() {
            return false;
        }
        entry.state = TunnelRuntimeState::Running;
        true
    }

    async fn complete(
        map: Arc<Mutex<RuntimeMap>>,
        name: String,
        generation: u64,
        result: Result<(), ServerRuntimeError>,
        cancelled: bool,
    ) {
        let mut runtime = map.lock();
        let Some(entry) = runtime.entries.get_mut(name.as_str()) else {
            return;
        };
        if entry.generation != generation {
            return;
        }
        entry.task = None;
        if cancelled {
            entry.state = TunnelRuntimeState::Stopped;
            entry.failure = None;
        } else if result.is_err() {
            entry.state = TunnelRuntimeState::Failed;
            entry.failure = Some("server tunnel runtime failed");
        } else {
            entry.state = TunnelRuntimeState::Stopped;
            entry.failure = None;
        }
    }

    async fn remove_generation(&self, name: &str, generation: u64) {
        let mut runtime = self.inner.lock();
        if runtime.entries.get(name).is_some_and(|entry| entry.generation == generation) {
            runtime.entries.remove(name);
        }
    }

    async fn stop_generation(&self, name: &str, generation: u64) -> BackendResult<()> {
        let (cancellation, task) = {
            let mut runtime = self.inner.lock();
            let Some(entry) = runtime.entries.get_mut(name) else {
                return Ok(());
            };
            if entry.generation != generation {
                return Ok(());
            }
            entry.state = TunnelRuntimeState::Stopping;
            entry.failure = None;
            (entry.cancellation.clone(), entry.task.take())
        };
        let Some(mut task) = task else {
            self.remove_generation(name, generation).await;
            return Ok(());
        };
        let _ = cancellation.send(true);
        if tokio::time::timeout(STOP_TIMEOUT, &mut task).await.is_err() {
            task.abort();
            let _ = task.await;
            self.remove_generation(name, generation).await;
            return Err(BackendError::Internal {
                message: "server tunnel stop timed out".to_string(),
            });
        }
        self.remove_generation(name, generation).await;
        Ok(())
    }

    /// Start one server runtime and wait for a real destination and forward.
    pub async fn start(&self, config: ServerTunnelRuntimeConfig) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.reserve(&name)?;
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let map = Arc::clone(&self.inner);
        let supervisor = self.clone();
        let observer: DestinationObserver = Arc::new(move |observed_name, destination| {
            supervisor.publish_destination(observed_name, generation, destination);
        });
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(run_single_server(
                config,
                ready_cancellation.clone(),
                ready_tx,
                Some(observer),
            ))
            .catch_unwind()
            .await
            .unwrap_or(Err(ServerRuntimeError::Panicked));
            let cancelled = *ready_cancellation.borrow();
            ServerRuntimeSupervisor::complete(map, task_name, generation, result, cancelled).await;
        });
        self.set_task(&name, generation, task);

        match tokio::time::timeout(START_TIMEOUT, ready_rx).await {
            Ok(Ok(Ok(()))) => {
                if self.mark_running(&name, generation) {
                    Ok(())
                } else {
                    let _ = self.stop_generation(&name, generation).await;
                    Err(BackendError::Internal {
                        message: "server tunnel runtime exited during start".to_string(),
                    })
                }
            }
            Ok(Ok(Err(message))) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal { message })
            }
            Ok(Err(_)) | Err(_) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "server tunnel runtime exited during start".to_string(),
                })
            }
        }
    }

    /// Stop one named server runtime and await its exact task.
    pub async fn stop(&self, name: &str) -> BackendResult<()> {
        let generation = self.inner.lock().entries.get(name).map(|entry| entry.generation);
        match generation {
            Some(generation) => self.stop_generation(name, generation).await,
            None => Ok(()),
        }
    }

    /// Return live state and the validated public destination, if available.
    pub fn inspect(&self, name: &str) -> (TunnelRuntimeState, &'static str, Option<String>) {
        let runtime = self.inner.lock();
        match runtime.entries.get(name) {
            Some(entry) => (entry.state, entry.failure.unwrap_or("server tunnel runtime is active"), entry.destination.clone()),
            None => (TunnelRuntimeState::Stopped, "server tunnel runtime is stopped", None),
        }
    }
}

/// Real backend for the generic Proposal 170 `server` tunnel type.
#[derive(Clone, Debug)]
pub struct ServerTunnelBackend {
    supervisor: ServerRuntimeSupervisor,
    destinations: Option<ServerDestinationStore>,
}

impl ServerTunnelBackend {
    /// Create a server backend with the fixed backend-owned secret store.
    pub fn new(sam_tcp_port: u16, destinations: ServerDestinationStore) -> Self {
        Self {
            supervisor: ServerRuntimeSupervisor::new(sam_tcp_port),
            destinations: Some(destinations),
        }
    }

    /// Create a test/inspection backend that fails closed until composed with a store.
    #[allow(dead_code)]
    pub fn without_store(sam_tcp_port: u16) -> Self {
        Self {
            supervisor: ServerRuntimeSupervisor::new(sam_tcp_port),
            destinations: None,
        }
    }

    fn identity(definition: &TunnelDefinition) -> BackendResult<&str> {
        definition
            .raw_config
            .get(SERVER_IDENTITY_KEY)
            .and_then(|value| value.as_str())
            .ok_or_else(|| BackendError::Internal {
                message: "server destination identity is not allocated".to_string(),
            })
    }

    fn runtime_config(
        &self,
        definition: &TunnelDefinition,
        destination: &str,
    ) -> BackendResult<ServerTunnelRuntimeConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::Server,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        let port = definition.options.target_port.or(definition.options.listen_port).ok_or_else(|| {
            BackendError::Internal {
                message: "server target port is required".to_string(),
            }
        })?;
        if let Some(host) = definition
            .raw_config
            .get("TargetHost")
            .or_else(|| definition.raw_config.get("Host"))
            .and_then(|value| value.as_str())
        {
            if !matches!(host, "127.0.0.1" | "localhost") {
                return Err(BackendError::Internal {
                    message: "server target host is not supported by the existing data plane".to_string(),
                });
            }
        }
        Ok(ServerTunnelRuntimeConfig {
            name: definition.name.as_str().to_string(),
            port,
            destination: destination.to_string(),
            sam_tcp_port: self.supervisor.sam_tcp_port,
            lease_set_enc_type: definition.options.i2cp_options.get("leaseSetEncType").cloned(),
        })
    }
}

#[async_trait::async_trait]
impl TunnelBackend for ServerTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::Server
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::Server,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        let store = self.destinations.as_ref().ok_or_else(|| BackendError::Internal {
            message: "server destination store is not composed".to_string(),
        })?;
        let identity = Self::identity(definition)?;
        let destination = store
            .get(identity)
            .await
            .map_err(|_| BackendError::Internal {
                message: "server destination store lookup failed".to_string(),
            })?
            .ok_or_else(|| BackendError::Internal {
                message: "server destination identity is unavailable".to_string(),
            })?;
        self.supervisor
            .start(self.runtime_config(definition, destination.as_str())?)
            .await
    }

    async fn stop(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        self.supervisor.stop(definition.name.as_str()).await
    }

    fn inspect(&self, definition: &TunnelDefinition) -> BackendStatus {
        let (runtime_state, message, destination) = self.supervisor.inspect(definition.name.as_str());
        BackendStatus {
            tunnel_type: TunnelType::Server,
            runtime_state,
            message: message.to_string(),
            destination,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::i2pcontrol::domain::tunnel::{StartIntent, TunnelName, TunnelOptions};
    use crate::i2pcontrol::server_secret_store::StoredDestination;
    use emissary_core::crypto::base64_encode;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    fn definition(name: &str, identity: &str) -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new(name).unwrap(),
            tunnel_type: TunnelType::Server,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions {
                listen_port: Some(0),
                ..Default::default()
            },
            raw_config: std::collections::BTreeMap::from([(
                SERVER_IDENTITY_KEY.to_string(),
                serde_json::json!(identity),
            )]),
        }
    }

    async fn fake_sam() -> (u16, tokio::task::JoinHandle<()>) {
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
                        let response = if line.starts_with("HELLO") {
                            "HELLO REPLY RESULT=OK VERSION=3.3\n"
                        } else if line.starts_with("SESSION CREATE") {
                            "SESSION STATUS DESTINATION=server-destination\n"
                        } else {
                            "STREAM STATUS RESULT=OK\n"
                        };
                        if write_half.write_all(response.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                });
            }
        });
        (port, task)
    }

    #[tokio::test]
    async fn server_lifecycle_preserves_public_destination_and_cancels_exact_task() {
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        store.load().await.unwrap();
        let identity = ServerDestinationStore::new_identity();
        store
            .put(&identity, StoredDestination::from_private(base64_encode([9u8; 128])))
            .await
            .unwrap();
        let (sam_port, sam_task) = fake_sam().await;
        let backend = Arc::new(ServerTunnelBackend::new(sam_port, store));
        let first = definition("first-server", &identity);
        let second_identity = ServerDestinationStore::new_identity();
        let second = definition("second-server", &second_identity);

        backend.start(&first).await.unwrap();
        assert_eq!(
            backend.inspect(&first).runtime_state,
            TunnelRuntimeState::Running
        );
        assert_eq!(
            backend.inspect(&first).destination.as_deref(),
            Some("server-destination")
        );
        assert!(matches!(
            backend.start(&first).await,
            Err(BackendError::InvalidState { .. })
        ));
        backend.stop(&first).await.unwrap();
        assert_eq!(
            backend.inspect(&first).runtime_state,
            TunnelRuntimeState::Stopped
        );
        assert_eq!(backend.inspect(&second).runtime_state, TunnelRuntimeState::Stopped);
        sam_task.abort();
    }

    #[tokio::test]
    async fn startup_server_lifecycle_is_rejected_before_store_access() {
        let backend = ServerTunnelBackend::without_store(1);
        let mut definition = definition("startup-server", "identity");
        definition.ownership = TunnelOwnership::StartupManaged;
        let result = backend.start(&definition).await;
        assert!(matches!(result, Err(BackendError::InvalidState { .. })));
    }
}
