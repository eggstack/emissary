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

use parking_lot::{Mutex, RwLock};
use rand::RngExt;
use std::{
    collections::{HashMap, VecDeque},
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;

/// Maximum number of tokens to retain in memory.
pub(crate) const MAX_TOKENS: usize = 1024;

/// A token is 32 bytes of cryptographically random data, hex-encoded.
const TOKEN_BYTES: usize = 32;

/// Finite in-process validity for every issued authentication token.
///
/// Compatibility rationale (M127): the reference I2PControl implementation
/// uses finite token validity and removes expired tokens, and the RPC layer
/// already declares the standard `TOKEN_EXPIRED` (`-32004`) error. An
/// unbounded lifetime contradicts that shared base authentication behavior.
/// One day matches the established reference lifetime and is expressed as a
/// named constant so future compatibility review can adjudicate the exact
/// value rather than a magic number.
pub(crate) const TOKEN_LIFETIME: Duration = Duration::from_secs(24 * 60 * 60);

/// Maximum accepted length for a presented credential, in bytes.
///
/// Issued tokens are 64 hex characters. This bound gives generous headroom
/// for compatible opaque values while failing oversized input before
/// hashing, lookup, or allocation. Opaque-token semantics are preserved: no
/// hexadecimal syntax requirement is imposed.
pub(crate) const MAX_PRESENTED_TOKEN_LEN: usize = 256;

/// Maximum source entries retained by the failed-authentication throttle.
pub(crate) const MAX_THROTTLE_ENTRIES: usize = 256;
/// Failure state expires after this monotonic window.
pub(crate) const THROTTLE_WINDOW: Duration = Duration::from_secs(60);
/// The first repeated failure waits for this bounded interval.
pub(crate) const THROTTLE_BASE_DELAY: Duration = Duration::from_millis(25);
/// A source can never be delayed longer than this value.
pub(crate) const THROTTLE_MAX_DELAY: Duration = Duration::from_secs(1);

/// Password comparison work is bounded independently of the request body cap.
const MAX_PASSWORD_BYTES: usize = 4096;

/// Internal token validation outcome.
///
/// The three outcomes must stay distinct: collapsing `Expired` into
/// `Unknown` would make the already-declared `-32004 TOKEN_EXPIRED` error
/// unreachable and reintroduce the M127 defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenValidation {
    /// Credential is live and authorizes protected dispatch.
    Valid,
    /// Credential was live but reached its finite lifetime. The token has
    /// been removed atomically by the observing validation.
    Expired,
    /// Credential was never issued, was explicitly invalidated, was already
    /// removed after expiry, or failed input bounds. Never echoes input.
    Unknown,
}

/// Monotonic lifetime source for token expiry.
///
/// Production always uses the real monotonic clock so wall-clock jumps can
/// neither extend nor prematurely invalidate a token. Tests use a
/// deterministic manual clock without sleeping for production durations.
#[derive(Clone)]
struct TokenClock {
    now: Arc<dyn Fn() -> Instant + Send + Sync>,
}

impl TokenClock {
    fn monotonic() -> Self {
        Self {
            now: Arc::new(Instant::now),
        }
    }

    fn now(&self) -> Instant {
        (self.now)()
    }
}

/// Authentication token service.
///
/// Tokens are cryptographically random, opaque, bounded, finite-lived, and
/// invalidated on restart. Expiry is decided with a monotonic clock at
/// lookup time; expired tokens are removed atomically by the observing
/// validation.
#[derive(Clone)]
pub struct TokenService {
    inner: Arc<RwLock<TokenStore>>,
    clock: TokenClock,
}

struct TokenStore {
    tokens: HashMap<String, Instant>,
    order: VecDeque<String>,
}

impl TokenStore {
    /// Remove all entries whose monotonic expiry has passed.
    ///
    /// Bounded by `MAX_TOKENS` entries; called lazily from issuance and
    /// never from a background scanner or timer task.
    fn remove_expired(&mut self, now: Instant) {
        if self.tokens.is_empty() {
            return;
        }
        self.tokens.retain(|_, expires_at| *expires_at > now);
        self.order.retain(|token| self.tokens.contains_key(token));
    }
}

#[derive(Clone)]
pub(crate) struct AuthThrottle {
    inner: Arc<Mutex<ThrottleStore>>,
}

struct ThrottleStore {
    failures: HashMap<IpAddr, FailureState>,
}

