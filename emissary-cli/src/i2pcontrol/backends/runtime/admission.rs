//! Bounded, peer-aware admission for accepted-stream server tunnels.

use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, HashMap, VecDeque},
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use parking_lot::Mutex;
use serde_json::Value;
use tokio::time::Instant;

use super::TrustedPeerIdentity;

pub const DEFAULT_MAX_CONCURRENT_CONNECTIONS: usize = 30;
pub const MAX_CONCURRENT_CONNECTIONS: usize = 128;
pub const DEFAULT_MAX_CONCURRENT_PER_PEER: usize = 8;
pub const MAX_PEER_ENTRIES: usize = 4096;
pub const MAX_RATE: u64 = 1_000_000;

const MINUTE: Duration = Duration::from_secs(60);
const HOUR: Duration = Duration::from_secs(60 * 60);
const DAY: Duration = Duration::from_secs(60 * 60 * 24);
const UNLIMITED_RETENTION: Duration = Duration::from_secs(60);

/// Reference-scale server admission policy. Zero means unlimited for a rate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerAdmissionPolicy {
    max_concurrent_connections: usize,
    max_concurrent_per_peer: usize,
    client_per_minute: u64,
    client_per_hour: u64,
    client_per_day: u64,
    total_in_per_minute: u64,
    total_in_per_hour: u64,
    total_in_per_day: u64,
    peer_capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionPolicyError {
    InvalidMaxConcurrent,
    InvalidRate,
}

impl ServerAdmissionPolicy {
    pub fn defaults() -> Self {
        Self {
            max_concurrent_connections: DEFAULT_MAX_CONCURRENT_CONNECTIONS,
            max_concurrent_per_peer: DEFAULT_MAX_CONCURRENT_PER_PEER,
            client_per_minute: 30,
            client_per_hour: 80,
            client_per_day: 200,
            total_in_per_minute: 50,
            total_in_per_hour: 0,
            total_in_per_day: 0,
            peer_capacity: MAX_PEER_ENTRIES,
        }
    }

    #[cfg(test)]
    fn with_peer_capacity(mut self, peer_capacity: usize) -> Self {
        self.peer_capacity = peer_capacity;
        self
    }

    pub fn new(
        max_concurrent_connections: u64,
        client_per_minute: u64,
        client_per_hour: u64,
        client_per_day: u64,
        total_in_per_minute: u64,
        total_in_per_hour: u64,
        total_in_per_day: u64,
    ) -> Result<Self, AdmissionPolicyError> {
        if !(1..=MAX_CONCURRENT_CONNECTIONS as u64).contains(&max_concurrent_connections) {
            return Err(AdmissionPolicyError::InvalidMaxConcurrent);
        }
        if [
            client_per_minute,
            client_per_hour,
            client_per_day,
            total_in_per_minute,
            total_in_per_hour,
            total_in_per_day,
        ]
        .iter()
        .any(|rate| *rate > MAX_RATE)
        {
            return Err(AdmissionPolicyError::InvalidRate);
        }
        let mut policy = Self::defaults();
        policy.max_concurrent_connections = max_concurrent_connections as usize;
        policy.client_per_minute = client_per_minute;
        policy.client_per_hour = client_per_hour;
        policy.client_per_day = client_per_day;
        policy.total_in_per_minute = total_in_per_minute;
        policy.total_in_per_hour = total_in_per_hour;
        policy.total_in_per_day = total_in_per_day;
        Ok(policy)
    }

