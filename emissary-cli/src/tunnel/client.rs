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

use crate::config::{ClientTunnelConfig, ClientTunnelOptions};

use tokio::{net::TcpListener, task::JoinSet};
use yosemite::{style, Session, SessionOptions, StreamOptions};

use std::{
    collections::BTreeMap,
    future::Future,
    sync::{Arc, RwLock},
    time::Duration,
};

/// Neutral lifecycle state for a startup-owned tunnel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupTunnelState {
    /// The runtime is being created.
    Starting,
    /// The runtime has completed readiness.
    Running,
    /// Cancellation has been requested and cleanup is in progress.
    Stopping,
    /// No runtime generation is active.
    Stopped,
    /// The most recent generation failed.
    Failed,
}

/// Runtime lifecycle operations understood by the neutral startup owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupTunnelAction {
    /// Start a generation if none is active.
    Start,
    /// Stop the active generation.
    Stop,
    /// Stop the active generation and start a successor.
    Restart,
}

/// A runtime owner registered for one exact startup tunnel name.
pub trait StartupTunnelController: Send + Sync {
    /// Apply a lifecycle operation.
    fn apply<'a>(
        &'a self,
        action: StartupTunnelAction,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;
    /// Return the current request-time state.
    fn state(&self) -> StartupTunnelState;
}

/// Bounded, name-indexed lifecycle handle shared by startup tunnel managers
/// and the I2PControl adapter. It contains controllers, never tunnel secrets.
#[derive(Clone, Default)]
pub struct StartupTunnelLifecycleHandle {
    controllers: Arc<RwLock<BTreeMap<String, Arc<dyn StartupTunnelController>>>>,
}

impl StartupTunnelLifecycleHandle {
    /// Create an empty startup lifecycle handle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one exact startup name. Duplicate names fail deterministically.
    pub fn register(
        &self,
        name: impl Into<String>,
        controller: Arc<dyn StartupTunnelController>,
    ) -> Result<(), String> {
        let name = name.into();
        let mut controllers = self
            .controllers
            .write()
            .map_err(|_| "startup lifecycle lock poisoned".to_string())?;
        if controllers.len() >= MAX_STARTUP_TUNNELS {
            return Err(format!(
                "startup lifecycle exceeds maximum of {MAX_STARTUP_TUNNELS} entries"
            ));
        }
        if controllers.insert(name, controller).is_some() {
            return Err("duplicate startup tunnel lifecycle name".to_string());
        }
        Ok(())
    }

    /// Return registered names in deterministic order.
    pub fn names(&self) -> Result<Vec<String>, String> {
        let controllers = self
            .controllers
            .read()
            .map_err(|_| "startup lifecycle lock poisoned".to_string())?;
        Ok(controllers.keys().cloned().collect())
    }

    /// Return one exact name's state.
    pub fn state(&self, name: &str) -> Result<Option<StartupTunnelState>, String> {
        let controller = self
            .controllers
            .read()
            .map_err(|_| "startup lifecycle lock poisoned".to_string())?
            .get(name)
            .cloned();
        Ok(controller.map(|controller| controller.state()))
    }

    /// Apply an operation to one exact name.
    pub async fn apply(&self, name: &str, action: StartupTunnelAction) -> Result<(), String> {
        let controller = self
            .controllers
            .read()
            .map_err(|_| "startup lifecycle lock poisoned".to_string())?
            .get(name)
            .cloned()
            .ok_or_else(|| format!("startup tunnel '{}' not found", name))?;
        controller.apply(action).await
    }

    /// Start every registered controller in deterministic order.
    pub async fn start_all(&self) {
        let names = self.names().unwrap_or_default();
        for name in names {
            if let Err(error) = self.apply(&name, StartupTunnelAction::Start).await {
                tracing::warn!(target: LOG_TARGET, %name, %error, "startup tunnel failed to start");
            }
        }
    }
}

