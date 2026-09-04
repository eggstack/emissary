# M127 — Base I2PControl Authentication Token-Lifetime Corrective

Status: **closed**

Closure authority: `plans/closure/i2pcontrol-proposal-170/127-closure.md`

Class: corrective / authentication / conformance / security

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Predecessor authority:

- M126 plan: `plans/implementation/i2pcontrol-proposal-170/126-post-m125-operational-security-and-spec-requalification.md`;
- M126 closure: `plans/closure/i2pcontrol-proposal-170/126-closure.md`;
- M107/M108 authentication/TLS closures;
- M061/M062 containment authority.

Planning baseline: `9948cfd0782a3defbd5f68cf2d4523603bdc7940`.

Pinned authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- the existing I2PControl API-1 authentication/error contract required by the Proposal 170 extension surface;
- the reference I2PControl token lifecycle as read-only compatibility evidence.

Current Proposal matrix entering M127: `284 apply / 96 blocked_primitive / 460 not_applicable`.

## 1. Objective

Correct the current authentication-session lifetime defect without broadening Proposal 170 scope or changing router/core behavior.

At the planning baseline, `TokenService` issues cryptographically random opaque tokens but stores only token membership. A token remains valid until explicit invalidation, bounded-capacity eviction, server shutdown, or process restart. The RPC layer defines `TOKEN_EXPIRED` (`-32004`) but the production validation path cannot reach it because no token carries an expiry.

This contradicts the shared base I2PControl authentication behavior that M126 claimed to have requalified. The reference I2PControl implementation uses finite token validity and removes expired tokens. M127 must add a finite lifetime, distinguish expired from unknown tokens, remove expired credentials atomically, and return the already-defined standard expired-token error.

M127 changes no Proposal 170 option/type cell. It supersedes only M126's affected authentication-lifetime qualification claim; historical closure records remain unchanged.

## 2. Why M126 missed the defect

M126 tested token issuance, unknown/malformed token rejection, conflicting header/parameter credentials, authentication throttling, and shutdown clearing, but did not exercise time-based token invalidation.

The M126 plan also contained an incorrect guard stating that it must not introduce a token-expiry requirement absent from the pinned base contract. That assumption prevented the audit from reconciling the already-declared `TOKEN_EXPIRED` error and the finite-lifetime reference behavior with the actual token store.

Regression evidence added by M127 must make that omission impossible to repeat: a token-lifetime test must fail if token storage loses expiry state or if expired tokens collapse into the generic unknown-token path.

## 3. Ownership and containment

Preferred production paths:

- `emissary-cli/src/i2pcontrol/auth.rs`;
- `emissary-cli/src/i2pcontrol/server.rs` only for mapping token validation outcomes to existing RPC errors;
- `emissary-cli/src/i2pcontrol/rpc.rs` only if existing error constants/messages require reconciliation;
- I2PControl-focused tests/docs/planning.

No `emissary-core/**`, `emissary-util/**`, Yosemite, proxy, tunnel, transport, router, frontend, or dependency change is authorized.

If a correct implementation appears to require a core clock service, global credential registry, router lifecycle mutation, or new dependency, stop and return the plan for correction. Token lifetime is an I2PControl server concern.

## 4. Hard invariants

M127 MUST preserve:

- API version 1-only authentication behavior;
- TLS-only production serving;
- password comparison through the reviewed bounded constant-time primitive;
- cryptographically random opaque token generation with at least the current entropy;
- token capacity bounds and deterministic oldest-token eviction;
- one unambiguous credential for protected dispatch; conflicting header/parameter values remain fail-closed;
- authentication metadata removal before method-specific domain validation;
- shutdown clears all active tokens;
- token values, passwords, and internal expiry state are never logged or returned except the newly issued token in the successful `Authenticate` result;
- no Proposal method, selector, action, type, option, status, or response extension is added;
- no token persistence across process restart;
- no matrix promotion or residual-capability work.

