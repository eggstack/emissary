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

use crate::{
    crypto::{base32_decode, base32_encode, base64_encode, sha256::Sha256, SigningPrivateKey},
    destination::{DeliveryStyle, Destination, DestinationEvent, LeaseSetStatus},
    error::QueryError,
    events::EventHandle,
    i2cp::{I2cpPayload, I2cpPayloadBuilder},
    primitives::{Destination as Dest, DestinationId, LeaseSet2, LeaseSet2Header, Mapping},
    protocol::Protocol,
    runtime::{AddressBook, Instant as InstantT, JoinSet, Runtime},
    sam::{
        parser::{DestinationContext, SamCommand, SessionKind},
        pending::session::SamSessionContext,
        protocol::{
            datagram::DatagramManager,
            streaming::{Direction, ListenerKind, StreamManager, StreamManagerEvent},
        },
        socket::SamSocket,
        types::{
            PendingSession, PendingSessionState, PublicKeyContext, SamSessionCommand,
            SamSessionCommandRecycle, SamSessionKind,
        },
        SamObservationEvent, SamObservationHook, SubSessionCommand,
    },
};

use bytes::{BufMut, Bytes, BytesMut};
use futures::StreamExt;
use hashbrown::HashMap;
use thingbuf::mpsc::{Receiver, Sender};

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};

/// Logging target for the file.
const LOG_TARGET: &str = "emissary::sam::session";

/// Default idle time before tunnel-quantity decrease (20 minutes, reference).
const IDLE_DEFAULT_MS: u64 = 1_200_000;

/// Minimum idle time before decrease can fire (5 minutes, reference).
const IDLE_MIN_MS: u64 = 300_000;

/// Default decreased inbound/outbound quantity (reference).
const IDLE_DEFAULT_QUANTITY: usize = 1;

/// Generation-local SAM session idle policy.
///
/// Neutral core vocabulary only. Extensible for future idle-close handling
/// using the same activity clock (close evaluated before decrease).
#[derive(Debug, Clone, Copy)]
struct IdlePolicy {
    /// Decrease enabled via standard `i2cp.reduceOnIdle`.
    enabled: bool,
    /// Idle duration before decrease (clamped to minimum).
    idle_time: Duration,
    /// Decreased inbound/outbound quantity (coerced to at least 1, bounded).
    target_quantity: usize,
}

impl IdlePolicy {
    /// Parse standard `i2cp.reduce*` options fail-safe.
    ///
    /// Malformed external input disables the policy rather than
    /// disrupting the session. Reference-compatible rules:
    /// - enabled only when `reduceOnIdle` equals `true` (case-insensitive);
    /// - default idle 1200000 ms, minimum 300000 ms;
    /// - default quantity 1, values below 1 coerce to 1 (reference),
    ///   values above the live-quantity bound clamp to that bound.
    fn parse(options: &HashMap<String, String>) -> Self {
        let enabled = options
            .get("i2cp.reduceOnIdle")
            .is_some_and(|value| value.eq_ignore_ascii_case("true"));

        if !enabled {
            return Self {
                enabled: false,
                idle_time: Duration::from_millis(IDLE_DEFAULT_MS),
                target_quantity: IDLE_DEFAULT_QUANTITY,
            };
        }

        let idle_ms = options
            .get("i2cp.reduceIdleTime")
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(IDLE_DEFAULT_MS);
        let idle_ms = idle_ms.max(IDLE_MIN_MS);

        let target_quantity = options
            .get("i2cp.reduceQuantity")
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(IDLE_DEFAULT_QUANTITY)
            .clamp(1, crate::tunnel::MAX_DESIRED_TUNNEL_QUANTITY);

        Self {
            enabled: true,
            idle_time: Duration::from_millis(idle_ms),
            target_quantity,
        }
    }
}

/// Next generation-local idle owner identifier.
///
/// Each `SamSession` captures one value at activation. The timer is
/// actor-local (owned by the session future), so a stale generation can
/// never reach a replacement; the identifier exists to make generation
/// isolation explicit in tests and diagnostics.
static NEXT_IDLE_GENERATION: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(1);

fn next_idle_generation() -> u64 {
    NEXT_IDLE_GENERATION.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
}

/// Create the initial actor-local idle timer when the policy is enabled.
///
/// Returns `None` when disabled so no timer/work exists.
fn enabled_idle_timer<R: Runtime>(config: &IdlePolicy) -> Option<R::Timer> {
    config.enabled.then(|| R::timer(config.idle_time))
}

/// Active SAMv3 session.
pub struct SamSession<R: Runtime> {
    /// Address book.
    address_book: Option<Arc<dyn AddressBook>>,

    /// I2P datagram manager.
    datagram_manager: DatagramManager<R>,

    /// [`Dest`] of the session.
    ///
    /// Used to create new lease sets.
    dest: Dest,

    /// [`Destination`] of the session.
    destination: Destination<R>,

    /// Event handle.
    #[allow(unused)]
    event_handle: EventHandle<R>,

    /// Pending host lookups
    lookup_futures: R::JoinSet<(String, Option<String>)>,

    /// Session options.
    options: HashMap<String, String>,

    /// Pending host lookups.
    ///
    /// Pending `NAMING LOOKUP` queries for `.b32.i2p` addresses are stored here
    /// while the corresponding lease set is being queried.
    pending_host_lookups: HashMap<DestinationId, String>,

    /// Pending outbound sessions.
    ///
    /// `STREAM CONNECT` is marked pending if there is no active lease set for the remote
    /// destination. The stream is moved from pending to active/rejected, based on the lease set
    /// query result. The stream is also set into pending state even if a lease set is found,
    /// for the duration of the handshake process and if the remote doesn't answer any of the three
    /// `SYN` messages that are sent, the stream is destroyed.
    ///
    /// If a datagram is sent to a remote destination whose lease set is not available, the session
    /// is marked as pending until the lease set is found and all datagrams sent while the lease
    /// set is being queried are stored in the pending session state.
    pending_outbound: HashMap<DestinationId, PendingSession<R>>,

    /// Public key context for the session.
    public_key_context: PublicKeyContext,

    /// Receiver for commands sent for this session.
    ///
    /// Commands are dispatched by `SamServer` which ensures that [`SamCommand::CreateSession`]
    /// is never received by an active session.
    receiver: Receiver<SamSessionCommand<R>, SamSessionCommandRecycle>,

    /// Session ID.
    session_id: Arc<str>,

    /// Session kind.
    session_kind: SamSessionKind,

    /// Signing key.
    signing_key: SigningPrivateKey,

    /// Socket for reading session-related commands from the client.
    ///
    /// Set to `None` after the socket has been closed and th session is being destroyed.
    socket: Option<Box<SamSocket<R>>>,

    /// I2P virtual stream manager.
    stream_manager: StreamManager<R>,

    /// TX channel for sending session IDs of sub-sessions to `SamServer`
    ///
    /// `None` if `session_kind` is not `SessionKind::Primary`.
    sub_session_tx: Option<Sender<SubSessionCommand>>,

    /// Waker.
    waker: Option<Waker>,

    /// Optional passive lifecycle observer.
    observation_hook: Option<Arc<dyn SamObservationHook>>,

    /// Generation-local idle policy (standard `i2cp.reduce*`).
    idle_policy: IdlePolicy,

    /// Last qualifying I2P application-message activity (monotonic).
    idle_last_activity: R::Instant,

    /// Whether the live desired target is currently at the low quantity.
    idle_reduced: bool,

    /// Actor-local idle timer; `None` when disabled, reduced, or shut down.
    idle_timer: Option<R::Timer>,

    /// Generation identifier for this session's idle owner.
    idle_generation: u64,
}

impl<R: Runtime> SamSession<R> {
    /// Create new [`SamSession`].
    #[allow(dead_code)]
    pub fn new(context: SamSessionContext<R>) -> Self {
        Self::new_inner(context, None)
    }

    /// Create new [`SamSession`] with an optional passive observation hook.
    pub(crate) fn new_with_observation_hook(
        context: SamSessionContext<R>,
        observation_hook: Option<Arc<dyn SamObservationHook>>,
    ) -> Self {
        Self::new_inner(context, observation_hook)
    }

    fn new_inner(
        context: SamSessionContext<R>,
        observation_hook: Option<Arc<dyn SamObservationHook>>,
    ) -> Self {
        let SamSessionContext {
            address_book,
            datagram_tx,
            destination,
            event_handle,
            inbound,
            mut socket,
            netdb_handle,
            options,
            outbound,
            profile_storage,
            receiver,
            session_id,
            session_kind,
            sub_session_tx,
            tunnel_pool_handle,
        } = context;

        let (session_destination, dest, privkey, public_key_context, signing_key) = {
            let DestinationContext {
                destination,
                private_key,
                signing_key,
            } = destination;
            let destination_id = destination.id();

            // from specification:
            //
            // "The $privkey is the base 64 of the concatenation of the Destination followed by the
            // Private Key followed by the Signing Private Key, optionally followed by the Offline
            // Signature, which is 663 or more bytes in binary and 884 or more bytes in base 64,
            // depending on signature type. The binary format is specified in Private Key File."
            let privkey = {
                let mut out = BytesMut::with_capacity(destination.serialized_len() + 2 * 32);
                out.put_slice(&destination.serialize());
                out.put_slice((*private_key).as_ref());
                out.put_slice((*signing_key).as_ref());

                base64_encode(out)
            };

            // create public key context for the session.
            let public_key_context = PublicKeyContext::new::<R>(&options);

            // create leaseset for the destination and store it in `NetDb`
            let is_unpublished = options
                .get("i2cp.dontPublishLeaseSet")
                .map(|value| value.parse::<bool>().unwrap_or(false))
                .unwrap_or(false);

            let local_leaseset = Bytes::from(
                LeaseSet2 {
                    header: LeaseSet2Header {
                        destination: destination.clone(),
                        expires: Duration::from_secs(10 * 60).as_secs() as u32,
                        is_unpublished,
                        offline_signature: None,
                        published: R::time_since_epoch().as_secs() as u32,
                    },
                    public_keys: public_key_context.public_keys(),
                    leases: inbound.values().cloned().collect(),
                }
                .serialize(&signing_key),
            );

            // publish the new destination to the event system
            if is_unpublished {
                event_handle.client_destination_started(session_id.to_string());
            } else {
                event_handle.server_destination_started(
                    session_id.to_string(),
                    base32_encode(destination_id.to_vec()),
                );
            }

            let mut session_destination = Destination::new(
                destination_id.clone(),
                public_key_context.private_key(),
                public_key_context.public_keys(),
                local_leaseset.clone(),
                netdb_handle,
                tunnel_pool_handle,
                outbound.into_iter().collect(),
                inbound.into_values().collect(),
                is_unpublished,
                profile_storage,
            );
            // TODO: not needed anymore?
            session_destination.publish_lease_set(local_leaseset.clone());

            tracing::info!(
                target: LOG_TARGET,
                %session_id,
                %destination_id,
                "start active session",
            );

            (
                session_destination,
                destination,
                privkey,
                public_key_context,
                signing_key,
            )
        };

        socket.send_message(
            format!("SESSION STATUS RESULT=OK DESTINATION={privkey}\n").as_bytes().to_vec(),
        );

        let idle_policy = IdlePolicy::parse(&options);
        let idle_timer = enabled_idle_timer::<R>(&idle_policy);

        Self {
            address_book,
            datagram_manager: DatagramManager::new(
                dest.clone(),
                datagram_tx,
                options.clone(),
                *signing_key.clone(),
            ),
            dest: dest.clone(),
            destination: session_destination,
            public_key_context,
            event_handle,
            lookup_futures: R::join_set(),
            options,
            pending_host_lookups: HashMap::new(),
            pending_outbound: HashMap::new(),
            receiver,
            session_id,
            session_kind: match session_kind {
                SessionKind::Stream => SamSessionKind::Stream,
                SessionKind::Datagram => SamSessionKind::Datagram {
                    kind: SessionKind::Datagram,
                },
                SessionKind::Anonymous => SamSessionKind::Datagram {
                    kind: SessionKind::Anonymous,
                },
                SessionKind::Datagram2 => SamSessionKind::Datagram {
                    kind: SessionKind::Datagram2,
                },
                SessionKind::Primary => SamSessionKind::Primary {
                    sub_sessions: HashMap::new(),
                },
            },
            signing_key: *signing_key.clone(),
            socket: Some(socket),
            stream_manager: StreamManager::new(dest, *signing_key),
            sub_session_tx,
            waker: None,
            observation_hook,
            idle_last_activity: R::now(),
            idle_reduced: false,
            idle_timer,
            idle_generation: next_idle_generation(),
            idle_policy,
        }
    }

