//! Bounded Streamr producer/consumer datagram backends.
//!
//! Streamr deliberately has its own runtime rather than sharing the streaming
//! tunnel adapters. Yosemite exposes the authenticated remote destination for
//! each repliable datagram, but not inbound port metadata, so subscription
//! identity is the trusted destination and the configured session port tuple.

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{
    net::UdpSocket,
    sync::{oneshot, watch},
    task::JoinHandle,
};
use yosemite::{style, DatagramOptions, DestinationKind, Session, SessionOptions};

use super::{
    options::{
        validate_options, OptionValidationError, STREAMR_CLIENT_OPTIONS, STREAMR_SERVER_OPTIONS,
    },
    server::SERVER_IDENTITY_KEY,
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
    server_secret_store::{ServerDestinationStore, StoredDestination},
};

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const UNSUBSCRIBE_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_RUNTIME_TASKS: usize = 1000;

/// The largest datagram Yosemite 0.7 can receive into its internal buffer.
pub const MAX_TRANSPORT_PACKET: usize = 0xfff;
/// Application payload cap kept below the raw datagram transport ceiling.
pub const MAX_STREAMR_PAYLOAD: usize = 1200;
/// Maximum number of active consumers for one producer.
pub const MAX_SUBSCRIBERS: usize = 16;
/// Consumers must refresh before this interval expires.
pub const SUBSCRIPTION_EXPIRY: Duration = Duration::from_secs(60);
/// Refresh cadence is safely below the expiry window.
pub const SUBSCRIPTION_REFRESH: Duration = Duration::from_secs(15);
const EXPIRY_SCAN: Duration = Duration::from_secs(5);
const DEFAULT_BIND_ADDRESS: IpAddr = IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);

#[derive(Debug, Clone)]
struct StreamrClientConfig {
    name: String,
    producer: String,
    local_target: SocketAddr,
    source_port: u16,
    sam_tcp_port: u16,
}

#[derive(Debug, Clone)]
struct StreamrServerConfig {
    name: String,
    bind_address: IpAddr,
    local_port: u16,
    source_port: u16,
    destination_port: u16,
    sam_tcp_port: u16,
    destination: StoredDestination,
}

#[derive(Debug)]
struct RuntimeEntry {
    generation: u64,
    state: TunnelRuntimeState,
    cancellation: watch::Sender<bool>,
    task: Option<JoinHandle<()>>,
    failure: Option<&'static str>,
    destination: Option<String>,
}

#[derive(Debug, Default)]
struct RuntimeMap {
    next_generation: u64,
    entries: HashMap<String, RuntimeEntry>,
}

#[derive(Clone, Debug)]
struct StreamrRuntimeSupervisor {
    inner: Arc<Mutex<RuntimeMap>>,
    tunnel_type: TunnelType,
}

