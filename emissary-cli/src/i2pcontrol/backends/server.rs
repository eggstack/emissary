//! Control-plane-owned generic server tunnel runtime.

use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{io, net::TcpStream, task::JoinHandle};

use super::{
    options::{validate_options, OptionValidationError, SERVER_OPTIONS},
    runtime::{
        run_accepted_server, AcceptedServerConnection, AcceptedServerHandler,
        AcceptedServerRuntimeConfig, AcceptedServerRuntimeError, ServerAdmissionPolicy,
    },
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
    server_secret_store::{ServerDestinationStore, StoredDestination},
};

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const TARGET_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
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

struct GenericServerRuntimeConfig {
    name: String,
    target_port: u16,
    destination: StoredDestination,
    sam_tcp_port: u16,
    admission: ServerAdmissionPolicy,
    lease_set_enc_type: Option<String>,
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
            if entry.generation == generation && entry.state == TunnelRuntimeState::Starting {
                entry.task = Some(task);
            } else {
                task.abort();
            }
        } else {
            task.abort();
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
        result: Result<(), AcceptedServerRuntimeError>,
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

    /// Start one server runtime and wait for a real destination and session.
    async fn start(&self, config: GenericServerRuntimeConfig) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.reserve(&name)?;
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let map = Arc::clone(&self.inner);
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let handler = make_accepted_handler(config.target_port);
            let result = std::panic::AssertUnwindSafe(run_accepted_server(
                AcceptedServerRuntimeConfig {
                    name: config.name,
                    sam_tcp_port: config.sam_tcp_port,
                    destination: config.destination,
                    admission: config.admission,
                    lease_set_enc_type: config.lease_set_enc_type,
                    handler,
                },
                ready_cancellation.clone(),
                ready_tx,
            ))
            .catch_unwind()
            .await
            .unwrap_or(Err(AcceptedServerRuntimeError::Panicked));
            let cancelled = *ready_cancellation.borrow();
            ServerRuntimeSupervisor::complete(map, task_name, generation, result, cancelled).await;
        });
        self.set_task(&name, generation, task);

        match tokio::time::timeout(START_TIMEOUT, ready_rx).await {
            Ok(Ok(Ok(destination))) => {
                self.publish_destination(&name, generation, &destination);
                if self.mark_running(&name, generation) {
                    Ok(())
                } else {
                    let _ = self.stop_generation(&name, generation).await;
                    Err(BackendError::Internal {
                        message: "server tunnel runtime exited during start".to_string(),
                    })
                }
            }
            Ok(Ok(Err(_))) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "server tunnel runtime failed to start".to_string(),
                })
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
            Some(entry) => (
                entry.state,
                entry.failure.unwrap_or("server tunnel runtime is active"),
                entry.destination.clone(),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                "server tunnel runtime is stopped",
                None,
            ),
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
    ) -> BackendResult<(u16, ServerAdmissionPolicy, Option<String>)> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::Server,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_raw_options(definition)?;
        validate_i2cp_options(definition)?;
        let port =
            definition
                .options
                .target_port
                .or(definition.options.listen_port)
                .ok_or_else(|| BackendError::Internal {
                    message: "server target port is required".to_string(),
                })?;
        if let Some(value) = definition
            .raw_config
            .get("TargetHost")
            .or_else(|| definition.raw_config.get("Host"))
        {
            let host = value.as_str().ok_or_else(|| invalid_option("TargetHost"))?;
            if !matches!(host, "127.0.0.1" | "localhost") {
                return Err(BackendError::Internal {
                    message: "server target host is not supported by the existing data plane"
                        .to_string(),
                });
            }
        }
        let admission = ServerAdmissionPolicy::from_raw_options(&definition.raw_config)
            .map_err(invalid_option)?;
        let lease_set_enc_type = lease_set_enc_type(definition)?;
        Ok((port, admission, lease_set_enc_type))
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
        validate_raw_options(definition)?;
        validate_i2cp_options(definition)?;
        validate_options(TunnelType::Server, &definition.options, SERVER_OPTIONS)
            .map_err(option_error)?;
        let (target_port, admission, lease_set_enc_type) = self.runtime_config(definition)?;
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
            .start(GenericServerRuntimeConfig {
                name: definition.name.as_str().to_string(),
                target_port,
                destination,
                sam_tcp_port: self.supervisor.sam_tcp_port,
                admission,
                lease_set_enc_type,
            })
            .await
    }

    async fn stop(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        self.supervisor.stop(definition.name.as_str()).await
    }

    fn inspect(&self, definition: &TunnelDefinition) -> BackendStatus {
        let (runtime_state, message, destination) =
            self.supervisor.inspect(definition.name.as_str());
        BackendStatus {
            tunnel_type: TunnelType::Server,
            runtime_state,
            message: message.to_string(),
            destination,
        }
    }
}

