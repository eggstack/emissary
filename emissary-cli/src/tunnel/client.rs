// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::config::{ClientTunnelConfig, ClientTunnelOptions};

use tokio::{net::TcpListener, task::JoinSet};
use yosemite::{style, Session, SessionOptions, StreamOptions};

use std::{future::Future, sync::Arc, time::Duration};

/// Errors returned by the reusable client runtime primitive.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ClientRuntimeError {
    /// Local listener or stream I/O failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Yosemite SAM operation failed.
    #[error("Yosemite error")]
    Yosemite(#[from] yosemite::Error),
    /// The runtime task panicked.
    #[error("client runtime task panicked")]
    Panicked,
}

/// Plain runtime configuration for one generic client tunnel.
///
/// This type deliberately contains no I2PControl or persistence concepts so
/// the startup manager and the administrative runtime adapter can share the
/// same data-plane primitive.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClientTunnelRuntimeConfig {
    /// Diagnostic nickname only; it is not used as a protocol identity.
    pub name: String,
    /// Local listener interface. `None` uses loopback.
    pub address: Option<String>,
    /// Local listener port.
    pub port: u16,
    /// Remote I2P destination.
    pub destination: String,
    /// Remote destination port. Zero preserves Yosemite's existing default.
    pub destination_port: Option<u16>,
    /// SAMv3 TCP endpoint port.
    pub sam_tcp_port: u16,
}

/// Run one cancellable generic client tunnel.
///
/// Readiness is reported only after the local listener and independent
/// Yosemite streaming session have both been established. Traffic failures
/// retain the startup manager's bounded retry behavior. Bind and session
/// setup failures are terminal for this instance so a caller can release its
/// named runtime reservation and report a truthful failure.
#[allow(dead_code)]
pub async fn run_single_client(
    config: ClientTunnelRuntimeConfig,
    mut cancellation: tokio::sync::watch::Receiver<bool>,
    ready: tokio::sync::oneshot::Sender<Result<(), String>>,
) -> std::result::Result<(), ClientRuntimeError> {
    let address = config.address.clone().unwrap_or_else(|| "127.0.0.1".to_string());
    let listener = match TcpListener::bind(format!("{address}:{}", config.port)).await {
        Ok(listener) => listener,
        Err(error) => {
            let _ = ready.send(Err("client tunnel listener bind failed".to_string()));
            return Err(error.into());
        }
    };

    let session = tokio::select! {
        _ = cancellation.changed() => {
            let _ = ready.send(Err("client tunnel start cancelled".to_string()));
            return Ok(());
        }
        result = Session::<style::Stream>::new(SessionOptions {
            publish: false,
            samv3_tcp_port: config.sam_tcp_port,
            nickname: format!("i2p-tunnel-{}", config.name),
            inbound_quantity: 4,
            outbound_quantity: 4,
            ..Default::default()
        }) => result,
    }?;

    let _ = ready.send(Ok(()));
    let mut session = session;

    loop {
        let (mut tcp_stream, _) = tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            result = listener.accept() => result?,
        };

        let mut i2p_stream = tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            result = session.connect_detached_with_options(
                &config.destination,
                StreamOptions {
                    dst_port: config.destination_port.unwrap_or(0),
                    ..Default::default()
                },
            ) => match result {
                Ok(stream) => stream,
                Err(error) => {
                    tracing::debug!(
                        target: LOG_TARGET,
                        name = %config.name,
                        ?error,
                        "client tunnel connection failed",
                    );
                    tokio::select! {
                        _ = cancellation.changed() => return Ok(()),
                        _ = tokio::time::sleep(RETRY_TIMEOUT) => continue,
                    }
                }
            },
        };

        let copy_result = tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            result = tokio::io::copy_bidirectional(&mut i2p_stream, &mut tcp_stream) => result,
        };

        if let Err(error) = copy_result {
            tracing::debug!(
                target: LOG_TARGET,
                name = %config.name,
                ?error,
                "client tunnel traffic path failed",
            );
        }

        tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            _ = tokio::time::sleep(RETRY_TIMEOUT) => {},
        }
    }
}

/// Logging target for the file.
const LOG_TARGET: &str = "emissary::client-tunnel";

/// Retry timeout.
const RETRY_TIMEOUT: Duration = Duration::from_secs(15);

