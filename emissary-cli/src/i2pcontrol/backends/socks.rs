//! Bounded Proposal 170 SOCKS4a/SOCKS5 client frontend.
//!
//! The parser owns no socket discovery or DNS capability.  A request is
//! classified into a direct I2P destination or an explicitly configured I2P
//! SOCKS5 outproxy before the M065 Yosemite connector is used.

use std::{collections::HashMap, fmt, io, net::IpAddr, sync::Arc, time::Duration};

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    task::JoinHandle,
};

use super::{
    filters::{
        http_client::{classify_target, parse_authority, HttpTarget, OutproxyTarget},
        irc::relay_client_stream,
    },
    options::{validate_options, OptionCapabilities, OptionValidationError, SOCKS_OPTIONS},
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    address_book_runtime::RuntimeAddressBookHandle,
    backends::runtime::{
        client_lifecycle_config, ClientConnectionHandler, ClientLifecycleConfig,
        ClientListenerRuntimeConfig, ClientListenerRuntimeError, ClientStreamConnector,
    },
    client_secret_store::ClientDestinationStore,
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelRuntimeState, TunnelType},
};
use yosemite_i2pcontrol::SessionOptions;

pub(crate) const NEGOTIATION_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_NEGOTIATION_BYTES: usize = 8 * 1024;
const MAX_METHODS: usize = 32;
const MAX_FIELD_BYTES: usize = 255;
const START_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RUNTIME_TASKS: usize = 1000;
const MAX_CONNECTIONS: usize = 128;
const SOCKS4_GRANTED: u8 = 0x5a;
const SOCKS4_REJECTED: u8 = 0x5b;
const SOCKS4_AUTH_FAILED: u8 = 0x5d;
const SOCKS5_SUCCEEDED: u8 = 0x00;
const SOCKS5_GENERAL_FAILURE: u8 = 0x01;
const SOCKS5_HOST_UNREACHABLE: u8 = 0x04;
const SOCKS5_CONNECTION_REFUSED: u8 = 0x05;
const SOCKS5_COMMAND_UNSUPPORTED: u8 = 0x07;
const SOCKS5_ADDRESS_UNSUPPORTED: u8 = 0x08;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PayloadMode {
    Raw,
    Irc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SocksVersion {
    V4,
    V5,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SocksRequest {
    version: SocksVersion,
    host: String,
    port: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NegotiationError {
    version: Option<SocksVersion>,
    reply: u8,
    close_only: bool,
}

impl NegotiationError {
    fn response(version: SocksVersion, reply: u8) -> Self {
        Self {
            version: Some(version),
            reply,
            close_only: false,
        }
    }

    fn close(version: SocksVersion) -> Self {
        Self {
            version: Some(version),
            reply: SOCKS5_GENERAL_FAILURE,
            close_only: true,
        }
    }

    fn unknown() -> Self {
        Self {
            version: None,
            reply: SOCKS5_GENERAL_FAILURE,
            close_only: true,
        }
    }
}

struct BudgetedReader<'a, R> {
    reader: &'a mut R,
    remaining: usize,
}

impl<'a, R: AsyncRead + Unpin> BudgetedReader<'a, R> {
    fn new(reader: &'a mut R) -> Self {
        Self {
            reader,
            remaining: MAX_NEGOTIATION_BYTES,
        }
    }

    async fn read_exact(&mut self, bytes: &mut [u8]) -> io::Result<()> {
        if bytes.len() > self.remaining {
            return Err(invalid("SOCKS negotiation exceeds byte limit"));
        }
        self.remaining -= bytes.len();
        self.reader.read_exact(bytes).await.map(|_| ())
    }

    async fn read_byte(&mut self) -> io::Result<u8> {
        let mut byte = [0u8; 1];
        self.read_exact(&mut byte).await?;
        Ok(byte[0])
    }

    async fn read_cstring(&mut self, maximum: usize) -> io::Result<Vec<u8>> {
        let mut value = Vec::with_capacity(maximum.min(64));
        for _ in 0..=maximum {
            let byte = self.read_byte().await?;
            if byte == 0 {
                return Ok(value);
            }
            value.push(byte);
        }
        Err(invalid("SOCKS NUL-terminated field exceeds limit"))
    }
}

/// Parse and complete the protocol negotiation, but do not connect the target.
async fn negotiate<R: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut R,
    require_auth: bool,
    credentials: Option<&(String, String)>,
) -> Result<SocksRequest, NegotiationError> {
    match tokio::time::timeout(
        NEGOTIATION_TIMEOUT,
        negotiate_inner(stream, require_auth, credentials),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(NegotiationError::response(
            SocksVersion::V5,
            SOCKS5_GENERAL_FAILURE,
        )),
    }
}

async fn negotiate_inner<R: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut R,
    require_auth: bool,
    credentials: Option<&(String, String)>,
) -> Result<SocksRequest, NegotiationError> {
    let mut reader = BudgetedReader::new(stream);
    let version = reader.read_byte().await.map_err(|_| NegotiationError::unknown())?;
    match version {
        4 => negotiate_v4(&mut reader, require_auth).await,
        5 => negotiate_v5(&mut reader, require_auth, credentials).await,
        _ => Err(NegotiationError::unknown()),
    }
}

