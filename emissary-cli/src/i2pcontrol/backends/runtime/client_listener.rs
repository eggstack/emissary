use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::{future::BoxFuture, FutureExt};
use parking_lot::Mutex;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot, watch, Notify},
};
use yosemite_i2pcontrol::{style, Session, SessionOptions, StreamOptions};

use super::task_group::BoundedTaskGroup;
use super::session::{SharedClientSessionRegistry, SharedStreamSessionLease};

const STOP_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_MAX_CONNECTIONS: usize = 128;

/// A callback for one local client connection.
pub type ClientConnectionHandler =
    Arc<dyn Fn(TcpStream, ClientStreamConnector) -> BoxFuture<'static, ()> + Send + Sync>;

type SharedSession = Arc<Mutex<Session<style::Stream>>>;

struct SessionResource {
    session: SharedSession,
    _shared_lease: Option<SharedStreamSessionLease>,
}

enum EagerSession {
    Local(Box<Session<style::Stream>>),
    Shared(SharedStreamSessionLease),
}

struct SessionState {
    session: Option<SessionResource>,
    creating: bool,
    started: bool,
}

/// Owns the session for one listener generation.
///
/// A session may be dropped and recreated after an idle close. Creation is
/// serialized by the small state machine, while network I/O remains outside
/// the bookkeeping lock. The owner is generation-local, so a cancelled old
/// generation cannot mutate a restarted tunnel.
struct ClientSessionOwner {
    state: tokio::sync::Mutex<SessionState>,
    changed: Arc<Notify>,
    session_options: SessionOptions,
    resume_session_options: SessionOptions,
    cancellation: watch::Receiver<bool>,
    setup_failed: mpsc::UnboundedSender<()>,
    shared_registry: Option<Arc<SharedClientSessionRegistry>>,
    shared: bool,
}

impl ClientSessionOwner {
    fn delayed(
        session_options: SessionOptions,
        resume_session_options: SessionOptions,
        cancellation: watch::Receiver<bool>,
        setup_failed: mpsc::UnboundedSender<()>,
        shared_registry: Option<Arc<SharedClientSessionRegistry>>,
        shared: bool,
    ) -> Self {
        Self {
            state: tokio::sync::Mutex::new(SessionState {
                session: None,
                creating: false,
                started: false,
            }),
            changed: Arc::new(Notify::new()),
            session_options,
            resume_session_options,
            cancellation,
            setup_failed,
            shared_registry,
            shared,
        }
    }

    fn eager(
        session: Session<style::Stream>,
        resume_session_options: SessionOptions,
        cancellation: watch::Receiver<bool>,
        setup_failed: mpsc::UnboundedSender<()>,
        shared_lease: Option<SharedStreamSessionLease>,
    ) -> Self {
        let session = shared_lease
            .as_ref()
            .map_or_else(|| Arc::new(Mutex::new(session)), |lease| Arc::clone(&lease.session));
        Self {
            state: tokio::sync::Mutex::new(SessionState {
                session: Some(SessionResource {
                    session,
                    _shared_lease: shared_lease,
                }),
                creating: false,
                started: true,
            }),
            changed: Arc::new(Notify::new()),
            session_options: SessionOptions::default(),
            resume_session_options,
            cancellation,
            setup_failed,
            shared_registry: None,
            shared: false,
        }
    }

    fn eager_shared(
        lease: SharedStreamSessionLease,
        resume_session_options: SessionOptions,
        cancellation: watch::Receiver<bool>,
        setup_failed: mpsc::UnboundedSender<()>,
    ) -> Self {
        Self {
            state: tokio::sync::Mutex::new(SessionState {
                session: Some(SessionResource {
                    session: Arc::clone(&lease.session),
                    _shared_lease: Some(lease),
                }),
                creating: false,
                started: true,
            }),
            changed: Arc::new(Notify::new()),
            session_options: SessionOptions::default(),
            resume_session_options,
            cancellation,
            setup_failed,
            shared_registry: None,
            shared: false,
        }
    }

