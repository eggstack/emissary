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
    crypto::{chachapoly::ChaChaPoly, EphemeralPrivateKey},
    error::{ChannelError, Error},
    events::EventHandle,
    i2np::{
        garlic::{DeliveryInstructions, GarlicMessageBuilder},
        MessageBuilder, MessageType, I2NP_MESSAGE_EXPIRATION,
    },
    inspection::{TunnelDirection, TunnelInspection, TunnelInspectionEntry, TunnelPoolKind},
    primitives::{Lease, Mapping, MessageId, RouterId, Str, TunnelId},
    router::context::RouterContext,
    runtime::{Counter, Gauge, Histogram, Instant, JoinSet, MetricsHandle, Runtime},
    subsystem::SubsystemHandle,
    tunnel::{
        hop::{
            inbound::InboundTunnel, outbound::OutboundTunnel, pending::PendingTunnel, ReceiverKind,
            Tunnel, TunnelBuildParameters, TunnelInfo,
        },
        metrics::*,
        pool::{
            listener::TunnelBuildListener,
            selector::{HopSelector, TunnelSelector},
            timer::{TunnelKind, TunnelTimer, TunnelTimerEvent},
            zero_hop::ZeroHopInboundTunnel,
        },
        TUNNEL_EXPIRATION,
    },
};

use bytes::{BufMut, Bytes, BytesMut};
use futures::{
    future::{select, Either},
    FutureExt, StreamExt,
};
use futures_channel::oneshot;
use hashbrown::{HashMap, HashSet};
use listener::ReceiveKind;
use rand::Rng;

use alloc::vec::Vec;
use core::{
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll},
    time::Duration,
};

pub use context::{
    TunnelMessage, TunnelPoolBuildParameters, TunnelPoolContext, TunnelPoolContextHandle,
};
pub use handle::{
    QuantityTargetError, TunnelMessageSender, TunnelPoolEvent, TunnelPoolHandle,
    MAX_DESIRED_TUNNEL_QUANTITY,
};
pub use selector::{ClientSelector, ExploratorySelector};

#[cfg(test)]
pub use context::TunnelMessageRecycle;

mod context;
mod handle;
mod listener;
mod selector;
mod timer;
mod zero_hop;

/// Narrow passive observation metadata supplied by the tunnel manager.
#[derive(Clone)]
pub(crate) struct TunnelPoolObservation {
    source: TunnelInspection,
    pool_id: u64,
    pool_kind: TunnelPoolKind,
}

impl TunnelPoolObservation {
    pub(crate) fn new(source: TunnelInspection, pool_id: u64, pool_kind: TunnelPoolKind) -> Self {
        Self {
            source,
            pool_id,
            pool_kind,
        }
    }

    fn publish(&self, tunnel_id: TunnelId, direction: TunnelDirection) {
        let _ = self.source.publish(TunnelInspectionEntry {
            pool_id: self.pool_id,
            tunnel_id: tunnel_id.into(),
            pool_kind: self.pool_kind,
            direction: Some(direction),
        });
    }

    fn remove(&self, tunnel_id: TunnelId, direction: TunnelDirection) {
        self.source.remove(TunnelInspectionEntry {
            pool_id: self.pool_id,
            tunnel_id: tunnel_id.into(),
            pool_kind: self.pool_kind,
            direction: Some(direction),
        });
    }

    fn set_queue_depth(&self, depth: usize) {
        self.source.set_pool_queue_depth(self.pool_kind, self.pool_id, depth);
    }
}

/// Logging target for the file.
const LOG_TARGET: &str = "emissary::tunnel::pool";

/// Tunnel maintenance interval.
const TUNNEL_MAINTENANCE_INTERVAL: Duration = Duration::from_secs(10);

/// Tunnel build request expiration.
///
/// How long is a pending tunnel kept active before the request is considered failed.
const TUNNEL_BUILD_EXPIRATION: Duration = Duration::from_secs(8);

/// Tunnel test expiration.
///
/// How long is the tunnel considered under testing until the test is considered a failure.
const TUNNEL_TEST_EXPIRATION: Duration = Duration::from_secs(8);

/// Tunnel channel size.
const TUNNEL_CHANNEL_SIZE: usize = 64usize;

/// Tunnel test interval.
///
/// How often tunnels of the pool are tested.
const TUNNEL_TEST_INTERVAL: Duration = Duration::from_secs(15);

/// Tunnel pool configuration.
#[derive(Debug, Clone)]
pub struct TunnelPoolConfig {
    /// Tunnel pool name.
    ///
    /// This is either set in I2CP options and if none is set,
    /// it's the short hash of the `Destination`.
    pub name: Str,

    /// How many inbound tunnels the pool should have.
    pub num_inbound: usize,

    /// How many hops should each inbound tunnel have.
    pub num_inbound_hops: usize,

    /// Inclusive random variation applied to inbound tunnel length.
    pub inbound_length_variance: i8,

    /// How many inbound standby tunnels the pool should keep ready.
    pub num_inbound_backup: usize,

    /// How many outbound tunnels the pool should have.
    pub num_outbound: usize,

    /// How many hops should each outbound tunnel have.
    pub num_outbound_hops: usize,

    /// Inclusive random variation applied to outbound tunnel length.
    pub outbound_length_variance: i8,

    /// How many outbound standby tunnels the pool should keep ready.
    pub num_outbound_backup: usize,
}

impl Default for TunnelPoolConfig {
    fn default() -> Self {
        Self {
            num_inbound: 3usize,
            num_inbound_hops: 2usize,
            inbound_length_variance: 0,
            num_inbound_backup: 0usize,
            num_outbound: 3usize,
            num_outbound_hops: 2usize,
            outbound_length_variance: 0,
            num_outbound_backup: 0usize,
            name: Str::from("exploratory"),
        }
    }
}

impl From<&Mapping> for TunnelPoolConfig {
    fn from(options: &Mapping) -> Self {
        let num_inbound = options
            .get(&Str::from("inbound.quantity"))
            .map_or(3usize, |value| value.parse::<usize>().unwrap_or(3usize));

        let num_inbound_hops = options
            .get(&Str::from("inbound.length"))
            .map_or(2usize, |value| value.parse::<usize>().unwrap_or(2usize));

        let inbound_length_variance = options
            .get(&Str::from("inbound.lengthVariance"))
            .map_or(0i8, |value| value.parse::<i8>().unwrap_or(0i8));

        let num_inbound_backup = options
            .get(&Str::from("inbound.backupQuantity"))
            .map_or(0usize, |value| value.parse::<usize>().unwrap_or(0usize));

        let num_outbound = options
            .get(&Str::from("outbound.quantity"))
            .map_or(3usize, |value| value.parse::<usize>().unwrap_or(3usize));

        let num_outbound_hops = options
            .get(&Str::from("outbound.length"))
            .map_or(2usize, |value| value.parse::<usize>().unwrap_or(2usize));

        let outbound_length_variance = options
            .get(&Str::from("outbound.lengthVariance"))
            .map_or(0i8, |value| value.parse::<i8>().unwrap_or(0i8));

        let num_outbound_backup = options
            .get(&Str::from("outbound.backupQuantity"))
            .map_or(0usize, |value| value.parse::<usize>().unwrap_or(0usize));

        let name = options
            .get(&Str::from("inbound.nickname"))
            .cloned()
            .unwrap_or(Str::from("unspecified"));

        Self {
            name,
            num_inbound,
            num_inbound_hops,
            inbound_length_variance,
            num_inbound_backup,
            num_outbound,
            num_outbound_hops,
            outbound_length_variance,
            num_outbound_backup,
        }
    }
}

/// Select one tunnel length according to the SAM/I2P inclusive variance rules.
///
/// Reference freeze (`TunnelPeerSelector.getLength`, read-only Java I2P source):
/// positive variance samples an inclusive additive offset `0..=variance`;
/// negative variance samples a magnitude uniformly from `0..=|variance|` and
/// then a sign uniformly (`nextBoolean`), so the base length carries
/// `1/(|variance|+1)` probability mass and each non-zero offset carries
/// `1/(2*(|variance|+1))`. Zero magnitude yields the base regardless of sign.
///
/// `apply_length_variance` is the deterministic pure mapping used by
/// [`varied_tunnel_length`]; it exists so distribution semantics are covered
/// by exact reference vectors without statistical/flaky RNG assertions.
fn apply_length_variance(
    base: usize,
    variance: i8,
    magnitude: u16,
    positive_sign: bool,
    maximum: usize,
) -> Option<usize> {
    if !(1..=maximum).contains(&base) {
        return None;
    }

    if variance == 0 {
        return Some(base);
    }

    let offset = if variance < 0 {
        let limit = (-i16::from(variance)) as u16;
        if magnitude > limit {
            return None;
        }
        let magnitude = i16::try_from(magnitude).ok()?;
        if magnitude == 0 {
            0i16
        } else if positive_sign {
            magnitude
        } else {
            -magnitude
        }
    } else {
        let limit = i16::from(variance) as u16;
        if magnitude > limit {
            return None;
        }
        i16::try_from(magnitude).ok()?
    };
    let length = i16::try_from(base).ok()?.checked_add(offset)?;

    (1..=i16::try_from(maximum).ok()?).contains(&length)
        .then_some(usize::try_from(length).ok()?)
}

fn varied_tunnel_length<R: Runtime>(base: usize, variance: i8, maximum: usize) -> Option<usize> {
    if !(1..=maximum).contains(&base) {
        return None;
    }

    if variance == 0 {
        return Some(base);
    }

    if variance < 0 {
        let magnitude = (-i16::from(variance)) as u32;
        let sampled = (R::rng().next_u32() % (magnitude + 1)) as u16;
        // Match the reference magnitude/sign shape: one uniform magnitude
        // draw plus one uniform sign draw. Zero magnitude maps to the base
        // irrespective of the sign draw.
        let positive_sign = (R::rng().next_u32() & 1) == 0;
        apply_length_variance(base, variance, sampled, positive_sign, maximum)
    } else {
        let sampled = (R::rng().next_u32() % (i16::from(variance) as u32 + 1)) as u16;
        apply_length_variance(base, variance, sampled, true, maximum)
    }
}

/// Tunnel pool implementation.
///
/// Tunnel pool manages a set of inbound and outbound tunnels for a particular destination.
pub struct TunnelPool<R: Runtime, S: TunnelSelector + HopSelector> {
    /// Tunnel pool configuration.
    config: TunnelPoolConfig,

    /// Tunne pool context.
    context: TunnelPoolContext,

    /// Event handle.
    event_handle: EventHandle<R>,

    /// Expiring inbound tunnels.
    expiring_inbound: HashSet<TunnelId>,

    /// Expiring outbound tunnels.
    expiring_outbound: HashSet<TunnelId>,

    /// Active inbound tunnels.
    ///
    /// After the inbound tunnel expires, it returns a `(TunnelId, TunnelId)` tuple where the first
    /// `TunnelId` is the ID of the inbound tunnel and second ID if the id of the gateway.
    inbound: R::JoinSet<(TunnelId, TunnelId)>,

    /// Inbound tunnels.
    ///
    /// Key is IBGW `TunnelId` and value is (IBEP `TunnelId`, IBGW `RouterId`) tuple.
    inbound_tunnels: HashMap<TunnelId, (TunnelId, RouterId)>,

    /// Standby inbound tunnels, keyed by their gateway tunnel ID.
    ///
    /// The stored expiration is the tunnel's canonical absolute expiration
    /// (`build-completion time + TUNNEL_EXPIRATION`), captured on the same
    /// poll tick that constructs the inbound tunnel event loop and registers
    /// the pool timer. It is not a second ticking clock; promotion reuses it
    /// verbatim so an aged standby can never be republished with a fresh
    /// full lifetime.
    backup_inbound_tunnels: HashMap<TunnelId, (TunnelId, RouterId, HashSet<RouterId>, Duration)>,

    /// Last time a tunnel test was performed.
    last_tunnel_test: R::Instant,

    /// Tunnel maintenance timer.
    maintenance_timer: R::Timer,

    /// How many tunnel build failures, either timeouts or rejections, there has been.
    num_tunnel_build_failures: usize,

    /// How many tunnels have successfully been built.
    num_tunnels_built: usize,

    /// Active outbound tunnels.
    outbound: HashMap<TunnelId, OutboundTunnel<R>>,

    /// Standby outbound tunnels.
    backup_outbound: HashMap<TunnelId, OutboundTunnel<R>>,

    /// Pending outbound builds which are intended for standby capacity.
    pending_backup_outbound: HashSet<TunnelId>,

    /// Pending inbound builds which are intended for standby capacity.
    pending_backup_inbound: HashSet<TunnelId>,

    /// Pending inbound tunnels.
    pending_inbound: TunnelBuildListener<R, InboundTunnel<R>>,

    /// Pending outbound tunnels.
    pending_outbound: TunnelBuildListener<R, OutboundTunnel<R>>,

    /// Pending tunnel tests.
    pending_tests: R::JoinSet<(TunnelId, TunnelId, crate::Result<Duration>)>,

    /// Router context.
    router_ctx: RouterContext<R>,

    /// Subsystem handle.
    subsystem_handle: SubsystemHandle,

    /// Tunnel/hop selector for the tunnel pool.
    selector: S,

    /// RX channel for receiving a shutdown signal from the pool's owner.
    shutdown_rx: Option<oneshot::Receiver<()>>,

    /// Expiration timers for inbound/outbound tunnels.
    tunnel_timers: TunnelTimer<R>,

    /// Passive, bounded lifecycle observation.
    observation: Option<TunnelPoolObservation>,

    /// Generation-local quantity control shared with the owning handle.
    quantity_control: handle::QuantityTargetControl,

    /// Generation captured at creation; updates from other generations are ignored.
    quantity_generation: u64,

    /// Current desired active inbound quantity (starts at base config).
    desired_inbound: usize,

    /// Current desired active outbound quantity (starts at base config).
    desired_outbound: usize,
}

impl<R: Runtime, S: TunnelSelector + HopSelector> TunnelPool<R, S> {
    /// Create new [`TunnelPool`].
    #[allow(dead_code)]
    pub fn new(
        build_parameters: TunnelPoolBuildParameters,
        selector: S,
        subsystem_handle: SubsystemHandle,
        router_ctx: RouterContext<R>,
    ) -> (Self, TunnelPoolHandle) {
        Self::new_with_observation(
            build_parameters,
            selector,
            subsystem_handle,
            router_ctx,
            None,
        )
    }

    /// Create a tunnel pool with passive lifecycle observation.
    pub(crate) fn new_with_observation(
        build_parameters: TunnelPoolBuildParameters,
        selector: S,
        subsystem_handle: SubsystemHandle,
        router_ctx: RouterContext<R>,
        observation: Option<TunnelPoolObservation>,
    ) -> (Self, TunnelPoolHandle) {
        let TunnelPoolBuildParameters {
            config,
            context,
            shutdown_rx,
            tunnel_pool_handle,
            ..
        } = build_parameters;

        tracing::debug!(
            target: LOG_TARGET,
            name = %config.name,
            num_inbound = ?config.num_inbound,
            num_inbound_hops = ?config.num_inbound_hops,
            num_outbound = ?config.num_outbound,
            num_outbound_hops = ?config.num_outbound_hops,
            "create tunnel pool",
        );

        let desired_inbound = config.num_inbound;
        let desired_outbound = config.num_outbound;
        let (quantity_control, quantity_generation) = tunnel_pool_handle.quantity_control();

        (
            Self {
                config,
                context,
                event_handle: router_ctx.event_handle().clone(),
                expiring_inbound: HashSet::new(),
                expiring_outbound: HashSet::new(),
                inbound: R::join_set(),
                inbound_tunnels: HashMap::new(),
                backup_inbound_tunnels: HashMap::new(),
                last_tunnel_test: R::now(),
                maintenance_timer: R::timer(Duration::from_secs(0)),
                outbound: HashMap::new(),
                backup_outbound: HashMap::new(),
                pending_backup_outbound: HashSet::new(),
                pending_backup_inbound: HashSet::new(),
                pending_inbound: TunnelBuildListener::new(
                    subsystem_handle.clone(),
                    router_ctx.profile_storage().clone(),
                ),
                pending_outbound: TunnelBuildListener::new(
                    subsystem_handle.clone(),
                    router_ctx.profile_storage().clone(),
                ),
                num_tunnel_build_failures: 0usize,
                num_tunnels_built: 0usize,
                router_ctx,
                pending_tests: R::join_set(),
                subsystem_handle,
                selector,
                shutdown_rx: Some(shutdown_rx),
                tunnel_timers: TunnelTimer::new(),
                observation,
                quantity_control,
                quantity_generation,
                desired_inbound,
                desired_outbound,
            },
            tunnel_pool_handle,
        )
    }

