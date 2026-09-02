//! Bounded registration-filtered control-plane-owned IRC server tunnel.

use std::{
    collections::HashMap,
    future::Future,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{
    io::{self, AsyncBufRead, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::TcpStream,
    task::JoinHandle,
};
use yosemite::{DestinationKind, SessionOptions};

use super::{
    filters::irc::{command_and_params, normalize_line, read_bounded_line, rewrite_server_user},
    options::{validate_options, OptionValidationError, IRC_SERVER_OPTIONS},
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    backends::{http_server::normalize_loopback_target, runtime::*, server::SERVER_IDENTITY_KEY},
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
    server_secret_store::{ServerDestinationStore, StoredDestination},
};

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RUNTIME_TASKS: usize = 1000;
const TARGET_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const RELAY_BUFFER_SIZE: usize = 8192;

/// Conservative registration bounds for the local IRCd boundary.
pub const REGISTRATION_LINE_TIMEOUT: Duration = Duration::from_secs(5);
pub const REGISTRATION_TIMEOUT: Duration = Duration::from_secs(15);
pub const MAX_REGISTRATION_LINES: usize = 12;
pub const MAX_REGISTRATION_LINE: usize = 1024;
pub const POST_REGISTRATION_INACTIVITY: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone)]
struct IrcServerConfig {
    name: String,
    target_address: IpAddr,
    target_port: u16,
    sam_tcp_port: u16,
    destination: StoredDestination,
    admission: ServerAdmissionPolicy,
    access: ServerAccessPolicy,
    session_options: SessionOptions,
}

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

#[derive(Clone, Debug)]
struct IrcServerRuntimeSupervisor {
    inner: Arc<Mutex<RuntimeMap>>,
}

impl IrcServerRuntimeSupervisor {
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
                    tunnel_type: TunnelType::IrcServer,
                    current_state: entry.state,
                    attempted_action: "start",
                });
            }
        }
        if runtime.entries.values().filter(|entry| entry.task.is_some()).count()
            >= MAX_RUNTIME_TASKS
        {
            return Err(BackendError::Internal {
                message: "ircserver runtime capacity exhausted".to_owned(),
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

    fn complete(
        map: Arc<Mutex<RuntimeMap>>,
        name: String,
        generation: u64,
        result: Result<(), AcceptedServerRuntimeError>,
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
            entry.failure = Some("ircserver tunnel runtime failed");
        } else {
            entry.state = TunnelRuntimeState::Stopped;
            entry.failure = None;
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
                message: "ircserver tunnel stop timed out".to_owned(),
            });
        }
        self.remove_generation(name, generation);
        Ok(())
    }

    async fn start(&self, config: IrcServerConfig) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.reserve(&name)?;
        let map = Arc::clone(&self.inner);
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let supervisor = self.clone();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let handler: AcceptedServerHandler = Arc::new(move |connection| {
                let address = config.target_address;
                let port = config.target_port;
                Box::pin(async move {
                    let _ = handle_accepted_connection(connection, address, port).await;
                })
            });
            let result = std::panic::AssertUnwindSafe(run_accepted_server(
                AcceptedServerRuntimeConfig {
                    name: config.name,
                    sam_tcp_port: config.sam_tcp_port,
                    destination: config.destination,
                    admission: config.admission,
                    access: config.access,
                    lease_set_enc_type: None,
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
            Self::complete(map, task_name, generation, result, cancelled);
        });
        self.set_task(&name, generation, task);
        match tokio::time::timeout(START_TIMEOUT, ready_rx).await {
            Ok(Ok(Ok(destination)))
                if {
                    supervisor.publish_destination(&name, generation, &destination);
                    supervisor.mark_running(&name, generation)
                } =>
            {
                Ok(())
            }
            Ok(Ok(Err(_))) | Ok(Err(_)) | Err(_) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "ircserver tunnel runtime failed to start".to_owned(),
                })
            }
            Ok(Ok(Ok(_))) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "ircserver tunnel runtime exited during start".to_owned(),
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

    fn inspect(&self, name: &str) -> (TunnelRuntimeState, &'static str, Option<String>) {
        let runtime = self.inner.lock();
        match runtime.entries.get(name) {
            Some(entry) => (
                entry.state,
                entry.failure.unwrap_or("ircserver tunnel runtime is active"),
                entry.destination.clone(),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                "ircserver tunnel runtime is stopped",
                None,
            ),
        }
    }
}

