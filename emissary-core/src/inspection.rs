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

//! Neutral, bounded, read-only core inspection snapshots.
//!
//! This module defines runtime-agnostic snapshot DTOs that carry actual
//! Emissary state across crate boundaries without exposing mutable handles,
//! private key material, subsystem authority, or JSON-RPC wire names.
//!
//! # Invariants
//!
//! - No JSON-RPC, Proposal 170, or wire-format terminology appears in these types.
//! - No private key, session key, or lease-set private material can enter a snapshot.
//! - All list fields are bounded at construction time.
//! - Snapshots are immutable after construction.

use crate::{
    primitives::RouterId,
    profile::{Bucket, ProfileStorage},
    runtime::Runtime,
    transport::FirewallStatus,
};

#[cfg(feature = "std")]
use parking_lot::RwLock;
#[cfg(feature = "no_std")]
use spin::rwlock::RwLock;

use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use core::fmt;

/// Neutral reason for a network failure known by a canonical runtime owner.
///
/// This enum deliberately contains no wire values. The administrative
/// adapter owns any compatibility mapping required by a control protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkErrorReason {
    /// The local clock is outside the accepted network skew.
    ClockSkew,
    /// The network is currently offline.
    Offline,
    /// The local endpoint is behind a symmetric NAT.
    SymmetricNat,
    /// The local endpoint is behind a full-cone NAT.
    FullConeNat,
    /// No usable network descriptors are available.
    NoDescriptors,
}

/// Current, independently tracked network state for one address family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkState {
    /// Reachability state observed by the transport owner.
    pub status: FirewallStatus,
    /// A failure reason only when a canonical owner knows one.
    pub error: Option<NetworkErrorReason>,
    /// Whether an existing reachability test is currently running.
    pub testing: bool,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            status: FirewallStatus::Unknown,
            error: None,
            testing: false,
        }
    }
}

/// A public router identity and its serialized public RouterInfo.
#[derive(Debug, Clone)]
pub struct PeerDirectoryEntry {
    /// Public router identity.
    pub router_id: RouterId,
    /// Serialized public RouterInfo bytes.
    pub router_info: Vec<u8>,
}

/// Owned, bounded snapshot of the current public router directory.
#[derive(Debug, Clone, Default)]
pub struct PeerDirectorySnapshot {
    /// Public router entries copied from canonical profile storage.
    pub entries: Vec<PeerDirectoryEntry>,
}

/// Failure while collecting a public router-directory snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerDirectoryInspectionError {
    /// The caller-supplied item bound would be exceeded.
    ItemLimitExceeded { limit: usize },
    /// A directory entry did not have a matching serialized RouterInfo.
    IncompleteEntry,
}

impl fmt::Display for PeerDirectoryInspectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemLimitExceeded { limit } => {
                write!(f, "public router directory exceeds item bound of {limit}")
            }
            Self::IncompleteEntry => write!(f, "public router directory snapshot is incomplete"),
        }
    }
}

/// Cloneable, request-time, read-only access to the canonical public router directory.
///
/// The storage owner remains private to the core crate. This handle exposes only
/// bounded owned public identities and serialized public RouterInfo bytes; it has
/// no mutation or router-control operations.
#[derive(Clone)]
pub struct PeerDirectoryInspection<R: Runtime> {
    profile_storage: ProfileStorage<R>,
}

impl<R: Runtime> PeerDirectoryInspection<R> {
    pub(crate) fn new(profile_storage: ProfileStorage<R>) -> Self {
        Self { profile_storage }
    }