const MAX_STARTUP_TUNNELS: usize = 1000;

struct ClientTunnelRuntimeState {
    state: StartupTunnelState,
    generation: u64,
    cancellation: Option<tokio::sync::watch::Sender<bool>>,
    task: Option<tokio::task::JoinHandle<()>>,
}

/// Lifecycle controller for one startup client tunnel.
pub struct ClientTunnelLifecycleController {
    config: ClientTunnelRuntimeConfig,
    lease_set_enc_type: Option<String>,
    shared_session: Option<Arc<tokio::sync::OnceCell<SharedClientSession>>>,
    state: Arc<tokio::sync::Mutex<ClientTunnelRuntimeState>>,
    operation: Arc<tokio::sync::Mutex<()>>,
}

impl ClientTunnelLifecycleController {
    /// Create a controller without exposing its runtime state or config.
    pub fn new(config: ClientTunnelRuntimeConfig, lease_set_enc_type: Option<String>) -> Self {
        Self::new_inner(config, lease_set_enc_type, None)
    }

    fn new_with_shared_session(
        config: ClientTunnelRuntimeConfig,
        lease_set_enc_type: Option<String>,
        shared_session: Arc<tokio::sync::OnceCell<SharedClientSession>>,
    ) -> Self {
        Self::new_inner(config, lease_set_enc_type, Some(shared_session))
    }

