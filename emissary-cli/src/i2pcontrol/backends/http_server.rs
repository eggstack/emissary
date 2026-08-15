//! Accepted-stream HTTP reverse tunnel for Proposal 170 `httpserver`.

use std::{
    collections::HashMap,
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::TcpStream,
    task::JoinHandle,
};

use super::{
    filters::http::{
        copy_body, copy_response_body, read_and_filter_response, read_and_sanitize_request,
        AccessOption, HttpServerPolicy,
    },
    options::{validate_options, CustomOptionPolicy, OptionCapabilities, OptionValidationError},
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    backends::{
        runtime::{
            run_accepted_server, AcceptedServerConnection, AcceptedServerHandler,
            AcceptedServerRuntimeConfig, AcceptedServerRuntimeError,
        },
        server::SERVER_IDENTITY_KEY,
    },
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
    server_secret_store::{ServerDestinationStore, StoredDestination},
};

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RUNTIME_TASKS: usize = 1000;
const DEFAULT_MAX_CONNECTIONS: usize = 128;
const MAX_THROTTLE_ENTRIES: usize = 1024;
const MAX_POST_LIMIT: usize = 1_000_000;
const MAX_POST_WINDOW: u64 = 86_400;

pub const HTTP_SERVER_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &[],
    &["TargetPort", "ListenPort"],
    &["HostingDestination", "AccessList", "HttpHost"],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Reject,
);

#[derive(Debug, Clone)]
struct HttpServerConfig {
    name: String,
    target_host: String,
    target_port: u16,
    sam_tcp_port: u16,
    destination: StoredDestination,
    max_connections: usize,
    policy: HttpServerPolicy,
    post_limiter: PostLimiter,
}

#[derive(Clone, Debug)]
struct PostLimiter {
    limit: usize,
    window: Duration,
    entries: Arc<Mutex<HashMap<String, (Instant, usize)>>>,
}

impl PostLimiter {
    fn new(limit: usize, window: Duration) -> Self {
        Self {
            limit,
            window,
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn allow(&self, peer: &str) -> bool {
        if self.limit == 0 {
            return true;
        }
        let now = Instant::now();
        let mut entries = self.entries.lock();
        entries.retain(|_, (started, _)| now.duration_since(*started) < self.window);
        let entry = entries.entry(peer.to_owned()).or_insert((now, 0));
        if entry.1 >= self.limit {
            return false;
        }
        entry.1 += 1;
        if entries.len() > MAX_THROTTLE_ENTRIES {
            if let Some(oldest) = entries
                .iter()
                .min_by_key(|(_, (started, _))| *started)
                .map(|(key, _)| key.clone())
            {
                entries.remove(&oldest);
            }
        }
        true
    }
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
struct HttpServerRuntimeSupervisor {
    inner: Arc<Mutex<RuntimeMap>>,
}

impl HttpServerRuntimeSupervisor {
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
                    tunnel_type: TunnelType::HttpServer,
                    current_state: entry.state,
                    attempted_action: "start",
                });
            }
        }
        if runtime.entries.values().filter(|entry| entry.task.is_some()).count()
            >= MAX_RUNTIME_TASKS
        {
            return Err(BackendError::Internal {
                message: "httpserver runtime capacity exhausted".to_owned(),
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
            entry.failure = Some("httpserver tunnel runtime failed");
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
                message: "httpserver tunnel stop timed out".to_owned(),
            });
        }
        self.remove_generation(name, generation);
        Ok(())
    }

    async fn start(&self, config: HttpServerConfig) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.reserve(&name)?;
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let map = Arc::clone(&self.inner);
        let supervisor = self.clone();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let handler: AcceptedServerHandler = Arc::new(move |connection| {
                let target_host = config.target_host.clone();
                let target_port = config.target_port;
                let policy = config.policy.clone();
                let limiter = config.post_limiter.clone();
                Box::pin(async move {
                    let _ =
                        handle_connection(connection, target_host, target_port, policy, limiter)
                            .await;
                })
            });
            let result = std::panic::AssertUnwindSafe(run_accepted_server(
                AcceptedServerRuntimeConfig {
                    name: config.name,
                    sam_tcp_port: config.sam_tcp_port,
                    destination: config.destination,
                    max_connections: config.max_connections,
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
            Ok(Ok(Ok(destination))) => {
                supervisor.publish_destination(&name, generation, &destination);
                if supervisor.mark_running(&name, generation) {
                    Ok(())
                } else {
                    let _ = self.stop_generation(&name, generation).await;
                    Err(BackendError::Internal {
                        message: "httpserver runtime exited during start".to_owned(),
                    })
                }
            }
            Ok(Ok(Err(_))) | Ok(Err(_)) | Err(_) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "httpserver tunnel runtime failed to start".to_owned(),
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
                entry.failure.unwrap_or("httpserver tunnel runtime is active"),
                entry.destination.clone(),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                "httpserver tunnel runtime is stopped",
                None,
            ),
        }
    }
}

