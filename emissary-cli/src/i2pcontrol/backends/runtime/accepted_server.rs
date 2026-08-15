use std::{fmt, sync::Arc, time::Duration};

use futures::{future::BoxFuture, FutureExt};
use tokio::sync::{oneshot, watch};
use yosemite::{style, DestinationKind, Session, SessionOptions, Stream};

use crate::i2pcontrol::server_secret_store::StoredDestination;

use super::{
    admission::{AdmissionDecision, ServerAdmissionPolicy, ServerAdmissionState},
    task_group::BoundedTaskGroup,
};

const STOP_TIMEOUT: Duration = Duration::from_secs(5);

/// Immutable public identity obtained from the accepted I2P stream.
#[derive(Clone, PartialEq, Eq)]
pub struct TrustedPeerIdentity {
    destination: Arc<str>,
}

impl fmt::Debug for TrustedPeerIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrustedPeerIdentity")
            .field("destination", &"<redacted>")
            .finish()
    }
}

impl TrustedPeerIdentity {
    fn from_stream(stream: &Stream) -> Option<Self> {
        let destination = stream.remote_destination();
        if destination.is_empty()
            || destination.len() > 64 * 1024
            || destination.chars().any(char::is_control)
        {
            return None;
        }
        Some(Self {
            destination: Arc::from(destination),
        })
    }

    /// Return the public destination reported by SAM for this connection.
    pub fn destination(&self) -> &str {
        &self.destination
    }

    #[cfg(test)]
    pub(crate) fn for_test(destination: &str) -> Self {
        Self {
            destination: Arc::from(destination),
        }
    }
}

/// One accepted I2P stream and the public identity authenticated by SAM.
pub struct AcceptedServerConnection {
    pub stream: Stream,
    pub peer: TrustedPeerIdentity,
}

impl fmt::Debug for AcceptedServerConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AcceptedServerConnection")
            .field("peer", &self.peer)
            .finish_non_exhaustive()
    }
}

/// A callback that decides protocol handling and any later local connection.
pub type AcceptedServerHandler =
    Arc<dyn Fn(AcceptedServerConnection) -> BoxFuture<'static, ()> + Send + Sync>;

/// Configuration for an application-visible accepted-stream server.
pub struct AcceptedServerRuntimeConfig {
    pub name: String,
    pub sam_tcp_port: u16,
    pub destination: StoredDestination,
    pub admission: ServerAdmissionPolicy,
    pub handler: AcceptedServerHandler,
}

impl fmt::Debug for AcceptedServerRuntimeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AcceptedServerRuntimeConfig")
            .field("name", &self.name)
            .field("sam_tcp_port", &self.sam_tcp_port)
            .field("destination", &self.destination)
            .field("admission", &self.admission)
            .finish_non_exhaustive()
    }
}

/// Errors from the accepted-stream server lifecycle.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum AcceptedServerRuntimeError {
    #[error("accepted server session setup failed")]
    SessionSetup,
    #[error("accepted server stream failed")]
    Accept,
    #[error("accepted server runtime panicked")]
    Panicked,
}

