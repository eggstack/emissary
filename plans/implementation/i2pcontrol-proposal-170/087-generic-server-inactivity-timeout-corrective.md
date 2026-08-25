# M087 — Generic Server Inactivity Timeout Corrective

Status: closed — implementation complete; see `plans/closure/i2pcontrol-proposal-170/087-closure.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Corrective predecessors / controlling evidence:

- M075: `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md`;
- M085 closure: `plans/closure/i2pcontrol-proposal-170/085-closure.md`;
- M086 closure: `plans/closure/i2pcontrol-proposal-170/086-closure.md`.

Planning baseline: `2b01bfd11ebcd768fcd5488f18b063ac336931a2`.

Classification: corrective pass / bounded server-lifetime hardening.

## 1. Objective

Close the remaining generic `server` accepted-stream resource-exhaustion gap by replacing its unbounded post-connect raw relay lifetime with a finite **inactivity/progress timeout** while preserving the raw byte-stream semantics required by the generic tunnel type.

Current `emissary-cli/src/i2pcontrol/backends/server.rs` applies a five-second local-target connect timeout and then enters `tokio::io::copy_bidirectional` without an inactivity bound. Shared admission limits bound the number of simultaneously admitted peers, but an attacker controlling several valid I2P Destinations can keep all admitted slots occupied indefinitely without moving useful traffic.

This is not a direct clearnet-address disclosure. It is an availability and active-correlation hardening defect: indefinite attacker-controlled occupancy creates a stable load/no-load modulation primitive and prevents legitimate users from reaching the service until the attacker releases slots.

M075 intentionally did not invent a generic idle timeout because the Java generic-server runner does not expose one. M075 also required a separate compatibility-evidenced plan if later security evidence justified such a limit. M087 is that corrective plan.

## 2. Security property

After M087:

- an accepted generic server stream may remain open indefinitely **only while actual relay progress continues**;
- a stream that makes no successful byte-transfer progress for the configured finite inactivity interval is closed;
- active long-lived protocols are not terminated merely because their total connection age is large;
- an idle or stalled peer cannot pin one shared admission lease forever;
- timeout completion releases the admission lease and all task/stream/target resources;
- half-close behavior remains compatible with ordinary bidirectional TCP relay semantics.

The timeout MUST measure successful I/O progress, not socket readiness, wakeups, polling, task scheduling, or connection age.

## 3. Scope and containment

Preferred production scope is limited to:

- `emissary-cli/src/i2pcontrol/backends/server.rs`;
- existing colocated generic-server tests in that module.

A small helper under `emissary-cli/src/i2pcontrol/backends/runtime/**` is permitted only if it clearly removes duplicated relay-timeout logic and is required by more than one already-existing Proposal 170 server backend. Do not create a generalized transport framework for this milestone.

M087 MUST NOT require changes to:

- `emissary-core/**`;
- router/tunnel-building code;
- startup ownership;
- frontend/UI code;
- Proposal 170 JSON-RPC fields or option spelling;
- Yosemite;
- Cargo dependency ownership or `Cargo.lock`.

If the intended property cannot be implemented inside the existing `i2pcontrol` runtime boundary, stop and record the blocker rather than widening scope.

## 4. Internal-only rule

All implementation, review, closure, and repository writes remain internal to `eggstack/emissary`.

External I2P, I2P+, Yosemite, and specification sources may be read for behavioral evidence only. M087 does not authorize an upstream issue, pull request, review request, contribution branch, patch submission, merge request, maintainer contact, or any external repository write.

## 5. Required design

### 5.1 Inactivity, not absolute lifetime

Do not wrap the whole relay in one absolute `timeout(...)`. That would disconnect legitimate long-lived but active protocols and would turn the hardening into an unnecessary compatibility change.

Implement a relay loop or narrowly scoped helper that:

1. forwards bytes in both directions;
2. records progress only when one or more bytes are successfully read/written through the relay;
3. resets the inactivity deadline after such progress;
4. expires when neither direction makes progress for the full inactivity interval;
5. handles EOF/half-close without retaining the peer indefinitely;
6. returns cleanly on target or I2P stream error.

The implementation SHOULD reuse Tokio primitives already present in the crate. Do not add a dependency for timeout bookkeeping.

### 5.2 Timeout value

The Proposal 170 wire contract does not define a generic-server inactivity value. M087 therefore MUST NOT add a new public Proposal 170 field merely to expose the hardening.

Before implementation, compare the existing bounded server-family behavior already in this fork:

- IRC post-registration inactivity handling;
- HTTP body/header/request deadlines;
- local target connect deadline;
- shutdown/drain deadlines.

Choose a conservative finite internal default that is long enough for ordinary interactive/long-lived tunnel use and short enough to prevent indefinite pinning. Record the value and rationale in the closure.

If an already-supported, semantically exact existing tunnel option can configure this timeout without changing Proposal 170 contract truthfulness, it may be used. Do not reinterpret an unrelated option.

### 5.3 Admission lease ownership

The existing accepted-server admission lease remains the concurrency authority. The relay timeout is a lifetime bound layered underneath it.

The handler task MUST retain its lease for the full relay lifetime and release it on:

- clean EOF;
- inactivity expiry;
- target-side error;
- I2P-side error;
- cancellation/task unwind through existing ownership paths.

Do not create a second concurrency counter in `server.rs`.

### 5.4 Half-close semantics

Preserve useful half-close behavior. In particular:

- EOF from one side must not automatically discard already-buffered or immediately forthcoming response data from the other side when the current raw relay would have allowed it;
- once no useful forwarding can continue, terminate promptly;
- the inactivity timer still bounds a peer that half-closes and then leaves the remaining direction permanently idle.

Document any behavioral difference from `copy_bidirectional` that is unavoidable.

## 6. Required tests

Keep tests deterministic and local. No public-I2P/network test is required.

At minimum prove:

1. **idle expiry** — after target connect, an accepted stream with no byte progress is terminated when the inactivity interval elapses;
2. **progress resets deadline** — periodic real byte transfer keeps the relay alive past one nominal inactivity interval;
3. **unidirectional progress counts** — legitimate traffic flowing in only one direction resets the deadline;
4. **readiness is not progress** — a socket that wakes without transferring bytes does not extend occupancy indefinitely, where practical to test without implementation-specific flakiness;
5. **half-close** — one-direction EOF still allows the other direction to drain/finish as intended and cannot pin forever;
6. **target-connect bound retained** — the existing five-second target connect deadline remains unchanged;
7. **admission slot release** — timeout completion drops the existing admission lease so another peer can be admitted;
8. **error hygiene** — no peer Destination/private server Destination is added to error text or diagnostics;
9. **no task leak** — timeout/error/EOF paths terminate the handler task.

Use Tokio paused time where it makes the test exact and non-sleeping. Avoid wall-clock-heavy soak tests.

## 7. Ordered work packages

### A. Pin current behavior

1. Record implementation head.
2. Confirm generic server still uses accepted-stream admission plus five-second target connect and unbounded `copy_bidirectional` relay.
3. Confirm no newer implementation already supplies an inactivity bound.

### B. Select the narrow timeout model

1. Review existing IRC/HTTP timeout patterns in `i2pcontrol`.
2. Select and document the finite internal inactivity default.
3. Define precisely what resets the deadline.
4. Confirm no Proposal 170 API field is needed.

### C. Implement bounded raw relay

1. Replace only the unbounded generic relay portion.
2. Preserve loopback target restriction, target connect timeout, trusted peer/admission path, and raw payload behavior.
3. Keep all runtime state task-local/generation-local.

### D. Add focused regressions

Implement the tests in Section 6 without introducing a new integration-test framework.

### E. Verify containment and close

Run the focused generic-server tests, the Proposal 170 i2pcontrol test target(s) needed to cover the changed module, M062 dependency containment, and `git diff --check`.

Create `plans/closure/i2pcontrol-proposal-170/087-closure.md` with the exact chosen timeout, relay semantics, changed-path list, tests, and unresolved findings.

## 8. Verification discipline

Required minimum verification:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol generic_server
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

If the repository's test naming does not permit the first filter to cover the new tests, run the smallest exact `emissary-cli` i2pcontrol test command that does and record it in closure.

Do not add CI/fuzz/soak infrastructure for this bounded correction.

## 9. Acceptance criteria

M087 may close only when:

1. generic `server` relays no longer permit indefinite zero-progress occupancy;
2. timeout semantics are inactivity/progress based, not absolute connection lifetime;
3. successful byte progress in either direction resets the inactivity deadline;
4. active long-lived streams remain supported;
5. half-close behavior is explicitly tested and bounded;
6. the existing five-second target connect deadline remains intact;
7. the existing shared admission lease remains the sole accepted-server concurrency authority;
8. timeout/error/EOF paths release the admission lease and handler task;
9. no new Proposal 170 wire/API field or unsupported option reinterpretation is introduced;
10. no dependency, core/router/startup/frontend change is introduced;
11. focused tests and M062 containment pass;
12. `git diff --check` passes;
13. no upstream interaction occurs.

## 10. Stop conditions

Stop M087 rather than broadening it if:

- preserving raw stream semantics requires a new protocol parser;
- a proposed solution requires `emissary-core/**`, router, startup, or Yosemite changes;
- a new dependency is proposed solely for inactivity timing;
- the only proposed limit is an absolute connection lifetime;
- a new Proposal 170 field is required;
- testing requires public-network/deanonymization infrastructure.

Any such condition requires a separate numbered plan and explicit scope decision.

## 11. Closure evidence required

`plans/closure/i2pcontrol-proposal-170/087-closure.md` MUST include:

- baseline and final SHA;
- exact changed paths;
- chosen inactivity interval and rationale;
- before/after relay lifetime semantics;
- half-close behavior statement;
- proof that progress resets the deadline;
- proof that idle occupancy releases the admission lease;
- focused test commands and outcomes;
- M062 result and `git diff --check` result;
- containment statement showing changes stayed inside `i2pcontrol` plus planning/test bookkeeping;
- unresolved findings, if any;
- explicit internal-only/no-upstream-interaction attestation.

## 12. Successor handoff

M088 is administratively held until M087 closes so the registry keeps one executable tunnel-security corrective at a time. M088 is otherwise technically independent and addresses the lower-layer/pre-accept admission boundary.