async fn handle_connection(
    connection: AcceptedServerConnection,
    target_host: String,
    target_port: u16,
    policy: HttpServerPolicy,
    limiter: PostLimiter,
) -> io::Result<()> {
    let peer = connection.peer.destination().to_owned();
    let (remote_read, remote_write) = tokio::io::split(connection.stream);
    handle_http_stream(
        remote_read,
        remote_write,
        peer,
        target_host,
        target_port,
        policy,
        limiter,
    )
    .await
}

async fn handle_http_stream<R, W>(
    remote_read: R,
    mut remote_write: W,
    peer: String,
    target_host: String,
    target_port: u16,
    policy: HttpServerPolicy,
    limiter: PostLimiter,
) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut remote_reader = BufReader::new(remote_read);
    let request = match read_and_sanitize_request(&mut remote_reader, &peer, &policy).await {
        Ok(request) => request,
        Err(error) => {
            send_error(&mut remote_write, error_kind_status(error.kind())).await?;
            return Ok(());
        }
    };
    if matches!(request.method.as_str(), "POST" | "PUT" | "PATCH") && !limiter.allow(&peer) {
        send_error(&mut remote_write, "429 Too Many Requests").await?;
        return Ok(());
    }

    let local = match tokio::time::timeout(
        CONNECT_TIMEOUT,
        TcpStream::connect((target_host.as_str(), target_port)),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        _ => {
            send_error(&mut remote_write, "502 Bad Gateway").await?;
            return Ok(());
        }
    };
    let (local_read, mut local_write) = tokio::io::split(local);
    local_write.write_all(&request.head).await?;
    copy_body(&mut remote_reader, &mut local_write, request.content_length).await?;
    local_write.shutdown().await?;

    let mut local_reader = BufReader::new(local_read);
    let response = match read_and_filter_response(&mut local_reader).await {
        Ok(response) => response,
        Err(_) => {
            send_error(&mut remote_write, "502 Bad Gateway").await?;
            return Ok(());
        }
    };
    remote_write.write_all(&response.head).await?;
    copy_response_body(&mut local_reader, &mut remote_write, &response).await
}

async fn send_error<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    status: &str,
) -> io::Result<()> {
    let response = format!("HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n");
    writer.write_all(response.as_bytes()).await
}

fn error_kind_status(kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::PermissionDenied => "403 Forbidden",
        io::ErrorKind::TimedOut => "408 Request Timeout",
        io::ErrorKind::InvalidData => "400 Bad Request",
        io::ErrorKind::Unsupported => "501 Not Implemented",
        _ => "400 Bad Request",
    }
}

/// Real backend for the control-plane-owned Proposal 170 `httpserver` type.
#[derive(Clone, Debug)]
pub struct HttpServerTunnelBackend {
    supervisor: HttpServerRuntimeSupervisor,
    destinations: ServerDestinationStore,
    sam_tcp_port: u16,
}

