//! Filtered control-plane-owned IRC client tunnel.

use std::{collections::HashMap, net::IpAddr, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::task::JoinHandle;

use super::{
    filters::irc::relay_client_stream,
    options::{validate_options, OptionValidationError, IRC_CLIENT_OPTIONS},
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use yosemite::SessionOptions;
use crate::i2pcontrol::{
    backends::runtime::{
        run_client_listener, ClientConnectionHandler, ClientListenerRuntimeConfig,
        ClientListenerRuntimeError,
    },
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
};

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RUNTIME_TASKS: usize = 1000;
const MAX_CONNECTIONS: usize = 128;

#[derive(Debug, Clone)]
struct IrcClientConfig {
    name: String,
    bind_address: IpAddr,
    port: u16,
    destination: String,
    destination_port: u16,
    sam_tcp_port: u16,
    session_options: SessionOptions,
}

#[derive(Debug)]
struct RuntimeEntry {
    generation: u64,
    state: TunnelRuntimeState,
    cancellation: tokio::sync::watch::Sender<bool>,
    task: Option<JoinHandle<()>>,
    failure: Option<&'static str>,
}

#[derive(Debug, Default)]
struct RuntimeMap {
    next_generation: u64,
    entries: HashMap<String, RuntimeEntry>,
}

#[derive(Clone, Debug)]
struct IrcClientRuntimeSupervisor {
    inner: Arc<Mutex<RuntimeMap>>,
}

impl IrcClientRuntimeSupervisor {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeMap::default())),
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
                    tunnel_type: TunnelType::IrcClient,
                    current_state: entry.state,
                    attempted_action: "start",
                });
            }
        }
        if runtime.entries.values().filter(|entry| entry.task.is_some()).count()
            >= MAX_RUNTIME_TASKS
        {
            return Err(BackendError::Internal {
                message: "ircclient runtime capacity exhausted".to_owned(),
            });
        }
        runtime.next_generation = runtime.next_generation.wrapping_add(1);
        let generation = runtime.next_generation;
        let (cancellation, receiver) = tokio::sync::watch::channel(false);
        runtime.entries.insert(
            name.to_owned(),
            RuntimeEntry {
                generation,
                state: TunnelRuntimeState::Starting,
                cancellation,
                task: None,
                failure: None,
            },
        );
        Ok((generation, receiver))
    }

    fn set_task(&self, name: &str, generation: u64, task: JoinHandle<()>) {
        let mut runtime = self.inner.lock();
        if let Some(entry) = runtime.entries.get_mut(name) {
            if entry.generation == generation && entry.state == TunnelRuntimeState::Starting {
                entry.task = Some(task);
                return;
            }
        }
        task.abort();
    }

    fn mark_running(&self, name: &str, generation: u64) -> bool {
        let mut runtime = self.inner.lock();
        let Some(entry) = runtime.entries.get_mut(name) else {
            return false;
        };
        if entry.generation != generation || entry.task.is_none() {
            return false;
        }
        entry.state = TunnelRuntimeState::Running;
        true
    }

    fn complete(
        map: Arc<Mutex<RuntimeMap>>,
        name: String,
        generation: u64,
        result: Result<(), ClientListenerRuntimeError>,
        cancelled: bool,
    ) {
        let mut runtime = map.lock();
        let Some(entry) = runtime.entries.get_mut(&name) else {
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
            entry.failure = Some("ircclient tunnel runtime failed");
        } else {
            entry.state = TunnelRuntimeState::Stopped;
            entry.failure = None;
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
            self.remove_generation(name, generation);
            return Ok(());
        };
        let _ = cancellation.send(true);
        if tokio::time::timeout(STOP_TIMEOUT, &mut task).await.is_err() {
            task.abort();
            let _ = task.await;
            self.remove_generation(name, generation);
            return Err(BackendError::Internal {
                message: "ircclient tunnel stop timed out".to_owned(),
            });
        }
        self.remove_generation(name, generation);
        Ok(())
    }

    fn remove_generation(&self, name: &str, generation: u64) {
        let mut runtime = self.inner.lock();
        if runtime.entries.get(name).is_some_and(|entry| entry.generation == generation) {
            runtime.entries.remove(name);
        }
    }

    async fn start(&self, config: IrcClientConfig) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.reserve(&name)?;
        let map = Arc::clone(&self.inner);
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let handler: ClientConnectionHandler = Arc::new(|stream, connector| {
            Box::pin(async move {
                let Ok(remote) = connector.connect().await else {
                    return;
                };
                let _ = relay_client_stream(stream, remote).await;
            })
        });
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(run_client_listener(
                ClientListenerRuntimeConfig {
                    name: config.name,
                    bind_address: config.bind_address,
                    port: config.port,
                    destination: config.destination,
                    destination_port: config.destination_port,
                    sam_tcp_port: config.sam_tcp_port,
                    session_options: config.session_options,
                    max_connections: MAX_CONNECTIONS,
                    handler,
                },
                ready_cancellation.clone(),
                ready_tx,
            ))
            .catch_unwind()
            .await
            .unwrap_or(Err(ClientListenerRuntimeError::Panicked));
            let cancelled = *ready_cancellation.borrow();
            Self::complete(map, task_name, generation, result, cancelled);
        });
        self.set_task(&name, generation, task);
        match tokio::time::timeout(START_TIMEOUT, ready_rx).await {
            Ok(Ok(Ok(_))) if self.mark_running(&name, generation) => Ok(()),
            Ok(Ok(Err(_))) | Ok(Err(_)) | Err(_) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "ircclient tunnel runtime failed to start".to_owned(),
                })
            }
            Ok(Ok(Ok(_))) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "ircclient tunnel runtime exited during start".to_owned(),
                })
            }
        }
    }

    async fn stop(&self, name: &str) -> BackendResult<()> {
        let generation = self.inner.lock().entries.get(name).map(|entry| entry.generation);
        match generation {
            Some(generation) => self.stop_generation(name, generation).await,
            None => Ok(()),
        }
    }

    fn inspect(&self, name: &str) -> (TunnelRuntimeState, &'static str) {
        let runtime = self.inner.lock();
        match runtime.entries.get(name) {
            Some(entry) => (
                entry.state,
                entry.failure.unwrap_or("ircclient tunnel runtime is active"),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                "ircclient tunnel runtime is stopped",
            ),
        }
    }
}

