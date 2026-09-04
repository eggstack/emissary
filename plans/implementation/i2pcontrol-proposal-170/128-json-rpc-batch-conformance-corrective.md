# M128 — JSON-RPC 2.0 Batch Conformance Corrective

Status: **queued / unregistered**

Class: corrective / JSON-RPC conformance / resource hardening

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Predecessor authority:

- M126 plan and closure;
- M127 authentication token-lifetime corrective;
- `plans/000-long-term-specification.md` protocol-exactness and containment requirements;
- M061/M062 containment authority.

Planning baseline: `9948cfd0782a3defbd5f68cf2d4523603bdc7940`.

Pinned authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- JSON-RPC 2.0 request, notification, batch, error, and request-ID semantics required by the implemented I2PControl extension surface.

Current Proposal matrix entering M128: `284 apply / 96 blocked_primitive / 460 not_applicable`.

## 1. Objective

Replace the current blanket rejection of top-level JSON-RPC batch arrays with bounded, authenticated JSON-RPC 2.0 batch behavior, without introducing request fan-out, cross-request credential sharing, new Proposal methods, or changes outside the I2PControl HTTP/RPC layer.

At the planning baseline, `parse_request` accepts only a top-level object. M126 explicitly treated batch arrays as invalid requests. That behavior is safe against auth bypass but does not implement the JSON-RPC 2.0 batch form. M128 must add the missing transport-level behavior while preserving the stricter I2PControl rule that method parameters are named objects.

M128 changes no Proposal 170 option/type cell and does not expand unrelated base I2PControl method support.

## 2. Readiness and dependency policy

M128 is intentionally queued rather than registered while M127 is the active handoff. It has no router/core dependency, but final implementation should begin from the closed M127 head so batch dispatch inherits the corrected token-lifetime semantics and its regression suite.

Registration rule: promote M128 to **ready** only after M127 closure is recorded and active docs/registry point to the corrected auth authority.

## 3. Why prior verification missed the defect

M126 tested that batch input could not bypass authentication, but its acceptance criterion allowed either protocol-correct handling or deterministic rejection. That proved a security property, not JSON-RPC 2.0 batch conformance.

M128 regression evidence must distinguish these two questions:

1. can malformed/mixed batch content bypass auth/resource limits? — must remain no;
2. does a valid batch receive JSON-RPC 2.0-compliant execution and response behavior? — must become yes.

## 4. Ownership and containment

Expected production paths:

- `emissary-cli/src/i2pcontrol/rpc.rs`;
- `emissary-cli/src/i2pcontrol/server.rs`;
- I2PControl tests/docs/planning.

No `emissary-core/**`, `emissary-util/**`, Yosemite, tunnel backend, AddressBook owner, router, transport, proxy, frontend, or dependency change is authorized.

Do not generalize this milestone into a new generic JSON-RPC framework or dependency adoption. The existing bounded parser/dispatcher should be extended locally.

## 5. Hard invariants

M128 MUST preserve:

- TLS-only production serving;
- one valid unambiguous credential per protected request element;
- M127 finite token validity and expired/unknown distinctions;
- named-object I2PControl parameters; positional-array params remain unsupported unless separately required by the pinned I2PControl contract;
- explicit-null request IDs remain requests, while absent IDs remain notifications;
- notification execution uses the same auth/domain validation as requests and suppresses its response;
- no shared implicit token, result, state, or authentication context across batch elements;
- total request-body cap, connection cap, request deadline, auth throttle, and global in-flight limits;
- no attacker-controlled unbounded task spawning or response allocation;
- exact method/domain semantics are unchanged;
- no Proposal-specific policy moves outside `i2pcontrol`;
- no matrix change.

## 6. Required JSON-RPC batch semantics

### 6.1 Top-level form

The HTTP body may contain either:

- one JSON-RPC request object; or
- one non-empty JSON array of JSON-RPC request entries.

A top-level empty array returns the standard invalid-request error with null ID.

A batch entry that is not a valid JSON-RPC request object contributes an invalid-request error for that entry with null ID rather than invalidating unrelated valid entries.

Nested arrays are not batch entries and are invalid requests.

### 6.2 Notifications

Each notification executes normally but contributes no response element.

If every valid executable element is a notification and there are no invalid entries requiring an error response, the HTTP layer must emit no JSON-RPC response body. Use an existing appropriate HTTP no-content response; do not serialize `[]`, `null`, or a fabricated success object.

### 6.3 Authentication

Authentication is per element.

A batch may contain `Authenticate` and protected methods, but there is no special intra-batch token propagation or ordering contract. A protected element must carry a token that was valid independently when that element was dispatched.

Do not allow an `Authenticate` result generated earlier in the same batch to be implicitly injected into later elements.

Header-token compatibility remains request-wide transport metadata, but every protected element must still reconcile that header against its own optional `params.Token` exactly as single-request dispatch does.

### 6.4 Ordering and execution

Responses SHOULD preserve input order for deterministic behavior even though JSON-RPC does not require response ordering.

Do not spawn one unconstrained task per batch element. Prefer bounded sequential dispatch under the existing HTTP request deadline, or an explicitly bounded local concurrency strategy that cannot exceed the global request budget.

The simplest acceptable design is sequential element dispatch with no nested task fan-out.

### 6.5 Batch cardinality

Add an explicit maximum batch element count no greater than the existing in-flight request bound. The exact constant must be documented and tested.