    fn new_inner(
        config: ClientTunnelRuntimeConfig,
        lease_set_enc_type: Option<String>,
        shared_session: Option<Arc<tokio::sync::OnceCell<SharedClientSession>>>,
    ) -> Self {
        Self {
            config,
            lease_set_enc_type,
            shared_session,
            state: Arc::new(tokio::sync::Mutex::new(ClientTunnelRuntimeState {
                state: StartupTunnelState::Stopped,
                generation: 0,
                cancellation: None,
                task: None,
            })),
            operation: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    async fn stop_generation(&self) -> Result<(), String> {
        let (cancellation, task) = {
            let mut state = self.state.lock().await;
            if state.state == StartupTunnelState::Stopped {
                return Ok(());
            }
            state.state = StartupTunnelState::Stopping;
            (state.cancellation.take(), state.task.take())
        };
        if let Some(cancellation) = cancellation {
            let _ = cancellation.send(true);
        }
        if let Some(mut task) = task {
            tokio::select! {
                result = &mut task => match result {
                    Ok(()) => {}
                    Err(_) => return self.mark_failed("client runtime task panicked").await,
                },
                _ = tokio::time::sleep(STARTUP_LIFECYCLE_TIMEOUT) => {
                    task.abort();
                    let _ = task.await;
                    return self.mark_failed("client tunnel stop timed out").await;
                }
            }
        }
        let mut state = self.state.lock().await;
        state.state = StartupTunnelState::Stopped;
        Ok(())
    }

    async fn mark_failed(&self, message: &str) -> Result<(), String> {
        self.state.lock().await.state = StartupTunnelState::Failed;
        Err(message.to_string())
    }
}

impl StartupTunnelController for ClientTunnelLifecycleController {
    fn apply<'a>(
        &'a self,
        action: StartupTunnelAction,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
        Box::pin(async move {
            let _operation = self.operation.lock().await;
            self.apply_inner(action).await
        })
    }

    fn state(&self) -> StartupTunnelState {
        self.state
            .try_lock()
            .map(|state| state.state)
            .unwrap_or(StartupTunnelState::Starting)
    }
}

impl ClientTunnelLifecycleController {
    async fn apply_inner(&self, action: StartupTunnelAction) -> Result<(), String> {
        if matches!(
            action,
            StartupTunnelAction::Stop | StartupTunnelAction::Restart
        ) {
            self.stop_generation().await?;
        }
        if matches!(action, StartupTunnelAction::Stop) {
            return Ok(());
        }

        let (cancellation, ready_receiver, generation) = {
            let mut state = self.state.lock().await;
            if state.state == StartupTunnelState::Running {
                return Ok(());
            }
            if state.state == StartupTunnelState::Starting {
                return Err("startup tunnel is already starting".to_string());
            }
            if state.state == StartupTunnelState::Failed {
                if state.task.as_ref().is_some_and(|task| !task.is_finished()) {
                    return Err("startup tunnel cleanup is still in progress".to_string());
                }
                state.task = None;
            }
            state.generation = state.generation.wrapping_add(1);
            state.state = StartupTunnelState::Starting;
            let (cancellation, _) = tokio::sync::watch::channel(false);
            let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
            let config = self.config.clone();
            let lease_set_enc_type = self.lease_set_enc_type.clone();
            let shared_session = self
                .shared_session
                .as_ref()
                .and_then(|session| session.get().cloned());
            if self.shared_session.is_some() && shared_session.is_none() {
                state.state = StartupTunnelState::Failed;
                return Err("shared client session is not ready".to_string());
            }
            let state_ref = Arc::clone(&self.state);
            let generation = state.generation;
            let task_cancellation = cancellation.clone();
            let task = tokio::spawn(async move {
                let result = run_single_client_inner(
                    config,
                    task_cancellation.subscribe(),
                    ready_sender,
                    lease_set_enc_type,
                    shared_session,
                )
                .await;
                let mut state = state_ref.lock().await;
                if state.generation == generation && state.state != StartupTunnelState::Failed {
                    state.state = if result.is_ok() {
                        StartupTunnelState::Stopped
                    } else {
                        StartupTunnelState::Failed
                    };
                    state.cancellation = None;
                }
            });
            state.cancellation = Some(cancellation.clone());
            state.task = Some(task);
            (cancellation, ready_receiver, generation)
        };

        match tokio::time::timeout(STARTUP_LIFECYCLE_TIMEOUT, ready_receiver).await {
            Ok(Ok(Ok(()))) => {
                let mut state = self.state.lock().await;
                if state.generation == generation {
                    state.state = StartupTunnelState::Running;
                }
                Ok(())
            }
            Ok(Ok(Err(error))) => self.mark_failed(&error).await,
            Ok(Err(_)) => self.mark_failed("client tunnel failed before readiness").await,
            Err(_) => {
                let _ = cancellation.send(true);
                self.mark_failed("client tunnel readiness timed out").await
            }
        }
    }
}

const STARTUP_LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(30);

/// Errors returned by the reusable client runtime primitive.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ClientRuntimeError {
    /// Local listener or stream I/O failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Yosemite SAM operation failed.
    #[error("Yosemite error")]
    Yosemite(#[from] yosemite::Error),
    /// The runtime task panicked.
    #[error("client runtime task panicked")]
    Panicked,
}

/// Plain runtime configuration for one generic client tunnel.
///
/// This type deliberately contains no I2PControl or persistence concepts so
/// the startup manager and the administrative runtime adapter can share the
/// same data-plane primitive.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClientTunnelRuntimeConfig {
    /// Diagnostic nickname only; it is not used as a protocol identity.
    pub name: String,
    /// Local listener interface. `None` uses loopback.
    pub address: Option<String>,
    /// Local listener port.
    pub port: u16,
    /// Remote I2P destination.
    pub destination: String,
    /// Remote destination port. Zero preserves Yosemite's existing default.
    pub destination_port: Option<u16>,
    /// SAMv3 TCP endpoint port.
    pub sam_tcp_port: u16,
}

type SharedClientSession = Arc<tokio::sync::Mutex<Session<style::Stream>>>;