/// Real filtered backend for Proposal 170 `ircclient`.
#[derive(Clone, Debug)]
pub struct IrcClientTunnelBackend {
    supervisor: IrcClientRuntimeSupervisor,
    sam_tcp_port: u16,
}

impl IrcClientTunnelBackend {
    pub fn new(sam_tcp_port: u16) -> Self {
        Self {
            supervisor: IrcClientRuntimeSupervisor::new(),
            sam_tcp_port,
        }
    }

    fn config(&self, definition: &TunnelDefinition) -> BackendResult<IrcClientConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::IrcClient,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_options(
            TunnelType::IrcClient,
            &definition.options,
            IRC_CLIENT_OPTIONS,
        )
        .map_err(option_error)?;
        let destination = definition
            .options
            .target_destination
            .as_deref()
            .filter(|value| valid_destination(value))
            .ok_or_else(|| BackendError::Internal {
                message: "ircclient target destination is invalid".to_owned(),
            })?;
        let port = definition.options.listen_port.ok_or_else(|| BackendError::MissingOption {
            tunnel_type: TunnelType::IrcClient,
            option: "ListenPort".to_owned(),
        })?;
        let bind_address = match definition.options.listen_interface.as_deref() {
            None => "127.0.0.1".parse().expect("loopback address is valid"),
            Some(value) => value.parse::<IpAddr>().map_err(|_| BackendError::Internal {
                message: "ircclient listen interface must be an IP address".to_owned(),
            })?,
        };
        Ok(IrcClientConfig {
            name: definition.name.as_str().to_owned(),
            bind_address,
            port,
            destination: destination.to_owned(),
            destination_port: definition.options.target_port.unwrap_or(0),
            sam_tcp_port: self.sam_tcp_port,
            session_options: SessionOptions::default(),
        })
    }
}

fn valid_destination(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && !value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        && !value.contains('/')
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

#[async_trait::async_trait]
impl TunnelBackend for IrcClientTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::IrcClient
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        let mut config = self.config(definition)?;
        config.session_options = super::runtime::session::build_session_options(
            definition,
            self.sam_tcp_port,
            false,
            yosemite::DestinationKind::Transient,
        )?;
        self.supervisor.start(config).await
    }

    async fn stop(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        self.supervisor.stop(definition.name.as_str()).await
    }

    fn inspect(&self, definition: &TunnelDefinition) -> BackendStatus {
        let (runtime_state, message) = self.supervisor.inspect(definition.name.as_str());
        BackendStatus {
            tunnel_type: TunnelType::IrcClient,
            runtime_state,
            message: message.to_owned(),
            destination: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2pcontrol::domain::tunnel::{StartIntent, TunnelName, TunnelOptions};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    fn definition() -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("irc-client").unwrap(),
            tunnel_type: TunnelType::IrcClient,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions {
                target_destination: Some("peer.b32.i2p".to_owned()),
                listen_port: Some(0),
                ..Default::default()
            },
            raw_config: Default::default(),
        }
    }

    #[tokio::test]
    async fn unsupported_irc_automation_is_rejected_before_runtime_start() {
        let backend = IrcClientTunnelBackend::new(1);
        let mut definition = definition();
        definition.options.irc_password =
            crate::i2pcontrol::domain::tunnel::OptionRedacted::new("secret");
        let result = backend.start(&definition).await;
        assert!(
            matches!(result, Err(BackendError::UnsupportedOption { ref option, .. }) if option == "IrcPassword")
        );
        assert!(!format!("{result:?}").contains("secret"));
        assert_eq!(
            backend.inspect(&definition).runtime_state,
            TunnelRuntimeState::Stopped
        );
    }

    #[test]
    fn destination_and_bind_validation_are_direct_and_non_resolving() {
        let backend = IrcClientTunnelBackend::new(1);
        let mut definition = definition();
        definition.options.target_destination = Some("irc.example.com/path".to_owned());
        assert!(matches!(
            backend.config(&definition),
            Err(BackendError::Internal { .. })
        ));
        definition.options.target_destination = Some("peer.b32.i2p".to_owned());
        definition.options.listen_interface = Some("not-a-hostname".to_owned());
        assert!(matches!(
            backend.config(&definition),
            Err(BackendError::Internal { .. })
        ));
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
                            "SESSION STATUS RESULT=OK DESTINATION=irc-client\n"
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
    async fn lifecycle_is_named_cancellable_and_restartable() {
        let (sam_port, sam_task) = fake_sam().await;
        let backend = IrcClientTunnelBackend::new(sam_port);
        let definition = definition();

        backend.start(&definition).await.unwrap();
        assert_eq!(
            backend.inspect(&definition).runtime_state,
            TunnelRuntimeState::Running
        );
        assert!(matches!(
            backend.start(&definition).await,
            Err(BackendError::InvalidState { .. })
        ));
        backend.stop(&definition).await.unwrap();
        backend.start(&definition).await.unwrap();
        backend.stop(&definition).await.unwrap();
        sam_task.abort();
    }
}
