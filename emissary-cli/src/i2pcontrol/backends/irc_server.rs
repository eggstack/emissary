//! Bounded registration-filtered control-plane-owned IRC server tunnel.

use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{
    io::{self, AsyncBufRead, AsyncWriteExt, BufReader},
    net::TcpStream,
    task::JoinHandle,
};

use super::{
    filters::irc::{command_and_params, normalize_line, read_bounded_line, rewrite_server_user},
    options::{validate_options, OptionValidationError, IRC_SERVER_OPTIONS},
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    backends::{runtime::*, server::SERVER_IDENTITY_KEY},
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
    server_secret_store::{ServerDestinationStore, StoredDestination},
};

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RUNTIME_TASKS: usize = 1000;

/// Conservative registration bounds for the local IRCd boundary.
pub const REGISTRATION_LINE_TIMEOUT: Duration = Duration::from_secs(5);
pub const REGISTRATION_TIMEOUT: Duration = Duration::from_secs(15);
pub const MAX_REGISTRATION_LINES: usize = 12;
pub const MAX_REGISTRATION_LINE: usize = 1024;

#[derive(Debug, Clone)]
struct IrcServerConfig {
    name: String,
    target_host: String,
    target_port: u16,
    sam_tcp_port: u16,
    destination: StoredDestination,
    admission: ServerAdmissionPolicy,
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
                let host = config.target_host.clone();
                let port = config.target_port;
                Box::pin(async move {
                    let _ = handle_accepted_connection(connection, host, port).await;
                })
            });
            let result = std::panic::AssertUnwindSafe(run_accepted_server(
                AcceptedServerRuntimeConfig {
                    name: config.name,
                    sam_tcp_port: config.sam_tcp_port,
                    destination: config.destination,
                    admission: config.admission,
                    lease_set_enc_type: None,
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
                Ok(()),
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
    target_host: String,
    target_port: u16,
) -> io::Result<()> {
    let peer_hostname = crate::i2pcontrol::address_book_runtime::base32_for_destination(
        connection.peer.destination(),
    );
    if !valid_hostname(&peer_hostname) {
        return Ok(());
    }
    let (remote_read, mut remote_write) = io::split(connection.stream);
    let mut remote_reader = BufReader::new(remote_read);
    let registration = tokio::time::timeout(
        REGISTRATION_TIMEOUT,
        read_registration(&mut remote_reader, peer_hostname.as_bytes()),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IRC registration timeout"))??;

    let mut local = TcpStream::connect((target_host.as_str(), target_port)).await?;
    for line in registration {
        local.write_all(&line).await?;
    }
    let (mut local_read, mut local_write) = local.into_split();
    let remote_to_local = io::copy(&mut remote_reader, &mut local_write);
    let local_to_remote = io::copy(&mut local_read, &mut remote_write);
    tokio::select! {
        result = remote_to_local => result.map(|_| ()),
        result = local_to_remote => result.map(|_| ()),
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
            _ =>
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unsupported IRC registration",
                )),
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
        let target_host = definition
            .raw_config
            .get("TargetHost")
            .or_else(|| definition.raw_config.get("Host"))
            .and_then(|value| value.as_str())
            .unwrap_or("127.0.0.1");
        if !matches!(target_host, "127.0.0.1" | "localhost") {
            return Err(BackendError::Internal {
                message: "ircserver target host must be loopback".to_owned(),
            });
        }
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
            target_host: target_host.to_owned(),
            target_port,
            sam_tcp_port: self.sam_tcp_port,
            destination: StoredDestination::from_private(String::new()),
            admission,
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

    #[tokio::test]
    async fn published_hosting_destination_does_not_become_local_target() {
        let backend = IrcServerTunnelBackend::new(1, ServerDestinationStore::new("."));
        let mut definition = definition("identity");
        definition.options.hosting_destination = Some("published-router-info".to_owned());
        let config = backend.config_without_destination(&definition).await.unwrap();
        assert_eq!(config.target_host, "127.0.0.1");
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