Reject an over-cap batch before executing any element so partially executed oversized batches cannot create surprising side effects.

The existing total body-size limit remains independently authoritative.

## 7. Ordered work packages

### WP1 — parser/envelope split

1. Introduce an internal parsed-envelope type representing single versus batch input.
2. Reuse the existing exact single-request parser for each batch element instead of duplicating validation rules.
3. Preserve parse-error versus invalid-request distinctions.
4. Add bounded batch cardinality validation before element execution.

### WP2 — authenticated batch dispatcher

1. Route each element through the same `Authenticate` or protected-auth path as a single request.
2. Preserve M127 token expiry semantics independently for every element.
3. Strip auth metadata before domain validation exactly as today.
4. Collect only non-notification responses.
5. Do not hold global locks across element dispatch awaits.

### WP3 — HTTP response shaping

1. Single request retains its current response shape.
2. Batch with one or more response-producing entries serializes one JSON array of response objects.
3. All-notification batch produces no JSON-RPC body.
4. Mixed valid/invalid batch entries preserve independent errors/results.
5. Internal serialization failures remain sanitized and bounded.

### WP4 — adversarial resource qualification

Exercise:

- empty batch;
- maximum-size batch;
- over-limit batch;
- mixed authenticate/protected/notification/error entries;
- invalid scalar/null/array entries;
- duplicate IDs;
- explicit-null IDs;
- missing/invalid/expired/conflicting tokens in different elements;
- slow element plus deadline expiration;
- large but body-cap-compliant elements;
- all-notification batch;
- notification containing invalid auth or invalid params (executes validation but emits no response);
- no task-count/concurrency amplification beyond the documented bound.

### WP5 — documentation and static regression guards

Update active I2PControl protocol/security docs to state bounded JSON-RPC batch support and the exact maximum cardinality.

Add a guard/test that fails if top-level valid batch arrays regress to blanket invalid-request handling.

## 8. Failure, cancellation, restart, and contention semantics

- Batch dispatch is one HTTP request lifetime; cancellation/deadline stops remaining undispatched elements.
- Already committed mutations from earlier elements are not transactionally rolled back merely because a later independent batch element fails or the client disconnects. JSON-RPC batching is not a transaction.
- Individual method implementations retain their existing transaction/cancellation guarantees.
- No lock spans the whole batch except immutable/request-local response collection.
- Restart has no special batch state.
- Oversized batches execute zero elements.

Document these semantics explicitly so clients cannot infer atomic batch transactions.

## 9. Compatibility and migration

This is additive transport compatibility for clients already using JSON-RPC 2.0 batches. Existing single-request behavior must remain byte/shape-compatible except where M127 has already corrected token expiry.

No persistence migration exists.

No unrelated base I2PControl methods become supported. The canonical non-goal in `plans/000-long-term-specification.md` remains controlling.

## 10. Focused tests

At minimum prove:

- valid two-request batch returns two responses;
- response IDs/types are preserved independently;
- mixed notification/request returns only request response;
- all-notification batch returns no JSON-RPC body;
- empty array returns invalid request;
- invalid entries produce per-entry invalid-request errors without suppressing valid siblings;
- over-cap batch executes no side effects;
- protected batch elements each require valid auth;
- equal header/parameter tokens still work per element;
- conflicting token in one element cannot affect siblings;
- expired token returns M127 `-32004` for the affected element only;
- no implicit token propagation from an Authenticate element;
- single-request golden fixtures remain unchanged;
- batch execution cannot exceed documented request/task bounds.

## 11. Broad verification

Run and record:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial --test i2pcontrol_live_runtime --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Record known formatter-toolchain limitations rather than rewriting unrelated files.

## 12. Acceptance criteria

M128 closes only when:

1. single and bounded batch JSON-RPC envelopes are both supported;
2. empty/invalid/oversized batches follow documented fail-closed semantics;
3. each protected element independently passes the existing auth boundary;
4. notifications suppress responses correctly, including all-notification batches;
5. no implicit intra-batch token/result sharing exists;
6. no unbounded task fan-out or response allocation exists;
7. prior single-request behavior remains correct;
8. method-level transaction/cancellation guarantees are unchanged;
9. all production changes remain under `emissary-cli/src/i2pcontrol/**`;
10. matrix remains `284 / 96 / 460` unless a separate cell-level defect is found;
11. broad verification passes or records only understood pre-existing limitations.

## 13. Stop conditions

Stop and return for corrective planning if batch support would require:

- changing method/domain public contracts;
- positional I2PControl parameter support not required by the pinned shared contract;
- cross-element transaction semantics;
- a generic external JSON-RPC server dependency;
- core/router changes;
- weakening auth, body, concurrency, timeout, or secret-safety limits.

## 14. Closure evidence required

Closure must include:

- implementation commit(s);
- exact batch cardinality bound;
- single/batch parser architecture;
- auth-per-element trace;
- notification/no-content evidence;
- mixed/invalid/over-cap adversarial results;
- request/task resource accounting;
- exact verification commands/outcomes;
- compatibility/containment/security review;
- unresolved findings and next-readiness decision;
- internal-only external-interaction attestation.

## 15. External-interaction boundary

All external JSON-RPC/I2P references are read-only evidence. Writes are authorized only to `eggstack/emissary`.

No upstream issue, PR, review, discussion, release, submission, merge/adoption request, maintainer contact, or contribution artifact is authorized.