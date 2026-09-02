//! Strict CONNECT client tunnel backend for Proposal 170 `connectclient`.

use std::{collections::HashMap, net::IpAddr, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    task::JoinHandle,
};

use super::{
    filters::{
        http_client::{
            classify_target, parse_authority, read_header_block, HttpTarget, OutproxyTarget,
        },
        proxy::{basic_authorization, credentials_match},
    },
    options::{validate_options, OptionValidationError, CONNECT_CLIENT_OPTIONS},
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    address_book_runtime::RuntimeAddressBookHandle,
    client_secret_store::ClientDestinationStore,
    backends::runtime::{
        ClientConnectionHandler, ClientListenerRuntimeConfig,
        ClientListenerRuntimeError,
    },
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
};
use yosemite::SessionOptions;

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RUNTIME_TASKS: usize = 1000;
const MAX_CONNECTIONS: usize = 128;
const DEFAULT_OUTPROXY_PORT: u16 = 4444;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectRequest {
    host: String,
    port: u16,
    authorization: Option<String>,
    content_length: usize,
}

impl ConnectRequest {
    fn parse(input: &[u8]) -> std::io::Result<Self> {
        if input.len() > super::filters::http_client::MAX_HEADER_BYTES
            || !input.ends_with(b"\r\n\r\n")
        {
            return Err(invalid("invalid CONNECT header block"));
        }
        let mut slots = [httparse::EMPTY_HEADER; super::filters::http_client::MAX_HEADER_COUNT];
        let mut request = httparse::Request::new(&mut slots);
        let complete = request.parse(input).map_err(|_| invalid("malformed CONNECT request"))?;
        if !matches!(complete, httparse::Status::Complete(_))
            || request.method.is_none_or(|method| !method.eq_ignore_ascii_case("CONNECT"))
        {
            return Err(invalid("only CONNECT is supported"));
        }
        if request.version != Some(1) {
            return Err(invalid("HTTP version unsupported"));
        }
        let target = request.path.ok_or_else(|| invalid("CONNECT target missing"))?;
        let (host, port) = parse_authority(target)?;
        let port = port.ok_or_else(|| invalid("CONNECT target requires a port"))?;
        if port == 0 {
            return Err(invalid("CONNECT port is invalid"));
        }
        let mut content_lengths = Vec::new();
        let mut authorization = None;
        for header in request.headers.iter() {
            let name = header.name.to_ascii_lowercase();
            let value =
                std::str::from_utf8(header.value).map_err(|_| invalid("invalid header"))?.trim();
            if value.chars().any(char::is_control) {
                return Err(invalid("invalid header"));
            }
            if name == "proxy-authorization" {
                authorization = Some(value.to_owned());
            } else if name == "content-length" {
                content_lengths
                    .push(value.parse::<usize>().map_err(|_| invalid("invalid Content-Length"))?);
            } else if name == "transfer-encoding" {
                return Err(invalid("Transfer-Encoding is unsupported"));
            }
        }
        if content_lengths.windows(2).any(|values| values[0] != values[1]) {
            return Err(invalid("conflicting Content-Length"));
        }
        if content_lengths.first().copied().unwrap_or(0) != 0 {
            return Err(invalid("CONNECT request body is unsupported"));
        }
        Ok(Self {
            host,
            port,
            authorization,
            content_length: 0,
        })
    }
}

#[derive(Clone)]
struct ConnectConfig {
    name: String,
    bind_address: IpAddr,
    port: u16,
    sam_tcp_port: u16,
    outproxy: Option<OutproxyTarget>,
    proxy_username: Option<String>,
    proxy_password: Option<String>,
    require_auth: bool,
    outproxy_authorization: Option<String>,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
    delay_open: bool,
    session_options: SessionOptions,
}

