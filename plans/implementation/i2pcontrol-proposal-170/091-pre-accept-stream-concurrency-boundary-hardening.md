# M091 — Pre-Accept Stream Concurrency Boundary Hardening

Status: blocked

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Corrective authority and predecessors:

- M074/M083 application admission hardening;
- M088 pre-accept admission-boundary evidence and `plans/closure/i2pcontrol-proposal-170/088-closure.md`;
- M089 current-head tunnel runtime/security reclosure;
- M090 server loopback/IRC half-close corrective must close before M091 becomes the active handoff.

Planning baseline: `f0f3fc2204318c2fac69817d347df2702c51287b`.

Classification: blocked dependency-boundary security corrective / streaming infrastructure.

## 1. Objective

Add the smallest defensible lower-layer concurrency bound that can reject an authenticated inbound streaming SYN before Emissary allocates normal pending/active stream state and before Yosemite `Session<style::Stream>::accept()` returns, while preserving the existing post-accept `ServerAdmissionState` as defense in depth.

M091 deliberately targets **pre-accept stream concurrency only**. It does not duplicate the full Proposal 170 per-peer minute/hour/day rate policy in `emissary-core`. Keeping the richer policy in `i2pcontrol` minimizes security-sensitive changes to the previously reviewed router/core codebase.

## 2. Current blocker

M091 is not dependency-ready at the planning baseline.

M088 established that:

- Yosemite 0.7.0 has no `SessionOptions` field for `i2p.streaming.maxConcurrentStreams` or related streaming-admission controls;
- Yosemite `Session::accept()` sends only the ordinary SAM `STREAM ACCEPT` command and returns after the lower-layer stream already exists;
- `emissary-core/src/sam/protocol/streaming/config.rs` declares `StreamConfig::max_concurrent_streams` and related fields, but current `StreamManager` construction/stream spawning uses defaults and does not consume the declared admission fields;
- passing Java option names through current I2CP/SAM options would therefore be persist-and-ignore behavior.

This remains true against the current read-only Yosemite `master` checked on 2026-08-26: `master` is still commit `d0fe71da214b212790773be12a93162ae71f3e03` (`prepare release v0.7.0`), and `SessionOptions` still exposes no streaming-concurrency field.

Therefore the exact blocker is:

> There is no currently supported in-repository configuration path by which `i2pcontrol` can carry its accepted-server concurrency policy through Yosemite/SAM into the Emissary streaming manager before `accept()`.

M091 MUST remain `blocked` until a maintainer explicitly authorizes one narrow internal configuration transport that resolves this gap. Merely registering this plan does not authorize vendoring/forking Yosemite, switching to an unreviewed git dependency, or adding a process-global magic registry.

## 3. Preferred architecture once the blocker is resolved

The target design is intentionally narrow and layered.

### 3.1 `i2pcontrol` remains policy owner

`ServerAdmissionPolicy::max_concurrent_connections()` remains the Proposal 170/application policy source for accepted-server families.

The lower-layer value is a defense-in-depth copy of the already-validated concurrency ceiling for the dedicated server session. It is not a second independent public option surface.

The common accepted-server path continues to enforce the existing post-accept `ServerAdmissionState` for:

- trusted peer identity;
- per-peer concurrency;
- aggregate/application concurrency;
- configured minute/hour/day peer and total rates;
- bounded peer-history/cardinality semantics;
- handler-task ownership.

### 3.2 Core consumes only a narrow concurrency configuration

The smallest acceptable `emissary-core` change is to make the already-declared streaming concurrency configuration real for inbound streams.

Preferred enforcement point:

1. parse and authenticate the inbound SYN;
2. verify replay-protection binding to the local Destination;
3. identify the authenticated remote `DestinationId`;
4. evaluate the configured stream-concurrency ceiling;
5. if over limit, reject/reset/drop according to the minimal selected fixed behavior;
6. only if allowed, proceed to `listener.pop_socket()`, `PendingStream::new`, `pending_inbound` insertion, active-channel allocation, routing-path binding, and stream task creation.

The check must occur before the allocations M088 identified as the residual resource/timing surface.

M091 SHOULD consume only `max_concurrent_streams` unless implementation evidence proves another lower-layer field is necessary for correctness. Do not port Java's complete `ConnThrottler` machinery merely for parity.

### 3.3 Defaults preserve unrelated streaming behavior

For non-I2PControl streaming sessions, the existing default must remain semantically unchanged unless a separate general-streaming hardening decision is explicitly approved.

In particular:

- `None` continues to mean no configured concurrency limit;
- M091 must not silently impose a new global hard cap on all Emissary streaming sessions;
- no default feature or unrelated application receives new policy because I2PControl exists.

### 3.4 Configuration transport is explicit and standard-shaped

The preferred transport, if available, is the standard I2P streaming option spelling `i2p.streaming.maxConcurrentStreams` carried on the server session creation path.

