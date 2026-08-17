//! Composed Proposal 170 `httpbidirserver` runtime.
//!
//! This module owns lifecycle composition only. HTTP parsing, request/response
//! filtering, and outbound request sanitization remain in the accepted M067
//! and M068 modules respectively.

use std::{collections::HashMap, net::IpAddr, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::task::JoinHandle;

use super::{
    filters::{
        http::{AccessOption, HttpServerPolicy},
        http_client::HttpClientPolicy,
    },
    http_client::make_no_outproxy_handler,
    http_server::{make_accepted_handler, PostLimiter},
    options::{validate_options, OptionValidationError, HTTP_BIDIR_SERVER_OPTIONS},
    runtime::{
        run_accepted_server, run_client_listener, AcceptedServerRuntimeConfig,
        AcceptedServerRuntimeError, ClientListenerRuntimeConfig, ClientListenerRuntimeError,
        ServerAdmissionPolicy,
    },
    server::SERVER_IDENTITY_KEY,
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    address_book_runtime::RuntimeAddressBookHandle,
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
    server_secret_store::{ServerDestinationStore, StoredDestination},
};

const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RUNTIME_TASKS: usize = 1000;
const MAX_CONNECTIONS: usize = 128;
const MAX_POST_LIMIT: usize = 1_000_000;
const MAX_POST_WINDOW: u64 = 86_400;

#[derive(Clone)]
struct HttpBidirConfig {
    name: String,
    target_host: String,
    target_port: u16,
    bind_address: IpAddr,
    listen_port: u16,
    sam_tcp_port: u16,
    destination: StoredDestination,
    admission: ServerAdmissionPolicy,
    server_policy: HttpServerPolicy,
    post_limiter: PostLimiter,
    proxy_username: Option<String>,
    proxy_password: Option<String>,
    require_proxy_auth: bool,
    client_policy: HttpClientPolicy,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompositeRuntimeError {
    Startup,
    ServerExited,
    ClientExited,
    Panicked,
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

#[derive(Clone, Debug, Default)]
struct RuntimeSupervisor {
    inner: Arc<Mutex<RuntimeMap>>,
}

impl RuntimeSupervisor {
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
                    tunnel_type: TunnelType::HttpBidirServer,
                    current_state: entry.state,
                    attempted_action: "start",
                });
            }
        }
        if runtime.entries.values().filter(|entry| entry.task.is_some()).count()
            >= MAX_RUNTIME_TASKS
        {
            return Err(BackendError::Internal {
                message: "httpbidirserver runtime capacity exhausted".to_owned(),
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
        result: Result<(), CompositeRuntimeError>,
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
            entry.failure = Some("httpbidirserver tunnel runtime failed");
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

    async fn stop_children(
        cancellation: &tokio::sync::watch::Sender<bool>,
        server_task: &mut JoinHandle<Result<(), AcceptedServerRuntimeError>>,
        client_task: &mut JoinHandle<Result<(), ClientListenerRuntimeError>>,
    ) {
        let _ = cancellation.send(true);
        if tokio::time::timeout(STOP_TIMEOUT, async {
            let _ = (&mut *server_task).await;
            let _ = (&mut *client_task).await;
        })
        .await
        .is_err()
        {
            server_task.abort();
            client_task.abort();
            let _ = server_task.await;
            let _ = client_task.await;
        }
    }

    async fn stop_client_task(
        client_task: &mut JoinHandle<Result<(), ClientListenerRuntimeError>>,
    ) {
        if tokio::time::timeout(STOP_TIMEOUT, &mut *client_task).await.is_err() {
            client_task.abort();
            let _ = client_task.await;
        }
    }

    async fn stop_server_task(
        server_task: &mut JoinHandle<Result<(), AcceptedServerRuntimeError>>,
    ) {
        if tokio::time::timeout(STOP_TIMEOUT, &mut *server_task).await.is_err() {
            server_task.abort();
            let _ = server_task.await;
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
                message: "httpbidirserver tunnel stop timed out".to_owned(),
            });
        }
        self.remove_generation(name, generation);
        Ok(())
    }

    async fn start(&self, config: HttpBidirConfig) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.reserve(&name)?;
        let map = Arc::clone(&self.inner);
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(run_composite(
                config,
                ready_cancellation.clone(),
                ready_tx,
            ))
            .catch_unwind()
            .await
            .unwrap_or(Err(CompositeRuntimeError::Panicked));
            let cancelled = *ready_cancellation.borrow();
            Self::complete(map, task_name, generation, result, cancelled);
        });
        self.set_task(&name, generation, task);
        match tokio::time::timeout(START_TIMEOUT, ready_rx).await {
            Ok(Ok(Ok(destination))) => {
                self.publish_destination(&name, generation, &destination);
                if self.mark_running(&name, generation) {
                    Ok(())
                } else {
                    let _ = self.stop_generation(&name, generation).await;
                    Err(BackendError::Internal {
                        message: "httpbidirserver runtime exited during start".to_owned(),
                    })
                }
            }
            Ok(Ok(Err(_))) | Ok(Err(_)) | Err(_) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: "httpbidirserver tunnel runtime failed to start".to_owned(),
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
                entry.failure.unwrap_or("httpbidirserver tunnel runtime is active"),
                entry.destination.clone(),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                "httpbidirserver tunnel runtime is stopped",
                None,
            ),
        }
    }
}

