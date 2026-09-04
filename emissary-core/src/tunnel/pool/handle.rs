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
    error::ChannelError,
    i2np::Message,
    primitives::{Lease, RouterId, TunnelId},
    tunnel::pool::{context::TunnelMessageRecycle, TunnelMessage, TunnelPoolConfig},
};

use futures::Stream;
use futures_channel::oneshot;
use thingbuf::mpsc;

#[cfg(feature = "std")]
use parking_lot::RwLock;
#[cfg(feature = "no_std")]
use spin::rwlock::RwLock;

use alloc::{sync::Arc, vec::Vec};
use core::{
    fmt,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
};

/// Events emitted by a `TunnelPool`.
#[derive(Default, Debug, Clone)]
pub enum TunnelPoolEvent {
    /// Tunnel pool has been shut down.
    TunnelPoolShutDown,

    /// Inbound tunnel has been built.
    InboundTunnelBuilt {
        /// Tunnel ID.
        tunnel_id: TunnelId,

        /// `Lease2` of the inbound tunnel.
        lease: Lease,
    },

    /// Outbound tunnel has been built.
    OutboundTunnelBuilt {
        /// Tunnel ID.
        tunnel_id: TunnelId,
    },

    /// Inbound tunnel has been expired.
    InboundTunnelExpired {
        /// Tunnel ID.
        tunnel_id: TunnelId,
    },

    /// Outbound tunnel has been expired.
    OutboundTunnelExpired {
        /// Tunnel ID.
        tunnel_id: TunnelId,
    },

    /// Inbound tunnel is about to expire.
    #[allow(unused)]
    InboundTunnelExpiring {
        /// Tunnel ID.
        tunnel_id: TunnelId,
    },

    /// Outbound tunnel is about to expire.
    #[allow(unused)]
    OutboundTunnelExpiring {
        /// Tunnel ID.
        tunnel_id: TunnelId,
    },

    /// Message received into one of the inbound tunnels.
    Message {
        /// Received I2NP message.
        message: Message,
    },

    /// Dummy event.
    #[default]
    Dummy,
}

impl fmt::Display for TunnelPoolEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TunnelPoolShutDown => write!(f, "TunnelPoolEvent::TunnelPoolShutDown"),
            Self::InboundTunnelBuilt { .. } => write!(f, "TunnelPoolEvent::InboundTunnelBuilt"),
            Self::OutboundTunnelBuilt { .. } => write!(f, "TunnelPoolEvent::OutboundTunnelBuilt"),
            Self::InboundTunnelExpired { .. } => write!(f, "TunnelPoolEvent::InboundTunnelExpired"),
            Self::OutboundTunnelExpired { .. } => {
                write!(f, "TunnelPoolEvent::OutboundTunnelExpired")
            }
            Self::InboundTunnelExpiring { .. } => {
                write!(f, "TunnelPoolEvent::InboundTunnelExpiring")
            }
            Self::OutboundTunnelExpiring { .. } => {
                write!(f, "TunnelPoolEvent::OutboundTunnelExpiring")
            }
            Self::Message { .. } => write!(f, "TunnelPoolEvent::Message"),
            Self::Dummy => write!(f, "TunnelPoolEvent::Dummy"),
        }
    }
}

/// Tunnel message sender.
#[derive(Clone)]
pub struct TunnelMessageSender(mpsc::Sender<TunnelMessage, TunnelMessageRecycle>);

impl TunnelMessageSender {
    /// Create [`TunnelSender`] with `message`.
    ///
    /// [`TunnelSender`] allows the sender to construct a tunnel message of correct kind
    /// (router/tunnel delivery) and send it either in blocking or non-blocking manner.
    pub fn send_message(&self, message: Vec<u8>) -> TunnelSender<'_> {
        TunnelSender {
            kind: None,
            message,
            outbound_tunnel: None,
            tx: &self.0,
        }
    }
}

/// Delivery kind.
enum DeliveryKind {
    /// Tunnel delivery.
    TunnelDelivery {
        /// ID of the IBGW tunenl.
        tunnel_id: TunnelId,

        /// ID of the IBGW router.
        router_id: RouterId,
    },