impl StreamrRuntimeSupervisor {
    fn new(tunnel_type: TunnelType) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeMap::default())),
            tunnel_type,
        }
    }

    fn reserve(&self, name: &str) -> BackendResult<(u64, watch::Receiver<bool>)> {
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
                    tunnel_type: self.tunnel_type,
                    current_state: entry.state,
                    attempted_action: "start",
                });
            }
        }
        if runtime.entries.values().filter(|entry| entry.task.is_some()).count()
            >= MAX_RUNTIME_TASKS
        {
            return Err(BackendError::Internal {
                message: format!("{} runtime capacity exhausted", self.tunnel_type),
            });
        }
        runtime.next_generation = runtime.next_generation.wrapping_add(1);
        let generation = runtime.next_generation;
        let (cancellation, receiver) = watch::channel(false);
        runtime.entries.insert(
            name.to_owned(),
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
                return;
            }
        }
        task.abort();
    }

    fn publish_destination(&self, name: &str, generation: u64, destination: &str) {
        let mut runtime = self.inner.lock();
        if let Some(entry) = runtime.entries.get_mut(name) {
            if entry.generation == generation && !destination.is_empty() {
                entry.destination = Some(destination.to_owned());
            }
        }
    }

    fn mark_running(&self, name: &str, generation: u64, requires_destination: bool) -> bool {
        let mut runtime = self.inner.lock();
        let Some(entry) = runtime.entries.get_mut(name) else {
            return false;
        };
        if entry.generation != generation
            || entry.task.is_none()
            || (requires_destination && entry.destination.is_none())
        {
            return false;
        }
        entry.state = TunnelRuntimeState::Running;
        true
    }

    fn complete(
        map: Arc<Mutex<RuntimeMap>>,
        name: String,
        generation: u64,
        result: bool,
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
        if cancelled || result {
            entry.state = TunnelRuntimeState::Stopped;
            entry.failure = None;
        } else {
            entry.state = TunnelRuntimeState::Failed;
            entry.failure = Some("streamr tunnel runtime failed");
        }
    }

    fn remove_generation(&self, name: &str, generation: u64) {
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
            self.remove_generation(name, generation);
            return Ok(());
        };
        let _ = cancellation.send(true);
        if tokio::time::timeout(STOP_TIMEOUT, &mut task).await.is_err() {
            task.abort();
            let _ = task.await;
            self.remove_generation(name, generation);
            return Err(BackendError::Internal {
                message: format!("{} tunnel stop timed out", self.tunnel_type),
            });
        }
        self.remove_generation(name, generation);
        Ok(())
    }

    async fn stop(&self, name: &str) -> BackendResult<()> {
        let generation = self.inner.lock().entries.get(name).map(|entry| entry.generation);
        match generation {
            Some(generation) => self.stop_generation(name, generation).await,
            None => Ok(()),
        }
    }

    fn inspect(&self, name: &str) -> (TunnelRuntimeState, &'static str, Option<String>) {
        let runtime = self.inner.lock();
        match runtime.entries.get(name) {
            Some(entry) => (
                entry.state,
                entry.failure.unwrap_or("streamr tunnel runtime is active"),
                entry.destination.clone(),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                "streamr tunnel runtime is stopped",
                None,
            ),
        }
    }
}

/// Bounded in-memory producer subscription state.
#[derive(Debug, Default)]
pub struct SubscriptionState {
    entries: HashMap<String, Instant>,
}

impl SubscriptionState {
    /// Apply one exact Streamr control packet.
    pub fn apply_control(&mut self, peer: &str, control: &[u8], now: Instant) -> bool {
        if peer.is_empty()
            || peer.len() > 64 * 1024
            || peer
                .chars()
                .any(|character| character.is_control() || character.is_whitespace())
        {
            return false;
        }
        if control.len() != 1 {
            return false;
        }
        match control[0] {
            0 =>
                if self.entries.contains_key(peer) || self.entries.len() < MAX_SUBSCRIBERS {
                    self.entries.insert(peer.to_owned(), now);
                    true
                } else {
                    false
                },
            1 => self.entries.remove(peer).is_some(),
            _ => false,
        }
    }

    /// Expire stale consumers without per-consumer timers.
    pub fn expire(&mut self, now: Instant) {
        self.entries
            .retain(|_, refreshed| now.saturating_duration_since(*refreshed) < SUBSCRIPTION_EXPIRY);
    }