/// Run one cancellable generic client tunnel.
///
/// Readiness is reported only after the local listener and Yosemite streaming
/// session have both been established. Controlled startup clients may share
/// the manager's single session. Traffic failures
/// retain the startup manager's bounded retry behavior. Bind and session
/// setup failures are terminal for this instance so a caller can release its
/// named runtime reservation and report a truthful failure.
#[allow(dead_code)]
pub async fn run_single_client(
    config: ClientTunnelRuntimeConfig,
    cancellation: tokio::sync::watch::Receiver<bool>,
    ready: tokio::sync::oneshot::Sender<Result<(), String>>,
) -> std::result::Result<(), ClientRuntimeError> {
    run_single_client_inner(config, cancellation, ready, None, None).await
}

async fn run_single_client_inner(
    config: ClientTunnelRuntimeConfig,
    mut cancellation: tokio::sync::watch::Receiver<bool>,
    ready: tokio::sync::oneshot::Sender<Result<(), String>>,
    lease_set_enc_type: Option<String>,
    shared_session: Option<SharedClientSession>,
) -> std::result::Result<(), ClientRuntimeError> {
    let address = config.address.clone().unwrap_or_else(|| "127.0.0.1".to_string());
    let listener = match TcpListener::bind(format!("{address}:{}", config.port)).await {
        Ok(listener) => listener,
        Err(error) => {
            let _ = ready.send(Err("client tunnel listener bind failed".to_string()));
            return Err(error.into());
        }
    };

    let session = if let Some(shared_session) = shared_session {
        Some(shared_session)
    } else {
        let session = tokio::select! {
            _ = cancellation.changed() => {
                let _ = ready.send(Err("client tunnel start cancelled".to_string()));
                return Ok(());
            }
            result = Session::<style::Stream>::new(SessionOptions {
                publish: false,
                samv3_tcp_port: config.sam_tcp_port,
                nickname: format!("i2p-tunnel-{}", config.name),
                inbound_quantity: 4,
                outbound_quantity: 4,
                lease_set_enc_type,
                ..Default::default()
            }) => result,
        }?;
        Some(Arc::new(tokio::sync::Mutex::new(session)))
    };

    let _ = ready.send(Ok(()));
    loop {
        let (mut tcp_stream, _) = tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            result = listener.accept() => result?,
        };

        let mut i2p_stream = tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            result = async {
                let stream = {
                    let session = session.as_ref().expect("client session is present");
                    let mut session = session.lock().await;
                    session.connect_detached_with_options(
                        &config.destination,
                        StreamOptions {
                            dst_port: config.destination_port.unwrap_or(0),
                            ..Default::default()
                        },
                    )
                };
                stream.await
            } => match result {
                Ok(stream) => stream,
                Err(error) => {
                    tracing::debug!(
                        target: LOG_TARGET,
                        name = %config.name,
                        ?error,
                        "client tunnel connection failed",
                    );
                    tokio::select! {
                        _ = cancellation.changed() => return Ok(()),
                        _ = tokio::time::sleep(RETRY_TIMEOUT) => continue,
                    }
                }
            },
        };

        let copy_result = tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            result = tokio::io::copy_bidirectional(&mut i2p_stream, &mut tcp_stream) => result,
        };

        if let Err(error) = copy_result {
            tracing::debug!(
                target: LOG_TARGET,
                name = %config.name,
                ?error,
                "client tunnel traffic path failed",
            );
        }

        tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            _ = tokio::time::sleep(RETRY_TIMEOUT) => {},
        }
    }
}

/// Logging target for the file.
const LOG_TARGET: &str = "emissary::client-tunnel";

/// Retry timeout.
const RETRY_TIMEOUT: Duration = Duration::from_secs(15);

/// Client tunnel manager.
pub struct ClientTunnelManager {
    /// Tunnel futures.
    futures: JoinSet<Arc<ClientTunnelConfig>>,

    /// SAMv3 server port of the router.
    sam_tcp_port: u16,

    /// Client tunnel configurations.
    tunnels: Vec<Arc<ClientTunnelConfig>>,

    /// Client tunnel options.
    client_tunnel_options: Option<ClientTunnelOptions>,

    /// Optional lifecycle owner used by the I2PControl composition seam.
    lifecycle: Option<StartupTunnelLifecycleHandle>,