## 5. Required production semantics

### 5.1 Token record

Replace membership-only token state with a bounded record containing enough monotonic lifetime information to decide whether the credential is valid at lookup time.

Production validity SHOULD match the established reference behavior of one day unless the pinned base contract or stronger repository evidence requires another exact value. The chosen duration must be a named constant with compatibility rationale, not an arbitrary magic value.

Use a monotonic lifetime source for in-process expiry so wall-clock jumps cannot extend or prematurely invalidate a token. Tests must not sleep for production-duration intervals.

### 5.2 Validation outcome

Token validation must produce at least three internal outcomes:

- valid;
- expired-and-removed;
- unknown/invalid.

An expired lookup must remove the token under the same synchronization regime used to decide expiry so concurrent requests cannot repeatedly observe a stale valid record after one request has terminally expired it.

Protected dispatch maps:

- valid -> continue;
- expired -> existing `-32004 TOKEN_EXPIRED` error/message;
- unknown -> existing `-32003 INVALID_TOKEN` error/message.

Do not add a public token-age field, expiry timestamp, refresh method, cookie, bearer scheme, or new JSON-RPC error.

### 5.3 Input bounds

A presented credential must be bounded before expensive lookup/allocation. Preserve opaque-token semantics: do not invent a new public hexadecimal syntax requirement merely because the current issuer encodes random bytes as hex.

Oversized/malformed values must fail as invalid credentials without echoing attacker-controlled input and without allocating proportional unbounded state.

### 5.4 Capacity and cleanup

Capacity eviction remains deterministic and bounded. Expired entries may be removed lazily during validation/issuance or by another bounded local strategy, but M127 MUST NOT introduce a background unbounded scanner or global timer task.

Issuance at capacity must not allow expired entries to crowd out live credentials indefinitely when a bounded cleanup opportunity is already available.

## 6. Ordered work packages

### WP1 — freeze the authentication contract

1. Re-read the pinned base I2PControl authentication/error behavior and the reference finite-lifetime implementation as read-only evidence.
2. Record the exact compatibility lifetime selected for Emissary.
3. Add a focused contract test proving `TOKEN_EXPIRED` is not dead protocol surface.
4. Confirm no Proposal 170 matrix cell changes.

### WP2 — token-store lifetime model

1. Replace membership-only storage with a token record containing monotonic expiry state.
2. Preserve capacity ordering and exact eviction behavior.
3. Add a testable time seam confined to `auth.rs`; production must use the real monotonic clock, while tests can deterministically advance time without sleeps.
4. Keep test-only clock construction unavailable to normal production composition.

### WP3 — atomic validation and dispatch mapping

1. Implement valid/expired/unknown internal validation outcomes.
2. Remove expired entries atomically when observed.
3. Map expiry to `-32004`, unknown credentials to `-32003`.
4. Preserve header/parameter conflict handling and token stripping.
5. Prove concurrent validation at the expiry boundary yields no post-expiry success.

### WP4 — bounds and secret safety

1. Bound presented token length before lookup.
2. Confirm error/log paths never echo token material or expiry internals.
3. Re-run failed-auth throttling and capacity-churn tests.
4. Confirm no new task, timer, file, persistent secret, or dependency is introduced.

### WP5 — live and documentation regression evidence

1. Extend the real server/auth tests to prove normal issued tokens continue to authorize protected methods before expiry.
2. Use deterministic unit/integration time control for the expiry boundary; do not add a 24-hour live test.
3. Update active I2PControl security/support documentation to state finite token lifetime and M127 supersession of the affected M126 claim.
4. Update planning/closure evidence without rewriting M126 history.

## 7. Failure, cancellation, restart, and contention semantics