    /// Router delivery.
    RouterDelivery {
        /// ID of the router.
        router_id: RouterId,
    },
}

/// Tunnel sender builder for a single message.
pub struct TunnelSender<'a> {
    /// Delivery kind.
    kind: Option<DeliveryKind>,

    /// Message.
    message: Vec<u8>,

    /// Outbound tunnel over which the message should be sent, if specified.
    ///
    /// If not specified, a random tunnel of the pool is used for delivery.
    outbound_tunnel: Option<TunnelId>,

    /// TX channel for sending the message.
    tx: &'a mpsc::Sender<TunnelMessage, TunnelMessageRecycle>,
}

impl TunnelSender<'_> {
    /// Send message to router identified by `router_id`.
    pub fn router_delivery(mut self, router_id: RouterId) -> Self {
        self.kind = Some(DeliveryKind::RouterDelivery { router_id });
        self
    }

    /// Send message to tunnel identified by (`router_id`, `tunnel_id`) tuple (IBGW).
    pub fn tunnel_delivery(mut self, router_id: RouterId, tunnel_id: TunnelId) -> Self {
        self.kind = Some(DeliveryKind::TunnelDelivery {
            tunnel_id,
            router_id,
        });
        self
    }

    /// Specify the ID of the outbound tunnel over which the messages should be sent.
    ///
    /// If not specified, a random outbound tunnel is selected for delivery.
    pub fn via_outbound_tunnel(mut self, tunnel_id: TunnelId) -> Self {
        self.outbound_tunnel = Some(tunnel_id);
        self
    }

    /// Attempt to send message to tunnel pool for delivery and return and error if the channel is
    /// full or closed.
    pub fn try_send(self) -> Result<(), ChannelError> {
        let message = match self.kind.expect("to exist") {
            DeliveryKind::TunnelDelivery {
                tunnel_id,
                router_id,
            } => TunnelMessage::TunnelDeliveryViaRoute {
                router_id,
                tunnel_id,
                outbound_tunnel: self.outbound_tunnel,
                message: self.message,
            },
            DeliveryKind::RouterDelivery { router_id } => TunnelMessage::RouterDeliveryViaRoute {
                router_id,
                outbound_tunnel: self.outbound_tunnel,
                message: self.message,
            },
        };

        self.tx.try_send(message).map_err(From::from)
    }

    /// Attempt to send message to tunnel pool for delivery and return and error if the channel is
    /// closed
    ///
    /// The function blocks until there's enough capacity in the channel to send the message.
    #[allow(unused)]
    pub async fn send(self) -> Result<(), ChannelError> {
        let message = match self.kind.expect("to exist") {
            DeliveryKind::TunnelDelivery {
                tunnel_id,
                router_id,
            } => TunnelMessage::TunnelDeliveryViaRoute {
                router_id,
                tunnel_id,
                outbound_tunnel: self.outbound_tunnel,
                message: self.message,
            },
            DeliveryKind::RouterDelivery { router_id } => TunnelMessage::RouterDeliveryViaRoute {
                router_id,
                outbound_tunnel: self.outbound_tunnel,
                message: self.message,
            },
        };

        self.tx.send(message).await.map_err(|_| ChannelError::Closed)
    }
}

/// Maximum value accepted for a desired inbound/outbound tunnel quantity target.
///
/// The bound covers the SAM `inbound.quantity`/`outbound.quantity` range
/// (`1..=16`) plus `0` for single-direction pools that keep one direction
/// disabled. Larger values are rejected before any pool or lease-set state
/// changes.
#[allow(dead_code)]
pub const MAX_DESIRED_TUNNEL_QUANTITY: usize = 16;

/// Error returned when a desired tunnel quantity target cannot be accepted.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityTargetError {
    /// One or both quantities are outside `0..=MAX_DESIRED_TUNNEL_QUANTITY`.
    InvalidQuantity,
    /// The owning pool is shut down; the update was not applied.
    PoolShutDown,
}