    fn observe_socket(&self, socket: &SamSocket<R>, socket_type: u8) {
        let Some(hook) = &self.observation_hook else {
            return;
        };
        let event = SamObservationEvent::SocketActivated {
            session_id: super::sanitized_text(&self.session_id, 256),
            socket_id: socket.observation_id(),
            socket_type,
            peer: super::sanitized_peer(socket.peer_addr()),
        };
        match hook.publish(event) {
            Ok(()) => {}
            Err(_) => {
                tracing::warn!(
                    target: LOG_TARGET,
                    session_id = %self.session_id,
                    "SAM observation hook rejected socket activation",
                );
            }
        }
    }

    fn remove_observed_socket(&mut self, socket_id: u64) {
        if let Some(hook) = &self.observation_hook {
            if hook
                .publish(SamObservationEvent::SocketRemoved {
                    session_id: super::sanitized_text(&self.session_id, 256),
                    socket_id,
                })
                .is_err()
            {
                tracing::warn!(
                    target: LOG_TARGET,
                    session_id = %self.session_id,
                    socket_id,
                    "SAM observation hook rejected socket removal",
                );
            }
        }
    }

    /// Record qualifying I2P application-message activity.
    ///
    /// Qualifying activity (reference boundary):
    /// - outbound streaming payload/protocol packets accepted for I2P
    ///   delivery (`destination.send_message` success);
    /// - inbound streaming payload/protocol packets successfully delivered
    ///   into the local streaming manager;
    /// - outbound datagrams accepted for I2P delivery;
    /// - inbound datagrams successfully delivered to the SAM/datagram
    ///   consumer;
    /// - activity from any member sharing this underlying session
    ///   generation (all subsession commands route to this same owner).
    ///
    /// Excluded: local handler count, idle TCP sockets, SAM PING/PONG,
    /// naming/address lookup, tunnel build/maintenance, NetDb, control
    /// RPC. Those paths must not call this method.
    ///
    /// When at the low target, the first qualifying activity requests
    /// restore to the base target. A failed restore keeps `idle_reduced`
    /// set so a later activity retries; it never falsely marks restored.
    /// When active, activity resets the monotonic idle age by recreating
    /// the single actor-local timer. No lock spans network I/O; the
    /// destination bridge is synchronous and bounded.
    fn note_qualifying_activity(&mut self) {
        if !self.idle_policy.enabled {
            return;
        }

        self.idle_last_activity = R::now();

        if self.idle_reduced {
            match self.destination.restore_tunnel_quantity_target() {
                Ok(()) => {
                    self.idle_reduced = false;
                    self.idle_timer = Some(R::timer(self.idle_policy.idle_time));
                }
                Err(error) => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        session_id = %self.session_id,
                        ?error,
                        "idle restore failed, remaining at low target until next activity",
                    );
                }
            }
        } else {
            self.idle_timer = Some(R::timer(self.idle_policy.idle_time));
        }
    }

    /// Poll the single actor-local idle timer and drive target decrease.
    ///
    /// At the deadline the desired inbound/outbound target becomes the
    /// configured low quantity via the live-quantity primitive. Only
    /// authoritative success marks the session at the low target. While
    /// there no further controls are enqueued. Failures leave
    /// `idle_reduced` unset without spinning; shutdown wins by dropping
    /// the timer. The state machine keeps one activity clock so a future
    /// close policy can evaluate close before decrease.
    fn poll_idle_timer(&mut self, cx: &mut Context<'_>) {
        if !self.idle_policy.enabled || self.idle_reduced {
            return;
        }

        let Some(timer) = self.idle_timer.as_mut() else {
            return;
        };

        match Pin::new(timer).poll(cx) {
            Poll::Pending => {}
            Poll::Ready(()) => {
                if self.idle_last_activity.elapsed() < self.idle_policy.idle_time {
                    let remaining = self.idle_policy.idle_time - self.idle_last_activity.elapsed();
                    self.idle_timer = Some(R::timer(remaining));
                    return;
                }

                let quantity = self.idle_policy.target_quantity;
                match self.destination.set_tunnel_quantity_target(quantity, quantity) {
                    Ok(()) => {
                        tracing::info!(
                            target: LOG_TARGET,
                            session_id = %self.session_id,
                            %quantity,
                            generation = %self.idle_generation,
                            "session idle threshold reached, using low tunnel target",
                        );
                        self.idle_reduced = true;
                        self.idle_timer = None;
                    }
                    Err(error) => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            session_id = %self.session_id,
                            ?error,
                            "idle target decrease failed, not marking at low target",
                        );
                        self.idle_timer = None;
                    }
                }
            }
        }
    }

    /// Cancel idle state on authoritative session teardown.
    ///
    /// Timer state is generation-local and never persisted; replacement
    /// generations start fresh active state.
    fn cancel_idle_state(&mut self) {
        self.idle_timer = None;
    }

    /// Create outbound stream for a remote destiantion who's lease set has been resolved.
    ///
    /// The stream is considered pending and it's acceptance contingent on the remote destination
    /// responding to us within a reasonable time frame.
    fn create_outbound_stream(
        &mut self,
        destination_id: DestinationId,
        socket: Box<SamSocket<R>>,
        options: HashMap<String, String>,
    ) {
        let handle = self.destination.routing_path_handle(destination_id.clone());
        let (stream_id, packet, delivery_style, src_port, dst_port) = self
            .stream_manager
            .create_stream(destination_id.clone(), handle, socket, options);

        tracing::trace!(
            target: LOG_TARGET,
            %destination_id,
            ?stream_id,
            ?src_port,
            ?dst_port,
            "create pending outbound stream",
        );

        // mark the stream as pending & waiting for session to be opened
        //
        // from now on `StreamManager` will drive forward the stream progress and will
        // emit an event when the stream opens/fails to open
        self.pending_outbound
            .entry(destination_id.clone())
            .or_insert(PendingSession::<R>::new())
            .streams
            .push(PendingSessionState::AwaitingSession { stream_id });

        let Some(message) = I2cpPayloadBuilder::<R>::new(&packet)
            .with_protocol(Protocol::Streaming)
            .with_source_port(src_port)
            .with_destination_port(dst_port)
            .build()
        else {
            tracing::error!(
                target: LOG_TARGET,
                session_id = ?self.session_id,
                "failed to create i2cp payload",
            );
            debug_assert!(false);
            return;
        };

        if let Err(error) = self.destination.send_message(delivery_style, message) {
            tracing::error!(
                target: LOG_TARGET,
                session_id = ?self.session_id,
                ?error,
                "failed to send message to remote peer",
            );
            debug_assert!(false);
        } else {
            // Outbound streaming SYN accepted for I2P delivery.
            self.note_qualifying_activity();
        }
    }

    /// Handle `STREAM CONNECT`.
    fn on_stream_connect(
        &mut self,
        mut socket: Box<SamSocket<R>>,
        destination_id: DestinationId,
        options: HashMap<String, String>,
        session_id: Arc<str>,
    ) {
        if !self.session_kind.supports_streams(&session_id) {
            tracing::warn!(
                target: LOG_TARGET,
                session_id = %self.session_id,
                stream_kind = ?self.session_kind,
                "session style doesn't support streams",
            );

            return drop(socket);
        };

        if destination_id == self.dest.id() {
            tracing::warn!(
                target: LOG_TARGET,
                "tried to open connection to self",
            );

            R::spawn(async move {
                let _ = socket
                    .send_message_blocking(b"STREAM STATUS RESULT=CANT_REACH_PEER\n".to_vec())
                    .await;
            });
            return;
        }

        self.observe_socket(&socket, 2);

        tracing::info!(
            target: LOG_TARGET,
            session_id = %self.session_id,
            destination_id = %destination_id,
            "connect to destination",
        );

        match self.destination.query_lease_set(&destination_id) {
            LeaseSetStatus::Found => {
                tracing::trace!(
                    target: LOG_TARGET,
                    session_id = ?self.session_id,
                    %destination_id,
                    "lease set found, create outbound stream",
                );

                self.create_outbound_stream(destination_id, socket, options);
            }
            status @ (LeaseSetStatus::NotFound | LeaseSetStatus::Pending) => {
                tracing::trace!(
                    target: LOG_TARGET,
                    session_id = %self.session_id,
                    %destination_id,
                    ?status,
                    "lease set query started or pending, mark outbound stream as pending",
                );

                self.pending_outbound
                    .entry(destination_id.clone())
                    .or_insert(PendingSession::<R>::new())
                    .streams
                    .push(PendingSessionState::AwaitingLeaseSet { socket, options });
            }
        }
    }

    /// Handle `STREAM ACCEPT` command.
    ///
    /// Register the socket as an active listener to [`StreamManager`].
    ///
    /// If the session wasn't configured to use streams, reject the accept request.
    fn on_stream_accept(
        &mut self,
        socket: Box<SamSocket<R>>,
        options: HashMap<String, String>,
        session_id: Arc<str>,
    ) {
        if !self.session_kind.supports_streams(&session_id) {
            tracing::warn!(
                target: LOG_TARGET,
                session_id = %self.session_id,
                stream_kind = ?self.session_kind,
                "session style doesn't support streams",
            );

            return drop(socket);
        };

        let socket_id = socket.observation_id();
        self.observe_socket(&socket, 3);
        if let Err(error) = self.stream_manager.register_listener(ListenerKind::Ephemeral {
            pending_routing_path_handle: self.destination.pending_routing_path_handle(),
            socket,
            silent: options
                .get("SILENT")
                .is_some_and(|value| value.parse::<bool>().unwrap_or(false)),
        }) {
            tracing::warn!(
                target: LOG_TARGET,
                ?error,
                session_id = %self.session_id,
                "failed to register ephemeral listener",
            );
            self.remove_observed_socket(socket_id);
        }
    }

    /// Handle `STREAM FORWARD` command.
    ///
    /// Register the socket as an active listener to [`StreamManager`].
    ///
    /// If the session wasn't configured to use streams, reject the forward request.
    fn on_stream_forward(
        &mut self,
        socket: Box<SamSocket<R>>,
        port: u16,
        options: HashMap<String, String>,
        session_id: Arc<str>,
    ) {
        if !self.session_kind.supports_streams(&session_id) {
            tracing::warn!(
                target: LOG_TARGET,
                session_id = %self.session_id,
                stream_kind = ?self.session_kind,
                "session style doesn't support streams",
            );

            return drop(socket);
        };

        let socket_id = socket.observation_id();
        self.observe_socket(&socket, 3);
        if let Err(error) = self.stream_manager.register_listener(ListenerKind::Persistent {
            pending_routing_path_handle: self.destination.pending_routing_path_handle(),
            socket,
            port,
            silent: options
                .get("SILENT")
                .is_some_and(|value| value.parse::<bool>().unwrap_or(false)),
        }) {
            tracing::warn!(
                target: LOG_TARGET,
                ?error,
                session_id = %self.session_id,
                "failed to register persistent listener",
            );
            self.remove_observed_socket(socket_id);
        }
    }

    /// Send datagram to destination.
    ///
    /// If the session wasn't configured to use streams, the datagram is dropped.
    fn on_send_datagram(
        &mut self,
        destination: Dest,
        datagram: Vec<u8>,
        session_id: Arc<str>,
        options: Option<Mapping>,
    ) {
        if !self.session_kind.supports_datagrams(&session_id) {
            tracing::warn!(
                target: LOG_TARGET,
                session_id = %self.session_id,
                stream_kind = ?self.session_kind,
                "session style doesn't support datagrams",
            );
            return;
        }

        tracing::info!(
            target: LOG_TARGET,
            session_id = %self.session_id,
            destination_id = %destination.id(),
            style = ?self.session_kind,
            "send datagram",
        );
        let destination_id = destination.id();
        let protocol = self.session_kind.as_protocol(&session_id);

        match self.destination.query_lease_set(&destination_id) {
            LeaseSetStatus::Found => {
                let datagram = match protocol {
                    Protocol::Anonymous => self.datagram_manager.make_anonymous(datagram),
                    Protocol::Datagram => self.datagram_manager.make_datagram(datagram),
                    Protocol::Datagram2 => self.datagram_manager.make_datagram2(
                        datagram,
                        &Sha256::new().update(&*destination).finalize(),
                        options,
                    ),
                    Protocol::Streaming => unreachable!(),
                };

                if let Some(message) =
                    I2cpPayloadBuilder::<R>::new(&datagram).with_protocol(protocol).build()
                {
                    if let Err(error) = self
                        .destination
                        .send_message(DeliveryStyle::Unspecified { destination_id }, message)
                    {
                        tracing::warn!(
                            target: LOG_TARGET,
                            session_id = %self.session_id,
                            destination_id = %destination.id(),
                            ?error,
                            "failed to send repliable datagram",
                        )
                    } else {
                        // Outbound datagram accepted for I2P delivery.
                        self.note_qualifying_activity();
                    }
                };
            }
            LeaseSetStatus::NotFound => {
                tracing::trace!(
                    target: LOG_TARGET,
                    session_id = %self.session_id,
                    %destination_id,
                    "lease set query started, mark outbound datagram as pending",
                );

                match self.pending_outbound.get_mut(&destination_id) {
                    Some(PendingSession { datagrams, .. }) => match datagrams {
                        None => {
                            *datagrams = Some((destination, vec![(protocol, datagram, options)]));
                        }
                        Some((_, datagrams)) => datagrams.push((protocol, datagram, options)),
                    },
                    None => {
                        self.pending_outbound.insert(
                            destination_id,
                            PendingSession {
                                streams: Vec::new(),
                                datagrams: Some((destination, vec![(protocol, datagram, options)])),
                            },
                        );
                    }
                }
            }
            LeaseSetStatus::Pending => {
                tracing::warn!(
                    target: LOG_TARGET,
                    session_id = %self.session_id,
                    %destination_id,
                    "received datagram while session was pending",
                );

                match self.pending_outbound.get_mut(&destination_id) {
                    Some(PendingSession { datagrams, .. }) => match datagrams {
                        None => {
                            *datagrams = Some((destination, vec![(protocol, datagram, options)]));
                        }
                        Some((_, datagrams)) => datagrams.push((protocol, datagram, options)),
                    },
                    None => {
                        self.pending_outbound.insert(
                            destination_id,
                            PendingSession {
                                streams: Vec::new(),
                                datagrams: Some((destination, vec![(protocol, datagram, options)])),
                            },
                        );
                    }
                }
            }
        }
    }

    /// Handle succeeded lease set query result.
    ///
    /// For each of the pending streams, create a new outbound stream which allocates context in
    /// [`StreamManger`] for it and creates a `SYN` packet which is sent sent in an NS message to
    /// remote destination.
    ///
    /// Same deal for datagrams: send all pending datagrams to remote destination in NS messages.
    ///
    /// All pending host lookups are also resolved with a success and the destination of the remote
    /// peer is sent via the active socket to client.
    fn on_lease_set_found(&mut self, destination_id: DestinationId) {
        tracing::trace!(
            target: LOG_TARGET,
            session_id = %self.session_id,
            %destination_id,
            "lease set found",
        );

        if let Some(PendingSession { streams, datagrams }) =
            self.pending_outbound.remove(&destination_id)
        {
            streams.into_iter().for_each(|state| match state {
                PendingSessionState::AwaitingLeaseSet { socket, options } => {
                    self.create_outbound_stream(destination_id.clone(), socket, options);
                }
                PendingSessionState::AwaitingSession { .. } => {
                    // new stream was opened but by the the time the initial `SYN` packet was sent,
                    // remote's lease set had expired and they had not sent us, a new lease set a
                    // lease set query was started and the lease set was found
                    //
                    // the new lease set can be ignored for `PendingSessionState::AwaitinSession`
                    // since the `SYN` packet was queued in `Destination` and
                    // was sent to remote destination when the lease set was
                    // received
                }
            });

            if let Some((destination, datagrams)) = datagrams {
                let mut delivered = false;
                datagrams.into_iter().for_each(|(protocol, datagram, options)| {
                    let datagram = match protocol {
                        Protocol::Anonymous => self.datagram_manager.make_anonymous(datagram),
                        Protocol::Datagram => self.datagram_manager.make_datagram(datagram),
                        Protocol::Datagram2 => self.datagram_manager.make_datagram2(
                            datagram,
                            &Sha256::new().update(&*destination).finalize(),
                            options,
                        ),
                        Protocol::Streaming => unreachable!(),
                    };

                    if let Some(message) =
                        I2cpPayloadBuilder::<R>::new(&datagram).with_protocol(protocol).build()
                    {
                        if let Err(error) = self.destination.send_message(
                            DeliveryStyle::Unspecified {
                                destination_id: destination_id.clone(),
                            },
                            message,
                        ) {
                            tracing::warn!(
                                target: LOG_TARGET,
                                session_id = %self.session_id,
                                destination_id = %destination.id(),
                                ?error,
                                "failed to send repliable datagram",
                            )
                        } else {
                            delivered = true;
                        }
                    };
                });
                // Flushed pending datagrams accepted for I2P delivery count
                // once as qualifying activity.
                if delivered {
                    self.note_qualifying_activity();
                }
            }
        } else {
            tracing::debug!(
                target: LOG_TARGET,
                session_id = ?self.session_id,
                %destination_id,
                "lease set query succeeded but no stream is interested in the lease set",
            );
        }

        if let Some(name) = self.pending_host_lookups.remove(&destination_id) {
            tracing::trace!(
                target: LOG_TARGET,
                session_id = ?self.session_id,
                %destination_id,
                ?name,
                "lease set query succeeded for pending host lookup",
            );

            if let Some(socket) = &mut self.socket {
                socket.send_message(
                    format!(
                        "NAMING REPLY RESULT=OK NAME={name} VALUE={}\n",
                        base64_encode(
                            self.destination
                                .lease_set(&destination_id)
                                .header
                                .destination
                                .serialized()
                        ),
                    )
                    .as_bytes()
                    .to_vec(),
                );

                if let Some(waker) = self.waker.take() {
                    waker.wake_by_ref();
                }
            }
        }
    }

    /// Handle lease set query error for `destination_id`.
    ///
    /// Lease set query can fail for either streams, datagrams or a host lookup, either one of them,
    /// some of them all or all of them at the same time, depending on what kind protocol is being
    /// used.
    ///
    /// Any pending datagrams for the unreachable destiantion are discarded, an error is sent to the
    /// user on each of the active stream and if there are pending host lookups, the client is
    /// notified of the error via the open socket
    fn on_lease_set_not_found(&mut self, destination_id: DestinationId, error: QueryError) {
        tracing::trace!(
            target: LOG_TARGET,
            session_id = %self.session_id,
            %destination_id,
            ?error,
            "lease set not found",
        );

        if let Some(PendingSession { streams, datagrams }) =
            self.pending_outbound.remove(&destination_id)
        {
            if let Some((_, datagrams)) = datagrams {
                tracing::debug!(
                    target: LOG_TARGET,
                    %destination_id,
                    num_datagrams = ?datagrams.len(),
                    "discarding pending datagrams, lease set not found",
                );
            }

            let sockets = streams
                .into_iter()
                .filter_map(|state| match state {
                    PendingSessionState::AwaitingLeaseSet { socket, .. } => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            session_id = ?self.session_id,
                            %destination_id,
                            ?error,
                            "unable to open stream, lease set not found",
                        );

                        self.remove_observed_socket(socket.observation_id());
                        Some(socket)
                    }
                    PendingSessionState::AwaitingSession { stream_id } => {
                        // new stream was opened but by the the time the initial `SYN` packet was
                        // sent, remote's lease set had expired and they had
                        // not sent us, a new lease set a lease
                        // set query was started but the lease set was not found in the netdb
                        //
                        // as the remote cannot be contacted, remove the pending stream from
                        // `StreamManager`
                        tracing::warn!(
                            target: LOG_TARGET,
                            session_id = ?self.session_id,
                            %destination_id,
                            ?stream_id,
                            "stream awaiting session but remote lease set not found",
                        );

                        self.stream_manager.remove_session(&destination_id);
                        None
                    }
                })
                .collect::<Vec<_>>();

            if !sockets.is_empty() {
                R::spawn(async move {
                    for mut socket in sockets {
                        let _ = socket
                            .send_message_blocking(b"STREAM STATUS RESULT=CANT_REACH_PEER".to_vec())
                            .await;
                    }
                });
            }
        } else {
            tracing::debug!(
                target: LOG_TARGET,
                session_id = ?self.session_id,
                %destination_id,
                ?error,
                "lease set query failure but no stream is interested in the lease set",
            );
        }

        if let Some(name) = self.pending_host_lookups.remove(&destination_id) {
            tracing::debug!(
                target: LOG_TARGET,
                session_id = ?self.session_id,
                %destination_id,
                ?name,
                ?error,
                "lease set query failed for pending host lookup",
            );

            if let Some(socket) = &mut self.socket {
                socket.send_message(
                    format!("NAMING REPLY RESULT=KEY_NOT_FOUND NAME={name}\n").as_bytes().to_vec(),
                );

                if let Some(waker) = self.waker.take() {
                    waker.wake_by_ref();
                }
            }
        }
    }

    /// Handle one or more inbound messages.
    fn on_inbound_message(&mut self, messages: Vec<Vec<u8>>) {
        let mut qualifying = false;
        messages
            .into_iter()
            .for_each(|message| match I2cpPayload::decompress::<R>(message) {
                Some(payload) => {
                    tracing::trace!(
                        target: LOG_TARGET,
                        session_id = %self.session_id,
                        src_port = ?payload.src_port,
                        dst_port = ?payload.dst_port,
                        protocol = ?payload.protocol,
                        "handle protocol payload",
                    );

                    match payload.protocol {
                        Protocol::Streaming => {
                            if let Err(error) = self.stream_manager.on_packet(payload) {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    session_id = ?self.session_id,
                                    ?error,
                                    "failed to handle streaming protocol packet",
                                );
                            } else {
                                // Inbound streaming packet delivered locally.
                                qualifying = true;
                            }
                        }
                        protocol => {
                            if let Err(error) = self.datagram_manager.on_datagram(payload) {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    session_id = ?self.session_id,
                                    ?protocol,
                                    ?error,
                                    "failed to handle datagram",
                                );
                            } else {
                                // Inbound datagram delivered to consumer.
                                qualifying = true;
                            }
                        }
                    }
                }
                None => tracing::warn!(
                    target: LOG_TARGET,
                    session_id = ?self.session_id,
                    "failed to decompress i2cp payload",
                ),
            });
        if qualifying {
            self.note_qualifying_activity();
        }
    }

    /// Handle `NAMING LOOKUP` query from the client.
    ///
    /// The query can either be for `ME`, meaning the [`Destination`] of [`SamSession`] is returned,
    /// a `.b32.i2p` which starts a lease set query for the destination. or a `.i2p` host name which
    /// is looked up from an address book if it exists.
    ///
    /// For `.b32.i2p`/`.i2p`, naming reply is deferred until the query is finished.
    fn on_naming_lookup(&mut self, name: String) {
        if name.as_str() == "ME" {
            tracing::debug!(
                target: LOG_TARGET,
                session_id = %self.session_id,
                "naming lookup for self",
            );

            if let Some(socket) = &mut self.socket {
                socket.send_message(
                    format!(
                        "NAMING REPLY RESULT=OK NAME=ME VALUE={}\n",
                        base64_encode(self.dest.serialized())
                    )
                    .as_bytes()
                    .to_vec(),
                );
            }

            return;
        }

        // if the host name ends in `.b32.i2p`, validate the hostname and check if [`Destination`]
        // already holds the host's lease set and if not, start a query
        //
        // once the query finishes, the naming reply is sent to client
        if let Some(end) = name.find(".b32.i2p") {
            tracing::debug!(
                target: LOG_TARGET,
                session_id = %self.session_id,
                "naming lookup for .b32.i2p address",
            );

            let start = if name.starts_with("http://") {
                7usize
            } else if name.starts_with("https://") {
                8usize
            } else {
                0usize
            };

            let message = match base32_decode(&name[start..end]) {
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        session_id = %self.session_id,
                        ?name,
                        "invalid .b32.i2p address",
                    );

                    Some(
                        format!("NAMING REPLY RESULT=INVALID_KEY NAME={name}\n")
                            .as_bytes()
                            .to_vec(),
                    )
                }
                Some(destination) => {
                    let destination_id = DestinationId::from(destination);

                    match self.destination.query_lease_set(&destination_id) {
                        LeaseSetStatus::Found => {
                            tracing::trace!(
                                target: LOG_TARGET,
                                session_id = %self.session_id,
                                %destination_id,
                                ?name,
                                "lease set found for host",
                            );

                            Some(
                                format!(
                                    "NAMING REPLY RESULT=OK NAME={name} VALUE={}\n",
                                    base64_encode(
                                        self.destination
                                            .lease_set(&destination_id)
                                            .header
                                            .destination
                                            .serialized()
                                    )
                                )
                                .as_bytes()
                                .to_vec(),
                            )
                        }
                        status => {
                            tracing::trace!(
                                target: LOG_TARGET,
                                session_id = %self.session_id,
                                %destination_id,
                                ?name,
                                ?status,
                                "lease set not found for host, query started",
                            );
                            self.pending_host_lookups.insert(destination_id, name);

                            None
                        }
                    }
                }
            };

            if let (Some(socket), Some(message)) = (&mut self.socket, message) {
                socket.send_message(message);
            }

            return;
        }

        let message = match name.find(".i2p") {
            None => {
                tracing::warn!(
                    target: LOG_TARGET,
                    session_id = %self.session_id,
                    ?name,
                    "invalid host name",
                );

                Some(format!("NAMING REPLY RESULT=INVALID_KEY NAME={name}\n").as_bytes().to_vec())
            }
            Some(_) => match &self.address_book {
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        session_id = %self.session_id,
                        ?name,
                        "address book doesn't exist",
                    );

                    Some(
                        format!("NAMING REPLY RESULT=KEY_NOT_FOUND NAME={name}\n")
                            .as_bytes()
                            .to_vec(),
                    )
                }
                Some(address_book) => {
                    tracing::debug!(
                        target: LOG_TARGET,
                        ?name,
                        "lookup name from address book",
                    );

                    let future = address_book.resolve_base64(name.clone());
                    self.lookup_futures.push(async move { (name, future.await) });

                    None
                }
            },
        };

        if let (Some(socket), Some(message)) = (&mut self.socket, message) {
            socket.send_message(message);
        }
    }

    /// Attempt to create new sub-session.
    ///
    /// The sub-session is rejected if [`SamSessionKind`] is not `Primary`, if there already exists
    /// a sub-session with the same session ID or if [`SamServer`] fails to send the sub-session ->
    /// primary session ID mapping to [`SamServer`].
    ///
    /// On success, the sub-session ID is added to the list of sub-sessions the primary session has.
    ///
    /// Returns a message indicating whether the sub-session was created successfully, which must be
    /// sent to the client.
    fn on_create_sub_session(
        &mut self,
        session_id: Arc<str>,
        session_kind: SessionKind,
        options: HashMap<String, String>,
    ) -> Vec<u8> {
        let SamSessionKind::Primary { sub_sessions } = &mut self.session_kind else {
            tracing::warn!(
                target: LOG_TARGET,
                session_id = %self.session_id,
                sub_session_id = %session_id,
                kind = ?self.session_kind,
                "sub-sessions not supported for the configured session kind",
            );

            return b"SESSION STATUS RESULT=I2P_ERROR MESSAGE=\"not a primary session\"\n".to_vec();
        };

        if let Some(session_kind) = sub_sessions.get(&session_id) {
            tracing::warn!(
                target: LOG_TARGET,
                session_id = %self.session_id,
                sub_session_id = %session_id,
                ?session_kind,
                "duplicate sub-session id",
            );

            return b"SESSION STATUS RESULT=DUPLICATE_ID\n".to_vec();
        }

        // `sub_session_tx` must exist since the session kind is `Primary`
        if let Err(error) =
            self.sub_session_tx
                .as_ref()
                .expect("to exist")
                .try_send(SubSessionCommand::Add {
                    primary_session_id: Arc::clone(&self.session_id),
                    sub_session_id: Arc::clone(&session_id),
                })
        {
            tracing::warn!(
                target: LOG_TARGET,
                session_id = %self.session_id,
                sub_session_id = %session_id,
                ?error,
                "failed register sub-session to sam server",
            );

            return b"SESSION STATUS RESULT=I2P_ERROR MESSAGE=\"internal error\"\n".to_vec();
        }

        // if session kind indicated datagrams, attempt to add listener into `DatagramManager`
        if core::matches!(
            session_kind,
            SessionKind::Datagram | SessionKind::Anonymous | SessionKind::Datagram2
        ) {
            if let Err(()) = self.datagram_manager.add_listener(options) {
                return b"SESSION STATUS RESULT=I2P_ERROR MESSAGE=\"invalid datagram configuration\"\n".to_vec();
            }
        }

        tracing::debug!(
            target: LOG_TARGET,
            session_id = %self.session_id,
            sub_session_id = %session_id,
            ?session_kind,
            "create new sub-session",
        );

        sub_sessions.insert(Arc::clone(&session_id), session_kind);

        format!("SESSION STATUS RESULT=OK ID=\"{session_id}\" MESSAGE=\"ADD {session_id}\"\n")
            .as_bytes()
            .to_vec()
    }
}