impl HttpServerTunnelBackend {
    pub fn new(sam_tcp_port: u16, destinations: ServerDestinationStore) -> Self {
        Self {
            supervisor: HttpServerRuntimeSupervisor::new(),
            destinations,
            sam_tcp_port,
        }
    }

    async fn config_without_destination(
        &self,
        definition: &TunnelDefinition,
    ) -> BackendResult<HttpServerConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::HttpServer,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_raw_options(definition)?;
        validate_options(
            TunnelType::HttpServer,
            &definition.options,
            HTTP_SERVER_OPTIONS,
        )
        .map_err(option_error)?;
        let target_port = definition
            .options
            .target_port
            .or(definition.options.listen_port)
            .ok_or_else(|| BackendError::MissingOption {
                tunnel_type: TunnelType::HttpServer,
                option: "TargetPort".to_owned(),
            })?;
        let target_host = raw_string(definition, "TargetHost")
            .or_else(|| raw_string(definition, "Host"))
            .or_else(|| definition.options.hosting_destination.clone())
            .unwrap_or_else(|| "127.0.0.1".to_owned());
        if !matches!(target_host.as_str(), "127.0.0.1" | "localhost" | "::1") {
            return Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::HttpServer,
                option: "TargetHost must be loopback".to_owned(),
            });
        }
        let website_host = configured_host(definition)?;
        let access_list = raw_string(definition, "AccessList")
            .or_else(|| definition.options.access_list.clone())
            .map(|value| {
                value
                    .split(',')
                    .map(|entry| entry.trim().to_owned())
                    .filter(|entry| !entry.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|entries| !entries.is_empty());
        let access_option = match raw_string(definition, "AccessOption").as_deref() {
            None | Some("allow") => AccessOption::Allow,
            Some("deny") => AccessOption::Deny,
            Some(_) => {
                return Err(BackendError::Internal {
                    message: "httpserver access option is invalid".to_owned(),
                });
            }
        };
        let max_connections = raw_u64(definition, "MaxConcurrentConns")
            .map_err(|_| invalid_option("MaxConcurrentConns"))?
            .unwrap_or(DEFAULT_MAX_CONNECTIONS as u64);
        if max_connections == 0 || max_connections > DEFAULT_MAX_CONNECTIONS as u64 {
            return Err(invalid_option("MaxConcurrentConns"));
        }
        let post_limit = raw_u64(definition, "PostLimit")
            .map_err(|_| invalid_option("PostLimit"))?
            .unwrap_or(0);
        let post_window = raw_u64(definition, "PostLimitTime")
            .map_err(|_| invalid_option("PostLimitTime"))?
            .unwrap_or(60);
        if post_limit > MAX_POST_LIMIT as u64
            || post_limit > 0 && !(1..=MAX_POST_WINDOW).contains(&post_window)
        {
            return Err(invalid_option("PostLimit/PostLimitTime"));
        }
        Ok(HttpServerConfig {
            name: definition.name.as_str().to_owned(),
            target_host,
            target_port,
            sam_tcp_port: self.sam_tcp_port,
            destination: StoredDestination::from_private(String::new()),
            max_connections: max_connections as usize,
            policy: HttpServerPolicy {
                website_host,
                block_access_in_proxies: raw_bool(definition, "BlockAccessInProxies")?
                    .unwrap_or(false),
                block_referers: raw_bool(definition, "BlockReferers")?.unwrap_or(false),
                allow_referer: raw_bool(definition, "AllowReferer")?.unwrap_or(true),
                block_user_agents: raw_bool(definition, "BlockUserAgents")?.unwrap_or(false),
                allow_user_agent: raw_bool(definition, "AllowUserAgent")?.unwrap_or(true),
                user_agents: raw_string(definition, "UserAgents").map(|value| {
                    value
                        .split(',')
                        .map(|entry| entry.trim().to_owned())
                        .filter(|entry| !entry.is_empty())
                        .collect()
                }),
                access_list,
                access_option,
            },
            post_limiter: PostLimiter::new(post_limit as usize, Duration::from_secs(post_window)),
        })
    }
}