impl fmt::Display for QuantityTargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidQuantity => write!(f, "invalid tunnel quantity target"),
            Self::PoolShutDown => write!(f, "tunnel pool is shut down"),
        }
    }
}

impl core::error::Error for QuantityTargetError {}

/// Next destination-scoped pool generation.
///
/// Each `TunnelPoolHandle`/pool pair captures one value. A stale handle
/// addresses only its own generation-local control cell and can never reach
/// a replacement pool's cell.
static NEXT_QUANTITY_GENERATION: AtomicU64 = AtomicU64::new(1);

fn next_quantity_generation() -> u64 {
    // Wrap is harmless: equality with the paired cell is what matters and a
    // collision would require 2^64 live pools.
    NEXT_QUANTITY_GENERATION.fetch_add(1, Ordering::Relaxed)
}

/// Generation-local desired quantity cell shared by one pool and its owner.
///
/// Single-slot latest-state storage: concurrent updates coalesce and the
/// newest `(inbound, outbound)` pair always wins. No queue, no task, no timer
/// is created per update.
#[derive(Debug)]
struct QuantityTargetInner {
    /// Generation captured at pool creation.
    generation: u64,
    /// Current desired active inbound quantity.
    desired_inbound: usize,
    /// Current desired active outbound quantity.
    desired_outbound: usize,
    /// Set once the pool is shut down; further updates are rejected.
    closed: bool,
    /// Waker stored by the pool task for prompt maintenance wake-up.
    waker: Option<Waker>,
}

/// Shared owner-to-pool quantity control.
///
/// Cloned between the handle and its pool only. Distinct pools never share a
/// cell, which gives per-destination isolation by construction.
#[derive(Debug, Clone)]
pub(crate) struct QuantityTargetControl {
    inner: Arc<RwLock<QuantityTargetInner>>,
}

impl QuantityTargetControl {
    fn new(generation: u64, inbound: usize, outbound: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(QuantityTargetInner {
                generation,
                desired_inbound: inbound,
                desired_outbound: outbound,
                closed: false,
                waker: None,
            })),
        }
    }

    fn validate(inbound: usize, outbound: usize) -> Result<(), QuantityTargetError> {
        if inbound > MAX_DESIRED_TUNNEL_QUANTITY || outbound > MAX_DESIRED_TUNNEL_QUANTITY {
            return Err(QuantityTargetError::InvalidQuantity);
        }
        Ok(())
    }

    /// Apply `(inbound, outbound)` if the cell generation matches and the
    /// cell is not closed. Returns the waker to wake outside the lock.
    fn apply(
        &self,
        generation: u64,
        inbound: usize,
        outbound: usize,
    ) -> Result<Option<Waker>, QuantityTargetError> {
        Self::validate(inbound, outbound)?;

        let mut inner = self.inner.write();
        if inner.generation != generation || inner.closed {
            return Err(QuantityTargetError::PoolShutDown);
        }
        inner.desired_inbound = inbound;
        inner.desired_outbound = outbound;
        Ok(inner.waker.take())
    }

    fn current(&self, generation: u64) -> Option<(usize, usize)> {
        let inner = self.inner.read();
        (inner.generation == generation && !inner.closed)
            .then_some((inner.desired_inbound, inner.desired_outbound))
    }

    pub(crate) fn mark_closed(&self) {
        let waker = {
            let mut inner = self.inner.write();
            inner.closed = true;
            inner.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }

    pub(crate) fn store_waker(&self, waker: &Waker) {
        let mut inner = self.inner.write();
        if !inner.closed {
            inner.waker = Some(waker.clone());
        }
    }

    /// Synchronized desired target for `generation`, if live.
    pub(crate) fn synchronized(&self, generation: u64) -> Option<(usize, usize)> {
        self.current(generation)
    }
}

/// Tunnel pool handle.
///
/// Allows `Destination`s to communicate with their `TunnelPool`.
pub struct TunnelPoolHandle {
    /// Tunnel pool configuration.
    config: TunnelPoolConfig,

    /// RX channel for receiving events from `TunnelPool`.
    event_rx: mpsc::Receiver<TunnelPoolEvent>,

