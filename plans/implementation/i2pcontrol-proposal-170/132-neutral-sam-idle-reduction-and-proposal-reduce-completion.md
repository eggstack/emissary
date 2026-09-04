# M132 — Neutral SAM Idle Reduction and Proposal Reduce Completion

Status: **ready / registered**

Class: capability / neutral lower-layer exception / Proposal-170 vertical slice

Planning baseline:

- M131 closure head `3a829d7d3d6314ecf09e42dbf0339506f0917c96`;
- current matrix `284 apply / 88 blocked_primitive / 468 not_applicable`.

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

Parent authority:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `plans/closure/i2pcontrol-proposal-170/131-closure.md`;
- `plans/implementation/i2pcontrol-proposal-170/131-residual-primitive-map.toml` cluster `session-lifecycle`, path budget `PB-SESSION-REDUCTION-01`.

Architecture/security authority:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- M110/M116 shared-session ownership;
- M121 Close/NewDest semantic correction;
- M123 cancellation atomicity.

Pinned external authority is Proposal 170 revision `2026-05-20` plus read-only Java I2P/I2CP session-idle behavior at the M131 reference snapshot. All external interaction is read-only. This plan authorizes writes only to `eggstack/emissary`.

## 1. Objective

Implement the smallest neutral Emissary SAM/tunnel-pool primitive required for exact idle tunnel-quantity reduction and consume it from I2PControl for Proposal fields:

- `Reduce`;
- `ReduceCount`;
- `ReduceTime`.

The runtime effect must be real: after the configured session-idle interval, the owning destination's active inbound and outbound tunnel quantity changes to the reduced target, and subsequent qualifying session activity restores the original configured active quantity.

M132 does **not** implement `Close`, `CloseTime`, `NewDest`, `Profile`, or any other M131 residual.

## 2. Readiness and current evidence

M132 is dependency-ready because every hard/interface dependency has a concrete current owner:

- `emissary-core/src/sam/session.rs` owns the active SAM session and the actual I2CP streaming/datagram payload boundary;
- `emissary-core/src/destination/mod.rs` owns the destination and its `TunnelPoolHandle`;
- `emissary-core/src/tunnel/pool/handle.rs`, `context.rs`, and `mod.rs` own pool control/population/maintenance;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` is already the Proposal-to-Yosemite session option boundary;
- Yosemite Y005 already serializes bounded `additional_options`; no Yosemite change is required to carry standard `i2cp.reduce*` options;
- the exact Y005 dependency pin remains unchanged.

The missing primitive is therefore not a new subsystem. It is a live active-quantity control in the existing pool actor plus a session-local idle/activity state in the existing SAM session.

## 3. Semantic freeze before production edits

Before changing production code, record the exact pinned/reference behavior in the M132 closure/work notes and tests:

1. `i2cp.reduceOnIdle=true` enables reduction;
2. default `i2cp.reduceIdleTime` is 20 minutes;
3. Java minimum idle time is 5 minutes;
4. default reduced quantity is 1 and Java coerces values below 1 to 1;
5. `updateTunnels(session, quantity)` reconfigures both inbound and outbound quantity;
6. `updateTunnels(session, 0)` restores the original configured values;
7. a reduced session is restored on the next qualifying session activity;
8. primary/subsession activity aggregates at the owning primary session;
9. exact router behavior when active quantity is lowered: whether excess tunnels are immediately retired, marked non-selectable, or allowed to expire without replacement;
10. exact LeaseSet update behavior while inbound active quantity changes;
11. whether the Streamr/datagram I2P session is governed by the same generic `SessionIdleTimer` owner.

The first eight points are planning-time reference facts. Items 9-11 MUST be resolved from direct reference/runtime source before code because an approximation can affect anonymity, LeaseSet truthfulness or matrix applicability.

If Streamr applicability remains genuinely ambiguous after direct reference review, M132 may close with the three Streamr `Reduce*` cells still blocked. Do not promote them merely because the new Emissary primitive could technically be wired to datagrams.

## 4. Neutral lower-layer contract

### 4.1 Activity

Add generation-local activity state to the existing SAM session owner. It must represent the reference I2CP application-message boundary, not listener/socket occupancy.

Activity includes:

- an accepted outbound datagram application payload, including one queued pending LeaseSet lookup;
- outbound streaming protocol packets that become I2CP payload messages, including SYN/ACK/retransmission/control packets emitted by the streaming manager;
- inbound successfully decoded I2CP streaming/datagram payload delivered to the protocol manager.

Activity excludes mere SAM control traffic, naming lookup, keepalive/PING, tunnel build/maintenance and local accepted TCP handler count.

The timer clock must use the runtime's monotonic/generation-local timer primitives. No wall-clock arithmetic and no global timer task.

### 4.2 Dynamic active quantity

The configured/base `TunnelPoolConfig.num_inbound` and `num_outbound` remain immutable configuration authority.

Add a separate live active target owned by `TunnelPool`. It must support a neutral operation equivalent to:

- set live inbound/outbound active target to a bounded quantity;
- restore live targets to configured/base values.

Do not expose the pool's mutable maps or selector internals.

The seam must not contain Proposal names or policy. Candidate neutral vocabulary is `active_quantity_target`, `set_active_quantity_target`, or equivalent.

### 4.3 Control delivery

Pool-control delivery must be bounded and restoration-safe.

Preferred design: an actor-local coalescing/watch/latest-state target where the newest desired quantity replaces stale pending state.

A finite FIFO command channel is acceptable only if deterministic tests prove:

- control overload cannot grow memory;
- an activity-triggered restore cannot be silently lost after a reduction;
- shutdown/cancellation wakes or invalidates pending control cleanly;
- stale generation commands cannot affect a replacement pool.

Do not multiplex administrative control into the existing data-message channel if doing so can starve or reorder payload traffic in a security-relevant way.

### 4.4 Pool maintenance

All tunnel building, retirement, standby promotion, selection and metrics remain owned by `TunnelPool`.

The active target must influence:

- active inbound build deficit;
- active outbound build deficit;
- standby promotion only when active capacity is below the current active target;
- selection visibility when the reference says an excess tunnel is no longer active;
- no replacement above the current active target.

Backup quantity remains a separate standby target. M132 MUST NOT reinterpret `TunnelBackupQuantity`.

### 4.5 LeaseSet correctness

Inbound live-target changes must remain synchronized with the destination/LeaseSet owner.

If an inbound tunnel ceases to be active/selectable, published LeaseSet state must converge according to the exact reference behavior and must never advertise a tunnel after Emissary has made it unusable. Restoring quantity must not publish a lease before a tunnel actually exists.

Do not add LeaseSet cryptography or encrypted-LeaseSet scope.

## 5. I2PControl contract

I2PControl parses/validates the three Proposal fields before any listener/session allocation.

Required mapping after validation:

- `Reduce` -> standard `i2cp.reduceOnIdle` session option;
- `ReduceCount` -> standard `i2cp.reduceQuantity` session option;
- `ReduceTime` -> standard `i2cp.reduceIdleTime` session option using the pinned Proposal/reference unit.

Use the existing Yosemite `SessionOptions` / validated additional-session-option mechanism. Do not construct a raw `SESSION CREATE` command.

The exact option values must participate in `CompatibilityKey` equality for shared sessions. Current exact additional-option identity is expected to provide this automatically; add regression evidence.

If `Reduce` is absent/false, `ReduceCount`/`ReduceTime` presence semantics must follow the pinned Proposal/reference contract rather than silently creating an enabled policy. Invalid or unsupported combinations fail before allocation.

## 6. Authorized production path budget

M132 is an explicit neutral lower-layer exception. Production changes are limited to the following paths unless an amendment is registered before code expands:

Neutral core:

- `emissary-core/src/sam/session.rs` — generic activity/idle-reduction state and standard I2CP option consumption;
- `emissary-core/src/destination/mod.rs` — narrow forwarding method from session owner to its pool handle if required;
- `emissary-core/src/tunnel/pool/handle.rs` — narrow generic live-target control handle;
- `emissary-core/src/tunnel/pool/context.rs` — only if a dedicated bounded/coalescing control transport is required;
- `emissary-core/src/tunnel/pool/mod.rs` — live active targets and exact pool maintenance behavior.

I2PControl consumer:

- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` — Proposal validation/translation into Yosemite session options;
- `emissary-cli/src/i2pcontrol/backends/options.rs` — only if the common fail-before-allocation capability gate requires an explicit known-field adjustment;
- existing I2PControl backend files only when a specific client family must pass the already-built session options through an existing call site; exact additions must be recorded in M062 before editing.

