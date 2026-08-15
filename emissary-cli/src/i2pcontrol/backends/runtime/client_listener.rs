use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use futures::{future::BoxFuture, FutureExt};
use parking_lot::Mutex;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{oneshot, watch},
};
use yosemite::{style, Session, SessionOptions, StreamOptions};

use super::task_group::BoundedTaskGroup;

const STOP_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_MAX_CONNECTIONS: usize = 128;

/// A callback for one local client connection.
pub type ClientConnectionHandler =
    Arc<dyn Fn(TcpStream, ClientStreamConnector) -> BoxFuture<'static, ()> + Send + Sync>;

/// The narrow session capability exposed to a client protocol handler.
#[derive(Clone)]
pub struct ClientStreamConnector {
    session: Arc<Mutex<Session<style::Stream>>>,
    destination: Arc<str>,
    destination_port: u16,
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
    pub async fn connect(&self) -> Result<yosemite::Stream, ClientListenerRuntimeError> {
        self.connect_to(self.destination.as_ref(), self.destination_port)
            .await
    }

    /// Open one outbound stream to a request-selected I2P destination without
    /// holding the session mutex across I/O.
    pub async fn connect_to(
        &self,
        destination: &str,
        destination_port: u16,
    ) -> Result<yosemite::Stream, ClientListenerRuntimeError> {
        let future = {
            let mut session = self.session.lock();
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
    mut cancellation: watch::Receiver<bool>,
    ready: oneshot::Sender<Result<SocketAddr, ClientListenerRuntimeError>>,
) -> Result<(), ClientListenerRuntimeError> {
    let session = tokio::select! {
        _ = cancellation.changed() => {
            let _ = ready.send(Err(ClientListenerRuntimeError::SessionSetup));
            return Ok(());
        }
        result = Session::<style::Stream>::new(SessionOptions {
            samv3_tcp_port: config.sam_tcp_port,
            nickname: config.name.clone(),
            publish: false,
            ..Default::default()
        }) => match result {
            Ok(session) => session,
            Err(_) => {
                let _ = ready.send(Err(ClientListenerRuntimeError::SessionSetup));
                return Err(ClientListenerRuntimeError::SessionSetup);
            }
        },
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
    let connector = ClientStreamConnector {
        session: Arc::new(Mutex::new(session)),
        destination: Arc::from(config.destination),
        destination_port: config.destination_port,
    };
    let max_connections = config.max_connections.clamp(1, DEFAULT_MAX_CONNECTIONS);
    let handler = config.handler;
    let _ = ready.send(Ok(local_address));
    let mut tasks = BoundedTaskGroup::new(max_connections);

    loop {
        tokio::select! {
            _ = cancellation.changed() => break,
            result = listener.accept() => {
                let Ok((stream, _)) = result else { break };
                let handler = Arc::clone(&handler);
                let connector = connector.clone();
                let _ = tasks.try_spawn(async move {
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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
}