impl std::fmt::Debug for ConnectConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectConfig")
            .field("name", &self.name)
            .field("bind_address", &self.bind_address)
            .field("port", &self.port)
            .field("sam_tcp_port", &self.sam_tcp_port)
            .field("outproxy", &self.outproxy)
            .field("proxy_username", &self.proxy_username)
            .field(
                "proxy_password",
                &self.proxy_password.as_ref().map(|_| "***"),
            )
            .field("require_auth", &self.require_auth)
            .finish_non_exhaustive()
    }
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
struct RuntimeSupervisor {
    inner: Arc<Mutex<RuntimeMap>>,
}

impl RuntimeSupervisor {
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
                    tunnel_type: TunnelType::ConnectClient,
                    current_state: entry.state,
                    attempted_action: "start",
                });
            }
        }
        if runtime.entries.values().filter(|entry| entry.task.is_some()).count()
            >= MAX_RUNTIME_TASKS
        {
            return Err(BackendError::Internal {
                message: "connectclient runtime capacity exhausted".to_owned(),
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
            entry.failure = Some("connectclient tunnel runtime failed");
        } else {
            entry.state = TunnelRuntimeState::Stopped;
            entry.failure = None;
        }
    }

    async fn start(
        &self,
        config: ConnectConfig,
        shared_registry: Option<Arc<super::runtime::session::SharedClientSessionRegistry>>,
        shared: bool,
    ) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.reserve(&name)?;
        let map = Arc::clone(&self.inner);
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let handler = make_handler(config.clone());
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(super::runtime::run_client_listener_with_shared_session(
                ClientListenerRuntimeConfig {
                    name: config.name,
                    bind_address: config.bind_address,
                    port: config.port,
                    destination: "unused-connectclient-destination".to_owned(),
                    destination_port: 1,
                    sam_tcp_port: config.sam_tcp_port,
                    session_options: config.session_options,
                    delay_open: config.delay_open,
                    max_connections: MAX_CONNECTIONS,
                    handler,
                },
                ready_cancellation.clone(),
                ready_tx,
                shared_registry,
                shared,
            ))
            .catch_unwind()
            .await
            .unwrap_or(Err(ClientListenerRuntimeError::Panicked));
            Self::complete(
                map,
                task_name,
                generation,
                result,
                *ready_cancellation.borrow(),
            );
        });
        self.set_task(&name, generation, task);
        match tokio::time::timeout(START_TIMEOUT, ready_rx).await {
            Ok(Ok(Ok(_))) if self.mark_running(&name, generation) => Ok(()),
            Ok(Ok(Err(_))) | Ok(Err(_)) | Err(_) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "connectclient tunnel runtime failed to start".to_owned(),
                })
            }
            Ok(Ok(Ok(_))) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "connectclient tunnel runtime exited during start".to_owned(),
                })
            }
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
                message: "connectclient tunnel stop timed out".to_owned(),
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
                entry.failure.unwrap_or("connectclient tunnel runtime is active"),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                "connectclient tunnel runtime is stopped",
            ),
        }
    }
}