async fn handle_accepted_connection(
    connection: AcceptedServerConnection,
    target_address: IpAddr,
    target_port: u16,
) -> io::Result<()> {
    let peer_hostname = crate::i2pcontrol::address_book_runtime::base32_for_destination(
        connection.peer.destination(),
    );
    if !valid_hostname(&peer_hostname) {
        return Ok(());
    }
    let (remote_read, remote_write) = io::split(connection.stream);
    let mut remote_reader = BufReader::new(remote_read);
    let registration = tokio::time::timeout(
        REGISTRATION_TIMEOUT,
        read_registration(&mut remote_reader, peer_hostname.as_bytes()),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IRC registration timeout"))??;

    let mut local = match connect_local_target(target_address, target_port).await {
        Ok(stream) => stream,
        Err(_) => return Ok(()),
    };
    for line in registration {
        local.write_all(&line).await?;
    }
    let (local_read, local_write) = local.into_split();
    relay_with_inactivity(remote_reader, remote_write, local_read, local_write).await
}

async fn connect_local_target(address: IpAddr, port: u16) -> io::Result<TcpStream> {
    bounded_connect(TcpStream::connect(SocketAddr::new(address, port))).await
}

async fn bounded_connect<F>(connect: F) -> io::Result<TcpStream>
where
    F: Future<Output = io::Result<TcpStream>>,
{
    match tokio::time::timeout(TARGET_CONNECT_TIMEOUT, connect).await {
        Ok(Ok(stream)) => Ok(stream),
        Ok(Err(_)) | Err(_) => Err(io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "IRC local target unavailable",
        )),
    }
}

async fn relay_with_inactivity<RemoteRead, RemoteWrite, LocalRead, LocalWrite>(
    remote_read: RemoteRead,
    remote_write: RemoteWrite,
    local_read: LocalRead,
    local_write: LocalWrite,
) -> io::Result<()>
where
    RemoteRead: AsyncRead + Unpin,
    RemoteWrite: AsyncWrite + Unpin,
    LocalRead: AsyncRead + Unpin,
    LocalWrite: AsyncWrite + Unpin,
{
    let (activity_tx, mut activity_rx) = tokio::sync::watch::channel(0_u64);
    let mut remote_to_local = Box::pin(relay_direction(
        remote_read,
        local_write,
        activity_tx.clone(),
    ));
    let mut local_to_remote = Box::pin(relay_direction(local_read, remote_write, activity_tx));
    let mut remote_to_local_active = true;
    let mut local_to_remote_active = true;
    let deadline = tokio::time::sleep(POST_REGISTRATION_INACTIVITY);
    tokio::pin!(deadline);

    loop {
        if !remote_to_local_active && !local_to_remote_active {
            return Ok(());
        }

        tokio::select! {
            result = &mut remote_to_local, if remote_to_local_active => {
                remote_to_local_active = false;
                result?;
            }
            result = &mut local_to_remote, if local_to_remote_active => {
                local_to_remote_active = false;
                result?;
            }
            changed = activity_rx.changed() => {
                if changed.is_err() {
                    return Ok(());
                }
                deadline.as_mut().reset(tokio::time::Instant::now() + POST_REGISTRATION_INACTIVITY);
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

async fn read_registration<R>(reader: &mut R, peer_hostname: &[u8]) -> io::Result<Vec<Vec<u8>>>
where
    R: AsyncBufRead + Unpin,
{
    let mut output = Vec::new();
    let mut saw_nick = false;
    let mut saw_user = false;
    for line_number in 0..MAX_REGISTRATION_LINES {
        let line = tokio::time::timeout(
            REGISTRATION_LINE_TIMEOUT,
            read_bounded_line(reader, MAX_REGISTRATION_LINE + 2),
        )
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IRC registration line timeout"))??
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "incomplete IRC registration")
        })?;
        if line_number == 0 && looks_like_wrong_protocol(&line) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "wrong protocol for IRC",
            ));
        }
        let Some((command, params)) = command_and_params(&line) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "malformed IRC registration",
            ));
        };
        match command.as_str() {
            "CAP" | "PASS" | "AUTHENTICATE" | "PING" | "PONG" => {}
            "NICK" if params.len() == 1 && !params[0].is_empty() => saw_nick = true,
            "USER" => {
                if params.len() < 4 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "incomplete IRC USER",
                    ));
                }
                saw_user = true;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unsupported IRC registration",
                ))
            }
        }
        let safe_line = if command == "USER" {
            rewrite_server_user(&line, peer_hostname).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid IRC USER registration")
            })?
        } else {
            normalize_line(&line).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid IRC registration")
            })?
        };
        output.push(safe_line);
        if saw_nick && saw_user {
            return Ok(output);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "too many IRC registration lines",
    ))
}

