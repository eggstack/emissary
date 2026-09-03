//! Control-plane-owned generic server tunnel runtime.

use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    task::JoinHandle,
};
use yosemite_i2pcontrol::{DestinationKind, SessionOptions};

use super::{
    options::{validate_common_options, validate_options, OptionValidationError, SERVER_OPTIONS},
    runtime::{
        run_accepted_server, AcceptedServerConnection, AcceptedServerHandler,
        AcceptedServerRuntimeConfig, AcceptedServerRuntimeError, ServerAccessPolicy,
        ServerAdmissionPolicy,
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
/// Match the established IRC server's activity-resetting lifetime bound.
///
/// This is an inactivity interval, not an absolute connection lifetime:
/// active raw protocols may remain connected as long as they keep making
/// successful relay progress.
pub(crate) const GENERIC_SERVER_INACTIVITY: Duration = Duration::from_secs(10 * 60);
const RELAY_BUFFER_SIZE: usize = 8192;
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
    access: ServerAccessPolicy,
    lease_set_enc_type: Option<String>,
    session_options: SessionOptions,
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
                    access: config.access,
                    lease_set_enc_type: config.lease_set_enc_type,
                    session_options: Some(config.session_options),
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
    ) -> BackendResult<(
        u16,
        ServerAdmissionPolicy,
        ServerAccessPolicy,
        Option<String>,
    )> {
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
        let access = ServerAccessPolicy::from_values(
            raw_string(definition, "AccessOption")?.as_deref(),
            raw_string(definition, "AccessList")?.as_deref(),
        )
        .map_err(invalid_option)?;
        let lease_set_enc_type = lease_set_enc_type(definition)?;
        Ok((port, admission, access, lease_set_enc_type))
    }
}

#[async_trait::async_trait]
impl TunnelBackend for ServerTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::Server
    }

    fn validate_start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        // Pure preflight: every deterministic gate `start` would reject before
        // secret-store lookup or runtime-map reservation. No store access, no
        // supervisor reservation, no network I/O.
        validate_common_options(TunnelType::Server, &definition.options)
            .map_err(option_error)?;
        validate_raw_options(definition)?;
        validate_i2cp_options(definition)?;
        validate_options(TunnelType::Server, &definition.options, SERVER_OPTIONS)
            .map_err(option_error)?;
        // Port, loopback, admission, access, and LeaseSet shape.
        let _ = self.runtime_config(definition)?;
        // Session-wire ranges (tunnel length/quantity, EncType, SigType,
        // variance, custom options) with a dummy destination so no secret is
        // required.
        let _ = super::runtime::session::build_session_options(
            definition,
            self.supervisor.sam_tcp_port,
            true,
            DestinationKind::Transient,
        )?;
        Ok(())
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        // Reuse the exact preflight helpers so validation cannot drift from
        // the fail-before-allocation gate the control plane runs first.
        self.validate_start(definition)?;
        let (target_port, admission, access, lease_set_enc_type) =
            self.runtime_config(definition)?;
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
        let session_options = super::runtime::session::build_session_options(
            definition,
            self.supervisor.sam_tcp_port,
            true,
            DestinationKind::Persistent {
                private_key: destination.as_str().to_owned(),
            },
        )?;
        self.supervisor
            .start(GenericServerRuntimeConfig {
                name: definition.name.as_str().to_string(),
                target_port,
                destination,
                sam_tcp_port: self.supervisor.sam_tcp_port,
                admission,
                access,
                lease_set_enc_type,
                session_options,
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
        "PerClientPeriod",
        "TotalPeriod",
        "TotalBanTime",
        "AccessOption",
        "AccessList",
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
        "PrivKeyFile",
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

fn raw_string(definition: &TunnelDefinition, key: &str) -> BackendResult<Option<String>> {
    definition
        .raw_config
        .get(key)
        .map(|value| value.as_str().map(str::to_owned).ok_or_else(|| invalid_option(key)))
        .transpose()
}

fn make_accepted_handler(target_port: u16) -> AcceptedServerHandler {
    Arc::new(move |connection| Box::pin(relay_accepted_connection(connection, target_port)))
}

async fn relay_accepted_connection(connection: AcceptedServerConnection, target_port: u16) {
    let Ok(target) = bounded_target_connect(TcpStream::connect(("127.0.0.1", target_port))).await
    else {
        return;
    };

    let (remote_read, remote_write) = io::split(connection.stream);
    let (target_read, target_write) = io::split(target);
    let _ = relay_with_inactivity(remote_read, remote_write, target_read, target_write).await;
}

async fn bounded_target_connect<F>(connect: F) -> io::Result<TcpStream>
where
    F: std::future::Future<Output = io::Result<TcpStream>>,
{
    tokio::time::timeout(TARGET_CONNECT_TIMEOUT, connect)
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "server target connect timeout"))?
}