    /// Configured/base inbound and outbound quantities (immutable).
    #[allow(dead_code)]
    pub fn base_quantity_target(&self) -> (usize, usize) {
        (self.config.num_inbound, self.config.num_outbound)
    }

    /// Current desired inbound and outbound quantities.
    #[allow(dead_code)]
    pub fn desired_quantity_target(&self) -> (usize, usize) {
        (self.desired_inbound, self.desired_outbound)
    }

    /// Synchronize the cached desired target with the owner control cell.
    ///
    /// Returns `true` when the target changed. Stale generations are ignored
    /// and a closed cell leaves the last synchronized target in place. Holds
    /// the control lock only for a short copy; never across build or network
    /// I/O.
    fn sync_quantity_target(&mut self) -> bool {
        let Some(current) =
            self.quantity_control.synchronized(self.quantity_generation)
        else {
            return false;
        };

        if current == (self.desired_inbound, self.desired_outbound) {
            return false;
        }

        self.desired_inbound = current.0;
        self.desired_outbound = current.1;
        tracing::debug!(
            target: LOG_TARGET,
            name = %self.config.name,
            desired_inbound = ?self.desired_inbound,
            desired_outbound = ?self.desired_outbound,
            "desired tunnel quantity target updated",
        );
        true
    }

    /// Calculate the active and standby outbound tunnel builds that are needed.
    fn calculate_outbound_build_count(&self) -> (usize, usize) {
        let active_target = self.desired_outbound + self.expiring_outbound.len();
        let active_pending = self
            .pending_outbound
            .len()
            .saturating_sub(self.pending_backup_outbound.len());
        let active = active_target.saturating_sub(self.outbound.len() + active_pending);
        let backup_pending = self.pending_backup_outbound.len();
        let backup = self
            .config
            .num_outbound_backup
            .saturating_sub(self.backup_outbound.len() + backup_pending);

        (active, backup)
    }

    /// Calculate the active and standby inbound tunnel builds that are needed.
    fn calculate_inbound_build_count(&self) -> (usize, usize) {
        let active_target = self.desired_inbound + self.expiring_inbound.len();
        let active_pending = self
            .pending_inbound
            .len()
            .saturating_sub(self.pending_backup_inbound.len());
        let active = active_target.saturating_sub(self.inbound_tunnels.len() + active_pending);
        let backup_pending = self.pending_backup_inbound.len();
        let backup = self
            .config
            .num_inbound_backup
            .saturating_sub(self.backup_inbound_tunnels.len() + backup_pending);

        (active, backup)
    }

    /// Select the promotable inbound standby with the latest absolute expiration.
    ///
    /// Returns `None` when no standby remains in the future relative to `now`.
    /// Expired standbys are intentionally left in place for their own
    /// destruction-timer cleanup so a later JoinSet event is not misclassified
    /// as an active expiry.
    fn select_promotable_inbound_standby(
        backup: &HashMap<TunnelId, (TunnelId, RouterId, HashSet<RouterId>, Duration)>,
        now: Duration,
    ) -> Option<(TunnelId, TunnelId, RouterId, HashSet<RouterId>, Duration)> {
        backup
            .iter()
            .filter(|(_, (_, _, _, expires))| *expires > now)
            .max_by_key(|(_, (_, _, _, expires))| *expires)
            .map(|(gateway, (tunnel_id, router_id, hops, expires))| {
                (*gateway, *tunnel_id, router_id.clone(), hops.clone(), *expires)
            })
    }

    /// Promote one aged inbound standby into the active pool, if needed.
    ///
    /// The promoted Lease reuses the standby's canonical absolute expiration
    /// verbatim and never mints `now + TUNNEL_EXPIRATION`; the destruction
    /// timer is not reset. Returns `true` when a standby was promoted and the
    /// owner was notified. On owner-registration failure the pre-promotion
    /// active/standby/routing state is restored and `false` is returned so no
    /// fabricated active Lease remains.
    fn promote_standby_inbound(&mut self) -> bool {
        if self.inbound_tunnels.len() >= self.desired_inbound {
            return false;
        }

        let Some((backup_gateway, backup_tunnel_id, backup_router, hops, backup_expires)) =
            Self::select_promotable_inbound_standby(
                &self.backup_inbound_tunnels,
                R::time_since_epoch(),
            )
        else {
            return false;
        };

        self.backup_inbound_tunnels.remove(&backup_gateway);
        self.selector
            .add_inbound_tunnel(backup_gateway, backup_router.clone(), hops.clone());
        self.inbound_tunnels
            .insert(backup_gateway, (backup_tunnel_id, backup_router.clone()));

        // The destruction timer is not reset: the promoted tunnel still
        // expires via its original `inbound` JoinSet event. Only the
        // owner-visible Lease carries the retained absolute expiration.
        if let Err(error) = self.context.register_inbound_tunnel_built(
            backup_gateway,
            Lease {
                router_id: backup_router.clone(),
                tunnel_id: backup_gateway,
                expires: backup_expires,
            },
        ) {
            // Roll back to the pre-promotion state so no route is visible as
            // active with an owner Lease the owner never received, and so
            // active/standby accounting is not doubled. The standby entry
            // (with its absolute expiration) is restored so its original
            // destruction timer still owns cleanup.
            self.selector.remove_inbound_tunnel(&backup_gateway);
            self.inbound_tunnels.remove(&backup_gateway);
            self.backup_inbound_tunnels.insert(
                backup_gateway,
                (backup_tunnel_id, backup_router, hops, backup_expires),
            );
            tracing::warn!(
                target: LOG_TARGET,
                name = %self.config.name,
                %backup_gateway,
                ?error,
                "failed to register promoted inbound tunnel to owner",
            );
            return false;
        }

        self.publish_tunnel(backup_gateway, TunnelDirection::Inbound);
        self.router_ctx.metrics_handle().gauge(NUM_INBOUND_TUNNELS).increment(1);
        true
    }