Acceptable implementation shape after authorization:

- a Yosemite release/API that explicitly exposes the field; or
- an explicitly approved internal dependency-boundary mechanism whose entire purpose is to carry this one standard option without forking application semantics.

The Emissary SAM/session side then parses the exact option into a bounded integer and supplies it to the `StreamManager` configuration.

Do not use nickname conventions, environment variables, hidden global registries, reused unrelated option fields, or request-selected filesystem state to smuggle policy across the boundary.

## 4. Scope if unblocked

Preferred `i2pcontrol` production scope:

- `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs` for translating the already-validated application concurrency ceiling into the lower-layer session option;
- focused accepted-server tests.

Minimal core scope, only as required to consume the option:

- `emissary-core/src/sam/protocol/streaming/config.rs`;
- `emissary-core/src/sam/protocol/streaming/mod.rs`;
- the narrow SAM session-create parser/composition path that constructs the per-session `StreamManager` configuration;
- focused core/SAM streaming tests.

Dependency scope:

- no Yosemite change is authorized while M091 remains blocked;
- if the blocker is resolved by a new released Yosemite version that exposes the exact option, a separately reviewed manifest/lockfile delta may be added to this plan before it is marked ready;
- if resolution requires vendoring/forking/patching Yosemite or a git dependency, M091 stays blocked until an explicit maintainer directive authorizes that exact dependency strategy and the containment plan is amended accordingly.

No other core/router/startup/frontend path is pre-authorized.

## 5. Security invariants

M091 MUST preserve:

- exact Proposal 170 API contract; no new JSON-RPC fields/actions/types;
- authenticated remote Destination as the only peer identity;
- SYN signature and replay-protection validation before admission accounting;
- existing post-accept application admission and its richer rate/peer semantics;
- existing server Destination identity/persistence;
- per-session/generation isolation of limiter state;
- no request-controlled configuration path;
- no private Destination/key material in logs/errors;
- no lock held across network I/O, sleeps, or task joins;
- bounded state for any newly introduced lower-layer counter;
- default behavior unchanged for sessions that do not configure a limit;
- no upstream interaction.

## 6. Explicit non-goals

M091 MUST NOT:

- duplicate all `ServerAdmissionState` logic in core;
- implement per-peer minute/hour/day or aggregate rate windows unless a separately demonstrated lower-layer defect requires them;
- introduce process-wide cross-Destination admission budgets;
- claim Sybil resistance;
- add randomized rejection delay, jitter, padding, proof-of-work, or reputation systems;
- change tunnel lengths/quantities/crypto algorithms;
- redesign SAM or implement a parallel SAM client/server stack;
- add a generic callback from `emissary-core` into `emissary-cli`;
- use a process-global registry keyed by nickname or Destination as an implicit policy channel;
- impose a new hard concurrency default on unrelated streaming clients;
- alter Streamr;
- alter `httpbidirserver` identity-sharing semantics;
- perform public-network load/deanonymization experiments;
- prepare or request upstream Yosemite/I2P review or merge.

## 7. Lower-layer rejection semantics

The selected lower-layer response must be fixed, bounded, and compatible with existing streaming behavior.

Preferred default is the existing streaming reset behavior represented by `LimitAction::Reset`, provided the implementation can emit it without constructing a normal active stream. If a reset itself requires disproportionate work under overload, a bounded drop path may be used only with source-level justification.

M091 must not add HTTP/custom rejection payload support. The purpose is resource bounding, not feature parity with Java `limitAction`.

A rejected SYN must not:

- allocate a normal `Stream` task;
- allocate a normal stream channel;
- enter `pending_inbound`;
- bind a routing path for an accepted stream;
- reach a local SAM accepted socket;
- reach Yosemite `Session::accept()` as an accepted stream;
- mutate unbounded per-peer state.

## 8. Counter semantics

If the core implementation uses the existing active/pending maps to determine occupancy, it must define exactly what counts toward the configured ceiling.

Required property:

- all lower-layer inbound states that can materially consume resources before application admission must count, including both pending inbound and active inbound streams.

Do not count unrelated outbound streams if the configured value is intended as an inbound server-session ceiling unless the standard option semantics and session usage require combined counting. The dedicated accepted-server Yosemite session is expected not to originate normal application outbound streams; document any exception rather than assuming it.

Counter release must be automatic on:

- rejected setup;
- malformed/invalid SYN;
- pending expiry;
- routing-path bind failure;
- stream task completion/error;
- SAM listener disappearance;
- session shutdown/restart.

## 9. Ordered work packages after unblocking

### A. Revalidate dependency/API evidence

Before changing code, inspect the exact Yosemite version/API proposed for the transport. Confirm it can carry one explicit standard streaming concurrency option without arbitrary raw-command construction.

If not, stop; M091 remains blocked.

### B. Make core concurrency configuration consumable

