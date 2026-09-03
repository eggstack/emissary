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

//! Control-plane-owned generic client tunnel runtime.

use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::task::JoinHandle;

use super::{
    options::{validate_options, OptionValidationError, CLIENT_OPTIONS},
    runtime::{ClientListenerRuntimeConfig, ClientListenerRuntimeError},
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::{
    i2pcontrol::domain::tunnel::{
        TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType,
    },
    i2pcontrol::client_secret_store::ClientDestinationStore,
    tunnel_client::{ClientRuntimeError, ClientTunnelRuntimeConfig},
};
use yosemite_i2pcontrol::SessionOptions;

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RUNTIME_TASKS: usize = 1000;

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

/// Bounded, per-name runtime supervisor for control-plane client tunnels.
#[derive(Clone, Debug)]
pub struct ClientRuntimeSupervisor {
    inner: Arc<Mutex<RuntimeMap>>,
    sam_tcp_port: u16,
    shared_registry: Option<Arc<super::runtime::session::SharedClientSessionRegistry>>,
    client_destinations: Option<ClientDestinationStore>,
}

impl ClientRuntimeSupervisor {
    /// Create a supervisor using the router's already-bound SAM endpoint.
    pub fn new(sam_tcp_port: u16) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeMap::default())),
            sam_tcp_port,
            shared_registry: None,
            client_destinations: None,
        }
    }

    pub(crate) fn with_client_runtime(
        mut self,
        shared_registry: Arc<super::runtime::session::SharedClientSessionRegistry>,
        client_destinations: ClientDestinationStore,
    ) -> Self {
        self.shared_registry = Some(shared_registry);
        self.client_destinations = Some(client_destinations);
        self
    }

    fn reserve(
        &self,
        config: &ClientTunnelRuntimeConfig,
    ) -> BackendResult<(u64, tokio::sync::watch::Receiver<bool>)> {
        let mut runtime = self.inner.lock();
        if let Some(entry) = runtime.entries.get(config.name.as_str()) {
            if entry.task.is_some()
                && matches!(
                    entry.state,
                    TunnelRuntimeState::Starting
                        | TunnelRuntimeState::Running
                        | TunnelRuntimeState::Stopping
                )
            {
                return Err(BackendError::InvalidState {
                    tunnel_type: TunnelType::Client,
                    current_state: entry.state,
                    attempted_action: "start",
                });
            }
        }

        let active_tasks = runtime.entries.values().filter(|entry| entry.task.is_some()).count();
        if active_tasks >= MAX_RUNTIME_TASKS {
            return Err(BackendError::Internal {
                message: "client runtime capacity exhausted".to_string(),
            });
        }

        runtime.next_generation = runtime.next_generation.wrapping_add(1);
        let generation = runtime.next_generation;
        let (cancellation, receiver) = tokio::sync::watch::channel(false);
        runtime.entries.insert(
            config.name.clone(),
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
            } else {
                task.abort();
            }
        } else {
            task.abort();
        }
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

    async fn complete(
        map: Arc<Mutex<RuntimeMap>>,
        name: String,
        generation: u64,
        result: std::result::Result<(), ClientRuntimeError>,
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
            entry.failure = Some("client tunnel runtime failed");
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
                message: "client tunnel stop timed out".to_string(),
            });
        }
        self.remove_generation(name, generation).await;
        Ok(())
    }

    /// Start one validated client definition and wait for runtime readiness.
    pub(crate) async fn start(
        &self,
        config: ClientTunnelRuntimeConfig,
        session_options: SessionOptions,
        delay_open: bool,
        lifecycle: super::runtime::ClientLifecycleConfig,
        shared_registry: Option<Arc<super::runtime::session::SharedClientSessionRegistry>>,
        shared: bool,
    ) -> BackendResult<()> {
        let (generation, cancellation) = self.reserve(&config)?;
        let ready_config = config.clone();
        let ready_cancellation = cancellation.clone();
        let map = Arc::clone(&self.inner);
        let name = config.name.clone();
        let task_name = name.clone();
        let (listener_ready_tx, listener_ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(super::runtime::run_generic_client_with_shared_session(
                ClientListenerRuntimeConfig {
                    name: ready_config.name,
                    bind_address: ready_config
                        .address
                        .as_deref()
                        .unwrap_or("127.0.0.1")
                        .parse()
                        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
                    port: ready_config.port,
                    destination: ready_config.destination,
                    destination_port: ready_config.destination_port.unwrap_or(0),
                    sam_tcp_port: ready_config.sam_tcp_port,
                    max_connections: 1,
                    delay_open,
                    connect_delay: lifecycle.connect_delay,
                    close_on_idle: lifecycle.close_on_idle,
                    close_idle_time: lifecycle.close_idle_time,
                    new_dest_on_resume: lifecycle.new_dest_on_resume,
                    session_options,
                    handler: std::sync::Arc::new(|_, _| Box::pin(async {})),
                },
                ready_cancellation.clone(),
                listener_ready_tx,
                shared_registry,
                shared,
            ))
            .catch_unwind()
            .await
            .unwrap_or(Err(ClientListenerRuntimeError::Panicked));
            let cancelled = *ready_cancellation.borrow();
            ClientRuntimeSupervisor::complete(
                map,
                task_name,
                generation,
                result.map_err(|_| crate::tunnel_client::ClientRuntimeError::Panicked),
                cancelled,
            )
            .await;
        });

        self.set_task(&name, generation, task);

        match tokio::time::timeout(START_TIMEOUT, listener_ready_rx).await {
            Ok(Ok(Ok(_))) => {
                if self.mark_running(&name, generation) {
                    Ok(())
                } else {
                    let _ = self.stop_generation(&name, generation).await;
                    Err(BackendError::Internal {
                        message: "client tunnel runtime exited during start".to_string(),
                    })
                }
            }
            Ok(Ok(Err(_))) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "client tunnel runtime failed to start".to_owned(),
                })
            }
            Ok(Err(_)) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "client tunnel runtime exited during start".to_string(),
                })
            }
            Err(_) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "client tunnel start timed out".to_string(),
                })
            }
        }
    }

    /// Stop one named client runtime. Absent and completed runtimes are safe.
    pub async fn stop(&self, name: &str) -> BackendResult<()> {
        let generation = self.inner.lock().entries.get(name).map(|entry| entry.generation);
        match generation {
            Some(generation) => self.stop_generation(name, generation).await,
            None => Ok(()),
        }
    }

    /// Return an internal runtime status without side effects.
    pub fn inspect(&self, name: &str) -> (TunnelRuntimeState, &'static str) {
        let runtime = self.inner.lock();
        match runtime.entries.get(name) {
            Some(entry) => (
                entry.state,
                entry.failure.unwrap_or("client tunnel runtime is active"),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                "client tunnel runtime is stopped",
            ),
        }
    }
}