async fn negotiate_v4<R: AsyncRead + AsyncWrite + Unpin>(
    reader: &mut BudgetedReader<'_, R>,
    require_auth: bool,
) -> Result<SocksRequest, NegotiationError> {
    let mut header = [0u8; 7];
    reader
        .read_exact(&mut header)
        .await
        .map_err(|_| NegotiationError::response(SocksVersion::V4, SOCKS4_REJECTED))?;
    let command = header[0];
    let port = u16::from_be_bytes([header[1], header[2]]);
    let address = [header[3], header[4], header[5], header[6]];
    let userid = reader
        .read_cstring(MAX_FIELD_BYTES)
        .await
        .map_err(|_| NegotiationError::response(SocksVersion::V4, SOCKS4_REJECTED))?;
    let is_socks4a = address[..3] == [0, 0, 0] && address[3] != 0;
    let host = if is_socks4a {
        let domain = reader
            .read_cstring(MAX_FIELD_BYTES)
            .await
            .map_err(|_| NegotiationError::response(SocksVersion::V4, SOCKS4_REJECTED))?;
        validate_domain(&domain)
            .map_err(|_| NegotiationError::response(SocksVersion::V4, SOCKS4_REJECTED))?
    } else {
        let _ = userid;
        return Err(NegotiationError::response(
            SocksVersion::V4,
            SOCKS4_REJECTED,
        ));
    };
    let _ = userid;
    if command != 1 || port == 0 {
        return Err(NegotiationError::response(
            SocksVersion::V4,
            SOCKS4_REJECTED,
        ));
    }
    if require_auth {
        return Err(NegotiationError::response(
            SocksVersion::V4,
            SOCKS4_AUTH_FAILED,
        ));
    }
    Ok(SocksRequest {
        version: SocksVersion::V4,
        host,
        port,
    })
}

async fn negotiate_v5<R: AsyncRead + AsyncWrite + Unpin>(
    reader: &mut BudgetedReader<'_, R>,
    require_auth: bool,
    credentials: Option<&(String, String)>,
) -> Result<SocksRequest, NegotiationError> {
    let method_count = reader
        .read_byte()
        .await
        .map_err(|_| NegotiationError::response(SocksVersion::V5, SOCKS5_GENERAL_FAILURE))?
        as usize;
    if method_count == 0 || method_count > MAX_METHODS {
        let _ = write_all(reader, &[5, 0xff]).await;
        return Err(NegotiationError::close(SocksVersion::V5));
    }
    let mut methods = vec![0u8; method_count];
    reader
        .read_exact(&mut methods)
        .await
        .map_err(|_| NegotiationError::response(SocksVersion::V5, SOCKS5_GENERAL_FAILURE))?;
    let method = if require_auth {
        methods.contains(&2).then_some(2)
    } else {
        methods.contains(&0).then_some(0)
    };
    let Some(method) = method else {
        let _ = write_all(reader, &[5, 0xff]).await;
        return Err(NegotiationError::close(SocksVersion::V5));
    };
    write_all(reader, &[5, method])
        .await
        .map_err(|_| NegotiationError::close(SocksVersion::V5))?;

    if method == 2 {
        let mut auth_header = [0u8; 2];
        reader
            .read_exact(&mut auth_header)
            .await
            .map_err(|_| NegotiationError::close(SocksVersion::V5))?;
        if auth_header[0] != 1 {
            let _ = write_all(reader, &[1, 1]).await;
            return Err(NegotiationError::close(SocksVersion::V5));
        }
        let username = read_length_prefixed(reader, auth_header[1] as usize).await;
        let password_length = match username {
            Ok(_) => reader.read_byte().await,
            Err(_) => Err(invalid("invalid SOCKS username")),
        };
        let password = match password_length {
            Ok(length) => read_length_prefixed(reader, length as usize).await,
            Err(error) => Err(error),
        };
        let valid = match (username, password, credentials) {
            (Ok(username), Ok(password), Some((expected_user, expected_password))) => {
                match (
                    std::str::from_utf8(&username),
                    std::str::from_utf8(&password),
                ) {
                    (Ok(username), Ok(password)) => {
                        crate::i2pcontrol::auth::compare_passwords(username, expected_user)
                            && crate::i2pcontrol::auth::compare_passwords(
                                password,
                                expected_password,
                            )
                    }
                    _ => false,
                }
            }
            _ => false,
        };
        if !valid {
            let _ = write_all(reader, &[1, 1]).await;
            return Err(NegotiationError::close(SocksVersion::V5));
        }
        write_all(reader, &[1, 0])
            .await
            .map_err(|_| NegotiationError::close(SocksVersion::V5))?;
    }

    let mut request_header = [0u8; 4];
    reader
        .read_exact(&mut request_header)
        .await
        .map_err(|_| NegotiationError::response(SocksVersion::V5, SOCKS5_GENERAL_FAILURE))?;
    if request_header[0] != 5 {
        return Err(NegotiationError::response(
            SocksVersion::V5,
            SOCKS5_GENERAL_FAILURE,
        ));
    }
    if request_header[2] != 0 {
        return Err(NegotiationError::response(
            SocksVersion::V5,
            SOCKS5_GENERAL_FAILURE,
        ));
    }
    let host = match request_header[3] {
        1 => {
            let mut address = [0u8; 4];
            reader.read_exact(&mut address).await.map_err(|_| {
                NegotiationError::response(SocksVersion::V5, SOCKS5_ADDRESS_UNSUPPORTED)
            })?;
            String::new()
        }
        3 => {
            let length = reader.read_byte().await.map_err(|_| {
                NegotiationError::response(SocksVersion::V5, SOCKS5_ADDRESS_UNSUPPORTED)
            })? as usize;
            let value = read_length_prefixed(reader, length).await.map_err(|_| {
                NegotiationError::response(SocksVersion::V5, SOCKS5_ADDRESS_UNSUPPORTED)
            })?;
            validate_domain(&value).map_err(|_| {
                NegotiationError::response(SocksVersion::V5, SOCKS5_ADDRESS_UNSUPPORTED)
            })?
        }
        4 => {
            let mut address = [0u8; 16];
            reader.read_exact(&mut address).await.map_err(|_| {
                NegotiationError::response(SocksVersion::V5, SOCKS5_ADDRESS_UNSUPPORTED)
            })?;
            String::new()
        }
        _ => {
            return Err(NegotiationError::response(
                SocksVersion::V5,
                SOCKS5_ADDRESS_UNSUPPORTED,
            ));
        }
    };
    let mut port_bytes = [0u8; 2];
    reader
        .read_exact(&mut port_bytes)
        .await
        .map_err(|_| NegotiationError::response(SocksVersion::V5, SOCKS5_GENERAL_FAILURE))?;
    let port = u16::from_be_bytes(port_bytes);
    if request_header[1] != 1 {
        return Err(NegotiationError::response(
            SocksVersion::V5,
            SOCKS5_COMMAND_UNSUPPORTED,
        ));
    }
    if host.is_empty() || port == 0 {
        return Err(NegotiationError::response(
            SocksVersion::V5,
            SOCKS5_ADDRESS_UNSUPPORTED,
        ));
    }
    Ok(SocksRequest {
        version: SocksVersion::V5,
        host,
        port,
    })
}