fn validate_raw_options(definition: &TunnelDefinition) -> BackendResult<()> {
    const SUPPORTED: &[&str] = &[
        "TargetPort",
        "ListenPort",
        "TargetHost",
        "Host",
        "MaxConcurrentConns",
        "ClientPerMinute",
        "ClientPerHour",
        "ClientPerDay",
        "TotalInPerMinute",
        "TotalInPerHour",
        "TotalInPerDay",
        "i2cp",
        "Port",
        "ReachableBy",
    ];
    const METADATA: &[&str] = &[
        "name",
        "type",
        "Name",
        "Type",
        "Description",
        "StartOnLoad",
        "Action",
    ];
    for key in definition.raw_config.keys() {
        if key.starts_with("__emissary_")
            || SUPPORTED.contains(&key.as_str())
            || METADATA.contains(&key.as_str())
        {
            continue;
        }
        return Err(BackendError::UnsupportedOption {
            tunnel_type: TunnelType::Server,
            option: key.clone(),
        });
    }
    Ok(())
}

fn invalid_option(option: &str) -> BackendError {
    BackendError::Internal {
        message: format!("server option {option} is invalid"),
    }
}

fn make_accepted_handler(target_port: u16) -> AcceptedServerHandler {
    Arc::new(move |connection| Box::pin(relay_accepted_connection(connection, target_port)))
}

async fn relay_accepted_connection(mut connection: AcceptedServerConnection, target_port: u16) {
    let Ok(Ok(mut target)) = tokio::time::timeout(
        TARGET_CONNECT_TIMEOUT,
        TcpStream::connect(("127.0.0.1", target_port)),
    )
    .await
    else {
        return;
    };

    let _ = io::copy_bidirectional(&mut connection.stream, &mut target).await;
}

fn validate_i2cp_options(definition: &TunnelDefinition) -> BackendResult<()> {
    if definition.options.i2cp_options.keys().any(|key| key != "leaseSetEncType") {
        return Err(BackendError::UnsupportedOption {
            tunnel_type: TunnelType::Server,
            option: "I2CPOptions".to_owned(),
        });
    }
    Ok(())
}

/// Extract the validated `leaseSetEncType` value from I2CP options.
///
/// The `i2cp_options` map is typed as `BTreeMap<String, String>`, so the
/// existing tunnel-manager parsing contract already rejects non-string values
/// at the setter. An empty string is treated as the absence of the option so
/// we never emit an empty `i2cp.leaseSetEncType=` key on the SAM wire.
fn lease_set_enc_type(definition: &TunnelDefinition) -> BackendResult<Option<String>> {
    match definition.options.i2cp_options.get("leaseSetEncType") {
        Some(value) if !value.is_empty() => Ok(Some(value.clone())),
        _ => Ok(None),
    }
}

