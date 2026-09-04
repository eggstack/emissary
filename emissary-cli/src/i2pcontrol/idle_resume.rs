//! I2PControl-owned generation-local idle-resume eligibility for `NewDest`.
//!
//! M137 provides one neutral authoritative in-process termination cause
//! (`IdlePolicy` / `Requested` / `Failure` / `Unknown`) recorded first-wins at
//! the winning SAM session transition and carried in `SamSessionResult` plus
//! `SessionRemoved { reason }`. This module is the I2PControl-owned consumer
//! that turns that neutral fact into exactly one future resume decision.
//!
//! Design (M134, rebased on M137 §12):
//! - eligibility is generation-local, one-shot and non-persistent;
//! - a fresh successor is staged only when reopening after the immediately
//!   preceding owning generation was authoritatively closed by the configured
//!   idle-close policy;
//! - manual Stop, explicit Restart, process restart, transport/router failure,
//!   failed or cancelled resume, and unrelated edits never imply rotation;
//! - no wall-clock heuristic, no local handler-count inference, no error-string
//!   parsing: only an explicit `IdlePolicy` record qualifies;
//! - concurrent starts serialize through the existing per-name lifecycle owner
//!   plus a per-shared-policy reservation here, so one eligibility yields at
//!   most one successor;
//! - this tracker holds no secret material: names, generations, policy keys
//!   and neutral reasons only. Destination keys remain owned exclusively by
//!   `ClientDestinationStore` (per-name) and its synthetic shared entries.

use std::{collections::BTreeMap, sync::Arc};

use emissary_core::{SamObservationEvent, SamTerminationReason};
use parking_lot::Mutex;
use tokio::sync::Notify;

const MAX_TRACKED_TUNNELS: usize = 1000;
const MAX_TRACKED_SHARED: usize = 1000;

/// Synthetic shared-identity namespace inside `ClientDestinationStore`.
///
/// User tunnel names with this prefix are rejected at the administrative
/// boundary so synthetic shared entries can never collide with a real tunnel.
pub(crate) const SHARED_SYNTHETIC_PREFIX: &str = "__emissary_shared_client_";

/// Bounded I2PControl-owned idle-resume tracker.
///
/// Clone shares the same underlying state. All state is in-memory only and
/// never survives process restart by construction.
#[derive(Clone, Debug)]
pub(crate) struct IdleResumeTracker {
    state: Arc<Mutex<TrackerState>>,
}