    /// Implementation of [`TunnelSender`].
    sender: TunnelMessageSender,

    /// TX channel for sending a shutdown command to `TunnelPool`.
    #[allow(unused)]
    shutdown_tx: Option<oneshot::Sender<()>>,

    /// Generation-local desired quantity control shared with the pool.
    quantity_control: QuantityTargetControl,

    /// Generation captured at creation; must match the pool's generation.
    quantity_generation: u64,
}

impl TunnelPoolHandle {
    /// Create new [`TunnelPoolHandle`].
    pub(super) fn new(
        config: TunnelPoolConfig,
        message_tx: mpsc::Sender<TunnelMessage, TunnelMessageRecycle>,
    ) -> (Self, mpsc::Sender<TunnelPoolEvent>, oneshot::Receiver<()>) {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (event_tx, event_rx) = mpsc::channel(64);
        let quantity_generation = next_quantity_generation();
        let quantity_control = QuantityTargetControl::new(
            quantity_generation,
            config.num_inbound,
            config.num_outbound,
        );

        (
            Self {
                config,
                event_rx,
                sender: TunnelMessageSender(message_tx),
                shutdown_tx: Some(shutdown_tx),
                quantity_control,
                quantity_generation,
            },
            event_tx,
            shutdown_rx,
        )
    }

    /// Send shutdown signal to `TunnelPool`.
    ///
    /// [`TunnelPoolEvent::TunnelPoolShutDown`] is emitted before `TunnelPool` is shut down.
    pub fn shutdown(&mut self) {
        self.quantity_control.mark_closed();
        self.shutdown_tx.take().map(|tx| tx.send(()));
    }

    /// Get reference to [`TunnelPoolConfig`] of the tunnel pool.
    pub fn config(&self) -> &TunnelPoolConfig {
        &self.config
    }

    /// Configured/base inbound and outbound quantities (immutable).
    ///
    /// M135 neutral primitive; consumed by the destination bridge now and by
    /// the future session policy owner. Allowed dead in builds without that
    /// consumer.
    #[allow(dead_code)]
    pub fn base_quantity_target(&self) -> (usize, usize) {
        (self.config.num_inbound, self.config.num_outbound)
    }

    /// Current desired inbound and outbound quantities.
    ///
    /// Returns the base quantities when the pool is closed.
    #[allow(dead_code)]
    pub fn desired_quantity_target(&self) -> (usize, usize) {
        self.quantity_control
            .current(self.quantity_generation)
            .unwrap_or_else(|| self.base_quantity_target())
    }

    /// Generation identifying this handle/pool pair.
    #[cfg(test)]
    pub fn quantity_generation(&self) -> u64 {
        self.quantity_generation
    }

    /// Update the current desired inbound/outbound quantities atomically.
    ///
    /// Both values travel as one latest-state update: concurrent callers
    /// coalesce and the newest pair wins, so a restore can never be lost to
    /// queue saturation. Existing tunnels are untouched; the pool converges
    /// through normal expiry/failure without building above the new target.
    /// Returns an explicit error without changing any state when the
    /// quantities are out of bounds or the pool is closed.
    #[allow(dead_code)]
    pub fn set_quantity_target(
        &self,
        inbound: usize,
        outbound: usize,
    ) -> Result<(), QuantityTargetError> {
        let waker = self.quantity_control.apply(self.quantity_generation, inbound, outbound)?;
        if let Some(waker) = waker {
            waker.wake();
        }
        Ok(())
    }

    /// Restore desired quantities to the configured/base values.
    #[allow(dead_code)]
    pub fn restore_quantity_target(&self) -> Result<(), QuantityTargetError> {
        let (inbound, outbound) = self.base_quantity_target();
        self.set_quantity_target(inbound, outbound)
    }

    /// Clone the generation-local control cell for the pool actor.
    pub(crate) fn quantity_control(&self) -> (QuantityTargetControl, u64) {
        (self.quantity_control.clone(), self.quantity_generation)
    }

