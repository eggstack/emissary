//! I2PControl-owned bounded SAM observation aggregation.
//!
//! Core supplies only ordered, sanitized lifecycle facts through
//! [`emissary_core::SamObservationHook`]. This module owns the Proposal 170 response bounds,
//! incomplete-state policy, recovery bookkeeping, and read-only snapshot shape.

use std::{collections::BTreeMap, sync::Arc};

use parking_lot::RwLock;

use emissary_core::{SamObservationEvent, SamObservationHook, SamObservationHookError};

/// Maximum number of active SAM sessions exposed by ClientServicesInfo.
pub const SAM_SESSION_OBSERVATION_LIMIT: usize = 1000;

/// Maximum number of sockets exposed for one observed SAM session.
pub const SAM_SOCKET_OBSERVATION_LIMIT: usize = 8;

const SAM_SESSION_RECOVERY_LIMIT: usize = SAM_SESSION_OBSERVATION_LIMIT * 2;
const SAM_SOCKET_RECOVERY_LIMIT: usize = SAM_SOCKET_OBSERVATION_LIMIT * 2;

/// Error returned when a complete snapshot cannot be reconstructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SamSessionObservationError {
    /// The source refuses to expose partial or stale-as-current state.
    Incomplete,
}

/// A sanitized socket in a SAM observation snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SamObservedSocket {
    /// i2pd-compatible SAM socket type.
    pub socket_type: u8,
    /// Sanitized remote TCP peer address.
    pub peer: Arc<str>,
}

/// A sanitized SAM session in an observation snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SamObservedSession {
    /// Configured or derived destination nickname.
    pub name: Arc<str>,
    /// `.b32.i2p` destination address.
    pub address: Arc<str>,
    /// Active sockets belonging to this session.
    pub sockets: Vec<SamObservedSocket>,
}

/// Complete, bounded, read-only SAM observation snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SamSessionObservationSnapshot {
    /// Active sessions keyed by their sanitized SAM session identifier.
    pub sessions: BTreeMap<Arc<str>, SamObservedSession>,
    /// Monotonic publication generation.
    pub generation: u64,
}

/// Read-only handle used by ClientServicesInfo.
///
/// Carries an optional clone of the I2PControl-owned idle-resume tracker
/// (M134) so production composition can share one volatile eligibility owner
/// between the neutral observation source and the tunnel manager without
/// persisting or logging any reason.
#[derive(Clone)]
pub struct SamSessionObservationHandle {
    state: Arc<RwLock<State>>,
    resume_tracker: Option<crate::i2pcontrol::idle_resume::IdleResumeTracker>,
}

/// Application-owned SAM observer and bounded aggregator.
///
/// Forwards neutral `SessionRemoved` facts to the I2PControl-owned
/// `IdleResumeTracker` (M134) without affecting snapshot identity: the reason
/// never enters snapshots, logs or RPC, and publication failure never blocks
/// authoritative teardown.
pub struct SamObservationSource {
    state: Arc<RwLock<State>>,
    resume_tracker: Option<crate::i2pcontrol::idle_resume::IdleResumeTracker>,
}

struct State {
    sessions: BTreeMap<Arc<str>, SessionState>,
    unknown_sockets: BTreeMap<(Arc<str>, u64), SocketState>,
    generation: u64,
    complete: bool,
    recovery_lost: bool,
}

struct SessionState {
    name: Arc<str>,
    address: Arc<str>,
    sockets: BTreeMap<u64, SocketState>,
}

#[derive(Clone)]
struct SocketState {
    socket_type: u8,
    peer: Option<Arc<str>>,
}

impl SamObservationSource {
    /// Create an empty source and its read-only handle.
    pub fn new() -> (Arc<Self>, SamSessionObservationHandle) {
        Self::new_with_resume_tracker(None)
    }

    /// Create a source that forwards neutral termination facts to the given
    /// idle-resume tracker (M134). Snapshot semantics are unchanged: the
    /// reason is ignored for snapshot identity and stays out of logs/RPC.
    pub(crate) fn new_with_resume_tracker(
        resume_tracker: Option<crate::i2pcontrol::idle_resume::IdleResumeTracker>,
    ) -> (Arc<Self>, SamSessionObservationHandle) {
        let state = Arc::new(RwLock::new(State {
            sessions: BTreeMap::new(),
            unknown_sockets: BTreeMap::new(),
            generation: 0,
            complete: true,
            recovery_lost: false,
        }));
        let handle_tracker = resume_tracker.clone();
        (
            Arc::new(Self {
                state: Arc::clone(&state),
                resume_tracker,
            }),
            SamSessionObservationHandle {
                state,
                resume_tracker: handle_tracker,
            },
        )
    }
}