    /// Copy the current public router directory, enforcing `max_items` before
    /// returning the collection. All storage guards are released on return.
    pub fn snapshot(
        &self,
        max_items: usize,
    ) -> Result<PeerDirectorySnapshot, PeerDirectoryInspectionError> {
        let router_ids = self.profile_storage.get_router_ids(Bucket::Any, |_, _, _| true);
        if router_ids.len() > max_items {
            return Err(PeerDirectoryInspectionError::ItemLimitExceeded { limit: max_items });
        }

        let reader = self.profile_storage.reader();
        let entries = router_ids
            .into_iter()
            .map(|router_id| {
                let router_info = reader
                    .raw_router_info(&router_id)
                    .ok_or(PeerDirectoryInspectionError::IncompleteEntry)?;
                Ok(PeerDirectoryEntry {
                    router_id,
                    router_info,
                })
            })
            .collect::<Result<Vec<_>, PeerDirectoryInspectionError>>()?;

        Ok(PeerDirectorySnapshot { entries })
    }
}

/// Bounded, owned facts about the current transport population and finite
/// connection limits.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransportInspectionSnapshot {
    /// Base64 router IDs for currently connected peers.
    pub connected_peer_ids: Vec<String>,
    /// Finite NTCP2 connection limit, or `None` when NTCP2 is disabled or
    /// configured as unlimited.
    pub ntcp2_limit: Option<usize>,
    /// Finite SSU2 connection limit, or `None` when SSU2 is disabled or
    /// configured as unlimited.
    pub ssu2_limit: Option<usize>,
    /// Current peer statistics copied from established transport sessions.
    pub peer_stats: Vec<TransportPeerInspection>,
}

/// Bounded, owned facts about one established transport session.
///
/// The boolean fields intentionally remain neutral core facts. I2PControl
/// owns their wire labels and any compatibility mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportPeerInspection {
    /// Base64 router ID of the connected peer.
    pub peer_id: String,
    /// Whether the connection was accepted inbound.
    pub inbound: bool,
    /// Whether the transport manager still owns this connection.
    pub connected: bool,
    /// Bytes received by the active transport session.
    pub bytes_received: u64,
    /// Bytes sent by the active transport session.
    pub bytes_sent: u64,
}

/// Cloneable, read-only access to current transport inspection facts.
///
/// The transport manager owns the mutable connection map and configuration.
/// This handle contains only an owned snapshot updated by that manager; it
/// has no socket, session, channel, key, or transport-control operation.
#[derive(Clone)]
pub struct TransportInspection {
    snapshot: Arc<RwLock<TransportInspectionSnapshot>>,
}

impl Default for TransportInspection {
    fn default() -> Self {
        Self::new(TransportInspectionSnapshot::default())
    }
}

impl TransportInspection {
    pub(crate) fn new(snapshot: TransportInspectionSnapshot) -> Self {
        Self {
            snapshot: Arc::new(RwLock::new(snapshot)),
        }
    }

