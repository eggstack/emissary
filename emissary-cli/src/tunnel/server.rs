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

#![allow(dead_code)]

use crate::{
    config::{I2cpOptions, ServerTunnelConfig},
    tunnel_client::{
        StartupTunnelAction, StartupTunnelController, StartupTunnelState,
        StartupTunnelStateSnapshot,
    },
};

use yosemite::{style, DestinationKind, RouterApi, Session, SessionOptions};

use std::{path::PathBuf, sync::Arc, time::Duration};

/// Errors returned by the reusable single-server runtime primitive.
///
/// The error intentionally contains no SAM/Yosemite detail. A persistent
/// destination is part of the session setup input and must never be copied
/// into an error or diagnostic value.
#[derive(Debug, thiserror::Error)]
pub enum ServerRuntimeError {
    /// The SAM session could not be created.
    #[error("server tunnel session setup failed")]
    SessionSetup,
    /// The runtime task panicked.
    #[error("server tunnel runtime task panicked")]
    Panicked,
}

/// Passive callback used only to publish the actual destination returned by
/// an existing server session into the composed startup inventory.
pub type DestinationObserver = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Plain runtime configuration for one server tunnel.
///
/// The destination is private session material. This type deliberately has no
/// `Debug` implementation so accidental diagnostics cannot print it.
#[derive(Clone)]
pub struct ServerTunnelRuntimeConfig {
    /// Diagnostic session nickname.
    pub name: String,
    /// Local TCP port receiving forwarded I2P streams.
    pub port: u16,
    /// Persistent destination private key material.
    pub destination: String,
    /// SAMv3 TCP port.
    pub sam_tcp_port: u16,
    /// Optional I2CP lease-set encryption type.
    pub lease_set_enc_type: Option<String>,
}

/// Generate one persistent destination through the existing router SAM API.
///
/// This is a purpose-specific data-plane helper. It does not know about
/// I2PControl, tunnel definitions, or filesystem paths.
pub async fn generate_persistent_destination(
    sam_tcp_port: u16,
) -> Result<String, ServerRuntimeError> {
    let router_api = RouterApi::new(sam_tcp_port);
    for attempt in 0..DESTINATION_CREATION_RETRY_COUNT {
        match router_api.generate_destination().await {
            Ok((_, private_key)) => return Ok(private_key),
            Err(_) if attempt + 1 < DESTINATION_CREATION_RETRY_COUNT => {
                tokio::time::sleep(DESTINATION_CREATION_BACKOFF).await;
            }
            Err(_) => break,
        }
    }
    Err(ServerRuntimeError::SessionSetup)
}

/// Run one cancellable generic server tunnel.
///
/// Readiness is reported only after the persistent session has published its
/// actual public destination and `STREAM FORWARD` has succeeded. Forward
/// failures retain the startup manager's bounded retry behavior. Cancellation
/// is observed during session setup, forward retry, and the idle lifetime.
pub async fn run_single_server(
    config: ServerTunnelRuntimeConfig,
    mut cancellation: tokio::sync::watch::Receiver<bool>,
    ready: tokio::sync::oneshot::Sender<Result<(), String>>,
    destination_observer: Option<DestinationObserver>,
) -> Result<(), ServerRuntimeError> {
    let mut session = tokio::select! {
        _ = cancellation.changed() => {
            let _ = ready.send(Err("server tunnel start cancelled".to_string()));
            return Ok(());
        }
        result = Session::<style::Stream>::new(SessionOptions {
            samv3_tcp_port: config.sam_tcp_port,
            nickname: config.name.clone(),
            silent_forward: true,
            destination: DestinationKind::Persistent {
                private_key: config.destination.clone(),
            },
            lease_set_enc_type: config.lease_set_enc_type,
            ..Default::default()
        }) => match result {
            Ok(session) => session,
            Err(_) => {
                let _ = ready.send(Err("server tunnel session setup failed".to_string()));
                return Err(ServerRuntimeError::SessionSetup);
            }
        },
    };

    if let Some(observer) = destination_observer.as_ref() {
        observer(&config.name, session.destination());
    }

    loop {
        let forward = tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            result = session.forward(config.port) => result,
        };
        if forward.is_ok() {
            break;
        }

        tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            _ = tokio::time::sleep(STREAM_FORWARD_BACKOFF) => {},
        }
    }

    let _ = ready.send(Ok(()));
    loop {
        tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            _ = tokio::time::sleep(Duration::from_secs(10)) => {},
        }
    }
}