#[async_trait::async_trait]
impl TunnelBackend for HttpServerTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::HttpServer
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        let mut config = self.config_without_destination(definition).await?;
        let identity = definition
            .raw_config
            .get(SERVER_IDENTITY_KEY)
            .and_then(|value| value.as_str())
            .ok_or_else(|| BackendError::Internal {
                message: "httpserver destination identity is not allocated".to_owned(),
            })?;
        config.destination = self
            .destinations
            .get(identity)
            .await
            .map_err(|_| BackendError::Internal {
                message: "httpserver destination store lookup failed".to_owned(),
            })?
            .ok_or_else(|| BackendError::Internal {
                message: "httpserver destination identity is unavailable".to_owned(),
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
            tunnel_type: TunnelType::HttpServer,
            runtime_state,
            message: message.to_owned(),
            destination,
        }
    }
}

fn configured_host(definition: &TunnelDefinition) -> BackendResult<String> {
    let website =
        raw_string(definition, "WebsiteHostname").or_else(|| definition.options.http_host.clone());
    let spoofed = raw_string(definition, "SpoofedHost");
    if website.is_some() && spoofed.is_some() && website != spoofed {
        return Err(invalid_option("WebsiteHostname/SpoofedHost"));
    }
    let host = spoofed.or(website).unwrap_or_else(|| "localhost".to_owned());
    if host.is_empty()
        || host.len() > 255
        || host.chars().any(|character| {
            character.is_control() || character.is_whitespace() || character == '/'
        })
    {
        return Err(invalid_option("WebsiteHostname"));
    }
    Ok(host)
}

fn raw_string(definition: &TunnelDefinition, key: &str) -> Option<String> {
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
        .map(|value| value.as_bool().ok_or_else(|| invalid_option(key)))
        .transpose()
}

fn raw_u64(definition: &TunnelDefinition, key: &str) -> Result<Option<u64>, BackendError> {
    definition
        .raw_config
        .get(key)
        .map(|value| value.as_u64().ok_or_else(|| invalid_option(key)))
        .transpose()
}

fn validate_raw_options(definition: &TunnelDefinition) -> BackendResult<()> {
    const SUPPORTED: &[&str] = &[
        "Port",
        "TargetPort",
        "TargetHost",
        "Host",
        "WebsiteHostname",
        "SpoofedHost",
        "BlockAccessInProxies",
        "BlockUserAgents",
        "UserAgents",
        "AllowUserAgent",
        "BlockReferers",
        "AllowReferer",
        "AccessOption",
        "AccessList",
        "MaxConcurrentConns",
        "PostLimit",
        "PostLimitTime",
        "HostingDestination",
        "i2p.tunnel.httpHost",
        "i2p.tunnel.accessList",
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
    const REJECTED: &[&str] = &[
        "TargetDestination",
        "Destination",
        "UseSSL",
        "SSLProxies",
        "SSLCertificate",
        "SSLKey",
        "ProxyList",
        "ProxyAuth",
        "ProxyUsername",
        "ProxyPassword",
        "OutproxyAuth",
        "OutproxyUsername",
        "OutproxyPassword",
        "OutproxyType",
        "UseOutproxyPlugin",
        "AllowAccept",
        "AllowInternalSSL",
        "UniqueLocalAddressPerClient",
        "FilterFilePath",
        "ClientPerMinute",
        "ClientPerHour",
        "ClientPerDay",
        "TotalInPerMinute",
        "TotalInPerHour",
        "TotalInPerDay",
        "PerClientPeriod",
        "TotalPeriod",
        "TotalBanTime",
        "TunnelLength",
        "TunnelVariance",
        "TunnelQuantity",
        "TunnelBackupQuantity",
        "SigType",
        "EncType",
        "EncryptLeaseSet",
        "LeaseSetClientAuths",
        "CustomOptions",
        "i2cp",
    ];
    for (key, value) in &definition.raw_config {
        if key.starts_with("__emissary_")
            || METADATA.contains(&key.as_str())
            || SUPPORTED.contains(&key.as_str())
        {
            continue;
        }
        if REJECTED.contains(&key.as_str()) || key.starts_with("i2p.tunnel.") {
            return Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::HttpServer,
                option: key.clone(),
            });
        }
        let _ = value;
    }
    Ok(())
}