    /// Copy current transport facts, enforcing the peer-item bound before
    /// returning. The synchronization guard is released before the caller
    /// can serialize or await on the owned result.
    pub fn snapshot(
        &self,
        max_items: usize,
    ) -> Result<TransportInspectionSnapshot, TransportInspectionError> {
        let snapshot = self.snapshot.read();
        if snapshot.connected_peer_ids.len() > max_items || snapshot.peer_stats.len() > max_items {
            return Err(TransportInspectionError::ItemLimitExceeded { limit: max_items });
        }
        Ok(snapshot.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn update_connected_peer_ids(&self, connected_peer_ids: Vec<String>) {
        let mut snapshot = self.snapshot.write();
        snapshot.connected_peer_ids = connected_peer_ids.clone();
        snapshot.peer_stats.retain(|peer| connected_peer_ids.contains(&peer.peer_id));
        for peer_id in connected_peer_ids {
            if !snapshot.peer_stats.iter().any(|peer| peer.peer_id == peer_id) {
                snapshot.peer_stats.push(TransportPeerInspection {
                    peer_id,
                    inbound: false,
                    connected: true,
                    bytes_received: 0,
                    bytes_sent: 0,
                });
            }
        }
    }

    pub(crate) fn set_ntcp2_limit(&self, limit: Option<usize>) {
        self.snapshot.write().ntcp2_limit = limit;
    }

    pub(crate) fn set_ssu2_limit(&self, limit: Option<usize>) {
        self.snapshot.write().ssu2_limit = limit;
    }

    pub(crate) fn peer_connected(&self, peer_id: String, inbound: bool) {
        let mut snapshot = self.snapshot.write();
        snapshot.connected_peer_ids.retain(|id| id != &peer_id);
        snapshot.connected_peer_ids.push(peer_id.clone());
        snapshot.connected_peer_ids.sort_unstable();
        snapshot.peer_stats.retain(|peer| peer.peer_id != peer_id);
        snapshot.peer_stats.push(TransportPeerInspection {
            peer_id,
            inbound,
            connected: true,
            bytes_received: 0,
            bytes_sent: 0,
        });
        snapshot
            .peer_stats
            .sort_unstable_by(|left, right| left.peer_id.cmp(&right.peer_id));
    }

    pub(crate) fn peer_disconnected(&self, peer_id: &str) {
        let mut snapshot = self.snapshot.write();
        snapshot.connected_peer_ids.retain(|id| id != peer_id);
        snapshot.peer_stats.retain(|peer| peer.peer_id != peer_id);
    }

    pub(crate) fn record_peer_bytes(&self, peer_id: &str, received: u64, sent: u64) {
        let mut snapshot = self.snapshot.write();
        if let Some(peer) = snapshot.peer_stats.iter_mut().find(|peer| peer.peer_id == peer_id) {
            peer.bytes_received = peer.bytes_received.saturating_add(received);
            peer.bytes_sent = peer.bytes_sent.saturating_add(sent);
        }
    }
}

/// Failure while collecting transport inspection facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportInspectionError {
    /// The caller-supplied peer-item bound would be exceeded.
    ItemLimitExceeded { limit: usize },
}

/// The neutral owner of a tunnel observed by the inspection seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TunnelPoolKind {
    /// A tunnel used by the router's own exploratory traffic.
    Exploratory,
    /// A tunnel owned by a client destination.
    Client,
    /// A tunnel accepted for transit traffic.
    Participating,
}

/// Direction of a tunnel owned by a pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TunnelDirection {
    /// Traffic enters the local destination through this tunnel.
    Inbound,
    /// Traffic leaves the local destination through this tunnel.
    Outbound,
}

/// Sanitized identity facts for one live tunnel.
///
/// The numeric tunnel ID is the public protocol identifier already used by
/// the routing table. No tunnel object, hop, RouterId, destination, key, or
/// message data is retained here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TunnelInspectionEntry {
    /// Stable identity of the pool within this process, when pool-owned.
    pub pool_id: u64,
    /// Public tunnel identifier used by the local routing owner.
    pub tunnel_id: u32,
    /// Neutral owner classification.
    pub pool_kind: TunnelPoolKind,
    /// Pool direction, or `None` for a participating tunnel.
    pub direction: Option<TunnelDirection>,
}

/// Bounded, owned facts about currently live tunnel owners.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TunnelInspectionSnapshot {
    /// Current live tunnel entries in deterministic order.
    pub entries: Vec<TunnelInspectionEntry>,
    /// Current number of local tunnel build requests in progress.
    pub queue_depth: usize,
    /// Current number of transit tunnel-build messages awaiting handling.
    pub tbm_queue_depth: usize,
}

/// Cloneable, read-only access to current tunnel lifecycle facts.
#[derive(Clone)]
pub struct TunnelInspection {
    snapshot: Arc<RwLock<TunnelInspectionState>>,
}

#[derive(Debug, Default)]
struct TunnelInspectionState {
    entries: Vec<TunnelInspectionEntry>,
    complete: bool,
    queue_depths: BTreeMap<(TunnelPoolKind, u64), usize>,
    tbm_queue_depth: usize,
}

/// Maximum number of live tunnel facts retained by the neutral source.
pub const MAX_TUNNEL_INSPECTION_ENTRIES: usize = 10_000;

impl Default for TunnelInspection {
    fn default() -> Self {
        Self::new()
    }
}

