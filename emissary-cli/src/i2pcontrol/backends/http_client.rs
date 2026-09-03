//! Control-plane-owned HTTP proxy backend for Proposal 170 `httpclient`.

use std::{collections::HashMap, net::IpAddr, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
    task::JoinHandle,
};

use super::{
    filters::{
        http_client::{
            copy_body, read_header_block, HttpClientPolicy, HttpClientRequest, HttpTarget,
            OutproxyTarget,
        },
        proxy::{basic_authorization, credentials_match},
    },
    options::{validate_options, OptionValidationError, HTTP_CLIENT_OPTIONS},
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    address_book_runtime::{RuntimeAddressBookHandle, RuntimeAddressBookType},
    client_secret_store::ClientDestinationStore,
    backends::runtime::{
        client_lifecycle_config, ClientConnectionHandler, ClientLifecycleConfig,
        ClientListenerRuntimeConfig,
        ClientListenerRuntimeError,
    },
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
};
use yosemite_i2pcontrol::SessionOptions;

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RUNTIME_TASKS: usize = 1000;
const MAX_CONNECTIONS: usize = 128;
const DEFAULT_OUTPROXY_PORT: u16 = 4444;

#[derive(Clone)]
struct HttpClientConfig {
    name: String,
    bind_address: IpAddr,
    port: u16,
    sam_tcp_port: u16,
    outproxy: Option<OutproxyTarget>,
    proxy_username: Option<String>,
    proxy_password: Option<String>,
    require_auth: bool,
    policy: HttpClientPolicy,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
    delay_open: bool,
    lifecycle: ClientLifecycleConfig,
    session_options: SessionOptions,
}

impl std::fmt::Debug for HttpClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClientConfig")
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
            .field("policy", &self.policy)
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
                    tunnel_type: TunnelType::HttpClient,
                    current_state: entry.state,
                    attempted_action: "start",
                });
            }
        }
        if runtime.entries.values().filter(|entry| entry.task.is_some()).count()
            >= MAX_RUNTIME_TASKS
        {
            return Err(BackendError::Internal {
                message: "httpclient runtime capacity exhausted".to_owned(),
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
            entry.failure = Some("httpclient tunnel runtime failed");
        } else {
            entry.state = TunnelRuntimeState::Stopped;
            entry.failure = None;
        }
    }

    async fn start(
        &self,
        config: HttpClientConfig,
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
                    destination: "unused-httpclient-destination".to_owned(),
                    destination_port: 80,
                    sam_tcp_port: config.sam_tcp_port,
                    session_options: config.session_options,
                    delay_open: config.delay_open,
                    connect_delay: config.lifecycle.connect_delay,
                    close_on_idle: config.lifecycle.close_on_idle,
                    close_idle_time: config.lifecycle.close_idle_time,
                    new_dest_on_resume: config.lifecycle.new_dest_on_resume,
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
            let cancelled = *ready_cancellation.borrow();
            Self::complete(map, task_name, generation, result, cancelled);
        });
        self.set_task(&name, generation, task);
        match tokio::time::timeout(START_TIMEOUT, ready_rx).await {
            Ok(Ok(Ok(_))) if self.mark_running(&name, generation) => Ok(()),
            Ok(Ok(Err(_))) | Ok(Err(_)) | Err(_) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "httpclient tunnel runtime failed to start".to_owned(),
                })
            }
            Ok(Ok(Ok(_))) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "httpclient tunnel runtime exited during start".to_owned(),
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
                message: "httpclient tunnel stop timed out".to_owned(),
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
                entry.failure.unwrap_or("httpclient tunnel runtime is active"),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                "httpclient tunnel runtime is stopped",
            ),
        }
    }
}

fn make_handler(config: HttpClientConfig) -> ClientConnectionHandler {
    make_handler_parts(
        config.outproxy,
        config.proxy_username,
        config.proxy_password,
        config.require_auth,
        config.policy,
        config.address_book,
    )
}