async fn read_length_prefixed<R: AsyncRead + AsyncWrite + Unpin>(
    reader: &mut BudgetedReader<'_, R>,
    length: usize,
) -> io::Result<Vec<u8>> {
    if length > MAX_FIELD_BYTES {
        return Err(invalid("SOCKS field exceeds limit"));
    }
    let mut value = vec![0u8; length];
    reader.read_exact(&mut value).await?;
    Ok(value)
}

async fn write_all<R: AsyncRead + AsyncWrite + Unpin>(
    reader: &mut BudgetedReader<'_, R>,
    bytes: &[u8],
) -> io::Result<()> {
    reader.reader.write_all(bytes).await
}

fn validate_domain(value: &[u8]) -> io::Result<String> {
    if value.is_empty()
        || value.len() > MAX_FIELD_BYTES
        || value
            .iter()
            .any(|byte| *byte == 0 || byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return Err(invalid("invalid SOCKS domain"));
    }
    let value = std::str::from_utf8(value).map_err(|_| invalid("SOCKS domain is not UTF-8"))?;
    Ok(value.to_owned())
}

async fn send_failure<R: AsyncWrite + Unpin>(
    stream: &mut R,
    version: SocksVersion,
    reply: u8,
) -> io::Result<()> {
    match version {
        SocksVersion::V4 => {
            let reply = if reply == SOCKS4_AUTH_FAILED {
                SOCKS4_AUTH_FAILED
            } else {
                SOCKS4_REJECTED
            };
            stream.write_all(&[0, reply, 0, 0, 0, 0, 0, 0]).await
        }
        SocksVersion::V5 => stream.write_all(&[5, reply, 0, 1, 0, 0, 0, 0, 0, 0]).await,
    }
}

async fn send_success<R: AsyncWrite + Unpin>(
    stream: &mut R,
    version: SocksVersion,
) -> io::Result<()> {
    match version {
        SocksVersion::V4 => stream.write_all(&[0, SOCKS4_GRANTED, 0, 0, 0, 0, 0, 0]).await,
        SocksVersion::V5 => stream.write_all(&[5, SOCKS5_SUCCEEDED, 0, 1, 0, 0, 0, 0, 0, 0]).await,
    }
}