impl TunnelInspection {
    pub(crate) fn new() -> Self {
        Self {
            snapshot: Arc::new(RwLock::new(TunnelInspectionState {
                entries: Vec::new(),
                complete: true,
                queue_depths: BTreeMap::new(),
                tbm_queue_depth: 0,
            })),
        }
    }

    /// Copy current tunnel facts, failing closed if an owner update overflowed
    /// or the caller's bound is smaller than the owned snapshot.
    pub fn snapshot(
        &self,
        max_items: usize,
    ) -> Result<TunnelInspectionSnapshot, TunnelInspectionError> {
        let state = self.snapshot.read();
        if !state.complete {
            return Err(TunnelInspectionError::Incomplete);
        }
        if state.entries.len() > max_items {
            return Err(TunnelInspectionError::ItemLimitExceeded { limit: max_items });
        }
        Ok(TunnelInspectionSnapshot {
            entries: state.entries.clone(),
            queue_depth: state.queue_depths.values().copied().fold(0, usize::saturating_add),
            tbm_queue_depth: state.tbm_queue_depth,
        })
    }

    /// Publish the current pending-build count for one canonical tunnel pool.
    pub(crate) fn set_pool_queue_depth(
        &self,
        pool_kind: TunnelPoolKind,
        pool_id: u64,
        depth: usize,
    ) {
        self.snapshot.write().queue_depths.insert((pool_kind, pool_id), depth);
    }

    /// Publish the current transit tunnel-build message queue depth.
    pub(crate) fn set_tbm_queue_depth(&self, depth: usize) {
        self.snapshot.write().tbm_queue_depth = depth;
    }

    pub(crate) fn publish(
        &self,
        entry: TunnelInspectionEntry,
    ) -> Result<(), TunnelInspectionError> {
        let mut state = self.snapshot.write();
        if let Some(existing) = state.entries.iter_mut().find(|existing| {
            existing.pool_id == entry.pool_id
                && existing.tunnel_id == entry.tunnel_id
                && existing.pool_kind == entry.pool_kind
                && existing.direction == entry.direction
        }) {
            *existing = entry;
            return Ok(());
        }
        if state.entries.len() >= MAX_TUNNEL_INSPECTION_ENTRIES {
            state.complete = false;
            return Err(TunnelInspectionError::ItemLimitExceeded {
                limit: MAX_TUNNEL_INSPECTION_ENTRIES,
            });
        }
        state.entries.push(entry);
        state.entries.sort_unstable();
        Ok(())
    }

    pub(crate) fn remove(&self, entry: TunnelInspectionEntry) {
        let mut state = self.snapshot.write();
        state.entries.retain(|existing| existing != &entry);
    }

    pub(crate) fn remove_pool(&self, pool_kind: TunnelPoolKind, pool_id: u64) {
        let mut state = self.snapshot.write();
        state
            .entries
            .retain(|entry| entry.pool_kind != pool_kind || entry.pool_id != pool_id);
        state.queue_depths.remove(&(pool_kind, pool_id));
    }

    /// Replace the complete source at an owner-provided recovery point.
    #[allow(dead_code)]
    pub(crate) fn recover(
        &self,
        entries: Vec<TunnelInspectionEntry>,
    ) -> Result<(), TunnelInspectionError> {
        if entries.len() > MAX_TUNNEL_INSPECTION_ENTRIES {
            return Err(TunnelInspectionError::ItemLimitExceeded {
                limit: MAX_TUNNEL_INSPECTION_ENTRIES,
            });
        }
        let mut sorted = entries;
        sorted.sort_unstable();
        sorted.dedup();
        let mut state = self.snapshot.write();
        state.entries = sorted;
        state.complete = true;
        state.queue_depths.clear();
        state.tbm_queue_depth = 0;
        Ok(())
    }
}

/// Failure while collecting tunnel inspection facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelInspectionError {
    /// The source lost completeness and must be recovered by an owner snapshot.
    Incomplete,
    /// The caller or source exceeded its item bound.
    ItemLimitExceeded { limit: usize },
}