fn invalid_option(option: &str) -> BackendError {
    BackendError::Internal {
        message: format!("httpserver option {option} is invalid"),
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
    use super::*;
    use crate::i2pcontrol::domain::tunnel::{StartIntent, TunnelName, TunnelOptions};
    use tokio::io::{duplex, AsyncReadExt, AsyncWriteExt};

    fn definition(raw_config: &[(&str, serde_json::Value)]) -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("http-server").unwrap(),
            tunnel_type: TunnelType::HttpServer,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions {
                target_port: Some(8080),
                ..Default::default()
            },
            raw_config: raw_config
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect(),
        }
    }

    #[tokio::test]
    async fn option_validation_rejects_unsupported_security_modes_before_destination_lookup() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        let result = backend
            .config_without_destination(&definition(&[("UseSSL", serde_json::json!(true))]))
            .await;
        assert!(matches!(
            result,
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::HttpServer,
                option
            }) if option == "UseSSL"
        ));
    }

    #[tokio::test]
    async fn target_host_is_loopback_confined() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        let result = backend
            .config_without_destination(&definition(&[(
                "TargetHost",
                serde_json::json!("10.0.0.1"),
            )]))
            .await;
        assert!(matches!(
            result,
            Err(BackendError::UnsupportedOption { .. })
        ));
    }

    #[test]
    fn post_limiter_is_bounded_and_peer_keyed() {
        let limiter = PostLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.allow("peer-a"));
        assert!(!limiter.allow("peer-a"));
        assert!(limiter.allow("peer-b"));
    }

    #[tokio::test]
    async fn end_to_end_path_sanitizes_before_local_connect_and_filters_response() {
        let local_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = local_listener.local_addr().unwrap().port();
        let local = tokio::spawn(async move {
            let (stream, _) = local_listener.accept().await.unwrap();
            let (mut reader, mut writer) = tokio::io::split(stream);
            let mut request = Vec::new();
            reader.read_to_end(&mut request).await.unwrap();
            let request = String::from_utf8(request).unwrap();
            assert!(request.starts_with("GET /safe HTTP/1.1\r\n"));
            assert!(!request.contains("attacker"));
            assert!(request.contains("Host: configured.i2p\r\n"));
            writer
                .write_all(b"HTTP/1.1 200 OK\r\nServer: hidden\r\nContent-Length: 2\r\n\r\nok")
                .await
                .unwrap();
            writer.shutdown().await.unwrap();
        });

        let (mut client, server) = duplex(64 * 1024);
        let (server_read, server_write) = tokio::io::split(server);
        let task = tokio::spawn(handle_http_stream(
            server_read,
            server_write,
            "peer-destination".to_owned(),
            "127.0.0.1".to_owned(),
            local_port,
            HttpServerPolicy {
                website_host: "configured.i2p".to_owned(),
                ..HttpServerPolicy {
                    website_host: "localhost".to_owned(),
                    block_access_in_proxies: false,
                    block_referers: false,
                    allow_referer: true,
                    block_user_agents: false,
                    allow_user_agent: true,
                    user_agents: None,
                    access_list: None,
                    access_option: AccessOption::Allow,
                }
            },
            PostLimiter::new(0, Duration::from_secs(60)),
        ));
        client
            .write_all(b"GET /safe HTTP/1.1\r\nHost: evil.i2p\r\nx-i2p-destb64: attacker\r\n\r\n")
            .await
            .unwrap();
        client.shutdown().await.unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).await.unwrap();
        let response = String::from_utf8(response).unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(!response.contains("hidden"));
        assert!(response.ends_with("\r\n\r\nok"));
        task.await.unwrap().unwrap();
        local.await.unwrap();
    }
}
