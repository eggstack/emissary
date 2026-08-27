use std::{fmt, sync::Arc, time::Duration};

use futures::{future::BoxFuture, FutureExt};
use tokio::sync::{oneshot, watch};
use yosemite::{style, DestinationKind, Session, SessionOptions, Stream};

use crate::i2pcontrol::server_secret_store::StoredDestination;

pub(super) use super::peer_identity_impl::TrustedPeerIdentity;

use super::{
    admission::{AdmissionDecision, ServerAdmissionPolicy, ServerAdmissionState},
    task_group::BoundedTaskGroup,
};

const STOP_TIMEOUT: Duration = Duration::from_secs(5);
pub const MAX_TRUSTED_PEER_DESTINATION_TEXT: usize = 524;

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
    /// Optional I2CP lease-set encryption type for the accepted-stream session.
    ///
    /// `None` keeps the Yosemite default. M081 reserves this field for the
    /// generic `server` backend's validated `leaseSetEncType`; other accepted
    /// server families must explicitly set `None` so they do not silently
    /// gain capabilities their own option contracts do not document.
    pub lease_set_enc_type: Option<String>,
    /// Fully translated common session settings, when supplied by a backend.
    pub session_options: Option<SessionOptions>,
    pub handler: AcceptedServerHandler,
}

impl fmt::Debug for AcceptedServerRuntimeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AcceptedServerRuntimeConfig")
            .field("name", &self.name)
            .field("sam_tcp_port", &self.sam_tcp_port)
            .field("destination", &self.destination)
            .field("admission", &self.admission)
            .field("lease_set_enc_type", &self.lease_set_enc_type)
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
        result = Session::<style::Stream>::new(config.session_options.clone().unwrap_or_else(|| SessionOptions {
            samv3_tcp_port: config.sam_tcp_port,
            nickname: config.name.clone(),
            publish: true,
            destination: DestinationKind::Persistent {
                private_key: config.destination.as_str().to_owned(),
            },
            lease_set_enc_type: config.lease_set_enc_type.clone(),
            ..Default::default()
        })) => match result {
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

    use super::super::peer_identity::test_fixtures::NULL_CERT_DESTINATION_BYTES;

    async fn fake_sam(peer_destination: Arc<String>) -> (u16, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let peer_text = peer_destination.clone();
        let task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let peer_text = Arc::clone(&peer_text);
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
                            format!("STREAM STATUS RESULT=OK\n{}\n", peer_text.as_str())
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
        let peer_text: Arc<String> =
            Arc::new(base64_encode(NULL_CERT_DESTINATION_BYTES.as_slice()));
        let (sam_port, sam_task) = fake_sam(peer_text.clone()).await;
        let observed = Arc::new(AtomicBool::new(false));
        let target_connected = Arc::new(AtomicBool::new(false));
        let observed_handler = Arc::clone(&observed);
        let target_for_handler = Arc::clone(&target_connected);
        let expected_destination = Arc::clone(&peer_text);
        let handler: AcceptedServerHandler = Arc::new(move |connection| {
            let observed = Arc::clone(&observed_handler);
            let target_connected = Arc::clone(&target_for_handler);
            let expected = Arc::clone(&expected_destination);
            Box::pin(async move {
                assert_eq!(connection.peer.destination(), expected.as_str());
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
            lease_set_enc_type: None,
            session_options: None,
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
    async fn malformed_remote_destination_is_rejected_before_handler_invocation() {
        // "peer-destination" is not a structurally valid I2P Destination. The
        // accepted-server boundary must drop it without invoking the handler
        // or admitting a peer record.
        let peer_text: Arc<String> = Arc::new("peer-destination".to_owned());
        let (sam_port, sam_task) = fake_sam(peer_text).await;
        let invoked = Arc::new(AtomicBool::new(false));
        let invoked_handler = Arc::clone(&invoked);
        let handler: AcceptedServerHandler = Arc::new(move |_connection| {
            let invoked = Arc::clone(&invoked_handler);
            Box::pin(async move {
                invoked.store(true, Ordering::Release);
            })
        });
        let config = AcceptedServerRuntimeConfig {
            name: "rejected".to_owned(),
            sam_tcp_port: sam_port,
            destination: StoredDestination::from_private(base64_encode([7u8; 128])),
            admission: ServerAdmissionPolicy::new(1, 0, 0, 0, 0, 0, 0).unwrap(),
            lease_set_enc_type: None,
            session_options: None,
            handler,
        };
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (ready_tx, ready_rx) = oneshot::channel();
        let runtime = tokio::spawn(run_accepted_server(config, cancel_rx, ready_tx));
        assert_eq!(ready_rx.await.unwrap().unwrap(), "server-destination");

        // Give the fake SAM a moment to deliver the malformed destination.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!invoked.load(Ordering::Acquire));

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
                lease_set_enc_type: None,
                session_options: None,
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