    /// Create [`TunnelSender`] with `message`.
    ///
    /// Note that this function doesn't send the message but creates a sender which the caller
    /// can use to construct a message with correct delivery style.
    pub fn send_message(&self, message: Vec<u8>) -> TunnelSender<'_> {
        self.sender.send_message(message)
    }

    /// Get a copy of [`TunnelMessageSender`].
    pub fn sender(&self) -> TunnelMessageSender {
        self.sender.clone()
    }

    /// Create new [`TunnelPoolHandle`] for testing.
    #[cfg(test)]
    pub fn create() -> (
        Self,
        mpsc::Receiver<TunnelMessage, TunnelMessageRecycle>,
        mpsc::Sender<TunnelPoolEvent>,
        oneshot::Receiver<()>,
    ) {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (event_tx, event_rx) = mpsc::channel(64);
        let (message_tx, message_rx) = mpsc::with_recycle(64, TunnelMessageRecycle::default());
        let config = TunnelPoolConfig::default();
        let quantity_generation = next_quantity_generation();

        (
            Self {
                event_rx,
                sender: TunnelMessageSender(message_tx),
                shutdown_tx: Some(shutdown_tx),
                quantity_control: QuantityTargetControl::new(
                    quantity_generation,
                    config.num_inbound,
                    config.num_outbound,
                ),
                quantity_generation,
                config,
            },
            message_rx,
            event_tx,
            shutdown_rx,
        )
    }

    #[cfg(test)]
    /// Create new [`TunnelPoolHandle`] from `config`
    pub fn from_config(
        config: TunnelPoolConfig,
    ) -> (
        Self,
        mpsc::Receiver<TunnelMessage, TunnelMessageRecycle>,
        mpsc::Sender<TunnelPoolEvent>,
        oneshot::Receiver<()>,
    ) {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (event_tx, event_rx) = mpsc::channel(64);
        let (message_tx, message_rx) = mpsc::with_recycle(64, TunnelMessageRecycle::default());
        let quantity_generation = next_quantity_generation();

        (
            Self {
                event_rx,
                sender: TunnelMessageSender(message_tx),
                shutdown_tx: Some(shutdown_tx),
                quantity_control: QuantityTargetControl::new(
                    quantity_generation,
                    config.num_inbound,
                    config.num_outbound,
                ),
                quantity_generation,
                config,
            },
            message_rx,
            event_tx,
            shutdown_rx,
        )
    }
}