fn make_handler(config: ConnectConfig) -> ClientConnectionHandler {
    Arc::new(move |stream, connector| {
        let config = config.clone();
        Box::pin(async move {
            let mut reader = BufReader::new(stream);
            let Ok(header_block) = read_header_block(&mut reader).await else {
                let mut stream = reader.into_inner();
                let _ = write_error(&mut stream, 400, "Bad Request").await;
                return;
            };
            let Ok(request) = ConnectRequest::parse(&header_block) else {
                let mut stream = reader.into_inner();
                let _ = write_error(&mut stream, 400, "Bad Request").await;
                return;
            };
            if config.require_auth
                && !credentials_match(
                    request.authorization.as_deref(),
                    config.proxy_username.as_deref().unwrap_or_default(),
                    config.proxy_password.as_deref().unwrap_or_default(),
                )
            {
                let mut stream = reader.into_inner();
                let _ = write_error(&mut stream, 407, "Proxy Authentication Required").await;
                return;
            }
            let Ok(mut target) =
                classify_target(&request.host, request.port, config.outproxy.clone())
            else {
                let mut stream = reader.into_inner();
                let _ = write_error(&mut stream, 403, "Forbidden").await;
                return;
            };
            let destination = match &mut target {
                HttpTarget::I2p { destination, .. } => {
                    match super::http_client::resolve_destination(
                        destination,
                        config.address_book.as_ref(),
                    )
                    .await
                    {
                        Some(resolved) => {
                            *destination = resolved.clone();
                            resolved
                        }
                        None => {
                            let mut stream = reader.into_inner();
                            let _ = write_error(&mut stream, 502, "Bad Gateway").await;
                            return;
                        }
                    }
                }
                HttpTarget::Clearnet { outproxy, .. } => {
                    match super::http_client::resolve_destination(
                        &outproxy.destination,
                        config.address_book.as_ref(),
                    )
                    .await
                    {
                        Some(resolved) => resolved,
                        None => {
                            let mut stream = reader.into_inner();
                            let _ = write_error(&mut stream, 502, "Bad Gateway").await;
                            return;
                        }
                    }
                }
            };
            let remote_port = match &target {
                HttpTarget::I2p { port, .. } => *port,
                HttpTarget::Clearnet { outproxy, .. } => outproxy.port,
            };
            let Ok(mut remote) = connector.connect_to(&destination, remote_port).await else {
                let mut stream = reader.into_inner();
                let _ = write_error(&mut stream, 502, "Bad Gateway").await;
                return;
            };
            if let HttpTarget::Clearnet { host, port, .. } = &target {
                let mut handshake = format!("CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n").into_bytes();
                if let Some(auth) = &config.outproxy_authorization {
                    handshake
                        .extend_from_slice(format!("Proxy-Authorization: {auth}\r\n").as_bytes());
                }
                handshake.extend_from_slice(b"\r\n");
                if remote.write_all(&handshake).await.is_err() {
                    return;
                }
                let mut remote_reader = BufReader::new(remote);
                let Ok(response) = read_header_block(&mut remote_reader).await else {
                    return;
                };
                if !successful_connect_response(&response) {
                    return;
                }
                let mut local = reader.into_inner();
                if write_established(&mut local).await.is_err() {
                    return;
                }
                let _ = tokio::io::copy_bidirectional(&mut local, &mut remote_reader).await;
            } else {
                let mut local = reader.into_inner();
                if write_established(&mut local).await.is_err() {
                    return;
                }
                let _ = tokio::io::copy_bidirectional(&mut local, &mut remote).await;
            }
        })
    })
}

fn successful_connect_response(response: &[u8]) -> bool {
    response
        .split(|byte| *byte == b'\n')
        .next()
        .and_then(|line| std::str::from_utf8(line).ok())
        .is_some_and(|line| line.starts_with("HTTP/1.0 2") || line.starts_with("HTTP/1.1 2"))
}

async fn write_established(stream: &mut tokio::net::TcpStream) -> std::io::Result<()> {
    stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await
}

async fn write_error(
    stream: &mut tokio::net::TcpStream,
    status: u16,
    reason: &str,
) -> std::io::Result<()> {
    stream
        .write_all(
            format!("HTTP/1.1 {status} {reason}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n")
                .as_bytes(),
        )
        .await
}

#[derive(Clone)]
pub struct ConnectClientTunnelBackend {
    supervisor: RuntimeSupervisor,
    sam_tcp_port: u16,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
    shared_registry: Option<Arc<super::runtime::session::SharedClientSessionRegistry>>,
    client_destinations: Option<ClientDestinationStore>,
}

impl std::fmt::Debug for ConnectClientTunnelBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectClientTunnelBackend")
            .field("sam_tcp_port", &self.sam_tcp_port)
            .finish_non_exhaustive()
    }
}

impl ConnectClientTunnelBackend {
    pub fn new(sam_tcp_port: u16) -> Self {
        Self {
            supervisor: RuntimeSupervisor::new(),
            sam_tcp_port,
            address_book: None,
            shared_registry: None,
            client_destinations: None,
        }
    }

