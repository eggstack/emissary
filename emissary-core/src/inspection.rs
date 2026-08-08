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
};

use alloc::{string::String, vec::Vec};
use core::fmt;

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
