//! Accepted-stream HTTP reverse tunnel for Proposal 170 `httpserver`.

use std::{
    collections::{HashMap, VecDeque},
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpSocket, TcpStream},
    task::JoinHandle,
    time::Instant,
};
use yosemite_i2pcontrol::{DestinationKind, SessionOptions};

use super::{
    filters::http::{
        copy_body, copy_response_body, read_and_filter_response, read_and_sanitize_request,
        AccessOption, HttpServerPolicy,
    },
    options::{
        validate_common_options, validate_options, CustomOptionPolicy, OptionCapabilities,
        OptionValidationError,
    },
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    backends::{
        runtime::{
            run_accepted_server, AcceptedServerConnection, AcceptedServerHandler,
            AcceptedServerRuntimeConfig, AcceptedServerRuntimeError, ServerAccessPolicy,
            ServerAdmissionPolicy, TrustedPeerIdentity,
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
    target_address: IpAddr,
    target_port: u16,
    unique_local: bool,
    sam_tcp_port: u16,
    destination: StoredDestination,
    admission: ServerAdmissionPolicy,
    access: ServerAccessPolicy,
    policy: HttpServerPolicy,
    post_limiter: PostLimiter,
    session_options: SessionOptions,
}

#[derive(Clone, Debug)]
pub(crate) struct PostLimiter {
    limit: usize,
    window: Duration,
    state: Arc<Mutex<PostLimiterState>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PostPeerKey([u8; 32]);

impl PostPeerKey {
    fn from_peer(peer: &TrustedPeerIdentity) -> Self {
        Self(*peer.canonical_id())
    }
}

#[derive(Debug)]
struct PostEntry {
    started: Instant,
    count: usize,
}

#[derive(Clone, Copy, Debug)]
struct PostExpiry {
    at: Instant,
    key: PostPeerKey,
}

#[derive(Debug, Default)]
struct PostLimiterState {
    entries: HashMap<PostPeerKey, PostEntry>,
    expirations: VecDeque<PostExpiry>,
}

impl PostLimiterState {
    fn reap(&mut self, now: Instant) {
        while self.expirations.front().is_some_and(|expiry| expiry.at <= now) {
            let expiry = self.expirations.pop_front().expect("front exists");
            let remove = self
                .entries
                .get(&expiry.key)
                .is_some_and(|entry| entry.started <= expiry.at && expiry.at <= now);
            if remove {
                self.entries.remove(&expiry.key);
            }
        }
    }
}

impl PostLimiter {
    pub(crate) fn new(limit: usize, window: Duration) -> Self {
        Self {
            limit,
            window,
            state: Arc::new(Mutex::new(PostLimiterState::default())),
        }
    }

    fn allow(&self, peer: &TrustedPeerIdentity) -> bool {
        if self.limit == 0 {
            return true;
        }
        let now = Instant::now();
        let key = PostPeerKey::from_peer(peer);
        let mut state = self.state.lock();
        state.reap(now);
        if let Some(entry) = state.entries.get_mut(&key) {
            if entry.count >= self.limit {
                return false;
            }
            entry.count += 1;
            return true;
        }
        if state.entries.len() >= MAX_THROTTLE_ENTRIES {
            // Active/unexpired state is never evicted to admit attacker-
            // controlled identity churn.  Expired state was reclaimed above.
            return false;
        }
        state.entries.insert(
            key,
            PostEntry {
                started: now,
                count: 1,
            },
        );
        state.expirations.push_back(PostExpiry {
            at: now + self.window,
            key,
        });
        true
    }

    #[cfg(test)]
    fn state_sizes(&self) -> (usize, usize) {
        let state = self.state.lock();
        (state.entries.len(), state.expirations.len())
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
            let handler = make_accepted_handler(
                config.target_address,
                config.target_port,
                config.policy.clone(),
                config.post_limiter.clone(),
                config.unique_local,
            );
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
    target_address: IpAddr,
    target_port: u16,
    policy: HttpServerPolicy,
    limiter: PostLimiter,
    unique_local: bool,
) -> io::Result<()> {
    let peer = connection.peer;
    let (remote_read, remote_write) = tokio::io::split(connection.stream);
    handle_http_stream(
        remote_read,
        remote_write,
        peer,
        target_address,
        target_port,
        policy,
        limiter,
        unique_local,
    )
    .await
}

/// Build the accepted-stream handler used by both `httpserver` and the
/// composed `httpbidirserver` backend. Keeping this seam here ensures the
/// composite cannot accidentally grow a second HTTP server filter path.
pub(crate) fn make_accepted_handler(
    target_address: IpAddr,
    target_port: u16,
    policy: HttpServerPolicy,
    limiter: PostLimiter,
    unique_local: bool,
) -> AcceptedServerHandler {
    Arc::new(move |connection| {
        let target_address = target_address;
        let policy = policy.clone();
        let limiter = limiter.clone();
        Box::pin(async move {
            let _ = handle_connection(
                connection,
                target_address,
                target_port,
                policy,
                limiter,
                unique_local,
            )
            .await;
        })
    })
}

#[allow(clippy::too_many_arguments)]
async fn handle_http_stream<R, W>(
    remote_read: R,
    mut remote_write: W,
    peer: TrustedPeerIdentity,
    target_address: IpAddr,
    target_port: u16,
    policy: HttpServerPolicy,
    limiter: PostLimiter,
    unique_local: bool,
) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut remote_reader = BufReader::new(remote_read);
    let request = match read_and_sanitize_request(&mut remote_reader, &peer, &policy).await {
        Ok(request) => request,
        Err(error) => {
            send_error(&mut remote_write, error.status_line()).await?;
            return Ok(());
        }
    };
    if matches!(request.method.as_str(), "POST" | "PUT" | "PATCH") && !limiter.allow(&peer) {
        send_error(&mut remote_write, "429 Too Many Requests").await?;
        return Ok(());
    }

    let local = match connect_to_target(target_address, target_port, &peer, unique_local).await {
        Ok(stream) => stream,
        Err(_) => {
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

/// Normalize the small, pre-existing local-target compatibility surface to a
/// literal address before any accepted stream can reach socket connection.
pub(crate) fn normalize_loopback_target(host: &str, allow_ipv6: bool) -> Option<IpAddr> {
    match host {
        "127.0.0.1" | "localhost" => Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        "::1" if allow_ipv6 => Some(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        _ => None,
    }
}

/// Derive the reference-compatible per-client local source address from the
/// canonical validated Destination hash.
///
/// Pinned Java `I2PTunnelServer.getSocket` behavior
/// (`I2PTunnelServer.java` at `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`,
/// `PROP_UNIQUE_LOCAL` / `enableUniqueLocal` branch):
///
/// - IPv4 loopback target: `127.<hash[0]>.<hash[1]>.<hash[2]>`;
/// - IPv6 loopback target: `fd` followed by the first 15 hash bytes
///   (`addr[0] = 0xfd; copy hash[0..15] into addr[1..16]`).
///
/// The caller must have already proven `target` is literal loopback via
/// [`normalize_loopback_target`]. The returned address is per connection and
/// never stored; collisions are allowed exactly as the reference allows them.
pub(crate) fn derive_unique_local_source(canonical_id: &[u8; 32], target: IpAddr) -> IpAddr {
    match target {
        IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::new(127, canonical_id[0], canonical_id[1], canonical_id[2])),
        IpAddr::V6(_) => {
            let mut octets = [0u8; 16];
            octets[0] = 0xfd;
            octets[1..].copy_from_slice(&canonical_id[..15]);
            IpAddr::V6(Ipv6Addr::from(octets))
        }
    }
}

/// Extract the exact Proposal `UniqueLocalAddressPerClient` boolean from raw
/// config before any accepted-stream allocation.
///
/// Absent means explicitly disabled. `false` is an explicit disabled value and
/// never fails merely because `true` has special behavior. Any present
/// non-boolean value fails before allocation.
pub(crate) fn unique_local_enabled(definition: &TunnelDefinition) -> BackendResult<bool> {
    match definition.raw_config.get("UniqueLocalAddressPerClient") {
        None => Ok(false),
        Some(value) => value.as_bool().ok_or_else(|| invalid_option("UniqueLocalAddressPerClient")),
    }
}

/// Connect to the already-normalized literal-loopback target, source-binding
/// the reference-derived local address first when `unique_local` is enabled.
///
/// Disabled behavior is the ordinary loopback connect. Enabled behavior binds
/// `derive_unique_local_source` with port 0 via `TcpSocket` before `connect`,
/// preserving [`CONNECT_TIMEOUT`]. No listener is bound, no registry is
/// allocated, no DNS is performed, and no fallback source is used: a failed
/// bind/connect returns `Err` so the caller can fail that accepted connection
/// closed without touching tunnel generation state.
async fn connect_to_target(
    target_address: IpAddr,
    target_port: u16,
    peer: &TrustedPeerIdentity,
    unique_local: bool,
) -> io::Result<TcpStream> {
    if !unique_local {
        let inner = tokio::time::timeout(
            CONNECT_TIMEOUT,
            TcpStream::connect(SocketAddr::new(target_address, target_port)),
        )
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "target connect timeout"))?;
        return inner;
    }
    let source = derive_unique_local_source(peer.canonical_id(), target_address);
    let socket = match target_address {
        IpAddr::V4(_) => TcpSocket::new_v4()?,
        IpAddr::V6(_) => TcpSocket::new_v6()?,
    };
    socket.bind(SocketAddr::new(source, 0))?;
    let inner = tokio::time::timeout(
        CONNECT_TIMEOUT,
        socket.connect(SocketAddr::new(target_address, target_port)),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "target connect timeout"))?;
    inner
}

async fn send_error<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    status: &str,
) -> io::Result<()> {
    let response = format!("HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n");
    writer.write_all(response.as_bytes()).await
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

    fn config_without_destination(
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
        let target_host = raw_string(definition, "TargetHost")?
            .or(raw_string(definition, "Host")?)
            .unwrap_or_else(|| "127.0.0.1".to_owned());
        let target_address = normalize_loopback_target(&target_host, true).ok_or_else(|| {
            BackendError::UnsupportedOption {
                tunnel_type: TunnelType::HttpServer,
                option: "TargetHost must be loopback".to_owned(),
            }
        })?;
        let unique_local = unique_local_enabled(definition)?;
        let website_host = configured_host(definition)?;
        let access_list_value = raw_string(definition, "AccessList")?
            .or_else(|| definition.options.access_list.clone());
        let access_list = access_list_value
            .clone()
            .map(|value| {
                value
                    .split(',')
                    .map(|entry| entry.trim().to_owned())
                    .filter(|entry| !entry.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|entries| !entries.is_empty());
        let access_option = match raw_string(definition, "AccessOption")?.as_deref() {
            None | Some("allow") => AccessOption::Allow,
            Some("deny") => AccessOption::Deny,
            Some(_) => {
                return Err(BackendError::Internal {
                    message: "httpserver access option is invalid".to_owned(),
                });
            }
        };
        let filter_file = raw_string(definition, "FilterFilePath")?;
        if filter_file.is_some() && access_list.is_some() {
            return Err(invalid_option("AccessList/FilterFilePath"));
        }
        let access = match filter_file {
            Some(path) => ServerAccessPolicy::from_filter_file(
                self.destinations.directory().parent().unwrap_or(self.destinations.directory()),
                &path,
            ),
            None => ServerAccessPolicy::from_values(
                Some(match access_option {
                    AccessOption::Allow => "allow",
                    AccessOption::Deny => "deny",
                }),
                access_list_value.as_deref(),
            ),
        }
        .map_err(invalid_option)?;
        let admission = ServerAdmissionPolicy::from_raw_options(&definition.raw_config)
            .map_err(invalid_option)?;
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
            target_address,
            target_port,
            unique_local,
            sam_tcp_port: self.sam_tcp_port,
            destination: StoredDestination::from_private(String::new()),
            admission,
            access,
            policy: HttpServerPolicy {
                website_host,
                block_access_in_proxies: raw_bool(definition, "BlockAccessInProxies")?
                    .unwrap_or(false),
                block_referers: raw_bool(definition, "BlockReferers")?.unwrap_or(false),
                allow_referer: raw_bool(definition, "AllowReferer")?.unwrap_or(true),
                allow_accept: raw_bool(definition, "AllowAccept")?.unwrap_or(true),
                block_user_agents: raw_bool(definition, "BlockUserAgents")?.unwrap_or(false),
                allow_user_agent: raw_bool(definition, "AllowUserAgent")?.unwrap_or(true),
                user_agents: raw_string(definition, "UserAgents")?.map(|value| {
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
            session_options: SessionOptions::default(),
        })
    }
}

#[async_trait::async_trait]
impl TunnelBackend for HttpServerTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::HttpServer
    }

    fn validate_start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        // Pure preflight mirroring `start` before identity/store/runtime work.
        // Filter-file reads are bounded local filesystem validation, never
        // network I/O, secret generation/import, or runtime reservation.
        validate_common_options(TunnelType::HttpServer, &definition.options)
            .map_err(option_error)?;
        let _ = self.config_without_destination(definition)?;
        let _ = super::runtime::session::build_session_options(
            definition,
            self.sam_tcp_port,
            true,
            DestinationKind::Transient,
        )?;
        Ok(())
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        // Reuse the exact preflight helpers so validation cannot drift.
        self.validate_start(definition)?;
        let mut config = self.config_without_destination(definition)?;
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
            tunnel_type: TunnelType::HttpServer,
            runtime_state,
            message: message.to_owned(),
            destination,
        }
    }
}

fn configured_host(definition: &TunnelDefinition) -> BackendResult<String> {
    let website =
        raw_string(definition, "WebsiteHostname")?.or_else(|| definition.options.http_host.clone());
    let spoofed = raw_string(definition, "SpoofedHost")?;
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

fn raw_string(definition: &TunnelDefinition, key: &str) -> BackendResult<Option<String>> {
    definition
        .raw_config
        .get(key)
        .map(|value| value.as_str().map(str::to_owned).ok_or_else(|| invalid_option(key)))
        .transpose()
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
        "AllowAccept",
        "AccessOption",
        "AccessList",
        "FilterFilePath",
        "MaxConcurrentConns",
        "ClientPerMinute",
        "ClientPerHour",
        "ClientPerDay",
        "TotalInPerMinute",
        "TotalInPerHour",
        "TotalInPerDay",
        "PostLimit",
        "PostLimitTime",
        "PerClientPeriod",
        "TotalPeriod",
        "TotalBanTime",
        "HostingDestination",
        "UniqueLocalAddressPerClient",
        "i2p.tunnel.httpHost",
        "i2p.tunnel.accessList",
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
        "AllowInternalSSL",
        "TunnelLength",
        "TunnelVariance",
        "TunnelQuantity",
        "TunnelBackupQuantity",
        "SigType",
        "EncType",
        "EncryptLeaseSet",
        "LeaseSetClientAuths",
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
    // Boolean-typed extraction fails before allocation on malformed values.
    let _ = unique_local_enabled(definition)?;
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
    use crate::i2pcontrol::{
        backends::runtime::peer_identity::test_fixtures::distinct_peer,
        domain::tunnel::{StartIntent, TunnelName, TunnelOptions},
    };
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
    async fn preflight_matches_start_without_identity_or_session() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        assert!(backend.validate_start(&definition(&[])).is_ok());

        let bad = definition(&[("TargetDestination", serde_json::json!("bogus"))]);
        assert!(matches!(
            backend.validate_start(&bad),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "TargetDestination"
        ));
        assert!(matches!(
            backend.start(&bad).await,
            Err(BackendError::UnsupportedOption { option, .. }) if option == "TargetDestination"
        ));
    }

    #[tokio::test]
    async fn option_validation_rejects_unsupported_security_modes_before_destination_lookup() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        let result = backend
            .config_without_destination(&definition(&[("UseSSL", serde_json::json!(true))]));
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
            )]));
        assert!(matches!(
            result,
            Err(BackendError::UnsupportedOption { .. })
        ));
    }

    #[tokio::test]
    async fn target_host_is_normalized_to_a_literal_loopback_address() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));

        let localhost = backend
            .config_without_destination(&definition(&[(
                "TargetHost",
                serde_json::json!("localhost"),
            )]))
            .unwrap();
        assert_eq!(localhost.target_address, IpAddr::V4(Ipv4Addr::LOCALHOST));

        let ipv6 = backend
            .config_without_destination(&definition(&[("TargetHost", serde_json::json!("::1"))]))
            .unwrap();
        assert_eq!(ipv6.target_address, IpAddr::V6(Ipv6Addr::LOCALHOST));
    }

    #[tokio::test]
    async fn persisted_public_destination_does_not_select_local_target() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        let mut definition = definition(&[]);
        definition.options.hosting_destination = Some("published-server-destination".to_owned());
        let config = backend.config_without_destination(&definition).unwrap();
        assert_eq!(config.target_address, IpAddr::V4(Ipv4Addr::LOCALHOST));
    }

    #[tokio::test]
    async fn malformed_raw_string_option_fails_before_allocation() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        let result = backend
            .config_without_destination(&definition(&[("TargetHost", serde_json::json!(true))]));
        assert!(matches!(
            result,
            Err(BackendError::Internal { message }) if message.contains("TargetHost")
        ));
    }

    #[tokio::test]
    async fn admission_options_use_reference_defaults_and_validate_before_allocation() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        let config = backend
            .config_without_destination(&definition(&[(
                "MaxConcurrentConns",
                serde_json::json!(7),
            )]))
            .unwrap();
        assert_eq!(config.admission.max_concurrent_connections(), 7);

        let invalid = backend
            .config_without_destination(&definition(&[(
                "MaxConcurrentConns",
                serde_json::json!(0),
            )]));
        assert!(
            matches!(invalid, Err(BackendError::Internal { message }) if message.contains("MaxConcurrentConns"))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn post_limiter_is_bounded_and_peer_keyed() {
        let limiter = PostLimiter::new(1, Duration::from_secs(60));
        let peer_a = distinct_peer(0xA0);
        let peer_b = distinct_peer(0xB0);
        assert!(limiter.allow(&peer_a));
        assert!(!limiter.allow(&peer_a));
        assert!(limiter.allow(&peer_b));
    }

    #[tokio::test(start_paused = true)]
    async fn post_limiter_keys_distinct_peers_independently() {
        let limiter = PostLimiter::new(1, Duration::from_secs(60));
        let peer_a = distinct_peer(0xA1);
        let peer_b = distinct_peer(0xB1);
        assert!(limiter.allow(&peer_a));
        assert!(!limiter.allow(&peer_a));
        assert!(limiter.allow(&peer_b));
        assert_eq!(limiter.state_sizes(), (2, 2));
    }

    #[tokio::test(start_paused = true)]
    async fn post_limiter_denies_churn_without_evicting_active_entries() {
        use crate::i2pcontrol::backends::runtime::peer_identity::test_fixtures::distinct_peer_u32;
        let limiter = PostLimiter::new(1, Duration::from_secs(60));
        let peers: Vec<_> = (0..MAX_THROTTLE_ENTRIES as u32).map(distinct_peer_u32).collect();
        for peer in &peers {
            assert!(limiter.allow(peer));
        }
        assert_eq!(
            limiter.state_sizes(),
            (MAX_THROTTLE_ENTRIES, MAX_THROTTLE_ENTRIES)
        );
        let new_peer = distinct_peer(0xFE);
        assert!(!limiter.allow(&new_peer));
        assert!(!limiter.allow(&peers[0]));
        assert_eq!(
            limiter.state_sizes(),
            (MAX_THROTTLE_ENTRIES, MAX_THROTTLE_ENTRIES)
        );

        tokio::time::advance(Duration::from_secs(60)).await;
        assert!(limiter.allow(&new_peer));
        assert_eq!(limiter.state_sizes(), (1, 1));
    }

    #[tokio::test(start_paused = true)]
    async fn post_limiter_counts_only_write_methods() {
        let limiter = PostLimiter::new(1, Duration::from_secs(60));
        let peer = distinct_peer(0xC0);
        assert!(limiter.allow(&peer));
        assert!(!limiter.allow(&peer));
        assert_eq!(limiter.state_sizes(), (1, 1));
    }

    #[tokio::test]
    async fn rejected_post_does_not_connect_to_local_backend() {
        let local_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = local_listener.local_addr().unwrap().port();
        let local = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(100), local_listener.accept()).await
        });
        let (mut client, server) = duplex(64 * 1024);
        let (server_read, server_write) = tokio::io::split(server);
        let limiter = PostLimiter::new(1, Duration::from_secs(60));
        let peer = distinct_peer(0xD0);
        assert!(limiter.allow(&peer));
        let task = tokio::spawn(handle_http_stream(
            server_read,
            server_write,
            peer,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port,
            HttpServerPolicy {
                website_host: "localhost".to_owned(),
                ..HttpServerPolicy {
                    website_host: "localhost".to_owned(),
                    block_access_in_proxies: false,
                    block_referers: false,
                    allow_referer: true,
                    allow_accept: true,
                    block_user_agents: false,
                    allow_user_agent: true,
                    user_agents: None,
                    access_list: None,
                    access_option: AccessOption::Allow,
                }
            },
            limiter,
            false,
        ));
        client
            .write_all(b"POST /write HTTP/1.1\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
        client.shutdown().await.unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).await.unwrap();
        assert!(String::from_utf8(response)
            .unwrap()
            .starts_with("HTTP/1.1 429 Too Many Requests\r\n"));
        task.await.unwrap().unwrap();
        assert!(local.await.unwrap().is_err());
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
        let peer = distinct_peer(0xE0);
        let task = tokio::spawn(handle_http_stream(
            server_read,
            server_write,
            peer,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port,
            HttpServerPolicy {
                website_host: "configured.i2p".to_owned(),
                ..HttpServerPolicy {
                    website_host: "localhost".to_owned(),
                    block_access_in_proxies: false,
                    block_referers: false,
                    allow_referer: true,
                    allow_accept: true,
                    block_user_agents: false,
                    allow_user_agent: true,
                    user_agents: None,
                    access_list: None,
                    access_option: AccessOption::Allow,
                }
            },
            PostLimiter::new(0, Duration::from_secs(60)),
            false,
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

    #[tokio::test]
    async fn expect_request_is_rejected_with_417_before_local_allocation() {
        let local_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = local_listener.local_addr().unwrap().port();
        let local = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(150), local_listener.accept()).await
        });
        let (mut client, server) = duplex(64 * 1024);
        let (server_read, server_write) = tokio::io::split(server);
        let task = tokio::spawn(handle_http_stream(
            server_read,
            server_write,
            distinct_peer(0xE1),
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port,
            HttpServerPolicy {
                website_host: "configured.i2p".to_owned(),
                ..HttpServerPolicy {
                    website_host: "localhost".to_owned(),
                    block_access_in_proxies: false,
                    block_referers: false,
                    allow_referer: true,
                    allow_accept: true,
                    block_user_agents: false,
                    allow_user_agent: true,
                    user_agents: None,
                    access_list: None,
                    access_option: AccessOption::Allow,
                }
            },
            PostLimiter::new(0, Duration::from_secs(60)),
            false,
        ));
        client
            .write_all(
                b"POST /upload HTTP/1.1\r\nExpect: 100-continue\r\nContent-Length: 0\r\n\r\n",
            )
            .await
            .unwrap();
        client.shutdown().await.unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).await.unwrap();
        let response = String::from_utf8(response).unwrap();
        assert!(
            response.starts_with("HTTP/1.1 417 Expectation Failed\r\n"),
            "got: {response}"
        );
        assert!(response.contains("Connection: close\r\n"));
        assert!(response.ends_with("\r\n\r\n"));
        task.await.unwrap().unwrap();
        assert!(
            local.await.unwrap().is_err(),
            "local backend must not be connected for Expect requests"
        );
    }

    #[test]
    fn unique_local_ipv4_derivation_matches_java_byte_layout() {
        // Java: addr[0]=127; copy hash[0..3] into addr[1..4].
        let mut hash = [0u8; 32];
        hash[0] = 0x01;
        hash[1] = 0x02;
        hash[2] = 0x03;
        let source = derive_unique_local_source(
            &hash,
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        );
        assert_eq!(source, IpAddr::V4(Ipv4Addr::new(127, 0x01, 0x02, 0x03)));

        // Edge bytes .0 and .255 are preserved byte-for-byte, not clamped.
        let mut edge = [0u8; 32];
        edge[0] = 0x00;
        edge[1] = 0xff;
        edge[2] = 0x00;
        assert_eq!(
            derive_unique_local_source(&edge, IpAddr::V4(Ipv4Addr::LOCALHOST)),
            IpAddr::V4(Ipv4Addr::new(127, 0, 255, 0))
        );
        let mut broadcast = [0u8; 32];
        broadcast[0] = 0xff;
        broadcast[1] = 0xff;
        broadcast[2] = 0xff;
        assert_eq!(
            derive_unique_local_source(&broadcast, IpAddr::V4(Ipv4Addr::LOCALHOST)),
            IpAddr::V4(Ipv4Addr::new(127, 255, 255, 255))
        );
    }

    #[test]
    fn unique_local_ipv4_different_hashes_map_deterministically() {
        let mut a = [0u8; 32];
        a[0] = 0x0a;
        a[1] = 0x0b;
        a[2] = 0x0c;
        let mut b = [0u8; 32];
        b[0] = 0x0d;
        b[1] = 0x0e;
        b[2] = 0x0f;
        let target = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let sa = derive_unique_local_source(&a, target);
        let sb = derive_unique_local_source(&b, target);
        assert_eq!(sa, IpAddr::V4(Ipv4Addr::new(127, 0x0a, 0x0b, 0x0c)));
        assert_eq!(sb, IpAddr::V4(Ipv4Addr::new(127, 0x0d, 0x0e, 0x0f)));
        assert_ne!(sa, sb);
    }

    #[test]
    fn unique_local_same_hash_repeats_without_retained_state() {
        let mut hash = [0x42u8; 32];
        hash[0] = 0x11;
        hash[1] = 0x22;
        hash[2] = 0x33;
        let target = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let first = derive_unique_local_source(&hash, target);
        let second = derive_unique_local_source(&hash, target);
        assert_eq!(first, second);
        assert_eq!(first, IpAddr::V4(Ipv4Addr::new(127, 0x11, 0x22, 0x33)));
    }

    #[test]
    fn unique_local_ipv6_derivation_matches_java_byte_layout() {
        // Java: addr[0]=0xfd; copy hash[0..15] into addr[1..16].
        let mut hash = [0u8; 32];
        for (i, byte) in hash.iter_mut().enumerate() {
            *byte = i as u8;
        }
        let source =
            derive_unique_local_source(&hash, IpAddr::V6(Ipv6Addr::LOCALHOST));
        let expected_octets = [
            0xfd, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a,
            0x0b, 0x0c, 0x0d, 0x0e,
        ];
        assert_eq!(source, IpAddr::V6(Ipv6Addr::from(expected_octets)));
        // Only the first 15 hash bytes participate; trailing hash bytes are ignored.
        let mut variant = hash;
        variant[14] = 0xff;
        let variant_source =
            derive_unique_local_source(&variant, IpAddr::V6(Ipv6Addr::LOCALHOST));
        assert_ne!(variant_source, source);
        let mut ignored_tail = hash;
        ignored_tail[15] = 0xff;
        ignored_tail[31] = 0xff;
        assert_eq!(
            derive_unique_local_source(&ignored_tail, IpAddr::V6(Ipv6Addr::LOCALHOST)),
            source
        );
    }

    #[test]
    fn unique_local_option_boolean_validation_is_exact() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        // Absent and explicit false are disabled, never a failure.
        assert!(!unique_local_enabled(&definition(&[])).unwrap());
        assert!(
            !unique_local_enabled(&definition(&[(
                "UniqueLocalAddressPerClient",
                serde_json::json!(false)
            )]))
            .unwrap()
        );
        assert!(
            unique_local_enabled(&definition(&[(
                "UniqueLocalAddressPerClient",
                serde_json::json!(true)
            )]))
            .unwrap()
        );
        // Malformed types fail before allocation.
        for bad in [
            serde_json::json!("true"),
            serde_json::json!(1),
            serde_json::json!(0),
            serde_json::json!(serde_json::json!({"v": true})),
        ] {
            let def = definition(&[("UniqueLocalAddressPerClient", bad)]);
            assert!(unique_local_enabled(&def).is_err());
            assert!(backend.validate_start(&def).is_err());
        }
        // Round-trip preserves exact boolean spelling via raw config.
        let enabled = definition(&[("UniqueLocalAddressPerClient", serde_json::json!(true))]);
        let json = serde_json::to_value(&enabled).unwrap();
        let restored: TunnelDefinition = serde_json::from_value(json).unwrap();
        assert_eq!(
            restored.raw_config.get("UniqueLocalAddressPerClient"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn unique_local_target_remains_literal_loopback() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        for hostile in [
            "10.0.0.1",
            "192.168.1.1",
            "8.8.8.8",
            "example.i2p",
            "example.com",
            "",
            "127.0.0.2",
            "::ffff:127.0.0.1",
        ] {
            let def = definition(&[
                ("TargetHost", serde_json::json!(hostile)),
                ("UniqueLocalAddressPerClient", serde_json::json!(true)),
            ]);
            assert!(
                backend.config_without_destination(&def).is_err(),
                "hostile host {hostile:?} must stay rejected with unique-local enabled"
            );
        }
    }

    #[tokio::test]
    async fn unique_local_disabled_uses_ordinary_loopback_connect() {
        let local_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = local_listener.local_addr().unwrap().port();
        let local = tokio::spawn(async move {
            let (stream, _) = local_listener.accept().await.unwrap();
            Ok::<_, std::io::Error>(stream.peer_addr().unwrap())
        });
        let (mut client, server) = duplex(64 * 1024);
        let (server_read, server_write) = tokio::io::split(server);
        let task = tokio::spawn(handle_http_stream(
            server_read,
            server_write,
            distinct_peer(0xF0),
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port,
            HttpServerPolicy {
                website_host: "localhost".to_owned(),
                ..HttpServerPolicy {
                    website_host: "localhost".to_owned(),
                    block_access_in_proxies: false,
                    block_referers: false,
                    allow_referer: true,
                    allow_accept: true,
                    block_user_agents: false,
                    allow_user_agent: true,
                    user_agents: None,
                    access_list: None,
                    access_option: AccessOption::Allow,
                }
            },
            PostLimiter::new(0, Duration::from_secs(60)),
            false,
        ));
        client
            .write_all(b"GET /plain HTTP/1.1\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
        client.shutdown().await.unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).await.unwrap();
        // Local listener is a raw TCP socket, not HTTP, so filtering yields 502,
        // but the ordinary connect must still have reached it.
        assert!(
            String::from_utf8(response)
                .unwrap()
                .starts_with("HTTP/1.1 502 Bad Gateway\r\n")
        );
        task.await.unwrap().unwrap();
        let peer = local.await.unwrap().unwrap();
        assert_eq!(peer.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    }

    #[tokio::test]
    async fn unique_local_enabled_uses_derived_source_observable() {
        let local_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = local_listener.local_addr().unwrap().port();
        let local = tokio::spawn(async move {
            let (stream, _) = local_listener.accept().await.unwrap();
            let peer = stream.peer_addr().unwrap();
            // Drain the request so the handler can complete the relay attempt.
            let (mut reader, mut writer) = tokio::io::split(stream);
            let mut buf = Vec::new();
            let _ = tokio::io::AsyncReadExt::read_to_end(&mut reader, &mut buf).await;
            let _ = writer.shutdown().await;
            peer
        });
        let peer = distinct_peer(0x51);
        let expected = derive_unique_local_source(
            peer.canonical_id(),
            IpAddr::V4(Ipv4Addr::LOCALHOST),
        );
        assert_ne!(expected, IpAddr::V4(Ipv4Addr::LOCALHOST));
        let (mut client, server) = duplex(64 * 1024);
        let (server_read, server_write) = tokio::io::split(server);
        let task = tokio::spawn(handle_http_stream(
            server_read,
            server_write,
            peer,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port,
            HttpServerPolicy {
                website_host: "localhost".to_owned(),
                ..HttpServerPolicy {
                    website_host: "localhost".to_owned(),
                    block_access_in_proxies: false,
                    block_referers: false,
                    allow_referer: true,
                    allow_accept: true,
                    block_user_agents: false,
                    allow_user_agent: true,
                    user_agents: None,
                    access_list: None,
                    access_option: AccessOption::Allow,
                }
            },
            PostLimiter::new(0, Duration::from_secs(60)),
            true,
        ));
        client
            .write_all(b"GET /derived HTTP/1.1\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
        client.shutdown().await.unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).await.unwrap();
        task.await.unwrap().unwrap();
        let observed = local.await.unwrap();
        assert_eq!(observed.ip(), expected);
    }

    #[tokio::test]
    async fn unique_local_headers_cannot_influence_source() {
        let local_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_port = local_listener.local_addr().unwrap().port();
        let local = tokio::spawn(async move {
            let (stream, _) = local_listener.accept().await.unwrap();
            let peer = stream.peer_addr().unwrap();
            let (mut reader, mut writer) = tokio::io::split(stream);
            let mut buf = Vec::new();
            let _ = tokio::io::AsyncReadExt::read_to_end(&mut reader, &mut buf).await;
            let _ = writer.shutdown().await;
            peer
        });
        let peer = distinct_peer(0x52);
        let expected = derive_unique_local_source(
            peer.canonical_id(),
            IpAddr::V4(Ipv4Addr::LOCALHOST),
        );
        let (mut client, server) = duplex(64 * 1024);
        let (server_read, server_write) = tokio::io::split(server);
        let task = tokio::spawn(handle_http_stream(
            server_read,
            server_write,
            peer,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port,
            HttpServerPolicy {
                website_host: "localhost".to_owned(),
                ..HttpServerPolicy {
                    website_host: "localhost".to_owned(),
                    block_access_in_proxies: false,
                    block_referers: false,
                    allow_referer: true,
                    allow_accept: true,
                    block_user_agents: false,
                    allow_user_agent: true,
                    user_agents: None,
                    access_list: None,
                    access_option: AccessOption::Allow,
                }
            },
            PostLimiter::new(0, Duration::from_secs(60)),
            true,
        ));
        client
            .write_all(
                b"GET /derived HTTP/1.1\r\nHost: evil.i2p\r\nX-Forwarded-For: 127.9.9.9\r\nx-i2p-destb64: attacker\r\nForwarded: for=127.8.8.8\r\nContent-Length: 0\r\n\r\n",
            )
            .await
            .unwrap();
        client.shutdown().await.unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).await.unwrap();
        task.await.unwrap().unwrap();
        assert_eq!(local.await.unwrap().ip(), expected);
    }

    #[tokio::test]
    async fn unique_local_bind_failure_is_bounded_without_fallback() {
        // IPv6 ULA source requires an OS-assigned address (same as Java). On a
        // default loopback without that assignment the exact bind must fail
        // closed with 502 and must not fall back to an ordinary source.
        let peer = distinct_peer(0x53);
        let derived =
            derive_unique_local_source(peer.canonical_id(), IpAddr::V6(Ipv6Addr::LOCALHOST));
        assert!(matches!(derived, IpAddr::V6(_)));
        // Probe whether this platform can bind the derived ULA at all.
        let can_bind = tokio::net::TcpSocket::new_v6()
            .and_then(|socket| {
                socket.bind(std::net::SocketAddr::new(derived, 0))?;
                Ok(())
            })
            .is_ok();
        if can_bind {
            return;
        }
        let local_listener = tokio::net::TcpListener::bind("[::1]:0").await.unwrap();
        let local_port = local_listener.local_addr().unwrap().port();
        let local = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(150), local_listener.accept()).await
        });
        let (mut client, server) = duplex(64 * 1024);
        let (server_read, server_write) = tokio::io::split(server);
        let task = tokio::spawn(handle_http_stream(
            server_read,
            server_write,
            peer,
            IpAddr::V6(Ipv6Addr::LOCALHOST),
            local_port,
            HttpServerPolicy {
                website_host: "localhost".to_owned(),
                ..HttpServerPolicy {
                    website_host: "localhost".to_owned(),
                    block_access_in_proxies: false,
                    block_referers: false,
                    allow_referer: true,
                    allow_accept: true,
                    block_user_agents: false,
                    allow_user_agent: true,
                    user_agents: None,
                    access_list: None,
                    access_option: AccessOption::Allow,
                }
            },
            PostLimiter::new(0, Duration::from_secs(60)),
            true,
        ));
        client
            .write_all(b"GET /v6 HTTP/1.1\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
        client.shutdown().await.unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).await.unwrap();
        assert!(
            String::from_utf8(response)
                .unwrap()
                .starts_with("HTTP/1.1 502 Bad Gateway\r\n")
        );
        task.await.unwrap().unwrap();
        assert!(
            local.await.unwrap().is_err(),
            "failed exact bind must not fall back to another source"
        );
    }

    #[tokio::test]
    async fn unique_local_malformed_option_fails_before_running_generation() {
        let root = tempfile::tempdir().unwrap();
        let backend = HttpServerTunnelBackend::new(7656, ServerDestinationStore::new(root.path()));
        let bad = definition(&[("UniqueLocalAddressPerClient", serde_json::json!("yes"))]);
        assert!(backend.validate_start(&bad).is_err());
        assert!(backend.start(&bad).await.is_err());
        assert_eq!(
            backend.inspect(&bad).runtime_state,
            TunnelRuntimeState::Stopped
        );
    }
}