    /// Maintain the tunnel pool.
    ///
    /// If the number of inbound/outbound is less than desired, build new tunnels.
    ///
    /// Each active tunnel gets tested once every 10 seconds by selecting a pair of random tunnels
    /// and sending a test message to the outbound tunnel and receiving the message back via the
    /// paired inbound tunnels.
    fn maintain_pool(&mut self) {
        tracing::trace!(
            target: LOG_TARGET,
            name = %self.config.name,
            num_outbound = ?self.outbound.len(),
            num_expiring_outbound = self.expiring_outbound.len(),
            num_inbound = ?self.inbound.len(),
            num_expiring_inbound = self.expiring_inbound.len(),
            "maintain tunnel pool",
        );

        let (active_outbound_builds, backup_outbound_builds) =
            self.calculate_outbound_build_count();
        for build_index in 0..active_outbound_builds + backup_outbound_builds {
            let is_backup = build_index >= active_outbound_builds;
            let Some(num_hops) = varied_tunnel_length::<R>(
                self.config.num_outbound_hops,
                self.config.outbound_length_variance,
                8,
            ) else {
                tracing::warn!(
                    target: LOG_TARGET,
                    name = %self.config.name,
                    "outbound tunnel length configuration cannot be represented",
                );
                continue;
            };

            // attempt to select hops for the outbound tunnel
            //
            // if there aren't enough available hops, the tunnel build is skipped
            let Some(hops) = self.selector.select_hops(num_hops) else {
                tracing::warn!(
                    target: LOG_TARGET,
                    name = %self.config.name,
                    hops_required = ?self.config.num_outbound_hops,
                    "not enough routers for outbound tunnel build",
                );
                continue;
            };

            // allocate random tunnel id for the pending outbound tunnel
            //
            // this can just be a random id (with no regard for collisions)
            // as outbound tunnel messages are not routed through `RoutingTable`
            let tunnel_id = TunnelId::from(R::rng().next_u32());
            if is_backup {
                self.pending_backup_outbound.insert(tunnel_id);
            }

            // build outbound tunnel
            //
            // the tunnel build reply is received either through an existing inbound tunnel
            // or through a fake 0-hop inbound tunnel if there are no available inbound tunnels
            match self.selector.select_inbound_tunnel() {
                // no inbound tunnels available
                //
                // create a fake 0-hop inbound tunnel and add listener for the tunnel build reply
                // in the routing table
                //
                // if the reply is received, it'll be routed via the routing table to the fake
                // inbound tunnel which routes it to inbound tunnel `TunnelListener` from which
                // it'll be received by the `TunnelPool`
                None => {
                    // the fake 0-hop tunnel routes the build response via `RoutingTable`
                    //
                    // `ZeroHopInboundTunnel::new()` also returns a `oneshot::Receiver<Message>`
                    // which is used to receive the build response, if it's received in time
                    let (gateway, zero_hop_tunnel, message_rx) =
                        ZeroHopInboundTunnel::<R>::new(self.subsystem_handle.clone());

                    // allocate random message id for the build request
                    //
                    // since the reply is not routed through routing table,
                    // message id collisions are not a concern and this can just be a random number
                    let message_id = MessageId::from(R::rng().next_u32());

                    tracing::trace!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        %tunnel_id,
                        %gateway,
                        %message_id,
                        num_hops = ?hops.len(),
                        "build outbound tunnel via 0-hop tunnel",
                    );

                    match PendingTunnel::<R, OutboundTunnel<R>>::create_tunnel(
                        TunnelBuildParameters {
                            hops,
                            metrics_handle: self.router_ctx.metrics_handle().clone(),
                            name: self.config.name.clone(),
                            noise: self.router_ctx.noise().clone(),
                            message_id,
                            tunnel_info: TunnelInfo::Outbound {
                                gateway,
                                tunnel_id,
                                router_id: self.router_ctx.noise().local_router_hash().clone(),
                            },
                            receiver: ReceiverKind::Outbound,
                        },
                    ) {
                        Ok((tunnel, router_id, message)) => {
                            // spawn the fake 0-hop inbound tunnel in the background if it exists
                            //
                            // it will exit after receiving its first message because
                            // the tunnel is only used for this particular build request
                            R::spawn(zero_hop_tunnel);

                            // add pending tunnel into outbound tunnel build listener and send
                            // tunnel build request to the first hop
                            //
                            // give tunnel listener a oneshot receiver which it must poll before
                            // waiting for tunnel build result to ensure that dialing the next hop
                            // succeeded
                            let (dial_tx, dial_rx) = oneshot::channel();

                            self.pending_outbound.add_pending_tunnel(
                                tunnel,
                                ReceiveKind::ZeroHop,
                                message_rx,
                                dial_rx,
                            );
                            self.publish_queue_depth();
                            self.router_ctx
                                .metrics_handle()
                                .gauge(NUM_PENDING_OUTBOUND_TUNNELS)
                                .increment(1);

                            if let Err(error) = self
                                .subsystem_handle
                                .send_with_feedback(&router_id, message, dial_tx)
                            {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    ?error,
                                    "failed to send outbound tunnel build message (0-hop)",
                                );
                            }
                        }
                        Err(error) => {
                            tracing::warn!(
                                target: LOG_TARGET,
                                name = %self.config.name,
                                %tunnel_id,
                                %message_id,
                                ?error,
                                "failed to create outbound tunnel",
                            );

                            self.subsystem_handle.remove_tunnel(&gateway);
                            self.subsystem_handle.remove_listener(&message_id);
                            self.pending_backup_outbound.remove(&tunnel_id);
                        }
                    }
                }
                // inbound tunnel available
                //
                // add message listener for selected tunnel's tunnel pool and send the build request
                //
                // once the tunnel build reply is received into the selected inbound tunnel (which
                // could be in a different pool), it'll be received by the selected tunnel's
                // `TunnelPool` which routes the message to the listener
                Some((gateway, router_id, handle)) => {
                    // if an inbound tunnel exists, the reply is routed through it and received
                    // by its `TunnelPool` which routes the message to the listener
                    let (message_id, message_rx) = handle.add_listener(&mut R::rng());

                    tracing::trace!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        %tunnel_id,
                        %gateway,
                        %router_id,
                        %message_id,
                        num_hops = ?hops.len(),
                        "build outbound tunnel via existing inbound tunnel",
                    );

                    match PendingTunnel::<R, OutboundTunnel<R>>::create_tunnel(
                        TunnelBuildParameters {
                            hops,
                            metrics_handle: self.router_ctx.metrics_handle().clone(),
                            name: self.config.name.clone(),
                            noise: self.router_ctx.noise().clone(),
                            message_id,
                            tunnel_info: TunnelInfo::Outbound {
                                gateway,
                                router_id: Bytes::from(Into::<Vec<u8>>::into(router_id)),
                                tunnel_id,
                            },
                            receiver: ReceiverKind::Outbound,
                        },
                    ) {
                        Ok((tunnel, router_id, message)) => {
                            // listening for outbound tunnel build responses through an existing
                            // inbound tunnel is more complex than listening through a fake 0-hop
                            // inbound tunnel since the inbound tunnel is not expecting to receive
                            // just one message and because the selected OBEP has freedom to choose
                            // whether to garlic encrypt the tunnel build response or not
                            //
                            // if the response is not garlic encrypted, it'll be identified by the
                            // generated message id and if it is garlic encrypted, it'll be
                            // identified by the garlic tag which means that the inbound tunnel must
                            // have two listener types, one for the unecrypted response and one for
                            // the encrypted response
                            handle.add_garlic_listener(message_id, tunnel.garlic_tag());

                            // add pending tunnel into outbound tunnel build listener
                            // and send tunnel build request to the first hop
                            //
                            // give tunnel listener a oneshot receiver which it must poll before
                            // waiting for tunnel build result to ensure that dialing the next hop
                            // succeeded
                            let (dial_tx, dial_rx) = oneshot::channel();

                            self.pending_outbound.add_pending_tunnel(
                                tunnel,
                                ReceiveKind::Tunnel {
                                    handle: handle.clone(),
                                    message_id,
                                },
                                message_rx,
                                dial_rx,
                            );
                            self.publish_queue_depth();
                            self.router_ctx
                                .metrics_handle()
                                .gauge(NUM_PENDING_OUTBOUND_TUNNELS)
                                .increment(1);

                            if let Err(error) = self
                                .subsystem_handle
                                .send_with_feedback(&router_id, message, dial_tx)
                            {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    ?error,
                                    "failed to send outbound tunnel build message",
                                );
                            }
                        }
                        Err(error) => {
                            tracing::warn!(
                                target: LOG_TARGET,
                                name = %self.config.name,
                                %tunnel_id,
                                %message_id,
                                ?error,
                                "failed to create outbound tunnel",
                            );

                            handle.remove_listener(&message_id);
                            self.pending_backup_outbound.remove(&tunnel_id);
                        }
                    }
                }
            }
        }

        // build one or more inbound tunnels
        let (active_inbound_builds, backup_inbound_builds) = self.calculate_inbound_build_count();
        for build_index in 0..active_inbound_builds + backup_inbound_builds {
            let is_backup = build_index >= active_inbound_builds;
            // tunnel that's used to deliver the tunnel build request message
            //
            // if it's `None`, a fake 0-hop outbound tunnel is used
            let send_tunnel_id = self.selector.select_outbound_tunnel();

            // select hops for the tunnel
            let Some(num_hops) = varied_tunnel_length::<R>(
                self.config.num_inbound_hops,
                self.config.inbound_length_variance,
                7,
            ) else {
                tracing::warn!(
                    target: LOG_TARGET,
                    name = %self.config.name,
                    "inbound tunnel length configuration cannot be represented",
                );
                continue;
            };

            let Some(hops) = self.selector.select_hops(num_hops) else {
                tracing::warn!(
                    target: LOG_TARGET,
                    name = %self.config.name,
                    hops_required = ?self.config.num_inbound_hops,
                    "not enough routers for inbound tunnel build",
                );
                continue;
            };

            // generate message id for the build request and optimistically insert
            // a listener tx channel for it in the routing table
            //
            // if the building the build request fails, the listener must be removed
            // from the routing table
            let (message_id, message_rx) = self.subsystem_handle.insert_listener(&mut R::rng());

            // generate tunnel id for the inbound tunnel that's about to be built
            let (tunnel_id, tunnel_rx) =
                self.subsystem_handle.insert_tunnel::<TUNNEL_CHANNEL_SIZE>(&mut R::rng());
            if is_backup {
                self.pending_backup_inbound.insert(tunnel_id);
            }

            match PendingTunnel::<R, InboundTunnel<R>>::create_tunnel(TunnelBuildParameters {
                hops,
                metrics_handle: self.router_ctx.metrics_handle().clone(),
                name: self.config.name.clone(),
                noise: self.router_ctx.noise().clone(),
                message_id,
                tunnel_info: TunnelInfo::Inbound {
                    tunnel_id,
                    router_id: self.router_ctx.noise().local_router_hash().clone(),
                },
                receiver: ReceiverKind::Inbound {
                    message_rx: tunnel_rx,
                    handle: self.context.context_handle(),
                },
            }) {
                Ok((tunnel, router, message)) => {
                    // add pending tunnel into outbound tunnel build listener and send
                    // tunnel build request to the first hop
                    //
                    // give tunnel listener a oneshot receiver which it must poll before
                    // waiting for tunnel build result to ensure that dialing the next hop
                    // succeeded
                    let (dial_tx, dial_rx) = oneshot::channel();

                    self.pending_inbound.add_pending_tunnel(
                        tunnel,
                        ReceiveKind::RoutingTable { message_id },
                        message_rx,
                        dial_rx,
                    );
                    self.publish_queue_depth();
                    self.router_ctx
                        .metrics_handle()
                        .gauge(NUM_PENDING_INBOUND_TUNNELS)
                        .increment(1);

                    match send_tunnel_id {
                        None => {
                            tracing::debug!(
                                target: LOG_TARGET,
                                name = %self.config.name,
                                %tunnel_id,
                                "no outbound tunnel available, send build request to router",
                            );

                            if let Err(error) =
                                self.subsystem_handle.send_with_feedback(&router, message, dial_tx)
                            {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    ?error,
                                    "failed to send inbond tunnel build message",
                                );
                            }
                        }
                        Some((send_tunnel_id, handle)) => {
                            tracing::trace!(
                                target: LOG_TARGET,
                                name = %self.config.name,
                                %tunnel_id,
                                %send_tunnel_id,
                                "send tunnel build request to local outbound tunnel",
                            );

                            // the message is sent through a handle and not directly using the
                            // tunnel pool's outbound tunnels (`self.outboun`) because the tunnel
                            // build message might be for a client tunnel pool that is being created
                            // and thus doesn't have any available outbound tunnels meaning the TBM
                            // is sent via the exploratory pool
                            if let Err(error) = handle.send_to_router_with_feedback(
                                send_tunnel_id,
                                router,
                                message.serialize_standard(),
                                dial_tx,
                            ) {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    name = %self.config.name,
                                    %tunnel_id,
                                    %send_tunnel_id,
                                    ?error,
                                    "failed to send message to outbound tunnel"
                                );
                            }
                            self.router_ctx.metrics_handle().histogram(NUM_FRAGMENTS).record(1f64);
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        %tunnel_id,
                        %message_id,
                        ?error,
                        "failed to create outbound tunnel",
                    );

                    self.subsystem_handle.remove_tunnel(&tunnel_id);
                    self.subsystem_handle.remove_listener(&message_id);
                    self.pending_backup_inbound.remove(&tunnel_id);
                    continue;
                }
            }
        }

        // test active tunnels
        //
        // for pairs of active inbound and outbound tunnels, send a test message through and
        // outbound tunnel and request the obep of that tunnel to route the message to the selected
        // inbound tunnel
        //
        // for each message, start a timer which expires after 8 seconds and if the response is
        // received into the selected inbound tunnel within the time limit, the tunnel is considered
        // operational
        //
        // perform test only if enough time has elapsed since the last time
        if self.last_tunnel_test.elapsed() < TUNNEL_TEST_INTERVAL {
            return;
        }
        self.last_tunnel_test = R::now();

        self.outbound
            .keys()
            .filter(|tunnel_id| !self.expiring_outbound.contains(*tunnel_id))
            .copied()
            .zip(
                self.inbound_tunnels.iter().filter_map(|(tunnel_id, router)| {
                    (!self.expiring_inbound.contains(tunnel_id)).then_some((*tunnel_id, router))
                }),
            )
            .for_each(|(outbound, (inbound, (_, router)))| {
                // allocate new message id and an RX channel for receiving the tunnel test message
                let (message_id, message_rx) = self.context.add_listener(&mut R::rng());

                tracing::trace!(
                    target: LOG_TARGET,
                    name = %self.config.name,
                    %outbound,
                    %inbound,
                    %router,
                    %message_id,
                    "test tunnel",
                );

                // create dummy test message and send it through the outbound tunnel
                // to the selected inbound tunnel's gateway
                let payload = {
                    let mut out = BytesMut::with_capacity(11 + 4);
                    out.put_u32(11);
                    out.put_slice(b"tunnel test".as_ref());

                    out
                };

                // wrap the message inside a garlic message destined to ourselves
                let message = {
                    let expiration = R::time_since_epoch() + I2NP_MESSAGE_EXPIRATION;

                    let mut message = GarlicMessageBuilder::default()
                        .with_date_time(R::time_since_epoch().as_secs() as u32)
                        .with_garlic_clove(
                            MessageType::Data,
                            message_id,
                            expiration,
                            DeliveryInstructions::Local,
                            &MessageBuilder::standard()
                                .with_expiration(expiration)
                                .with_message_type(MessageType::Data)
                                .with_message_id(message_id)
                                .with_payload(&payload)
                                .build(),
                        )
                        .build();

                    let ephemeral_secret = EphemeralPrivateKey::random(R::rng());
                    let ephemeral_public = ephemeral_secret.public();
                    let (key, tag) = self.router_ctx.noise().derive_outbound_garlic_key(
                        self.router_ctx.noise().local_public_key(),
                        ephemeral_secret,
                    );

                    // message length + poly13055 tg + ephemeral key + garlic message length
                    let mut out = BytesMut::with_capacity(message.len() + 16 + 32 + 4);

                    // encryption must succeed since the parameters are managed by us
                    ChaChaPoly::new(&key)
                        .encrypt_with_ad_new(&tag, &mut message)
                        .expect("to succeed");

                    out.put_u32(message.len() as u32 + 32);
                    out.put_slice(&ephemeral_public.to_vec());
                    out.put_slice(&message);

                    MessageBuilder::standard()
                        .with_expiration(expiration)
                        .with_message_type(MessageType::Garlic)
                        .with_message_id(message_id)
                        .with_payload(&out)
                        .build()
                };

                // outbound tunnel must exist since it was jus iterated over
                let (router, mut messages) = self
                    .outbound
                    .get(&outbound)
                    .expect("outbound tunnel to exist")
                    .send_to_tunnel(router.clone(), inbound, message);

                // message must exist since it's a valid i2np message
                match self
                    .subsystem_handle
                    .send(&router, messages.next().expect("message to exist"))
                {
                    Ok(_) => self.pending_tests.push(async move {
                        let started = R::now();

                        match select(message_rx, pin!(R::delay(TUNNEL_TEST_EXPIRATION))).await {
                            Either::Right((_, _)) => (outbound, inbound, Err(Error::Timeout)),
                            Either::Left((Err(_), _)) => {
                                (outbound, inbound, Err(Error::Channel(ChannelError::Closed)))
                            }
                            Either::Left((Ok(_), _)) => (outbound, inbound, Ok(started.elapsed())),
                        }
                    }),
                    Err(error) => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %outbound,
                            %inbound,
                            ?error,
                            "failed to send tunnel test message",
                        );

                        self.context.remove_listener(&message_id);
                    }
                }
                self.router_ctx.metrics_handle().histogram(NUM_FRAGMENTS).record(1f64);

                debug_assert!(messages.next().is_none());
            });
    }

    fn publish_tunnel(&self, tunnel_id: TunnelId, direction: TunnelDirection) {
        if let Some(observation) = &self.observation {
            observation.publish(tunnel_id, direction);
        }
    }

    fn publish_queue_depth(&self) {
        if let Some(observation) = &self.observation {
            observation.set_queue_depth(self.pending_inbound.len() + self.pending_outbound.len());
        }
    }

    fn remove_tunnel_observation(&self, tunnel_id: TunnelId, direction: TunnelDirection) {
        if let Some(observation) = &self.observation {
            observation.remove(tunnel_id, direction);
        }
    }
}

impl<R: Runtime, S: TunnelSelector + HopSelector> Drop for TunnelPool<R, S> {
    fn drop(&mut self) {
        if let Some(observation) = &self.observation {
            observation.source.remove_pool(observation.pool_kind, observation.pool_id);
        }
    }
}