    /// Shared Yosemite client session for controlled startup tunnels.
    shared_session: Option<Arc<tokio::sync::OnceCell<SharedClientSession>>>,
}

impl ClientTunnelManager {
    /// Create new [`ClientTunnelManager`].
    pub fn new(
        tunnels: Vec<ClientTunnelConfig>,
        client_tunnel_options: Option<ClientTunnelOptions>,
        sam_tcp_port: u16,
    ) -> Self {
        Self {
            client_tunnel_options,
            futures: JoinSet::new(),
            sam_tcp_port,
            tunnels: tunnels.into_iter().map(Arc::from).collect(),
            lifecycle: None,
            shared_session: None,
        }
    }

    /// Create a manager whose startup definitions are lifecycle-controlled.
    pub fn new_with_lifecycle(
        tunnels: Vec<ClientTunnelConfig>,
        client_tunnel_options: Option<ClientTunnelOptions>,
        sam_tcp_port: u16,
        lifecycle: StartupTunnelLifecycleHandle,
    ) -> Result<Self, String> {
        let lease_set_enc_type = client_tunnel_options.as_ref().and_then(|options| {
            options.i2cp.as_ref().and_then(|i2cp| i2cp.lease_set_enc_type.clone())
        });
        let manager = Self::new(tunnels, client_tunnel_options, sam_tcp_port);
        let shared_session = Arc::new(tokio::sync::OnceCell::new());
        for tunnel in &manager.tunnels {
            lifecycle.register(
                tunnel.name.clone(),
                Arc::new(ClientTunnelLifecycleController::new_with_shared_session(
                    ClientTunnelRuntimeConfig {
                        name: tunnel.name.clone(),
                        address: tunnel.address.clone(),
                        port: tunnel.port,
                        destination: tunnel.destination.clone(),
                        destination_port: tunnel.destination_port,
                        sam_tcp_port,
                    },
                    lease_set_enc_type.clone(),
                    Arc::clone(&shared_session),
                )),
            )?;
        }
        Ok(Self {
            lifecycle: Some(lifecycle),
            shared_session: Some(shared_session),
            ..manager
        })
    }

    /// Run the event loop of a client tunnel.
    async fn tunnel_event_loop(
        future: impl Future<Output = yosemite::Result<yosemite::Stream>>,
        tunnel: &Arc<ClientTunnelConfig>,
    ) -> std::result::Result<(), ClientRuntimeError> {
        let listener = TcpListener::bind(format!(
            "{}:{}",
            tunnel.address.clone().unwrap_or(String::from("127.0.0.1")),
            tunnel.port
        ))
        .await?;

        let (mut tcp_stream, _) = listener.accept().await?;
        let mut i2p_stream = future.await?;

        tokio::io::copy_bidirectional(&mut i2p_stream, &mut tcp_stream).await?;

        Ok(())
    }