    pub fn from_raw_options(raw: &BTreeMap<String, Value>) -> Result<Self, &'static str> {
        let defaults = Self::defaults();
        let value = |key: &'static str, default: u64| {
            raw.get(key)
                .map(|value| value.as_u64().ok_or(key))
                .transpose()
                .map(|value| value.unwrap_or(default))
        };
        let max = value(
            "MaxConcurrentConns",
            defaults.max_concurrent_connections as u64,
        )?;
        let client_per_minute = value("ClientPerMinute", defaults.client_per_minute)?;
        let client_per_hour = value("ClientPerHour", defaults.client_per_hour)?;
        let client_per_day = value("ClientPerDay", defaults.client_per_day)?;
        let total_in_per_minute = value("TotalInPerMinute", defaults.total_in_per_minute)?;
        let total_in_per_hour = value("TotalInPerHour", defaults.total_in_per_hour)?;
        let total_in_per_day = value("TotalInPerDay", defaults.total_in_per_day)?;
        Self::new(
            max,
            client_per_minute,
            client_per_hour,
            client_per_day,
            total_in_per_minute,
            total_in_per_hour,
            total_in_per_day,
        )
        .map_err(|error| match error {
            AdmissionPolicyError::InvalidMaxConcurrent => "MaxConcurrentConns",
            AdmissionPolicyError::InvalidRate => "connection rate",
        })
    }

    pub fn max_concurrent_connections(&self) -> usize {
        self.max_concurrent_connections
    }
}

/// Fixed-size identity used for accounting. The trusted textual destination
/// remains owned by the accepted connection and is never stored here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PeerKey([u8; 8]);

impl PeerKey {
    fn from_identity(peer: &TrustedPeerIdentity) -> Self {
        let mut hasher = DefaultHasher::new();
        peer.destination().hash(&mut hasher);
        Self(hasher.finish().to_be_bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionRejection {
    GlobalConcurrency,
    PeerConcurrency,
    PeerRate,
    AggregateRate,
    PeerStateCapacity,
}

/// Result of a common admission check. A denied stream is intentionally left
/// to its caller to drop; no protocol bytes are required for rejection.
pub enum AdmissionDecision {
    Allowed(AdmissionLease),
    Denied(AdmissionRejection),
}

impl std::fmt::Debug for AdmissionDecision {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allowed(_) => formatter.write_str("Allowed(..)"),
            Self::Denied(reason) => formatter.debug_tuple("Denied").field(reason).finish(),
        }
    }
}

struct Window {
    started: Instant,
    count: u64,
}

impl Window {
    fn new(now: Instant) -> Self {
        Self {
            started: now,
            count: 0,
        }
    }

    fn current_count(&mut self, now: Instant, duration: Duration) -> u64 {
        if now.saturating_duration_since(self.started) >= duration {
            self.started = now;
            self.count = 0;
        }
        self.count
    }

    fn allow(&mut self, now: Instant, duration: Duration, limit: u64) -> bool {
        limit == 0 || self.current_count(now, duration) < limit
    }

    fn record(&mut self, now: Instant, duration: Duration) {
        self.current_count(now, duration);
        self.count = self.count.saturating_add(1);
    }

    fn expires_at(&self, duration: Duration) -> Instant {
        self.started + duration
    }
}

struct Counters {
    minute: Window,
    hour: Window,
    day: Window,
}

impl Counters {
    fn new(now: Instant) -> Self {
        Self {
            minute: Window::new(now),
            hour: Window::new(now),
            day: Window::new(now),
        }
    }

    fn allow(&mut self, now: Instant, minute: u64, hour: u64, day: u64) -> bool {
        self.minute.allow(now, MINUTE, minute)
            && self.hour.allow(now, HOUR, hour)
            && self.day.allow(now, DAY, day)
    }

    fn record(&mut self, now: Instant) {
        self.minute.record(now, MINUTE);
        self.hour.record(now, HOUR);
        self.day.record(now, DAY);
    }