    pub fn with_address_book(mut self, address_book: Arc<RuntimeAddressBookHandle>) -> Self {
        self.address_book = Some(address_book);
        self
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

    fn config(&self, definition: &TunnelDefinition) -> BackendResult<ConnectConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::ConnectClient,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_options(
            TunnelType::ConnectClient,
            &definition.options,
            CONNECT_CLIENT_OPTIONS,
        )
        .map_err(option_error)?;
        validate_raw_options(definition)?;
        let bind_address = match definition.options.listen_interface.as_deref() {
            None => "127.0.0.1".parse().expect("loopback address is valid"),
            Some(value) => value.parse::<IpAddr>().map_err(|_| BackendError::Internal {
                message: "connectclient listen interface must be an IP address".to_owned(),
            })?,
        };
        let proxy_username = raw_string(definition, "ProxyUsername")
            .or_else(|| definition.options.proxy_username.clone());
        let proxy_password = raw_secret(definition, "ProxyPassword")
            .or_else(|| definition.options.proxy_password.as_deref().map(str::to_owned));
        let proxy_credentials = credentials(proxy_username, proxy_password)?;
        let require_auth = raw_bool(definition, "ProxyAuth")?
            .unwrap_or(proxy_credentials.is_some())
            || !bind_address.is_loopback();
        if require_auth && proxy_credentials.is_none() {
            return Err(BackendError::Internal {
                message: "connectclient non-loopback listeners require proxy authentication"
                    .to_owned(),
            });
        }
        let outproxy =
            raw_string(definition, "ProxyList").as_deref().map(parse_outproxy).transpose()?;
        let outproxy_username = raw_string(definition, "OutproxyUsername");
        let outproxy_password = raw_secret(definition, "OutproxyPassword")
            .or_else(|| definition.options.outproxy_password.as_deref().map(str::to_owned));
        let outproxy_credentials = credentials(outproxy_username, outproxy_password)?;
        if raw_bool(definition, "OutproxyAuth")?.unwrap_or(false) && outproxy_credentials.is_none()
        {
            return Err(BackendError::Internal {
                message: "connectclient outproxy authentication is incomplete".to_owned(),
            });
        }
        let outproxy_authorization = outproxy_credentials
            .as_ref()
            .map(|(username, password)| basic_authorization(username, password));
        let (proxy_username, proxy_password) = proxy_credentials
            .map(|(username, password)| (Some(username), Some(password)))
            .unwrap_or((None, None));
        Ok(ConnectConfig {
            name: definition.name.as_str().to_owned(),
            bind_address,
            port: definition.options.listen_port.ok_or_else(|| BackendError::MissingOption {
                tunnel_type: TunnelType::ConnectClient,
                option: "ListenPort".to_owned(),
            })?,
            sam_tcp_port: self.sam_tcp_port,
            outproxy,
            proxy_username,
            proxy_password,
            require_auth,
            outproxy_authorization,
            address_book: self.address_book.clone(),
            delay_open: definition.options.delay_open.unwrap_or(false),
            session_options: SessionOptions::default(),
        })
    }
}

fn parse_outproxy(value: &str) -> BackendResult<OutproxyTarget> {
    let value = value.trim();
    if value.is_empty() || value.contains(',') {
        return Err(BackendError::UnsupportedOption {
            tunnel_type: TunnelType::ConnectClient,
            option: "ProxyList".to_owned(),
        });
    }
    let (destination, port) = parse_authority(value).map_err(|_| BackendError::Internal {
        message: "connectclient outproxy is invalid".to_owned(),
    })?;
    if !(destination.ends_with(".i2p")
        || destination.ends_with(".b32.i2p")
        || crate::i2pcontrol::address_book_runtime::is_valid_full_destination(&destination))
    {
        return Err(BackendError::Internal {
            message: "connectclient outproxy must be an I2P destination".to_owned(),
        });
    }
    let port = port.unwrap_or(DEFAULT_OUTPROXY_PORT);
    if port == 0 {
        return Err(BackendError::Internal {
            message: "connectclient outproxy port is invalid".to_owned(),
        });
    }
    Ok(OutproxyTarget { destination, port })
}