    /// Run the event loop of [`ClientTunnelManger`].
    ///
    /// If there are no client tunnels congigured, [`ClientTunnelManager`] exits immediately.
    pub async fn run(mut self) {
        if self.tunnels.is_empty() {
            return;
        }

        if let Some(lifecycle) = self.lifecycle.take() {
            let session = Session::<style::Stream>::new(SessionOptions {
                publish: false,
                samv3_tcp_port: self.sam_tcp_port,
                nickname: "i2p-tunnel".to_string(),
                inbound_quantity: 4,
                outbound_quantity: 4,
                lease_set_enc_type: self.client_tunnel_options.as_ref().and_then(|config| {
                    config.i2cp.as_ref().and_then(|config| config.lease_set_enc_type.clone())
                }),
                ..Default::default()
            })
            .await;
            match session {
                Ok(session) => {
                    if let Some(shared_session) = self.shared_session {
                        let _ = shared_session.set(Arc::new(tokio::sync::Mutex::new(session)));
                    }
                }
                Err(error) => {
                    tracing::error!(
                        target: LOG_TARGET,
                        ?error,
                        "failed to start controlled client tunnel session",
                    );
                }
            }
            lifecycle.start_all().await;
            return;
        }

        tracing::info!(
            target: LOG_TARGET,
            num_tunnels = ?self.tunnels.len(),
            "starting client tunnel manager",
        );

        let mut session = match Session::<style::Stream>::new(SessionOptions {
            publish: false,
            samv3_tcp_port: self.sam_tcp_port,
            nickname: "i2p-tunnel".to_string(),
            inbound_quantity: 4,
            outbound_quantity: 4,
            lease_set_enc_type: self.client_tunnel_options.and_then(|config| {
                config.i2cp.and_then(|config| config.lease_set_enc_type.clone())
            }),
            ..Default::default()
        })
        .await
        {
            Ok(session) => session,
            Err(error) => {
                tracing::error!(
                    target: LOG_TARGET,
                    ?error,
                    "failed to start client tunnel manager",
                );
                return;
            }
        };

        for tunnel in self.tunnels.iter().cloned() {
            let future = session.connect_detached_with_options(
                &tunnel.destination,
                StreamOptions {
                    dst_port: tunnel.destination_port.unwrap_or(0),
                    ..Default::default()
                },
            );

            self.futures.spawn(async move {
                match Self::tunnel_event_loop(future, &tunnel).await {
                    Ok(()) => tunnel,
                    Err(error) => {
                        tracing::debug!(
                            target: LOG_TARGET,
                            name = %tunnel.name,
                            ?error,
                            "client tunnel exited with error",
                        );

                        tokio::time::sleep(RETRY_TIMEOUT).await;
                        tunnel
                    }
                }
            });
        }

        while let Some(result) = self.futures.join_next().await {
            match result {
                Err(error) => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        ?error,
                        "client tunnel panicked, unable to restart",
                    );
                    debug_assert!(false);
                }
                Ok(tunnel) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        name = %tunnel.name,
                        "tunnel returned, restart event loop"
                    );

                    let future = session.connect_detached_with_options(
                        &tunnel.destination,
                        StreamOptions {
                            dst_port: tunnel.destination_port.unwrap_or(0),
                            ..Default::default()
                        },
                    );

                    self.futures.spawn(async move {
                        match Self::tunnel_event_loop(future, &tunnel).await {
                            Ok(()) => tunnel,
                            Err(error) => {
                                tracing::debug!(
                                    target: LOG_TARGET,
                                    name = %tunnel.name,
                                    ?error,
                                    "client tunnel exited with error",
                                );

                                tokio::time::sleep(RETRY_TIMEOUT).await;
                                tunnel
                            }
                        }
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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
    async fn startup_client_lifecycle_is_generation_safe_and_restartable() {
        let (sam_port, sam_task) = fake_sam().await;
        let controller = ClientTunnelLifecycleController::new(
            ClientTunnelRuntimeConfig {
                name: "startup-client".to_string(),
                address: Some("127.0.0.1".to_string()),
                port: 0,
                destination: "remote-destination".to_string(),
                destination_port: None,
                sam_tcp_port: sam_port,
            },
            None,
        );

        assert_eq!(controller.state(), StartupTunnelState::Stopped);
        controller.apply(StartupTunnelAction::Start).await.unwrap();
        assert_eq!(controller.state(), StartupTunnelState::Running);
        controller.apply(StartupTunnelAction::Stop).await.unwrap();
        assert_eq!(controller.state(), StartupTunnelState::Stopped);
        controller.apply(StartupTunnelAction::Restart).await.unwrap();
        assert_eq!(controller.state(), StartupTunnelState::Running);
        controller.apply(StartupTunnelAction::Stop).await.unwrap();
        assert_eq!(controller.state(), StartupTunnelState::Stopped);

        sam_task.abort();
    }
}