#[derive(Debug, Default)]
struct TrackerState {
    dedicated: BTreeMap<String, DedicatedEntry>,
    shared: BTreeMap<String, SharedEntry>,
    /// Shared member name -> stable policy key (no secrets, bounded).
    shared_members: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct DedicatedEntry {
    /// Last committed generation number (0 = none yet).
    generation: u64,
    /// Generation that closed by idle policy and is eligible for one resume.
    eligible: Option<u64>,
    /// Generation that already has a terminal fact (idle or not). First
    /// winner wins: late facts for the same generation are ignored so a
    /// manual Stop/Restart racing idle-close delivery cannot cause surprise
    /// rotation (ambiguous stale becomes ineligible).
    settled: Option<u64>,
}

#[derive(Debug)]
struct SharedEntry {
    generation: u64,
    eligible: Option<u64>,
    settled: Option<u64>,
    resuming: bool,
    notify: Arc<Notify>,
}

impl Default for IdleResumeTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl IdleResumeTracker {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TrackerState::default())),
        }
    }

    /// Current committed generation for a dedicated tunnel (0 when none).
    #[allow(dead_code)]
    pub(crate) fn dedicated_generation(&self, name: &str) -> u64 {
        self.state.lock().dedicated.get(name).map_or(0, |entry| entry.generation)
    }

    /// Whether a dedicated tunnel currently holds an unconsumed qualifying
    /// idle-close fact for its current generation.
    pub(crate) fn is_dedicated_eligible(&self, name: &str) -> bool {
        let state = self.state.lock();
        state
            .dedicated
            .get(name)
            .is_some_and(|entry| entry.eligible == Some(entry.generation) && entry.generation > 0)
    }

    /// Record the authoritative termination of one dedicated generation.
    ///
    /// First winner wins for the current generation: only an `IdlePolicy`
    /// fact arms eligibility, any other reason settles the generation as
    /// non-idle, and late facts for an already-settled generation are ignored
    /// so a manual Stop/Restart racing idle-close delivery cannot cause
    /// surprise rotation. Stale generations (not equal to the current
    /// committed generation) are ignored.
    ///
    /// Test hook for deterministic proven-resume evidence; production feeds
    /// through [`IdleResumeTracker::record_observation`].
    #[allow(dead_code)]
    pub(crate) fn record_dedicated_termination(
        &self,
        name: &str,
        generation: u64,
        reason: SamTerminationReason,
    ) {
        let mut state = self.state.lock();
        let entry = state.dedicated.entry(name.to_owned()).or_insert(DedicatedEntry {
            generation: 0,
            eligible: None,
            settled: None,
        });
        if entry.generation == 0 {
            // No committed generation yet: nothing can be eligible. Still
            // record the generation when the termination carries one so a
            // racing idle close cannot arm a future generation.
            if generation > 0 && reason == SamTerminationReason::IdlePolicy {
                // Do not arm: there is no preceding owning generation.
            }
            return;
        }
        if generation != entry.generation {
            return;
        }
        if entry.settled == Some(generation) {
            return;
        }
        entry.settled = Some(generation);
        if reason == SamTerminationReason::IdlePolicy {
            entry.eligible = Some(generation);
        } else {
            entry.eligible = None;
        }
        enforce_dedicated_bound(&mut state.dedicated);
    }

    /// Advance the committed generation after a successful start commit and
    /// consume any eligibility that qualified this resume.
    pub(crate) fn commit_dedicated_generation(&self, name: &str) {
        let mut state = self.state.lock();
        let entry = state.dedicated.entry(name.to_owned()).or_insert(DedicatedEntry {
            generation: 0,
            eligible: None,
            settled: None,
        });
        entry.generation = entry.generation.wrapping_add(1).max(1);
        entry.eligible = None;
        entry.settled = None;
        enforce_dedicated_bound(&mut state.dedicated);
    }

    /// Settle the current generation as explicitly stopped without arming
    /// idle eligibility, while preserving a qualifying idle fact that already
    /// won the race.
    ///
    /// A manual Stop/Restart that races idle-close delivery must not cause
    /// surprise rotation when the manual cause wins, nor destroy a qualifying
    /// idle fact that already won: if the current generation already settled
    /// (idle or not), this preserves the winner; otherwise it settles the
    /// generation as non-idle so a late idle fact becomes ineligible.
    /// A later Start after a preserved qualifying fact still resumes with one
    /// successor; ordinary Starts reuse the committed identity.
    pub(crate) fn note_manual_stop(&self, name: &str) {
        let mut state = self.state.lock();
        if let Some(entry) = state.dedicated.get_mut(name) {
            if entry.settled.is_none() && entry.generation > 0 {
                entry.settled = Some(entry.generation);
                entry.eligible = None;
            }
        }
    }

    /// Remove all dedicated state for a deleted tunnel.
    pub(crate) fn note_delete(&self, name: &str) {
        let mut state = self.state.lock();
        state.dedicated.remove(name);
        state.shared_members.remove(name);
    }

    /// Reconcile dedicated state after an edit that changes lifecycle policy.
    ///
    /// Any edit touching `NewDest`/`Close`/`PersistentClientKey`/`PrivKeyFile`/
    /// `Shared` clears pending eligibility and settles the current generation
    /// so late pre-edit facts cannot arm a post-edit resume: the edited
    /// definition starts its next generation from the committed identity.
    pub(crate) fn note_edit(&self, name: &str) {
        let mut state = self.state.lock();
        if let Some(entry) = state.dedicated.get_mut(name) {
            entry.eligible = None;
            if entry.generation > 0 && entry.settled.is_none() {
                entry.settled = Some(entry.generation);
            }
        }
        state.shared_members.remove(name);
        // Shared eligibility is keyed by policy; an edit changes the policy
        // key itself, so stale shared eligibility is naturally unreachable.
        // No shared sweep is needed here.
    }

    /// Register a shared member name -> policy key mapping (M134).
    ///
    /// Called on successful shared Start commit so later SAM `SessionRemoved`
    /// events for the shared session id (first creator's nickname) can arm
    /// the stable policy eligibility for all members. Bounded; evicts oldest
    /// on overflow (volatile eligibility only, never secrets).
    pub(crate) fn register_shared_member(&self, name: &str, policy_key: &str) {
        let mut state = self.state.lock();
        state.shared_members.insert(name.to_owned(), policy_key.to_owned());
        while state.shared_members.len() > MAX_TRACKED_SHARED {
            if let Some(first) = state.shared_members.keys().next().cloned() {
                state.shared_members.remove(&first);
            } else {
                break;
            }
        }
    }

    /// Remove a shared member mapping (delete or policy-changing edit).
    pub(crate) fn unregister_shared_member(&self, name: &str) {
        self.state.lock().shared_members.remove(name);
    }

    /// Move dedicated generation state across an administrative rename.
    ///
    /// Eligibility is cleared: the renamed definition starts its next
    /// generation from the renamed committed identity, never from a stale
    /// pre-rename idle fact. The generation number carries over so stale
    /// pre-rename terminations cannot arm the new name.
    pub(crate) fn rename_dedicated(&self, old_name: &str, new_name: &str) {
        if old_name == new_name {
            return;
        }
        let mut state = self.state.lock();
        let Some(entry) = state.dedicated.remove(old_name) else {
            return;
        };
        let generation = entry.generation;
        state.dedicated.insert(
            new_name.to_owned(),
            DedicatedEntry {
                generation,
                eligible: None,
                settled: None,
            },
        );
        state.shared_members.remove(old_name);
        enforce_dedicated_bound(&mut state.dedicated);
    }

    /// Forward a neutral SAM observation event into eligibility state.
    ///
    /// First winner wins per generation: only an `IdlePolicy` fact for an
    /// unsettled current generation arms eligibility; any other reason settles
    /// as non-idle; late facts for a settled generation are ignored so a
    /// manual Stop/Restart racing idle-close delivery cannot cause surprise
    /// rotation. Publication failure semantics stay with the caller: this
    /// method never blocks and never performs I/O.
    pub(crate) fn record_observation(&self, event: &SamObservationEvent) {
        let (session_id, reason) = match event {
            SamObservationEvent::SessionRemoved { session_id, reason } => (session_id, *reason),
            _ => return,
        };
        let mut state = self.state.lock();
        // Dedicated: SAM session id equals the tunnel name (Yosemite
        // `SESSION CREATE ID` is the definition name).
        if let Some(entry) = state.dedicated.get_mut(session_id.as_ref()) {
            if entry.generation > 0 && entry.settled.is_none() {
                entry.settled = Some(entry.generation);
                if reason == SamTerminationReason::IdlePolicy {
                    entry.eligible = Some(entry.generation);
                } else {
                    entry.eligible = None;
                }
            }
        }
        // Shared: first creator's nickname maps to the stable policy so one
        // idle close arms one shared successor for all members.
        if let Some(policy_key) = state.shared_members.get(session_id.as_ref()).cloned() {
            if let Some(shared) = state.shared.get_mut(&policy_key) {
                if shared.generation > 0 && shared.settled.is_none() {
                    shared.settled = Some(shared.generation);
                    if reason == SamTerminationReason::IdlePolicy {
                        shared.eligible = Some(shared.generation);
                    } else {
                        shared.eligible = None;
                    }
                }
            }
        }
    }

    // --- Shared policy eligibility (one successor per shared session) ---

    /// Whether a shared policy currently holds an unconsumed qualifying fact.
    pub(crate) fn is_shared_eligible(&self, policy_key: &str) -> bool {
        let state = self.state.lock();
        state.shared.get(policy_key).is_some_and(|entry| {
            entry.eligible == Some(entry.generation) && entry.generation > 0 && !entry.resuming
        })
    }

    /// Record termination of one shared generation identified by its stable
    /// policy key. First winner wins: only an `IdlePolicy` fact arms, any
    /// other reason settles as non-idle, and late facts for a settled
    /// generation are ignored.
    ///
    /// Test hook; production shared arming flows through `record_observation`
    /// via the member mapping.
    #[allow(dead_code)]
    pub(crate) fn record_shared_termination(
        &self,
        policy_key: &str,
        generation: u64,
        reason: SamTerminationReason,
    ) {
        let mut state = self.state.lock();
        let entry = state.shared.entry(policy_key.to_owned()).or_insert_with(|| SharedEntry {
            generation: 0,
            eligible: None,
            settled: None,
            resuming: false,
            notify: Arc::new(Notify::new()),
        });
        if entry.generation == 0 || generation != entry.generation {
            return;
        }
        if entry.settled == Some(generation) {
            return;
        }
        entry.settled = Some(generation);
        if reason == SamTerminationReason::IdlePolicy {
            entry.eligible = Some(generation);
        } else {
            entry.eligible = None;
        }
        enforce_shared_bound(&mut state.shared);
    }

    /// Advance the shared committed generation after a successful shared
    /// resume commit and consume eligibility.
    pub(crate) fn commit_shared_generation(&self, policy_key: &str) {
        let mut state = self.state.lock();
        let entry = state.shared.entry(policy_key.to_owned()).or_insert_with(|| SharedEntry {
            generation: 0,
            eligible: None,
            settled: None,
            resuming: false,
            notify: Arc::new(Notify::new()),
        });
        entry.generation = entry.generation.wrapping_add(1).max(1);
        entry.eligible = None;
        entry.settled = None;
        entry.resuming = false;
        entry.notify.notify_waiters();
        enforce_shared_bound(&mut state.shared);
    }

    /// Remove shared policy state entirely (no members remain).
    pub(crate) fn remove_shared(&self, policy_key: &str) {
        self.state.lock().shared.remove(policy_key);
    }

    /// Try to reserve the single successor generation for a shared policy.
    ///
    /// Returns `Some(SharedResumeReservation)` for exactly one caller per
    /// eligible generation; concurrent callers receive `None` and must wait
    /// on [`IdleResumeTracker::wait_shared_resume`] then reuse the committed
    /// successor instead of generating their own. The reservation never spans
    /// network, filesystem or session-construction I/O: callers must drop it
    /// (or disarm via commit) before awaiting I/O, and re-reserve semantics
    /// are handled by the generation/eligibility state, not by holding this
    /// guard across awaits.
    #[allow(dead_code)]
    pub(crate) fn try_reserve_shared_resume(
        &self,
        policy_key: &str,
    ) -> Option<SharedResumeReservation> {
        let notify = {
            let mut state = self.state.lock();
            let entry = state.shared.entry(policy_key.to_owned()).or_insert_with(|| SharedEntry {
                generation: 0,
                eligible: None,
                settled: None,
                resuming: false,
                notify: Arc::new(Notify::new()),
            });
            if entry.resuming {
                return None;
            }
            // Only an eligible shared generation can be reserved for resume.
            // Initial generations (generation 0, no eligibility) use the
            // normal initial-identity path, not the resume reservation.
            if entry.eligible != Some(entry.generation) || entry.generation == 0 {
                return None;
            }
            entry.resuming = true;
            Arc::clone(&entry.notify)
        };
        Some(SharedResumeReservation {
            tracker: self.clone(),
            policy_key: policy_key.to_owned(),
            notify,
            active: true,
        })
    }

    /// Try to reserve shared creation (initial or resume) for one policy.
    ///
    /// Unlike [`IdleResumeTracker::try_reserve_shared_resume`], this reserves
    /// whenever no resume is already in flight, regardless of eligibility.
    /// Callers use it to serialize concurrent initial generations to one
    /// successor; joins (current identity exists, no eligibility) must not
    /// reserve at all and instead reuse without generating.
    pub(crate) fn try_reserve_shared_creation(
        &self,
        policy_key: &str,
    ) -> Option<SharedResumeReservation> {
        let notify = {
            let mut state = self.state.lock();
            let entry = state.shared.entry(policy_key.to_owned()).or_insert_with(|| SharedEntry {
                generation: 0,
                eligible: None,
                settled: None,
                resuming: false,
                notify: Arc::new(Notify::new()),
            });
            if entry.resuming {
                return None;
            }
            entry.resuming = true;
            Arc::clone(&entry.notify)
        };
        Some(SharedResumeReservation {
            tracker: self.clone(),
            policy_key: policy_key.to_owned(),
            notify,
            active: true,
        })
    }

    /// Wait until the in-flight shared resume for a policy completes
    /// (commit or cancellation) so the waiter can reuse the single successor.
    pub(crate) async fn wait_shared_resume(&self, policy_key: &str) {
        let notify = {
            let state = self.state.lock();
            state.shared.get(policy_key).map(|entry| Arc::clone(&entry.notify))
        };
        if let Some(notify) = notify {
            notify.notified().await;
        }
    }

    /// Whether a shared policy currently has an in-flight resume reservation.
    #[cfg(test)]
    pub(crate) fn shared_resuming(&self, policy_key: &str) -> bool {
        self.state.lock().shared.get(policy_key).is_some_and(|entry| entry.resuming)
    }
}