async fn run_composite(
    config: HttpBidirConfig,
    mut cancellation: tokio::sync::watch::Receiver<bool>,
    ready: tokio::sync::oneshot::Sender<Result<String, CompositeRuntimeError>>,
) -> Result<(), CompositeRuntimeError> {
    let (child_cancellation, child_receiver) = tokio::sync::watch::channel(false);
    let server_handler = make_accepted_handler(
        config.target_host.clone(),
        config.target_port,
        config.server_policy.clone(),
        config.post_limiter.clone(),
    );
    let client_handler = make_no_outproxy_handler(
        config.proxy_username.clone(),
        config.proxy_password.clone(),
        config.require_proxy_auth,
        config.client_policy.clone(),
        config.address_book.clone(),
    );
    let (server_ready_tx, server_ready_rx) = tokio::sync::oneshot::channel();
    let (client_ready_tx, client_ready_rx) = tokio::sync::oneshot::channel();
    let mut server_task = tokio::spawn(run_accepted_server(
        AcceptedServerRuntimeConfig {
            name: format!("{}-server", config.name),
            sam_tcp_port: config.sam_tcp_port,
            destination: config.destination,
            admission: config.admission,
            handler: server_handler,
        },
        child_receiver.clone(),
        server_ready_tx,
    ));
    let mut client_task = tokio::spawn(run_client_listener(
        ClientListenerRuntimeConfig {
            name: format!("{}-client", config.name),
            bind_address: config.bind_address,
            port: config.listen_port,
            destination: "direct-i2p-only".to_owned(),
            destination_port: 80,
            sam_tcp_port: config.sam_tcp_port,
            max_connections: MAX_CONNECTIONS,
            handler: client_handler,
        },
        child_receiver,
        client_ready_tx,
    ));

    let mut server_wait = Box::pin(server_ready_rx);
    let mut client_wait = Box::pin(client_ready_rx);
    let mut server_destination = None;
    let mut client_ready = false;

    while server_destination.is_none() || !client_ready {
        if *cancellation.borrow() {
            let _ = ready.send(Err(CompositeRuntimeError::Startup));
            RuntimeSupervisor::stop_children(
                &child_cancellation,
                &mut server_task,
                &mut client_task,
            )
            .await;
            return Ok(());
        }
        tokio::select! {
            _ = cancellation.changed() => {
                let _ = ready.send(Err(CompositeRuntimeError::Startup));
                RuntimeSupervisor::stop_children(
                    &child_cancellation,
                    &mut server_task,
                    &mut client_task,
                ).await;
                return Ok(());
            }
            result = &mut server_wait, if server_destination.is_none() => {
                match result {
                    Ok(Ok(destination)) => server_destination = Some(destination),
                    _ => {
                        let _ = ready.send(Err(CompositeRuntimeError::Startup));
                        RuntimeSupervisor::stop_children(
                            &child_cancellation,
                            &mut server_task,
                            &mut client_task,
                        ).await;
                        return Err(CompositeRuntimeError::Startup);
                    }
                }
            }
            result = &mut client_wait, if !client_ready => {
                match result {
                    Ok(Ok(_)) => client_ready = true,
                    _ => {
                        let _ = ready.send(Err(CompositeRuntimeError::Startup));
                        RuntimeSupervisor::stop_children(
                            &child_cancellation,
                            &mut server_task,
                            &mut client_task,
                        ).await;
                        return Err(CompositeRuntimeError::Startup);
                    }
                }
            }
        }
    }

    let _ = ready.send(Ok(
        server_destination.expect("server readiness was observed")
    ));
    tokio::select! {
        _ = cancellation.changed() => {
            RuntimeSupervisor::stop_children(
                &child_cancellation,
                &mut server_task,
                &mut client_task,
            ).await;
            Ok(())
        }
        result = &mut server_task => {
            let _ = child_cancellation.send(true);
            RuntimeSupervisor::stop_client_task(&mut client_task).await;
            match result {
                Ok(Ok(())) | Ok(Err(_)) | Err(_) => Err(CompositeRuntimeError::ServerExited),
            }
        }
        result = &mut client_task => {
            let _ = child_cancellation.send(true);
            RuntimeSupervisor::stop_server_task(&mut server_task).await;
            match result {
                Ok(Ok(())) | Ok(Err(_)) | Err(_) => Err(CompositeRuntimeError::ClientExited),
            }
        }
    }
}