/// Build the M068 HTTP client handler with direct-I2P-only routing. The
/// composed backend uses this seam instead of reimplementing request parsing,
/// header sanitization, or response relay behavior.
pub(crate) fn make_no_outproxy_handler(
    proxy_username: Option<String>,
    proxy_password: Option<String>,
    require_auth: bool,
    policy: HttpClientPolicy,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
) -> ClientConnectionHandler {
    make_handler_parts(
        None,
        proxy_username,
        proxy_password,
        require_auth,
        policy,
        address_book,
    )
}

fn make_handler_parts(
    outproxy: Option<OutproxyTarget>,
    proxy_username: Option<String>,
    proxy_password: Option<String>,
    require_auth: bool,
    policy: HttpClientPolicy,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
) -> ClientConnectionHandler {
    Arc::new(move |stream, connector| {
        let outproxy = outproxy.clone();
        let proxy_username = proxy_username.clone();
        let proxy_password = proxy_password.clone();
        let policy = policy.clone();
        let address_book = address_book.clone();
        Box::pin(async move {
            let mut reader = BufReader::new(stream);
            let Ok(header_block) = read_header_block(&mut reader).await else {
                let mut stream = reader.into_inner();
                let _ = write_proxy_error(&mut stream, 400, "Bad Request").await;
                return;
            };
            let Ok(request) = HttpClientRequest::parse(&header_block, outproxy.clone()) else {
                let mut stream = reader.into_inner();
                let _ = write_proxy_error(&mut stream, 400, "Bad Request").await;
                return;
            };
            if require_auth
                && !credentials_match(
                    request.proxy_authorization.as_deref(),
                    proxy_username.as_deref().unwrap_or_default(),
                    proxy_password.as_deref().unwrap_or_default(),
                )
            {
                let mut stream = reader.into_inner();
                let _ = write_proxy_error(&mut stream, 407, "Proxy Authentication Required").await;
                return;
            }
            let Ok(mut target) = request.target(outproxy.clone()) else {
                let mut stream = reader.into_inner();
                let _ = write_proxy_error(&mut stream, 403, "Forbidden").await;
                return;
            };
            let destination = match &mut target {
                HttpTarget::I2p { destination, .. } => {
                    match resolve_destination(destination, address_book.as_ref()).await {
                        Some(resolved) => {
                            *destination = resolved.clone();
                            resolved
                        }
                        None => {
                            let mut stream = reader.into_inner();
                            let _ = write_proxy_error(&mut stream, 502, "Bad Gateway").await;
                            return;
                        }
                    }
                }
                HttpTarget::Clearnet { outproxy, .. } => {
                    match resolve_destination(&outproxy.destination, address_book.as_ref()).await {
                        Some(resolved) => resolved,
                        None => {
                            let mut stream = reader.into_inner();
                            let _ = write_proxy_error(&mut stream, 502, "Bad Gateway").await;
                            return;
                        }
                    }
                }
            };
            let Ok(serialized) = request.serialize(&destination, &target, &policy) else {
                let mut stream = reader.into_inner();
                let _ = write_proxy_error(&mut stream, 400, "Bad Request").await;
                return;
            };
            let Ok(mut remote) = connector
                .connect_to(
                    &destination,
                    match &target {
                        HttpTarget::I2p { port, .. } => *port,
                        HttpTarget::Clearnet { outproxy, .. } => outproxy.port,
                    },
                )
                .await
            else {
                let mut stream = reader.into_inner();
                let _ = write_proxy_error(&mut stream, 502, "Bad Gateway").await;
                return;
            };
            if remote.write_all(&serialized).await.is_err() {
                return;
            }
            if copy_body(&mut reader, &mut remote, request.content_length).await.is_err() {
                return;
            }
            let mut local = reader.into_inner();
            let _ = tokio::time::timeout(
                super::filters::http_client::BODY_TIMEOUT,
                tokio::io::copy(&mut remote, &mut local),
            )
            .await;
            let _ = local.shutdown().await;
        })
    })
}