fn looks_like_wrong_protocol(line: &[u8]) -> bool {
    let upper = line.iter().map(u8::to_ascii_uppercase).collect::<Vec<_>>();
    [
        b"GET ".as_slice(),
        b"POST ".as_slice(),
        b"PUT ".as_slice(),
        b"HEAD ".as_slice(),
        b"CONNECT ".as_slice(),
        b"OPTIONS ".as_slice(),
        b"PRI * HTTP/".as_slice(),
        b"HTTP/".as_slice(),
        b"D1:AD".as_slice(),
        b"D4:INFO".as_slice(),
    ]
    .iter()
    .any(|prefix| upper.starts_with(prefix))
}

fn valid_hostname(hostname: &str) -> bool {
    !hostname.is_empty()
        && hostname.len() <= 255
        && !hostname
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
}

/// Real filtered backend for Proposal 170 `ircserver`.
#[derive(Clone, Debug)]
pub struct IrcServerTunnelBackend {
    supervisor: IrcServerRuntimeSupervisor,
    destinations: ServerDestinationStore,
    sam_tcp_port: u16,
}

impl IrcServerTunnelBackend {
    pub fn new(sam_tcp_port: u16, destinations: ServerDestinationStore) -> Self {
        Self {
            supervisor: IrcServerRuntimeSupervisor::new(),
            destinations,
            sam_tcp_port,
        }
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

#[async_trait::async_trait]
impl TunnelBackend for IrcServerTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::IrcServer
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        let mut config = self.config_without_destination(definition).await?;
        let identity = definition
            .raw_config
            .get(SERVER_IDENTITY_KEY)
            .and_then(|value| value.as_str())
            .ok_or_else(|| BackendError::Internal {
                message: "ircserver destination identity is not allocated".to_owned(),
            })?;
        config.destination = self
            .destinations
            .get(identity)
            .await
            .map_err(|_| BackendError::Internal {
                message: "ircserver destination store lookup failed".to_owned(),
            })?
            .ok_or_else(|| BackendError::Internal {
                message: "ircserver destination identity is unavailable".to_owned(),
            })?;
        config.session_options = super::runtime::session::build_session_options(
            definition,
            self.sam_tcp_port,
            true,
            DestinationKind::Persistent {
                private_key: config.destination.as_str().to_owned(),
            },
        )?;
        self.supervisor.start(config).await
    }

    async fn stop(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        self.supervisor.stop(definition.name.as_str()).await
    }

    fn inspect(&self, definition: &TunnelDefinition) -> BackendStatus {
        let (runtime_state, message, destination) =
            self.supervisor.inspect(definition.name.as_str());
        BackendStatus {
            tunnel_type: TunnelType::IrcServer,
            runtime_state,
            message: message.to_owned(),
            destination,
        }
    }
}

impl IrcServerTunnelBackend {
    async fn config_without_destination(
        &self,
        definition: &TunnelDefinition,
    ) -> BackendResult<IrcServerConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::IrcServer,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_options(
            TunnelType::IrcServer,
            &definition.options,
            IRC_SERVER_OPTIONS,
        )
        .map_err(option_error)?;
        validate_raw_options(definition)?;
        let admission = ServerAdmissionPolicy::from_raw_options(&definition.raw_config)
            .map_err(invalid_option)?;
        let access = if let Some(path) = raw_string(definition, "FilterFilePath")? {
            ServerAccessPolicy::from_filter_file(
                self.destinations.directory().parent().unwrap_or(self.destinations.directory()),
                &path,
            )
        } else {
            ServerAccessPolicy::from_values(
                raw_string(definition, "AccessOption")?.as_deref(),
                raw_string(definition, "AccessList")?.as_deref(),
            )
        }
        .map_err(invalid_option)?;
        let target_host = definition
            .raw_config
            .get("TargetHost")
            .or_else(|| definition.raw_config.get("Host"))
            .and_then(|value| value.as_str())
            .unwrap_or("127.0.0.1");
        let target_address = normalize_loopback_target(target_host, false).ok_or_else(|| {
            BackendError::Internal {
                message: "ircserver target host must be loopback".to_owned(),
            }
        })?;
        let target_port = definition
            .options
            .target_port
            .or(definition.options.listen_port)
            .ok_or_else(|| BackendError::MissingOption {
                tunnel_type: TunnelType::IrcServer,
                option: "TargetPort".to_owned(),
            })?;
        Ok(IrcServerConfig {
            name: definition.name.as_str().to_owned(),
            target_address,
            target_port,
            sam_tcp_port: self.sam_tcp_port,
            destination: StoredDestination::from_private(String::new()),
            admission,
            access,
            session_options: SessionOptions::default(),
        })
    }
}