    /// Snapshot destinations before performing network sends.
    pub fn snapshot(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    /// Number of active consumers, useful for deterministic tests/inspection.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the producer has no active consumers.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

async fn run_streamr_client(
    config: StreamrClientConfig,
    mut cancellation: watch::Receiver<bool>,
    ready: oneshot::Sender<Result<(), String>>,
) -> bool {
    let mut session = tokio::select! {
        _ = cancellation.changed() => {
            let _ = ready.send(Err("streamr client session setup cancelled".to_owned()));
            return true;
        }
        result = Session::<style::Repliable>::new(SessionOptions {
            samv3_tcp_port: config.sam_tcp_port,
            nickname: config.name.clone(),
            publish: false,
            from_port: config.source_port,
            ..Default::default()
        }) => match result {
            Ok(session) => session,
            Err(_) => {
                let _ = ready.send(Err("streamr client session setup failed".to_owned()));
                return false;
            }
        },
    };
    let output = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(socket) => socket,
        Err(_) => {
            let _ = ready.send(Err("streamr client UDP setup failed".to_owned()));
            return false;
        }
    };
    let _ = ready.send(Ok(()));
    let mut receive_buffer = vec![0u8; MAX_TRANSPORT_PACKET];
    let mut refresh = tokio::time::interval(SUBSCRIPTION_REFRESH);
    refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = cancellation.changed() => {
                let _ = tokio::time::timeout(
                    UNSUBSCRIBE_TIMEOUT,
                    session.send_to_with_options(
                        &[1],
                        &config.producer,
                        DatagramOptions { from_port: config.source_port, ..Default::default() },
                    ),
                ).await;
                return true;
            }
            _ = refresh.tick() => {
                let _ = session.send_to_with_options(
                    &[0],
                    &config.producer,
                    DatagramOptions { from_port: config.source_port, ..Default::default() },
                ).await;
            }
            result = session.recv_from(&mut receive_buffer) => {
                let Ok((length, _peer)) = result else { return false; };
                if length <= MAX_STREAMR_PAYLOAD {
                    let _ = output.send_to(&receive_buffer[..length], config.local_target).await;
                }
            }
        }
    }
}