/// Real control-plane-owned Proposal 170 `httpbidirserver` backend.
#[derive(Clone)]
pub struct HttpBidirServerTunnelBackend {
    supervisor: RuntimeSupervisor,
    destinations: ServerDestinationStore,
    sam_tcp_port: u16,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
}

impl std::fmt::Debug for HttpBidirServerTunnelBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpBidirServerTunnelBackend")
            .field("sam_tcp_port", &self.sam_tcp_port)
            .finish_non_exhaustive()
    }
}

impl HttpBidirServerTunnelBackend {
    pub fn new(
        sam_tcp_port: u16,
        destinations: ServerDestinationStore,
        address_book: Option<Arc<RuntimeAddressBookHandle>>,
    ) -> Self {
        Self {
            supervisor: RuntimeSupervisor::default(),
            destinations,
            sam_tcp_port,
            address_book,
        }
    }

    fn config_without_destination(
        &self,
        definition: &TunnelDefinition,
    ) -> BackendResult<HttpBidirConfig> {
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::HttpBidirServer,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        validate_options(
            TunnelType::HttpBidirServer,
            &definition.options,
            HTTP_BIDIR_SERVER_OPTIONS,
        )
        .map_err(option_error)?;
        validate_raw_options(definition)?;

        let target_host = raw_string(definition, "TargetHost")?
            .or(raw_string(definition, "Host")?)
            .unwrap_or_else(|| "127.0.0.1".to_owned());
        if !matches!(target_host.as_str(), "127.0.0.1" | "localhost" | "::1") {
            return Err(invalid_option("TargetHost must be loopback"));
        }
        let target_port =
            definition.options.target_port.ok_or_else(|| BackendError::MissingOption {
                tunnel_type: TunnelType::HttpBidirServer,
                option: "TargetPort".to_owned(),
            })?;
        let listen_port =
            definition.options.listen_port.ok_or_else(|| BackendError::MissingOption {
                tunnel_type: TunnelType::HttpBidirServer,
                option: "ListenPort".to_owned(),
            })?;
        let bind_address = match definition.options.listen_interface.as_deref() {
            None => "127.0.0.1".parse().expect("loopback address is valid"),
            Some(value) =>
                value.parse::<IpAddr>().map_err(|_| invalid_option("ListenInterface"))?,
        };
        let proxy_username = raw_string(definition, "ProxyUsername")?
            .or_else(|| definition.options.proxy_username.clone());
        let proxy_password = definition.options.proxy_password.as_deref().map(str::to_owned);
        if proxy_username.is_some() != proxy_password.is_some() {
            return Err(invalid_option("ProxyUsername/ProxyPassword"));
        }
        let require_proxy_auth = raw_bool(definition, "ProxyAuth")?
            .unwrap_or(proxy_username.is_some())
            || !bind_address.is_loopback();
        if require_proxy_auth && proxy_username.is_none() {
            return Err(invalid_option("ProxyAuth requires credentials"));
        }

        let website = raw_string(definition, "WebsiteHostname")?
            .or_else(|| definition.options.http_host.clone());
        let spoofed = raw_string(definition, "SpoofedHost")?;
        if website.is_some() && spoofed.is_some() && website != spoofed {
            return Err(invalid_option("WebsiteHostname/SpoofedHost"));
        }
        let website_host = spoofed.or(website).unwrap_or_else(|| "localhost".to_owned());
        if website_host.is_empty()
            || website_host.len() > 255
            || website_host.chars().any(|character| {
                character.is_control() || character.is_whitespace() || character == '/'
            })
        {
            return Err(invalid_option("WebsiteHostname"));
        }
        let access_list = raw_string(definition, "AccessList")?
            .or_else(|| definition.options.access_list.clone())
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
            Some(_) => return Err(invalid_option("AccessOption")),
        };
        let admission = ServerAdmissionPolicy::from_raw_options(&definition.raw_config)
            .map_err(invalid_option)?;
        let post_limit = raw_u64(definition, "PostLimit")?.unwrap_or(0);
        let post_window = raw_u64(definition, "PostLimitTime")?.unwrap_or(60);
        if post_limit > MAX_POST_LIMIT as u64
            || post_limit > 0 && !(1..=MAX_POST_WINDOW).contains(&post_window)
        {
            return Err(invalid_option("PostLimit/PostLimitTime"));
        }