impl SamSessionObservationHandle {
    /// Construct an empty source for isolated I2PControl tests.
    pub fn empty_for_test() -> Self {
        SamObservationSource::new().1
    }

    /// Attach (or replace) the idle-resume tracker shared with the tunnel
    /// manager (M134 composition seam). Snapshot semantics are unchanged.
    ///
    /// Used by the binary composition root; the library build keeps it for
    /// API symmetry.
    #[allow(dead_code)]
    pub(crate) fn set_resume_tracker(
        &mut self,
        tracker: crate::i2pcontrol::idle_resume::IdleResumeTracker,
    ) {
        self.resume_tracker = Some(tracker);
    }

    /// Borrow the shared idle-resume tracker, if composition provided one.
    pub(crate) fn resume_tracker(
        &self,
    ) -> Option<crate::i2pcontrol::idle_resume::IdleResumeTracker> {
        self.resume_tracker.clone()
    }

    /// Return a snapshot only when every active fact is represented completely.
    pub fn snapshot(&self) -> Result<SamSessionObservationSnapshot, SamSessionObservationError> {
        let state = self.state.read();
        if !state.complete || state.recovery_lost || !state.unknown_sockets.is_empty() {
            return Err(SamSessionObservationError::Incomplete);
        }
        let sessions = state
            .sessions
            .iter()
            .map(|(session_id, session)| {
                let sockets = session
                    .sockets
                    .values()
                    .map(|socket| {
                        Some(SamObservedSocket {
                            socket_type: socket.socket_type,
                            peer: Arc::clone(socket.peer.as_ref()?),
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some((
                    Arc::clone(session_id),
                    SamObservedSession {
                        name: Arc::clone(&session.name),
                        address: Arc::clone(&session.address),
                        sockets,
                    },
                ))
            })
            .collect::<Option<BTreeMap<_, _>>>()
            .ok_or(SamSessionObservationError::Incomplete)?;
        Ok(SamSessionObservationSnapshot {
            sessions,
            generation: state.generation,
        })
    }
}

impl SamObservationHook for SamObservationSource {
    fn publish(&self, event: SamObservationEvent) -> Result<(), SamObservationHookError> {
        // M134: forward the neutral termination fact to the idle-resume
        // tracker before snapshot bookkeeping. Forwarding is passive and
        // never blocks teardown, and no secret enters the observation path.
        if matches!(event, SamObservationEvent::SessionRemoved { .. }) {
            if let Some(tracker) = &self.resume_tracker {
                tracker.record_observation(&event);
            }
        }
        let mut state = self.state.write();
        let complete = match event {
            SamObservationEvent::SessionActivated {
                session_id,
                name,
                address,
                socket_id,
                socket_type,
                peer,
            } => {
                if state.sessions.contains_key(&session_id)
                    || state.sessions.len() >= SAM_SESSION_RECOVERY_LIMIT
                {
                    state.recovery_lost = true;
                    false
                } else {
                    state.sessions.insert(
                        Arc::clone(&session_id),
                        SessionState {
                            name,
                            address,
                            sockets: BTreeMap::from([(
                                socket_id,
                                SocketState {
                                    socket_type,
                                    peer: peer.clone(),
                                },
                            )]),
                        },
                    );
                    let unknown = state
                        .unknown_sockets
                        .keys()
                        .filter(|(id, _)| id.as_ref() == session_id.as_ref())
                        .cloned()
                        .collect::<Vec<_>>();
                    for key @ (_, socket_id) in unknown {
                        let Some(socket) = state.unknown_sockets.remove(&key) else {
                            continue;
                        };
                        let Some(session) = state.sessions.get_mut(&session_id) else {
                            state.recovery_lost = true;
                            break;
                        };
                        if session.sockets.len() >= SAM_SOCKET_RECOVERY_LIMIT
                            || session.sockets.contains_key(&socket_id)
                        {
                            state.recovery_lost = true;
                            break;
                        }
                        session.sockets.insert(socket_id, socket);
                    }
                    state.generation = state.generation.wrapping_add(1);
                    peer.is_some()
                        && state.sessions.len() <= SAM_SESSION_OBSERVATION_LIMIT
                        && state.sessions.get(&session_id).is_some_and(|session| {
                            session.sockets.len() <= SAM_SOCKET_OBSERVATION_LIMIT
                        })
                }
            }
            SamObservationEvent::SocketActivated {
                session_id,
                socket_id,
                socket_type,
                peer,
            } => {
                if state.sessions.contains_key(&session_id) {
                    let representable = {
                        let session = state.sessions.get_mut(&session_id).expect("session exists");
                        if session.sockets.contains_key(&socket_id)
                            || session.sockets.len() >= SAM_SOCKET_RECOVERY_LIMIT
                        {
                            state.recovery_lost = true;
                            false
                        } else {
                            session.sockets.insert(
                                socket_id,
                                SocketState {
                                    socket_type,
                                    peer: peer.clone(),
                                },
                            );
                            peer.is_some() && session.sockets.len() <= SAM_SOCKET_OBSERVATION_LIMIT
                        }
                    };
                    state.generation = state.generation.wrapping_add(1);
                    representable
                } else {
                    if state.unknown_sockets.len() >= SAM_SESSION_RECOVERY_LIMIT {
                        state.recovery_lost = true;
                    } else {
                        state.unknown_sockets.insert(
                            (Arc::clone(&session_id), socket_id),
                            SocketState { socket_type, peer },
                        );
                    }
                    false
                }
            }
            SamObservationEvent::SocketRemoved {
                session_id,
                socket_id,
            } => {
                let removed =
                    state.unknown_sockets.remove(&(Arc::clone(&session_id), socket_id)).is_some()
                        || state
                            .sessions
                            .get_mut(&session_id)
                            .is_some_and(|session| session.sockets.remove(&socket_id).is_some());
                if removed {
                    state.generation = state.generation.wrapping_add(1);
                }
                state.is_representable()
            }
            SamObservationEvent::SessionRemoved { session_id, .. } => {
                let removed = state.sessions.remove(&session_id).is_some();
                state.unknown_sockets.retain(|(id, _), _| id.as_ref() != session_id.as_ref());
                if removed {
                    state.generation = state.generation.wrapping_add(1);
                }
                state.is_representable()
            }
        };
        state.complete = complete && state.is_representable();
        if state.complete {
            Ok(())
        } else {
            Err(SamObservationHookError)
        }
    }
}

impl State {
    fn is_representable(&self) -> bool {
        !self.recovery_lost
            && self.unknown_sockets.is_empty()
            && self.sessions.len() <= SAM_SESSION_OBSERVATION_LIMIT
            && self.sessions.values().all(|session| {
                session.sockets.len() <= SAM_SOCKET_OBSERVATION_LIMIT
                    && session.sockets.values().all(|socket| socket.peer.is_some())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event_session() -> SamObservationEvent {
        SamObservationEvent::SessionActivated {
            session_id: Arc::from("session"),
            name: Arc::from("name"),
            address: Arc::from("address.b32.i2p"),
            socket_id: 1,
            socket_type: 1,
            peer: Some(Arc::from("127.0.0.1:7656")),
        }
    }

    #[test]
    fn complete_snapshot_matches_lifecycle_fixture() {
        let (source, handle) = SamObservationSource::new();
        source.publish(event_session()).unwrap();
        source
            .publish(SamObservationEvent::SocketActivated {
                session_id: Arc::from("session"),
                socket_id: 2,
                socket_type: 2,
                peer: Some(Arc::from("127.0.0.1:7656")),
            })
            .unwrap();
        let snapshot = handle.snapshot().unwrap();
        assert_eq!(snapshot.sessions[&Arc::from("session")].sockets.len(), 2);
    }

    #[test]
    fn incomplete_state_recovers_after_authoritative_removal() {
        let (source, handle) = SamObservationSource::new();
        source
            .publish(SamObservationEvent::SocketActivated {
                session_id: Arc::from("unknown"),
                socket_id: 7,
                socket_type: 2,
                peer: Some(Arc::from("127.0.0.1:7656")),
            })
            .unwrap_err();
        assert_eq!(
            handle.snapshot(),
            Err(SamSessionObservationError::Incomplete)
        );
        source
            .publish(SamObservationEvent::SocketRemoved {
                session_id: Arc::from("unknown"),
                socket_id: 7,
            })
            .unwrap();
        assert!(handle.snapshot().unwrap().sessions.is_empty());
    }

    #[test]
    fn absent_observer_has_no_core_state() {
        let (source, handle) = SamObservationSource::new();
        drop(source);
        assert!(handle.snapshot().unwrap().sessions.is_empty());
    }
}