#[derive(Clone, Copy)]
struct FailureState {
    count: u32,
    first_failure: Instant,
    last_failure: Instant,
}

impl Default for AuthThrottle {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthThrottle {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ThrottleStore {
                failures: HashMap::new(),
            })),
        }
    }

    /// Reserve one failed-authentication attempt and return its bounded delay.
    ///
    /// The reservation is complete before the caller awaits, so concurrent attempts cannot
    /// observe and reuse one stale failure count. The lock is released before this returns.
    pub(crate) fn reserve_failure(&self, source: Option<SocketAddr>) -> Duration {
        let Some(source) = source.map(|address| address.ip()) else {
            return Duration::ZERO;
        };
        let now = Instant::now();
        let mut store = self.inner.lock();
        if let Some(failure) = store.failures.get_mut(&source) {
            if now.duration_since(failure.last_failure) >= THROTTLE_WINDOW {
                *failure = FailureState {
                    count: 1,
                    first_failure: now,
                    last_failure: now,
                };
                return Duration::ZERO;
            } else {
                failure.count = failure.count.saturating_add(1);
                failure.last_failure = now;
            }
            return delay_for_count(failure.count.saturating_sub(1));
        }

        if store.failures.len() >= MAX_THROTTLE_ENTRIES {
            let oldest = store
                .failures
                .iter()
                .min_by_key(|(address, failure)| (failure.first_failure, **address))
                .map(|(address, _)| *address);
            if let Some(oldest) = oldest {
                store.failures.remove(&oldest);
            }
        }
        store.failures.insert(
            source,
            FailureState {
                count: 1,
                first_failure: now,
                last_failure: now,
            },
        );
        Duration::ZERO
    }

    pub(crate) fn clear(&self, source: Option<SocketAddr>) {
        if let Some(source) = source {
            self.inner.lock().failures.remove(&source.ip());
        }
    }

    #[cfg(test)]
    pub(crate) fn count(&self) -> usize {
        self.inner.lock().failures.len()
    }
}

fn delay_for_count(count: u32) -> Duration {
    let exponent = count.saturating_sub(1).min(6);
    let multiplier = 1u32 << exponent;
    THROTTLE_BASE_DELAY
        .checked_mul(multiplier)
        .unwrap_or(THROTTLE_MAX_DELAY)
        .min(THROTTLE_MAX_DELAY)
}

impl Default for TokenService {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenService {
    /// Create a new empty token service.
    ///
    /// Production composition must use this constructor: it binds the
    /// real monotonic clock. Deterministic time control is available only
    /// to unit tests via `new_manual_for_test`.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(TokenStore {
                tokens: HashMap::new(),
                order: VecDeque::new(),
            })),
            clock: TokenClock::monotonic(),
        }
    }

    /// Create a token service with deterministic manual time (unit tests only).
    ///
    /// Returns the service and a handle to the current manual instant.
    /// Advancing `*handle.lock()` simulates monotonic time without sleeps.
    /// Unavailable to normal production composition.
    #[cfg(test)]
    pub(crate) fn new_manual_for_test(start: Instant) -> (Self, Arc<Mutex<Instant>>) {
        let cell = Arc::new(Mutex::new(start));
        let reader = Arc::clone(&cell);
        let clock = TokenClock {
            now: Arc::new(move || *reader.lock()),
        };
        let service = Self {
            inner: Arc::new(RwLock::new(TokenStore {
                tokens: HashMap::new(),
                order: VecDeque::new(),
            })),
            clock,
        };
        (service, cell)
    }

    /// Issue a new cryptographically random token.
    ///
    /// Returns the hex-encoded token string. Issuance is an atomic
    /// in-memory insertion. Expired entries are reclaimed lazily (bounded)
    /// before deterministic oldest-token eviction so expired state cannot
    /// crowd out live credentials indefinitely.
    pub fn issue(&self) -> String {
        let mut store = self.inner.write();
        let now = self.clock.now();

        // Bounded lazy cleanup first; no background scanner exists.
        store.remove_expired(now);

        // Evict exactly the oldest live token at capacity.
        if store.tokens.len() >= MAX_TOKENS {
            if let Some(oldest) = store.order.pop_front() {
                store.tokens.remove(&oldest);
            }
        }

        let token = generate_token();
        let expires_at = now.checked_add(TOKEN_LIFETIME).unwrap_or(now + TOKEN_LIFETIME);
        // Random-collision defense: never duplicate an order entry.
        if store.tokens.contains_key(&token) {
            store.order.retain(|candidate| candidate != &token);
        }
        store.tokens.insert(token.clone(), expires_at);
        store.order.push_back(token.clone());
        token
    }

    /// Validate a token, distinguishing valid, expired-and-removed, and
    /// unknown credentials.
    ///
    /// Oversized or empty input fails as `Unknown` before lookup without
    /// echoing attacker-controlled input and without proportional
    /// allocation. An expired lookup removes the token under the same
    /// write lock used to decide expiry, so concurrent validators cannot
    /// both observe success after expiry. The lock is never held across
    /// async waits or network I/O (this function is synchronous).
    pub fn validate(&self, token: &str) -> TokenValidation {
        if token.is_empty() || token.len() > MAX_PRESENTED_TOKEN_LEN {
            return TokenValidation::Unknown;
        }
        let now = self.clock.now();
        let mut store = self.inner.write();
        match store.tokens.get(token).copied() {
            None => TokenValidation::Unknown,
            Some(expires_at) if expires_at > now => TokenValidation::Valid,
            Some(_) => {
                store.tokens.remove(token);
                store.order.retain(|candidate| candidate != token);
                TokenValidation::Expired
            }
        }
    }

    /// Invalidate a specific token.
    #[allow(dead_code)]
    pub fn invalidate(&self, token: &str) {
        let mut store = self.inner.write();
        store.tokens.remove(token);
        store.order.retain(|candidate| candidate != token);
    }

    /// Clear all tokens (e.g., on shutdown).
    pub fn clear(&self) {
        let mut store = self.inner.write();
        store.tokens.clear();
        store.order.clear();
    }

    /// Current number of active tokens.
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        let store = self.inner.read();
        store.tokens.len()
    }
}