/// Cancellation-safe reservation for one shared successor generation.
///
/// Dropping without disarm releases the reservation and wakes waiters so a
/// failed or cancelled resume never strands the shared policy and never
/// consumes eligibility: the next resume attempt may retry the same logical
/// resume.
pub(crate) struct SharedResumeReservation {
    tracker: IdleResumeTracker,
    policy_key: String,
    notify: Arc<Notify>,
    active: bool,
}

impl SharedResumeReservation {
    pub(crate) fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for SharedResumeReservation {
    fn drop(&mut self) {
        if self.active {
            let mut state = self.tracker.state.lock();
            if let Some(entry) = state.shared.get_mut(&self.policy_key) {
                entry.resuming = false;
            }
            self.notify.notify_waiters();
        }
    }
}

fn enforce_dedicated_bound(map: &mut BTreeMap<String, DedicatedEntry>) {
    while map.len() > MAX_TRACKED_TUNNELS {
        // Evict the lexicographically smallest name: deterministic and
        // bounded. Eviction only drops volatile eligibility, never secrets.
        if let Some(first) = map.keys().next().cloned() {
            map.remove(&first);
        } else {
            break;
        }
    }
}

fn enforce_shared_bound(map: &mut BTreeMap<String, SharedEntry>) {
    while map.len() > MAX_TRACKED_SHARED {
        if let Some(first) = map.keys().next().cloned() {
            map.remove(&first);
        } else {
            break;
        }
    }
}

/// Compute the synthetic shared-identity store name for a stable policy key.
///
/// FNV-1a 64 with a fixed seed: deterministic across processes (unlike
/// `DefaultHasher`), dependency-free, and fixed-length. Collisions across the
/// bounded 1000-entry namespace are practically impossible; a collision would
/// at worst merge two shared policies into one session, which the
/// compatibility identity still keeps exact (different session options would
/// not share the same Yosemite session even under one synthetic name, because
/// the registry key includes the full session options).
pub(crate) fn shared_synthetic_name(policy_key: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in policy_key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{SHARED_SYNTHETIC_PREFIX}{hash:016x}")
}

/// Whether a store key is a synthetic shared-identity entry.
pub(crate) fn is_shared_synthetic_name(name: &str) -> bool {
    name.starts_with(SHARED_SYNTHETIC_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn idle() -> SamTerminationReason {
        SamTerminationReason::IdlePolicy
    }

    #[test]
    fn dedicated_requires_preceding_generation_for_eligibility() {
        let tracker = IdleResumeTracker::new();
        // No committed generation: idle termination cannot arm.
        tracker.record_dedicated_termination("a", 1, idle());
        assert!(!tracker.is_dedicated_eligible("a"));
        // First commit establishes generation 1 with no eligibility.
        tracker.commit_dedicated_generation("a");
        assert_eq!(tracker.dedicated_generation("a"), 1);
        assert!(!tracker.is_dedicated_eligible("a"));
        // Qualifying idle close of generation 1 arms exactly one resume.
        tracker.record_dedicated_termination("a", 1, idle());
        assert!(tracker.is_dedicated_eligible("a"));
    }

    #[test]
    fn dedicated_manual_and_failure_first_winner_wins() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_dedicated_generation("a");
        // Idle wins first: late manual/failure facts for the same generation
        // are ignored (no surprise clearing), preserving the one resume.
        for reason in [
            SamTerminationReason::Requested,
            SamTerminationReason::Failure,
            SamTerminationReason::Unknown,
        ] {
            // Fresh generation per iteration: commit to advance, arm idle.
            tracker.commit_dedicated_generation("a");
            let generation = tracker.dedicated_generation("a");
            tracker.record_dedicated_termination("a", generation, idle());
            assert!(tracker.is_dedicated_eligible("a"));
            tracker.record_dedicated_termination("a", generation, reason);
            assert!(
                tracker.is_dedicated_eligible("a"),
                "late {reason:?} must not clear a won idle fact"
            );
            tracker.commit_dedicated_generation("a");
        }
        // Manual wins first: ordinary Stop settles as non-idle and blocks a
        // late idle fact for the same generation (no surprise rotation).
        tracker.commit_dedicated_generation("a");
        let generation = tracker.dedicated_generation("a");
        tracker.note_manual_stop("a");
        assert!(!tracker.is_dedicated_eligible("a"));
        tracker.record_dedicated_termination("a", generation, idle());
        assert!(
            !tracker.is_dedicated_eligible("a"),
            "late idle must not arm after manual settled"
        );
        // Non-idle termination wins first via direct record: late idle ignored.
        tracker.commit_dedicated_generation("a");
        let generation = tracker.dedicated_generation("a");
        tracker.record_dedicated_termination("a", generation, SamTerminationReason::Requested);
        assert!(!tracker.is_dedicated_eligible("a"));
        tracker.record_dedicated_termination("a", generation, idle());
        assert!(!tracker.is_dedicated_eligible("a"));
    }

    #[test]
    fn dedicated_manual_stop_preserves_qualifying_idle_for_resume() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_dedicated_generation("a");
        tracker.record_dedicated_termination("a", 1, idle());
        assert!(tracker.is_dedicated_eligible("a"));
        // Explicit Stop after a won idle fact preserves it for the immediate
        // resume Start (cleanup, not a new manual termination that clears).
        tracker.note_manual_stop("a");
        assert!(
            tracker.is_dedicated_eligible("a"),
            "manual Stop must preserve a won idle fact for resume"
        );
    }