/// Real backend for the generic Proposal 170 `client` tunnel type.
#[derive(Clone, Debug)]
pub struct ClientTunnelBackend {
    supervisor: ClientRuntimeSupervisor,
}

impl ClientTunnelBackend {
    /// Create a client backend with the existing router SAM endpoint.
    pub fn new(sam_tcp_port: u16) -> Self {
        Self {
            supervisor: ClientRuntimeSupervisor::new(sam_tcp_port),
        }
    }

    pub(crate) fn with_client_runtime(
        mut self,
        shared_registry: Arc<super::runtime::session::SharedClientSessionRegistry>,
        client_destinations: ClientDestinationStore,
    ) -> Self {
        self.supervisor = self
            .supervisor
            .with_client_runtime(shared_registry, client_destinations);
        self
    }

    fn config(&self, definition: &TunnelDefinition) -> BackendResult<ClientTunnelRuntimeConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::Client,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_raw_options(definition)?;
        validate_options(TunnelType::Client, &definition.options, CLIENT_OPTIONS)
            .map_err(option_error)?;
        let destination = definition
            .options
            .target_destination
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| BackendError::Internal {
                message: "client target destination is required".to_string(),
            })?;
        let port = definition.options.listen_port.ok_or_else(|| BackendError::Internal {
            message: "client listen port is required".to_string(),
        })?;
        let address = definition.options.listen_interface.clone();
        if address
            .as_deref()
            .is_some_and(|value| value.is_empty() || value.chars().any(char::is_control))
        {
            return Err(BackendError::Internal {
                message: "client listen interface is invalid".to_string(),
            });
        }
        Ok(ClientTunnelRuntimeConfig {
            name: definition.name.as_str().to_string(),
            address,
            port,
            destination: destination.to_string(),
            destination_port: definition.options.target_port,
            sam_tcp_port: self.supervisor.sam_tcp_port,
        })
    }
}