async fn run_streamr_server(
    config: StreamrServerConfig,
    mut cancellation: watch::Receiver<bool>,
    ready: oneshot::Sender<Result<String, String>>,
) -> bool {
    let mut session = tokio::select! {
        _ = cancellation.changed() => {
            let _ = ready.send(Err("streamr server session setup cancelled".to_owned()));
            return true;
        }
        result = Session::<style::Repliable>::new(SessionOptions {
            samv3_tcp_port: config.sam_tcp_port,
            nickname: config.name.clone(),
            publish: true,
            destination: DestinationKind::Persistent { private_key: config.destination.as_str().to_owned() },
            from_port: config.source_port,
            to_port: config.destination_port,
            ..Default::default()
        }) => match result {
            Ok(session) => session,
            Err(_) => {
                let _ = ready.send(Err("streamr server session setup failed".to_owned()));
                return false;
            }
        },
    };
    let socket =
        match UdpSocket::bind(SocketAddr::new(config.bind_address, config.local_port)).await {
            Ok(socket) => socket,
            Err(_) => {
                let _ = ready.send(Err("streamr server UDP bind failed".to_owned()));
                return false;
            }
        };
    let destination = session.destination().to_owned();
    let _ = ready.send(Ok(destination));
    let mut datagram_buffer = vec![0u8; MAX_TRANSPORT_PACKET];
    let mut udp_buffer = vec![0u8; MAX_TRANSPORT_PACKET];
    let mut state = SubscriptionState::default();
    let mut expiry = tokio::time::interval(EXPIRY_SCAN);
    expiry.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = cancellation.changed() => return true,
            _ = expiry.tick() => state.expire(Instant::now()),
            result = session.recv_from(&mut datagram_buffer) => {
                let Ok((length, peer)) = result else { return false; };
                if length == 1 {
                    let _ = state.apply_control(&peer, &datagram_buffer[..length], Instant::now());
                }
            }
            result = socket.recv_from(&mut udp_buffer) => {
                let Ok((length, _source)) = result else { return false; };
                if length > MAX_STREAMR_PAYLOAD { continue; }
                let peers = state.snapshot();
                for peer in peers {
                    let _ = session.send_to_with_options(
                        &udp_buffer[..length],
                        &peer,
                        DatagramOptions {
                            from_port: config.source_port,
                            to_port: config.destination_port,
                            ..Default::default()
                        },
                    ).await;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct StreamrClientTunnelBackend {
    supervisor: StreamrRuntimeSupervisor,
    sam_tcp_port: u16,
}

impl StreamrClientTunnelBackend {
    pub fn new(sam_tcp_port: u16) -> Self {
        Self {
            supervisor: StreamrRuntimeSupervisor::new(TunnelType::StreamrClient),
            sam_tcp_port,
        }
    }

    fn config(&self, definition: &TunnelDefinition) -> BackendResult<StreamrClientConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::StreamrClient,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_options(
            TunnelType::StreamrClient,
            &definition.options,
            STREAMR_CLIENT_OPTIONS,
        )
        .map_err(option_error)?;
        validate_raw_streamr_options(definition, TunnelType::StreamrClient, true)?;
        let producer = definition
            .options
            .target_destination
            .as_deref()
            .or(definition.options.streamr_target.as_deref())
            .filter(|value| valid_destination(value))
            .ok_or_else(|| BackendError::Internal {
                message: "streamrclient producer destination is invalid".to_owned(),
            })?;
        let target_host = local_host(definition)?;
        let target_port =
            definition.options.target_port.ok_or_else(|| BackendError::MissingOption {
                tunnel_type: TunnelType::StreamrClient,
                option: "TargetPort".to_owned(),
            })?;
        Ok(StreamrClientConfig {
            name: definition.name.as_str().to_owned(),
            producer: producer.to_owned(),
            local_target: SocketAddr::new(target_host, target_port),
            source_port: definition.options.listen_port.unwrap_or(0),
            sam_tcp_port: self.sam_tcp_port,
        })
    }

    async fn start_config(&self, config: StreamrClientConfig) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.supervisor.reserve(&name)?;
        let map = Arc::clone(&self.supervisor.inner);
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let (ready_tx, ready_rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(run_streamr_client(
                config,
                ready_cancellation.clone(),
                ready_tx,
            ))
            .catch_unwind()
            .await
            .unwrap_or(false);
            let cancelled = *ready_cancellation.borrow();
            StreamrRuntimeSupervisor::complete(map, task_name, generation, result, cancelled);
        });
        self.supervisor.set_task(&name, generation, task);
        match tokio::time::timeout(START_TIMEOUT, ready_rx).await {
            Ok(Ok(Ok(()))) if self.supervisor.mark_running(&name, generation, false) => Ok(()),
            Ok(Ok(Err(message))) => {
                let _ = self.supervisor.stop_generation(&name, generation).await;
                Err(BackendError::Internal { message })
            }
            Ok(Ok(Ok(()))) | Ok(Err(_)) | Err(_) => {
                let _ = self.supervisor.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "streamrclient tunnel runtime failed to start".to_owned(),
                })
            }
        }
    }
}