impl fmt::Display for TunnelInspectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Incomplete => write!(f, "tunnel inspection snapshot is incomplete"),
            Self::ItemLimitExceeded { limit } => {
                write!(
                    f,
                    "tunnel inspection snapshot exceeds bound of {limit} items"
                )
            }
        }
    }
}

impl fmt::Display for TransportInspectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemLimitExceeded { limit } => {
                write!(f, "connected peer inventory exceeds item bound of {limit}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{primitives::RouterInfoBuilder, runtime::mock::MockRuntime};
    use bytes::Bytes;

    #[tokio::test]
    async fn peer_directory_snapshot_is_live_after_construction() {
        let (initial_info, _, initial_signing_key) = RouterInfoBuilder::default().build();
        let initial_id = initial_info.identity.id();
        let initial_bytes = initial_info.serialize(&initial_signing_key);
        let storage = ProfileStorage::<MockRuntime>::new(&[initial_bytes.clone()], &[], None);
        let inspection = PeerDirectoryInspection::new(storage.clone());

        let first = inspection.snapshot(10).unwrap();
        assert_eq!(first.entries.len(), 1);
        assert_eq!(first.entries[0].router_id, initial_id);
        assert_eq!(first.entries[0].router_info, initial_bytes);

        let (discovered_info, _, _) = RouterInfoBuilder::default().build();
        let discovered_id = discovered_info.identity.id();
        let discovered_bytes = b"current-discovered-router-info".to_vec();
        assert!(storage.discover_router(
            discovered_info.clone(),
            Bytes::from(discovered_bytes.clone()),
        ));

        let second = inspection.snapshot(10).unwrap();
        assert_eq!(second.entries.len(), 2);
        let discovered =
            second.entries.iter().find(|entry| entry.router_id == discovered_id).unwrap();
        assert_eq!(discovered.router_info, discovered_bytes);

        let updated_bytes = b"current-updated-router-info".to_vec();
        assert!(storage.discover_router(discovered_info, Bytes::from(updated_bytes.clone())));
        let third = inspection.snapshot(10).unwrap();
        let updated = third.entries.iter().find(|entry| entry.router_id == discovered_id).unwrap();
        assert_eq!(updated.router_info, updated_bytes);
    }

    #[tokio::test]
    async fn peer_directory_snapshot_rejects_oversize_and_incomplete_results() {
        let (first_info, _, first_signing_key) = RouterInfoBuilder::default().build();
        let storage = ProfileStorage::<MockRuntime>::new(
            &[first_info.serialize(&first_signing_key)],
            &[],
            None,
        );
        let inspection = PeerDirectoryInspection::new(storage.clone());

        let (second_info, _, _) = RouterInfoBuilder::default().build();
        storage.add_router(second_info);
        assert!(matches!(
            inspection.snapshot(1),
            Err(PeerDirectoryInspectionError::ItemLimitExceeded { limit: 1 })
        ));
        assert!(matches!(
            inspection.snapshot(10),
            Err(PeerDirectoryInspectionError::IncompleteEntry)
        ));
    }

    #[test]
    fn transport_inspection_is_cloneable_bounded_and_live() {
        let inspection = TransportInspection::new(TransportInspectionSnapshot {
            connected_peer_ids: vec!["peer-b".into(), "peer-a".into()],
            ntcp2_limit: Some(64),
            ssu2_limit: None,
            peer_stats: Vec::new(),
        });
        let clone = inspection.clone();

        assert_eq!(
            clone.snapshot(2).unwrap().connected_peer_ids,
            ["peer-b", "peer-a"]
        );
        assert_eq!(clone.snapshot(2).unwrap().ntcp2_limit, Some(64));
        assert_eq!(clone.snapshot(2).unwrap().ssu2_limit, None);
        assert!(matches!(
            inspection.snapshot(1),
            Err(TransportInspectionError::ItemLimitExceeded { limit: 1 })
        ));

        inspection.update_connected_peer_ids(vec!["peer-c".into()]);
        assert_eq!(clone.snapshot(2).unwrap().connected_peer_ids, ["peer-c"]);
    }

    #[test]
    fn transport_peer_stats_are_live_bounded_and_removed_with_sessions() {
        let inspection = TransportInspection::new(TransportInspectionSnapshot::default());
        inspection.peer_connected("peer-b".into(), false);
        inspection.peer_connected("peer-a".into(), true);
        inspection.record_peer_bytes("peer-a", 7, 11);

        let snapshot = inspection.snapshot(2).unwrap();
        assert_eq!(snapshot.connected_peer_ids, ["peer-a", "peer-b"]);
        assert_eq!(snapshot.peer_stats[0].peer_id, "peer-a");
        assert!(snapshot.peer_stats[0].inbound);
        assert!(snapshot.peer_stats[0].connected);
        assert_eq!(snapshot.peer_stats[0].bytes_received, 7);
        assert_eq!(snapshot.peer_stats[0].bytes_sent, 11);

        assert_eq!(
            inspection.snapshot(1),
            Err(TransportInspectionError::ItemLimitExceeded { limit: 1 })
        );
        inspection.peer_disconnected("peer-a");
        assert_eq!(inspection.snapshot(2).unwrap().peer_stats.len(), 1);
    }

    #[test]
    fn tunnel_inspection_is_live_bounded_and_recoverable() {
        let inspection = TunnelInspection::default();
        let entry = TunnelInspectionEntry {
            pool_id: 7,
            tunnel_id: 42,
            pool_kind: TunnelPoolKind::Client,
            direction: Some(TunnelDirection::Outbound),
        };
        inspection.publish(entry).unwrap();
        assert_eq!(inspection.snapshot(1).unwrap().entries, vec![entry]);

        inspection.remove(entry);
        assert!(inspection.snapshot(1).unwrap().entries.is_empty());

        inspection.recover(vec![entry]).unwrap();
        assert_eq!(inspection.snapshot(1).unwrap().entries, vec![entry]);
    }

    #[test]
    fn tunnel_queue_gauges_are_live_and_removed_with_pool() {
        let inspection = TunnelInspection::default();
        inspection.set_pool_queue_depth(TunnelPoolKind::Exploratory, 0, 3);
        inspection.set_pool_queue_depth(TunnelPoolKind::Client, 4, 2);
        inspection.set_tbm_queue_depth(5);
        let snapshot = inspection.snapshot(0).unwrap();
        assert_eq!(snapshot.queue_depth, 5);
        assert_eq!(snapshot.tbm_queue_depth, 5);

        inspection.set_pool_queue_depth(TunnelPoolKind::Client, 4, 0);
        inspection.remove_pool(TunnelPoolKind::Exploratory, 0);
        let snapshot = inspection.snapshot(0).unwrap();
        assert_eq!(snapshot.queue_depth, 0);
        assert_eq!(snapshot.tbm_queue_depth, 5);
    }

    #[test]
    fn tunnel_inspection_fails_closed_after_overflow_until_recovery() {
        let inspection = TunnelInspection::default();
        inspection
            .recover(
                (0..MAX_TUNNEL_INSPECTION_ENTRIES)
                    .map(|tunnel_id| TunnelInspectionEntry {
                        pool_id: 1,
                        tunnel_id: tunnel_id as u32,
                        pool_kind: TunnelPoolKind::Exploratory,
                        direction: Some(TunnelDirection::Inbound),
                    })
                    .collect(),
            )
            .unwrap();
        assert!(inspection
            .publish(TunnelInspectionEntry {
                pool_id: 1,
                tunnel_id: u32::MAX,
                pool_kind: TunnelPoolKind::Exploratory,
                direction: Some(TunnelDirection::Inbound),
            })
            .is_err());
        assert_eq!(
            inspection.snapshot(MAX_TUNNEL_INSPECTION_ENTRIES),
            Err(TunnelInspectionError::Incomplete)
        );
        inspection.recover(Vec::new()).unwrap();
        assert!(inspection.snapshot(0).unwrap().entries.is_empty());
    }
}

/// Bounded transport connection snapshot.
///
/// Carries actual UDP/TCP transport state from canonical core owners
/// to inspection adapters without exposing session or transport handles.
#[derive(Debug, Clone)]
pub struct TransportSnapshot {
    /// Whether any transport is currently active (has connected peers).
    pub udp_active: bool,
    /// Whether UDP is firewalled.
    pub udp_firewalled: bool,
    /// Whether TCP is currently active.
    pub tcp_active: bool,
    /// Number of currently connected peers (across all transports).
    pub connected_peer_count: usize,
    /// Bounded list of connected peer IDs.
    pub connected_peer_ids: Vec<String>,
    /// IPv4 firewall status as a string code: "ok", "firewalled", "symmetric_nat", "unknown".
    pub ipv4_firewall_status: String,
    /// IPv6 firewall status as a string code.
    pub ipv6_firewall_status: String,
}

/// Bounded tunnel pool snapshot.
///
/// Carries actual tunnel pool state from canonical core owners.
/// Does not include configured definition counts — those remain
/// in the shared administrative tunnel manager store.
#[derive(Debug, Clone)]
pub struct TunnelSnapshot {
    /// Active participating (transit) tunnel count.
    pub active_participating: usize,
    /// Active exploratory inbound tunnel count.
    pub exploratory_inbound: usize,
    /// Active exploratory outbound tunnel count.
    pub exploratory_outbound: usize,
    /// Active client inbound tunnel count.
    pub client_inbound: usize,
    /// Active client outbound tunnel count.
    pub client_outbound: usize,
    /// Pending tunnel build queue depth.
    pub queue_depth: usize,
}

/// Bounded NetDB and peer storage snapshot.
///
/// Carries actual NetDB state from canonical core owners.
/// Profile classifications are only included where Emissary
/// already maintains them canonically.
#[derive(Debug, Clone)]
pub struct NetDbSnapshot {
    /// Whether the NetDB subsystem is active.
    pub active: bool,
    /// Number of stored router infos (floodfill only; zero on non-floodfill).
    pub router_info_count: usize,
    /// Number of stored lease sets (floodfill only; zero on non-floodfill).
    pub lease_set_count: usize,
    /// Bounded list of known router IDs.
    pub known_router_ids: Vec<String>,
    /// Total number of known peers in profile storage.
    pub known_peer_count: usize,
    /// Number of connected (active) peers.
    pub active_peer_count: usize,
    /// Bounded map of peer ID → serialized public RouterInfo bytes.
    ///
    /// Populated for bounded peer RouterInfo lookup. Keys are Base64 router IDs.
    /// Values are the raw serialized RouterInfo (public only, no private material).
    pub peer_router_infos: alloc::collections::BTreeMap<String, Vec<u8>>,
}

/// Pre-computed core inspection snapshot.
///
/// This struct is populated by `Router::inspection_snapshot()` and
/// passed across crate boundaries to inspection adapters. It is non-generic
/// and contains only owned, bounded, public data.
///
/// # Invariants
///
/// - All fields are populated from canonical core owners at construction time.
/// - No private key, session key, or mutable handle is stored.
/// - List fields are bounded at construction time.
/// - The snapshot is immutable after construction.
#[derive(Debug, Clone)]
pub struct CoreSnapshot {
    /// Router identity in Base64.
    pub router_id_b64: String,
    /// Serialized local RouterInfo bytes (public only).
    pub router_info_bytes: Vec<u8>,
    /// Transport connection state.
    pub transport: TransportSnapshot,
    /// Tunnel pool state.
    pub tunnels: TunnelSnapshot,
    /// NetDB and peer storage state.
    pub netdb: NetDbSnapshot,
}