    async fn get(&self) -> Result<SharedSession, ClientListenerRuntimeError> {
        loop {
            let waiter = {
                let mut state = self.state.lock().await;
                if let Some(resource) = &state.session {
                    return Ok(Arc::clone(&resource.session));
                }
                if state.creating {
                    let mut waiter = Box::pin(Arc::clone(&self.changed).notified_owned());
                    waiter.as_mut().enable();
                    Some(waiter)
                } else {
                    state.creating = true;
                    None
                }
            };
            if let Some(waiter) = waiter {
                waiter.await;
                continue;
            }

            let session_options = {
                let state = self.state.lock().await;
                if state.session.is_some() {
                    continue;
                }
                if state.started {
                    self.resume_session_options.clone()
                } else {
                    self.session_options.clone()
                }
            };
            let cancellation = self.cancellation.clone();
            let shared_registry = self.shared_registry.clone();
            let result = tokio::select! {
                biased;
                _ = cancellation_won(cancellation) => Err(ClientListenerRuntimeError::SessionSetup),
                result = async {
                    if self.shared {
                        let Some(registry) = shared_registry else {
                            return Err(ClientListenerRuntimeError::SessionSetup);
                        };
                        registry.acquire_stream(session_options).await
                            .map(|lease| SessionResource {
                                session: Arc::clone(&lease.session),
                                _shared_lease: Some(lease),
                            })
                            .map_err(|_| ClientListenerRuntimeError::SessionSetup)
                    } else {
                        Session::<style::Stream>::new(session_options).await
                            .map(|session| SessionResource {
                                session: Arc::new(Mutex::new(session)),
                                _shared_lease: None,
                            })
                            .map_err(|_| ClientListenerRuntimeError::SessionSetup)
                    }
                } => result,
            };
            let mut state = self.state.lock().await;
            state.creating = false;
            state.started = true;
            match result {
                Ok(resource) => {
                    let session = Arc::clone(&resource.session);
                    state.session = Some(resource);
                    self.changed.notify_waiters();
                    return Ok(session);
                }
                Err(error) => {
                    self.changed.notify_waiters();
                    let _ = self.setup_failed.send(());
                    return Err(error);
                }
            }
        }
    }

    async fn close_if_idle(&self) {
        let mut state = self.state.lock().await;
        let session = if !state.creating {
            // Dropping SessionResource closes the generation-owned SAM stream.
            // Shared lifecycle options are rejected before reaching here.
            state.session.take()
        } else {
            None
        };
        drop(state);
        drop(session);
    }
}

async fn cancellation_won(mut cancellation: watch::Receiver<bool>) {
    if !*cancellation.borrow() {
        let _ = cancellation.changed().await;
    }
}

struct ConnectionActivity {
    active: AtomicUsize,
    changed: Notify,
}

impl ConnectionActivity {
    fn new() -> Self {
        Self {
            active: AtomicUsize::new(0),
            changed: Notify::new(),
        }
    }

    fn acquire(self: &Arc<Self>) -> ConnectionActivityGuard {
        self.active.fetch_add(1, Ordering::AcqRel);
        ConnectionActivityGuard {
            activity: Arc::clone(self),
        }
    }
}

struct ConnectionActivityGuard {
    activity: Arc<ConnectionActivity>,
}

impl Drop for ConnectionActivityGuard {
    fn drop(&mut self) {
        self.activity.active.fetch_sub(1, Ordering::AcqRel);
        self.activity.changed.notify_one();
    }
}

async fn run_idle_closer(
    session: Arc<ClientSessionOwner>,
    activity: Arc<ConnectionActivity>,
    cancellation: watch::Receiver<bool>,
    idle_time: Duration,
) {
    loop {
        let mut activity_changed = Box::pin(activity.changed.notified());
        activity_changed.as_mut().enable();
        if activity.active.load(Ordering::Acquire) != 0 {
            tokio::select! {
                _ = cancellation_won(cancellation.clone()) => return,
                _ = &mut activity_changed => continue,
            }
        }
        tokio::select! {
            _ = cancellation_won(cancellation.clone()) => return,
            _ = &mut activity_changed => continue,
            _ = tokio::time::sleep(idle_time) => {
                if activity.active.load(Ordering::Acquire) == 0 {
                    session.close_if_idle().await;
                }
            }
        }
    }
}

/// The narrow session capability exposed to a client protocol handler.
#[derive(Clone)]
pub struct ClientStreamConnector {
    session: Arc<ClientSessionOwner>,
    destination: Arc<str>,
    destination_port: u16,
    connect_delay: Option<Duration>,
    cancellation: watch::Receiver<bool>,
}