    fn expires_at(
        &self,
        now: Instant,
        retention: Duration,
        minute: u64,
        hour: u64,
        day: u64,
    ) -> Instant {
        let mut expiry = now + retention;
        if minute != 0 {
            expiry = expiry.max(self.minute.expires_at(MINUTE));
        }
        if hour != 0 {
            expiry = expiry.max(self.hour.expires_at(HOUR));
        }
        if day != 0 {
            expiry = expiry.max(self.day.expires_at(DAY));
        }
        expiry
    }
}

struct PeerRecord {
    active: usize,
    counters: Counters,
    expires_at: Instant,
}

struct ExpiryEntry {
    at: Instant,
    key: PeerKey,
}

struct State {
    active: usize,
    aggregate: Counters,
    peers: HashMap<PeerKey, PeerRecord>,
    expirations: VecDeque<ExpiryEntry>,
    retention: Duration,
}

impl State {
    fn new(policy: &ServerAdmissionPolicy) -> Self {
        let now = Instant::now();
        let retention = if policy.client_per_day != 0
            || policy.client_per_hour != 0
            || policy.client_per_minute != 0
        {
            DAY
        } else {
            UNLIMITED_RETENTION
        };
        Self {
            active: 0,
            aggregate: Counters::new(now),
            peers: HashMap::new(),
            expirations: VecDeque::new(),
            retention,
        }
    }

    fn reap(&mut self, now: Instant) {
        while self.expirations.front().is_some_and(|entry| entry.at <= now) {
            let entry = self.expirations.pop_front().expect("front exists");
            let remove = self
                .peers
                .get(&entry.key)
                .is_some_and(|peer| peer.active == 0 && peer.expires_at <= now);
            if remove {
                self.peers.remove(&entry.key);
            }
        }
    }

    fn queue_expiry(&mut self, key: PeerKey, expires_at: Instant) {
        self.expirations.push_back(ExpiryEntry {
            at: expires_at,
            key,
        });
    }
}

struct AdmissionInner {
    policy: ServerAdmissionPolicy,
    state: Mutex<State>,
}

/// Ephemeral state for one accepted-server runtime generation.
#[derive(Clone)]
pub struct ServerAdmissionState {
    inner: Arc<AdmissionInner>,
}

impl ServerAdmissionState {
    pub fn new(policy: ServerAdmissionPolicy) -> Self {
        Self {
            inner: Arc::new(AdmissionInner {
                state: Mutex::new(State::new(&policy)),
                policy,
            }),
        }
    }

    pub fn try_acquire(&self, peer: &TrustedPeerIdentity) -> AdmissionDecision {
        let now = Instant::now();
        let key = PeerKey::from_identity(peer);
        let mut state = self.inner.state.lock();
        state.reap(now);

        let is_new = !state.peers.contains_key(&key);
        if is_new && state.peers.len() >= self.inner.policy.peer_capacity {
            return AdmissionDecision::Denied(AdmissionRejection::PeerStateCapacity);
        }
        if state.active >= self.inner.policy.max_concurrent_connections {
            return AdmissionDecision::Denied(AdmissionRejection::GlobalConcurrency);
        }

        if !state.aggregate.allow(
            now,
            self.inner.policy.total_in_per_minute,
            self.inner.policy.total_in_per_hour,
            self.inner.policy.total_in_per_day,
        ) {
            return AdmissionDecision::Denied(AdmissionRejection::AggregateRate);
        }

        if is_new {
            if state.peers.try_reserve(1).is_err() {
                return AdmissionDecision::Denied(AdmissionRejection::PeerStateCapacity);
            }
            let retention = state.retention;
            state.peers.insert(
                key,
                PeerRecord {
                    active: 0,
                    counters: Counters::new(now),
                    expires_at: now + retention,
                },
            );
        }
        let peer_record = state.peers.get_mut(&key).expect("peer inserted above");
        if peer_record.active >= self.inner.policy.max_concurrent_per_peer {
            return AdmissionDecision::Denied(AdmissionRejection::PeerConcurrency);
        }
        if !peer_record.counters.allow(
            now,
            self.inner.policy.client_per_minute,
            self.inner.policy.client_per_hour,
            self.inner.policy.client_per_day,
        ) {
            return AdmissionDecision::Denied(AdmissionRejection::PeerRate);
        }

        let retention = state.retention;
        let expires_at = {
            let peer_record = state.peers.get_mut(&key).expect("peer inserted above");
            peer_record.active += 1;
            peer_record.counters.record(now);
            peer_record.expires_at = peer_record.counters.expires_at(
                now,
                retention,
                self.inner.policy.client_per_minute,
                self.inner.policy.client_per_hour,
                self.inner.policy.client_per_day,
            );
            peer_record.expires_at
        };
        state.queue_expiry(key, expires_at);
        state.aggregate.record(now);
        state.active += 1;
        drop(state);

        AdmissionDecision::Allowed(AdmissionLease {
            inner: Some(Arc::clone(&self.inner)),
            key,
        })
    }

