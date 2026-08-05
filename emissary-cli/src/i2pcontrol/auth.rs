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
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;

/// Maximum number of tokens to retain in memory.
const MAX_TOKENS: usize = 1024;

/// A token is 32 bytes of cryptographically random data, hex-encoded.
const TOKEN_BYTES: usize = 32;

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

/// Authentication token service.
///
/// Tokens are cryptographically random, opaque, bounded, and invalidated on restart.
#[derive(Clone)]
pub struct TokenService {
    inner: Arc<RwLock<TokenStore>>,
}

struct TokenStore {
    tokens: HashMap<String, ()>,
    order: VecDeque<String>,
}

#[derive(Clone)]
pub(crate) struct AuthThrottle {
    inner: Arc<Mutex<ThrottleStore>>,
}

struct ThrottleStore {
    failures: HashMap<SocketAddr, FailureState>,
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

    /// Compute a delay without changing state. Callers sleep after the lock
    /// is released and record the completed failure afterwards.
    pub(crate) fn delay_for_failure(&self, source: Option<SocketAddr>) -> Duration {
        let Some(source) = source else {
            return Duration::ZERO;
        };
        let now = Instant::now();
        let store = self.inner.lock();
        let Some(failure) = store.failures.get(&source) else {
            return Duration::ZERO;
        };
        if now.duration_since(failure.last_failure) >= THROTTLE_WINDOW {
            return Duration::ZERO;
        }
        delay_for_count(failure.count)
    }

    pub(crate) fn record_failure(&self, source: Option<SocketAddr>) {
        let Some(source) = source else {
            return;
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
            } else {
                failure.count = failure.count.saturating_add(1);
                failure.last_failure = now;
            }
            return;
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
    }

    pub(crate) fn clear(&self, source: Option<SocketAddr>) {
        if let Some(source) = source {
            self.inner.lock().failures.remove(&source);
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
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(TokenStore {
                tokens: HashMap::new(),
                order: VecDeque::new(),
            })),
        }
    }

    /// Issue a new cryptographically random token.
    ///
    /// Returns the hex-encoded token string.
    pub fn issue(&self) -> String {
        let mut store = self.inner.write();

        // Evict exactly the oldest token at capacity.
        if store.tokens.len() >= MAX_TOKENS {
            if let Some(oldest) = store.order.pop_front() {
                store.tokens.remove(&oldest);
            }
        }

        let token = generate_token();
        store.tokens.insert(token.clone(), ());
        store.order.push_back(token.clone());
        token
    }

    /// Validate a token. Returns true if the token is valid.
    pub fn validate(&self, token: &str) -> bool {
        let store = self.inner.read();
        store.tokens.contains_key(token)
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

/// Validate the API version. Accepts 1 or 2.
pub fn validate_api_version(version: i32) -> bool {
    version == 1 || version == 2
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
        assert!(svc.validate(&token));
        assert!(!svc.validate("invalid-token"));
    }

    #[test]
    fn invalidate_token() {
        let svc = TokenService::new();
        let token = svc.issue();
        assert!(svc.validate(&token));
        svc.invalidate(&token);
        assert!(!svc.validate(&token));
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
        assert!(!svc.validate(&first));
        assert!(svc.validate(&replacement));
    }

    #[test]
    fn validate_api_version_valid() {
        assert!(validate_api_version(1));
        assert!(validate_api_version(2));
    }

    #[test]
    fn validate_api_version_invalid() {
        assert!(!validate_api_version(0));
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
        for port in 0..(MAX_THROTTLE_ENTRIES as u16 + 32) {
            throttle.record_failure(Some(([127, 0, 0, 1], port).into()));
        }
        assert_eq!(throttle.count(), MAX_THROTTLE_ENTRIES);
    }

    #[test]
    fn throttle_delay_is_bounded() {
        let throttle = AuthThrottle::new();
        let source = Some(([127, 0, 0, 1], 7650).into());
        for _ in 0..1000 {
            throttle.record_failure(source);
        }
        assert!(throttle.delay_for_failure(source) <= THROTTLE_MAX_DELAY);
    }
}