impl<R: Runtime> Future for SamSession<R> {
    type Output = Arc<str>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let command = match &mut self.socket {
                None => break,
                Some(socket) => match socket.poll_next_unpin(cx) {
                    Poll::Pending => break,
                    Poll::Ready(None) => {
                        tracing::info!(
                            target: LOG_TARGET,
                            session_id = %self.session_id,
                            "session socket closed, destroy session",
                        );

                        self.stream_manager.shutdown();
                        self.socket = None;
                        break;
                    }
                    Poll::Ready(Some(command)) => command,
                },
            };

            match command {
                SamCommand::NamingLookup { name } => self.on_naming_lookup(name),
                SamCommand::CreateSubSession {
                    session_id,
                    session_kind,
                    options,
                } => {
                    let message =
                        self.on_create_sub_session(Arc::from(session_id), session_kind, options);

                    if let Some(socket) = &mut self.socket {
                        socket.send_message(message);

                        if let Some(waker) = self.waker.take() {
                            waker.wake_by_ref();
                        }
                    }
                }
                SamCommand::Quit => {
                    tracing::info!(
                        target: LOG_TARGET,
                        session_id = %self.session_id,
                        "shutting down session",
                    );
                    return Poll::Ready(Arc::clone(&self.session_id));
                }
                SamCommand::Ping(text) => {
                    let Some(ref mut socket) = self.socket else {
                        continue;
                    };

                    match text {
                        Some(text) => {
                            socket.send_message(format!("PONG {text}\n").as_bytes().to_vec());
                        }
                        None => {
                            socket.send_message(b"PONG\n".to_vec());
                        }
                    }
                }
                command => tracing::warn!(
                    target: LOG_TARGET,
                    %command,
                    "ignoring command for active session",
                ),
            }
        }

        loop {
            match self.receiver.poll_recv(cx) {
                Poll::Pending => break,
                Poll::Ready(None) => return Poll::Ready(Arc::clone(&self.session_id)),
                Poll::Ready(Some(SamSessionCommand::Connect {
                    socket,
                    destination_id,
                    options,
                    session_id,
                })) => self.on_stream_connect(socket, destination_id, options, session_id),
                Poll::Ready(Some(SamSessionCommand::Accept {
                    socket,
                    options,
                    session_id,
                })) => self.on_stream_accept(socket, options, session_id),
                Poll::Ready(Some(SamSessionCommand::Forward {
                    socket,
                    port,
                    options,
                    session_id,
                })) => self.on_stream_forward(socket, port, options, session_id),
                Poll::Ready(Some(SamSessionCommand::SendDatagram {
                    destination,
                    datagram,
                    session_id,
                    options,
                })) => self.on_send_datagram(*destination, datagram, session_id, options),
                Poll::Ready(Some(SamSessionCommand::Dummy)) => unreachable!(),
            }
        }

        loop {
            match self.stream_manager.poll_next_unpin(cx) {
                Poll::Pending => break,
                Poll::Ready(None) => return Poll::Ready(Arc::clone(&self.session_id)),
                Poll::Ready(Some(StreamManagerEvent::SendPacket {
                    delivery_style,
                    dst_port,
                    packet,
                    src_port,
                })) => {
                    let Some(message) = I2cpPayloadBuilder::<R>::new(&packet)
                        .with_protocol(Protocol::Streaming)
                        .with_source_port(src_port)
                        .with_destination_port(dst_port)
                        .build()
                    else {
                        tracing::warn!(
                            target: LOG_TARGET,
                            session_id = ?self.session_id,
                            "failed to create i2cp payload",
                        );
                        continue;
                    };

                    if let Err(error) = self.destination.send_message(delivery_style, message) {
                        tracing::warn!(
                            target: LOG_TARGET,
                            session_id = ?self.session_id,
                            ?error,
                            "failed to encrypt message",
                        );
                        debug_assert!(false);
                    } else {
                        // Outbound streaming packet accepted for I2P delivery.
                        self.note_qualifying_activity();
                    };
                }
                Poll::Ready(Some(StreamManagerEvent::StreamOpened {
                    destination_id,
                    direction,
                })) => match direction {
                    Direction::Inbound => {}
                    Direction::Outbound => {
                        self.pending_outbound.remove(&destination_id);
                    }
                },
                Poll::Ready(Some(StreamManagerEvent::StreamRejected {
                    destination_id,
                    socket_id,
                })) => {
                    self.pending_outbound.remove(&destination_id);
                    self.remove_observed_socket(socket_id);
                }
                Poll::Ready(Some(StreamManagerEvent::StreamClosed {
                    destination_id,
                    socket_id,
                })) => {
                    tracing::debug!(
                        target: LOG_TARGET,
                        session_id = ?self.session_id,
                        ?destination_id,
                        "stream closed",
                    );
                    if let Some(socket_id) = socket_id {
                        self.remove_observed_socket(socket_id);
                    }
                }
                Poll::Ready(Some(StreamManagerEvent::ShutDown)) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        session_id = ?self.session_id,
                        "stream manager shut down, shutting down tunnel pool",
                    );
                    self.destination.shutdown();
                }
            }
        }

        loop {
            match self.destination.poll_next_unpin(cx) {
                Poll::Pending => break,
                Poll::Ready(None) => return Poll::Ready(Arc::clone(&self.session_id)),
                Poll::Ready(Some(DestinationEvent::Messages { messages })) => {
                    self.on_inbound_message(messages)
                }
                Poll::Ready(Some(DestinationEvent::LeaseSetFound { destination_id })) => {
                    self.on_lease_set_found(destination_id)
                }
                Poll::Ready(Some(DestinationEvent::LeaseSetNotFound {
                    destination_id,
                    error,
                })) => self.on_lease_set_not_found(destination_id, error),
                Poll::Ready(Some(DestinationEvent::TunnelPoolShutDown)) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        session_id = ?self.session_id,
                        "tunnel pool shut down, shutting down session",
                    );

                    return Poll::Ready(Arc::clone(&self.session_id));
                }
                Poll::Ready(Some(DestinationEvent::CreateLeaseSet { leases })) => {
                    tracing::trace!(
                        target: LOG_TARGET,
                        session_id = ?self.session_id,
                        num_leases = ?leases.len(),
                        "create new lease set",
                    );

                    let lease_set = Bytes::from(
                        LeaseSet2 {
                            header: LeaseSet2Header {
                                destination: self.dest.clone(),
                                is_unpublished: self
                                    .options
                                    .get("i2cp.dontPublishLeaseSet")
                                    .map(|value| value.parse::<bool>().unwrap_or(false))
                                    .unwrap_or(false),
                                expires: Duration::from_secs(10 * 60).as_secs() as u32,
                                offline_signature: None,
                                published: R::time_since_epoch().as_secs() as u32,
                            },
                            public_keys: self.public_key_context.public_keys(),
                            leases,
                        }
                        .serialize(&self.signing_key),
                    );
                    self.destination.publish_lease_set(lease_set);
                }
                Poll::Ready(Some(DestinationEvent::SessionTerminated { destination_id })) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        session_id = ?self.session_id,
                        destination_id = %destination_id,
                        "session termianted with remote",
                    );
                    self.stream_manager.remove_session(&destination_id);
                }
            }
        }

        loop {
            match self.lookup_futures.poll_next_unpin(cx) {
                Poll::Pending => break,
                Poll::Ready(None) => return Poll::Ready(Arc::clone(&self.session_id)),
                Poll::Ready(Some((name, result))) => {
                    let message = match result {
                        Some(destination) => {
                            tracing::trace!(
                                target: LOG_TARGET,
                                session_id = ?self.session_id,
                                %name,
                                "naming lookup succeeded",
                            );

                            format!("NAMING REPLY RESULT=OK NAME={name} VALUE={destination}\n")
                                .as_bytes()
                                .to_vec()
                        }
                        None => {
                            tracing::warn!(
                                target: LOG_TARGET,
                                session_id = ?self.session_id,
                                %name,
                                "naming lookup failed",
                            );

                            format!("NAMING REPLY RESULT=KEY_NOT_FOUND NAME={name}\n")
                                .as_bytes()
                                .to_vec()
                        }
                    };

                    if let Some(socket) = &mut self.socket {
                        socket.send_message(message);

                        if let Some(waker) = self.waker.take() {
                            waker.wake_by_ref();
                        }
                    }
                }
            }
        }

        // Drive the single generation-local idle timer after all qualifying
        // activity in this turn has been recorded, so activity in the same
        // turn wins over the deadline. The timer wakes this future at the
        // reduction deadline even when no other event arrives. Shutdown
        // paths above return early; teardown drops the timer via
        // `cancel_idle_state` on socket close below.
        if self.socket.is_none() {
            self.cancel_idle_state();
        } else {
            self.poll_idle_timer(cx);
        }

        self.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        crypto::SigningPrivateKey,
        events::{EventManager, EventSubscriber},
        netdb::{NetDbAction, NetDbActionRecycle, NetDbHandle},
        primitives::Destination,
        profile::ProfileStorage,
        runtime::{
            mock::{MockRuntime, MockTcpStream},
            TcpStream,
        },
        sam::{parser::SessionKind, socket::SamSocket},
        tunnel::{TunnelMessage, TunnelMessageRecycle, TunnelPoolEvent, TunnelPoolHandle},
    };
    use thingbuf::mpsc;
    use tokio::{
        io::{AsyncBufReadExt, AsyncReadExt, BufReader},
        net,
    };

    #[allow(unused)]
    struct TestSessionContext {
        client_socket: net::TcpStream,
        datagram_rx: Receiver<(u16, Vec<u8>)>,
        event_manager: EventManager<MockRuntime>,
        event_subscriber: EventSubscriber,
        netdb_rx: mpsc::Receiver<NetDbAction, NetDbActionRecycle>,
        shutdown_rx: futures_channel::oneshot::Receiver<()>,
        sub_rx: Receiver<SubSessionCommand>,
        tm_recv: mpsc::Receiver<TunnelMessage, TunnelMessageRecycle>,
        tp_event: mpsc::Sender<TunnelPoolEvent>,
        tx: Sender<SamSessionCommand<MockRuntime>, SamSessionCommandRecycle>,
    }

    async fn create_session() -> (SamSession<MockRuntime>, TestSessionContext) {
        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let signing_key = SigningPrivateKey::random(MockRuntime::rng());
        let destination = Destination::new::<MockRuntime>(signing_key.public());

        let (datagram_tx, datagram_rx) = mpsc::channel(10);
        let (sub_tx, sub_rx) = mpsc::channel(10);
        let (netdb_handle, netdb_rx) = NetDbHandle::create();
        let (event_manager, event_subscriber, event_handle) =
            EventManager::new(None, MockRuntime::register_metrics(vec![], None));
        let (tunnel_pool_handle, tm_recv, tp_event, shutdown_rx) = TunnelPoolHandle::create();
        let (tx, rx) = mpsc::with_recycle(64, SamSessionCommandRecycle::default());

        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());

        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (client_socket, _) = stream2.unwrap();
        let options =
            HashMap::from_iter([(String::from("i2cp.leaseSetEncType"), String::from("6,4"))]);

        (
            SamSession::new(SamSessionContext {
                address_book: None,
                datagram_tx,
                destination: DestinationContext {
                    destination,
                    private_key: Vec::new(),
                    signing_key: Box::new(signing_key),
                },
                event_handle,
                inbound: Default::default(),
                netdb_handle,
                options,
                outbound: Default::default(),
                profile_storage: ProfileStorage::new(&[], &[], None),
                receiver: rx,
                session_id: "test".into(),
                session_kind: SessionKind::Stream,
                socket,
                sub_session_tx: Some(sub_tx),
                tunnel_pool_handle,
            }),
            TestSessionContext {
                client_socket,
                datagram_rx,
                event_manager,
                event_subscriber,
                netdb_rx,
                shutdown_rx,
                sub_rx,
                tm_recv,
                tp_event,
                tx,
            },
        )
    }

    #[tokio::test]
    async fn create_stream_sub_session() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Primary {
            sub_sessions: HashMap::new(),
        };

        let result =
            session.on_create_sub_session("sub1".into(), SessionKind::Stream, HashMap::new());
        assert!(String::from_utf8_lossy(&result).contains("RESULT=OK"));

        if let SamSessionKind::Primary { sub_sessions } = &session.session_kind {
            assert!(sub_sessions.contains_key("sub1"));
            assert_eq!(sub_sessions.get("sub1"), Some(&SessionKind::Stream));
        }
    }

    #[tokio::test]
    async fn duplicate_sub_session() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Primary {
            sub_sessions: HashMap::new(),
        };

        // Create first sub-session
        let _ = session.on_create_sub_session("sub1".into(), SessionKind::Stream, HashMap::new());

        // Try to create duplicate
        let result =
            session.on_create_sub_session("sub1".into(), SessionKind::Stream, HashMap::new());
        assert!(String::from_utf8_lossy(&result).contains("DUPLICATE_ID"));
    }

    #[tokio::test]
    async fn non_primary_sub_session() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Stream;

        let result =
            session.on_create_sub_session("sub1".into(), SessionKind::Stream, HashMap::new());
        assert!(String::from_utf8_lossy(&result).contains("not a primary session"));
    }

    #[tokio::test]
    async fn create_datagram_sub_session() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Primary {
            sub_sessions: HashMap::new(),
        };

        let mut options = HashMap::new();
        options.insert("HOST".to_string(), "127.0.0.1".to_string());
        options.insert("PORT".to_string(), "1234".to_string());

        let result = session.on_create_sub_session("sub1".into(), SessionKind::Datagram, options);
        assert!(String::from_utf8_lossy(&result).contains("RESULT=OK"));

        if let SamSessionKind::Primary { sub_sessions } = &session.session_kind {
            assert!(sub_sessions.contains_key("sub1"));
            assert_eq!(sub_sessions.get("sub1"), Some(&SessionKind::Datagram));
        }
    }

    #[tokio::test]
    async fn create_multiple_datagram_sub_sessions() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Primary {
            sub_sessions: HashMap::new(),
        };

        // First subsession with default FROM_PORT (0)
        let mut options1 = HashMap::new();
        options1.insert("HOST".to_string(), "127.0.0.1".to_string());
        options1.insert("PORT".to_string(), "1234".to_string());

        let result = session.on_create_sub_session("sub1".into(), SessionKind::Datagram, options1);
        assert!(String::from_utf8_lossy(&result).contains("RESULT=OK"));

        // Second subsession with explicit FROM_PORT
        let mut options2 = HashMap::new();
        options2.insert("HOST".to_string(), "127.0.0.1".to_string());
        options2.insert("PORT".to_string(), "5678".to_string());
        options2.insert("FROM_PORT".to_string(), "9999".to_string());

        let result = session.on_create_sub_session("sub2".into(), SessionKind::Datagram, options2);
        assert!(String::from_utf8_lossy(&result).contains("RESULT=OK"));

        // Third subsession with different FROM_PORT
        let mut options3 = HashMap::new();
        options3.insert("HOST".to_string(), "127.0.0.1".to_string());
        options3.insert("PORT".to_string(), "8080".to_string());
        options3.insert("FROM_PORT".to_string(), "7777".to_string());

        let result = session.on_create_sub_session("sub3".into(), SessionKind::Datagram, options3);
        assert!(String::from_utf8_lossy(&result).contains("RESULT=OK"));

        // Verify all subsessions were registered
        if let SamSessionKind::Primary { sub_sessions } = &session.session_kind {
            assert_eq!(sub_sessions.len(), 3);
            assert_eq!(sub_sessions.get("sub1"), Some(&SessionKind::Datagram));
            assert_eq!(sub_sessions.get("sub2"), Some(&SessionKind::Datagram));
            assert_eq!(sub_sessions.get("sub3"), Some(&SessionKind::Datagram));
        }

        // Try to create subsession with duplicate FROM_PORT (should fail)
        let mut options4 = HashMap::new();
        options4.insert("HOST".to_string(), "127.0.0.1".to_string());
        options4.insert("PORT".to_string(), "4444".to_string());
        options4.insert("FROM_PORT".to_string(), "9999".to_string());

        let result = session.on_create_sub_session("sub4".into(), SessionKind::Datagram, options4);
        assert!(String::from_utf8_lossy(&result).contains("invalid datagram configuration"));

        // Verify the failed attempt didn't affect existing mappings
        if let SamSessionKind::Primary { sub_sessions } = &session.session_kind {
            assert_eq!(sub_sessions.len(), 3);
        }
    }

    #[tokio::test]
    async fn reject_datagram_sub_session_with_occupied_port() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Primary {
            sub_sessions: HashMap::new(),
        };

        // Create first subsession with PORT 1234
        let mut options1 = HashMap::new();
        options1.insert("HOST".to_string(), "127.0.0.1".to_string());
        options1.insert("PORT".to_string(), "1234".to_string());
        options1.insert("FROM_PORT".to_string(), "5555".to_string());

        let result = session.on_create_sub_session("sub1".into(), SessionKind::Datagram, options1);
        assert!(String::from_utf8_lossy(&result).contains("RESULT=OK"));

        // Try to create another subsession with same PORT but different FROM_PORT
        let mut options2 = HashMap::new();
        options2.insert("HOST".to_string(), "127.0.0.1".to_string());
        options2.insert("PORT".to_string(), "1234".to_string());
        options2.insert("FROM_PORT".to_string(), "5555".to_string());

        let result = session.on_create_sub_session("sub2".into(), SessionKind::Datagram, options2);
        assert!(String::from_utf8_lossy(&result).contains("invalid datagram configuration"));

        // Verify only the first subsession was registered
        if let SamSessionKind::Primary { sub_sessions } = &session.session_kind {
            assert_eq!(sub_sessions.len(), 1);
            assert_eq!(sub_sessions.get("sub1"), Some(&SessionKind::Datagram));
        }
    }

    #[tokio::test]
    async fn register_sub_session_sam_server_exited() {
        let (mut session, _) = create_session().await;
        session.session_kind = SamSessionKind::Primary {
            sub_sessions: HashMap::new(),
        };

        let result =
            session.on_create_sub_session("sub1".into(), SessionKind::Stream, HashMap::new());
        assert!(String::from_utf8_lossy(&result).contains("internal error"));
    }

    #[tokio::test]
    async fn naming_lookup_me() {
        let (mut session, mut ctx) = create_session().await;
        session.on_naming_lookup("ME".to_string());
        tokio::spawn(async move { session.socket.as_mut().expect("to exist").next().await });

        // verify response contains base64 encoded destination
        let mut reader = BufReader::new(&mut ctx.client_socket);
        let mut response = String::new();

        // discard `SESSION STATUS` message
        reader.read_line(&mut response).await.expect("to succeed");

        // read `NAMING LOOKUP` message
        reader.read_line(&mut response).await.expect("to succeed");

        assert!(response.contains("NAMING REPLY RESULT=OK NAME=ME VALUE="));
        assert!(response.ends_with("\n"));
    }

    #[tokio::test]
    async fn naming_lookup_b32_invalid() {
        let (mut session, mut ctx) = create_session().await;
        session.on_naming_lookup("invalid.b32.i2p".to_string());
        tokio::spawn(async move { session.socket.as_mut().expect("to exist").next().await });

        // verify response contains base64 encoded destination
        let mut reader = BufReader::new(&mut ctx.client_socket);
        let mut response = String::new();

        // discard `SESSION STATUS` message
        reader.read_line(&mut response).await.expect("to succeed");

        // read `NAMING LOOKUP` message
        reader.read_line(&mut response).await.expect("to succeed");
        assert!(response.contains("RESULT=INVALID_KEY"));
        assert!(response.ends_with("\n"));
    }

    #[tokio::test]
    async fn naming_lookup_b32_with_http() {
        let (mut session, mut ctx) = create_session().await;

        // test with http:// prefix
        session.on_naming_lookup("http://abcdef.b32.i2p".to_string());
        tokio::spawn(async move { session.socket.as_mut().expect("to exist").next().await });

        // verify error response when no address book exists
        let mut reader = BufReader::new(&mut ctx.client_socket);
        let mut response = String::new();

        // discard `SESSION STATUS` message
        reader.read_line(&mut response).await.expect("to succeed");

        // read `NAMING LOOKUP` message
        reader.read_line(&mut response).await.expect("to succeed");
        assert!(response.contains("RESULT=INVALID_KEY"));
    }

    #[tokio::test]
    async fn naming_lookup_b32_with_https() {
        let (mut session, mut ctx) = create_session().await;

        // test with https:// prefix
        session.on_naming_lookup("https://abcdef.b32.i2p".to_string());
        tokio::spawn(async move { session.socket.as_mut().expect("to exist").next().await });

        // verify error response when no address book exists
        let mut reader = BufReader::new(&mut ctx.client_socket);
        let mut response = String::new();

        // discard `SESSION STATUS` message
        reader.read_line(&mut response).await.expect("to succeed");

        // read `NAMING LOOKUP` message
        reader.read_line(&mut response).await.expect("to succeed");
        assert!(response.contains("RESULT=INVALID_KEY"));
    }

    #[tokio::test]
    async fn naming_lookup_i2p_no_addressbook() {
        let (mut session, mut ctx) = create_session().await;
        session.on_naming_lookup("example.i2p".to_string());
        tokio::spawn(async move { session.socket.as_mut().expect("to exist").next().await });

        // verify error response when no address book exists
        let mut reader = BufReader::new(&mut ctx.client_socket);
        let mut response = String::new();

        // discard `SESSION STATUS` message
        reader.read_line(&mut response).await.expect("to succeed");

        // read `NAMING LOOKUP` message
        reader.read_line(&mut response).await.expect("to succeed");

        assert!(response.contains("RESULT=KEY_NOT_FOUND"));
        assert!(response.ends_with("\n"));
    }

    #[tokio::test]
    async fn naming_lookup_invalid_name() {
        let (mut session, mut ctx) = create_session().await;
        session.on_naming_lookup("invalid-name-without-tld".to_string());
        tokio::spawn(async move { session.socket.as_mut().expect("to exist").next().await });

        // verify error response for invalid hostname
        let mut reader = BufReader::new(&mut ctx.client_socket);
        let mut response = String::new();

        // discard `SESSION STATUS` message
        reader.read_line(&mut response).await.expect("to succeed");

        // read `NAMING LOOKUP` message
        reader.read_line(&mut response).await.expect("to succeed");

        assert!(response.contains("RESULT=INVALID_KEY"));
        assert!(response.ends_with("\n"));
    }

    #[tokio::test]
    async fn naming_lookup_clearnet() {
        let (mut session, mut ctx) = create_session().await;
        session.on_naming_lookup("https://google.com".to_string());
        tokio::spawn(async move { session.socket.as_mut().expect("to exist").next().await });

        // verify error response for invalid hostname
        let mut reader = BufReader::new(&mut ctx.client_socket);
        let mut response = String::new();

        // discard `SESSION STATUS` message
        reader.read_line(&mut response).await.expect("to succeed");

        // read `NAMING LOOKUP` message
        reader.read_line(&mut response).await.expect("to succeed");

        assert!(response.contains("RESULT=INVALID_KEY"));
        assert!(response.ends_with("\n"));
    }

    #[tokio::test]
    async fn stream_connect_for_repliable() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Datagram {
            kind: SessionKind::Datagram,
        };

        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_connect(
            socket,
            DestinationId::random(),
            HashMap::new(),
            Arc::from("hello"),
        );

        let mut buf = vec![0u8; 128];
        match client_socket.read(&mut buf).await {
            Err(_) | Ok(0) => {}
            _ => panic!("invalid response"),
        }
    }

    #[tokio::test]
    async fn stream_connect_for_anonymous() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Datagram {
            kind: SessionKind::Anonymous,
        };

        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_connect(
            socket,
            DestinationId::random(),
            HashMap::new(),
            Arc::from("hello"),
        );

        let mut buf = vec![0u8; 128];
        match client_socket.read(&mut buf).await {
            Err(_) | Ok(0) => {}
            _ => panic!("invalid response"),
        }
    }

    #[tokio::test]
    async fn stream_connect_for_self() {
        let (mut session, _ctx) = create_session().await;

        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_connect(
            socket,
            session.dest.id(),
            HashMap::new(),
            Arc::from("hello"),
        );
        tokio::spawn(async move { session.socket.as_mut().expect("to exist").next().await });

        // verify error response for invalid hostname
        let mut reader = BufReader::new(&mut client_socket);
        let mut response = String::new();

        // read `NAMING LOOKUP` message
        reader.read_line(&mut response).await.expect("to succeed");

        assert!(response.contains("STREAM STATUS RESULT=CANT_REACH_PEER"));
    }

    #[tokio::test]
    async fn stream_accept_for_repliable() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Datagram {
            kind: SessionKind::Datagram,
        };

        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_accept(socket, HashMap::new(), Arc::from("hello"));

        let mut buf = vec![0u8; 128];
        match client_socket.read(&mut buf).await {
            Err(_) | Ok(0) => {}
            _ => panic!("invalid response"),
        }
    }

    #[tokio::test]
    async fn stream_accept_for_anonymous() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Datagram {
            kind: SessionKind::Anonymous,
        };

        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_accept(socket, HashMap::new(), Arc::from("hello"));

        let mut buf = vec![0u8; 128];
        match client_socket.read(&mut buf).await {
            Err(_) | Ok(0) => {}
            _ => panic!("invalid response"),
        }
    }

    #[tokio::test]
    async fn stream_forward_for_repliable() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Datagram {
            kind: SessionKind::Datagram,
        };

        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_forward(socket, 8888, HashMap::new(), Arc::from("hello"));

        let mut buf = vec![0u8; 128];
        match client_socket.read(&mut buf).await {
            Err(_) | Ok(0) => {}
            _ => panic!("invalid response"),
        }
    }

    #[tokio::test]
    async fn stream_forward_for_anonymous() {
        let (mut session, _ctx) = create_session().await;
        session.session_kind = SamSessionKind::Datagram {
            kind: SessionKind::Anonymous,
        };

        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_forward(socket, 8888, HashMap::new(), Arc::from("hello"));

        let mut buf = vec![0u8; 128];
        match client_socket.read(&mut buf).await {
            Err(_) | Ok(0) => {}
            _ => panic!("invalid response"),
        }
    }

    #[tokio::test]
    async fn stream_accept_then_forward() {
        let (mut session, _ctx) = create_session().await;

        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_accept(socket, HashMap::new(), Arc::from("hello"));

        // read `STREAM ACCEPT` response
        let future = async {
            let mut reader = BufReader::new(&mut client_socket);
            let mut response = String::new();
            reader.read_line(&mut response).await.expect("to succeed");
            assert!(response.contains("STREAM STATUS RESULT=OK"));
        };
        assert!(tokio::time::timeout(Duration::from_secs(1), future).await.is_ok());

        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_forward(socket, 8888, HashMap::new(), Arc::from("hello"));

        // read `STREAM FORWARD` response
        let future = async {
            let mut reader = BufReader::new(&mut client_socket);
            let mut response = String::new();
            reader.read_line(&mut response).await.expect("to succeed");
            assert!(response.contains("STREAM STATUS RESULT=I2P_ERROR"));
        };
        assert!(tokio::time::timeout(Duration::from_secs(1), future).await.is_ok());
    }

    #[tokio::test]
    async fn stream_forward_then_accept() {
        let (mut session, _ctx) = create_session().await;

        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_forward(socket, address.port(), HashMap::new(), Arc::from("hello"));

        // read `STREAM FORWARD` response
        let future = async {
            let mut reader = BufReader::new(&mut client_socket);
            let mut response = String::new();
            reader.read_line(&mut response).await.expect("to succeed");
            assert!(response.contains("STREAM STATUS RESULT=OK"));
        };
        assert!(tokio::time::timeout(Duration::from_secs(1), future).await.is_ok());

        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (mut client_socket, _) = stream2.unwrap();

        session.on_stream_accept(socket, HashMap::new(), Arc::from("hello"));

        // read `STREAM ACCEPT` response
        let future = async {
            let mut reader = BufReader::new(&mut client_socket);
            let mut response = String::new();
            reader.read_line(&mut response).await.expect("to succeed");
            assert!(response.contains("STREAM STATUS RESULT=I2P_ERROR"));
        };
        assert!(tokio::time::timeout(Duration::from_secs(1), future).await.is_ok());
    }

    // M136 reference/activity freeze and state-machine evidence.
    //
    // Call-site table (Emissary boundaries):
    // - qualifying outbound streaming: `create_outbound_stream` SYN accepted
    //   + `StreamManagerEvent::SendPacket` accepted (`destination.send_message` Ok);
    // - qualifying inbound streaming: `on_inbound_message` Streaming
    //   `stream_manager.on_packet` Ok;
    // - qualifying outbound datagram: `on_send_datagram` Found + send Ok,
    //   plus flushed pending datagrams in `on_lease_set_found`;
    // - qualifying inbound datagram: `on_inbound_message` non-Streaming
    //   `datagram_manager.on_datagram` Ok;
    // - excluded: `on_naming_lookup`, `SamCommand::Ping`, tunnel/NetDb
    //   maintenance, `on_stream_accept`/`on_stream_forward` listener
    //   registration, handler counts.
    // Reference freeze: I2CP `reduceOnIdle` master switch, default
    // 1200000 ms, minimum 300000 ms, default quantity 1 coerced to >=1,
    // `updateTunnels(session, q)` decrease + `updateTunnels(session, 0)`
    // restore shape via the live-quantity bridge, primary/subsession
    // aggregation at the owning session, Streamr/datagram sessions use the
    // same generic session owner.

    fn idle_options(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        HashMap::from_iter(pairs.iter().map(|(k, v)| (String::from(*k), String::from(*v))))
    }

    async fn create_idle_session(
        mut options: HashMap<String, String>,
    ) -> (SamSession<MockRuntime>, TestSessionContext) {
        options
            .entry(String::from("i2cp.leaseSetEncType"))
            .or_insert_with(|| String::from("6,4"));
        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let signing_key = SigningPrivateKey::random(MockRuntime::rng());
        let destination = Destination::new::<MockRuntime>(signing_key.public());

        let (datagram_tx, datagram_rx) = mpsc::channel(10);
        let (sub_tx, sub_rx) = mpsc::channel(10);
        let (netdb_handle, netdb_rx) = NetDbHandle::create();
        let (event_manager, event_subscriber, event_handle) =
            EventManager::new(None, MockRuntime::register_metrics(vec![], None));
        let (tunnel_pool_handle, tm_recv, tp_event, shutdown_rx) = TunnelPoolHandle::create();
        let (tx, rx) = mpsc::with_recycle(64, SamSessionCommandRecycle::default());

        let (stream1, stream2) = tokio::join!(MockTcpStream::connect(address), listener.accept());

        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        let (client_socket, _) = stream2.unwrap();

        (
            SamSession::new(SamSessionContext {
                address_book: None,
                datagram_tx,
                destination: DestinationContext {
                    destination,
                    private_key: Vec::new(),
                    signing_key: Box::new(signing_key),
                },
                event_handle,
                inbound: Default::default(),
                netdb_handle,
                options,
                outbound: Default::default(),
                profile_storage: ProfileStorage::new(&[], &[], None),
                receiver: rx,
                session_id: "idle-test".into(),
                session_kind: SessionKind::Stream,
                socket,
                sub_session_tx: Some(sub_tx),
                tunnel_pool_handle,
            }),
            TestSessionContext {
                client_socket,
                datagram_rx,
                event_manager,
                event_subscriber,
                netdb_rx,
                shutdown_rx,
                sub_rx,
                tm_recv,
                tp_event,
                tx,
            },
        )
    }

    #[test]
    fn m136_idle_policy_parsing_is_deterministic_and_bounded() {
        // Disabled unless exactly true (case-insensitive).
        assert!(!IdlePolicy::parse(&HashMap::new()).enabled);
        assert!(!IdlePolicy::parse(&idle_options(&[("i2cp.reduceOnIdle", "false")])).enabled);
        assert!(!IdlePolicy::parse(&idle_options(&[("i2cp.reduceOnIdle", "1")])).enabled);
        assert!(!IdlePolicy::parse(&idle_options(&[("i2cp.reduceOnIdle", "")])).enabled);
        assert!(IdlePolicy::parse(&idle_options(&[("i2cp.reduceOnIdle", "true")])).enabled);
        assert!(IdlePolicy::parse(&idle_options(&[("i2cp.reduceOnIdle", "TRUE")])).enabled);

        // Defaults.
        let cfg = IdlePolicy::parse(&idle_options(&[("i2cp.reduceOnIdle", "true")]));
        assert_eq!(cfg.idle_time, Duration::from_millis(1_200_000));
        assert_eq!(cfg.target_quantity, 1);

        // Minimum clamp.
        let cfg = IdlePolicy::parse(&idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "1000"),
        ]));
        assert_eq!(cfg.idle_time, Duration::from_millis(300_000));

        // Malformed time falls back to default (fail-safe, no disruption).
        let cfg = IdlePolicy::parse(&idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "not-a-number"),
        ]));
        assert_eq!(cfg.idle_time, Duration::from_millis(1_200_000));

        // Quantity coerces <1 to 1 (reference) and clamps to live bound.
        let cfg = IdlePolicy::parse(&idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceQuantity", "0"),
        ]));
        assert_eq!(cfg.target_quantity, 1);
        let cfg = IdlePolicy::parse(&idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceQuantity", "9999"),
        ]));
        assert_eq!(
            cfg.target_quantity,
            crate::tunnel::MAX_DESIRED_TUNNEL_QUANTITY
        );

        // Disabled policy carries no timer/work values.
        let cfg = IdlePolicy::parse(&HashMap::new());
        assert!(!cfg.enabled);
    }

    #[test]
    fn m136_idle_policy_carries_no_admin_vocabulary_in_diagnostics() {
        let cfg = IdlePolicy::parse(&idle_options(&[("i2cp.reduceOnIdle", "true")]));
        let debug = format!("{cfg:?}");
        for forbidden in [
            "Proposal",
            "I2PControl",
            "TunnelManager",
            "JsonRpc",
            "jsonrpc",
        ] {
            assert!(
                !debug.contains(forbidden),
                "diagnostic contains forbidden term {forbidden}: {debug}"
            );
        }
    }

    #[tokio::test]
    async fn m136_no_reduce_options_means_no_timer_and_unchanged_target() {
        let (session, _ctx) = create_idle_session(HashMap::new()).await;
        assert!(!session.idle_policy.enabled);
        assert!(session.idle_timer.is_none());
        assert!(!session.idle_reduced);
        assert_eq!(session.destination.base_quantity_target(), (3, 3));
        assert_eq!(session.destination.desired_quantity_target(), (3, 3));
        assert_eq!(session.destination.desired_inbound_count(), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn m136_reduce_enabled_no_reduction_before_threshold() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        // Override to a short deterministic deadline for the test while
        // keeping parsing evidence above for the reference minimum.
        session.idle_policy.idle_time = Duration::from_millis(200);
        session.idle_policy.target_quantity = 1;
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(200)));

        tokio::time::advance(Duration::from_millis(100)).await;
        // Poll the timer: still pending, no reduction.
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        session.poll_idle_timer(&mut cx);
        assert!(!session.idle_reduced);
        assert_eq!(session.destination.desired_quantity_target(), (3, 3));
        assert_eq!(session.destination.base_quantity_target(), (3, 3));
    }

    #[tokio::test(start_paused = true)]
    async fn m136_exact_threshold_reduces_and_base_unchanged() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session.idle_policy.idle_time = Duration::from_millis(100);
        session.idle_policy.target_quantity = 1;
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(100)));

        tokio::time::advance(Duration::from_millis(100)).await;
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        session.poll_idle_timer(&mut cx);

        assert!(session.idle_reduced);
        assert!(session.idle_timer.is_none());
        assert_eq!(session.destination.desired_quantity_target(), (1, 1));
        assert_eq!(session.destination.base_quantity_target(), (3, 3));
        assert_eq!(session.destination.desired_inbound_count(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn m136_outbound_streaming_activity_resets_idle_before_reduction() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session.idle_policy.idle_time = Duration::from_millis(200);
        session.idle_policy.target_quantity = 1;
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(200)));

        tokio::time::advance(Duration::from_millis(100)).await;
        // Simulate accepted outbound streaming packet.
        session.note_qualifying_activity();

        tokio::time::advance(Duration::from_millis(150)).await;
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        session.poll_idle_timer(&mut cx);
        // Only 150ms since last activity (<200ms): no reduction.
        assert!(!session.idle_reduced);
        assert_eq!(session.destination.desired_quantity_target(), (3, 3));
    }

    #[tokio::test(start_paused = true)]
    async fn m136_inbound_streaming_activity_resets_idle() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session.idle_policy.idle_time = Duration::from_millis(200);
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(200)));

        tokio::time::advance(Duration::from_millis(100)).await;
        // Inbound streaming delivery uses the same owner.
        session.note_qualifying_activity();

        tokio::time::advance(Duration::from_millis(150)).await;
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        session.poll_idle_timer(&mut cx);
        assert!(!session.idle_reduced);
    }

    #[tokio::test(start_paused = true)]
    async fn m136_outbound_datagram_activity_resets_idle() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session.idle_policy.idle_time = Duration::from_millis(200);
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(200)));

        tokio::time::advance(Duration::from_millis(100)).await;
        session.note_qualifying_activity();

        tokio::time::advance(Duration::from_millis(150)).await;
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        session.poll_idle_timer(&mut cx);
        assert!(!session.idle_reduced);
    }

    #[tokio::test(start_paused = true)]
    async fn m136_inbound_datagram_activity_resets_idle_and_uses_same_owner() {
        // Datagram/Streamr sessions share the same generic owner: prove the
        // inbound datagram boundary drives the same timer.
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session.session_kind = SamSessionKind::Datagram {
            kind: SessionKind::Datagram,
        };
        session.idle_policy.idle_time = Duration::from_millis(200);
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(200)));

        tokio::time::advance(Duration::from_millis(100)).await;
        session.note_qualifying_activity();

        tokio::time::advance(Duration::from_millis(150)).await;
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        session.poll_idle_timer(&mut cx);
        assert!(!session.idle_reduced);
        // Same generation-local owner, not a second scheduler.
        assert!(session.idle_timer.is_some());
    }

    #[tokio::test]
    async fn m136_control_and_lookup_do_not_reset_activity() {
        let (mut session, mut ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        let before = session.idle_last_activity.elapsed();
        // PING/PONG and naming lookup are excluded.
        session.on_naming_lookup("ME".to_string());
        tokio::spawn(async move { session.socket.as_mut().expect("to exist").next().await });
        let mut reader = BufReader::new(&mut ctx.client_socket);
        let mut response = String::new();
        reader.read_line(&mut response).await.expect("to succeed");
        reader.read_line(&mut response).await.expect("to succeed");
        assert!(response.contains("NAMING REPLY"));

        let (mut session2, _ctx2) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        let last = session2.idle_last_activity;
        // Local listener registration does not define activity.
        let listener = net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stream1, _) = tokio::join!(MockTcpStream::connect(address), listener.accept());
        let socket = Box::new(SamSocket::<MockRuntime>::new(stream1.unwrap()));
        session2.on_stream_accept(socket, HashMap::new(), Arc::from("test"));
        // No qualifying activity recorded: timer still the initial one and
        // desired target unchanged.
        assert_eq!(session2.destination.desired_quantity_target(), (3, 3));
        assert!(!session2.idle_reduced);
        assert!(session2.idle_timer.is_some());
        let _ = (before, last);
    }

    #[tokio::test(start_paused = true)]
    async fn m136_activity_after_reduction_restores_base_target() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session.idle_policy.idle_time = Duration::from_millis(100);
        session.idle_policy.target_quantity = 1;
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(100)));

        tokio::time::advance(Duration::from_millis(100)).await;
        {
            let waker = futures::task::noop_waker();
            let mut cx = Context::from_waker(&waker);
            session.poll_idle_timer(&mut cx);
        }
        assert!(session.idle_reduced);
        assert_eq!(session.destination.desired_quantity_target(), (1, 1));

        // First qualifying activity while at low target restores base.
        session.note_qualifying_activity();
        assert!(!session.idle_reduced);
        assert_eq!(session.destination.desired_quantity_target(), (3, 3));
        assert_eq!(session.destination.base_quantity_target(), (3, 3));
        assert!(session.idle_timer.is_some());
    }

    #[tokio::test(start_paused = true)]
    async fn m136_failed_reduction_does_not_mark_reduced() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        // Force an out-of-bounds target to prove explicit failure handling
        // without mutating real state (parsing would never produce this).
        session.idle_policy.idle_time = Duration::from_millis(50);
        session.idle_policy.target_quantity = 9999;
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(50)));

        tokio::time::advance(Duration::from_millis(60)).await;
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        session.poll_idle_timer(&mut cx);

        assert!(!session.idle_reduced);
        assert_eq!(session.destination.desired_quantity_target(), (3, 3));
        // No unbounded retry: timer dropped after explicit failure.
        assert!(session.idle_timer.is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn m136_failed_restore_does_not_falsely_mark_restored() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session.idle_policy.idle_time = Duration::from_millis(50);
        session.idle_policy.target_quantity = 1;
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(50)));

        tokio::time::advance(Duration::from_millis(60)).await;
        {
            let waker = futures::task::noop_waker();
            let mut cx = Context::from_waker(&waker);
            session.poll_idle_timer(&mut cx);
        }
        assert!(session.idle_reduced);

        // Shut down the pool so restore fails with PoolShutDown.
        session.destination.shutdown();
        session.note_qualifying_activity();
        // Still marked at low target, never falsely restored.
        assert!(session.idle_reduced);
        assert_eq!(session.destination.desired_quantity_target(), (3, 3));
    }

    #[tokio::test(start_paused = true)]
    async fn m136_repeated_idle_ticks_do_not_enqueue_duplicate_controls() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session.idle_policy.idle_time = Duration::from_millis(50);
        session.idle_policy.target_quantity = 1;
        session.idle_last_activity = MockRuntime::now();
        session.idle_timer = Some(MockRuntime::timer(Duration::from_millis(50)));

        tokio::time::advance(Duration::from_millis(60)).await;
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        session.poll_idle_timer(&mut cx);
        assert!(session.idle_reduced);
        let desired = session.destination.desired_quantity_target();

        // While still idle and at low target, further polls do nothing.
        tokio::time::advance(Duration::from_millis(500)).await;
        session.poll_idle_timer(&mut cx);
        session.poll_idle_timer(&mut cx);
        assert!(session.idle_reduced);
        assert_eq!(session.destination.desired_quantity_target(), desired);
        assert!(session.idle_timer.is_none());
    }

    #[tokio::test]
    async fn m136_shutdown_clears_timer_state() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        assert!(session.idle_timer.is_some());
        session.cancel_idle_state();
        assert!(session.idle_timer.is_none());

        // Socket close path also clears.
        let (mut session2, _ctx2) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session2.socket = None;
        session2.cancel_idle_state();
        assert!(session2.idle_timer.is_none());
    }

    #[tokio::test]
    async fn m136_replacement_generation_ignores_stale_state() {
        let (first, _c1) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        let (second, _c2) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        assert_ne!(first.idle_generation, second.idle_generation);
        // Fresh generation starts active, never inherits reduced/timer.
        assert!(!second.idle_reduced);
        assert!(second.idle_timer.is_some());
        assert_eq!(second.destination.desired_quantity_target(), (3, 3));
    }

    #[tokio::test]
    async fn m136_shared_member_activity_aggregates_and_release_does_not_reset() {
        let (mut session, _ctx) = create_idle_session(idle_options(&[
            ("i2cp.reduceOnIdle", "true"),
            ("i2cp.reduceIdleTime", "300000"),
            ("i2cp.reduceQuantity", "1"),
        ]))
        .await;
        session.session_kind = SamSessionKind::Primary {
            sub_sessions: HashMap::new(),
        };
        let _ = session.on_create_sub_session("sub1".into(), SessionKind::Stream, HashMap::new());
        // Sub-session registration itself is not activity.
        assert!(!session.idle_reduced);
        assert_eq!(session.destination.desired_quantity_target(), (3, 3));

        // Activity from any member sharing the generation resets the one clock.
        session.note_qualifying_activity();
        assert!(!session.idle_reduced);
        assert!(session.idle_timer.is_some());
    }
}