#[derive(Clone)]
pub(crate) struct SocksConfig {
    pub(crate) name: String,
    pub(crate) bind_address: IpAddr,
    pub(crate) port: u16,
    pub(crate) sam_tcp_port: u16,
    pub(crate) outproxy: Option<OutproxyTarget>,
    pub(crate) proxy_credentials: Option<(String, String)>,
    pub(crate) outproxy_credentials: Option<(String, String)>,
    pub(crate) address_book: Option<Arc<RuntimeAddressBookHandle>>,
    pub(crate) require_auth: bool,
    pub(crate) lifecycle: ClientLifecycleConfig,
    pub(crate) delay_open: bool,
    pub(crate) session_options: SessionOptions,
}

impl fmt::Debug for SocksConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SocksConfig")
            .field("name", &self.name)
            .field("bind_address", &self.bind_address)
            .field("port", &self.port)
            .field("sam_tcp_port", &self.sam_tcp_port)
            .field("outproxy", &self.outproxy)
            .field(
                "proxy_username",
                &self.proxy_credentials.as_ref().map(|(user, _)| user),
            )
            .field(
                "proxy_password",
                &self.proxy_credentials.as_ref().map(|_| "***"),
            )
            .field(
                "outproxy_credentials",
                &self.outproxy_credentials.as_ref().map(|_| "***"),
            )
            .field("require_auth", &self.require_auth)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TargetRoute {
    Direct {
        destination: String,
        port: u16,
    },
    Outproxy {
        destination: String,
        port: u16,
        host: String,
        target_port: u16,
        credentials: Option<(String, String)>,
    },
}

async fn target_route(request: &SocksRequest, config: &SocksConfig) -> Result<TargetRoute, ()> {
    if request.host.parse::<IpAddr>().is_ok() {
        return Err(());
    }
    let host = if crate::i2pcontrol::address_book_runtime::is_valid_full_destination(&request.host)
    {
        request.host.clone()
    } else {
        request.host.to_ascii_lowercase()
    };
    let target = classify_target(&host, request.port, config.outproxy.clone()).map_err(|_| ())?;
    match target {
        HttpTarget::I2p { destination, port } => {
            let destination =
                super::http_client::resolve_destination(&destination, config.address_book.as_ref())
                    .await
                    .ok_or(())?;
            Ok(TargetRoute::Direct { destination, port })
        }
        HttpTarget::Clearnet {
            host,
            port: target_port,
            outproxy,
        } => {
            let destination = super::http_client::resolve_destination(
                &outproxy.destination,
                config.address_book.as_ref(),
            )
            .await
            .ok_or(())?;
            Ok(TargetRoute::Outproxy {
                destination,
                port: outproxy.port,
                host,
                target_port,
                credentials: config.outproxy_credentials.clone(),
            })
        }
    }
}

async fn connect_target(
    connector: &ClientStreamConnector,
    route: TargetRoute,
) -> Result<yosemite_i2pcontrol::Stream, ()> {
    match route {
        TargetRoute::Direct { destination, port } => {
            connector.connect_to(&destination, port).await.map_err(|_| ())
        }
        TargetRoute::Outproxy {
            destination,
            port,
            host,
            target_port,
            credentials,
        } => {
            let mut remote = connector.connect_to(&destination, port).await.map_err(|_| ())?;
            connect_via_outproxy(&mut remote, &host, target_port, credentials.as_ref()).await?;
            Ok(remote)
        }
    }
}

async fn connect_via_outproxy<R: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut R,
    host: &str,
    port: u16,
    credentials: Option<&(String, String)>,
) -> Result<(), ()> {
    tokio::time::timeout(NEGOTIATION_TIMEOUT, async {
        let method = if credentials.is_some() { 2 } else { 0 };
        stream.write_all(&[5, 1, method]).await.map_err(|_| ())?;
        let mut response = [0u8; 2];
        stream.read_exact(&mut response).await.map_err(|_| ())?;
        if response[0] != 5 || response[1] == 0xff || response[1] != method {
            return Err(());
        }
        if method == 2 {
            let (username, password) = credentials.ok_or(())?;
            if username.len() > MAX_FIELD_BYTES || password.len() > MAX_FIELD_BYTES {
                return Err(());
            }
            stream.write_all(&[1, username.len() as u8]).await.map_err(|_| ())?;
            stream.write_all(username.as_bytes()).await.map_err(|_| ())?;
            stream.write_all(&[password.len() as u8]).await.map_err(|_| ())?;
            stream.write_all(password.as_bytes()).await.map_err(|_| ())?;
            let mut auth_response = [0u8; 2];
            stream.read_exact(&mut auth_response).await.map_err(|_| ())?;
            if auth_response != [1, 0] {
                return Err(());
            }
        }
        if host.len() > MAX_FIELD_BYTES {
            return Err(());
        }
        stream.write_all(&[5, 1, 0, 3, host.len() as u8]).await.map_err(|_| ())?;
        stream.write_all(host.as_bytes()).await.map_err(|_| ())?;
        stream.write_all(&port.to_be_bytes()).await.map_err(|_| ())?;
        let mut header = [0u8; 4];
        stream.read_exact(&mut header).await.map_err(|_| ())?;
        if header[0] != 5 || header[1] != 0 {
            return Err(());
        }
        match header[3] {
            1 => {
                let mut address = [0u8; 4];
                stream.read_exact(&mut address).await.map_err(|_| ())?;
            }
            3 => {
                let length = stream.read_u8().await.map_err(|_| ())? as usize;
                if length > MAX_FIELD_BYTES {
                    return Err(());
                }
                let mut address = vec![0u8; length];
                stream.read_exact(&mut address).await.map_err(|_| ())?;
            }
            4 => {
                let mut address = [0u8; 16];
                stream.read_exact(&mut address).await.map_err(|_| ())?;
            }
            _ => return Err(()),
        }
        let mut bind_port = [0u8; 2];
        stream.read_exact(&mut bind_port).await.map_err(|_| ())?;
        Ok(())
    })
    .await
    .map_err(|_| ())?
}