impl Stream for TunnelPoolHandle {
    type Item = TunnelPoolEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.event_rx.poll_recv(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_to_router_via_any() {
        let (tx, rx) = mpsc::with_recycle(64, TunnelMessageRecycle::default());
        let sender = TunnelMessageSender(tx);

        let remote = RouterId::random();

        sender
            .send_message(vec![1, 3, 3, 7])
            .router_delivery(remote.clone())
            .send()
            .await
            .unwrap();

        match rx.recv().await.unwrap() {
            TunnelMessage::RouterDeliveryViaRoute {
                router_id,
                outbound_tunnel,
                message,
            } => {
                assert_eq!(router_id, remote);
                assert_eq!(message, vec![1, 3, 3, 7]);
                assert!(outbound_tunnel.is_none());
            }
            _ => panic!("invalid message"),
        }
    }

    #[tokio::test]
    async fn send_to_tunnel_via_any() {
        let (tx, rx) = mpsc::with_recycle(64, TunnelMessageRecycle::default());
        let sender = TunnelMessageSender(tx);

        let remote_router = RouterId::random();
        let remote_tunnel = TunnelId::random();

        sender
            .send_message(vec![1, 3, 3, 7])
            .tunnel_delivery(remote_router.clone(), remote_tunnel)
            .send()
            .await
            .unwrap();

        match rx.recv().await.unwrap() {
            TunnelMessage::TunnelDeliveryViaRoute {
                router_id,
                tunnel_id,
                outbound_tunnel,
                message,
            } => {
                assert_eq!(router_id, remote_router);
                assert_eq!(tunnel_id, remote_tunnel);
                assert_eq!(message, vec![1, 3, 3, 7]);
                assert!(outbound_tunnel.is_none());
            }
            _ => panic!("invalid message"),
        }
    }

    #[tokio::test]
    async fn send_to_router_via_route() {
        let (tx, rx) = mpsc::with_recycle(64, TunnelMessageRecycle::default());
        let sender = TunnelMessageSender(tx);

        let remote = RouterId::random();
        let obgw = TunnelId::random();

        sender
            .send_message(vec![1, 3, 3, 7])
            .router_delivery(remote.clone())
            .via_outbound_tunnel(obgw)
            .send()
            .await
            .unwrap();

        match rx.recv().await.unwrap() {
            TunnelMessage::RouterDeliveryViaRoute {
                router_id,
                outbound_tunnel,
                message,
            } => {
                assert_eq!(router_id, remote);
                assert_eq!(message, vec![1, 3, 3, 7]);
                assert_eq!(outbound_tunnel, Some(obgw));
            }
            _ => panic!("invalid message"),
        }
    }

    #[tokio::test]
    async fn send_to_tunnel_via_route() {
        let (tx, rx) = mpsc::with_recycle(64, TunnelMessageRecycle::default());
        let sender = TunnelMessageSender(tx);

        let remote_router = RouterId::random();
        let remote_tunnel = TunnelId::random();
        let obgw = TunnelId::random();

        sender
            .send_message(vec![1, 3, 3, 7])
            .tunnel_delivery(remote_router.clone(), remote_tunnel)
            .via_outbound_tunnel(obgw)
            .send()
            .await
            .unwrap();

        match rx.recv().await.unwrap() {
            TunnelMessage::TunnelDeliveryViaRoute {
                router_id,
                tunnel_id,
                outbound_tunnel,
                message,
            } => {
                assert_eq!(router_id, remote_router);
                assert_eq!(tunnel_id, remote_tunnel);
                assert_eq!(message, vec![1, 3, 3, 7]);
                assert_eq!(outbound_tunnel, Some(obgw));
            }
            _ => panic!("invalid message"),
        }
    }

    // M135 §8.1: desired targets initialize to configured quantities.
    #[test]
    fn m135_desired_targets_initialize_to_base() {
        let config = TunnelPoolConfig {
            num_inbound: 3,
            num_outbound: 2,
            ..Default::default()
        };
        let (handle, _msg_rx, _event_tx, _shutdown_rx) =
            TunnelPoolHandle::from_config(config.clone());

        assert_eq!(handle.base_quantity_target(), (3, 2));
        assert_eq!(handle.desired_quantity_target(), (3, 2));
        assert_eq!(handle.config().num_inbound, 3);
        assert_eq!(handle.config().num_outbound, 2);
    }

    // M135 §8.2 (handle half): lowering target changes desired without
    // mutating base config.
    #[test]
    fn m135_lowering_preserves_base_config() {
        let (handle, _msg_rx, _event_tx, _shutdown_rx) = TunnelPoolHandle::from_config(
            TunnelPoolConfig {
                num_inbound: 3,
                num_outbound: 3,
                ..Default::default()
            },
        );

        handle.set_quantity_target(1, 1).expect("valid target");
        assert_eq!(handle.desired_quantity_target(), (1, 1));
        assert_eq!(handle.base_quantity_target(), (3, 3));
        assert_eq!(handle.config().num_inbound, 3);
        assert_eq!(handle.config().num_outbound, 3);
    }

    // M135 §8.8 (handle half): restore resumes base quantities.
    #[test]
    fn m135_restore_returns_to_base() {
        let (handle, _msg_rx, _event_tx, _shutdown_rx) = TunnelPoolHandle::from_config(
            TunnelPoolConfig {
                num_inbound: 3,
                num_outbound: 2,
                ..Default::default()
            },
        );

        handle.set_quantity_target(1, 1).unwrap();
        assert_eq!(handle.desired_quantity_target(), (1, 1));
        handle.restore_quantity_target().expect("restore succeeds");
        assert_eq!(handle.desired_quantity_target(), (3, 2));
        assert_eq!(handle.base_quantity_target(), (3, 2));
    }

    // M135 §8.18/§9: closed control is rejected; invalid quantities fail
    // before any state change.
    #[test]
    fn m135_invalid_and_shut_down_rejected() {
        let (handle, _msg_rx, _event_tx, _shutdown_rx) = TunnelPoolHandle::from_config(
            TunnelPoolConfig {
                num_inbound: 2,
                num_outbound: 2,
                ..Default::default()
            },
        );

        assert_eq!(
            handle.set_quantity_target(MAX_DESIRED_TUNNEL_QUANTITY + 1, 1),
            Err(QuantityTargetError::InvalidQuantity)
        );
        assert_eq!(
            handle.set_quantity_target(1, MAX_DESIRED_TUNNEL_QUANTITY + 1),
            Err(QuantityTargetError::InvalidQuantity)
        );
        // Failed validation leaves desired untouched.
        assert_eq!(handle.desired_quantity_target(), (2, 2));

        let (mut closed_handle, _msg_rx, _event_tx, _shutdown_rx) =
            TunnelPoolHandle::from_config(TunnelPoolConfig {
                num_inbound: 2,
                num_outbound: 2,
                ..Default::default()
            });
        closed_handle.shutdown();
        assert_eq!(
            closed_handle.set_quantity_target(1, 1),
            Err(QuantityTargetError::PoolShutDown)
        );
        assert_eq!(
            closed_handle.restore_quantity_target(),
            Err(QuantityTargetError::PoolShutDown)
        );
        // Closed handle reports base quantities without adopting the update.
        assert_eq!(closed_handle.desired_quantity_target(), (2, 2));
    }

    // M135 §8.12/§5: one destination cannot alter another destination's
    // target; generations are unique per handle.
    #[test]
    fn m135_quantity_targets_isolated_per_handle() {
        let (first, _m1, _e1, _s1) = TunnelPoolHandle::from_config(TunnelPoolConfig {
            num_inbound: 3,
            num_outbound: 3,
            ..Default::default()
        });
        let (second, _m2, _e2, _s2) = TunnelPoolHandle::from_config(TunnelPoolConfig {
            num_inbound: 3,
            num_outbound: 3,
            ..Default::default()
        });

        assert_ne!(first.quantity_generation(), second.quantity_generation());
        first.set_quantity_target(1, 1).unwrap();
        assert_eq!(first.desired_quantity_target(), (1, 1));
        assert_eq!(second.desired_quantity_target(), (3, 3));
    }

    // M135 §8.19: bounded single-slot control coalesces; the newest pair
    // (here a restore) always wins.
    #[test]
    fn m135_latest_update_wins() {
        let (handle, _msg_rx, _event_tx, _shutdown_rx) = TunnelPoolHandle::from_config(
            TunnelPoolConfig {
                num_inbound: 3,
                num_outbound: 3,
                ..Default::default()
            },
        );

        handle.set_quantity_target(1, 2).unwrap();
        handle.set_quantity_target(2, 1).unwrap();
        handle.restore_quantity_target().unwrap();
        assert_eq!(handle.desired_quantity_target(), (3, 3));
    }

    // M135 §8.20/§11: changed core surface carries no administrative
    // policy vocabulary in runtime strings.
    #[test]
    fn m135_api_carries_no_policy_vocabulary() {
        for text in [
            format!("{}", QuantityTargetError::InvalidQuantity),
            format!("{}", QuantityTargetError::PoolShutDown),
            format!("{:?}", QuantityTargetError::InvalidQuantity),
            format!("{:?}", TunnelPoolConfig::default()),
        ] {
            for forbidden in [
                "Proposal",
                "I2PControl",
                "TunnelManager",
                "JsonRpc",
                "jsonrpc",
                "SAM",
            ] {
                assert!(
                    !text.contains(forbidden),
                    "runtime string contains forbidden term {forbidden}: {text}"
                );
            }
        }
        // The neutral API spelling itself avoids policy terms.
        for name in [
            stringify!(QuantityTargetError),
            stringify!(MAX_DESIRED_TUNNEL_QUANTITY),
        ] {
            assert!(!name.contains("Proposal"));
            assert!(!name.contains("I2PControl"));
        }
    }
}