impl fmt::Debug for ClientStreamConnector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientStreamConnector")
            .field("destination", &self.destination)
            .field("destination_port", &self.destination_port)
            .finish()
    }
}

impl ClientStreamConnector {
    /// Open one outbound stream without holding the session mutex across I/O.
    pub async fn connect(&self) -> Result<yosemite_i2pcontrol::Stream, ClientListenerRuntimeError> {
        self.connect_to(self.destination.as_ref(), self.destination_port).await
    }

    /// Open one outbound stream to a request-selected I2P destination without
    /// holding the session mutex across I/O.
    pub async fn connect_to(
        &self,
        destination: &str,
        destination_port: u16,
    ) -> Result<yosemite_i2pcontrol::Stream, ClientListenerRuntimeError> {
        if let Some(delay) = self.connect_delay {
            tokio::select! {
                _ = tokio::time::sleep(delay) => {},
                _ = cancellation_won(self.cancellation.clone()) => {
                    return Err(ClientListenerRuntimeError::SessionSetup);
                }
            }
        }
        let session = self.session.get().await?;
        let future = {
            let mut session = session.lock();
            session.connect_detached_with_options(
                destination,
                StreamOptions {
                    dst_port: destination_port,
                    ..Default::default()
                },
            )
        };

        future.await.map_err(|_| ClientListenerRuntimeError::StreamSetup)
    }
}

/// Configuration for one control-plane-owned local client listener.
pub struct ClientListenerRuntimeConfig {
    pub name: String,
    pub bind_address: IpAddr,
    pub port: u16,
    pub destination: String,
    pub destination_port: u16,
    pub sam_tcp_port: u16,
    pub max_connections: usize,
    pub delay_open: bool,
    pub connect_delay: Option<Duration>,
    pub close_on_idle: bool,
    pub close_idle_time: Duration,
    pub new_dest_on_resume: bool,
    pub session_options: SessionOptions,
    pub handler: ClientConnectionHandler,
}

impl fmt::Debug for ClientListenerRuntimeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientListenerRuntimeConfig")
            .field("name", &self.name)
            .field("bind_address", &self.bind_address)
            .field("port", &self.port)
            .field("destination", &self.destination)
            .field("destination_port", &self.destination_port)
            .field("sam_tcp_port", &self.sam_tcp_port)
            .field("max_connections", &self.max_connections)
            .field("delay_open", &self.delay_open)
            .field("connect_delay", &self.connect_delay)
            .field("close_on_idle", &self.close_on_idle)
            .field("close_idle_time", &self.close_idle_time)
            .field("new_dest_on_resume", &self.new_dest_on_resume)
            .finish_non_exhaustive()
    }
}

/// Errors from the client listener lifecycle, with no raw SAM or request data.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ClientListenerRuntimeError {
    #[error("client listener session setup failed")]
    SessionSetup,
    #[error("client listener bind failed")]
    Bind,
    #[error("client stream setup failed")]
    StreamSetup,
    #[error("client listener runtime panicked")]
    Panicked,
}

/// Own a local listener and one outbound Yosemite streaming session until cancelled.
pub async fn run_client_listener(
    config: ClientListenerRuntimeConfig,
    cancellation: watch::Receiver<bool>,
    ready: oneshot::Sender<Result<SocketAddr, ClientListenerRuntimeError>>,
) -> Result<(), ClientListenerRuntimeError> {
    run_client_listener_with_shared_session(config, cancellation, ready, None, false).await
}