struct ServerTunnelRuntimeState {
    state: StartupTunnelState,
    generation: u64,
    cancellation: Option<tokio::sync::watch::Sender<bool>>,
    task: Option<tokio::task::JoinHandle<()>>,
}

/// Lifecycle controller for one startup server tunnel. The private
/// destination remains inside this controller and is never returned by the
/// neutral lifecycle API.
pub struct ServerTunnelLifecycleController {
    config: ServerTunnelRuntimeConfig,
    destination_observer: Option<DestinationObserver>,
    state: Arc<tokio::sync::Mutex<ServerTunnelRuntimeState>>,
    state_snapshot: Arc<StartupTunnelStateSnapshot>,
    operation: Arc<tokio::sync::Mutex<()>>,
}

impl ServerTunnelLifecycleController {
    /// Create a controller for one already-loaded startup server definition.
    pub fn new(
        config: ServerTunnelRuntimeConfig,
        destination_observer: Option<DestinationObserver>,
    ) -> Self {
        Self {
            config,
            destination_observer,
            state: Arc::new(tokio::sync::Mutex::new(ServerTunnelRuntimeState {
                state: StartupTunnelState::Stopped,
                generation: 0,
                cancellation: None,
                task: None,
            })),
            state_snapshot: Arc::new(StartupTunnelStateSnapshot::new(StartupTunnelState::Stopped)),
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
            self.state_snapshot.store(StartupTunnelState::Stopping);
            (state.cancellation.take(), state.task.take())
        };
        if let Some(cancellation) = cancellation {
            let _ = cancellation.send(true);
        }
        if let Some(mut task) = task {
            tokio::select! {
                result = &mut task => match result {
                    Ok(()) => {}
                    Err(_) => return self.mark_failed("server runtime task panicked").await,
                },
                _ = tokio::time::sleep(STARTUP_LIFECYCLE_TIMEOUT) => {
                    task.abort();
                    let _ = task.await;
                    return self.mark_failed("server tunnel stop timed out").await;
                }
            }
        }
        self.state.lock().await.state = StartupTunnelState::Stopped;
        self.state_snapshot.store(StartupTunnelState::Stopped);
        Ok(())
    }

    async fn mark_failed(&self, message: &str) -> Result<(), String> {
        let mut state = self.state.lock().await;
        state.state = StartupTunnelState::Failed;
        self.state_snapshot.store(StartupTunnelState::Failed);
        Err(message.to_string())
    }
}

impl StartupTunnelController for ServerTunnelLifecycleController {
    fn apply<'a>(
        &'a self,
        action: StartupTunnelAction,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
        Box::pin(async move {
            let _operation = self.operation.lock().await;
            self.apply_inner(action).await
        })
    }

    fn state(&self) -> StartupTunnelState {
        self.state_snapshot.load()
    }
}

impl ServerTunnelLifecycleController {
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
            self.state_snapshot.store(StartupTunnelState::Starting);
            let (cancellation, _) = tokio::sync::watch::channel(false);
            let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
            let config = self.config.clone();
            let observer = self.destination_observer.clone();
            let state_ref = Arc::clone(&self.state);
            let state_snapshot = Arc::clone(&self.state_snapshot);
            let generation = state.generation;
            let task_cancellation = cancellation.clone();
            let task = tokio::spawn(async move {
                let result = run_single_server(
                    config,
                    task_cancellation.subscribe(),
                    ready_sender,
                    observer,
                )
                .await;
                let mut state = state_ref.lock().await;
                if state.generation == generation && state.state != StartupTunnelState::Failed {
                    let next_state = if result.is_ok() {
                        StartupTunnelState::Stopped
                    } else {
                        StartupTunnelState::Failed
                    };
                    state.state = next_state;
                    state_snapshot.store(next_state);
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
                    self.state_snapshot.store(StartupTunnelState::Running);
                }
                Ok(())
            }
            Ok(Ok(Err(error))) => self.mark_failed(&error).await,
            Ok(Err(_)) => self.mark_failed("server tunnel failed before readiness").await,
            Err(_) => {
                let _ = cancellation.send(true);
                self.mark_failed("server tunnel readiness timed out").await
            }
        }
    }
}