        Ok(HttpBidirConfig {
            name: definition.name.as_str().to_owned(),
            target_host,
            target_port,
            bind_address,
            listen_port,
            sam_tcp_port: self.sam_tcp_port,
            destination: StoredDestination::from_private(String::new()),
            admission,
            server_policy: HttpServerPolicy {
                website_host,
                block_access_in_proxies: raw_bool(definition, "BlockAccessInProxies")?
                    .unwrap_or(false),
                block_referers: raw_bool(definition, "BlockReferers")?.unwrap_or(false),
                allow_referer: raw_bool(definition, "AllowReferer")?.unwrap_or(true),
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
            proxy_username,
            proxy_password,
            require_proxy_auth,
            client_policy: HttpClientPolicy {
                allow_user_agent: raw_bool(definition, "AllowUserAgent")?.unwrap_or(false),
                allow_referer: raw_bool(definition, "AllowReferer")?.unwrap_or(false)
                    && !raw_bool(definition, "BlockReferers")?.unwrap_or(false),
                allow_accept: raw_bool(definition, "AllowAccept")?.unwrap_or(false),
                outproxy_authorization: None,
            },
            address_book: self.address_book.clone(),
        })
    }
}

#[async_trait::async_trait]
impl TunnelBackend for HttpBidirServerTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::HttpBidirServer
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        let mut config = self.config_without_destination(definition)?;
        let identity = definition
            .raw_config
            .get(SERVER_IDENTITY_KEY)
            .and_then(|value| value.as_str())
            .ok_or_else(|| BackendError::Internal {
                message: "httpbidirserver destination identity is not allocated".to_owned(),
            })?;
        config.destination = self
            .destinations
            .get(identity)
            .await
            .map_err(|_| BackendError::Internal {
                message: "httpbidirserver destination store lookup failed".to_owned(),
            })?
            .ok_or_else(|| BackendError::Internal {
                message: "httpbidirserver destination identity is unavailable".to_owned(),
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
            tunnel_type: TunnelType::HttpBidirServer,
            runtime_state,
            message: message.to_owned(),
            destination,
        }
    }
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
        "ProxyAuth",
        "ProxyUsername",
        "AccessOption",
        "AccessList",
        "MaxConcurrentConns",
        "ClientPerMinute",
        "ClientPerHour",
        "ClientPerDay",
        "TotalInPerMinute",
        "TotalInPerHour",
        "TotalInPerDay",
        "PostLimit",
        "PostLimitTime",
        "HostingDestination",
        "Description",
        "StartOnLoad",
    ];
    const METADATA: &[&str] = &["name", "type", "Name", "Type", "Action"];
    for key in definition.raw_config.keys() {
        if key.starts_with("__emissary_")
            || SUPPORTED.contains(&key.as_str())
            || METADATA.contains(&key.as_str())
        {
            continue;
        }
        return Err(BackendError::UnsupportedOption {
            tunnel_type: TunnelType::HttpBidirServer,
            option: key.clone(),
        });
    }
    Ok(())
}