pub(crate) async fn resolve_destination(
    destination: &str,
    address_book: Option<&Arc<RuntimeAddressBookHandle>>,
) -> Option<String> {
    if destination.ends_with(".b32.i2p")
        || crate::i2pcontrol::address_book_runtime::is_valid_full_destination(destination)
    {
        return Some(destination.to_owned());
    }
    let address_book = address_book?;
    for book in [
        RuntimeAddressBookType::Private,
        RuntimeAddressBookType::Local,
        RuntimeAddressBookType::Router,
        RuntimeAddressBookType::Published,
    ] {
        if let Ok(Some(entry)) = address_book.runtime_lookup(book, destination).await {
            return Some(format!(
                "{}.b32.i2p",
                crate::i2pcontrol::address_book_runtime::base32_for_destination(&entry.destination)
            ));
        }
    }
    None
}

async fn write_proxy_error(
    stream: &mut TcpStream,
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
pub struct HttpClientTunnelBackend {
    supervisor: RuntimeSupervisor,
    sam_tcp_port: u16,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
    shared_registry: Option<Arc<super::runtime::session::SharedClientSessionRegistry>>,
    client_destinations: Option<ClientDestinationStore>,
}

impl std::fmt::Debug for HttpClientTunnelBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClientTunnelBackend")
            .field("sam_tcp_port", &self.sam_tcp_port)
            .finish_non_exhaustive()
    }
}

impl HttpClientTunnelBackend {
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

    fn config(&self, definition: &TunnelDefinition) -> BackendResult<HttpClientConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::HttpClient,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_options(
            TunnelType::HttpClient,
            &definition.options,
            HTTP_CLIENT_OPTIONS,
        )
        .map_err(option_error)?;
        validate_raw_options(definition)?;
        let bind_address = match definition.options.listen_interface.as_deref() {
            None => "127.0.0.1".parse().expect("loopback address is valid"),
            Some(value) => value.parse::<IpAddr>().map_err(|_| BackendError::Internal {
                message: "httpclient listen interface must be an IP address".to_owned(),
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
                message: "httpclient non-loopback listeners require proxy authentication"
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
                message: "httpclient outproxy authentication is incomplete".to_owned(),
            });
        }
        let outproxy_authorization = outproxy_credentials
            .as_ref()
            .map(|(username, password)| basic_authorization(username, password));
        let (proxy_username, proxy_password) = proxy_credentials
            .map(|(username, password)| (Some(username), Some(password)))
            .unwrap_or((None, None));
        Ok(HttpClientConfig {
            name: definition.name.as_str().to_owned(),
            bind_address,
            port: definition.options.listen_port.ok_or_else(|| BackendError::MissingOption {
                tunnel_type: TunnelType::HttpClient,
                option: "ListenPort".to_owned(),
            })?,
            sam_tcp_port: self.sam_tcp_port,
            outproxy,
            proxy_username,
            proxy_password,
            require_auth,
            policy: HttpClientPolicy {
                allow_user_agent: raw_bool(definition, "AllowUserAgent")?.unwrap_or(false),
                allow_referer: raw_bool(definition, "AllowReferer")?.unwrap_or(false)
                    && !raw_bool(definition, "BlockReferers")?.unwrap_or(false),
                allow_accept: raw_bool(definition, "AllowAccept")?.unwrap_or(false),
                outproxy_authorization,
            },
            address_book: self.address_book.clone(),
            delay_open: definition.options.delay_open.unwrap_or(false),
            lifecycle: client_lifecycle_config(definition)?,
            session_options: SessionOptions::default(),
        })
    }
}

