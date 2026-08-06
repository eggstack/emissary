# M041 — Authentication Throttle Source and Accounting Correction

Status: closed

Hard dependency:

- M040 closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrective authority:

- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md`

Applicable governance and retained evidence:

- `plans/003-planning-process.md`
- M036 implementation and closure records
- M039 retained constant-time password-comparison evidence

Repository defect baseline:

- `563e093ba1e65b4edc31104e3045c8b5a665e8ed`

## 1. Bounded objective

Make failed-authentication throttling effective across ordinary reconnects and
concurrent invalid attempts without changing the I2PControl authentication wire
contract.

The throttle must identify a source by normalized `IpAddr`, not full
`SocketAddr`, and must atomically reserve/increment the failure count before the
caller sleeps. The table, delay, and retention bounds remain small and local to
I2PControl.

M041 does not add accounts, persistent lockouts, trusted-proxy parsing, global
rate limiting, password hashing, token changes, or router-wide abuse controls.

## 2. Demonstrated defect and prior evidence gap

The current table uses `HashMap<SocketAddr, FailureState>`. A client that
reconnects from the same IP with a new ephemeral port receives a new entry and
can avoid accumulated delay.

The current request path calls `delay_for_failure`, sleeps, and only then calls
`record_failure`. Concurrent failures can read the same prior state before any
one records its attempt, undercounting the burst for delay purposes.

Existing tests cover capacity and maximum delay but not same-IP/different-port
identity or concurrent reservation ordering.

## 3. Required invariants

1. Failure identity is the network source IP only.
2. IPv4 and IPv6 addresses remain distinct exact keys.
3. Source ports never create separate throttle identities.
4. Every completed invalid-password decision reserves one failure count before
   its delay is awaited.
5. The throttle lock is not held across sleep or request dispatch.
6. Successful authentication clears the source IP's failure state.
7. Malformed Authenticate parameters and unsupported API versions retain their
   existing error behavior unless the current contract already classifies them
   as password failures.
8. The table remains bounded to the existing small capacity.
9. Delay remains monotonic, capped, and based on monotonic time.
10. Tokens, API versions, error codes/messages, TLS, body limits, and password
    comparison remain unchanged.
11. No persistent ban database or cross-service rate limiter.
12. No upstream interaction.

## 4. Scope and production file budget

### Primary production files

- `emissary-cli/src/i2pcontrol/auth.rs`
- `emissary-cli/src/i2pcontrol/server.rs`

### Authorized tests and records

- focused auth unit tests;
- focused handler/server tests using distinct source ports for one IP;
- M041 implementation disposition and closure record;
- directly affected security documentation.

### Prohibited changes

- `emissary-core/**`;
- tunnel, AddressBook, RouterInfo, ClientServicesInfo, or persistence behavior;
- token format/lifetime/storage;
- password storage or hashing redesign;
- forwarded-header or proxy trust logic;
- global firewall/ban integration;
- database-backed lockouts;
- CI/release expansion.

## 5. Target throttle contract

Introduce one operation that performs the complete state transition under the
throttle lock and returns the delay to await after lock release. A representative
shape is:

```rust
fn reserve_failure(&self, source: Option<SocketAddr>) -> Duration
```

or a boundary-normalized equivalent taking `Option<IpAddr>`.

The operation must:

1. normalize `SocketAddr` to `IpAddr` before map access;
2. expire/reset stale state according to the existing monotonic window;
3. increment or create the failure state atomically;
4. enforce table capacity and deterministic eviction;
5. compute the delay from the newly reserved count;
6. release the lock before returning.

The request handler then sleeps for the returned bounded duration and emits the
existing invalid-password response.

The exact first-failure schedule must be documented and tested. Preserve the
existing user-visible intent unless a direct bug requires adjustment:

- first failure: zero delay;
- first repeated failure: base delay;
- later failures: bounded exponential increase;
- maximum: existing cap.

## 6. Ordered work packages

### WP1 — Add failing identity regressions

Add tests proving that:

- `127.0.0.1:10001` and `127.0.0.1:50000` share one throttle state;
- one IPv6 address with different ports shares one state;
- different IPs remain independent;
- successful authentication from a new port on the same IP clears the shared
  state.

### WP2 — Add failing atomic-accounting regression

Exercise several concurrent invalid attempts from one normalized IP. The test
must prove that each attempt reserves a distinct monotonically advancing count
before sleeping, rather than all observing one stale count.

Prefer testing the pure reservation API without wall-clock sleeps. One small
handler-level test must confirm the server uses that API.

### WP3 — Normalize source identity

Change the internal key to `IpAddr` or a private newtype wrapping it. Keep
`SocketAddr` only at the HTTP/TLS boundary where the peer address is supplied.

Do not inspect `X-Forwarded-For`, `Forwarded`, or other untrusted headers.

### WP4 — Make reservation atomic

Replace split read/sleep/write behavior with one reserve-and-return-delay
operation. Remove or make private obsolete APIs so production cannot continue
using the non-atomic sequence.

### WP5 — Revalidate bounds and cancellation

Prove:

- the table cannot exceed its configured entry cap;
- eviction remains deterministic enough for tests and does not panic;
- cancelled requests after reservation leave a conservative recorded failure;
- the lock is released before sleep;
- successful authentication clears only the normalized source IP;
- token issuance and protected-request authentication are unchanged.

## 7. Failure, cancellation, and contention semantics

- Reservation occurs only after password comparison has failed.
- A request cancelled during the subsequent sleep leaves the failure recorded;
  this is conservative and prevents cancellation from becoming a bypass.
- No mutex is held across sleep.
- Concurrent invalid attempts serialize only the short in-memory reservation.
- Capacity pressure evicts one bounded historical source according to the
  existing or explicitly documented deterministic policy.
- A successful authentication clears the source IP state after password and API
  validation succeed.
- Restart clears all in-memory throttle state, preserving current behavior.

## 8. Compatibility and migration

- No JSON-RPC change.
- No error-code or message change.
- No configuration or persistence format change.
- No token migration.
- The only behavior change is that reconnects and concurrent attempts are
  actually subject to the intended bounded throttle.

## 9. Required tests

At minimum:

1. same IPv4, different ports share state;
2. same IPv6, different ports share state;
3. different IPs remain independent;
4. first and repeated failure delays match the documented schedule;
5. concurrent reservations receive monotonically nondecreasing delays/counts;
6. cancelled post-reservation request does not erase the failure;
7. successful auth from a different port on the same IP clears state;
8. table capacity remains bounded under IP churn;
9. oversized password comparison and token behavior remain unchanged;
10. handler-level authentication still emits exact standard errors.

The ephemeral-port regression must fail on baseline `563e093` and pass after
correction.

## 10. Verification commands

At minimum:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol i2pcontrol::auth
cargo test -p emissary-cli --no-default-features --features i2pcontrol authentication
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test golden_fixtures
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Run the no-feature CLI check to prove optional isolation. No remote CI expansion
is required.

## 11. Documentation and guards

Update `docs/i2pcontrol/security.md` to state that failed-authentication state is
bounded per source IP and in-memory only. Do not claim distributed, persistent,
or proxy-aware rate limiting.

The implementation disposition must list every changed production path and
state that no authentication wire behavior changed.

## 12. Acceptance criteria

M041 may close only when:

- reconnects from the same IP cannot reset accumulated delay by changing source
  port;
- failure count is reserved atomically before sleep;
- concurrent attempts cannot all observe one stale count;
- success clears normalized source state;
- table and delay bounds remain enforced;
- no lock is held across await;
- exact authentication errors/tokens remain unchanged;
- no unrelated production scope is touched;
- no unresolved high- or medium-severity finding remains in this slice.

## 13. Stop conditions

Stop rather than:

- add persistent bans/accounts;
- trust forwarded headers;
- add router-wide or OS firewall integration;
- change password storage or token semantics;
- modify core;
- expand CI/release infrastructure;
- interact with upstream.

## 14. Closure evidence required

The M041 disposition and closure must include:

- implementation/test commit SHA;
- failing-before/passing-after ephemeral-port evidence;
- concurrent reservation evidence;
- exact delay schedule and capacity result;
- handler-level compatibility evidence;
- verification command outcomes;
- changed-path classification;
- unresolved findings with severity;
- internal-only/no-upstream attestation.