/// Relay a raw bidirectional stream while bounding only periods without
/// successful byte-transfer progress.
///
/// Each direction shuts down its opposite write half on EOF, matching the
/// useful half-close behavior of `copy_bidirectional`. The completed
/// direction is then left inactive while the other direction drains or until
/// the shared inactivity deadline expires.
async fn relay_with_inactivity<RemoteRead, RemoteWrite, TargetRead, TargetWrite>(
    remote_read: RemoteRead,
    remote_write: RemoteWrite,
    target_read: TargetRead,
    target_write: TargetWrite,
) -> io::Result<()>
where
    RemoteRead: AsyncRead + Unpin,
    RemoteWrite: AsyncWrite + Unpin,
    TargetRead: AsyncRead + Unpin,
    TargetWrite: AsyncWrite + Unpin,
{
    let (activity_tx, mut activity_rx) = tokio::sync::watch::channel(0_u64);
    let mut remote_to_target = Box::pin(relay_direction(
        remote_read,
        target_write,
        activity_tx.clone(),
    ));
    let mut target_to_remote = Box::pin(relay_direction(target_read, remote_write, activity_tx));
    let mut remote_to_target_active = true;
    let mut target_to_remote_active = true;
    let deadline = tokio::time::sleep(GENERIC_SERVER_INACTIVITY);
    tokio::pin!(deadline);

    loop {
        if !remote_to_target_active && !target_to_remote_active {
            return Ok(());
        }

        tokio::select! {
            result = &mut remote_to_target, if remote_to_target_active => {
                remote_to_target_active = false;
                result?;
            }
            result = &mut target_to_remote, if target_to_remote_active => {
                target_to_remote_active = false;
                result?;
            }
            changed = activity_rx.changed() => {
                if changed.is_err() {
                    return Ok(());
                }
                deadline.as_mut().reset(tokio::time::Instant::now() + GENERIC_SERVER_INACTIVITY);
            }
            _ = &mut deadline => return Ok(()),
        }
    }
}