fn validate_raw_options(definition: &TunnelDefinition) -> BackendResult<()> {
    const SUPPORTED: &[&str] = &[
        "TargetDestination",
        "Destination",
        "TargetPort",
        "ListenInterface",
        "ListenPort",
        "ReachableBy",
        "Port",
        "i2p.tunnel.clientDest",
        "i2p.tunnel.clientDestPort",
        "i2p.tunnel.listenInterface",
        "i2p.tunnel.listenPort",
        "DelayOpen",
        "ConnectDelay",
        "Shared",
        "PersistentClientKey",
        "PrivKeyFile",
    ];
    // M121: "Close", "CloseTime", and "NewDest" are demoted to
    // blocked_primitive (reference I2P-session idle semantics have no local
    // observation primitive). Any supplied value fails in
    // client_lifecycle_config / validate_common_options before allocation.
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
            tunnel_type: TunnelType::Client,
            option: key.clone(),
        });
    }
    Ok(())
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
impl TunnelBackend for ClientTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::Client
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        let config = self.config(definition)?;
        let lifecycle = super::runtime::session::client_lifecycle_config(definition)?;
        let session_options = super::runtime::session::build_client_session_options(
            definition,
            self.supervisor.sam_tcp_port,
            self.supervisor.client_destinations.as_ref(),
        )
        .await?;
        self.supervisor
            .start(
                config,
                session_options,
                definition.options.delay_open.unwrap_or(false),
                lifecycle,
                self.supervisor.shared_registry.clone(),
                definition.options.shared.unwrap_or(false),
            )
            .await
    }

    async fn stop(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        self.supervisor.stop(definition.name.as_str()).await
    }

    fn inspect(&self, definition: &TunnelDefinition) -> BackendStatus {
        let (runtime_state, message) = self.supervisor.inspect(definition.name.as_str());
        BackendStatus {
            tunnel_type: TunnelType::Client,
            runtime_state,
            message: message.to_string(),
            destination: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    fn definition(name: &str) -> TunnelDefinition {
        TunnelDefinition {
            name: crate::i2pcontrol::domain::tunnel::TunnelName::new(name).unwrap(),
            tunnel_type: TunnelType::Client,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: crate::i2pcontrol::domain::tunnel::StartIntent::DoNotStart,
            options: crate::i2pcontrol::domain::tunnel::TunnelOptions {
                target_destination: Some("destination".to_string()),
                listen_port: Some(0),
                ..Default::default()
            },
            raw_config: Default::default(),
        }
    }

    #[test]
    fn unimplemented_client_options_fail_before_runtime_allocation() {
        let backend = ClientTunnelBackend::new(1);
        let mut def = definition("unsupported-client-option");
        def.options.access_list = Some("private-peer".to_owned());
        assert!(matches!(
            backend.config(&def),
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::Client,
                option
            }) if option == "AccessList"
        ));

        let mut raw = definition("unsupported-client-raw");
        raw.raw_config
            .insert("i2p.tunnel.future".to_owned(), serde_json::json!("secret"));
        assert!(matches!(
            backend.config(&raw),
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::Client,
                option
            }) if option == "i2p.tunnel.future"
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
                            "SESSION STATUS DESTINATION=test-destination\n"
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
    async fn client_backend_requires_runtime_fields_before_allocating() {
        let backend = ClientTunnelBackend::new(1);
        let mut def = definition("missing-destination");
        def.options.target_destination = None;

        let result = backend.start(&def).await;
        assert!(matches!(
            result,
            Err(BackendError::MissingOption {
                tunnel_type: TunnelType::Client,
                option
            }) if option == "TargetDestination"
        ));
        assert_eq!(
            backend.inspect(&def).runtime_state,
            TunnelRuntimeState::Stopped
        );
    }

    #[tokio::test]
    async fn client_lifecycle_is_named_cancellable_and_restartable() {
        let (sam_port, sam_task) = fake_sam().await;
        let backend = Arc::new(ClientTunnelBackend::new(sam_port));
        let def = definition("client-lifecycle");

        backend.start(&def).await.unwrap();
        assert_eq!(
            backend.inspect(&def).runtime_state,
            TunnelRuntimeState::Running
        );
        assert!(matches!(
            backend.start(&def).await,
            Err(BackendError::InvalidState { .. })
        ));

        backend.stop(&def).await.unwrap();
        assert_eq!(
            backend.inspect(&def).runtime_state,
            TunnelRuntimeState::Stopped
        );
        backend.stop(&def).await.unwrap();

        backend.start(&def).await.unwrap();
        backend.stop(&def).await.unwrap();
        sam_task.abort();
    }

    #[tokio::test]
    async fn client_bind_failure_releases_runtime_slot() {
        let (sam_port, sam_task) = fake_sam().await;
        let backend = ClientTunnelBackend::new(sam_port);
        let occupied = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = occupied.local_addr().unwrap().port();
        let mut def = definition("bind-failure");
        def.options.listen_port = Some(port);

        let result = backend.start(&def).await;
        assert!(matches!(result, Err(BackendError::Internal { .. })));
        drop(occupied);

        backend.start(&def).await.unwrap();
        backend.stop(&def).await.unwrap();
        sam_task.abort();
    }

    #[tokio::test]
    async fn client_failure_isolated_by_exact_name() {
        let (sam_port, sam_task) = fake_sam().await;
        let backend = ClientTunnelBackend::new(sam_port);
        let first = definition("first");
        let second = definition("second");

        backend.start(&first).await.unwrap();
        backend.start(&second).await.unwrap();
        backend.stop(&first).await.unwrap();
        assert_eq!(
            backend.inspect(&second).runtime_state,
            TunnelRuntimeState::Running
        );
        backend.stop(&second).await.unwrap();
        sam_task.abort();
    }
}