/// Run a client listener using a shared session owner when requested.
pub async fn run_client_listener_with_shared_session(
    config: ClientListenerRuntimeConfig,
    mut cancellation: watch::Receiver<bool>,
    ready: oneshot::Sender<Result<SocketAddr, ClientListenerRuntimeError>>,
    shared_registry: Option<Arc<SharedClientSessionRegistry>>,
    shared: bool,
) -> Result<(), ClientListenerRuntimeError> {
    let (setup_failed, mut setup_failed_rx) = mpsc::unbounded_channel();
    let eager_session = if config.delay_open {
        None
    } else {
        Some(tokio::select! {
            _ = cancellation.changed() => {
                let _ = ready.send(Err(ClientListenerRuntimeError::SessionSetup));
                return Ok(());
            }
            result = async {
                if shared {
                    let Some(registry) = shared_registry.as_ref() else {
                        return Err(());
                    };
                    registry
                        .acquire_stream(config.session_options.clone())
                        .await
                        .map(EagerSession::Shared)
                        .map_err(|_| ())
                } else {
                    Session::<style::Stream>::new(config.session_options.clone()).await
                        .map_err(|_| ())
                        .map(|session| EagerSession::Local(Box::new(session)))
                }
            } => match result {
                Ok(session) => session,
                Err(_) => {
                    let _ = ready.send(Err(ClientListenerRuntimeError::SessionSetup));
                    return Err(ClientListenerRuntimeError::SessionSetup);
                }
            },
        })
    };

    let listener = match TcpListener::bind(SocketAddr::new(config.bind_address, config.port)).await
    {
        Ok(listener) => listener,
        Err(_) => {
            let _ = ready.send(Err(ClientListenerRuntimeError::Bind));
            return Err(ClientListenerRuntimeError::Bind);
        }
    };
    let local_address = listener.local_addr().map_err(|_| ClientListenerRuntimeError::Bind)?;
    let initial_session_options = config.session_options.clone();
    let mut resume_session_options = initial_session_options.clone();
    if config.new_dest_on_resume {
        resume_session_options.destination = yosemite_i2pcontrol::DestinationKind::Transient;
    }
    let session = Arc::new(match eager_session {
        Some(EagerSession::Shared(lease)) => {
            ClientSessionOwner::eager_shared(
                lease,
                resume_session_options.clone(),
                cancellation.clone(),
                setup_failed,
            )
        }
        Some(EagerSession::Local(session)) => {
            ClientSessionOwner::eager(
                *session,
                resume_session_options.clone(),
                cancellation.clone(),
                setup_failed,
                None,
            )
        },
        None => {
            ClientSessionOwner::delayed(
                initial_session_options,
                resume_session_options,
                cancellation.clone(),
                setup_failed,
                shared_registry,
                shared,
            )
        }
    });
    let connector = ClientStreamConnector {
        session,
        destination: Arc::from(config.destination),
        destination_port: config.destination_port,
        connect_delay: config.connect_delay,
        cancellation: cancellation.clone(),
    };
    let max_connections = config.max_connections.clamp(1, DEFAULT_MAX_CONNECTIONS);
    let handler = config.handler;
    let _ = ready.send(Ok(local_address));
    let mut tasks = BoundedTaskGroup::new(max_connections);
    let activity = Arc::new(ConnectionActivity::new());
    let mut idle_closer = config.close_on_idle.then(|| {
        tokio::spawn(run_idle_closer(
            Arc::clone(&connector.session),
            Arc::clone(&activity),
            cancellation.clone(),
            config.close_idle_time,
        ))
    });

    loop {
        if *cancellation.borrow() {
            break;
        }
        tokio::select! {
            biased;
            _ = cancellation.changed() => break,
            Some(()) = setup_failed_rx.recv() => {
                tasks.drain(STOP_TIMEOUT).await;
                if let Some(idle_closer) = idle_closer.take() {
                    idle_closer.abort();
                    let _ = idle_closer.await;
                }
                return Err(ClientListenerRuntimeError::SessionSetup);
            }
            result = listener.accept() => {
                let Ok((stream, _)) = result else { break };
                let handler = Arc::clone(&handler);
                let connector = connector.clone();
                let activity_guard = activity.acquire();
                let _ = tasks.try_spawn(async move {
                    let _activity_guard = activity_guard;
                    let _ = std::panic::AssertUnwindSafe((handler)(stream, connector))
                        .catch_unwind()
                        .await;
                });
            }
            result = tasks.join_next(), if !tasks.is_empty() => {
                let _ = result;
            }
        }
    }

    tasks.drain(STOP_TIMEOUT).await;
    if let Some(idle_closer) = idle_closer.take() {
        idle_closer.abort();
        let _ = idle_closer.await;
    }
    Ok(())
}

