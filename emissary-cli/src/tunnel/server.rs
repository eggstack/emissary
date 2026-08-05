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

#![allow(dead_code)]

use crate::config::{I2cpOptions, ServerTunnelConfig};

use yosemite::{style, DestinationKind, RouterApi, Session, SessionOptions};

use std::{path::PathBuf, sync::Arc, time::Duration};

/// Errors returned by the reusable single-server runtime primitive.
///
/// The error intentionally contains no SAM/Yosemite detail. A persistent
/// destination is part of the session setup input and must never be copied
/// into an error or diagnostic value.
#[derive(Debug, thiserror::Error)]
pub enum ServerRuntimeError {
    /// The SAM session could not be created.
    #[error("server tunnel session setup failed")]
    SessionSetup,
    /// The runtime task panicked.
    #[error("server tunnel runtime task panicked")]
    Panicked,
}

/// Passive callback used only to publish the actual destination returned by
/// an existing server session into the composed startup inventory.
pub type DestinationObserver = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Plain runtime configuration for one server tunnel.
///
/// The destination is private session material. This type deliberately has no
/// `Debug` implementation so accidental diagnostics cannot print it.
pub struct ServerTunnelRuntimeConfig {
    /// Diagnostic session nickname.
    pub name: String,
    /// Local TCP port receiving forwarded I2P streams.
    pub port: u16,
    /// Persistent destination private key material.
    pub destination: String,
    /// SAMv3 TCP port.
    pub sam_tcp_port: u16,
    /// Optional I2CP lease-set encryption type.
    pub lease_set_enc_type: Option<String>,
}

/// Generate one persistent destination through the existing router SAM API.
///
/// This is a purpose-specific data-plane helper. It does not know about
/// I2PControl, tunnel definitions, or filesystem paths.
pub async fn generate_persistent_destination(
    sam_tcp_port: u16,
) -> Result<String, ServerRuntimeError> {
    let router_api = RouterApi::new(sam_tcp_port);
    for attempt in 0..DESTINATION_CREATION_RETRY_COUNT {
        match router_api.generate_destination().await {
            Ok((_, private_key)) => return Ok(private_key),
            Err(_) if attempt + 1 < DESTINATION_CREATION_RETRY_COUNT => {
                tokio::time::sleep(DESTINATION_CREATION_BACKOFF).await;
            }
            Err(_) => break,
        }
    }
    Err(ServerRuntimeError::SessionSetup)
}

/// Run one cancellable generic server tunnel.
///
/// Readiness is reported only after the persistent session has published its
/// actual public destination and `STREAM FORWARD` has succeeded. Forward
/// failures retain the startup manager's bounded retry behavior. Cancellation
/// is observed during session setup, forward retry, and the idle lifetime.
pub async fn run_single_server(
    config: ServerTunnelRuntimeConfig,
    mut cancellation: tokio::sync::watch::Receiver<bool>,
    ready: tokio::sync::oneshot::Sender<Result<(), String>>,
    destination_observer: Option<DestinationObserver>,
) -> Result<(), ServerRuntimeError> {
    let mut session = tokio::select! {
        _ = cancellation.changed() => {
            let _ = ready.send(Err("server tunnel start cancelled".to_string()));
            return Ok(());
        }
        result = Session::<style::Stream>::new(SessionOptions {
            samv3_tcp_port: config.sam_tcp_port,
            nickname: config.name.clone(),
            silent_forward: true,
            destination: DestinationKind::Persistent {
                private_key: config.destination.clone(),
            },
            lease_set_enc_type: config.lease_set_enc_type,
            ..Default::default()
        }) => match result {
            Ok(session) => session,
            Err(_) => {
                let _ = ready.send(Err("server tunnel session setup failed".to_string()));
                return Err(ServerRuntimeError::SessionSetup);
            }
        },
    };

    if let Some(observer) = destination_observer.as_ref() {
        observer(&config.name, session.destination());
    }

    loop {
        let forward = tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            result = session.forward(config.port) => result,
        };
        if forward.is_ok() {
            break;
        }

        tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            _ = tokio::time::sleep(STREAM_FORWARD_BACKOFF) => {},
        }
    }

    let _ = ready.send(Ok(()));
    loop {
        tokio::select! {
            _ = cancellation.changed() => return Ok(()),
            _ = tokio::time::sleep(Duration::from_secs(10)) => {},
        }
    }
}

/// Logging target for the file.
const LOG_TARGET: &str = "emissary::server-tunnel";

/// Number of destination generation retries.
const DESTINATION_CREATION_RETRY_COUNT: usize = 3usize;

/// Destination generation failure backoff.
const DESTINATION_CREATION_BACKOFF: Duration = Duration::from_secs(10);