const STARTUP_LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(30);

/// Logging target for the file.
const LOG_TARGET: &str = "emissary::server-tunnel";

/// Number of destination generation retries.
const DESTINATION_CREATION_RETRY_COUNT: usize = 3usize;

/// Destination generation failure backoff.
const DESTINATION_CREATION_BACKOFF: Duration = Duration::from_secs(10);

/// Backoff for `STREAM FORWARD` failure.
const STREAM_FORWARD_BACKOFF: Duration = Duration::from_secs(10);

/// Server tunnel configuration
pub struct TunnelConfig {
    /// Base64 destination.
    destination: String,

    /// I2CP options.
    i2cp: Option<I2cpOptions>,

    /// Name of the tunnel.
    name: String,

    /// Server port.
    port: u16,

    /// SAMv3 TCP port.
    sam_tcp_port: u16,
}

/// Server tunnel manager.
pub struct ServerTunnelManager {
    /// Server tunnels.
    tunnels: Vec<Arc<TunnelConfig>>,

    /// Optional passive destination publication callback.
    destination_observer: Option<DestinationObserver>,

    /// Optional lifecycle owner used by the I2PControl composition seam.
    lifecycle: Option<crate::tunnel_client::StartupTunnelLifecycleHandle>,
}

impl ServerTunnelManager {
    /// Create new [`ServerTunnelManager`].
    pub async fn new(
        configs: Vec<ServerTunnelConfig>,
        sam_tcp_port: u16,
        base_path: PathBuf,
        destination_observer: Option<DestinationObserver>,
    ) -> Self {
        Self::new_inner(configs, sam_tcp_port, base_path, destination_observer, None)
            .await
            .expect("uncontrolled startup server manager construction cannot fail")
    }

    /// Create a server manager with neutral startup lifecycle control.
    pub async fn new_with_lifecycle(
        configs: Vec<ServerTunnelConfig>,
        sam_tcp_port: u16,
        base_path: PathBuf,
        destination_observer: Option<DestinationObserver>,
        lifecycle: crate::tunnel_client::StartupTunnelLifecycleHandle,
    ) -> Result<Self, String> {
        Self::new_inner(
            configs,
            sam_tcp_port,
            base_path,
            destination_observer,
            Some(lifecycle),
        )
        .await
    }