pub(crate) fn make_handler(config: SocksConfig, mode: PayloadMode) -> ClientConnectionHandler {
    Arc::new(move |mut stream, connector| {
        let config = config.clone();
        Box::pin(async move {
            let request = match negotiate(
                &mut stream,
                config.require_auth,
                config.proxy_credentials.as_ref(),
            )
            .await
            {
                Ok(request) => request,
                Err(error) => {
                    if !error.close_only {
                        if let Some(version) = error.version {
                            let _ = send_failure(&mut stream, version, error.reply).await;
                        }
                    }
                    return;
                }
            };
            let route = match target_route(&request, &config).await {
                Ok(route) => route,
                Err(()) => {
                    let _ =
                        send_failure(&mut stream, request.version, SOCKS5_HOST_UNREACHABLE).await;
                    return;
                }
            };
            let Ok(mut remote) = connect_target(&connector, route).await else {
                let _ = send_failure(&mut stream, request.version, SOCKS5_CONNECTION_REFUSED).await;
                return;
            };
            if send_success(&mut stream, request.version).await.is_err() {
                return;
            }
            match mode {
                PayloadMode::Raw => {
                    let _ = tokio::io::copy_bidirectional(&mut stream, &mut remote).await;
                }
                PayloadMode::Irc => {
                    let _ = relay_client_stream(stream, remote).await;
                }
            }
        })
    })
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
pub(crate) struct SocksRuntimeSupervisor {
    inner: Arc<Mutex<RuntimeMap>>,
    tunnel_type: TunnelType,
}

impl SocksRuntimeSupervisor {
    pub(crate) fn new(tunnel_type: TunnelType) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeMap::default())),
            tunnel_type,
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
        tunnel_type: TunnelType,
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
            entry.failure = Some(match tunnel_type {
                TunnelType::Socks => "socks tunnel runtime failed",
                TunnelType::SocksIrc => "socksirc tunnel runtime failed",
                _ => "SOCKS tunnel runtime failed",
            });
        } else {
            entry.state = TunnelRuntimeState::Stopped;
            entry.failure = None;
        }
    }

    pub(crate) async fn start(
        &self,
        config: SocksConfig,
        mode: PayloadMode,
        shared_registry: Option<Arc<super::runtime::session::SharedClientSessionRegistry>>,
        shared: bool,
    ) -> BackendResult<()> {
        let name = config.name.clone();
        let (generation, cancellation) = self.reserve(&name)?;
        let map = Arc::clone(&self.inner);
        let task_name = name.clone();
        let ready_cancellation = cancellation.clone();
        let tunnel_type = self.tunnel_type;
        let handler = make_handler(config.clone(), mode);
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(
                super::runtime::run_client_listener_with_shared_session(
                    ClientListenerRuntimeConfig {
                        name: config.name,
                        bind_address: config.bind_address,
                        port: config.port,
                        destination: "unused-socks-destination".to_owned(),
                        destination_port: 1,
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
                ),
            )
            .catch_unwind()
            .await
            .unwrap_or(Err(ClientListenerRuntimeError::Panicked));
            Self::complete(
                map,
                tunnel_type,
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
                    message: format!("{} tunnel runtime failed to start", self.tunnel_type),
                })
            }
            Ok(Ok(Ok(_))) => {
                let _ = self.stop_generation(&name, generation).await;
                Err(BackendError::Internal {
                    message: format!("{} tunnel runtime exited during start", self.tunnel_type),
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
                message: format!("{} tunnel stop timed out", self.tunnel_type),
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

    pub(crate) async fn stop(&self, name: &str) -> BackendResult<()> {
        let generation = self.inner.lock().entries.get(name).map(|entry| entry.generation);
        match generation {
            Some(generation) => self.stop_generation(name, generation).await,
            None => Ok(()),
        }
    }

    pub(crate) fn inspect(&self, name: &str) -> (TunnelRuntimeState, &'static str) {
        let runtime = self.inner.lock();
        match runtime.entries.get(name) {
            Some(entry) => (
                entry.state,
                entry.failure.unwrap_or(match self.tunnel_type {
                    TunnelType::Socks => "socks tunnel runtime is active",
                    TunnelType::SocksIrc => "socksirc tunnel runtime is active",
                    _ => "SOCKS tunnel runtime is active",
                }),
            ),
            None => (
                TunnelRuntimeState::Stopped,
                match self.tunnel_type {
                    TunnelType::Socks => "socks tunnel runtime is stopped",
                    TunnelType::SocksIrc => "socksirc tunnel runtime is stopped",
                    _ => "SOCKS tunnel runtime is stopped",
                },
            ),
        }
    }
}

pub(crate) fn config_for(
    definition: &TunnelDefinition,
    tunnel_type: TunnelType,
    sam_tcp_port: u16,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
    capabilities: OptionCapabilities,
) -> BackendResult<SocksConfig> {
    if definition.ownership != TunnelOwnership::ControlPlane {
        return Err(BackendError::InvalidState {
            tunnel_type,
            current_state: definition.runtime_state,
            attempted_action: "start",
        });
    }
    validate_options(tunnel_type, &definition.options, capabilities).map_err(option_error)?;
    validate_raw_options(definition, tunnel_type)?;
    let bind_address = match definition.options.listen_interface.as_deref() {
        None => "127.0.0.1".parse().expect("loopback address is valid"),
        Some(value) => value.parse::<IpAddr>().map_err(|_| BackendError::Internal {
            message: format!("{} listen interface must be an IP address", tunnel_type),
        })?,
    };
    let username = raw_string(definition, "ProxyUsername")
        .or_else(|| definition.options.proxy_username.clone());
    let password = raw_secret(definition, "ProxyPassword")
        .or_else(|| definition.options.proxy_password.as_deref().map(str::to_owned));
    let proxy_credentials = credentials(username, password, tunnel_type)?;
    let require_auth = raw_bool(definition, "ProxyAuth", tunnel_type)?.unwrap_or(false)
        || proxy_credentials.is_some()
        || !bind_address.is_loopback();
    if require_auth && proxy_credentials.is_none() {
        return Err(BackendError::Internal {
            message: format!(
                "{} non-loopback listeners require proxy authentication",
                tunnel_type
            ),
        });
    }

    let proxy_list = raw_string(definition, "ProxyList");
    let outproxy = proxy_list
        .as_deref()
        .map(|value| parse_outproxy(value, tunnel_type))
        .transpose()?;
    let outproxy_username = raw_string(definition, "OutproxyUsername");
    let outproxy_password = raw_secret(definition, "OutproxyPassword")
        .or_else(|| definition.options.outproxy_password.as_deref().map(str::to_owned));
    let outproxy_credentials = credentials(outproxy_username, outproxy_password, tunnel_type)?;
    let outproxy_auth = raw_bool(definition, "OutproxyAuth", tunnel_type)?.unwrap_or(false);
    if outproxy_auth && outproxy_credentials.is_none() {
        return Err(BackendError::Internal {
            message: format!("{} outproxy credentials are incomplete", tunnel_type),
        });
    }
    if outproxy_credentials.is_some() && outproxy.is_none() {
        return Err(BackendError::Internal {
            message: format!("{} outproxy credentials require ProxyList", tunnel_type),
        });
    }
    if raw_string(definition, "OutproxyType").is_some() && outproxy.is_none() {
        return Err(BackendError::UnsupportedOption {
            tunnel_type,
            option: "OutproxyType".to_owned(),
        });
    }
    Ok(SocksConfig {
        name: definition.name.as_str().to_owned(),
        bind_address,
        port: definition.options.listen_port.ok_or_else(|| BackendError::MissingOption {
            tunnel_type,
            option: "ListenPort".to_owned(),
        })?,
        sam_tcp_port,
        outproxy,
        proxy_credentials,
        outproxy_credentials,
        address_book,
        require_auth,
        lifecycle: client_lifecycle_config(definition)?,
        delay_open: definition.options.delay_open.unwrap_or(false),
        session_options: SessionOptions::default(),
    })
}

fn credentials(
    username: Option<String>,
    password: Option<String>,
    tunnel_type: TunnelType,
) -> BackendResult<Option<(String, String)>> {
    match (username, password) {
        (None, None) => Ok(None),
        (Some(username), Some(password))
            if !username.is_empty()
                && !password.is_empty()
                && username.len() <= MAX_FIELD_BYTES
                && password.len() <= MAX_FIELD_BYTES =>
        {
            Ok(Some((username, password)))
        }
        _ => Err(BackendError::Internal {
            message: format!(
                "{} proxy credentials are invalid or incomplete",
                tunnel_type
            ),
        }),
    }
}

fn parse_outproxy(value: &str, tunnel_type: TunnelType) -> BackendResult<OutproxyTarget> {
    let value = value.trim();
    if value.is_empty() || value.contains(',') {
        return Err(BackendError::UnsupportedOption {
            tunnel_type,
            option: "ProxyList".to_owned(),
        });
    }
    let (destination, port) =
        parse_authority(value.trim()).map_err(|_| BackendError::Internal {
            message: "socks outproxy is invalid".to_owned(),
        })?;
    if !(destination.ends_with(".i2p")
        || destination.ends_with(".b32.i2p")
        || crate::i2pcontrol::address_book_runtime::is_valid_full_destination(&destination))
    {
        return Err(BackendError::Internal {
            message: "socks outproxy must be an I2P destination".to_owned(),
        });
    }
    let port = port.unwrap_or(1080);
    if port == 0 {
        return Err(BackendError::Internal {
            message: format!("{} outproxy port is invalid", tunnel_type),
        });
    }
    Ok(OutproxyTarget { destination, port })
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

fn raw_bool(
    definition: &TunnelDefinition,
    key: &str,
    tunnel_type: TunnelType,
) -> BackendResult<Option<bool>> {
    definition
        .raw_config
        .get(key)
        .map(|value| {
            value.as_bool().ok_or_else(|| BackendError::UnsupportedOption {
                tunnel_type,
                option: key.to_owned(),
            })
        })
        .transpose()
}

fn validate_raw_options(
    definition: &TunnelDefinition,
    tunnel_type: TunnelType,
) -> BackendResult<()> {
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
        "ConnectDelay",
        "Reduce",
        "ReduceCount",
        "ReduceTime",
        "Shared",
        "PersistentClientKey",
        "PrivKeyFile",
    ];
    // M121: "Close", "CloseTime", and "NewDest" demoted to blocked_primitive;
    // "ConnectDelay" remains applied via the shared lifecycle connector.
    // M136: Reduce family supported via the canonical idle owner.
    for key in definition.raw_config.keys() {
        if key.starts_with("__emissary_") || SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        return Err(BackendError::UnsupportedOption {
            tunnel_type,
            option: key.clone(),
        });
    }
    if let Some(kind) = raw_string(definition, "OutproxyType") {
        if !kind.eq_ignore_ascii_case("socks") && !kind.eq_ignore_ascii_case("socks5") {
            return Err(BackendError::UnsupportedOption {
                tunnel_type,
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

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Clone)]
pub struct SocksTunnelBackend {
    supervisor: SocksRuntimeSupervisor,
    sam_tcp_port: u16,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
    shared_registry: Option<Arc<super::runtime::session::SharedClientSessionRegistry>>,
    client_destinations: Option<ClientDestinationStore>,
}

impl fmt::Debug for SocksTunnelBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SocksTunnelBackend")
            .field("sam_tcp_port", &self.sam_tcp_port)
            .finish_non_exhaustive()
    }
}

impl SocksTunnelBackend {
    pub fn new(sam_tcp_port: u16) -> Self {
        Self {
            supervisor: SocksRuntimeSupervisor::new(TunnelType::Socks),
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
}

#[async_trait::async_trait]
impl TunnelBackend for SocksTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::Socks
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        let mut config = config_for(
            definition,
            TunnelType::Socks,
            self.sam_tcp_port,
            self.address_book.clone(),
            SOCKS_OPTIONS,
        )?;
        config.session_options = super::runtime::session::build_client_session_options(
            definition,
            self.sam_tcp_port,
            self.client_destinations.as_ref(),
        )
        .await?;
        self.supervisor
            .start(
                config,
                PayloadMode::Raw,
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
            tunnel_type: TunnelType::Socks,
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
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

    fn config(require_auth: bool) -> SocksConfig {
        SocksConfig {
            name: "socks-test".to_owned(),
            bind_address: "127.0.0.1".parse().unwrap(),
            port: 0,
            sam_tcp_port: 1,
            outproxy: None,
            proxy_credentials: require_auth.then(|| ("user".to_owned(), "secret".to_owned())),
            outproxy_credentials: None,
            address_book: None,
            require_auth,
            lifecycle: ClientLifecycleConfig::DISABLED,
            delay_open: false,
            session_options: SessionOptions::default(),
        }
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
                            "SESSION STATUS RESULT=OK DESTINATION=socks-test\n"
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
    async fn socks4a_domain_connect_is_bounded_and_ignores_userid() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        let task = tokio::spawn(async move {
            let request = negotiate(&mut server, false, None).await.unwrap();
            assert_eq!(request.host, "peer.b32.i2p");
            assert_eq!(request.port, 4444);
        });
        client
            .write_all(b"\x04\x01\x11\x5c\x00\x00\x00\x01ignored\0peer.b32.i2p\0")
            .await
            .unwrap();
        task.await.unwrap();
    }

    #[tokio::test]
    async fn socks4_literal_and_port_zero_fail_before_request() {
        for request in [
            b"\x04\x01\x01\xbb\x01\x02\x03\x04user\0".as_slice(),
            b"\x04\x01\0\x00\x00\x00\x00\x01user\0peer.i2p\0".as_slice(),
        ] {
            let (mut client, mut server) = tokio::io::duplex(1024);
            let task = tokio::spawn(async move { negotiate(&mut server, false, None).await });
            client.write_all(request).await.unwrap();
            assert!(task.await.unwrap().is_err());
        }
    }

    #[tokio::test]
    async fn socks5_auth_success_and_failure_are_protocol_correct() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        let task = tokio::spawn(async move {
            let request = negotiate(
                &mut server,
                true,
                Some(&("user".to_owned(), "secret".to_owned())),
            )
            .await
            .unwrap();
            assert_eq!(request.host, "peer.i2p");
        });
        client.write_all(b"\x05\x01\x02").await.unwrap();
        let mut method = [0u8; 2];
        client.read_exact(&mut method).await.unwrap();
        assert_eq!(method, [5, 2]);
        client.write_all(b"\x01\x04user\x06secret").await.unwrap();
        let mut auth = [0u8; 2];
        client.read_exact(&mut auth).await.unwrap();
        assert_eq!(auth, [1, 0]);
        client.write_all(b"\x05\x01\x00\x03\x08peer.i2p\x01\xbb").await.unwrap();
        task.await.unwrap();

        let (mut client, mut server) = tokio::io::duplex(1024);
        let task = tokio::spawn(async move {
            negotiate(
                &mut server,
                true,
                Some(&("user".to_owned(), "secret".to_owned())),
            )
            .await
        });
        client.write_all(b"\x05\x01\x02").await.unwrap();
        client.read_exact(&mut method).await.unwrap();
        client.write_all(b"\x01\x04user\x05wrong").await.unwrap();
        client.read_exact(&mut auth).await.unwrap();
        assert_eq!(auth, [1, 1]);
        assert!(task.await.unwrap().is_err());
    }

    #[tokio::test]
    async fn socks5_unsupported_commands_and_literal_addresses_fail_without_connect() {
        for request in [
            b"\x05\x01\0\x05\0\0\0\x01\0\0\0\0\0\x50".as_slice(),
            b"\x05\x01\0\x01\x7f\0\0\x01\0\x50".as_slice(),
            b"\x05\x01\0\x03\x08host.i2p\0\0".as_slice(),
        ] {
            let (mut client, mut server) = tokio::io::duplex(1024);
            let task = tokio::spawn(async move { negotiate(&mut server, false, None).await });
            client.write_all(request).await.unwrap();
            let mut method = [0u8; 2];
            client.read_exact(&mut method).await.unwrap();
            assert_eq!(method, [5, 0]);
            assert!(task.await.unwrap().is_err());
        }
    }

    #[tokio::test]
    async fn both_payload_modes_start_stop_and_restart_with_exact_generation_cleanup() {
        let (sam_port, sam_task) = fake_sam().await;
        for (tunnel_type, mode, name) in [
            (TunnelType::Socks, PayloadMode::Raw, "socks"),
            (TunnelType::SocksIrc, PayloadMode::Irc, "socksirc"),
        ] {
            let supervisor = SocksRuntimeSupervisor::new(tunnel_type);
            let mut runtime_config = config(false);
            runtime_config.name = name.to_owned();
            runtime_config.sam_tcp_port = sam_port;
            runtime_config.session_options.samv3_tcp_port = sam_port;
            runtime_config.session_options.nickname = name.to_owned();
            supervisor.start(runtime_config.clone(), mode, None, false).await.unwrap();
            assert_eq!(supervisor.inspect(name).0, TunnelRuntimeState::Running);
            assert!(supervisor.start(runtime_config.clone(), mode, None, false).await.is_err());
            supervisor.stop(name).await.unwrap();
            assert_eq!(supervisor.inspect(name).0, TunnelRuntimeState::Stopped);
            supervisor.start(runtime_config, mode, None, false).await.unwrap();
            supervisor.stop(name).await.unwrap();
        }
        sam_task.abort();
    }

    #[test]
    fn target_routing_rejects_literal_and_requires_i2p_outproxy() {
        let request = SocksRequest {
            version: SocksVersion::V5,
            host: "127.0.0.1".to_owned(),
            port: 80,
        };
        let config = config(false);
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        assert!(runtime.block_on(target_route(&request, &config)).is_err());
        let request = SocksRequest {
            host: "example.com".to_owned(),
            ..request
        };
        assert!(runtime.block_on(target_route(&request, &config)).is_err());
    }

    #[test]
    fn config_rejects_unsupported_options_before_runtime_reservation() {
        let definition = TunnelDefinition {
            name: TunnelName::new("socks").unwrap(),
            tunnel_type: TunnelType::Socks,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions {
                listen_port: Some(0),
                ..Default::default()
            },
            raw_config: std::collections::BTreeMap::from([(
                "UDPAssociate".to_owned(),
                serde_json::json!(true),
            )]),
        };
        assert!(matches!(
            config_for(&definition, TunnelType::Socks, 1, None, SOCKS_OPTIONS),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "UDPAssociate"
        ));
    }

    #[test]
    fn socksirc_cannot_regress_to_raw_relay() {
        let shared_source = include_str!("socks.rs");
        let composition_source = include_str!("socks_irc.rs");
        assert!(shared_source.contains("PayloadMode::Irc"));
        assert!(shared_source.contains("relay_client_stream"));
        assert!(!composition_source.contains("copy_bidirectional"));
    }
}