/// Run the plain client tunnel shape used by the generic `client` backend.
/// Keeping it in the I2PControl runtime owner lets that backend apply the
/// same validated session settings as the filtered client families.
pub async fn run_generic_client(
    config: ClientListenerRuntimeConfig,
    cancellation: watch::Receiver<bool>,
    ready: oneshot::Sender<Result<SocketAddr, ClientListenerRuntimeError>>,
) -> Result<(), ClientListenerRuntimeError> {
    run_generic_client_with_shared_session(config, cancellation, ready, None, false).await
}

/// Run the generic client through the shared-session-aware listener owner.
pub async fn run_generic_client_with_shared_session(
    config: ClientListenerRuntimeConfig,
    cancellation: watch::Receiver<bool>,
    ready: oneshot::Sender<Result<SocketAddr, ClientListenerRuntimeError>>,
    shared_registry: Option<Arc<SharedClientSessionRegistry>>,
    shared: bool,
) -> Result<(), ClientListenerRuntimeError> {
    let handler: ClientConnectionHandler = Arc::new(|mut local, connector| {
        Box::pin(async move {
            if let Ok(mut remote) = connector.connect().await {
                let _ = tokio::io::copy_bidirectional(&mut local, &mut remote).await;
            }
        })
    });
    let config = ClientListenerRuntimeConfig { handler, ..config };
    run_client_listener_with_shared_session(config, cancellation, ready, shared_registry, shared)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        sync::Notify,
    };

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
                            "SESSION STATUS RESULT=OK DESTINATION=client-destination\n"
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

    async fn counting_fake_sam() -> (
        u16,
        tokio::task::JoinHandle<()>,
        Arc<AtomicUsize>,
        Arc<AtomicUsize>,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let hellos = Arc::new(AtomicUsize::new(0));
        let session_creates = Arc::new(AtomicUsize::new(0));
        let hellos_for_task = Arc::clone(&hellos);
        let session_creates_for_task = Arc::clone(&session_creates);
        let task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let hellos = Arc::clone(&hellos_for_task);
                let session_creates = Arc::clone(&session_creates_for_task);
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            break;
                        }
                        if line.starts_with("HELLO") {
                            hellos.fetch_add(1, Ordering::AcqRel);
                        } else if line.starts_with("SESSION CREATE") {
                            session_creates.fetch_add(1, Ordering::AcqRel);
                        }
                        let response = if line.starts_with("HELLO") {
                            "HELLO REPLY RESULT=OK VERSION=3.3\n"
                        } else if line.starts_with("SESSION CREATE") {
                            "SESSION STATUS RESULT=OK DESTINATION=client-destination\n"
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
        (port, task, hellos, session_creates)
    }

    async fn gated_fake_sam(
        started: Arc<Notify>,
        release: Arc<Notify>,
    ) -> (u16, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let started = Arc::clone(&started);
                let release = Arc::clone(&release);
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                            break;
                        }
                        if line.starts_with("HELLO") {
                            started.notify_one();
                            release.notified().await;
                        }
                        let response = if line.starts_with("HELLO") {
                            "HELLO REPLY RESULT=OK VERSION=3.3\n"
                        } else if line.starts_with("SESSION CREATE") {
                            "SESSION STATUS RESULT=OK DESTINATION=client-destination\n"
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

    fn config(
        sam_tcp_port: u16,
        handler: ClientConnectionHandler,
        max_connections: usize,
    ) -> ClientListenerRuntimeConfig {
        ClientListenerRuntimeConfig {
            name: "client-listener".to_owned(),
            bind_address: "127.0.0.1".parse().unwrap(),
            port: 0,
            destination: "remote-destination".to_owned(),
            destination_port: 8080,
            sam_tcp_port,
            max_connections,
            delay_open: false,
            connect_delay: None,
            close_on_idle: false,
            close_idle_time: Duration::from_secs(30 * 60),
            new_dest_on_resume: false,
            session_options: SessionOptions {
                samv3_tcp_port: sam_tcp_port,
                nickname: "client-listener".to_owned(),
                ..Default::default()
            },
            handler,
        }
    }

    #[tokio::test]
    async fn listener_reports_bound_address_and_stops_with_bounded_tasks() {
        let (sam_port, sam_task) = fake_sam().await;
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = Arc::clone(&calls);
        let handler: ClientConnectionHandler = Arc::new(move |mut stream, _connector| {
            let calls = Arc::clone(&calls_for_handler);
            Box::pin(async move {
                calls.fetch_add(1, Ordering::Release);
                let _ = stream.shutdown().await;
            })
        });
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_client_listener(
            config(sam_port, handler, 2),
            cancel_rx,
            ready_tx,
        ));
        let address = ready_rx.await.unwrap().unwrap();
        let stream = TcpStream::connect(address).await.unwrap();
        drop(stream);
        tokio::time::timeout(Duration::from_secs(1), async {
            while calls.load(Ordering::Acquire) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        cancel_tx.send(true).unwrap();
        assert!(runtime.await.unwrap().is_ok());
        sam_task.abort();
    }

    #[tokio::test]
    async fn handler_panic_does_not_stop_listener() {
        let (sam_port, sam_task) = fake_sam().await;
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = Arc::clone(&calls);
        let handler: ClientConnectionHandler = Arc::new(move |stream, _connector| {
            let calls = Arc::clone(&calls_for_handler);
            Box::pin(async move {
                let count = calls.fetch_add(1, Ordering::AcqRel);
                if count == 0 {
                    panic!("test handler panic");
                }
                drop(stream);
            })
        });
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_client_listener(
            config(sam_port, handler, 1),
            cancel_rx,
            ready_tx,
        ));
        let address = ready_rx.await.unwrap().unwrap();
        drop(TcpStream::connect(address).await.unwrap());
        tokio::time::timeout(Duration::from_secs(1), async {
            while calls.load(Ordering::Acquire) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        drop(TcpStream::connect(address).await.unwrap());
        tokio::time::timeout(Duration::from_secs(1), async {
            while calls.load(Ordering::Acquire) < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        cancel_tx.send(true).unwrap();
        assert!(runtime.await.unwrap().is_ok());
        sam_task.abort();
    }

    #[tokio::test]
    async fn session_setup_failure_leaves_no_ready_listener() {
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let result = run_client_listener(
            config(1, Arc::new(|_, _| Box::pin(async {})), 1),
            cancel_rx,
            ready_tx,
        )
        .await;
        assert_eq!(result, Err(ClientListenerRuntimeError::SessionSetup));
        assert_eq!(
            ready_rx.await.unwrap(),
            Err(ClientListenerRuntimeError::SessionSetup)
        );
    }

    #[tokio::test]
    async fn delayed_listener_binds_without_sam_until_first_connection() {
        let (sam_port, sam_task, hellos, session_creates) = counting_fake_sam().await;
        let handler: ClientConnectionHandler = Arc::new(|stream, connector| {
            Box::pin(async move {
                let _ = connector.connect().await;
                drop(stream);
            })
        });
        let mut runtime_config = config(sam_port, handler, 2);
        runtime_config.delay_open = true;
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_client_listener(runtime_config, cancel_rx, ready_tx));
        let address = ready_rx.await.unwrap().unwrap();
        tokio::time::sleep(Duration::from_millis(25)).await;
        assert_eq!(hellos.load(Ordering::Acquire), 0);
        assert_eq!(session_creates.load(Ordering::Acquire), 0);

        drop(TcpStream::connect(address).await.unwrap());
        tokio::time::timeout(Duration::from_secs(1), async {
            while session_creates.load(Ordering::Acquire) < 1 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(session_creates.load(Ordering::Acquire), 1);
        cancel_tx.send(true).unwrap();
        assert!(runtime.await.unwrap().is_ok());
        sam_task.abort();
    }

    #[tokio::test]
    async fn connect_delay_is_applied_before_remote_session_use() {
        let (sam_port, sam_task) = fake_sam().await;
        let started = tokio::time::Instant::now();
        let (elapsed_tx, elapsed_rx) = oneshot::channel();
        let elapsed_tx = Arc::new(Mutex::new(Some(elapsed_tx)));
        let handler: ClientConnectionHandler = Arc::new(move |stream, connector| {
            let elapsed_tx = Arc::clone(&elapsed_tx);
            Box::pin(async move {
                let _ = connector.connect().await;
                let elapsed = started.elapsed();
                let _ = elapsed_tx.lock().take().map(|sender| sender.send(elapsed));
                drop(stream);
            })
        });
        let mut runtime_config = config(sam_port, handler, 1);
        runtime_config.connect_delay = Some(Duration::from_millis(60));
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_client_listener(runtime_config, cancel_rx, ready_tx));
        let address = ready_rx.await.unwrap().unwrap();
        drop(TcpStream::connect(address).await.unwrap());
        let elapsed = tokio::time::timeout(Duration::from_secs(1), elapsed_rx)
            .await
            .unwrap()
            .unwrap();
        assert!(elapsed >= Duration::from_millis(45), "elapsed={elapsed:?}");
        cancel_tx.send(true).unwrap();
        assert!(runtime.await.unwrap().is_ok());
        sam_task.abort();
    }

    #[tokio::test]
    async fn close_on_idle_recreates_the_session_for_a_new_generation() {
        let (sam_port, sam_task, _hellos, session_creates) = counting_fake_sam().await;
        let handler: ClientConnectionHandler = Arc::new(|stream, connector| {
            Box::pin(async move {
                let _ = connector.connect().await;
                drop(stream);
            })
        });
        let mut runtime_config = config(sam_port, handler, 1);
        runtime_config.close_on_idle = true;
        runtime_config.close_idle_time = Duration::from_millis(30);
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_client_listener(runtime_config, cancel_rx, ready_tx));
        let address = ready_rx.await.unwrap().unwrap();
        assert_eq!(session_creates.load(Ordering::Acquire), 1);
        drop(TcpStream::connect(address).await.unwrap());
        tokio::time::sleep(Duration::from_millis(90)).await;
        assert_eq!(session_creates.load(Ordering::Acquire), 1);
        drop(TcpStream::connect(address).await.unwrap());
        tokio::time::timeout(Duration::from_secs(1), async {
            while session_creates.load(Ordering::Acquire) < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        cancel_tx.send(true).unwrap();
        assert!(runtime.await.unwrap().is_ok());
        sam_task.abort();
    }

    #[tokio::test]
    async fn concurrent_first_connections_create_one_session() {
        let (sam_port, sam_task, _hellos, session_creates) = counting_fake_sam().await;
        let handler: ClientConnectionHandler = Arc::new(|stream, connector| {
            Box::pin(async move {
                let _ = connector.connect().await;
                drop(stream);
            })
        });
        let mut runtime_config = config(sam_port, handler, 2);
        runtime_config.delay_open = true;
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_client_listener(runtime_config, cancel_rx, ready_tx));
        let address = ready_rx.await.unwrap().unwrap();
        let _first = TcpStream::connect(address).await.unwrap();
        let _second = TcpStream::connect(address).await.unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            while session_creates.load(Ordering::Acquire) < 1 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        tokio::time::sleep(Duration::from_millis(25)).await;
        assert_eq!(session_creates.load(Ordering::Acquire), 1);
        cancel_tx.send(true).unwrap();
        assert!(runtime.await.unwrap().is_ok());
        sam_task.abort();
    }

    #[tokio::test]
    async fn shared_listeners_retain_one_session_until_last_listener_stops() {
        let (sam_port, sam_task, _hellos, session_creates) = counting_fake_sam().await;
        let registry = Arc::new(SharedClientSessionRegistry::new());
        let handler: ClientConnectionHandler = Arc::new(|_, _| Box::pin(async {}));
        let (first_cancel, first_rx) = watch::channel(false);
        let (first_ready, first_ready_rx) = oneshot::channel();
        let first = tokio::spawn(run_client_listener_with_shared_session(
            config(sam_port, Arc::clone(&handler), 1),
            first_rx,
            first_ready,
            Some(Arc::clone(&registry)),
            true,
        ));
        first_ready_rx.await.unwrap().unwrap();

        let (second_cancel, second_rx) = watch::channel(false);
        let (second_ready, second_ready_rx) = oneshot::channel();
        let second = tokio::spawn(run_client_listener_with_shared_session(
            config(sam_port, handler, 1),
            second_rx,
            second_ready,
            Some(registry),
            true,
        ));
        second_ready_rx.await.unwrap().unwrap();
        assert_eq!(session_creates.load(Ordering::Acquire), 1);

        first_cancel.send(true).unwrap();
        assert!(first.await.unwrap().is_ok());
        assert!(!second.is_finished());
        second_cancel.send(true).unwrap();
        assert!(second.await.unwrap().is_ok());
        sam_task.abort();
    }

    #[tokio::test]
    async fn cancellation_before_first_connection_does_not_create_session() {
        let (sam_port, sam_task, hellos, session_creates) = counting_fake_sam().await;
        let handler: ClientConnectionHandler = Arc::new(|_, _| Box::pin(async {}));
        let mut runtime_config = config(sam_port, handler, 1);
        runtime_config.delay_open = true;
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_client_listener(runtime_config, cancel_rx, ready_tx));
        ready_rx.await.unwrap().unwrap();
        cancel_tx.send(true).unwrap();
        assert!(runtime.await.unwrap().is_ok());
        assert_eq!(hellos.load(Ordering::Acquire), 0);
        assert_eq!(session_creates.load(Ordering::Acquire), 0);
        sam_task.abort();
    }

    #[tokio::test]
    async fn cancellation_during_lazy_setup_returns_deterministic_setup_error() {
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let (sam_port, sam_task) = gated_fake_sam(Arc::clone(&started), Arc::clone(&release)).await;
        let (error_tx, error_rx) = oneshot::channel();
        let error_tx = Arc::new(Mutex::new(Some(error_tx)));
        let handler: ClientConnectionHandler = Arc::new(move |stream, connector| {
            let error_tx = Arc::clone(&error_tx);
            Box::pin(async move {
                let result = connector.connect().await;
                let _ = error_tx.lock().take().map(|sender| {
                    sender.send(matches!(
                        result,
                        Err(ClientListenerRuntimeError::SessionSetup)
                    ))
                });
                drop(stream);
            })
        });
        let mut runtime_config = config(sam_port, handler, 1);
        runtime_config.delay_open = true;
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_client_listener(runtime_config, cancel_rx, ready_tx));
        let address = ready_rx.await.unwrap().unwrap();
        let _stream = TcpStream::connect(address).await.unwrap();
        started.notified().await;
        cancel_tx.send(true).unwrap();
        release.notify_one();
        assert!(error_rx.await.unwrap());
        assert!(runtime.await.unwrap().is_ok());
        sam_task.abort();
    }

    #[tokio::test]
    async fn failed_lazy_setup_fails_the_generation() {
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (error_tx, error_rx) = oneshot::channel();
        let error_tx = Arc::new(Mutex::new(Some(error_tx)));
        let handler: ClientConnectionHandler = Arc::new(move |stream, connector| {
            let error_tx = Arc::clone(&error_tx);
            Box::pin(async move {
                let result = connector.connect().await;
                let _ = error_tx.lock().take().map(|sender| {
                    sender.send(matches!(
                        result,
                        Err(ClientListenerRuntimeError::SessionSetup)
                    ))
                });
                drop(stream);
            })
        });
        let mut runtime_config = config(1, handler, 1);
        runtime_config.delay_open = true;
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_client_listener(runtime_config, cancel_rx, ready_tx));
        let address = ready_rx.await.unwrap().unwrap();
        drop(TcpStream::connect(address).await.unwrap());
        assert!(error_rx.await.unwrap());
        assert_eq!(
            runtime.await.unwrap(),
            Err(ClientListenerRuntimeError::SessionSetup)
        );
        drop(cancel_tx);
    }

    #[tokio::test]
    async fn restarted_generation_does_not_reuse_the_prior_session() {
        let (sam_port, sam_task, _hellos, session_creates) = counting_fake_sam().await;
        for expected in [1, 2] {
            let handler: ClientConnectionHandler = Arc::new(|stream, connector| {
                Box::pin(async move {
                    let _ = connector.connect().await;
                    drop(stream);
                })
            });
            let mut runtime_config = config(sam_port, handler, 1);
            runtime_config.delay_open = true;
            let (cancel_tx, cancel_rx) = watch::channel(false);
            let (ready_tx, ready_rx) = oneshot::channel();
            let runtime = tokio::spawn(run_client_listener(runtime_config, cancel_rx, ready_tx));
            let address = ready_rx.await.unwrap().unwrap();
            assert_eq!(session_creates.load(Ordering::Acquire), expected - 1);
            drop(TcpStream::connect(address).await.unwrap());
            tokio::time::timeout(Duration::from_secs(1), async {
                while session_creates.load(Ordering::Acquire) < expected {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            cancel_tx.send(true).unwrap();
            assert!(runtime.await.unwrap().is_ok());
        }
        sam_task.abort();
    }
}