fn parse_outproxy(value: &str) -> BackendResult<OutproxyTarget> {
    let value = value.trim();
    if value.is_empty() || value.contains(',') {
        return Err(BackendError::UnsupportedOption {
            tunnel_type: TunnelType::HttpClient,
            option: "ProxyList".to_owned(),
        });
    }
    let (destination, port) =
        super::filters::http_client::parse_authority(value).map_err(|_| {
            BackendError::Internal {
                message: "httpclient outproxy is invalid".to_owned(),
            }
        })?;
    if !(destination.ends_with(".i2p")
        || destination.ends_with(".b32.i2p")
        || crate::i2pcontrol::address_book_runtime::is_valid_full_destination(&destination))
    {
        return Err(BackendError::Internal {
            message: "httpclient outproxy must be an I2P destination".to_owned(),
        });
    }
    let port = port.unwrap_or(DEFAULT_OUTPROXY_PORT);
    if port == 0 {
        return Err(BackendError::Internal {
            message: "httpclient outproxy port is invalid".to_owned(),
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
            message: "httpclient proxy credentials are invalid or incomplete".to_owned(),
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
                tunnel_type: TunnelType::HttpClient,
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
        "AllowUserAgent",
        "AllowReferer",
        "AllowAccept",
        "BlockReferers",
        "Description",
        "StartOnLoad",
        "DelayOpen",
        "ConnectDelay",
        "Shared",
        "PersistentClientKey",
        "PrivKeyFile",
    ];
    // M121: "Close", "CloseTime", and "NewDest" demoted to blocked_primitive.
    for key in definition.raw_config.keys() {
        if key.starts_with("__emissary_") || SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        return Err(BackendError::UnsupportedOption {
            tunnel_type: TunnelType::HttpClient,
            option: key.clone(),
        });
    }
    if raw_bool(definition, "BlockReferers")?.unwrap_or(false) {
        // BlockReferers is a stronger spelling of the default safe policy.
    }
    if let Some(kind) = raw_string(definition, "OutproxyType") {
        if !kind.eq_ignore_ascii_case("http") {
            return Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::HttpClient,
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
impl TunnelBackend for HttpClientTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::HttpClient
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
            tunnel_type: TunnelType::HttpClient,
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

    fn definition() -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("http-client").unwrap(),
            tunnel_type: TunnelType::HttpClient,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions {
                listen_port: Some(0),
                ..Default::default()
            },
            raw_config: Default::default(),
        }
    }

    #[tokio::test]
    async fn unsafe_exposure_and_secret_options_fail_before_runtime_allocation() {
        let backend = HttpClientTunnelBackend::new(1);
        let mut definition = definition();
        definition.options.listen_interface = Some("0.0.0.0".to_owned());
        assert!(matches!(
            backend.start(&definition).await,
            Err(BackendError::Internal { .. })
        ));
        assert_eq!(
            backend.inspect(&definition).runtime_state,
            TunnelRuntimeState::Stopped
        );
        definition.options.listen_interface = None;
        definition.options.proxy_password =
            crate::i2pcontrol::domain::tunnel::OptionRedacted::new("secret");
        assert!(matches!(
            backend.start(&definition).await,
            Err(BackendError::UnsupportedOption { .. } | BackendError::Internal { .. })
        ));
        assert!(!format!("{backend:?}").contains("secret"));
    }

    #[test]
    fn invalid_clearnet_outproxy_is_rejected() {
        let mut definition = definition();
        definition
            .raw_config
            .insert("ProxyList".to_owned(), serde_json::json!("127.0.0.1:4444"));
        assert!(matches!(
            HttpClientTunnelBackend::new(1).config(&definition),
            Err(BackendError::Internal { .. })
        ));
    }

    #[test]
    fn typed_outproxy_secret_is_used_without_leaking_or_accepting_lists() {
        let mut definition = definition();
        definition.raw_config.insert(
            "ProxyList".to_owned(),
            serde_json::json!("outproxy.i2p:4444"),
        );
        definition
            .raw_config
            .insert("OutproxyUsername".to_owned(), serde_json::json!("user"));
        definition.options.outproxy_password =
            crate::i2pcontrol::domain::tunnel::OptionRedacted::new("outproxy-secret");
        let config = HttpClientTunnelBackend::new(1).config(&definition).unwrap();
        assert!(config.policy.outproxy_authorization.is_some());
        assert!(!format!("{config:?}").contains("outproxy-secret"));

        definition.raw_config.insert(
            "ProxyList".to_owned(),
            serde_json::json!("first.i2p:4444,second.i2p:4444"),
        );
        assert!(matches!(
            HttpClientTunnelBackend::new(1).config(&definition),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "ProxyList"
        ));
    }
}