Evidence/planning/docs:

- M061/M062 containment tests/authority;
- M095/M105/M110 matrix/ledger evidence;
- focused M132 tests;
- I2PControl support/tunnel-manager documentation;
- M132 closure/registry/roadmaps.

No `Cargo.toml`, lockfile, Yosemite, frontend, startup tunnel, transport, NetDb, crypto, profile-selection, or unrelated core path is authorized.

## 7. Ordered work packages

### WP1 — Reference/runtime freeze

Resolve semantic-freeze items 9-11 and record exact source evidence. Recompute the starting `Reduce*` cell list mechanically from M095.

### WP2 — Neutral pool control

Add the smallest live active-quantity target/control mechanism. Preserve base config and standby counts. Add deterministic actor-level tests before wiring SAM idle policy.

### WP3 — Destination bridge

If needed, expose only a narrow generic destination method to request live pool active quantity. Do not expose `TunnelPoolHandle` mutation to arbitrary consumers.

### WP4 — SAM activity state

Add one session-local activity timestamp/generation and timer state. Mark activity only at the frozen I2CP payload boundaries.

No per-byte callback, no unbounded event stream, no I2PControl observer dependency.

### WP5 — Generic reduction policy

Parse/consume the standard I2CP reduction settings in the SAM session. At idle threshold, request the reduced active target. On next qualifying activity, restore configured target before/while processing that activity according to reference ordering.

Reduction failure must not mark the session as successfully reduced if the pool owner did not accept the state change.

### WP6 — I2PControl validation/translation

Enable the Proposal fields only after core capability exists. Map them through Yosemite's validated generic session-option path. Preserve fail-before-allocation for malformed values and unsupported families.

### WP7 — Shared-session semantics

Prove:

- compatible members with identical reduction policy share one session;
- differing reduction policy prevents sharing;
- activity from any member sharing one session restores that session's pool;
- final-member release still tears down the owner exactly once;
- creator cancellation cannot strand idle-policy state.

### WP8 — Matrix/docs/closure

Promote only cells with end-to-end reference-equivalent evidence. Mechanically reconcile M095/M105/M110 and docs. Do not use a target count as proof.

## 8. Failure, cancellation, restart and contention

- Idle timers are owned by one SAM session generation and disappear with it.
- Pool controls carry generation-local ownership; stale controls cannot reach a replacement pool.
- A failed reduction control remains retryable on a later timer/activity transition and must not permanently suppress restoration.
- A failed restore is a correctness failure: do not report normal reduced/restored state while the pool target remains stale.
- Pool shutdown wins over pending reduction/restore without an orphan task.
- Process restart starts a fresh timer from the new session generation; no idle timestamp is persisted.
- Shared-session registry locks are never held across Yosemite/network/core I/O.
- Pool locks/state are never held across tunnel build network I/O.
- Control saturation behavior is finite and deterministic.

## 9. Compatibility and migration

Absent reduction options, SAM-created destination behavior must remain byte-for-byte/configuration-equivalent at the option level and behaviorally equivalent at runtime.

No durable-store migration is required. Existing raw Proposal fields remain round-trippable. If implementation chooses to add typed I2PControl fields for internal validation, old stored definitions must deserialize without migration and canonical `get` output must not change unexpectedly.

No API version, method, tunnel type or action changes.

## 10. Security review requirements

M132 closure must explicitly verify:

- one destination cannot alter another destination's pool target;
- exploratory/participating pools are unreachable from this session control seam;
- reduction never broadens peer selection, hop ranges or direct-clearnet behavior;
- backup tunnels are not accidentally exposed as active traffic paths merely to satisfy reduced quantity;
- LeaseSet state does not advertise unusable tunnels;
- no secret/session key/raw private destination appears in new logs/events/debug output;
- malformed options fail before I2PControl allocation;
- core contains no Proposal/I2PControl names.

## 11. Focused tests

At minimum add deterministic tests for:

1. no idle options -> unchanged active/base quantities;
2. reduction enabled -> no reduction before threshold;
3. exact threshold -> live inbound/outbound active target changes to configured reduced quantity;
4. subsequent qualifying outbound activity restores base target;
5. subsequent qualifying inbound activity restores base target;
6. naming/PING/control activity does not restore/reset;
7. streaming protocol send boundary resets activity;
8. datagram send/receive resets activity where reference applicability is proven;
9. primary/subsession activity aggregation matches reference;
10. shared I2PControl members aggregate activity and differing policies do not share;
11. backup quantity remains unchanged across reduction/restore;
12. active build deficit follows the live target and does not rebuild excess capacity;
13. excess active tunnel retirement/expiry matches the frozen reference behavior;
14. LeaseSet updates remain truthful during reduction/restore;
15. stale generation control is ignored/dropped;
16. bounded control saturation cannot lose the newest restore target;
17. pool/session shutdown clears idle/control state;
18. malformed, overflow, below-minimum and wrong-type Proposal inputs fail before allocation as required by the Proposal/reference boundary;
19. Yosemite command serialization contains the exact standard I2CP keys and no raw-command injection;
20. exact additional-option equality participates in shared compatibility;
21. M061/M062 changed-path guards reject unauthorized expansion.

## 12. Broad verification

Run:

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Record pre-existing stable/nightly rustfmt drift rather than formatting unrelated files.

## 13. Matrix accounting

Starting authority is `284 / 88 / 468`.

There are 21 mechanically present `Reduce`/`ReduceCount`/`ReduceTime` client cells across seven client families. M132 does not assume all 21 become `apply`.

- If reference evidence proves the generic session policy applies to Streamr/datagram sessions and end-to-end tests pass, the maximum M132 promotion is 21 cells.
- If Streamr remains ambiguous or behaviorally non-equivalent, only the six non-Streamr client families may be promoted (maximum 18 cells) and the three Streamr cells remain blocked.

Closure must mechanically compute the actual matrix. Count reduction is not acceptance evidence.

## 14. Documentation/static guards

Update support documentation to describe exact idle-reduction behavior only after runtime evidence exists.

Add static containment guards that:

- changed core paths contain no `I2PControl`, `Proposal 170`, `ReduceCount`, `ReduceTime`, `TunnelManager`, or JSON-RPC names;
- M062 exact-path authority matches the real production diff;
- no Cargo/Yosemite dependency change occurred;
- M095 and M105 counts/hash agree.

## 15. Acceptance criteria

M132 closes only when all are true:

1. reference behavior and Streamr applicability are frozen with direct evidence;
2. SAM activity is measured at the correct payload boundary;
3. live pool active targets can reduce and restore without mutating configured/base quantities;
4. backup and LeaseSet behavior remain reference-correct;
5. reduction/restore is bounded, cancellation-safe and generation-local;
6. Proposal fields are validated before allocation and serialized through Yosemite without raw SAM;
7. shared-session equality/activity behavior is exact;
8. every promoted matrix cell has end-to-end runtime evidence;
9. no unauthorized production/dependency path changed;
10. broad verification has no unexplained regression;
11. M133 readiness is explicitly decided from the closure head.

## 16. Stop conditions

Stop and close the affected slice blocked if:

- reference pool downsizing behavior cannot be established without guessing;
- truthful LeaseSet maintenance requires a broad LeaseSet/router redesign;
- dynamic quantity requires exposing general mutable router control outside the canonical pool owner;
- correct control delivery cannot be bounded/restoration-safe;
- a Yosemite modification or new dependency becomes necessary without a separately registered dependency plan;
- Streamr applicability cannot be established;
- an implementation path outside the authorized budget becomes necessary before a plan amendment.

Partial completion is acceptable. Approximate support is not.

## 17. Closure evidence required

The M132 closure must contain:

- exact implementation commit(s);
- exact changed production paths and M062 amendment;
- reference semantic table;
- starting/final cell list and mechanically recomputed matrix;
- requirement-to-evidence matrix;
- focused test names/results;
- broad verification commands/results;
- pool/LeaseSet/security review;
- cancellation/restart/contention review;
- shared-session review;
- compatibility/migration statement;
- unresolved findings with severity;
- M133 readiness decision;
- internal-only/read-only-upstream attestation.