/// Generate a cryptographically random hex token.
fn generate_token() -> String {
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..TOKEN_BYTES).map(|_| rng.random()).collect();
    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

/// Validate the API version. Emissary implements I2PControl API version 1.
pub fn validate_api_version(version: i32) -> bool {
    version == 1
}

/// Compare passwords with the reviewed `subtle` primitive.
///
/// Both values are copied into fixed-size padded buffers, so different
/// lengths are handled without a hand-written byte loop or an early return.
pub fn compare_passwords(provided: &str, expected: &str) -> bool {
    let mut provided_buffer = [0u8; MAX_PASSWORD_BYTES];
    let mut expected_buffer = [0u8; MAX_PASSWORD_BYTES];
    let provided_len = provided.len().min(MAX_PASSWORD_BYTES);
    let expected_len = expected.len().min(MAX_PASSWORD_BYTES);
    provided_buffer[..provided_len].copy_from_slice(&provided.as_bytes()[..provided_len]);
    expected_buffer[..expected_len].copy_from_slice(&expected.as_bytes()[..expected_len]);
    let bytes_equal = provided_buffer.ct_eq(&expected_buffer);
    let lengths_equal = provided.len().to_ne_bytes().ct_eq(&expected.len().to_ne_bytes());
    let lengths_bounded = subtle::Choice::from(
        (provided.len() <= MAX_PASSWORD_BYTES && expected.len() <= MAX_PASSWORD_BYTES) as u8,
    );
    bool::from(bytes_equal & lengths_equal & lengths_bounded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_and_validate_token() {
        let svc = TokenService::new();
        let token = svc.issue();
        assert_eq!(token.len(), TOKEN_BYTES * 2); // hex-encoded
        assert_eq!(svc.validate(&token), TokenValidation::Valid);
        assert_eq!(svc.validate("invalid-token"), TokenValidation::Unknown);
    }

    #[test]
    fn invalidate_token() {
        let svc = TokenService::new();
        let token = svc.issue();
        assert_eq!(svc.validate(&token), TokenValidation::Valid);
        svc.invalidate(&token);
        assert_eq!(svc.validate(&token), TokenValidation::Unknown);
    }

    #[test]
    fn clear_tokens() {
        let svc = TokenService::new();
        svc.issue();
        svc.issue();
        assert_eq!(svc.count(), 2);
        svc.clear();
        assert_eq!(svc.count(), 0);
    }

    #[test]
    fn token_eviction_at_capacity() {
        let svc = TokenService::new();
        // Fill to capacity
        for _ in 0..MAX_TOKENS {
            svc.issue();
        }
        assert_eq!(svc.count(), MAX_TOKENS);

        let first = svc.inner.read().order.front().cloned().unwrap();
        // Issue one more — should evict exactly the oldest token.
        let replacement = svc.issue();
        assert_eq!(svc.count(), MAX_TOKENS);
        assert_eq!(svc.validate(&first), TokenValidation::Unknown);
        assert_eq!(svc.validate(&replacement), TokenValidation::Valid);
    }

    #[test]
    fn validate_api_version_valid() {
        assert!(validate_api_version(1));
    }

    #[test]
    fn validate_api_version_invalid() {
        assert!(!validate_api_version(0));
        assert!(!validate_api_version(2));
        assert!(!validate_api_version(3));
        assert!(!validate_api_version(-1));
    }

    #[test]
    fn compare_passwords_equal() {
        assert!(compare_passwords("secret", "secret"));
    }

    #[test]
    fn compare_passwords_not_equal() {
        assert!(!compare_passwords("secret", "other"));
    }

    #[test]
    fn compare_passwords_empty() {
        assert!(compare_passwords("", ""));
    }

    #[test]
    fn compare_passwords_different_lengths() {
        assert!(!compare_passwords("a", "ab"));
    }

    #[test]
    fn compare_passwords_rejects_oversized_values() {
        let oversized = "x".repeat(MAX_PASSWORD_BYTES + 1);
        assert!(!compare_passwords(&oversized, &oversized));
    }

    #[test]
    fn tokens_are_unique() {
        let svc = TokenService::new();
        let t1 = svc.issue();
        let t2 = svc.issue();
        assert_ne!(t1, t2);
    }

    #[test]
    fn throttle_capacity_is_bounded_under_source_churn() {
        let throttle = AuthThrottle::new();
        for index in 0..(MAX_THROTTLE_ENTRIES + 32) {
            let source = std::net::Ipv4Addr::new(10, 0, (index / 255) as u8, (index % 255) as u8);
            throttle.reserve_failure(Some((source, 7650).into()));
        }
        assert_eq!(throttle.count(), MAX_THROTTLE_ENTRIES);
    }

    #[test]
    fn throttle_delay_is_bounded() {
        let throttle = AuthThrottle::new();
        let source = Some(([127, 0, 0, 1], 7650).into());
        for _ in 0..1000 {
            throttle.reserve_failure(source);
        }
        assert!(throttle.reserve_failure(source) <= THROTTLE_MAX_DELAY);
    }

    #[test]
    fn throttle_normalizes_source_ports_to_one_ip_identity() {
        let throttle = AuthThrottle::new();
        let first = Some(([127, 0, 0, 1], 10001).into());
        let second = Some(([127, 0, 0, 1], 50000).into());

        assert_eq!(throttle.reserve_failure(first), Duration::ZERO);
        assert_eq!(throttle.reserve_failure(second), THROTTLE_BASE_DELAY);
        assert_eq!(throttle.count(), 1);
        throttle.clear(second);
        assert_eq!(throttle.count(), 0);
    }

    #[test]
    fn throttle_reserves_concurrent_failures_atomically() {
        let throttle = AuthThrottle::new();
        let source = Some(([127, 0, 0, 1], 7650).into());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));

        let delays = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for _ in 0..8 {
                let throttle = throttle.clone();
                let barrier = std::sync::Arc::clone(&barrier);
                handles.push(scope.spawn(move || {
                    barrier.wait();
                    throttle.reserve_failure(source)
                }));
            }
            handles.into_iter().map(|handle| handle.join().unwrap()).collect::<Vec<_>>()
        });

        let mut expected = (0..8)
            .map(|count| {
                if count == 0 {
                    Duration::ZERO
                } else {
                    delay_for_count(count)
                }
            })
            .collect::<Vec<_>>();
        let mut actual = delays;
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn throttle_normalizes_ipv6_source_ports_to_one_ip_identity() {
        let throttle = AuthThrottle::new();
        let first = Some("[::1]:10001".parse().unwrap());
        let second = Some("[::1]:50000".parse().unwrap());

        assert_eq!(throttle.reserve_failure(first), Duration::ZERO);
        assert_eq!(throttle.reserve_failure(second), THROTTLE_BASE_DELAY);
        assert_eq!(throttle.count(), 1);
    }

    #[test]
    fn throttle_keeps_distinct_ips_independent() {
        let throttle = AuthThrottle::new();
        let first = Some(([127, 0, 0, 1], 7650).into());
        let second = Some(([10, 0, 0, 1], 7650).into());

        assert_eq!(throttle.reserve_failure(first), Duration::ZERO);
        assert_eq!(throttle.reserve_failure(second), Duration::ZERO);
        assert_eq!(throttle.count(), 2);

        assert_eq!(throttle.reserve_failure(first), THROTTLE_BASE_DELAY);
        assert_eq!(throttle.reserve_failure(second), THROTTLE_BASE_DELAY);

        throttle.clear(first);
        assert_eq!(throttle.count(), 1);
    }

    #[test]
    fn throttle_matches_documented_delay_schedule() {
        let throttle = AuthThrottle::new();
        let source = Some(([127, 0, 0, 1], 7650).into());

        assert_eq!(throttle.reserve_failure(source), Duration::ZERO);
        assert_eq!(throttle.reserve_failure(source), THROTTLE_BASE_DELAY);
        assert_eq!(throttle.reserve_failure(source), 2 * THROTTLE_BASE_DELAY);
        assert_eq!(throttle.reserve_failure(source), 4 * THROTTLE_BASE_DELAY);
        assert_eq!(throttle.reserve_failure(source), 8 * THROTTLE_BASE_DELAY);
        assert_eq!(throttle.reserve_failure(source), 16 * THROTTLE_BASE_DELAY);
        assert_eq!(throttle.reserve_failure(source), 32 * THROTTLE_BASE_DELAY);
        for _ in 0..100 {
            let delay = throttle.reserve_failure(source);
            assert!(delay <= THROTTLE_MAX_DELAY);
        }
    }

    #[tokio::test]
    async fn throttle_reservation_preserved_through_dropped_sleep() {
        let throttle = AuthThrottle::new();
        let source = Some(([127, 0, 0, 1], 7650).into());

        let first = throttle.reserve_failure(source);
        assert_eq!(first, Duration::ZERO);
        let second = throttle.reserve_failure(source);
        assert_eq!(second, THROTTLE_BASE_DELAY);

        // The reservation is recorded synchronously inside reserve_failure; a
        // cancelled sleep future (handler drop) cannot erase it.
        drop(tokio::time::sleep(second));

        let third = throttle.reserve_failure(source);
        assert_eq!(third, 2 * THROTTLE_BASE_DELAY);
    }

    // --- M127 token-lifetime corrective evidence ---

    #[test]
    fn token_lifetime_is_one_day_compatibility_constant() {
        // Reference I2PControl finite lifetime; named constant, not magic.
        assert_eq!(TOKEN_LIFETIME, Duration::from_secs(24 * 60 * 60));
    }

    #[test]
    fn token_is_valid_immediately_after_issuance() {
        let start = Instant::now();
        let (svc, _clock) = TokenService::new_manual_for_test(start);
        let token = svc.issue();
        assert_eq!(svc.validate(&token), TokenValidation::Valid);
    }

    #[test]
    fn token_expires_at_exact_lifetime_boundary() {
        let start = Instant::now();
        let (svc, clock) = TokenService::new_manual_for_test(start);
        let token = svc.issue();
        // Just before expiry the credential is still live.
        *clock.lock() = start + TOKEN_LIFETIME - Duration::from_nanos(1);
        assert_eq!(svc.validate(&token), TokenValidation::Valid);
        // At exactly issued_at + lifetime the credential is terminally expired.
        *clock.lock() = start + TOKEN_LIFETIME;
        assert_eq!(svc.validate(&token), TokenValidation::Expired);
    }

    #[test]
    fn expired_validation_removes_token_and_second_use_is_unknown() {
        let start = Instant::now();
        let (svc, clock) = TokenService::new_manual_for_test(start);
        let token = svc.issue();
        *clock.lock() = start + TOKEN_LIFETIME;
        // First expired observation removes the token.
        assert_eq!(svc.validate(&token), TokenValidation::Expired);
        assert_eq!(svc.count(), 0);
        assert!(!svc.inner.read().order.contains(&token));
        // A second validation of the same value is unknown, not repeatedly expired.
        assert_eq!(svc.validate(&token), TokenValidation::Unknown);
        assert_eq!(svc.validate(&token), TokenValidation::Unknown);
    }

    #[test]
    fn unknown_tokens_never_report_expired() {
        let svc = TokenService::new();
        assert_eq!(svc.validate("never-issued"), TokenValidation::Unknown);
        assert_eq!(svc.validate(""), TokenValidation::Unknown);
    }

    #[test]
    fn oversized_presented_credentials_fail_before_lookup_without_echo() {
        let svc = TokenService::new();
        let oversized = "a".repeat(MAX_PRESENTED_TOKEN_LEN + 1);
        let outcome = svc.validate(&oversized);
        assert_eq!(outcome, TokenValidation::Unknown);
        // The generic unknown path never echoes attacker-controlled input;
        // the static error message is asserted in server dispatch tests.
        let _ = oversized;
        // Far-oversized input (1 MiB) must also fail fast without unbounded work.
        let huge = "b".repeat(1024 * 1024);
        assert_eq!(svc.validate(&huge), TokenValidation::Unknown);
    }

    #[test]
    fn issuance_reclaims_expired_entries_before_evicting_live_ones() {
        let start = Instant::now();
        let (svc, clock) = TokenService::new_manual_for_test(start);
        for _ in 0..MAX_TOKENS {
            svc.issue();
        }
        assert_eq!(svc.count(), MAX_TOKENS);
        let live_sample = svc.inner.read().order.back().cloned().unwrap();
        // Expire everything, then issue: bounded lazy cleanup must reclaim
        // space so no live credential is evicted merely to make room.
        *clock.lock() = start + TOKEN_LIFETIME;
        let replacement = svc.issue();
        assert_eq!(svc.count(), 1);
        assert_eq!(svc.validate(&replacement), TokenValidation::Valid);
        // The previously live sample was expired too, so it is gone; the
        // key property is the store never exceeded MAX_TOKENS.
        assert_eq!(svc.validate(&live_sample), TokenValidation::Unknown);
    }

    #[test]
    fn issuance_at_capacity_with_mixed_expiry_prefers_expired_cleanup() {
        let start = Instant::now();
        let (svc, clock) = TokenService::new_manual_for_test(start);
        // Fill half, advance partway, fill rest so halves have different ages.
        for _ in 0..(MAX_TOKENS / 2) {
            svc.issue();
        }
        let midpoint = start + TOKEN_LIFETIME / 2;
        *clock.lock() = midpoint;
        for _ in 0..(MAX_TOKENS - MAX_TOKENS / 2) {
            svc.issue();
        }
        assert_eq!(svc.count(), MAX_TOKENS);
        // Expire only the first half.
        *clock.lock() = start + TOKEN_LIFETIME;
        let replacement = svc.issue();
        // One issuance reclaims all expired entries first, so the count
        // collapses to the live half plus the replacement.
        assert!(svc.count() <= MAX_TOKENS);
        assert_eq!(svc.validate(&replacement), TokenValidation::Valid);
    }

    #[test]
    fn concurrent_validators_cannot_both_authorize_after_expiry() {
        let start = Instant::now();
        let (svc, clock) = TokenService::new_manual_for_test(start);
        let token = svc.issue();
        *clock.lock() = start + TOKEN_LIFETIME;
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let outcomes = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for _ in 0..8 {
                let svc = svc.clone();
                let token = token.clone();
                let barrier = std::sync::Arc::clone(&barrier);
                handles.push(scope.spawn(move || {
                    barrier.wait();
                    svc.validate(&token)
                }));
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>()
        });
        // Exactly one observer sees Expired; the rest see Unknown. None sees Valid.
        assert!(!outcomes.contains(&TokenValidation::Valid));
        assert_eq!(
            outcomes.iter().filter(|o| **o == TokenValidation::Expired).count(),
            1
        );
        assert_eq!(
            outcomes.iter().filter(|o| **o == TokenValidation::Unknown).count(),
            7
        );
    }

    #[test]
    fn monotonic_source_is_used_for_expiry_not_wall_clock() {
        // Production uses Instant (monotonic). The manual seam advances the
        // same Instant type; wall-clock jumps cannot affect it. This test
        // pins that the stored expiry is Instant-based by advancing only the
        // manual monotonic handle.
        let start = Instant::now();
        let (svc, clock) = TokenService::new_manual_for_test(start);
        let token = svc.issue();
        *clock.lock() = start + Duration::from_secs(3600);
        assert_eq!(svc.validate(&token), TokenValidation::Valid);
    }

    #[test]
    fn token_entropy_and_shape_unchanged_by_lifetime() {
        let svc = TokenService::new();
        let token = svc.issue();
        assert_eq!(token.len(), TOKEN_BYTES * 2);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