    #[cfg(test)]
    fn active_counts(&self) -> (usize, usize) {
        let state = self.inner.state.lock();
        (
            state.active,
            state.peers.values().map(|peer| peer.active).sum(),
        )
    }

    #[cfg(test)]
    fn peer_state_len(&self) -> usize {
        self.inner.state.lock().peers.len()
    }
}

/// Exact ownership of one global and one peer active-count reservation.
pub struct AdmissionLease {
    inner: Option<Arc<AdmissionInner>>,
    key: PeerKey,
}

impl Drop for AdmissionLease {
    fn drop(&mut self) {
        let Some(inner) = self.inner.take() else {
            return;
        };
        let now = Instant::now();
        let mut state = inner.state.lock();
        state.active = state.active.saturating_sub(1);
        if let Some(peer) = state.peers.get_mut(&self.key) {
            peer.active = peer.active.saturating_sub(1);
            if peer.active == 0 {
                let expires_at = peer.expires_at.max(now);
                let _ = peer;
                state.queue_expiry(self.key, expires_at);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(value: &str) -> TrustedPeerIdentity {
        TrustedPeerIdentity::for_test(value)
    }

    #[tokio::test(start_paused = true)]
    async fn defaults_and_global_limit_are_finite() {
        let policy = ServerAdmissionPolicy::defaults();
        assert_eq!(policy.max_concurrent_connections(), 30);
        let state =
            ServerAdmissionState::new(ServerAdmissionPolicy::new(30, 0, 0, 0, 0, 0, 0).unwrap());
        let leases = (0..30)
            .map(|index| {
                let peer = peer(&format!("peer-{index}"));
                match state.try_acquire(&peer) {
                    AdmissionDecision::Allowed(lease) => lease,
                    AdmissionDecision::Denied(reason) => panic!("unexpected denial: {reason:?}"),
                }
            })
            .collect::<Vec<_>>();
        assert!(matches!(
            state.try_acquire(&peer("one-more")),
            AdmissionDecision::Denied(AdmissionRejection::GlobalConcurrency)
        ));
        drop(leases);
        assert_eq!(state.active_counts(), (0, 0));
    }

    #[tokio::test(start_paused = true)]
    async fn peer_fairness_preserves_other_peer_capacity() {
        let policy = ServerAdmissionPolicy::new(20, 0, 0, 0, 0, 0, 0).unwrap();
        let state = ServerAdmissionState::new(policy);
        let first = peer("first");
        let mut leases = Vec::new();
        for _ in 0..8 {
            if let AdmissionDecision::Allowed(lease) = state.try_acquire(&first) {
                leases.push(lease);
            }
        }
        assert_eq!(leases.len(), DEFAULT_MAX_CONCURRENT_PER_PEER);
        assert!(matches!(
            state.try_acquire(&peer("second")),
            AdmissionDecision::Allowed(_)
        ));
        drop(leases.pop());
        assert!(matches!(
            state.try_acquire(&peer("second")),
            AdmissionDecision::Allowed(_)
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn peer_rate_and_aggregate_windows_expire_without_sleeping() {
        let policy = ServerAdmissionPolicy::new(10, 2, 0, 0, 3, 0, 0).unwrap();
        let state = ServerAdmissionState::new(policy);
        let first = peer("first");
        let second = peer("second");
        let _a = match state.try_acquire(&first) {
            AdmissionDecision::Allowed(lease) => lease,
            other => panic!("unexpected result: {other:?}"),
        };
        let _b = match state.try_acquire(&first) {
            AdmissionDecision::Allowed(lease) => lease,
            other => panic!("unexpected result: {other:?}"),
        };
        assert!(matches!(
            state.try_acquire(&first),
            AdmissionDecision::Denied(AdmissionRejection::PeerRate)
        ));
        assert!(matches!(
            state.try_acquire(&second),
            AdmissionDecision::Allowed(_)
        ));
        tokio::time::advance(MINUTE).await;
        assert!(matches!(
            state.try_acquire(&first),
            AdmissionDecision::Allowed(_)
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn peer_hour_and_day_windows_are_independent_and_deterministic() {
        let policy = ServerAdmissionPolicy::new(10, 1, 2, 3, 0, 0, 0).unwrap();
        let state = ServerAdmissionState::new(policy);
        let first = peer("first");
        let _one = match state.try_acquire(&first) {
            AdmissionDecision::Allowed(lease) => lease,
            other => panic!("unexpected result: {other:?}"),
        };
        tokio::time::advance(MINUTE).await;
        let _two = match state.try_acquire(&first) {
            AdmissionDecision::Allowed(lease) => lease,
            other => panic!("unexpected result: {other:?}"),
        };
        tokio::time::advance(MINUTE).await;
        assert!(matches!(
            state.try_acquire(&first),
            AdmissionDecision::Denied(AdmissionRejection::PeerRate)
        ));
        tokio::time::advance(HOUR - MINUTE * 2).await;
        assert!(matches!(
            state.try_acquire(&first),
            AdmissionDecision::Allowed(_)
        ));
        tokio::time::advance(DAY - HOUR).await;
        assert!(matches!(
            state.try_acquire(&first),
            AdmissionDecision::Allowed(_)
        ));
    }

    #[test]
    fn raw_policy_defaults_and_rejects_invalid_ranges() {
        let empty = BTreeMap::new();
        assert_eq!(
            ServerAdmissionPolicy::from_raw_options(&empty)
                .unwrap()
                .max_concurrent_connections(),
            DEFAULT_MAX_CONCURRENT_CONNECTIONS
        );
        let zero = BTreeMap::from([("MaxConcurrentConns".to_owned(), Value::from(0u64))]);
        assert_eq!(
            ServerAdmissionPolicy::from_raw_options(&zero),
            Err("MaxConcurrentConns")
        );
        let huge = BTreeMap::from([("ClientPerMinute".to_owned(), Value::from(MAX_RATE + 1))]);
        assert_eq!(
            ServerAdmissionPolicy::from_raw_options(&huge),
            Err("connection rate")
        );
    }

    #[tokio::test(start_paused = true)]
    async fn full_peer_table_does_not_evict_active_state_and_reclaims_expired_state() {
        let policy =
            ServerAdmissionPolicy::new(10, 0, 0, 0, 0, 0, 0).unwrap().with_peer_capacity(1);
        let state = ServerAdmissionState::new(policy);
        let lease = match state.try_acquire(&peer("active")) {
            AdmissionDecision::Allowed(lease) => lease,
            other => panic!("unexpected result: {other:?}"),
        };
        assert!(matches!(
            state.try_acquire(&peer("new")),
            AdmissionDecision::Denied(AdmissionRejection::PeerStateCapacity)
        ));
        drop(lease);
        tokio::time::advance(UNLIMITED_RETENTION).await;
        assert!(matches!(
            state.try_acquire(&peer("new")),
            AdmissionDecision::Allowed(_)
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn aggregate_rate_denial_does_not_allocate_new_peer_state() {
        let policy =
            ServerAdmissionPolicy::new(10, 0, 0, 0, 1, 0, 0).unwrap().with_peer_capacity(2);
        let state = ServerAdmissionState::new(policy);
        let lease = match state.try_acquire(&peer("first")) {
            AdmissionDecision::Allowed(lease) => lease,
            other => panic!("unexpected result: {other:?}"),
        };

        assert!(matches!(
            state.try_acquire(&peer("second")),
            AdmissionDecision::Denied(AdmissionRejection::AggregateRate)
        ));
        assert_eq!(state.peer_state_len(), 1);
        drop(lease);
    }
}