#[async_trait::async_trait]
impl TunnelBackend for StreamrClientTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::StreamrClient
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        self.start_config(self.config(definition)?).await
    }

    async fn stop(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        self.supervisor.stop(definition.name.as_str()).await
    }

    fn inspect(&self, definition: &TunnelDefinition) -> BackendStatus {
        let (runtime_state, message, _) = self.supervisor.inspect(definition.name.as_str());
        BackendStatus {
            tunnel_type: TunnelType::StreamrClient,
            runtime_state,
            message: message.to_owned(),
            destination: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StreamrServerTunnelBackend {
    supervisor: StreamrRuntimeSupervisor,
    sam_tcp_port: u16,
    destinations: ServerDestinationStore,
}

impl StreamrServerTunnelBackend {
    pub fn new(sam_tcp_port: u16, destinations: ServerDestinationStore) -> Self {
        Self {
            supervisor: StreamrRuntimeSupervisor::new(TunnelType::StreamrServer),
            sam_tcp_port,
            destinations,
        }
    }

    fn identity(definition: &TunnelDefinition) -> BackendResult<&str> {
        definition
            .raw_config
            .get(SERVER_IDENTITY_KEY)
            .and_then(|value| value.as_str())
            .ok_or_else(|| BackendError::Internal {
                message: "server destination identity is not allocated".to_owned(),
            })
    }

    fn config(
        &self,
        definition: &TunnelDefinition,
        destination: StoredDestination,
    ) -> BackendResult<StreamrServerConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::StreamrServer,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_options(
            TunnelType::StreamrServer,
            &definition.options,
            STREAMR_SERVER_OPTIONS,
        )
        .map_err(option_error)?;
        validate_raw_streamr_options(definition, TunnelType::StreamrServer, false)?;
        let local_port =
            definition.options.listen_port.ok_or_else(|| BackendError::MissingOption {
                tunnel_type: TunnelType::StreamrServer,
                option: "ListenPort".to_owned(),
            })?;
        Ok(StreamrServerConfig {
            name: definition.name.as_str().to_owned(),
            bind_address: local_host(definition)?,
            local_port,
            source_port: local_port,
            destination_port: definition.options.target_port.unwrap_or(0),
            sam_tcp_port: self.sam_tcp_port,
            destination,
        })
    }

    async fn start_config(&self, config: StreamrServerConfig) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.supervisor.reserve(&name)?;
        let map = Arc::clone(&self.supervisor.inner);
        let supervisor = self.supervisor.clone();
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let (ready_tx, ready_rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(run_streamr_server(
                config,
                ready_cancellation.clone(),
                ready_tx,
            ))
            .catch_unwind()
            .await
            .unwrap_or(false);
            let cancelled = *ready_cancellation.borrow();
            StreamrRuntimeSupervisor::complete(map, task_name, generation, result, cancelled);
        });
        self.supervisor.set_task(&name, generation, task);
        match tokio::time::timeout(START_TIMEOUT, ready_rx).await {
            Ok(Ok(Ok(destination))) => {
                supervisor.publish_destination(&name, generation, &destination);
                if supervisor.mark_running(&name, generation, true) {
                    Ok(())
                } else {
                    let _ = supervisor.stop_generation(&name, generation).await;
                    Err(BackendError::Internal {
                        message: "streamrserver tunnel exited during start".to_owned(),
                    })
                }
            }
            Ok(Ok(Err(message))) => {
                let _ = supervisor.stop_generation(&name, generation).await;
                Err(BackendError::Internal { message })
            }
            Ok(Err(_)) | Err(_) => {
                let _ = supervisor.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "streamrserver tunnel runtime failed to start".to_owned(),
                })
            }
        }
    }
}

#[async_trait::async_trait]
impl TunnelBackend for StreamrServerTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::StreamrServer
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        let identity = Self::identity(definition)?;
        let destination = self
            .destinations
            .get(identity)
            .await
            .map_err(|_| BackendError::Internal {
                message: "server destination store lookup failed".to_owned(),
            })?
            .ok_or_else(|| BackendError::Internal {
                message: "server destination identity is unavailable".to_owned(),
            })?;
        self.start_config(self.config(definition, destination)?).await
    }

    async fn stop(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        self.supervisor.stop(definition.name.as_str()).await
    }

    fn inspect(&self, definition: &TunnelDefinition) -> BackendStatus {
        let (runtime_state, message, destination) =
            self.supervisor.inspect(definition.name.as_str());
        BackendStatus {
            tunnel_type: TunnelType::StreamrServer,
            runtime_state,
            message: message.to_owned(),
            destination,
        }
    }
}

fn local_host(definition: &TunnelDefinition) -> BackendResult<IpAddr> {
    let value = definition
        .raw_config
        .get("TargetHost")
        .or_else(|| definition.raw_config.get("Host"))
        .and_then(|value| value.as_str())
        .or(definition.options.listen_interface.as_deref());
    match value {
        None => Ok(DEFAULT_BIND_ADDRESS),
        Some(value) => value.parse::<IpAddr>().map_err(|_| BackendError::Internal {
            message: "Streamr local UDP host must be an IP address".to_owned(),
        }),
    }
}