/// Client tunnel manager.
pub struct ClientTunnelManager {
    /// Tunnel futures.
    futures: JoinSet<Arc<ClientTunnelConfig>>,

    /// SAMv3 server port of the router.
    sam_tcp_port: u16,

    /// Client tunnel configurations.
    tunnels: Vec<Arc<ClientTunnelConfig>>,

    /// Client tunnel options.
    client_tunnel_options: Option<ClientTunnelOptions>,
}

impl ClientTunnelManager {
    /// Create new [`ClientTunnelManager`].
    pub fn new(
        tunnels: Vec<ClientTunnelConfig>,
        client_tunnel_options: Option<ClientTunnelOptions>,
        sam_tcp_port: u16,
    ) -> Self {
        Self {
            client_tunnel_options,
            futures: JoinSet::new(),
            sam_tcp_port,
            tunnels: tunnels.into_iter().map(Arc::from).collect(),
        }
    }

    /// Run the event loop of a client tunnel.
    async fn tunnel_event_loop(
        future: impl Future<Output = yosemite::Result<yosemite::Stream>>,
        tunnel: &Arc<ClientTunnelConfig>,
    ) -> std::result::Result<(), ClientRuntimeError> {
        let listener = TcpListener::bind(format!(
            "{}:{}",
            tunnel.address.clone().unwrap_or(String::from("127.0.0.1")),
            tunnel.port
        ))
        .await?;

        let (mut tcp_stream, _) = listener.accept().await?;
        let mut i2p_stream = future.await?;

        tokio::io::copy_bidirectional(&mut i2p_stream, &mut tcp_stream).await?;

        Ok(())
    }

    /// Run the event loop of [`ClientTunnelManger`].
    ///
    /// If there are no client tunnels congigured, [`ClientTunnelManager`] exits immediately.
    pub async fn run(mut self) {
        if self.tunnels.is_empty() {
            return;
        }

        tracing::info!(
            target: LOG_TARGET,
            num_tunnels = ?self.tunnels.len(),
            "starting client tunnel manager",
        );

        let mut session = match Session::<style::Stream>::new(SessionOptions {
            publish: false,
            samv3_tcp_port: self.sam_tcp_port,
            nickname: "i2p-tunnel".to_string(),
            inbound_quantity: 4,
            outbound_quantity: 4,
            lease_set_enc_type: self.client_tunnel_options.and_then(|config| {
                config.i2cp.and_then(|config| config.lease_set_enc_type.clone())
            }),
            ..Default::default()
        })
        .await
        {
            Ok(session) => session,
            Err(error) => {
                tracing::error!(
                    target: LOG_TARGET,
                    ?error,
                    "failed to start client tunnel manager",
                );
                return;
            }
        };

        for tunnel in self.tunnels.iter().cloned() {
            let future = session.connect_detached_with_options(
                &tunnel.destination,
                StreamOptions {
                    dst_port: tunnel.destination_port.unwrap_or(0),
                    ..Default::default()
                },
            );

            self.futures.spawn(async move {
                match Self::tunnel_event_loop(future, &tunnel).await {
                    Ok(()) => tunnel,
                    Err(error) => {
                        tracing::debug!(
                            target: LOG_TARGET,
                            name = %tunnel.name,
                            ?error,
                            "client tunnel exited with error",
                        );

                        tokio::time::sleep(RETRY_TIMEOUT).await;
                        tunnel
                    }
                }
            });
        }

        while let Some(result) = self.futures.join_next().await {
            match result {
                Err(error) => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        ?error,
                        "client tunnel panicked, unable to restart",
                    );
                    debug_assert!(false);
                }
                Ok(tunnel) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        name = %tunnel.name,
                        "tunnel returned, restart event loop"
                    );

                    let future = session.connect_detached_with_options(
                        &tunnel.destination,
                        StreamOptions {
                            dst_port: tunnel.destination_port.unwrap_or(0),
                            ..Default::default()
                        },
                    );

                    self.futures.spawn(async move {
                        match Self::tunnel_event_loop(future, &tunnel).await {
                            Ok(()) => tunnel,
                            Err(error) => {
                                tracing::debug!(
                                    target: LOG_TARGET,
                                    name = %tunnel.name,
                                    ?error,
                                    "client tunnel exited with error",
                                );

                                tokio::time::sleep(RETRY_TIMEOUT).await;
                                tunnel
                            }
                        }
                    });
                }
            }
        }
    }
}