/// Backoff for `STREAM FORWARD` failure.
const STREAM_FORWARD_BACKOFF: Duration = Duration::from_secs(10);

/// Server tunnel configuration
pub struct TunnelConfig {
    /// Base64 destination.
    destination: String,

    /// I2CP options.
    i2cp: Option<I2cpOptions>,

    /// Name of the tunnel.
    name: String,

    /// Server port.
    port: u16,

    /// SAMv3 TCP port.
    sam_tcp_port: u16,
}

/// Server tunnel manager.
pub struct ServerTunnelManager {
    /// Server tunnels.
    tunnels: Vec<Arc<TunnelConfig>>,

    /// Optional passive destination publication callback.
    destination_observer: Option<DestinationObserver>,
}

impl ServerTunnelManager {
    /// Create new [`ServerTunnelManager`].
    pub async fn new(
        configs: Vec<ServerTunnelConfig>,
        sam_tcp_port: u16,
        base_path: PathBuf,
        destination_observer: Option<DestinationObserver>,
    ) -> Self {
        let mut tunnels = Vec::<Arc<TunnelConfig>>::new();
        let mut router_api = RouterApi::new(sam_tcp_port);

        for ServerTunnelConfig {
            name,
            port,
            destination_path,
            i2cp,
            ..
        } in configs
        {
            match Self::load_or_create_destination(
                &mut router_api,
                base_path.join(&destination_path),
            )
            .await
            {
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        %name,
                        %destination_path,
                        "failed to load or create destination for server tunnel",
                    );
                    continue;
                }
                Some(destination) => {
                    tunnels.push(Arc::from(TunnelConfig {
                        destination,
                        name,
                        port,
                        sam_tcp_port,
                        i2cp: i2cp.clone(),
                    }));
                }
            }
        }

        Self {
            tunnels,
            destination_observer,
        }
    }

    /// Attempt to load destination from `path` and if it does't exist, call router over SAMv3 to
    /// create new persistent destination.
    ///
    /// Destination generation is attempted three times before bailing out.
    async fn load_or_create_destination(
        router_api: &mut RouterApi,
        path: PathBuf,
    ) -> Option<String> {
        if let Some(destination) = tokio::fs::read(&path).await.ok().and_then(|contents| {
            std::str::from_utf8(&contents).ok().map(|destination| destination.to_string())
        }) {
            return Some(destination);
        };

        tracing::debug!(
            target: LOG_TARGET,
            ?path,
            "destination not found from disk, create new destination",
        );

        for _ in 0..DESTINATION_CREATION_RETRY_COUNT {
            match router_api.generate_destination().await {
                Ok((_, private_key)) => {
                    if let Err(error) = tokio::fs::write(&path, private_key.as_bytes()).await {
                        tracing::warn!(
                            target: LOG_TARGET,
                            ?path,
                            ?error,
                            "failed to write destination to disk",
                        );
                    }

                    return Some(private_key);
                }
                Err(error) => {
                    tracing::debug!(
                        target: LOG_TARGET,
                        ?path,
                        ?error,
                        "failed to generate destination",
                    );
                    tokio::time::sleep(DESTINATION_CREATION_BACKOFF).await;
                }
            }
        }

        tracing::warn!(
            target: LOG_TARGET,
            ?path,
            retry_count = ?DESTINATION_CREATION_RETRY_COUNT,
            "failed to generate destination after multiple retries",
        );

        None
    }

    /// Run the event loop of server tunnel.
    async fn server_event_loop(
        config: Arc<TunnelConfig>,
        destination_observer: Option<DestinationObserver>,
    ) {
        tracing::info!(
            target: LOG_TARGET,
            name = %config.name,
            port = %config.port,
            "starting server tunnel",
        );

        let (_, cancellation) = tokio::sync::watch::channel(false);
        let (ready, _ready_result) = tokio::sync::oneshot::channel();
        let result = run_single_server(
            ServerTunnelRuntimeConfig {
                name: config.name.clone(),
                port: config.port,
                destination: config.destination.clone(),
                sam_tcp_port: config.sam_tcp_port,
                lease_set_enc_type: config
                    .i2cp
                    .as_ref()
                    .and_then(|i2cp| i2cp.lease_set_enc_type.clone()),
            },
            cancellation,
            ready,
            destination_observer,
        )
        .await;
        if result.is_err() {
            tracing::error!(
                target: LOG_TARGET,
                name = %config.name,
                "failed to start server tunnel runtime",
            );
        }
    }

    /// Run the event loop of [`ServerTunnelManager`].
    pub async fn run(self) {
        if self.tunnels.is_empty() {
            return;
        }

        for tunnel in self.tunnels {
            tokio::spawn(Self::server_event_loop(
                Arc::clone(&tunnel),
                self.destination_observer.clone(),
            ));
        }
    }
}