fn validate_raw_streamr_options(
    definition: &TunnelDefinition,
    tunnel_type: TunnelType,
    client: bool,
) -> BackendResult<()> {
    for key in definition.raw_config.keys() {
        let supported = matches!(
            key.as_str(),
            "TargetHost"
                | "Host"
                | "TargetPort"
                | "Port"
                | "ReachableBy"
                | "TargetDestination"
                | "Destination"
                | "StreamrTarget"
                | "Description"
                | "StartOnLoad"
                | "CustomOptions"
                | "i2cp"
                | "i2p.tunnel.streamrTarget"
        );
        if !supported
            && (key.starts_with("Tunnel")
                || key.starts_with("Use")
                || key == "SigType"
                || key == "EncType")
        {
            return Err(BackendError::UnsupportedOption {
                tunnel_type,
                option: key.clone(),
            });
        }
    }
    if !client && definition.options.streamr_target.is_some() {
        return Err(BackendError::UnsupportedOption {
            tunnel_type,
            option: "StreamrTarget".to_owned(),
        });
    }
    Ok(())
}

fn valid_destination(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64 * 1024
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2pcontrol::domain::tunnel::{StartIntent, TunnelName, TunnelOptions};

    fn definition(tunnel_type: TunnelType) -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("streamr-test").unwrap(),
            tunnel_type,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions::default(),
            raw_config: Default::default(),
        }
    }

    #[test]
    fn subscriptions_are_bounded_and_refresh_in_place() {
        let mut state = SubscriptionState::default();
        let now = Instant::now();
        for index in 0..MAX_SUBSCRIBERS {
            assert!(state.apply_control(&format!("peer-{index}"), &[0], now));
        }
        assert_eq!(state.len(), MAX_SUBSCRIBERS);
        assert!(!state.apply_control("overflow", &[0], now));
        assert!(state.apply_control("peer-0", &[0], now + Duration::from_secs(30)));
        assert_eq!(state.len(), MAX_SUBSCRIBERS);
    }

    #[test]
    fn invalid_controls_and_expiry_do_not_leak_state() {
        let mut state = SubscriptionState::default();
        let now = Instant::now();
        assert!(!state.apply_control("peer", &[], now));
        assert!(!state.apply_control("peer", &[2], now));
        assert!(!state.apply_control("peer", &[0, 0], now));
        assert!(state.apply_control("peer", &[0], now));
        assert!(state.apply_control("peer", &[1], now));
        assert_eq!(state.len(), 0);
        assert!(state.apply_control("peer", &[0], now));
        state.expire(now + SUBSCRIPTION_EXPIRY);
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn streamr_client_requires_target_and_local_port() {
        let backend = StreamrClientTunnelBackend::new(7656);
        let def = definition(TunnelType::StreamrClient);
        assert!(matches!(
            backend.config(&def),
            Err(BackendError::MissingOption { option, .. }) if option == "TargetPort"
        ));
        let mut def = def;
        def.options.target_port = Some(9000);
        assert!(matches!(
            backend.config(&def),
            Err(BackendError::MissingOption { option, .. }) if option == "TargetDestination"
        ));
        def.options.target_destination = Some("producer-destination".to_owned());
        assert!(backend.config(&def).is_ok());
    }

    #[test]
    fn streamr_server_rejects_unsupported_tunnel_shape_before_lookup() {
        let root = tempfile::tempdir().unwrap();
        let backend =
            StreamrServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        let mut def = definition(TunnelType::StreamrServer);
        def.options.listen_port = Some(0);
        def.options.signature_type = Some("EdDSA".to_owned());
        assert!(matches!(
            backend.config(&def, StoredDestination::from_private("private".to_owned())),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "SignatureType"
        ));
    }
}