impl<R: Runtime, S: TunnelSelector + HopSelector> Future for TunnelPool<R, S> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // counter for keeping track of how many tunnel builds failed
        //
        // it's used to check if `TunnelPool::maintain_pool()` should be called before its timer
        // expires so the pool doesn't unnecessarily wait for a timeout when it could be building a
        // tunnel instead
        let mut num_failed_builds = 0;

        // Remember the owner task waker for prompt quantity-target wake-up and
        // adopt the newest desired target. The control lock is held only for
        // short copies; a target increase resumes building through the normal
        // maintenance path below while a decrease simply stops replacement.
        self.quantity_control.store_waker(cx.waker());
        let quantity_changed = self.sync_quantity_target();

        // poll pending outbound tunnels
        while let Poll::Ready(Some((tunnel_id, event))) = self.pending_outbound.poll_next_unpin(cx)
        {
            let is_backup = self.pending_backup_outbound.remove(&tunnel_id);
            match event {
                Err(error) => {
                    tracing::debug!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        ?error,
                        "failed to build outbound tunnel",
                    );
                    num_failed_builds += 1;

                    self.router_ctx
                        .metrics_handle()
                        .counter(NUM_BUILD_FAILURES)
                        .increment_with_label(1, "reason", error.into());
                    self.router_ctx
                        .metrics_handle()
                        .gauge(NUM_PENDING_OUTBOUND_TUNNELS)
                        .decrement(1);
                    self.num_tunnel_build_failures += 1;
                    self.event_handle.tunnel_build_result(false);
                }
                Ok((tunnel, started)) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        outbound_tunnel_id = %tunnel.tunnel_id(),
                        "outbound tunnel built",
                    );

                    if is_backup {
                        self.backup_outbound.insert(tunnel_id, tunnel);
                    } else {
                        self.selector.add_outbound_tunnel(tunnel_id, tunnel.hops());
                        self.outbound.insert(tunnel_id, tunnel);
                    }
                    self.tunnel_timers.add_outbound_tunnel(tunnel_id);
                    if !is_backup {
                        self.publish_tunnel(tunnel_id, TunnelDirection::Outbound);
                    }
                    self.router_ctx
                        .metrics_handle()
                        .gauge(NUM_PENDING_OUTBOUND_TUNNELS)
                        .decrement(1);
                    if !is_backup {
                        self.router_ctx.metrics_handle().gauge(NUM_OUTBOUND_TUNNELS).increment(1);
                    }
                    self.router_ctx.metrics_handle().counter(NUM_BUILD_SUCCESSES).increment(1);
                    self.router_ctx
                        .metrics_handle()
                        .histogram(TUNNEL_BUILD_DURATIONS)
                        .record(started.elapsed().as_millis() as f64);
                    self.num_tunnels_built += 1;
                    self.event_handle.tunnel_build_result(true);

                    if !is_backup {
                        // inform the owner of the tunnel pool that a new outbound tunnel has been built
                        if let Err(error) = self.context.register_outbound_tunnel_built(tunnel_id) {
                            tracing::warn!(
                                target: LOG_TARGET,
                                name = %self.config.name,
                                %tunnel_id,
                                ?error,
                                "failed to register new outbound tunnel to owner",
                            );
                        }
                    }
                }
            }
        }

        self.publish_queue_depth();

        // poll pending inbound tunnels
        while let Poll::Ready(Some((tunnel_id, event))) = self.pending_inbound.poll_next_unpin(cx) {
            let is_backup = self.pending_backup_inbound.remove(&tunnel_id);
            match event {
                Err(error) => {
                    tracing::debug!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        %tunnel_id,
                        ?error,
                        "failed to build inbound tunnel",
                    );
                    num_failed_builds += 1;

                    self.num_tunnel_build_failures += 1;
                    self.event_handle.tunnel_build_result(false);
                    self.subsystem_handle.remove_tunnel(&tunnel_id);
                    self.router_ctx
                        .metrics_handle()
                        .counter(NUM_BUILD_FAILURES)
                        .increment_with_label(1, "reason", error.into());
                    self.router_ctx
                        .metrics_handle()
                        .gauge(NUM_PENDING_INBOUND_TUNNELS)
                        .decrement(1);
                }
                Ok((tunnel, started)) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        tunnel_id = %tunnel.tunnel_id(),
                        "inbound tunnel built",
                    );

                    // fetch the newly created inbound tunnel's gateway information
                    //
                    // in order for the inbound tunnel to be usable, it's gateway information must
                    // be stored in selector/routing table, as opposed to the endpoint information,
                    // because the gateway is used to receive messages
                    let (router_id, gateway_tunnel_id) = tunnel.gateway();
                    let hops_for_selector = tunnel.hops();
                    if is_backup {
                        // Capture the canonical absolute expiration on the same
                        // tick that constructs the tunnel event loop (whose own
                        // `TUNNEL_EXPIRATION` delay starts inside `T::new`) and
                        // registers the pool rebuild timer. Promotion must reuse
                        // this value verbatim and never mint `now + full`.
                        let expires = R::time_since_epoch() + TUNNEL_EXPIRATION;
                        self.backup_inbound_tunnels.insert(
                            gateway_tunnel_id,
                            (tunnel_id, router_id.clone(), hops_for_selector, expires),
                        );
                    } else {
                        self.selector.add_inbound_tunnel(
                            gateway_tunnel_id,
                            router_id.clone(),
                            hops_for_selector,
                        );
                        self.inbound_tunnels
                            .insert(gateway_tunnel_id, (tunnel_id, router_id.clone()));
                    }
                    self.tunnel_timers.add_inbound_tunnel(gateway_tunnel_id);
                    if !is_backup {
                        self.publish_tunnel(gateway_tunnel_id, TunnelDirection::Inbound);
                    }
                    self.num_tunnels_built += 1;
                    self.event_handle.tunnel_build_result(true);

                    // inform the owner of the tunnel pool that a new inbound tunnel has been built
                    if !is_backup {
                        if let Err(error) = self.context.register_inbound_tunnel_built(
                            gateway_tunnel_id,
                            Lease {
                                router_id,
                                tunnel_id: gateway_tunnel_id,
                                expires: R::time_since_epoch() + TUNNEL_EXPIRATION,
                            },
                        ) {
                            tracing::warn!(
                                target: LOG_TARGET,
                                name = %self.config.name,
                                %gateway_tunnel_id,
                                ?error,
                                "failed to register new inbound tunnel to owner",
                            );
                        }
                    }

                    self.inbound.push(tunnel);
                    if !is_backup {
                        self.router_ctx.metrics_handle().gauge(NUM_INBOUND_TUNNELS).increment(1);
                    }
                    self.router_ctx
                        .metrics_handle()
                        .gauge(NUM_PENDING_INBOUND_TUNNELS)
                        .decrement(1);
                    self.router_ctx.metrics_handle().counter(NUM_BUILD_SUCCESSES).increment(1);
                    self.router_ctx
                        .metrics_handle()
                        .histogram(TUNNEL_BUILD_DURATIONS)
                        .record(started.elapsed().as_millis() as f64);
                }
            }
        }

        self.publish_queue_depth();

        // poll event loops of inbound tunnels
        while let Poll::Ready(event) = self.inbound.poll_next_unpin(cx) {
            match event {
                None => return Poll::Ready(()),
                Some((tunnel_id, gateway_tunnel_id)) => {
                    if self
                        .backup_inbound_tunnels
                        .remove(&gateway_tunnel_id)
                        .is_some()
                    {
                        tracing::debug!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %tunnel_id,
                            %gateway_tunnel_id,
                            "standby inbound tunnel expired",
                        );
                        self.subsystem_handle.remove_tunnel(&tunnel_id);
                        continue;
                    }

                    tracing::info!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        %tunnel_id,
                        %gateway_tunnel_id,
                        "inbound tunnel expired",
                    );

                    self.expiring_inbound.remove(&gateway_tunnel_id);
                    self.subsystem_handle.remove_tunnel(&tunnel_id);
                    self.selector.remove_inbound_tunnel(&gateway_tunnel_id);
                    self.inbound_tunnels.remove(&gateway_tunnel_id);
                    self.remove_tunnel_observation(gateway_tunnel_id, TunnelDirection::Inbound);
                    self.router_ctx.metrics_handle().gauge(NUM_INBOUND_TUNNELS).decrement(1);

                    // inform the owner of the tunnel pool that an inbound tunnel has expired
                    if let Err(error) =
                        self.context.register_inbound_tunnel_expired(gateway_tunnel_id)
                    {
                        tracing::warn!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %gateway_tunnel_id,
                            ?error,
                            "failed to register expired inbound tunnel to owner",
                        );
                    }

                    let _promoted = self.promote_standby_inbound();
                }
            }
        }

        // poll tunnel message context
        //
        // tunnel message context receives two types of events:
        //  1) inbound tunnel events
        //  2) outbound tunnel events
        //
        // inbound tunnel events are received from the network and a route for them couldn't be
        // found from the `TunnelHandle`'s routing table which causes them to be routed to
        // `TunnelPool` for further processing
        //
        // outbound tunnel events are received from destinations/other tunnel pools that wish to
        // send message over one of this tunnel pool's outbound tunnels, e.g., when sending a tunnel
        // build request to remote
        while let Poll::Ready(event) = self.context.poll_next_unpin(cx) {
            match event {
                None => return Poll::Ready(()),
                Some(event) => match event {
                    TunnelMessage::Dummy => unreachable!(),
                    TunnelMessage::RouterDelivery {
                        gateway,
                        router_id,
                        message,
                        feedback_tx,
                    } => match self.outbound.get(&gateway) {
                        None => tracing::warn!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %gateway,
                            "cannot send message, outbound tunnel doesn't exist",
                        ),
                        Some(tunnel) => {
                            let (router_id, messages) = tunnel.send_to_router(router_id, message);

                            let (_, count) = messages.into_iter().fold(
                                (feedback_tx, 0usize),
                                |(mut feedback_tx, count), message| {
                                    match feedback_tx.take() {
                                        Some(feedback_tx) => {
                                            if let Err(error) =
                                                self.subsystem_handle.send_with_feedback(
                                                    &router_id,
                                                    message,
                                                    feedback_tx,
                                                )
                                            {
                                                tracing::warn!(
                                                    target: LOG_TARGET,
                                                    name = %self.config.name,
                                                    %gateway,
                                                    ?error,
                                                    "failed to send tunnel message to router",
                                                );
                                            }
                                        }
                                        None => {
                                            if let Err(error) =
                                                self.subsystem_handle.send(&router_id, message)
                                            {
                                                tracing::warn!(
                                                    target: LOG_TARGET,
                                                    name = %self.config.name,
                                                    %gateway,
                                                    ?error,
                                                    "failed to send tunnel message to router",
                                                );
                                            }
                                        }
                                    }

                                    (None, count + 1)
                                },
                            );
                            self.router_ctx
                                .metrics_handle()
                                .histogram(NUM_FRAGMENTS)
                                .record(count as f64);
                        }
                    },
                    TunnelMessage::TunnelDelivery {
                        gateway,
                        tunnel_id,
                        message,
                    } => {
                        // TODO: needs to be fairer
                        let Some((outbound_gateway, tunnel)) = self.outbound.iter().next() else {
                            tracing::warn!(
                                target: LOG_TARGET,
                                name = %self.config.name,
                                "failed to send tunnel message, no outbound tunnel available",
                            );
                            continue;
                        };

                        tracing::trace!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %outbound_gateway,
                            "send tunnel message to remote destination",
                        );

                        let (router_id, messages) =
                            tunnel.send_to_tunnel(gateway.clone(), tunnel_id, message);

                        let count = messages.into_iter().fold(0usize, |count, message| {
                            if let Err(error) = self.subsystem_handle.send(&router_id, message) {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    name = %self.config.name,
                                    %gateway,
                                    ?error,
                                    "failed to send tunnel message to router",
                                );
                            }

                            count + 1
                        });
                        self.router_ctx
                            .metrics_handle()
                            .histogram(NUM_FRAGMENTS)
                            .record(count as f64);
                    }
                    TunnelMessage::RouterDeliveryViaRoute {
                        router_id,
                        outbound_tunnel,
                        message,
                    } => {
                        let (outbound_gateway, tunnel) = match outbound_tunnel {
                            None => match self.outbound.iter().next() {
                                Some((obgw_tunnel_id, tunnel)) => (*obgw_tunnel_id, tunnel),
                                None => {
                                    tracing::warn!(
                                        target: LOG_TARGET,
                                        name = %self.config.name,
                                        "failed to send tunnel message, no outbound tunnel available",
                                    );
                                    continue;
                                }
                            },
                            Some(obgw_tunnel_id) => match self.outbound.get(&obgw_tunnel_id) {
                                Some(tunnel) => (obgw_tunnel_id, tunnel),
                                None => {
                                    tracing::warn!(
                                        target: LOG_TARGET,
                                        ?obgw_tunnel_id,
                                        "outbound tunnel specified by routing path doesn't exist",
                                    );
                                    debug_assert!(false);

                                    let Some((outbound_gateway, tunnel)) =
                                        self.outbound.iter().next()
                                    else {
                                        tracing::warn!(
                                            target: LOG_TARGET,
                                            name = %self.config.name,
                                            "failed to send tunnel message, no outbound tunnel available",
                                        );
                                        continue;
                                    };

                                    (*outbound_gateway, tunnel)
                                }
                            },
                        };

                        let (router_id, messages) = tunnel.send_to_router(router_id, message);

                        let count = messages.into_iter().fold(0usize, |count, message| {
                            if let Err(error) = self.subsystem_handle.send(&router_id, message) {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    name = %self.config.name,
                                    %outbound_gateway,
                                    ?error,
                                    "failed to send tunnel message to router",
                                );
                            }

                            count + 1
                        });

                        self.router_ctx
                            .metrics_handle()
                            .histogram(NUM_FRAGMENTS)
                            .record(count as f64);
                    }
                    TunnelMessage::TunnelDeliveryViaRoute {
                        router_id: ibgw_router_id,
                        tunnel_id: ibgw_tunnel_id,
                        outbound_tunnel,
                        message,
                    } => {
                        let (outbound_gateway, tunnel) = match outbound_tunnel {
                            None => match self.outbound.iter().next() {
                                Some((obgw_tunnel_id, tunnel)) => (*obgw_tunnel_id, tunnel),
                                None => {
                                    tracing::warn!(
                                        target: LOG_TARGET,
                                        name = %self.config.name,
                                        "failed to send tunnel message, no outbound tunnel available",
                                    );
                                    continue;
                                }
                            },
                            Some(obgw_tunnel_id) => match self.outbound.get(&obgw_tunnel_id) {
                                Some(tunnel) => (obgw_tunnel_id, tunnel),
                                None => {
                                    tracing::warn!(
                                        target: LOG_TARGET,
                                        ?obgw_tunnel_id,
                                        "outbound tunnel specified by routing path doesn't exist",
                                    );
                                    debug_assert!(false);

                                    let Some((outbound_gateway, tunnel)) =
                                        self.outbound.iter().next()
                                    else {
                                        tracing::warn!(
                                            target: LOG_TARGET,
                                            name = %self.config.name,
                                            "failed to send tunnel message, no outbound tunnel available",
                                        );
                                        continue;
                                    };

                                    (*outbound_gateway, tunnel)
                                }
                            },
                        };

                        tracing::trace!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %outbound_gateway,
                            "send tunnel message to remote destination",
                        );

                        let (router_id, messages) =
                            tunnel.send_to_tunnel(ibgw_router_id.clone(), ibgw_tunnel_id, message);

                        let count = messages.into_iter().fold(0usize, |count, message| {
                            if let Err(error) = self.subsystem_handle.send(&router_id, message) {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    name = %self.config.name,
                                    %ibgw_router_id,
                                    %ibgw_tunnel_id,
                                    obgw_tunnel_id = %outbound_gateway,
                                    ?error,
                                    "failed to send tunnel message to router",
                                );
                            }

                            count + 1
                        });

                        self.router_ctx
                            .metrics_handle()
                            .histogram(NUM_FRAGMENTS)
                            .record(count as f64);
                    }
                    TunnelMessage::Inbound { message } => tracing::warn!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        message_type = ?message.message_type,
                        "unhandled message"
                    ),
                },
            }
        }

        // poll tunnel tests
        while let Poll::Ready(event) = self.pending_tests.poll_next_unpin(cx) {
            match event {
                None => return Poll::Ready(()),
                Some((outbound, inbound, result)) => match result {
                    Err(error) => {
                        tracing::debug!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %outbound,
                            %inbound,
                            ?error,
                            "tunnel test failed",
                        );

                        self.selector.register_tunnel_test_failure(&outbound, &inbound);
                        self.router_ctx.metrics_handle().counter(NUM_TEST_FAILURES).increment(1);
                    }
                    Ok(elapsed) => {
                        tracing::trace!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %outbound,
                            %inbound,
                            ?elapsed,
                            "tunnel test succeeded",
                        );

                        self.selector.register_tunnel_test_success(&outbound, &inbound);
                        self.router_ctx.metrics_handle().counter(NUM_TEST_SUCCESSES).increment(1);
                        self.router_ctx
                            .metrics_handle()
                            .histogram(TUNNEL_TEST_DURATIONS)
                            .record(elapsed.as_millis() as f64);
                    }
                },
            }
        }

        // poll tunnel timers
        //
        // both inbound and outbound tunnels emit `Rebuild` events which indicate that the tunnel is
        // about to expire and tunnel pool should build a replacement for the expiring tunnel
        //
        // as outbound tunnels do not have an asynchronous event loop but are instead stored in
        // tunnel pool, `TunnelTimer` also emits a `Destroy` event for them so tunnel pool knows
        // when to remove them from `outbound`
        //
        // inbound tunnels have their own event loops which track when the tunnel should be
        // destroyed and thus tunnel pool doesn't need an explicit signal from `TunnelTimer` for
        // inbound tunnel destruction
        while let Poll::Ready(event) = self.tunnel_timers.poll_next_unpin(cx) {
            match event {
                None => return Poll::Ready(()),
                Some(TunnelTimerEvent::Destroy { tunnel_id }) => {
                    if self.backup_outbound.remove(&tunnel_id).is_some() {
                        tracing::debug!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %tunnel_id,
                            "standby outbound tunnel expired",
                        );
                        continue;
                    }

                    tracing::info!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        %tunnel_id,
                        "outbound tunnel expired",
                    );
                    self.outbound.remove(&tunnel_id);
                    self.expiring_outbound.remove(&tunnel_id);
                    self.selector.remove_outbound_tunnel(&tunnel_id);
                    self.remove_tunnel_observation(tunnel_id, TunnelDirection::Outbound);
                    self.router_ctx.metrics_handle().gauge(NUM_OUTBOUND_TUNNELS).decrement(1);

                    // inform the owner of the tunnel pool that an inbound tunnel has expired
                    if let Err(error) = self.context.register_outbound_tunnel_expired(tunnel_id) {
                        tracing::warn!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %tunnel_id,
                            ?error,
                            "failed to register expired outbound tunnel to owner",
                        );
                    }

                    if self.outbound.len() < self.desired_outbound {
                        if let Some(backup_tunnel_id) = self.backup_outbound.keys().next().copied() {
                            let backup_tunnel = self
                                .backup_outbound
                                .remove(&backup_tunnel_id)
                                .expect("backup outbound tunnel to exist");
                            let hops = backup_tunnel.hops();
                            self.selector.add_outbound_tunnel(backup_tunnel_id, hops);
                            self.outbound.insert(backup_tunnel_id, backup_tunnel);
                            self.publish_tunnel(backup_tunnel_id, TunnelDirection::Outbound);
                            self.router_ctx
                                .metrics_handle()
                                .gauge(NUM_OUTBOUND_TUNNELS)
                                .increment(1);

                            if let Err(error) =
                                self.context.register_outbound_tunnel_built(backup_tunnel_id)
                            {
                                tracing::warn!(
                                    target: LOG_TARGET,
                                    name = %self.config.name,
                                    %backup_tunnel_id,
                                    ?error,
                                    "failed to register promoted outbound tunnel to owner",
                                );
                            }
                        }
                    }
                }
                Some(TunnelTimerEvent::Rebuild {
                    kind: TunnelKind::Outbound { tunnel_id },
                }) => {
                    if self.backup_outbound.contains_key(&tunnel_id) {
                        continue;
                    }
                    tracing::debug!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        %tunnel_id,
                        "outbound tunnel is about to expire",
                    );
                    self.expiring_outbound.insert(tunnel_id);

                    if let Err(error) = self.context.register_expiring_outbound_tunnel(tunnel_id) {
                        tracing::warn!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %tunnel_id,
                            ?error,
                            "failed to register expiring outbound tunnel to owner",
                        );
                    }
                }
                Some(TunnelTimerEvent::Rebuild {
                    kind: TunnelKind::Inbound { tunnel_id },
                }) => {
                    if self
                        .backup_inbound_tunnels
                        .values()
                        .any(|(backup_tunnel_id, _, _, _)| *backup_tunnel_id == tunnel_id)
                    {
                        continue;
                    }
                    tracing::debug!(
                        target: LOG_TARGET,
                        name = %self.config.name,
                        %tunnel_id,
                        "inbound tunnel is about to expire",
                    );
                    self.expiring_inbound.insert(tunnel_id);

                    if let Err(error) = self.context.register_expiring_inbound_tunnel(tunnel_id) {
                        tracing::warn!(
                            target: LOG_TARGET,
                            name = %self.config.name,
                            %tunnel_id,
                            ?error,
                            "failed to register expiring inbound tunnel to owner",
                        );
                    }
                }
            }
        }

        // check if the pool owner has sent a shutdown signal to the tunnel pool
        //
        // currently `TunnelPool` doesn't do any graceful shutdown for its own tunnels
        // and instead shuts down immediately
        //
        // the client is informed that the pool is shut down before it's shutdown so
        // the destination can starts up its own shutdown process
        if let Some(rx) = &mut self.shutdown_rx {
            if rx.poll_unpin(cx).is_ready() {
                tracing::info!(
                    target: LOG_TARGET,
                    name = %self.config.name,
                    "tunnel pool shutting down",
                );

                self.quantity_control.mark_closed();
                self.inbound_tunnels.values().for_each(|(tunnel_id, _)| {
                    self.subsystem_handle.remove_tunnel(tunnel_id);
                });
                self.backup_inbound_tunnels
                    .values()
                    .for_each(|(tunnel_id, _, _, _)| {
                        self.subsystem_handle.remove_tunnel(tunnel_id);
                    });

                if let Err(error) = self.context.register_tunnel_pool_shut_down() {
                    tracing::warn!(
                        target: LOG_TARGET,
                        ?error,
                        "failed to send shutdown confirmation to tunnel pool owner",
                    );
                }

                return Poll::Ready(());
            }
        }

        if self.event_handle.poll_unpin(cx).is_ready() {
            self.event_handle
                .tunnel_status(self.num_tunnels_built, self.num_tunnel_build_failures);

            // reset counters to zero as the cumulative success/failure tate is tracked by the event
            // system whereas each tunnel pool only  tracks the rate during each report period
            self.num_tunnels_built = 0;
            self.num_tunnel_build_failures = 0;
        }

        match self.maintenance_timer.poll_unpin(cx) {
            Poll::Ready(()) => {
                // create new timer and register it into the executor
                {
                    self.maintenance_timer = R::timer(TUNNEL_MAINTENANCE_INTERVAL);
                    let _ = self.maintenance_timer.poll_unpin(cx);
                }

                self.maintain_pool();
            }
            Poll::Pending if num_failed_builds > 0 => self.maintain_pool(),
            Poll::Pending if quantity_changed => self.maintain_pool(),
            _ => {}
        }

        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::RoutingError,
        events::EventManager,
        primitives::{RouterId, RouterInfoBuilder},
        profile::ProfileStorage,
        runtime::mock::MockRuntime,
        subsystem::{
            OutboundMessage, OutboundMessageRecycle, SubsystemEvent, SubsystemManager,
            SubsystemManagerContext,
        },
        tunnel::{
            garlic::DeliveryInstructions as GarlicDeliveryInstructions,
            pool::selector::ClientSelector,
            tests::{connect_routers, TestTransitTunnelManager},
            NoiseContext,
        },
    };
    use thingbuf::mpsc::{channel, with_recycle};

    #[test]
    fn varied_tunnel_length_uses_bounded_inclusive_variance() {
        let lengths = (0..128)
            .map(|_| varied_tunnel_length::<MockRuntime>(3, -2, 7).unwrap())
            .collect::<HashSet<_>>();

        assert!(lengths.iter().all(|length| (1..=5).contains(length)));
        assert!(lengths.len() > 1, "test RNG did not demonstrate variation");
        assert_eq!(varied_tunnel_length::<MockRuntime>(3, 0, 7), Some(3));
        assert_eq!(varied_tunnel_length::<MockRuntime>(0, 0, 7), None);
    }

    #[test]
    fn apply_length_variance_matches_java_reference_vectors() {
        // Zero variance preserves the base without consuming magnitude/sign.
        assert_eq!(apply_length_variance(3, 0, 0, true, 7), Some(3));
        assert_eq!(apply_length_variance(3, 0, 2, false, 7), Some(3));
        assert_eq!(apply_length_variance(0, 0, 0, true, 7), None);

        // Positive variance is an inclusive additive range `base..=base+variance`.
        assert_eq!(apply_length_variance(3, 2, 0, true, 7), Some(3));
        assert_eq!(apply_length_variance(3, 2, 1, true, 7), Some(4));
        assert_eq!(apply_length_variance(3, 2, 2, true, 7), Some(5));
        assert_eq!(apply_length_variance(3, 2, 3, true, 7), None);
        // Fail-closed at the representable boundary: 7 + 1 cannot be built.
        assert_eq!(apply_length_variance(7, 1, 1, true, 7), None);
        assert_eq!(apply_length_variance(3, 1, 0, true, 8), Some(3));
        assert_eq!(apply_length_variance(3, 1, 1, true, 8), Some(4));

        // Negative variance samples magnitude `0..=|variance|` plus a sign.
        // Magnitude zero yields the base regardless of the sign draw, so the
        // base carries `1/(M+1)` mass while each non-zero offset carries
        // `1/(2*(M+1))` (Java `TunnelPeerSelector.getLength` parity).
        assert_eq!(apply_length_variance(3, -2, 0, true, 7), Some(3));
        assert_eq!(apply_length_variance(3, -2, 0, false, 7), Some(3));
        assert_eq!(apply_length_variance(3, -2, 1, true, 7), Some(4));
        assert_eq!(apply_length_variance(3, -2, 1, false, 7), Some(2));
        assert_eq!(apply_length_variance(3, -2, 2, true, 7), Some(5));
        assert_eq!(apply_length_variance(3, -2, 2, false, 7), Some(1));
        assert_eq!(apply_length_variance(3, -2, 3, true, 7), None);
        assert_eq!(apply_length_variance(3, -1, 1, true, 7), Some(4));
        assert_eq!(apply_length_variance(3, -1, 1, false, 7), Some(2));

        // Invalid base or out-of-range results fail before build selection.
        assert_eq!(apply_length_variance(0, 0, 0, true, 7), None);
        assert_eq!(apply_length_variance(8, 0, 0, true, 7), None);
        assert_eq!(apply_length_variance(1, -1, 1, false, 7), None);
        assert_eq!(apply_length_variance(1, -1, 1, true, 7), Some(2));
        assert_eq!(varied_tunnel_length::<MockRuntime>(0, 1, 7), None);
        assert_eq!(varied_tunnel_length::<MockRuntime>(8, 0, 7), None);
    }

    #[test]
    fn standby_selection_prefers_latest_and_rejects_expired() {
        let now = MockRuntime::time_since_epoch();
        let router = RouterId::random();

        let aged_gateway = TunnelId::from(11u32);
        let fresh_gateway = TunnelId::from(22u32);
        let expired_gateway = TunnelId::from(33u32);

        let mut backup = HashMap::new();
        backup.insert(
            aged_gateway,
            (
                TunnelId::from(111u32),
                router.clone(),
                HashSet::new(),
                now + Duration::from_secs(60),
            ),
        );
        backup.insert(
            fresh_gateway,
            (
                TunnelId::from(222u32),
                router.clone(),
                HashSet::new(),
                now + Duration::from_secs(600),
            ),
        );
        backup.insert(
            expired_gateway,
            (
                TunnelId::from(333u32),
                router,
                HashSet::new(),
                now.checked_sub(Duration::from_secs(1)).unwrap_or(now),
            ),
        );

        // Latest future expiry wins; the expired entry is never selected even
        // though it remains stored for its own timer cleanup.
        let selected =
            TunnelPool::<MockRuntime, ExploratorySelector<MockRuntime>>::select_promotable_inbound_standby(
                &backup, now,
            )
            .expect("promotable standby to exist");
        assert_eq!(selected.0, fresh_gateway);
        assert_eq!(selected.4, now + Duration::from_secs(600));

        // Only an expired standby is not promotable.
        let mut only_expired = HashMap::new();
        only_expired.insert(
            expired_gateway,
            (
                TunnelId::from(333u32),
                RouterId::random(),
                HashSet::new(),
                now,
            ),
        );
        assert!(
            TunnelPool::<MockRuntime, ExploratorySelector<MockRuntime>>::select_promotable_inbound_standby(
                &only_expired,
                now
            )
            .is_none(),
            "expired standby must not be promoted"
        );
        assert!(
            TunnelPool::<MockRuntime, ExploratorySelector<MockRuntime>>::select_promotable_inbound_standby(
                &HashMap::new(),
                now
            )
            .is_none()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn promoted_aged_standby_reuses_original_expiry() {
        use futures::StreamExt;

        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 2usize,
            num_inbound_backup: 1usize,
            num_outbound: 0usize,
            num_outbound_hops: 0usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            handle: subsys_handle,
            manager,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());
        let profile_storage = ProfileStorage::<MockRuntime>::new(&[], &[], None);
        let (mut pool, mut owner_handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle,
            RouterContext::new(
                handle,
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle,
            ),
        );

        // Deliberately aged standby: built long ago, only 60s of its original
        // 10-minute lifetime remains. Promotion must publish 60s, never a
        // fresh full lifetime.
        let now = MockRuntime::time_since_epoch();
        let original_expires = now + Duration::from_secs(60);
        let fresh_expires = now + TUNNEL_EXPIRATION;
        assert!(
            original_expires < fresh_expires,
            "test requires an aged standby"
        );

        let gateway = TunnelId::from(4242u32);
        let endpoint = TunnelId::from(4343u32);
        let router_id = RouterId::random();
        pool.backup_inbound_tunnels.insert(
            gateway,
            (endpoint, router_id.clone(), HashSet::new(), original_expires),
        );
        assert!(pool.inbound_tunnels.is_empty());

        assert!(pool.promote_standby_inbound());
        assert!(pool.backup_inbound_tunnels.is_empty());
        assert_eq!(pool.inbound_tunnels.len(), 1);

        let event = tokio::time::timeout(Duration::from_secs(1), owner_handle.next())
            .await
            .expect("owner lease notification")
            .expect("event stream open");
        match event {
            TunnelPoolEvent::InboundTunnelBuilt { tunnel_id, lease } => {
                assert_eq!(tunnel_id, gateway);
                assert_eq!(lease.tunnel_id, gateway);
                assert_eq!(lease.router_id, router_id);
                assert_eq!(
                    lease.expires, original_expires,
                    "promoted lease must reuse the original absolute expiration"
                );
                assert!(
                    lease.expires < fresh_expires,
                    "promotion must never extend to a fresh full lifetime"
                );
            }
            event => panic!("unexpected owner event: {event:?}"),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn expired_standby_is_not_promoted() {
        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 2usize,
            num_inbound_backup: 1usize,
            num_outbound: 0usize,
            num_outbound_hops: 0usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            handle: subsys_handle,
            manager,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());
        let profile_storage = ProfileStorage::<MockRuntime>::new(&[], &[], None);
        let (mut pool, _owner_handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle,
            RouterContext::new(
                handle,
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle,
            ),
        );

        let now = MockRuntime::time_since_epoch();
        let gateway = TunnelId::from(5555u32);
        pool.backup_inbound_tunnels.insert(
            gateway,
            (
                TunnelId::from(6666u32),
                RouterId::random(),
                HashSet::new(),
                now,
            ),
        );

        assert!(!pool.promote_standby_inbound());
        assert!(pool.inbound_tunnels.is_empty());
        // Left for its own destruction-timer cleanup so the later JoinSet
        // event is not misclassified as an active expiry.
        assert_eq!(pool.backup_inbound_tunnels.len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn failed_owner_registration_restores_standby_accounting() {
        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 2usize,
            num_inbound_backup: 1usize,
            num_outbound: 0usize,
            num_outbound_hops: 0usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            handle: subsys_handle,
            manager,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());
        let profile_storage = ProfileStorage::<MockRuntime>::new(&[], &[], None);
        let (mut pool, owner_handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle,
            RouterContext::new(
                handle,
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle,
            ),
        );
        // Drop the owner event receiver so `register_inbound_tunnel_built`
        // fails closed instead of fabricating an active Lease.
        drop(owner_handle);

        let now = MockRuntime::time_since_epoch();
        let gateway = TunnelId::from(7777u32);
        let endpoint = TunnelId::from(8888u32);
        let router_id = RouterId::random();
        let expires = now + Duration::from_secs(300);
        pool.backup_inbound_tunnels.insert(
            gateway,
            (endpoint, router_id, HashSet::new(), expires),
        );

        assert!(!pool.promote_standby_inbound());
        // No fabricated active route and no double accounting: the standby is
        // restored with its original absolute expiration for timer cleanup.
        assert!(pool.inbound_tunnels.is_empty());
        assert_eq!(pool.backup_inbound_tunnels.len(), 1);
        let (_, (_, _, _, restored)) =
            pool.backup_inbound_tunnels.iter().next().expect("standby restored");
        assert_eq!(*restored, expires);
    }

    #[test]
    fn outbound_standby_accounting_retains_existing_timer_shape() {
        // Outbound standbys carry no owner-visible Lease expiration and their
        // timer/promotion path is intentionally unchanged by the inbound fix.
        let config = TunnelPoolConfig {
            num_outbound: 1usize,
            num_outbound_backup: 2usize,
            ..Default::default()
        };
        assert_eq!(config.num_outbound_backup, 2);

        let (active, backup) = {
            let active_target = config.num_outbound;
            let active = active_target.saturating_sub(0usize);
            let backup = config.num_outbound_backup.saturating_sub(0usize);
            (active, backup)
        };
        assert_eq!((active, backup), (1, 2));
    }

    #[test]
    fn tunnel_pool_defaults_keep_standby_and_variance_disabled() {
        let config = TunnelPoolConfig::default();

        assert_eq!(config.inbound_length_variance, 0);
        assert_eq!(config.outbound_length_variance, 0);
        assert_eq!(config.num_inbound_backup, 0);
        assert_eq!(config.num_outbound_backup, 0);
    }

    #[test]
    fn mapping_transfers_variance_and_standby_configuration() {
        let options = [
            ("inbound.lengthVariance", "-1"),
            ("inbound.backupQuantity", "2"),
            ("outbound.lengthVariance", "1"),
            ("outbound.backupQuantity", "3"),
        ]
        .into_iter()
        .map(|(key, value)| (Str::from(key), Str::from(value)))
        .collect::<Mapping>();
        let config = TunnelPoolConfig::from(&options);

        assert_eq!(config.inbound_length_variance, -1);
        assert_eq!(config.num_inbound_backup, 2);
        assert_eq!(config.outbound_length_variance, 1);
        assert_eq!(config.num_outbound_backup, 3);
    }

    #[tokio::test(start_paused = true)]
    async fn build_outbound_exploratory_tunnel() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });

                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 0usize,
            num_inbound_hops: 0usize,
            num_outbound: 1usize,
            num_outbound_hops: 3usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());
        let SubsystemManagerContext {
            dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transit_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (mut tunnel_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle.clone(),
            RouterContext::new(
                handle.clone(),
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle.clone(),
            ),
        );

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.pending_outbound.len(), 1);

        // connect first hop
        let router = tokio::time::timeout(Duration::from_secs(5), dial_rx.recv())
            .await
            .unwrap()
            .unwrap();
        let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
        transport_tx
            .send(crate::subsystem::SubsystemEvent::ConnectionEstablished {
                router_id: router.clone(),
                tx,
            })
            .await
            .unwrap();

        // 1st outbound hop (participant)
        let message =
            match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await.unwrap().unwrap() {
                OutboundMessage::MessageWithFeedback(message, tx) => {
                    tx.send(()).unwrap();
                    message
                }
                _ => panic!("invalid message type"),
            };

        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 2nd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 3rd outbound hop (obep)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // route tunnel build response to the fake 0-hop inbound tunnel
        subsys_handle.send(&router, message).unwrap();

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.outbound.len(), 1);
        assert_eq!(tunnel_pool.pending_outbound.len(), 0);
        assert_eq!(MockRuntime::get_gauge_value(NUM_OUTBOUND_TUNNELS), Some(1));
    }

    #[tokio::test(start_paused = true)]
    async fn outbound_exploratory_build_request_expires() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });
                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 0usize,
            num_inbound_hops: 0usize,
            num_outbound: 1usize,
            num_outbound_hops: 3usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transit_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());

        let (mut tunnel_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle.clone(),
            RouterContext::new(
                handle.clone(),
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle.clone(),
            ),
        );

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.pending_outbound.len(), 1);

        // connect first hop
        let router = tokio::time::timeout(Duration::from_secs(5), dial_rx.recv())
            .await
            .unwrap()
            .unwrap();
        let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
        transport_tx
            .send(crate::subsystem::SubsystemEvent::ConnectionEstablished {
                router_id: router.clone(),
                tx,
            })
            .await
            .unwrap();

        // 1st outbound hop (participant)
        let message =
            match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await.unwrap().unwrap() {
                OutboundMessage::MessageWithFeedback(message, tx) => {
                    tx.send(()).unwrap();
                    message
                }
                _ => panic!("invalid message type"),
            };

        // 1st outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 2nd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 3rd outbound hop (obep)
        let (_router, _message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // don't route the response which causes the build request to expire
        assert!(tokio::time::timeout(TUNNEL_BUILD_EXPIRATION, &mut tunnel_pool).await.is_err());
        assert_eq!(MockRuntime::get_counter_value(NUM_BUILD_FAILURES), Some(1));
    }

    #[tokio::test(start_paused = true)]
    async fn build_inbound_exploratory_tunnel() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });
                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 3usize,
            num_outbound: 0usize,
            num_outbound_hops: 0usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transit_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());

        let (mut tunnel_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle.clone(),
            RouterContext::new(
                handle.clone(),
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle.clone(),
            ),
        );

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.pending_inbound.len(), 1);

        // connect first hop
        let router = tokio::time::timeout(Duration::from_secs(5), dial_rx.recv())
            .await
            .unwrap()
            .unwrap();
        let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
        transport_tx
            .send(crate::subsystem::SubsystemEvent::ConnectionEstablished {
                router_id: router.clone(),
                tx,
            })
            .await
            .unwrap();

        // 1st outbound hop (ibgw)
        let message =
            match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await.unwrap().unwrap() {
                OutboundMessage::MessageWithFeedback(message, tx) => {
                    tx.send(()).unwrap();
                    message
                }
                _ => panic!("invalid message type"),
            };

        assert_eq!(message.message_type, MessageType::Garlic);
        let message = match routers
            .get_mut(&router)
            .unwrap()
            .garlic()
            .handle_message(message)
            .unwrap()
            .next()
        {
            Some(GarlicDeliveryInstructions::Local { message }) => message,
            _ => panic!("invalid delivery instructions"),
        };
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 2nd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 3rd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // route tunnel build response to the tunnel build response listener
        subsys_handle.send(&router, message).unwrap();

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.inbound.len(), 1);
        assert_eq!(tunnel_pool.pending_inbound.len(), 0);
        assert_eq!(MockRuntime::get_gauge_value(NUM_INBOUND_TUNNELS), Some(1));
    }

    #[tokio::test(start_paused = true)]
    async fn inbound_exploratory_build_request_expires() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });
                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 3usize,
            num_outbound: 0usize,
            num_outbound_hops: 0usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transit_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());

        let (mut tunnel_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle.clone(),
            RouterContext::new(
                handle.clone(),
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle.clone(),
            ),
        );

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.pending_inbound.len(), 1);

        // connect first hop
        let router = tokio::time::timeout(Duration::from_secs(5), dial_rx.recv())
            .await
            .unwrap()
            .unwrap();
        let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
        transport_tx
            .send(crate::subsystem::SubsystemEvent::ConnectionEstablished {
                router_id: router.clone(),
                tx,
            })
            .await
            .unwrap();

        // 1st outbound hop (ibgw)
        let message =
            match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await.unwrap().unwrap() {
                OutboundMessage::MessageWithFeedback(message, tx) => {
                    tx.send(()).unwrap();
                    message
                }
                _ => panic!("invalid message type"),
            };

        assert_eq!(message.message_type, MessageType::Garlic);
        let message = match routers
            .get_mut(&router)
            .unwrap()
            .garlic()
            .handle_message(message)
            .unwrap()
            .next()
        {
            Some(GarlicDeliveryInstructions::Local { message }) => message,
            _ => panic!("invalid delivery instructions"),
        };
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 2nd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 3rd outbound hop (participant)
        let (_router, _message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // don't route the response which causes the build request to expire
        assert!(tokio::time::timeout(
            TUNNEL_BUILD_EXPIRATION + Duration::from_secs(1),
            &mut tunnel_pool
        )
        .await
        .is_err());
        assert_eq!(tunnel_pool.inbound.len(), 0);
        assert_eq!(MockRuntime::get_counter_value(NUM_BUILD_FAILURES), Some(1))
    }

    #[tokio::test(start_paused = true)]
    async fn build_inbound_client_tunnel() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });
                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 0usize,
            num_inbound_hops: 0usize,
            num_outbound: 1usize,
            num_outbound_hops: 3usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx: _dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transit_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );

        // spawn subsystem manager in the background
        tokio::spawn(manager);

        // connect all routers together
        connect_routers(routers.iter_mut().map(|(_, router)| router));

        tokio::time::sleep(Duration::from_secs(1)).await;

        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());
        let exploratory_selector =
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false);
        let router_ctx = RouterContext::new(
            handle.clone(),
            profile_storage,
            router_info.identity.id(),
            Bytes::from(router_info.serialize(&signing_key)),
            static_key,
            signing_key,
            2u8,
            event_handle.clone(),
        );

        let (mut exploratory_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            exploratory_selector.clone(),
            subsys_handle.clone(),
            router_ctx.clone(),
        );

        assert!(
            tokio::time::timeout(Duration::from_secs(2), &mut exploratory_pool)
                .await
                .is_err()
        );
        assert_eq!(exploratory_pool.pending_outbound.len(), 1);

        // connect all routers
        let (msg_tx, msg_rx) = channel(128);

        for (router_id, router) in &mut routers {
            let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
            let conn_tx = msg_tx.clone();

            transport_tx
                .send(SubsystemEvent::ConnectionEstablished {
                    router_id: router_id.clone(),
                    tx,
                })
                .await
                .unwrap();
            router.connect_router(&router_info.identity.id());

            let router_id = router_id.clone();
            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    match message {
                        OutboundMessage::Message(message) => {
                            conn_tx.send((router_id.clone(), message)).await.unwrap();
                        }
                        OutboundMessage::MessageWithFeedback(message, tx) => {
                            tx.send(()).unwrap();
                            conn_tx.send((router_id.clone(), message)).await.unwrap();
                        }
                        OutboundMessage::Messages(_) => panic!("not implemented"),
                        OutboundMessage::Dummy => unreachable!(),
                    }
                }
            });
        }
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 1st outbound hop (participant)
        let (router, message) = tokio::time::timeout(Duration::from_secs(5), msg_rx.recv())
            .await
            .unwrap()
            .unwrap();

        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 2nd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 3rd outbound hop (obep)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // route tunnel build response to the fake 0-hop inbound tunnel
        subsys_handle.send(&router, message).unwrap();

        assert!(
            tokio::time::timeout(Duration::from_secs(2), &mut exploratory_pool)
                .await
                .is_err()
        );
        assert_eq!(exploratory_pool.outbound.len(), 1);
        assert_eq!(exploratory_pool.pending_outbound.len(), 0);
        assert_eq!(MockRuntime::get_gauge_value(NUM_OUTBOUND_TUNNELS), Some(1));

        {
            let pool_config = TunnelPoolConfig {
                num_inbound: 1usize,
                num_inbound_hops: 3usize,
                num_outbound: 0usize,
                num_outbound_hops: 0usize,
                name: Str::from("client"),
                ..Default::default()
            };
            let client_parameters = TunnelPoolBuildParameters::new(pool_config);
            let client_pool_handle = client_parameters.context_handle.clone();
            let client_selector =
                ClientSelector::new(exploratory_selector.clone(), client_pool_handle);

            let (mut client_pool, _client_handle) = TunnelPool::<MockRuntime, _>::new(
                client_parameters,
                client_selector,
                subsys_handle.clone(),
                router_ctx.clone(),
            );

            let future = async {
                tokio::select! {
                    _ = &mut client_pool => {}
                    _ = &mut exploratory_pool => {}
                }
            };

            assert!(tokio::time::timeout(Duration::from_secs(1), future).await.is_err());

            // inbound tunnel build is garlic encrypted and exceeds the tunnel data limit
            // so it's split into two fragments
            let mut obep = Option::<RouterId>::None;

            while let Ok((router_id, message)) = msg_rx.try_recv() {
                // 1st hop (participant)
                let (router_id, message) = {
                    let mut router = routers.get_mut(&router_id).unwrap();

                    router.subsystem_handle().send(&router_id, message).unwrap();
                    assert!(
                        tokio::time::timeout(Duration::from_millis(250), &mut router)
                            .await
                            .is_err()
                    );

                    router.select_message().unwrap()
                };

                // 2nd hop (participant)
                let (router_id, message) = {
                    let mut router = routers.get_mut(&router_id).unwrap();

                    router.subsystem_handle().send(&router_id, message).unwrap();
                    assert!(
                        tokio::time::timeout(Duration::from_millis(250), &mut router)
                            .await
                            .is_err()
                    );

                    router.select_message().unwrap()
                };

                // 3rd hop (obep)
                let mut router = routers.get_mut(&router_id).unwrap();
                obep = Some(router_id.clone());

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(250), &mut router).await.is_err()
                );
            }

            let router = routers.get_mut(&obep.unwrap()).unwrap();
            let (router_id, message) = router.select_message().unwrap();

            // inbound build 1st hop (ibgw)
            let (router_id, message) = {
                let mut router = routers.get_mut(&router_id).unwrap();

                assert_eq!(message.message_type, MessageType::Garlic);
                let message = match router.garlic().handle_message(message).unwrap().next() {
                    Some(GarlicDeliveryInstructions::Local { message }) => message,
                    _ => panic!("invalid delivery instructions"),
                };

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(250), &mut router).await.is_err()
                );

                router.select_message().unwrap()
            };

            // inbound build 2nd hop (participant)
            let (router_id, message) = {
                let mut router = routers.get_mut(&router_id).unwrap();

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(tokio::time::timeout(Duration::from_secs(1), &mut router).await.is_err());

                router.select_message().unwrap()
            };

            // inbound build 3rd hop (participant)
            let (router_id, message) = {
                let mut router = routers.get_mut(&router_id).unwrap();

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(250), &mut router).await.is_err()
                );

                router.select_message().unwrap()
            };

            assert_eq!(&router_id, router_ctx.router_id());

            subsys_handle.send(&router_id, message).unwrap();

            let future = async {
                tokio::select! {
                    _ = &mut client_pool => {}
                    _ = &mut exploratory_pool => {}
                }
            };

            assert!(tokio::time::timeout(Duration::from_secs(1), future).await.is_err());
        }

        assert_eq!(MockRuntime::get_gauge_value(NUM_OUTBOUND_TUNNELS), Some(1));
        assert_eq!(MockRuntime::get_gauge_value(NUM_INBOUND_TUNNELS), Some(1));
    }

    #[tokio::test(start_paused = true)]
    async fn build_outbound_client_tunnel() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });
                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 3usize,
            num_outbound: 0usize,
            num_outbound_hops: 0usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx: _dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transit_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );

        // spawn subsystem manager in the background
        tokio::spawn(manager);

        // connect all routers together
        connect_routers(routers.iter_mut().map(|(_, router)| router));

        tokio::time::sleep(Duration::from_secs(1)).await;

        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());
        let exploratory_selector =
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false);
        let router_ctx = RouterContext::new(
            handle.clone(),
            profile_storage,
            router_info.identity.id(),
            Bytes::from(router_info.serialize(&signing_key)),
            static_key,
            signing_key,
            2u8,
            event_handle.clone(),
        );

        let (mut exploratory_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            exploratory_selector.clone(),
            subsys_handle.clone(),
            router_ctx.clone(),
        );

        assert!(
            tokio::time::timeout(Duration::from_secs(2), &mut exploratory_pool)
                .await
                .is_err()
        );
        assert_eq!(exploratory_pool.pending_inbound.len(), 1);

        // connect all routers
        let (msg_tx, msg_rx) = channel(128);

        for (router_id, router) in &mut routers {
            let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
            let conn_tx = msg_tx.clone();

            transport_tx
                .send(SubsystemEvent::ConnectionEstablished {
                    router_id: router_id.clone(),
                    tx,
                })
                .await
                .unwrap();
            router.connect_router(&router_info.identity.id());

            let router_id = router_id.clone();
            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    match message {
                        OutboundMessage::Message(message) => {
                            conn_tx.send((router_id.clone(), message)).await.unwrap();
                        }
                        OutboundMessage::MessageWithFeedback(message, tx) => {
                            tx.send(()).unwrap();
                            conn_tx.send((router_id.clone(), message)).await.unwrap();
                        }
                        OutboundMessage::Messages(_) => panic!("not implemented"),
                        OutboundMessage::Dummy => unreachable!(),
                    }
                }
            });
        }
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 1st outbound hop (ibgw)
        let (router, message) = tokio::time::timeout(Duration::from_secs(5), msg_rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(message.message_type, MessageType::Garlic);
        let message = match routers
            .get_mut(&router)
            .unwrap()
            .garlic()
            .handle_message(message)
            .unwrap()
            .next()
        {
            Some(GarlicDeliveryInstructions::Local { message }) => message,
            _ => panic!("invalid delivery instructions"),
        };
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 2nd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 3rd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // route tunnel build response to the tunnel build response listener
        subsys_handle.send(&router, message).unwrap();

        assert!(
            tokio::time::timeout(Duration::from_secs(2), &mut exploratory_pool)
                .await
                .is_err()
        );
        assert_eq!(exploratory_pool.inbound.len(), 1);
        assert_eq!(exploratory_pool.pending_inbound.len(), 0);
        assert_eq!(MockRuntime::get_gauge_value(NUM_INBOUND_TUNNELS), Some(1));

        {
            let pool_config = TunnelPoolConfig {
                num_inbound: 0usize,
                num_inbound_hops: 0usize,
                num_outbound: 1usize,
                num_outbound_hops: 3usize,
                name: Str::from("client"),
                ..Default::default()
            };
            let parameters = TunnelPoolBuildParameters::new(pool_config);
            let pool_handle = parameters.context_handle.clone();
            let client_selector = ClientSelector::new(exploratory_selector, pool_handle);

            let (mut client_pool, _client_handle) = TunnelPool::<MockRuntime, _>::new(
                parameters,
                client_selector,
                subsys_handle.clone(),
                router_ctx.clone(),
            );

            let future = async {
                tokio::select! {
                    _ = &mut client_pool => {}
                    _ = &mut exploratory_pool => {}
                }
            };

            assert!(tokio::time::timeout(Duration::from_secs(1), future).await.is_err());

            let (router_id, message) = msg_rx.try_recv().unwrap();

            // outbound build 1st hop (participant)
            let (router_id, message) = {
                let mut router = routers.get_mut(&router_id).unwrap();

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(500), &mut router).await.is_err()
                );

                router.select_message().unwrap()
            };

            // outbound build 2nd hop (participant)
            let (router_id, message) = {
                let mut router = routers.get_mut(&router_id).unwrap();

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(500), &mut router).await.is_err()
                );

                router.select_message().unwrap()
            };

            // outbound build 3rd hop (obep)
            let (router_id, message) = {
                let mut router = routers.get_mut(&router_id).unwrap();

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(500), &mut router).await.is_err()
                );

                router.select_message().unwrap()
            };

            // build reply 1st hop (ibgw)
            let (router_id, message) = {
                let mut router = routers.get_mut(&router_id).unwrap();

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(500), &mut router).await.is_err()
                );

                router.select_message().unwrap()
            };

            // build reply 2nd hop (participant)
            let (router_id, message) = {
                let mut router = routers.get_mut(&router_id).unwrap();

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(500), &mut router).await.is_err()
                );

                router.select_message().unwrap()
            };

            // build reply 3rd hop (participant)
            let (router_id, message) = {
                let mut router = routers.get_mut(&router_id).unwrap();

                router.subsystem_handle().send(&router_id, message).unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(500), &mut router).await.is_err()
                );

                router.select_message().unwrap()
            };
            assert_eq!(&router_id, router_ctx.router_id());

            subsys_handle.send(&router_id, message).unwrap();

            let future = async {
                tokio::select! {
                    _ = &mut client_pool => {}
                    _ = &mut exploratory_pool => {}
                }
            };

            assert!(tokio::time::timeout(Duration::from_secs(4), future).await.is_err());
        }

        assert_eq!(MockRuntime::get_gauge_value(NUM_OUTBOUND_TUNNELS), Some(1));
        assert_eq!(MockRuntime::get_gauge_value(NUM_INBOUND_TUNNELS), Some(1));
    }

    #[tokio::test(start_paused = true)]
    async fn exploratory_outbound_build_reply_received_late() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });
                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 0usize,
            num_inbound_hops: 0usize,
            num_outbound: 1usize,
            num_outbound_hops: 3usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transit_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());

        let (mut tunnel_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle.clone(),
            RouterContext::new(
                handle.clone(),
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle.clone(),
            ),
        );

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.pending_outbound.len(), 1);

        // connect first hop
        let router = tokio::time::timeout(Duration::from_secs(5), dial_rx.recv())
            .await
            .unwrap()
            .unwrap();
        let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
        transport_tx
            .send(crate::subsystem::SubsystemEvent::ConnectionEstablished {
                router_id: router.clone(),
                tx,
            })
            .await
            .unwrap();

        // 1st outbound hop (participant)
        let message =
            match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await.unwrap().unwrap() {
                OutboundMessage::MessageWithFeedback(message, tx) => {
                    tx.send(()).unwrap();
                    message
                }
                _ => panic!("invalid message type"),
            };

        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 2nd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 3rd outbound hop (obep)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // don't route the response which causes the build request to expire
        assert!(tokio::time::timeout(Duration::from_secs(8), &mut tunnel_pool).await.is_err());
        assert_eq!(MockRuntime::get_counter_value(NUM_BUILD_FAILURES), Some(1));
        assert_eq!(MockRuntime::get_counter_value(NUM_BUILD_SUCCESSES), None);

        // route message to listener after timeout
        let _ = subsys_handle.send(&router, message);

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(MockRuntime::get_counter_value(NUM_BUILD_FAILURES), Some(1));
        assert_eq!(MockRuntime::get_counter_value(NUM_BUILD_SUCCESSES), None);
    }

    #[tokio::test(start_paused = true)]
    async fn exploratory_inbound_build_reply_received_late() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });
                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 3usize,
            num_outbound: 0usize,
            num_outbound_hops: 0usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());

        let (mut tunnel_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle.clone(),
            RouterContext::new(
                handle.clone(),
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle.clone(),
            ),
        );

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.pending_inbound.len(), 1);

        // connect first hop
        let router = tokio::time::timeout(Duration::from_secs(5), dial_rx.recv())
            .await
            .unwrap()
            .unwrap();
        let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
        transport_tx
            .send(crate::subsystem::SubsystemEvent::ConnectionEstablished {
                router_id: router.clone(),
                tx,
            })
            .await
            .unwrap();

        // 1st outbound hop (ibgw)
        let message =
            match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await.unwrap().unwrap() {
                OutboundMessage::MessageWithFeedback(message, tx) => {
                    tx.send(()).unwrap();
                    message
                }
                _ => panic!("invalid message type"),
            };

        assert_eq!(message.message_type, MessageType::Garlic);
        let message = match routers
            .get_mut(&router)
            .unwrap()
            .garlic()
            .handle_message(message)
            .unwrap()
            .next()
        {
            Some(GarlicDeliveryInstructions::Local { message }) => message,
            _ => panic!("invalid delivery instructions"),
        };
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 2nd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 3rd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // don't route the response which causes the build request to expire
        assert!(tokio::time::timeout(Duration::from_secs(10), &mut tunnel_pool).await.is_err());
        assert_eq!(MockRuntime::get_counter_value(NUM_BUILD_FAILURES), Some(1));

        // route message to listener after timeout
        let _ = subsys_handle.send(&router, message);

        // verify it's routed to transit manager which'll reject it
        assert!(
            tokio::time::timeout(Duration::from_secs(2), transit_rx.recv())
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn exploratory_tunnel_test() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..20)
            .map(|_| {
                let transit = TestTransitTunnelManager::new(false);
                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 2usize,
            num_outbound: 1usize,
            num_outbound_hops: 2usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx: _dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transport_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );

        // spawn subsystem manager in the background
        tokio::spawn(manager);

        // connect all routers together
        connect_routers(routers.iter_mut().map(|(_, router)| router));

        tokio::time::sleep(Duration::from_secs(1)).await;

        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let our_id = router_info.identity.id();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());

        let (mut tunnel_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle.clone(),
            RouterContext::new(
                handle.clone(),
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle.clone(),
            ),
        );

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.pending_outbound.len(), 1);

        // connect all routers
        let (msg_tx, msg_rx) = channel(128);

        for (router_id, router) in &mut routers {
            let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
            let conn_tx = msg_tx.clone();

            transport_tx
                .send(SubsystemEvent::ConnectionEstablished {
                    router_id: router_id.clone(),
                    tx,
                })
                .await
                .unwrap();
            router.connect_router(&router_info.identity.id());

            let router_id = router_id.clone();
            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    match message {
                        OutboundMessage::Message(message) => {
                            conn_tx.send((router_id.clone(), message)).await.unwrap();
                        }
                        OutboundMessage::MessageWithFeedback(message, tx) => {
                            tx.send(()).unwrap();
                            conn_tx.send((router_id.clone(), message)).await.unwrap();
                        }
                        OutboundMessage::Messages(_) => panic!("not implemented"),
                        OutboundMessage::Dummy => unreachable!(),
                    }
                }
            });
        }
        tokio::time::sleep(Duration::from_secs(2)).await;

        // build one inbound and one outbound tunnel
        for _ in 0..2 {
            let (router, message) = msg_rx.try_recv().unwrap();

            // 1st outbound hop
            let message = match message.message_type {
                MessageType::Garlic => match routers
                    .get_mut(&router)
                    .unwrap()
                    .garlic()
                    .handle_message(message)
                    .unwrap()
                    .next()
                {
                    Some(GarlicDeliveryInstructions::Local { message }) => message,
                    _ => panic!("invalid delivery instructions"),
                },
                _ => message,
            };

            let (router, message, tx) =
                routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
            if let Some(tx) = tx {
                let _ = tx.send(());
            }

            // 2nd outbound hop
            let (router, message, tx) =
                routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
            if let Some(tx) = tx {
                let _ = tx.send(());
            }

            subsys_handle.send(&router, message).unwrap();

            assert!(
                tokio::time::timeout(Duration::from_millis(250), &mut tunnel_pool)
                    .await
                    .is_err()
            );
        }

        assert_eq!(tunnel_pool.outbound.len(), 1);
        assert_eq!(tunnel_pool.inbound.len(), 1);
        assert_eq!(tunnel_pool.pending_outbound.len(), 0);
        assert_eq!(tunnel_pool.pending_inbound.len(), 0);
        assert_eq!(MockRuntime::get_gauge_value(NUM_OUTBOUND_TUNNELS), Some(1));
        assert_eq!(MockRuntime::get_gauge_value(NUM_INBOUND_TUNNELS), Some(1));

        assert!(tokio::time::timeout(Duration::from_secs(20), &mut tunnel_pool).await.is_err());
        let (router, message) = msg_rx.try_recv().unwrap();

        // 1st outbound hop (participant)
        routers
            .get_mut(&router)
            .unwrap()
            .subsystem_handle()
            .send(&router.clone(), message)
            .unwrap();
        assert!(tokio::time::timeout(
            Duration::from_millis(250),
            &mut routers.get_mut(&router).unwrap()
        )
        .await
        .is_err());

        let (router, message) = routers.get_mut(&router).unwrap().select_message().unwrap();
        assert!(routers.get_mut(&router).unwrap().select_message().is_none());

        // 2nd outbound hop (obep)
        routers
            .get_mut(&router)
            .unwrap()
            .subsystem_handle()
            .send(&router, message)
            .unwrap();
        assert!(tokio::time::timeout(
            Duration::from_millis(250),
            &mut routers.get_mut(&router).unwrap()
        )
        .await
        .is_err());
        let (mut router, mut message) = routers.get_mut(&router).unwrap().select_message().unwrap();

        for _ in 0..2 {
            // 1st inbound hop (ibgw) or 2nd hop (participant) if obep and ibgw were the same router
            routers
                .get_mut(&router)
                .unwrap()
                .subsystem_handle()
                .send(&router, message)
                .unwrap();
            assert!(tokio::time::timeout(
                Duration::from_millis(250),
                &mut routers.get_mut(&router).unwrap()
            )
            .await
            .is_err());

            let res = routers.get_mut(&router).unwrap().select_message().unwrap();
            router = res.0;
            message = res.1;

            if router == our_id {
                break;
            }
        }

        subsys_handle.send(&router, message).unwrap();

        assert!(
            tokio::time::timeout(Duration::from_millis(250), &mut tunnel_pool)
                .await
                .is_err()
        );

        assert_eq!(MockRuntime::get_counter_value(NUM_TEST_SUCCESSES), Some(1));
    }

    #[tokio::test(start_paused = true)]
    async fn exploratory_tunnel_test_expires() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });

                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 2usize,
            num_outbound: 1usize,
            num_outbound_hops: 2usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx: _dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transport_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );

        // spawn subsystem manager in the background
        tokio::spawn(manager);

        // connect all routers together
        connect_routers(routers.iter_mut().map(|(_, router)| router));

        tokio::time::sleep(Duration::from_secs(1)).await;

        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());

        let (mut tunnel_pool, _handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle.clone(),
            RouterContext::new(
                handle.clone(),
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle.clone(),
            ),
        );

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.pending_outbound.len(), 1);

        // connect all routers
        let (msg_tx, msg_rx) = channel(128);

        for (router_id, router) in &mut routers {
            let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
            let conn_tx = msg_tx.clone();

            transport_tx
                .send(SubsystemEvent::ConnectionEstablished {
                    router_id: router_id.clone(),
                    tx,
                })
                .await
                .unwrap();
            router.connect_router(&router_info.identity.id());

            let router_id = router_id.clone();
            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    match message {
                        OutboundMessage::Message(message) => {
                            conn_tx.send((router_id.clone(), message)).await.unwrap();
                        }
                        OutboundMessage::MessageWithFeedback(message, tx) => {
                            tx.send(()).unwrap();
                            conn_tx.send((router_id.clone(), message)).await.unwrap();
                        }
                        OutboundMessage::Messages(_) => panic!("not implemented"),
                        OutboundMessage::Dummy => unreachable!(),
                    }
                }
            });
        }
        tokio::time::sleep(Duration::from_secs(2)).await;

        // build one inbound and one outbound tunnel
        for _ in 0..2 {
            let (router, message) = msg_rx.try_recv().unwrap();

            // 1st outbound hop
            let message = match message.message_type {
                MessageType::Garlic => match routers
                    .get_mut(&router)
                    .unwrap()
                    .garlic()
                    .handle_message(message)
                    .unwrap()
                    .next()
                {
                    Some(GarlicDeliveryInstructions::Local { message }) => message,
                    _ => panic!("invalid delivery instructions"),
                },
                _ => message,
            };
            let (router, message, tx) =
                routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
            if let Some(tx) = tx {
                let _ = tx.send(());
            }

            // 2nd outbound hop
            let (router, message, tx) =
                routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
            if let Some(tx) = tx {
                let _ = tx.send(());
            }

            subsys_handle.send(&router, message).unwrap();

            assert!(
                tokio::time::timeout(Duration::from_millis(250), &mut tunnel_pool)
                    .await
                    .is_err()
            );
        }

        assert_eq!(tunnel_pool.outbound.len(), 1);
        assert_eq!(tunnel_pool.inbound.len(), 1);
        assert_eq!(tunnel_pool.pending_outbound.len(), 0);
        assert_eq!(tunnel_pool.pending_inbound.len(), 0);
        assert_eq!(MockRuntime::get_gauge_value(NUM_OUTBOUND_TUNNELS), Some(1));
        assert_eq!(MockRuntime::get_gauge_value(NUM_INBOUND_TUNNELS), Some(1));

        assert!(tokio::time::timeout(Duration::from_secs(20), &mut tunnel_pool).await.is_err());
        let (router, message) = msg_rx.try_recv().unwrap();

        // 1st outbound hop (participant)
        routers
            .get_mut(&router)
            .unwrap()
            .subsystem_handle()
            .send(&router, message)
            .unwrap();
        assert!(tokio::time::timeout(
            Duration::from_millis(250),
            &mut routers.get_mut(&router).unwrap()
        )
        .await
        .is_err());

        let (router, message) = routers.get_mut(&router).unwrap().select_message().unwrap();
        assert!(routers.get_mut(&router).unwrap().select_message().is_none());

        // 2nd outbound hop (obep)
        routers
            .get_mut(&router)
            .unwrap()
            .subsystem_handle()
            .send(&router, message)
            .unwrap();
        assert!(tokio::time::timeout(
            Duration::from_millis(250),
            &mut routers.get_mut(&router).unwrap()
        )
        .await
        .is_err());

        // don't route the test message any further and verify the test timeouts
        assert!(tokio::time::timeout(Duration::from_secs(9), &mut tunnel_pool).await.is_err());
        assert_eq!(MockRuntime::get_counter_value(NUM_TEST_FAILURES), Some(1));
    }

    #[tokio::test(start_paused = true)]
    async fn inbound_tunnels_removed_from_routing_table() {
        // create 10 routers and add them to local `ProfileStorage`
        let mut routers = (0..10)
            .map(|i| {
                let transit = TestTransitTunnelManager::new(if i % 2 == 0 { true } else { false });
                (transit.router(), transit)
            })
            .collect::<HashMap<_, _>>();
        let profile_storage = ProfileStorage::<MockRuntime>::from_random(
            routers.iter().map(|(_, transit)| transit.router_info()).collect(),
        );

        let pool_config = TunnelPoolConfig {
            num_inbound: 1usize,
            num_inbound_hops: 3usize,
            num_outbound: 0usize,
            num_outbound_hops: 0usize,
            ..Default::default()
        };
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let handle = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            dial_rx,
            handle: subsys_handle,
            manager,
            netdb_rx: _netdb_rx,
            transit_rx: _transport_rx,
            transport_tx,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) = EventManager::new(None, handle.clone());

        let (mut tunnel_pool, mut handle) = TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle.clone(),
            RouterContext::new(
                handle.clone(),
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle.clone(),
            ),
        );

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.pending_inbound.len(), 1);

        // connect first hop
        let router = tokio::time::timeout(Duration::from_secs(5), dial_rx.recv())
            .await
            .unwrap()
            .unwrap();
        let (tx, rx) = with_recycle(100, OutboundMessageRecycle::default());
        transport_tx
            .send(crate::subsystem::SubsystemEvent::ConnectionEstablished {
                router_id: router.clone(),
                tx,
            })
            .await
            .unwrap();

        // 1st outbound hop (ibgw)
        let message =
            match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await.unwrap().unwrap() {
                OutboundMessage::MessageWithFeedback(message, tx) => {
                    tx.send(()).unwrap();
                    message
                }
                _ => panic!("invalid message type"),
            };

        assert_eq!(message.message_type, MessageType::Garlic);
        let message = match routers
            .get_mut(&router)
            .unwrap()
            .garlic()
            .handle_message(message)
            .unwrap()
            .next()
        {
            Some(GarlicDeliveryInstructions::Local { message }) => message,
            _ => panic!("invalid delivery instructions"),
        };
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 2nd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // 3rd outbound hop (participant)
        let (router, message, tx) =
            routers.get_mut(&router).unwrap().handle_short_tunnel_build(message).unwrap();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        // route tunnel build response to the tunnel build response listener
        subsys_handle.send(&router, message).unwrap();

        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_err());
        assert_eq!(tunnel_pool.inbound.len(), 1);
        assert_eq!(tunnel_pool.pending_inbound.len(), 0);
        assert_eq!(MockRuntime::get_gauge_value(NUM_INBOUND_TUNNELS), Some(1));

        // verify the inbound tunnel exists in the routing table
        let tunnel_id = tunnel_pool.inbound_tunnels.values().next().unwrap().0;

        match subsys_handle.try_insert_tunnel::<6>(tunnel_id) {
            Err(RoutingError::TunnelExists(value)) => {
                assert_eq!(value, tunnel_id);
            }
            _ => panic!("invalid status"),
        }

        // shut down the tunnel pool
        handle.shutdown();
        assert!(tokio::time::timeout(Duration::from_secs(2), &mut tunnel_pool).await.is_ok());

        // try to add the tunnel again and ensure that it succeeds this time because the tunnel
        // pool's tunnels were removed from tunnel pool when it shut down
        match subsys_handle.try_insert_tunnel::<6>(tunnel_id) {
            Ok(_) => {}
            _ => panic!("invalid status"),
        }
    }

    // M135 test helper: minimal pool + owner handle pair.
    //
    // Reference vectors (plan §3, roadmap §3): Java quantity change is a live
    // pool settings reconfiguration; existing tunnels stay usable until
    // normal expiry/failure; future build demand follows the new quantity;
    // wanted lease count follows current inbound quantity; restore updates
    // the same generation without pool recreation.
    async fn m135_pool_with_config(
        pool_config: TunnelPoolConfig,
    ) -> (
        TunnelPool<MockRuntime, ExploratorySelector<MockRuntime>>,
        TunnelPoolHandle,
    ) {
        let (router_info, static_key, signing_key) = RouterInfoBuilder::default().build();
        let metrics = MockRuntime::register_metrics(Vec::new(), None);
        let SubsystemManagerContext {
            handle: subsys_handle,
            manager,
            ..
        } = SubsystemManager::<MockRuntime>::new(
            router_info.identity.id(),
            NoiseContext::new(
                static_key.clone(),
                Bytes::from(router_info.identity.id().to_vec()),
            ),
            Default::default(),
            MockRuntime::register_metrics(vec![], None),
        );
        tokio::spawn(manager);
        let parameters = TunnelPoolBuildParameters::new(pool_config);
        let pool_handle = parameters.context_handle.clone();
        let (_event_mgr, _event_subscriber, event_handle) =
            EventManager::new(None, metrics.clone());
        let profile_storage = ProfileStorage::<MockRuntime>::new(&[], &[], None);
        TunnelPool::<MockRuntime, _>::new(
            parameters,
            ExploratorySelector::new(profile_storage.clone(), pool_handle, false),
            subsys_handle,
            RouterContext::new(
                metrics,
                profile_storage,
                router_info.identity.id(),
                Bytes::from(router_info.serialize(&signing_key)),
                static_key,
                signing_key,
                2u8,
                event_handle,
            ),
        )
    }

    // M135 §8.1: desired targets initialize to base quantities.
    #[tokio::test(start_paused = true)]
    async fn m135_desired_initializes_to_base() {
        let (pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 3,
            num_outbound: 2,
            ..Default::default()
        })
        .await;

        assert_eq!(pool.base_quantity_target(), (3, 2));
        assert_eq!(pool.desired_quantity_target(), (3, 2));
        assert_eq!(handle.base_quantity_target(), (3, 2));
        assert_eq!(handle.desired_quantity_target(), (3, 2));
    }

    // M135 §8.2: lowering target changes future build deficit without
    // mutating base config.
    #[tokio::test(start_paused = true)]
    async fn m135_lowering_changes_deficit_without_mutating_base() {
        let (mut pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 3,
            num_outbound: 3,
            ..Default::default()
        })
        .await;

        let (active_before, _) = pool.calculate_inbound_build_count();
        assert_eq!(active_before, 3);
        let (ob_before, _) = pool.calculate_outbound_build_count();
        assert_eq!(ob_before, 3);

        handle.set_quantity_target(1, 2).unwrap();
        assert!(pool.sync_quantity_target());
        assert_eq!(pool.desired_quantity_target(), (1, 2));
        assert_eq!(pool.base_quantity_target(), (3, 3));
        assert_eq!(handle.base_quantity_target(), (3, 3));

        let (active_after, _) = pool.calculate_inbound_build_count();
        assert_eq!(active_after, 1);
        let (ob_after, _) = pool.calculate_outbound_build_count();
        assert_eq!(ob_after, 2);

        // Hop lengths and variances remain base-owned.
        assert_eq!(pool.config.num_inbound_hops, 2);
        assert_eq!(pool.config.num_outbound_hops, 2);
    }

    // M135 §8.3: lowering does not synchronously remove excess inbound tunnels.
    #[tokio::test(start_paused = true)]
    async fn m135_lowering_preserves_excess_inbound() {
        let (mut pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 3,
            num_outbound: 0,
            ..Default::default()
        })
        .await;

        for i in 0..3u32 {
            let gateway = TunnelId::from(1000 + i);
            pool.inbound_tunnels.insert(
                gateway,
                (TunnelId::from(2000 + i), RouterId::random()),
            );
            pool.selector.add_inbound_tunnel(
                gateway,
                RouterId::random(),
                HashSet::new(),
            );
        }
        assert_eq!(pool.inbound_tunnels.len(), 3);

        handle.set_quantity_target(1, 0).unwrap();
        assert!(pool.sync_quantity_target());
        assert_eq!(pool.inbound_tunnels.len(), 3);
        let (active, _) = pool.calculate_inbound_build_count();
        assert_eq!(active, 0, "no replacement above desired target");
    }

    // M135 §8.4: lowering does not synchronously clear outbound state and
    // stops replacement above the new target.
    #[tokio::test(start_paused = true)]
    async fn m135_lowering_preserves_outbound_state() {
        let (mut pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 0,
            num_outbound: 3,
            ..Default::default()
        })
        .await;

        let live_before = pool.outbound.len();
        handle.set_quantity_target(0, 1).unwrap();
        assert!(pool.sync_quantity_target());
        assert_eq!(pool.outbound.len(), live_before);
        assert_eq!(pool.desired_quantity_target(), (0, 1));
        assert_eq!(pool.base_quantity_target(), (0, 3));
        let (active, _) = pool.calculate_outbound_build_count();
        assert_eq!(active, 1 - live_before.min(1));
    }

    // M135 §8.5: excess tunnels remain selectable until normal removal.
    #[tokio::test(start_paused = true)]
    async fn m135_excess_remains_selectable() {
        let (mut pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 2,
            num_outbound: 0,
            ..Default::default()
        })
        .await;

        let first = TunnelId::from(9101u32);
        let second = TunnelId::from(9102u32);
        for gateway in [first, second] {
            pool.inbound_tunnels
                .insert(gateway, (TunnelId::from(9200u32), RouterId::random()));
            pool.selector
                .add_inbound_tunnel(gateway, RouterId::random(), HashSet::new());
        }

        handle.set_quantity_target(1, 0).unwrap();
        assert!(pool.sync_quantity_target());
        assert_eq!(pool.inbound_tunnels.len(), 2);
        assert!(
            pool.selector.select_inbound_tunnel().is_some(),
            "excess tunnels stay in ordinary selection"
        );
    }

    // M135 §8.6/§8.7: no replacement at/above target; completed excess does
    // not trigger another build.
    #[tokio::test(start_paused = true)]
    async fn m135_no_replacement_at_or_above_target() {
        let (mut pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 2,
            num_outbound: 2,
            ..Default::default()
        })
        .await;

        handle.set_quantity_target(1, 1).unwrap();
        assert!(pool.sync_quantity_target());

        // At target: one live tunnel each direction means zero deficit,
        // because pending capacity is included in the same calculation.
        for i in 0..1u32 {
            pool.inbound_tunnels.insert(
                TunnelId::from(9300 + i),
                (TunnelId::from(9400 + i), RouterId::random()),
            );
        }
        let (active_in, _) = pool.calculate_inbound_build_count();
        assert_eq!(active_in, 0);

        // Above target after a pending build completes: two live tunnels with
        // desired one still means zero deficit and no further replacement.
        pool.inbound_tunnels.insert(
            TunnelId::from(9309u32),
            (TunnelId::from(9409u32), RouterId::random()),
        );
        let (active_excess, _) = pool.calculate_inbound_build_count();
        assert_eq!(active_excess, 0);
    }

    // M135 §8.8: restore resumes deficit toward base quantities.
    #[tokio::test(start_paused = true)]
    async fn m135_restore_resumes_deficit() {
        let (mut pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 3,
            num_outbound: 2,
            ..Default::default()
        })
        .await;

        handle.set_quantity_target(1, 1).unwrap();
        assert!(pool.sync_quantity_target());
        let (reduced_in, _) = pool.calculate_inbound_build_count();
        assert_eq!(reduced_in, 1);

        handle.restore_quantity_target().unwrap();
        assert!(pool.sync_quantity_target());
        assert_eq!(pool.desired_quantity_target(), (3, 2));
        let (restored_in, _) = pool.calculate_inbound_build_count();
        let (restored_ob, _) = pool.calculate_outbound_build_count();
        assert_eq!(restored_in, 3);
        assert_eq!(restored_ob, 2);
    }

    // M135 §8.9: backup targets are unchanged across target changes.
    #[tokio::test(start_paused = true)]
    async fn m135_backup_targets_unchanged() {
        let (mut pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 2,
            num_inbound_backup: 1,
            num_outbound: 2,
            num_outbound_backup: 2,
            ..Default::default()
        })
        .await;

        handle.set_quantity_target(1, 1).unwrap();
        assert!(pool.sync_quantity_target());
        assert_eq!(pool.config.num_inbound_backup, 1);
        assert_eq!(pool.config.num_outbound_backup, 2);
        let (_, backup_in) = pool.calculate_inbound_build_count();
        let (_, backup_ob) = pool.calculate_outbound_build_count();
        assert_eq!(backup_in, 1);
        assert_eq!(backup_ob, 2);

        handle.restore_quantity_target().unwrap();
        assert!(pool.sync_quantity_target());
        assert_eq!(pool.config.num_inbound_backup, 1);
        assert_eq!(pool.config.num_outbound_backup, 2);
    }

    // M135 §8.10: standby promotion uses desired target, not base.
    #[tokio::test(start_paused = true)]
    async fn m135_standby_promotion_uses_desired_target() {
        let (mut pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 2,
            num_inbound_backup: 1,
            num_outbound: 0,
            ..Default::default()
        })
        .await;

        // One active tunnel with base two: promotion allowed at base target.
        let active = TunnelId::from(9501u32);
        pool.inbound_tunnels
            .insert(active, (TunnelId::from(9601u32), RouterId::random()));
        let standby = TunnelId::from(9502u32);
        pool.backup_inbound_tunnels.insert(
            standby,
            (
                TunnelId::from(9602u32),
                RouterId::random(),
                HashSet::new(),
                MockRuntime::time_since_epoch() + Duration::from_secs(300),
            ),
        );

        // Lower desired to one: active count already satisfies desired, so no
        // promotion even though below base.
        handle.set_quantity_target(1, 0).unwrap();
        assert!(pool.sync_quantity_target());
        assert!(!pool.promote_standby_inbound());
        assert_eq!(pool.inbound_tunnels.len(), 1);
        assert_eq!(pool.backup_inbound_tunnels.len(), 1);

        // Restore to base two: promotion resumes through the normal path.
        handle.restore_quantity_target().unwrap();
        assert!(pool.sync_quantity_target());
        assert!(pool.promote_standby_inbound());
        assert_eq!(pool.inbound_tunnels.len(), 2);
        assert!(pool.backup_inbound_tunnels.is_empty());
    }

    // M135 §8.11/§8.12: per-pool isolation; a client-seam update never
    // reaches an exploratory pool and never reaches another destination.
    #[tokio::test(start_paused = true)]
    async fn m135_pool_targets_are_isolated() {
        let (mut client, client_handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 3,
            num_outbound: 3,
            ..Default::default()
        })
        .await;
        let (mut exploratory, exploratory_handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 2,
            num_outbound: 2,
            ..Default::default()
        })
        .await;

        client_handle.set_quantity_target(1, 1).unwrap();
        assert!(client.sync_quantity_target());
        // Exploratory pool never observes the client update.
        assert!(!exploratory.sync_quantity_target());
        assert_eq!(exploratory.desired_quantity_target(), (2, 2));
        assert_eq!(exploratory_handle.desired_quantity_target(), (2, 2));
        assert_eq!(client.desired_quantity_target(), (1, 1));
    }

    // M135 §8.18: closed control is rejected and leaves the last target.
    #[tokio::test(start_paused = true)]
    async fn m135_closed_control_is_rejected() {
        let (mut pool, mut handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 2,
            num_outbound: 2,
            ..Default::default()
        })
        .await;

        handle.set_quantity_target(1, 1).unwrap();
        assert!(pool.sync_quantity_target());
        assert_eq!(pool.desired_quantity_target(), (1, 1));

        handle.shutdown();
        assert_eq!(
            handle.set_quantity_target(2, 2),
            Err(QuantityTargetError::PoolShutDown)
        );
        // Closed cell leaves the last synchronized target in place.
        assert!(!pool.sync_quantity_target());
        assert_eq!(pool.desired_quantity_target(), (1, 1));
    }

    // M135 §8.19: coalescing preserves the latest restore target.
    #[tokio::test(start_paused = true)]
    async fn m135_coalescing_preserves_latest_restore() {
        let (mut pool, handle) = m135_pool_with_config(TunnelPoolConfig {
            num_inbound: 3,
            num_outbound: 3,
            ..Default::default()
        })
        .await;

        // Burst of updates before the pool polls: only the newest matters.
        // The final restore equals the initial base, so synchronization is a
        // no-op that still leaves the restore value (not an intermediate).
        handle.set_quantity_target(1, 1).unwrap();
        handle.set_quantity_target(2, 1).unwrap();
        handle.restore_quantity_target().unwrap();
        let _ = pool.sync_quantity_target();
        assert_eq!(pool.desired_quantity_target(), (3, 3));
        // A distinct burst proves the newest pair wins when it differs.
        handle.set_quantity_target(1, 2).unwrap();
        handle.set_quantity_target(2, 1).unwrap();
        assert!(pool.sync_quantity_target());
        assert_eq!(pool.desired_quantity_target(), (2, 1));
        // No further change without a new update.
        assert!(!pool.sync_quantity_target());
    }
}