fn validate_raw_options(definition: &TunnelDefinition) -> BackendResult<()> {
    const SUPPORTED: &[&str] = &[
        "TargetPort",
        "ListenPort",
        "TargetHost",
        "Host",
        "HostingDestination",
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
        "FilterFilePath",
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
            tunnel_type: TunnelType::IrcServer,
            option: key.clone(),
        });
    }
    Ok(())
}

fn invalid_option(option: &str) -> BackendError {
    BackendError::Internal {
        message: format!("ircserver option {option} is invalid"),
    }
}

fn raw_string(definition: &TunnelDefinition, key: &str) -> BackendResult<Option<String>> {
    definition
        .raw_config
        .get(key)
        .map(|value| value.as_str().map(str::to_owned).ok_or_else(|| invalid_option(key)))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use emissary_core::crypto::base64_encode;
    use tokio::io::{duplex, AsyncBufReadExt, AsyncWriteExt};

    fn definition(identity: &str) -> TunnelDefinition {
        TunnelDefinition {
            name: crate::i2pcontrol::domain::tunnel::TunnelName::new("irc-server").unwrap(),
            tunnel_type: TunnelType::IrcServer,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: crate::i2pcontrol::domain::tunnel::StartIntent::DoNotStart,
            options: crate::i2pcontrol::domain::tunnel::TunnelOptions {
                target_port: Some(0),
                ..Default::default()
            },
            raw_config: std::collections::BTreeMap::from([(
                SERVER_IDENTITY_KEY.to_owned(),
                serde_json::json!(identity),
            )]),
        }
    }

    #[tokio::test]
    async fn registration_rewrites_trusted_peer_and_rejects_http() {
        let (mut client, server) = duplex(4096);
        let read = async move {
            let mut reader = BufReader::new(server);
            let result = read_registration(&mut reader, b"peer.b32.i2p").await.unwrap();
            assert_eq!(result[1], b"USER alice 0 peer.b32.i2p :Alice\r\n");
        };
        let write = async move {
            client.write_all(b"NICK alice\r\n").await.unwrap();
            client.write_all(b"USER alice 0 spoofed-host :Alice\r\n").await.unwrap();
        };
        tokio::join!(read, write);

        let (mut client, server) = duplex(4096);
        let read = async move {
            let mut reader = BufReader::new(server);
            assert!(read_registration(&mut reader, b"peer.b32.i2p").await.is_err());
        };
        let write = async move {
            client.write_all(b"GET / HTTP/1.1\r\n").await.unwrap();
        };
        tokio::join!(read, write);
    }

    #[tokio::test]
    async fn registration_bounds_are_enforced() {
        let (mut client, server) = duplex(4096);
        let read = async move {
            let mut reader = BufReader::new(server);
            assert!(read_registration(&mut reader, b"peer.b32.i2p").await.is_err());
        };
        let write = async move {
            for _ in 0..MAX_REGISTRATION_LINES {
                client.write_all(b"CAP LS\r\n").await.unwrap();
            }
        };
        tokio::join!(read, write);
    }

    #[tokio::test(start_paused = true)]
    async fn registered_idle_peer_expires_and_releases_admission() {
        let (remote_peer, remote_stream) = duplex(4096);
        let (local_peer, local_stream) = duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (local_read, local_write) = io::split(local_stream);
        let admission =
            ServerAdmissionState::new(ServerAdmissionPolicy::new(1, 0, 0, 0, 0, 0, 0).unwrap());
        let peer =
            crate::i2pcontrol::backends::runtime::peer_identity::test_fixtures::distinct_peer(7);
        let lease = match admission.try_acquire(&peer) {
            AdmissionDecision::Allowed(lease) => lease,
            other => panic!("unexpected admission result: {other:?}"),
        };
        let relay = tokio::spawn(async move {
            let _lease = lease;
            relay_with_inactivity(remote_read, remote_write, local_read, local_write).await
        });

        tokio::time::advance(POST_REGISTRATION_INACTIVITY).await;
        assert!(relay.await.unwrap().is_ok());
        assert!(matches!(
            admission.try_acquire(&peer),
            AdmissionDecision::Allowed(_)
        ));
        drop(remote_peer);
        drop(local_peer);
    }

    #[tokio::test(start_paused = true)]
    async fn activity_resets_idle_deadline_without_fixed_lifetime() {
        let (mut remote_peer, remote_stream) = duplex(4096);
        let (mut local_peer, local_stream) = duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (local_read, local_write) = io::split(local_stream);
        let relay = tokio::spawn(relay_with_inactivity(
            remote_read,
            remote_write,
            local_read,
            local_write,
        ));

        for sequence in 0..3 {
            tokio::time::advance(POST_REGISTRATION_INACTIVITY - Duration::from_secs(60)).await;
            let message = [b'0' + sequence, b'\n'];
            remote_peer.write_all(&message).await.unwrap();
            let mut received = [0_u8; 2];
            local_peer.read_exact(&mut received).await.unwrap();
            assert_eq!(received, message);
        }
        tokio::time::advance(Duration::from_secs(2 * 60)).await;
        assert!(!relay.is_finished());

        drop(remote_peer);
        drop(local_peer);
        relay.abort();
        let _ = relay.await;
    }

    #[tokio::test(start_paused = true)]
    async fn traffic_in_either_direction_resets_idle_deadline() {
        let (mut remote_peer, remote_stream) = duplex(4096);
        let (mut local_peer, local_stream) = duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (local_read, local_write) = io::split(local_stream);
        let relay = tokio::spawn(relay_with_inactivity(
            remote_read,
            remote_write,
            local_read,
            local_write,
        ));

        tokio::time::advance(POST_REGISTRATION_INACTIVITY - Duration::from_secs(60)).await;
        local_peer.write_all(b"PING\r\n").await.unwrap();
        let mut received = [0_u8; 6];
        remote_peer.read_exact(&mut received).await.unwrap();
        assert_eq!(&received, b"PING\r\n");
        tokio::time::advance(Duration::from_secs(2 * 60)).await;
        assert!(!relay.is_finished());

        drop(remote_peer);
        drop(local_peer);
        relay.abort();
        let _ = relay.await;
    }

    #[tokio::test(start_paused = true)]
    async fn inactivity_closes_both_relay_directions() {
        let (mut remote_peer, remote_stream) = duplex(4096);
        let (mut local_peer, local_stream) = duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (local_read, local_write) = io::split(local_stream);
        let relay = tokio::spawn(relay_with_inactivity(
            remote_read,
            remote_write,
            local_read,
            local_write,
        ));

        tokio::time::advance(POST_REGISTRATION_INACTIVITY).await;
        assert!(relay.await.unwrap().is_ok());
        assert!(remote_peer.write_all(b"after-timeout").await.is_err());
        assert!(local_peer.write_all(b"after-timeout").await.is_err());
    }

    #[tokio::test]
    async fn remote_eof_allows_local_to_remote_drain() {
        let (mut remote_peer, remote_stream) = duplex(4096);
        let (mut local_peer, local_stream) = duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (local_read, local_write) = io::split(local_stream);
        let relay = tokio::spawn(relay_with_inactivity(
            remote_read,
            remote_write,
            local_read,
            local_write,
        ));
        remote_peer.shutdown().await.unwrap();
        local_peer.write_all(b"response").await.unwrap();
        let mut response = [0_u8; 8];
        remote_peer.read_exact(&mut response).await.unwrap();
        assert_eq!(&response, b"response");
        local_peer.shutdown().await.unwrap();
        assert!(relay.await.unwrap().is_ok());
    }

    #[tokio::test]
    async fn local_eof_allows_remote_to_local_drain() {
        let (mut remote_peer, remote_stream) = duplex(4096);
        let (mut local_peer, local_stream) = duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (local_read, local_write) = io::split(local_stream);
        let relay = tokio::spawn(relay_with_inactivity(
            remote_read,
            remote_write,
            local_read,
            local_write,
        ));
        local_peer.shutdown().await.unwrap();
        remote_peer.write_all(b"request").await.unwrap();
        let mut request = [0_u8; 7];
        local_peer.read_exact(&mut request).await.unwrap();
        assert_eq!(&request, b"request");
        remote_peer.shutdown().await.unwrap();
        assert!(relay.await.unwrap().is_ok());
    }

    #[tokio::test]
    async fn half_close_completion_releases_admission_lease() {
        let (mut remote_peer, remote_stream) = duplex(4096);
        let (mut local_peer, local_stream) = duplex(4096);
        let (remote_read, remote_write) = io::split(remote_stream);
        let (local_read, local_write) = io::split(local_stream);
        let admission =
            ServerAdmissionState::new(ServerAdmissionPolicy::new(1, 0, 0, 0, 0, 0, 0).unwrap());
        let peer =
            crate::i2pcontrol::backends::runtime::peer_identity::test_fixtures::distinct_peer(8);
        let lease = match admission.try_acquire(&peer) {
            AdmissionDecision::Allowed(lease) => lease,
            other => panic!("unexpected admission result: {other:?}"),
        };
        let relay = tokio::spawn(async move {
            let _lease = lease;
            relay_with_inactivity(remote_read, remote_write, local_read, local_write).await
        });

        remote_peer.shutdown().await.unwrap();
        local_peer.shutdown().await.unwrap();
        assert!(relay.await.unwrap().is_ok());
        assert!(matches!(
            admission.try_acquire(&peer),
            AdmissionDecision::Allowed(_)
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn local_target_connect_is_bounded_and_sanitized() {
        let error = bounded_connect(std::future::pending::<io::Result<TcpStream>>())
            .await
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::ConnectionAborted);
        let message = error.to_string();
        assert_eq!(message, "IRC local target unavailable");
        assert!(!message.contains("127.0.0.1"));
        assert!(!message.contains("6667"));
        assert!(!message.contains("timed out"));
    }

    #[tokio::test]
    async fn target_host_is_normalized_to_a_literal_loopback_address() {
        let backend = IrcServerTunnelBackend::new(1, ServerDestinationStore::new("."));
        let mut definition = definition("identity");
        definition
            .raw_config
            .insert("TargetHost".to_owned(), serde_json::json!("localhost"));
        let config = backend.config_without_destination(&definition).await.unwrap();
        assert_eq!(
            config.target_address,
            "127.0.0.1".parse::<IpAddr>().unwrap()
        );
    }

    #[tokio::test]
    async fn target_host_is_loopback_confined() {
        let backend = IrcServerTunnelBackend::new(1, ServerDestinationStore::new("."));
        let mut definition = definition("identity");
        definition
            .raw_config
            .insert("TargetHost".to_owned(), serde_json::json!("10.0.0.1"));
        assert!(matches!(
            backend.config_without_destination(&definition).await,
            Err(BackendError::Internal { message })
                if message == "ircserver target host must be loopback"
        ));
    }

    #[tokio::test]
    async fn published_hosting_destination_does_not_become_local_target() {
        let backend = IrcServerTunnelBackend::new(1, ServerDestinationStore::new("."));
        let mut definition = definition("identity");
        definition.options.hosting_destination = Some("published-router-info".to_owned());
        let config = backend.config_without_destination(&definition).await.unwrap();
        assert_eq!(
            config.target_address,
            "127.0.0.1".parse::<IpAddr>().unwrap()
        );
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
                            "HELLO REPLY RESULT=OK VERSION=3.3\n".to_owned()
                        } else if line.starts_with("SESSION CREATE") {
                            "SESSION STATUS RESULT=OK DESTINATION=irc-server\n".to_owned()
                        } else {
                            "STREAM STATUS RESULT=OK\n".to_owned()
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
    async fn lifecycle_publishes_identity_and_restarts_exact_generation() {
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        store.load().await.unwrap();
        let identity = ServerDestinationStore::new_identity();
        store
            .put(
                &identity,
                StoredDestination::from_private(base64_encode([7u8; 128])),
            )
            .await
            .unwrap();
        let (sam_port, sam_task) = fake_sam().await;
        let backend = IrcServerTunnelBackend::new(sam_port, store);
        let definition = definition(&identity);

        backend.start(&definition).await.unwrap();
        assert_eq!(
            backend.inspect(&definition).runtime_state,
            TunnelRuntimeState::Running
        );
        assert_eq!(
            backend.inspect(&definition).destination.as_deref(),
            Some("irc-server")
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