fn credentials(
    username: Option<String>,
    password: Option<String>,
) -> BackendResult<Option<(String, String)>> {
    match (username, password) {
        (None, None) => Ok(None),
        (Some(username), Some(password))
            if !username.is_empty()
                && !password.is_empty()
                && username.len() <= 255
                && password.len() <= 255 =>
        {
            Ok(Some((username, password)))
        }
        _ => Err(BackendError::Internal {
            message: "connectclient proxy credentials are invalid or incomplete".to_owned(),
        }),
    }
}

fn raw_string(definition: &TunnelDefinition, key: &str) -> Option<String> {
    definition
        .raw_config
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_owned)
}
fn raw_secret(definition: &TunnelDefinition, key: &str) -> Option<String> {
    definition
        .raw_config
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_owned)
}
fn raw_bool(definition: &TunnelDefinition, key: &str) -> Result<Option<bool>, BackendError> {
    definition
        .raw_config
        .get(key)
        .map(|value| {
            value.as_bool().ok_or_else(|| BackendError::UnsupportedOption {
                tunnel_type: TunnelType::ConnectClient,
                option: key.to_owned(),
            })
        })
        .transpose()
}

fn validate_raw_options(definition: &TunnelDefinition) -> BackendResult<()> {
    const SUPPORTED: &[&str] = &[
        "Port",
        "ReachableBy",
        "ProxyAuth",
        "ProxyUsername",
        "ProxyPassword",
        "ProxyList",
        "OutproxyAuth",
        "OutproxyUsername",
        "OutproxyPassword",
        "OutproxyType",
        "Description",
        "StartOnLoad",
        "DelayOpen",
        "Shared",
        "NewDest",
        "PersistentClientKey",
        "PrivKeyFile",
    ];
    for key in definition.raw_config.keys() {
        if key.starts_with("__emissary_") || SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        return Err(BackendError::UnsupportedOption {
            tunnel_type: TunnelType::ConnectClient,
            option: key.clone(),
        });
    }
    if let Some(kind) = raw_string(definition, "OutproxyType") {
        if !kind.eq_ignore_ascii_case("http") && !kind.eq_ignore_ascii_case("connect") {
            return Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::ConnectClient,
                option: "OutproxyType".to_owned(),
            });
        }
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
impl TunnelBackend for ConnectClientTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::ConnectClient
    }
    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        let mut config = self.config(definition)?;
        config.session_options = super::runtime::session::build_client_session_options(
            definition,
            self.sam_tcp_port,
            self.client_destinations.as_ref(),
        )
        .await?;
        self.supervisor
            .start(
                config,
                self.shared_registry.clone(),
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
            tunnel_type: TunnelType::ConnectClient,
            runtime_state,
            message: message.to_owned(),
            destination: None,
        }
    }
}

fn invalid(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_connect_and_no_body_are_accepted() {
        let request = ConnectRequest::parse(
            b"CONNECT peer.b32.i2p:443 HTTP/1.1\r\nHost: peer.b32.i2p:443\r\nX-Leak: no\r\n\r\n",
        )
        .unwrap();
        assert_eq!(request.host, "peer.b32.i2p");
        assert_eq!(request.port, 443);
        assert!(ConnectRequest::parse(b"GET peer.b32.i2p:443 HTTP/1.1\r\n\r\n").is_err());
        assert!(ConnectRequest::parse(
            b"CONNECT peer.b32.i2p:443 HTTP/1.1\r\nContent-Length: 1\r\n\r\n"
        )
        .is_err());
    }

    #[test]
    fn direct_target_cannot_be_local() {
        assert!(classify_target("127.0.0.1", 443, None).is_err());
        assert!(classify_target("peer.b32.i2p", 443, None).is_ok());
    }
}