/// Own one persistent Yosemite session and expose accepted streams to a handler.
pub async fn run_accepted_server(
    config: AcceptedServerRuntimeConfig,
    mut cancellation: watch::Receiver<bool>,
    ready: oneshot::Sender<Result<String, AcceptedServerRuntimeError>>,
) -> Result<(), AcceptedServerRuntimeError> {
    let mut session = tokio::select! {
        _ = cancellation.changed() => {
            let _ = ready.send(Err(AcceptedServerRuntimeError::SessionSetup));
            return Ok(());
        }
        result = Session::<style::Stream>::new(SessionOptions {
            samv3_tcp_port: config.sam_tcp_port,
            nickname: config.name.clone(),
            publish: true,
            destination: DestinationKind::Persistent {
                private_key: config.destination.as_str().to_owned(),
            },
            ..Default::default()
        }) => match result {
            Ok(session) => session,
            Err(_) => {
                let _ = ready.send(Err(AcceptedServerRuntimeError::SessionSetup));
                return Err(AcceptedServerRuntimeError::SessionSetup);
            }
        },
    };

    let public_destination = session.destination().to_owned();
    let _ = ready.send(Ok(public_destination));
    let max_connections = config.admission.max_concurrent_connections();
    let admission = ServerAdmissionState::new(config.admission);
    let handler = config.handler;
    let mut tasks = BoundedTaskGroup::new(max_connections);

    loop {
        tokio::select! {
            _ = cancellation.changed() => break,
            result = session.accept() => {
                let stream = match result {
                    Ok(stream) => stream,
                    Err(_) => return Err(AcceptedServerRuntimeError::Accept),
                };
                let Some(peer) = TrustedPeerIdentity::from_stream(&stream) else {
                    continue;
                };
                let AdmissionDecision::Allowed(lease) = admission.try_acquire(&peer) else {
                    continue;
                };
                let handler = Arc::clone(&handler);
                let _ = tasks.try_spawn(async move {
                    let _lease = lease;
                    let _ = std::panic::AssertUnwindSafe((handler)(AcceptedServerConnection {
                        stream,
                        peer,
                    }))
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
    use std::sync::atomic::{AtomicBool, Ordering};

    use emissary_core::crypto::base64_encode;
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
                            "HELLO REPLY RESULT=OK VERSION=3.3\n".to_owned()
                        } else if line.starts_with("SESSION CREATE") {
                            "SESSION STATUS RESULT=OK DESTINATION=server-destination\n".to_owned()
                        } else if line.starts_with("STREAM ACCEPT") {
                            "STREAM STATUS RESULT=OK\npeer-destination\n".to_owned()
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
    async fn accepted_peer_identity_reaches_handler_before_local_target() {
        let (sam_port, sam_task) = fake_sam().await;
        let observed = Arc::new(AtomicBool::new(false));
        let target_connected = Arc::new(AtomicBool::new(false));
        let observed_handler = Arc::clone(&observed);
        let target_for_handler = Arc::clone(&target_connected);
        let handler: AcceptedServerHandler = Arc::new(move |connection| {
            let observed = Arc::clone(&observed_handler);
            let target_connected = Arc::clone(&target_for_handler);
            Box::pin(async move {
                assert_eq!(connection.peer.destination(), "peer-destination");
                observed.store(true, Ordering::Release);
                // A real HTTP/IRC handler would inspect bytes here. It must
                // connect a local target only after that decision.
                drop(connection);
                assert!(!target_connected.load(Ordering::Acquire));
            })
        });
        let config = AcceptedServerRuntimeConfig {
            name: "accepted-server".to_owned(),
            sam_tcp_port: sam_port,
            destination: StoredDestination::from_private(base64_encode([7u8; 128])),
            admission: ServerAdmissionPolicy::new(1, 0, 0, 0, 0, 0, 0).unwrap(),
            handler,
        };
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_accepted_server(config, cancel_rx, ready_tx));

        assert_eq!(ready_rx.await.unwrap().unwrap(), "server-destination");
        tokio::time::timeout(Duration::from_secs(1), async {
            while !observed.load(Ordering::Acquire) {
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
    async fn session_setup_failure_is_sanitized() {
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let result = run_accepted_server(
            AcceptedServerRuntimeConfig {
                name: "failure".to_owned(),
                sam_tcp_port: 1,
                destination: StoredDestination::from_private("private".to_owned()),
                admission: ServerAdmissionPolicy::new(1, 0, 0, 0, 0, 0, 0).unwrap(),
                handler: Arc::new(|_| Box::pin(async {})),
            },
            cancel_rx,
            ready_tx,
        )
        .await;
        assert_eq!(result, Err(AcceptedServerRuntimeError::SessionSetup));
        assert_eq!(
            ready_rx.await.unwrap(),
            Err(AcceptedServerRuntimeError::SessionSetup)
        );
        assert!(!format!("{result:?}").contains("private"));
    }
}