- Authentication requests are request-local; caller cancellation must not leave a half-issued token.
- Token issuance remains an atomic in-memory insertion.
- Validation/removal must be atomic with respect to concurrent validation of the same token at expiry.
- Restart invalidates every token exactly as today because the store remains process-local.
- Shutdown clearing remains idempotent.
- Capacity cleanup/eviction must remain bounded under attacker-driven authentication churn.
- Lock guards must not be held across async waits or network I/O.

## 8. Compatibility and migration

No persisted schema migration exists because tokens are intentionally process-local.

Clients using a token beyond its finite lifetime will now receive the standard expired-token error and must re-authenticate. This is the intended compatibility correction.

No base I2PControl methods unrelated to authentication are added. In particular, M127 does not implement `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings`, or other unrelated base methods; `plans/000-long-term-specification.md` keeps those outside Proposal 170 scope.

## 9. Focused tests

At minimum add deterministic tests for:

- token is valid immediately after issuance;
- token becomes expired at the exact configured boundary;
- expired validation removes the token;
- a second validation of the same token is unknown rather than repeatedly expired;
- protected dispatch returns `-32004` for the first expired observation and `-32003` after removal;
- two concurrent validators cannot both authorize after expiry;
- issuance/capacity behavior preferentially cleans bounded expired state without exceeding `MAX_TOKENS`;
- oversized presented credentials fail before unbounded allocation and are not echoed;
- valid header-only, params-only and equal header+params credentials still work;
- conflicting credentials remain rejected;
- shutdown/restart semantics still clear credentials;
- failed-auth throttle behavior remains unchanged.

## 10. Broad verification

Run and record exact outcomes:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Record the repository's known stable/nightly rustfmt limitation rather than introducing unrelated formatting churn.

## 11. Documentation and static guards

Update only active status/security documentation needed to state the corrected token semantics and M127 disposition.

Containment tests must prove M127 production changes stay under `emissary-cli/src/i2pcontrol/**`.

Add or extend a static/contract guard so future requalification cannot claim full shared-auth conformance while `TOKEN_EXPIRED` exists but token storage has no expiry-capable state/behavior.

## 12. Acceptance criteria

M127 closes only when:

1. every issued token has finite in-process validity;
2. the compatibility lifetime is explicitly justified and tested;
3. expired tokens are removed atomically;
4. the first expired observation returns existing `-32004` and later unknown use returns existing `-32003`;
5. no protected request can succeed after expiry;
6. capacity and attacker-controlled input remain bounded;
7. no token/password/expiry secret is exposed;
8. restart/shutdown behavior remains correct;
9. no production path outside `i2pcontrol` changes;
10. Proposal matrix remains `284 / 96 / 460` unless an independent cell-level defect is discovered, in which case stop and register a separate corrective;
11. broad verification has no unexplained regression;
12. a closure record supersedes only M126's affected authentication-lifetime claim.

## 13. Stop conditions

Stop and do not broaden M127 if implementation would require:

- persistent/token-sharing infrastructure;
- a router/core clock or credential service;
- new public authentication fields/errors/methods;
- base-method parity work unrelated to the Proposal extension surface;
- a non-I2PControl production change;
- weakening TLS, password comparison, conflict rejection, throttle, capacity, or request bounds.

Any newly discovered independent auth/TLS/JSON-RPC defect becomes a separately numbered successor rather than opportunistic M127 scope.

## 14. Closure evidence required

The M127 closure record must contain:

- exact implementation commit(s);
- exact selected lifetime and authority/rationale;
- before/after token-store state model;
- requirement-to-evidence table;
- deterministic expiry/concurrency results;
- exact broad verification commands/outcomes;
- secret/logging review;
- containment/path review;
- compatibility/migration review;
- unresolved findings with severity;
- successor readiness decision;
- internal-only external-interaction attestation.

## 15. External-interaction boundary

All external I2P specifications/reference implementations are read-only evidence. Repository writes are authorized only to `eggstack/emissary` for this plan.

M127 authorizes no upstream issue, PR, review, discussion, release, submission, merge/adoption request, maintainer contact, contribution package, or third-party repository mutation.