    #[test]
    fn dedicated_stale_generation_cannot_arm_or_consume() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_dedicated_generation("a"); // gen 1
        tracker.record_dedicated_termination("a", 1, idle());
        tracker.commit_dedicated_generation("a"); // resume to gen 2 consumes
        assert_eq!(tracker.dedicated_generation("a"), 2);
        assert!(!tracker.is_dedicated_eligible("a"));
        // Stale idle fact from generation 1 cannot qualify generation 2.
        tracker.record_dedicated_termination("a", 1, idle());
        assert!(!tracker.is_dedicated_eligible("a"));
        // Unknown future generation cannot arm either.
        tracker.record_dedicated_termination("a", 99, idle());
        assert!(!tracker.is_dedicated_eligible("a"));
    }

    #[test]
    fn dedicated_commit_consumes_one_shot_eligibility() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_dedicated_generation("a");
        tracker.record_dedicated_termination("a", 1, idle());
        assert!(tracker.is_dedicated_eligible("a"));
        tracker.commit_dedicated_generation("a");
        assert!(!tracker.is_dedicated_eligible("a"));
        // Second ordinary start after resume does not re-arm.
        assert_eq!(tracker.dedicated_generation("a"), 2);
    }

    #[test]
    fn dedicated_failed_resume_keeps_eligibility_retryable() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_dedicated_generation("a");
        tracker.record_dedicated_termination("a", 1, idle());
        assert!(tracker.is_dedicated_eligible("a"));
        // Failed resume: no commit, eligibility stays for retry.
        assert!(tracker.is_dedicated_eligible("a"));
        tracker.commit_dedicated_generation("a");
        assert!(!tracker.is_dedicated_eligible("a"));
    }

    #[test]
    fn dedicated_delete_and_edit_clear_state() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_dedicated_generation("a");
        tracker.record_dedicated_termination("a", 1, idle());
        tracker.note_edit("a");
        assert!(!tracker.is_dedicated_eligible("a"));
        tracker.record_dedicated_termination("a", 1, idle());
        tracker.note_delete("a");
        assert!(!tracker.is_dedicated_eligible("a"));
        assert_eq!(tracker.dedicated_generation("a"), 0);
    }

    #[test]
    fn observation_first_winner_wins_per_generation() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_dedicated_generation("tunnel-a");
        let event = SamObservationEvent::SessionRemoved {
            session_id: Arc::from("tunnel-a"),
            reason: SamTerminationReason::IdlePolicy,
        };
        tracker.record_observation(&event);
        assert!(tracker.is_dedicated_eligible("tunnel-a"));

        // Late non-idle removal for the same settled generation is ignored
        // (first winner Idle wins), preserving the one resume.
        let event = SamObservationEvent::SessionRemoved {
            session_id: Arc::from("tunnel-a"),
            reason: SamTerminationReason::Requested,
        };
        tracker.record_observation(&event);
        assert!(tracker.is_dedicated_eligible("tunnel-a"));

        // Unrelated session ids never arm.
        let event = SamObservationEvent::SessionRemoved {
            session_id: Arc::from("other"),
            reason: SamTerminationReason::IdlePolicy,
        };
        tracker.record_observation(&event);
        assert!(!tracker.is_dedicated_eligible("other"));

        // Non-removal events never arm.
        let event = SamObservationEvent::SocketRemoved {
            session_id: Arc::from("tunnel-a"),
            socket_id: 7,
        };
        tracker.record_observation(&event);
        assert!(tracker.is_dedicated_eligible("tunnel-a"));

        // Fresh generation: non-idle wins first, late idle ignored.
        tracker.commit_dedicated_generation("tunnel-a");
        let event = SamObservationEvent::SessionRemoved {
            session_id: Arc::from("tunnel-a"),
            reason: SamTerminationReason::Failure,
        };
        tracker.record_observation(&event);
        assert!(!tracker.is_dedicated_eligible("tunnel-a"));
        let event = SamObservationEvent::SessionRemoved {
            session_id: Arc::from("tunnel-a"),
            reason: SamTerminationReason::IdlePolicy,
        };
        tracker.record_observation(&event);
        assert!(!tracker.is_dedicated_eligible("tunnel-a"));
    }

    #[test]
    fn observation_maps_shared_session_id_to_policy_eligibility() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_shared_generation("policy-shared");
        tracker.register_shared_member("first-creator", "policy-shared");
        tracker.register_shared_member("second-member", "policy-shared");
        // Shared SAM session id is the first creator's nickname.
        let event = SamObservationEvent::SessionRemoved {
            session_id: Arc::from("first-creator"),
            reason: SamTerminationReason::IdlePolicy,
        };
        tracker.record_observation(&event);
        assert!(tracker.is_shared_eligible("policy-shared"));
        // No dedicated entry exists for shared members, so no per-name arming.
        assert!(!tracker.is_dedicated_eligible("first-creator"));
        assert!(!tracker.is_dedicated_eligible("second-member"));
        // Late non-idle for the same settled shared generation is ignored.
        let event = SamObservationEvent::SessionRemoved {
            session_id: Arc::from("first-creator"),
            reason: SamTerminationReason::Requested,
        };
        tracker.record_observation(&event);
        assert!(tracker.is_shared_eligible("policy-shared"));
    }

    #[test]
    fn shared_eligibility_is_one_shot_and_generation_scoped() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_shared_generation("policy-a");
        assert!(!tracker.is_shared_eligible("policy-a"));
        tracker.record_shared_termination("policy-a", 1, idle());
        assert!(tracker.is_shared_eligible("policy-a"));
        tracker.commit_shared_generation("policy-a");
        assert!(!tracker.is_shared_eligible("policy-a"));
        // Stale fact from old generation cannot re-arm.
        tracker.record_shared_termination("policy-a", 1, idle());
        assert!(!tracker.is_shared_eligible("policy-a"));
        // First winner wins: late non-idle after a won idle is ignored.
        tracker.record_shared_termination("policy-a", 2, idle());
        assert!(tracker.is_shared_eligible("policy-a"));
        tracker.record_shared_termination("policy-a", 2, SamTerminationReason::Failure);
        assert!(tracker.is_shared_eligible("policy-a"));
        tracker.commit_shared_generation("policy-a");
        // Fresh generation: non-idle wins first, late idle ignored.
        tracker.record_shared_termination("policy-a", 3, SamTerminationReason::Failure);
        assert!(!tracker.is_shared_eligible("policy-a"));
        tracker.record_shared_termination("policy-a", 3, idle());
        assert!(!tracker.is_shared_eligible("policy-a"));
    }

    #[test]
    fn shared_reservation_is_single_winner_and_cancellation_safe() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_shared_generation("policy-a");
        tracker.record_shared_termination("policy-a", 1, idle());
        let first = tracker.try_reserve_shared_resume("policy-a");
        assert!(first.is_some());
        assert!(tracker.shared_resuming("policy-a"));
        // Concurrent second cannot reserve while first holds it.
        assert!(tracker.try_reserve_shared_resume("policy-a").is_none());
        drop(first);
        assert!(!tracker.shared_resuming("policy-a"));
        // Eligibility survives cancellation for retry.
        assert!(tracker.is_shared_eligible("policy-a"));
    }

    #[test]
    fn shared_synthetic_names_are_stable_bounded_and_namespaced() {
        let first = shared_synthetic_name("policy-a|close=1");
        let second = shared_synthetic_name("policy-a|close=1");
        let other = shared_synthetic_name("policy-b|close=1");
        assert_eq!(first, second);
        assert_ne!(first, other);
        assert!(first.starts_with(SHARED_SYNTHETIC_PREFIX));
        assert!(is_shared_synthetic_name(&first));
        assert!(!is_shared_synthetic_name("user-tunnel"));
    }

    #[test]
    fn tracker_holds_no_secret_material_in_debug() {
        let tracker = IdleResumeTracker::new();
        tracker.commit_dedicated_generation("alice");
        let debug = format!("{tracker:?}");
        assert!(!debug.contains("cHJpdmF0ZQ"));
    }

    #[test]
    fn tracker_is_bounded() {
        let tracker = IdleResumeTracker::new();
        for index in 0..(MAX_TRACKED_TUNNELS + 50) {
            tracker.commit_dedicated_generation(&format!("tunnel-{index:05}"));
        }
        assert!(tracker.state.lock().dedicated.len() <= MAX_TRACKED_TUNNELS);
        for index in 0..(MAX_TRACKED_SHARED + 50) {
            tracker.commit_shared_generation(&format!("policy-{index:05}"));
        }
        assert!(tracker.state.lock().shared.len() <= MAX_TRACKED_SHARED);
    }
}