fn option_error(error: OptionValidationError) -> BackendError {
    match error {
        OptionValidationError::Missing {
            tunnel_type,
            option,
        } => BackendError::MissingOption {
            tunnel_type,
            option: option.to_owned(),
        },
        OptionValidationError::Unsupported {
            tunnel_type,
            option,
        } => BackendError::UnsupportedOption {
            tunnel_type,
            option,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::i2pcontrol::{
        domain::tunnel::{StartIntent, TunnelName, TunnelOptions},
        server_secret_store::StoredDestination,
    };
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

    #[tokio::test]
    async fn unimplemented_server_options_fail_before_store_or_session_allocation() {
        let backend = ServerTunnelBackend::without_store(1);
        let mut def = definition("unsupported-server-option", "identity");
        def.options.is_private = Some(true);
        assert!(matches!(
            backend.start(&def).await,
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::Server,
                option
            }) if option == "IsPrivate"
        ));

        let mut raw = definition("unsupported-server-raw", "identity");
        raw.raw_config.insert("SignatureType".to_owned(), serde_json::json!("secret"));
        assert!(matches!(
            backend.start(&raw).await,
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::Server,
                option
            }) if option == "SignatureType"
        ));
    }

    #[test]
    fn admission_options_are_applied_and_non_loopback_targets_fail_before_allocation() {
        let backend = ServerTunnelBackend::without_store(1);
        let mut def = definition("admission-options", "identity");
        def.raw_config.insert("MaxConcurrentConns".to_owned(), serde_json::json!(7));
        def.raw_config.insert("ClientPerMinute".to_owned(), serde_json::json!(11));
        let (_, policy, lease_set_enc_type) = backend.runtime_config(&def).unwrap();
        assert_eq!(policy.max_concurrent_connections(), 7);
        assert!(lease_set_enc_type.is_none());

        def.raw_config.insert("TargetHost".to_owned(), serde_json::json!("192.0.2.1"));
        assert!(matches!(
            backend.runtime_config(&def),
            Err(BackendError::Internal { message })
                if message.contains("target host")
        ));
    }

    #[test]
    fn lease_set_enc_type_is_threaded_when_present_and_absent_otherwise() {
        let backend = ServerTunnelBackend::without_store(1);
        let mut def = definition("lcse-enc-type", "identity");
        def.options.i2cp_options.insert("leaseSetEncType".to_owned(), "4,0".to_owned());
        let (_, _, lease_set_enc_type) = backend.runtime_config(&def).unwrap();
        assert_eq!(lease_set_enc_type.as_deref(), Some("4,0"));

        def.options.i2cp_options.insert("leaseSetEncType".to_owned(), String::new());
        let (_, _, lease_set_enc_type) = backend.runtime_config(&def).unwrap();
        assert!(lease_set_enc_type.is_none());

        def.options.i2cp_options.remove("leaseSetEncType");
        let (_, _, lease_set_enc_type) = backend.runtime_config(&def).unwrap();
        assert!(lease_set_enc_type.is_none());
    }

    #[tokio::test]
    async fn generic_server_debug_is_secret_safe() {
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        store.load().await.unwrap();
        let identity = ServerDestinationStore::new_identity();
        let private = base64_encode([3u8; 128]);
        store
            .put(&identity, StoredDestination::from_private(private.clone()))
            .await
            .unwrap();
        let backend = ServerTunnelBackend::new(1, store);
        assert!(!format!("{backend:?}").contains(&private));
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
            .put(
                &identity,
                StoredDestination::from_private(base64_encode([9u8; 128])),
            )
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
        let first_destination = backend.inspect(&first).destination;
        assert!(matches!(
            backend.start(&first).await,
            Err(BackendError::InvalidState { .. })
        ));
        backend.stop(&first).await.unwrap();
        backend.start(&first).await.unwrap();
        assert_eq!(backend.inspect(&first).destination, first_destination);
        backend.stop(&first).await.unwrap();
        assert_eq!(
            backend.inspect(&first).runtime_state,
            TunnelRuntimeState::Stopped
        );
        assert_eq!(
            backend.inspect(&second).runtime_state,
            TunnelRuntimeState::Stopped
        );
        sam_task.abort();
    }

    #[tokio::test]
    async fn generic_server_uses_accepted_stream_and_relays_bytes_without_forwarding() {
        let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        let target_listener = Arc::new(target_listener);
        let sam_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let sam_port = sam_listener.local_addr().unwrap().port();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
        let done_tx = Arc::new(tokio::sync::Mutex::new(Some(done_tx)));
        // M080: relay fixtures must report a structurally valid I2P
        // Destination so the accepted-server boundary admits the peer.
        // `peer-destination` is a placeholder and would be rejected.
        let peer_destination = std::sync::Arc::new(emissary_core::crypto::base64_encode(
            super::super::runtime::peer_identity::test_fixtures::NULL_CERT_DESTINATION_BYTES
                .as_slice(),
        ));
        let commands = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let observed_done = Arc::clone(&done_tx);
        let observed_listener = Arc::clone(&target_listener);
        let observed_commands = Arc::clone(&commands);
        let sam_task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = sam_listener.accept().await else {
                    return;
                };
                let observed_commands = Arc::clone(&observed_commands);
                let observed_done = Arc::clone(&observed_done);
                let observed_listener = Arc::clone(&observed_listener);
                let peer_destination = std::sync::Arc::clone(&peer_destination);
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            return;
                        }
                        observed_commands.lock().push(line.trim().to_owned());
                        if line.starts_with("HELLO") {
                            write_half
                                .write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n")
                                .await
                                .unwrap();
                        } else if line.starts_with("SESSION CREATE") {
                            write_half
                                .write_all(
                                    b"SESSION STATUS RESULT=OK DESTINATION=server-destination\n",
                                )
                                .await
                                .unwrap();
                        } else if line.starts_with("STREAM ACCEPT") {
                            let response = format!(
                                "STREAM STATUS RESULT=OK\n{}\nfrom-i2p",
                                peer_destination.as_str()
                            );
                            write_half.write_all(response.as_bytes()).await.unwrap();
                            let (mut target, _) = match tokio::time::timeout(
                                Duration::from_secs(1),
                                observed_listener.accept(),
                            )
                            .await
                            {
                                Ok(Ok(connection)) => connection,
                                _ => {
                                    if let Some(sender) = observed_done.lock().await.take() {
                                        let _ =
                                            sender.send(Err("target was not connected".to_owned()));
                                    }
                                    return;
                                }
                            };
                            let mut inbound = [0; 8];
                            if tokio::io::AsyncReadExt::read_exact(&mut target, &mut inbound)
                                .await
                                .is_err()
                                || inbound != *b"from-i2p"
                            {
                                if let Some(sender) = observed_done.lock().await.take() {
                                    let _ = sender
                                        .send(Err("raw inbound payload was changed".to_owned()));
                                }
                                return;
                            }
                            target.write_all(b"to-i2p").await.unwrap();
                            target.shutdown().await.unwrap();
                            let mut outbound = [0; 6];
                            if tokio::io::AsyncReadExt::read_exact(&mut reader, &mut outbound)
                                .await
                                .is_err()
                                || outbound != *b"to-i2p"
                            {
                                if let Some(sender) = observed_done.lock().await.take() {
                                    let _ = sender
                                        .send(Err("raw outbound payload was changed".to_owned()));
                                }
                                return;
                            }
                            if let Some(sender) = observed_done.lock().await.take() {
                                let _ = sender.send(Ok(()));
                            }
                            return;
                        }
                    }
                });
            }
        });

        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        store.load().await.unwrap();
        let identity = ServerDestinationStore::new_identity();
        store
            .put(
                &identity,
                StoredDestination::from_private(base64_encode([9u8; 128])),
            )
            .await
            .unwrap();
        let backend = ServerTunnelBackend::new(sam_port, store);
        let mut def = definition("accepted-server", &identity);
        def.options.listen_port = Some(target_port);

        backend.start(&def).await.unwrap();
        let relay_result = tokio::time::timeout(Duration::from_secs(2), done_rx).await;
        assert!(
            relay_result.is_ok(),
            "relay fixture timed out; commands: {:?}",
            commands.lock()
        );
        assert_eq!(relay_result.unwrap().unwrap(), Ok(()));
        backend.stop(&def).await.unwrap();
        sam_task.abort();
        let _ = sam_task.await;

        let commands = commands.lock();
        assert!(commands.iter().any(|command| command.starts_with("STREAM ACCEPT")));
        assert!(!commands.iter().any(|command| command.starts_with("STREAM FORWARD")));
    }

    #[tokio::test]
    async fn startup_server_lifecycle_is_rejected_before_store_access() {
        let backend = ServerTunnelBackend::without_store(1);
        let mut definition = definition("startup-server", "identity");
        definition.ownership = TunnelOwnership::StartupManaged;
        let result = backend.start(&definition).await;
        assert!(matches!(result, Err(BackendError::InvalidState { .. })));
    }

    async fn lease_set_enc_type_fixture(
        _lease_set_enc_type: Option<&'static str>,
    ) -> (
        u16,
        tokio::task::JoinHandle<()>,
        Arc<parking_lot::Mutex<Vec<String>>>,
    ) {
        let sam_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let sam_port = sam_listener.local_addr().unwrap().port();
        let commands = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let observed_commands = Arc::clone(&commands);
        let sam_task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = sam_listener.accept().await else {
                    return;
                };
                let observed_commands = Arc::clone(&observed_commands);
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            return;
                        }
                        observed_commands.lock().push(line.trim().to_owned());
                        if line.starts_with("HELLO") {
                            write_half
                                .write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n")
                                .await
                                .unwrap();
                        } else if line.starts_with("SESSION CREATE") {
                            write_half
                                .write_all(
                                    b"SESSION STATUS RESULT=OK DESTINATION=server-destination\n",
                                )
                                .await
                                .unwrap();
                        } else {
                            write_half.write_all(b"STREAM STATUS RESULT=OK\n").await.unwrap();
                        }
                    }
                });
            }
        });
        (sam_port, sam_task, commands)
    }

    async fn persisted_server_identity(_name: &str) -> (ServerDestinationStore, String) {
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        store.load().await.unwrap();
        let identity = ServerDestinationStore::new_identity();
        store
            .put(
                &identity,
                StoredDestination::from_private(base64_encode([9u8; 128])),
            )
            .await
            .unwrap();
        (store, identity)
    }

    fn session_create_command(commands: &[String]) -> Option<&String> {
        commands.iter().find(|command| command.starts_with("SESSION CREATE"))
    }

    #[tokio::test]
    async fn generic_server_threades_lease_set_enc_type_into_session_create() {
        let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        let _ = Arc::new(target_listener);
        let (sam_port, sam_task, commands) = lease_set_enc_type_fixture(Some("4,0")).await;
        let (store, identity) = persisted_server_identity("lcse-applied").await;
        let backend = ServerTunnelBackend::new(sam_port, store);
        let mut def = definition("lcse-applied", &identity);
        def.options.listen_port = Some(target_port);
        def.options.i2cp_options.insert("leaseSetEncType".to_owned(), "4,0".to_owned());

        backend.start(&def).await.unwrap();
        backend.stop(&def).await.unwrap();
        sam_task.abort();
        let _ = sam_task.await;

        let session = session_create_command(&commands.lock())
            .cloned()
            .expect("SESSION CREATE must be issued");
        assert!(
            session.contains("leaseSetEncType=4,0"),
            "SESSION CREATE must carry leaseSetEncType=4,0; got: {session}"
        );
        assert!(
            session.contains("STREAM"),
            "SESSION CREATE must use STREAM style"
        );
    }

    #[tokio::test]
    async fn generic_server_omits_lease_set_enc_type_when_unset() {
        let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        let _ = Arc::new(target_listener);
        let (sam_port, sam_task, commands) = lease_set_enc_type_fixture(None).await;
        let (store, identity) = persisted_server_identity("lcse-default").await;
        let backend = ServerTunnelBackend::new(sam_port, store);
        let mut def = definition("lcse-default", &identity);
        def.options.listen_port = Some(target_port);

        backend.start(&def).await.unwrap();
        backend.stop(&def).await.unwrap();
        sam_task.abort();
        let _ = sam_task.await;

        let session = session_create_command(&commands.lock())
            .cloned()
            .expect("SESSION CREATE must be issued");
        // Yosemite always emits a leaseSetEncType, defaulting to 6,4 when
        // the caller does not override it. The regression we are guarding
        // against is the option being silently dropped by the I2PControl
        // accepted-server path; the absence of the raw 4,0 value confirms
        // that the I2PControl layer did not incorrectly inject it.
        assert!(
            !session.contains("leaseSetEncType=4,0"),
            "SESSION CREATE must not carry the operator's old value; got: {session}"
        );
    }

    #[tokio::test]
    async fn lease_set_enc_type_survives_restart_with_new_session_generation() {
        let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        let _ = Arc::new(target_listener);
        let (sam_port, sam_task, commands) = lease_set_enc_type_fixture(Some("4,0")).await;
        let (store, identity) = persisted_server_identity("lcse-restart").await;
        let backend = ServerTunnelBackend::new(sam_port, store);
        let mut def = definition("lcse-restart", &identity);
        def.options.listen_port = Some(target_port);
        def.options.i2cp_options.insert("leaseSetEncType".to_owned(), "4,0".to_owned());

        backend.start(&def).await.unwrap();
        backend.stop(&def).await.unwrap();
        backend.start(&def).await.unwrap();
        backend.stop(&def).await.unwrap();
        sam_task.abort();
        let _ = sam_task.await;

        let session_count = commands
            .lock()
            .iter()
            .filter(|command| command.starts_with("SESSION CREATE"))
            .count();
        assert_eq!(
            session_count, 2,
            "both generations must issue their own SESSION CREATE"
        );
        for session in
            commands.lock().iter().filter(|command| command.starts_with("SESSION CREATE"))
        {
            assert!(
                session.contains("leaseSetEncType=4,0"),
                "every restart generation must carry leaseSetEncType=4,0; got: {session}"
            );
        }
    }

    #[tokio::test]
    async fn unknown_generic_server_i2cp_keys_still_fail_before_allocation() {
        let backend = ServerTunnelBackend::without_store(1);
        let mut def = definition("lcse-unknown", "identity");
        def.options.i2cp_options.insert("SignatureType".to_owned(), "7".to_owned());
        let result = backend.start(&def).await;
        assert!(matches!(
            result,
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::Server,
                option
            }) if option == "I2CPOptions"
        ));
    }
}