    async fn new_inner(
        configs: Vec<ServerTunnelConfig>,
        sam_tcp_port: u16,
        base_path: PathBuf,
        destination_observer: Option<DestinationObserver>,
        lifecycle: Option<crate::tunnel_client::StartupTunnelLifecycleHandle>,
    ) -> Result<Self, String> {
        let mut tunnels = Vec::<Arc<TunnelConfig>>::new();
        let mut router_api = RouterApi::new(sam_tcp_port);

        for ServerTunnelConfig {
            name,
            port,
            destination_path,
            i2cp,
            ..
        } in configs
        {
            match Self::load_or_create_destination(
                &mut router_api,
                base_path.join(&destination_path),
            )
            .await
            {
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        %name,
                        %destination_path,
                        "failed to load or create destination for server tunnel",
                    );
                    continue;
                }
                Some(destination) => {
                    tunnels.push(Arc::from(TunnelConfig {
                        destination,
                        name,
                        port,
                        sam_tcp_port,
                        i2cp: i2cp.clone(),
                    }));
                }
            }
        }

        if let Some(lifecycle) = lifecycle.as_ref() {
            for tunnel in &tunnels {
                lifecycle.register(
                    tunnel.name.clone(),
                    Arc::new(ServerTunnelLifecycleController::new(
                        ServerTunnelRuntimeConfig {
                            name: tunnel.name.clone(),
                            port: tunnel.port,
                            destination: tunnel.destination.clone(),
                            sam_tcp_port: tunnel.sam_tcp_port,
                            lease_set_enc_type: tunnel
                                .i2cp
                                .as_ref()
                                .and_then(|options| options.lease_set_enc_type.clone()),
                        },
                        destination_observer.clone(),
                    )),
                )?;
            }
        }

        Ok(Self {
            tunnels,
            destination_observer,
            lifecycle,
        })
    }

    /// Attempt to load destination from `path` and if it does't exist, call router over SAMv3 to
    /// create new persistent destination.
    ///
    /// Destination generation is attempted three times before bailing out.
    async fn load_or_create_destination(
        router_api: &mut RouterApi,
        path: PathBuf,
    ) -> Option<String> {
        if let Some(destination) = tokio::fs::read(&path).await.ok().and_then(|contents| {
            std::str::from_utf8(&contents).ok().map(|destination| destination.to_string())
        }) {
            return Some(destination);
        };

        tracing::debug!(
            target: LOG_TARGET,
            ?path,
            "destination not found from disk, create new destination",
        );

        for _ in 0..DESTINATION_CREATION_RETRY_COUNT {
            match router_api.generate_destination().await {
                Ok((_, private_key)) => {
                    if let Err(error) = tokio::fs::write(&path, private_key.as_bytes()).await {
                        tracing::warn!(
                            target: LOG_TARGET,
                            ?path,
                            ?error,
                            "failed to write destination to disk",
                        );
                    }

                    return Some(private_key);
                }
                Err(error) => {
                    tracing::debug!(
                        target: LOG_TARGET,
                        ?path,
                        ?error,
                        "failed to generate destination",
                    );
                    tokio::time::sleep(DESTINATION_CREATION_BACKOFF).await;
                }
            }
        }

        tracing::warn!(
            target: LOG_TARGET,
            ?path,
            retry_count = ?DESTINATION_CREATION_RETRY_COUNT,
            "failed to generate destination after multiple retries",
        );

        None
    }

    /// Run the event loop of server tunnel.
    async fn server_event_loop(
        config: Arc<TunnelConfig>,
        destination_observer: Option<DestinationObserver>,
    ) {
        tracing::info!(
            target: LOG_TARGET,
            name = %config.name,
            port = %config.port,
            "starting server tunnel",
        );

        // The startup manager has no administrative stop handle. Keep the sender alive for the
        // whole runtime invocation so its receiver does not interpret sender drop as cancellation.
        let (_cancellation_keepalive, cancellation) = tokio::sync::watch::channel(false);
        let (ready, _ready_result) = tokio::sync::oneshot::channel();
        let result = run_single_server(
            ServerTunnelRuntimeConfig {
                name: config.name.clone(),
                port: config.port,
                destination: config.destination.clone(),
                sam_tcp_port: config.sam_tcp_port,
                lease_set_enc_type: config
                    .i2cp
                    .as_ref()
                    .and_then(|i2cp| i2cp.lease_set_enc_type.clone()),
            },
            cancellation,
            ready,
            destination_observer,
        )
        .await;
        if result.is_err() {
            tracing::error!(
                target: LOG_TARGET,
                name = %config.name,
                "failed to start server tunnel runtime",
            );
        }
    }

    /// Run the event loop of [`ServerTunnelManager`].
    pub async fn run(self) {
        if self.tunnels.is_empty() {
            return;
        }

        if let Some(lifecycle) = self.lifecycle {
            lifecycle.start_all().await;
            return;
        }

        for tunnel in self.tunnels {
            tokio::spawn(Self::server_event_loop(
                Arc::clone(&tunnel),
                self.destination_observer.clone(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    #[tokio::test]
    async fn startup_manager_reaches_forward_and_keeps_runtime_alive() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let sam_port = listener.local_addr().unwrap().port();
        let commands = Arc::new(std::sync::Mutex::new(Vec::new()));
        let command_notify = Arc::new(tokio::sync::Notify::new());
        let sam_commands = Arc::clone(&commands);
        let sam_notify = Arc::clone(&command_notify);
        let sam_task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let sam_commands = Arc::clone(&sam_commands);
                let sam_notify = Arc::clone(&sam_notify);
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            break;
                        }
                        sam_commands.lock().unwrap().push(line.trim_end().to_string());
                        sam_notify.notify_waiters();
                        let response = if line.starts_with("HELLO") {
                            "HELLO REPLY RESULT=OK VERSION=3.3\n"
                        } else if line.starts_with("SESSION CREATE") {
                            "SESSION STATUS DESTINATION=startup-destination\n"
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

        let (destination_sender, destination_receiver) = tokio::sync::oneshot::channel();
        let destination_sender = Arc::new(std::sync::Mutex::new(Some(destination_sender)));
        let observer: DestinationObserver = Arc::new(move |name, destination| {
            if let Some(sender) = destination_sender.lock().unwrap().take() {
                let _ = sender.send((name.to_string(), destination.to_string()));
            }
        });
        let config = Arc::new(TunnelConfig {
            destination: "startup-private-key".to_string(),
            i2cp: None,
            name: "startup-server".to_string(),
            port: 0,
            sam_tcp_port: sam_port,
        });
        let runtime = tokio::spawn(ServerTunnelManager::server_event_loop(
            config,
            Some(observer),
        ));

        let (name, destination) =
            tokio::time::timeout(Duration::from_secs(1), destination_receiver)
                .await
                .expect("startup server should reach SAM session setup")
                .expect("destination observer should be called");
        assert_eq!(name, "startup-server");
        assert_eq!(destination, "startup-destination");

        let forward_result = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if commands
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|command| command.starts_with("STREAM FORWARD"))
                {
                    break;
                }
                command_notify.notified().await;
            }
        })
        .await;
        assert!(
            forward_result.is_ok(),
            "startup server should reach STREAM FORWARD; commands: {:?}",
            commands.lock().unwrap()
        );

        assert!(
            !runtime.is_finished(),
            "startup runtime must remain owned and alive"
        );
        runtime.abort();
        sam_task.abort();
    }

    #[tokio::test]
    async fn startup_server_lifecycle_publishes_and_restarts_without_exposing_secret() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let sam_port = listener.local_addr().unwrap().port();
        let sam_task = tokio::spawn(async move {
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
                            "SESSION STATUS DESTINATION=public-destination\n"
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
        let observed = Arc::new(std::sync::Mutex::new(Vec::new()));
        let observed_for_callback = Arc::clone(&observed);
        let observer: DestinationObserver = Arc::new(move |name, destination| {
            observed_for_callback
                .lock()
                .unwrap()
                .push((name.to_string(), destination.to_string()));
        });
        let controller = ServerTunnelLifecycleController::new(
            ServerTunnelRuntimeConfig {
                name: "startup-server".to_string(),
                port: 0,
                destination: "private-destination-secret".to_string(),
                sam_tcp_port: sam_port,
                lease_set_enc_type: None,
            },
            Some(observer),
        );

        controller.apply(StartupTunnelAction::Start).await.unwrap();
        assert_eq!(controller.state(), StartupTunnelState::Running);
        controller.apply(StartupTunnelAction::Stop).await.unwrap();
        assert_eq!(controller.state(), StartupTunnelState::Stopped);
        controller.apply(StartupTunnelAction::Restart).await.unwrap();
        assert_eq!(controller.state(), StartupTunnelState::Running);
        controller.apply(StartupTunnelAction::Stop).await.unwrap();
        assert_eq!(controller.state(), StartupTunnelState::Stopped);
        assert_eq!(observed.lock().unwrap().len(), 2);
        assert!(observed
            .lock()
            .unwrap()
            .iter()
            .all(|(_, destination)| destination == "public-destination"));
        sam_task.abort();
    }

    #[tokio::test]
    async fn lifecycle_state_snapshot_is_truthful_while_state_lock_is_contended() {
        let controller = ServerTunnelLifecycleController::new(
            ServerTunnelRuntimeConfig {
                name: "contended-server".to_string(),
                port: 0,
                destination: "private-destination-secret".to_string(),
                sam_tcp_port: 1,
                lease_set_enc_type: None,
            },
            None,
        );
        let _state_guard = controller.state.lock().await;
        assert_eq!(controller.state(), StartupTunnelState::Stopped);
    }

    #[tokio::test]
    async fn closed_cancellation_sender_still_cancels_reusable_runtime() {
        let (cancellation_sender, cancellation) = tokio::sync::watch::channel(false);
        drop(cancellation_sender);
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        let result = run_single_server(
            ServerTunnelRuntimeConfig {
                name: "cancelled-server".to_string(),
                port: 0,
                destination: "unused-private-key".to_string(),
                sam_tcp_port: 1,
                lease_set_enc_type: None,
            },
            cancellation,
            ready_sender,
            None,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(
            ready_receiver.await.unwrap().unwrap_err(),
            "server tunnel start cancelled"
        );
    }
}