Thread the per-session concurrency setting into `StreamManager` with default-off semantics. Implement the pre-allocation check after authentication/replay validation and before pending/active allocation.

Use existing maps/counts where possible. Avoid a second shadow registry if current state already contains the necessary occupancy information.

### C. Carry the policy from `i2pcontrol`

Translate `ServerAdmissionPolicy::max_concurrent_connections()` into the lower-layer session option only for common accepted-server sessions (`server`, `httpserver`, inbound `httpbidirserver`, `ircserver`).

Do not apply it to client sessions or Streamr.

Retain the existing post-accept `BoundedTaskGroup` size and `ServerAdmissionState` unchanged as defense in depth.

### D. Add deterministic regression tests

Prove rejection happens before normal pending/active/application allocation and that capacity is released on teardown.

### E. Update containment authority

Any authorized `emissary-core/**`, manifest, lockfile, or dependency change must be added as an exact path/delta to M061/M062-style containment evidence. Do not broaden globs.

## 10. Required tests after unblocking

At minimum cover:

1. default `None` configuration preserves prior unrestricted streaming behavior;
2. configured ceiling allows exactly the permitted number of inbound streams;
3. the next authenticated valid SYN is rejected before `pending_inbound`/active stream allocation;
4. pending inbound states count toward the ceiling;
5. active inbound states count toward the ceiling;
6. malformed/unsigned/replay-invalid SYNs do not consume admission capacity;
7. capacity is released after pending expiry;
8. capacity is released after active stream close/error;
9. session shutdown/restart starts with fresh lower-layer state;
10. the accepted-server Yosemite session receives the translated limit while client/Streamr sessions do not;
11. post-accept `ServerAdmissionState` still independently rejects over-policy peers/rates;
12. no full/private Destination material is logged by the new lower-layer denial path.

Tests should be local/deterministic. No public I2P load testing is required.

## 11. Verification after unblocking

At minimum:

```text
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

If a dependency version changes, also verify the exact manifest/lockfile diff and all existing containment guards that previously required the lockfile to remain byte-identical. Such a change requires an explicit containment amendment rather than weakening the old assertion silently.

Review changed paths against the M091 approved set. Any unrelated core/router/frontend/startup change is a blocker.

## 12. Compatibility and migration effects

With the preferred design:

- Proposal 170 wire/API compatibility is unchanged;
- existing accepted-server concurrency values gain earlier defense-in-depth enforcement;
- richer per-peer/rate semantics remain application-level and unchanged;
- unrelated streaming sessions keep default behavior unless they explicitly configure the standard option;
- no stored state migration is required;
- restart discards lower-layer ephemeral admission state;
- a remote peer may observe earlier reset/drop when the server is saturated, which is the intended lower-layer availability behavior and must be documented in closure evidence.

## 13. Readiness gate

M091 may move from `blocked` to `ready` only when all of the following are true:

1. M090 has an accepted closure record;
2. a concrete supported configuration transport from `i2pcontrol` through Yosemite/SAM to the Emissary streaming manager is identified;
3. that transport does not depend on magic naming/global registries or a parallel SAM implementation;
4. if a Yosemite dependency change is required, the exact internal dependency strategy is explicitly authorized and containment impacts are written down;
5. the implementation can preserve default behavior for unrelated streaming sessions;
6. exact production paths outside `i2pcontrol` are enumerated rather than glob-authorized;
7. no upstream interaction is required.

## 14. Stop conditions

Stop rather than widening M091 if:

- the only path forward is an unapproved Yosemite fork/vendor/git dependency;
- completing the concurrency check requires a broad streaming rewrite;
- a new router algorithm or routing protocol behavior is required;
- a callback from core into CLI/application policy is proposed;
- application rate-limit logic would be duplicated wholesale in core;
- unrelated streaming defaults would change without separate approval;
- a new Proposal 170 public field/action/type is proposed;
- upstream review/submission is proposed as a prerequisite.

## 15. Closure evidence required

When and only when M091 is unblocked and implemented, create `plans/closure/i2pcontrol-proposal-170/091-closure.md` containing:

- exact blocker resolution and dependency/API evidence;
- implementation baseline/head;
- exact changed-path matrix distinguishing `i2pcontrol`, core, dependency, and planning paths;
- inbound SYN ordering evidence showing rejection before pending/active/application allocation;
- requirement-to-test matrix;
- failure/release/restart/contention review;
- compatibility impact on non-I2PControl streaming sessions;
- exact manifest/lockfile disposition if changed;
- full verification results;
- residual lower-layer rate/Sybil/timing limitations with severity;
- explicit internal-only/no-upstream-interaction attestation.

## 16. Future reclosure

After M090 and M091 both have accepted closure records, the security roadmap should register one independent current-head tunnel-security reclosure milestone. That future reclosure is intentionally not registered now because M091 is blocked and planning governance requires future milestones to remain in the roadmap until dependencies are satisfied.