fn invalid_option(option: &str) -> BackendError {
    BackendError::Internal {
        message: format!("httpbidirserver option {option} is invalid"),
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
    use emissary_core::crypto::base64_encode;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    fn definition() -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("http-bidir").unwrap(),
            tunnel_type: TunnelType::HttpBidirServer,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions {
                target_port: Some(8080),
                listen_port: Some(0),
                ..Default::default()
            },
            raw_config: Default::default(),
        }
    }

    #[test]
    fn clearnet_and_outproxy_options_fail_before_allocation() {
        let mut definition = definition();
        definition.raw_config.insert(
            "ProxyList".to_owned(),
            serde_json::json!("outproxy.i2p:4444"),
        );
        let backend = HttpBidirServerTunnelBackend::new(1, ServerDestinationStore::new("."), None);
        assert!(matches!(
            backend.config_without_destination(&definition),
            Err(BackendError::UnsupportedOption {
                tunnel_type: TunnelType::HttpBidirServer,
                option
            }) if option == "ProxyList"
        ));
    }

    #[test]
    fn direct_only_client_handler_rejects_clearnet_without_connecting() {
        let request = b"GET http://example.com/ HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert!(
            super::super::filters::http_client::HttpClientRequest::parse(request, None).is_err()
        );
    }

    #[test]
    fn inbound_server_uses_shared_admission_policy() {
        let mut definition = definition();
        definition
            .raw_config
            .insert("MaxConcurrentConns".to_owned(), serde_json::json!(9));
        let backend = HttpBidirServerTunnelBackend::new(1, ServerDestinationStore::new("."), None);
        let config = backend.config_without_destination(&definition).unwrap();
        assert_eq!(config.admission.max_concurrent_connections(), 9);
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
                            "SESSION STATUS RESULT=OK DESTINATION=server-destination\n"
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
    async fn composite_start_stop_restart_is_single_generation_and_keeps_identity() {
        let (sam_port, sam_task) = fake_sam().await;
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        let identity = ServerDestinationStore::new_identity();
        store
            .put(
                &identity,
                StoredDestination::from_private(base64_encode([7u8; 128])),
            )
            .await
            .unwrap();
        let backend = HttpBidirServerTunnelBackend::new(sam_port, store, None);
        let mut definition = definition();
        definition
            .raw_config
            .insert(SERVER_IDENTITY_KEY.to_owned(), serde_json::json!(identity));

        backend.start(&definition).await.unwrap();
        let first = backend.inspect(&definition);
        assert_eq!(first.runtime_state, TunnelRuntimeState::Running);
        assert_eq!(first.destination.as_deref(), Some("server-destination"));
        assert!(matches!(
            backend.start(&definition).await,
            Err(BackendError::InvalidState { .. })
        ));

        backend.stop(&definition).await.unwrap();
        assert_eq!(
            backend.inspect(&definition).runtime_state,
            TunnelRuntimeState::Stopped
        );
        backend.start(&definition).await.unwrap();
        let second = backend.inspect(&definition);
        assert_eq!(second.destination, first.destination);
        backend.stop(&definition).await.unwrap();
        sam_task.abort();
    }

    #[tokio::test]
    async fn local_proxy_bind_failure_does_not_leave_server_half_running() {
        let (sam_port, sam_task) = fake_sam().await;
        let occupied = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = occupied.local_addr().unwrap().port();
        let root = tempfile::tempdir().unwrap();
        let store = ServerDestinationStore::new(root.path());
        let identity = ServerDestinationStore::new_identity();
        store
            .put(
                &identity,
                StoredDestination::from_private(base64_encode([8u8; 128])),
            )
            .await
            .unwrap();
        let backend = HttpBidirServerTunnelBackend::new(sam_port, store, None);
        let mut definition = definition();
        definition.options.listen_port = Some(port);
        definition
            .raw_config
            .insert(SERVER_IDENTITY_KEY.to_owned(), serde_json::json!(identity));

        assert!(backend.start(&definition).await.is_err());
        assert_eq!(
            backend.inspect(&definition).runtime_state,
            TunnelRuntimeState::Stopped
        );
        drop(occupied);
        sam_task.abort();
    }

    #[test]
    fn persisted_public_destination_does_not_select_local_target() {
        let backend = HttpBidirServerTunnelBackend::new(
            7656,
            ServerDestinationStore::new(tempfile::tempdir().unwrap().path()),
            None,
        );
        let mut definition = definition();
        definition.options.hosting_destination = Some("published-server-destination".to_owned());
        let config = backend.config_without_destination(&definition).unwrap();
        assert_eq!(config.target_host, "127.0.0.1");
    }
}