async fn relay_direction<ReadHalf, WriteHalf>(
    mut reader: ReadHalf,
    mut writer: WriteHalf,
    activity: tokio::sync::watch::Sender<u64>,
) -> io::Result<()>
where
    ReadHalf: AsyncRead + Unpin,
    WriteHalf: AsyncWrite + Unpin,
{
    let mut buffer = [0_u8; RELAY_BUFFER_SIZE];
    loop {
        let length = reader.read(&mut buffer).await?;
        if length == 0 {
            writer.shutdown().await?;
            return Ok(());
        }
        writer.write_all(&buffer[..length]).await?;
        activity.send_modify(|sequence| *sequence = sequence.wrapping_add(1));
    }
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
    use std::{
        pin::Pin,
        sync::Arc,
        task::{Context, Poll},
    };

    use super::*;
    use crate::i2pcontrol::{
        domain::tunnel::{StartIntent, TunnelName, TunnelOptions},
        server_secret_store::StoredDestination,
    };
    use emissary_core::crypto::base64_encode;
    use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader, ReadBuf};

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
    async fn preflight_matches_start_without_store_or_session_allocation() {
        let backend = ServerTunnelBackend::without_store(1);
        // Valid shape passes preflight even though no store is composed;
        // store lookup stays a dynamic `start` failure.
        let valid = definition("preflight-valid", "identity");
        assert!(backend.validate_start(&valid).is_ok());

        // Deterministic failures agree between preflight and start, before
        // any store or session work.
        let mut invalid = definition("preflight-invalid", "identity");
        invalid.options.is_private = Some(true);
        assert!(matches!(
            backend.validate_start(&invalid),
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::Server,
                option
            }) if option == "IsPrivate"
        ));
        assert!(matches!(
            backend.start(&invalid).await,
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::Server,
                option
            }) if option == "IsPrivate"
        ));

        let mut raw = definition("preflight-raw", "identity");
        raw.raw_config.insert("SignatureType".to_owned(), serde_json::json!("secret"));
        assert!(matches!(
            backend.validate_start(&raw),
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::Server,
                option
            }) if option == "SignatureType"
        ));
        assert!(matches!(
            backend.start(&raw).await,
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::Server,
                option
            }) if option == "SignatureType"
        ));
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
        let (_, policy, _, lease_set_enc_type) = backend.runtime_config(&def).unwrap();
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
        let (_, _, _, lease_set_enc_type) = backend.runtime_config(&def).unwrap();
        assert_eq!(lease_set_enc_type.as_deref(), Some("4,0"));

        def.options.i2cp_options.insert("leaseSetEncType".to_owned(), String::new());
        let (_, _, _, lease_set_enc_type) = backend.runtime_config(&def).unwrap();
        assert!(lease_set_enc_type.is_none());

        def.options.i2cp_options.remove("leaseSetEncType");
        let (_, _, _, lease_set_enc_type) = backend.runtime_config(&def).unwrap();
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

    #[tokio::test(start_paused = true)]
    async fn generic_server_idle_expiry_releases_admission_lease() {
        let (remote_peer, remote_stream) = tokio::io::duplex(4096);
        let (target_peer, target_stream) = tokio::io::duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (target_read, target_write) = io::split(target_stream);
        let admission = super::super::runtime::ServerAdmissionState::new(
            ServerAdmissionPolicy::new(1, 0, 0, 0, 0, 0, 0).unwrap(),
        );
        let peer = super::super::runtime::peer_identity::test_fixtures::distinct_peer(17);
        let lease = match admission.try_acquire(&peer) {
            super::super::runtime::AdmissionDecision::Allowed(lease) => lease,
            other => panic!("unexpected admission result: {other:?}"),
        };
        let relay = tokio::spawn(async move {
            let _lease = lease;
            relay_with_inactivity(remote_read, remote_write, target_read, target_write).await
        });

        tokio::task::yield_now().await;
        tokio::time::advance(GENERIC_SERVER_INACTIVITY).await;
        assert!(relay.await.unwrap().is_ok());
        assert!(matches!(
            admission.try_acquire(&peer),
            super::super::runtime::AdmissionDecision::Allowed(_)
        ));
        drop(remote_peer);
        drop(target_peer);
    }

    #[tokio::test(start_paused = true)]
    async fn generic_server_progress_resets_deadline_without_fixed_lifetime() {
        let (mut remote_peer, remote_stream) = tokio::io::duplex(4096);
        let (mut target_peer, target_stream) = tokio::io::duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (target_read, target_write) = io::split(target_stream);
        let relay = tokio::spawn(relay_with_inactivity(
            remote_read,
            remote_write,
            target_read,
            target_write,
        ));

        for sequence in 0..3 {
            tokio::time::advance(GENERIC_SERVER_INACTIVITY - Duration::from_secs(1)).await;
            let message = [b'0' + sequence, b'\n'];
            remote_peer.write_all(&message).await.unwrap();
            let mut received = [0_u8; 2];
            target_peer.read_exact(&mut received).await.unwrap();
            assert_eq!(received, message);
            tokio::task::yield_now().await;
        }
        tokio::time::advance(Duration::from_secs(2)).await;
        assert!(!relay.is_finished());

        drop(remote_peer);
        drop(target_peer);
        relay.abort();
        let _ = relay.await;
    }

    #[tokio::test(start_paused = true)]
    async fn generic_server_unidirectional_progress_resets_deadline() {
        let (mut remote_peer, remote_stream) = tokio::io::duplex(4096);
        let (mut target_peer, target_stream) = tokio::io::duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (target_read, target_write) = io::split(target_stream);
        let relay = tokio::spawn(relay_with_inactivity(
            remote_read,
            remote_write,
            target_read,
            target_write,
        ));

        for message in [b"one".as_slice(), b"two".as_slice()] {
            tokio::time::advance(GENERIC_SERVER_INACTIVITY - Duration::from_secs(1)).await;
            remote_peer.write_all(message).await.unwrap();
            let mut received = vec![0_u8; message.len()];
            target_peer.read_exact(&mut received).await.unwrap();
            assert_eq!(received, message);
            tokio::task::yield_now().await;
        }
        tokio::time::advance(Duration::from_secs(2)).await;
        assert!(!relay.is_finished());

        drop(remote_peer);
        drop(target_peer);
        relay.abort();
        let _ = relay.await;
    }

    struct WakesOnce<R> {
        reader: R,
        woke: bool,
    }

    impl<R> AsyncRead for WakesOnce<R>
    where
        R: AsyncRead + Unpin,
    {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            _buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            if !self.woke {
                self.woke = true;
                cx.waker().wake_by_ref();
            }
            let _ = &self.reader;
            Poll::Pending
        }
    }

    #[tokio::test(start_paused = true)]
    async fn generic_server_readiness_wakeup_without_progress_does_not_extend_deadline() {
        let (remote_peer, remote_stream) = tokio::io::duplex(4096);
        let (target_peer, target_stream) = tokio::io::duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (target_read, target_write) = io::split(target_stream);
        let relay = tokio::spawn(relay_with_inactivity(
            WakesOnce {
                reader: remote_read,
                woke: false,
            },
            remote_write,
            target_read,
            target_write,
        ));

        tokio::task::yield_now().await;
        tokio::time::advance(GENERIC_SERVER_INACTIVITY).await;
        assert!(relay.await.unwrap().is_ok());
        drop(remote_peer);
        drop(target_peer);
    }

    #[tokio::test(start_paused = true)]
    async fn generic_server_half_close_drains_the_other_direction() {
        let (mut remote_peer, remote_stream) = tokio::io::duplex(4096);
        let (mut target_peer, target_stream) = tokio::io::duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (target_read, target_write) = io::split(target_stream);
        let relay = tokio::spawn(relay_with_inactivity(
            remote_read,
            remote_write,
            target_read,
            target_write,
        ));

        remote_peer.write_all(b"request").await.unwrap();
        let mut request = [0_u8; 7];
        target_peer.read_exact(&mut request).await.unwrap();
        assert_eq!(&request, b"request");
        remote_peer.shutdown().await.unwrap();

        target_peer.write_all(b"response").await.unwrap();
        let mut response = [0_u8; 8];
        remote_peer.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"response");
        target_peer.shutdown().await.unwrap();

        assert!(relay.await.unwrap().is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn generic_server_target_connect_timeout_remains_five_seconds() {
        let connect = bounded_target_connect(std::future::pending());
        let task = tokio::spawn(connect);
        tokio::task::yield_now().await;
        tokio::time::advance(TARGET_CONNECT_TIMEOUT).await;
        let error = task.await.unwrap().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert_eq!(TARGET_CONNECT_TIMEOUT, Duration::from_secs(5));
    }

    #[test]
    fn generic_server_peer_diagnostics_are_redacted() {
        let peer = super::super::runtime::peer_identity::test_fixtures::distinct_peer(23);
        let destination = peer.destination().to_owned();
        let diagnostics = format!("{peer:?}");
        assert!(!diagnostics.contains(&destination));
        assert!(diagnostics.contains("<redacted>"));
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
