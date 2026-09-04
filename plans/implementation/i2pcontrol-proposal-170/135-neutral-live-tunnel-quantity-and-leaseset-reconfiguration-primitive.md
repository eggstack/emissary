# M135 — Neutral Live Tunnel Quantity and LeaseSet Reconfiguration Primitive

Status: **ready / registered**

Class: corrective prerequisite / neutral lower-layer capability / zero-P170-promotion primitive

Planning baseline:

- current `master` before M135 planning: `517decf733352dfc2bf24ad349c5ab4cf9315742`;
- M132 closure head `6618c49a4bcf962a1ee263fa97fa95a3b70f1ad2`;
- M133 closure head `517decf733352dfc2bf24ad349c5ab4cf9315742`;
- current matrix `284 apply / 88 blocked_primitive / 468 not_applicable`.

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

Corrective ancestry:

- M132 `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md` closed as blocked;
- M132 closure `plans/closure/i2pcontrol-proposal-170/132-closure.md`;
- M131 primitive cluster `session-lifecycle`, path budget `PB-SESSION-REDUCTION-01`.

Architecture/security authority:

- canonical plans 000-003;
- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- M118/M119 neutral tunnel-pool variance/backup precedent;
- M121 semantic-truthfulness correction;
- M123 cancellation/commit atomicity;
- M130 current runtime/security qualification;
- M131 current residual applicability/primitive authority.

Pinned external authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- read-only Java I2P reference snapshot `i2p/i2p.i2p@2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`.

All external repositories remain read-only. M135 authorizes writes only to `eggstack/emissary`.

## 1. Objective

Establish the smallest generic Emissary lower-layer primitive needed to change a client destination's **current desired inbound/outbound tunnel quantity** at runtime while preserving:

- immutable configured/base quantities;
- existing active tunnels until normal lifecycle removal;
- backup/standby quantity semantics;
- correct future build deficit;
- truthful LeaseSet publication target;
- destination isolation;
- bounded, generation-local control.

M135 intentionally implements **no Proposal-170 field**, no SAM idle timer and no matrix promotion.

Its sole purpose is to make the lower-layer quantity reconfiguration operation independently correct, testable and reusable. M136 may consume it later for Proposal `Reduce`, `ReduceCount` and `ReduceTime` only after M135 closes successfully.

## 2. Why M132 blocked and what M135 corrects

M132 combined too many contracts into one closure gate:

1. Java/reference quantity behavior;
2. a live tunnel-pool target;
3. LeaseSet synchronization;
4. SAM application-activity ownership;
5. idle timer/reduction policy;
6. I2PControl validation/translation;
7. matrix promotion.

Its closure stopped before production edits because reference items 9-11 were not directly retrieved and because the proposed path budget omitted the actual LeaseSet target owner.

M135 corrects the decomposition. It freezes the lower-layer behavior from direct source evidence and authorizes only the neutral pool/destination/LeaseSet primitive. No Proposal acceptance can depend on M135 until a later plan proves session policy.

## 3. Direct reference freeze

The following behavior is frozen before implementation and is part of M135 acceptance.

### 3.1 Client-side reduction is a session reconfiguration

Java `I2CPMessageProducer.updateTunnels(session, tunnels)` sends a `ReconfigureSessionMessage` with inbound/outbound quantity replaced by `tunnels`; `tunnels == 0` restores the original configured quantities.

Reference:

- `core/java/src/net/i2p/client/impl/I2CPMessageProducer.java`.

### 3.2 Router-side reconfiguration replaces current pool settings

Java `ClientMessageEventListener.handleReconfigureSession()` converts the updated session config into `ClientTunnelSettings` and invokes:

- `TunnelManagerFacade.setInboundSettings(dest, settings)`;
- `TunnelManagerFacade.setOutboundSettings(dest, settings)`.

`TunnelPoolManager.setSettings()` forwards those settings to the existing pool, and `TunnelPool.setSettings()` replaces the current pool settings and wakes maintenance.

References:

- `router/java/src/net/i2p/router/client/ClientMessageEventListener.java`;
- `router/java/src/net/i2p/router/tunnel/pool/TunnelPoolManager.java`;
- `router/java/src/net/i2p/router/tunnel/pool/TunnelPool.java`.

### 3.3 Lowering quantity does not immediately purge excess live tunnels

The Java pool build algorithm computes future build demand from the **current settings quantity**. Existing live tunnels remain in the pool/selection set and age out through normal expiration/failure. Lowering quantity therefore stops replacement/building above the new desired target rather than synchronously destroying or hiding excess tunnels.

M135 MUST implement this lifecycle shape unless direct pinned-source review before code proves a contradictory newer behavior at the same authority snapshot.

### 3.4 LeaseSet wanted count follows current inbound quantity

Java `TunnelPool.locked_buildNewLeaseSet()` computes:

`wanted = min(current inbound settings quantity, LeaseSet.MAX_LEASES)`.

The LeaseSet is rebuilt on normal inbound tunnel lifecycle events. M135 therefore must update the local LeaseSet owner's **desired inbound count** when the pool target changes, but must not invent immediate tunnel retirement or publish a lease for a tunnel that does not exist.

Reference:

- `router/java/src/net/i2p/router/tunnel/pool/TunnelPool.java`.

### 3.5 Restore is a quantity reconfiguration, not pool recreation

The Java session restores original configured quantities by reconfiguration. Emissary must likewise retain one destination/pool generation and update its desired targets; M135 must not tear down/recreate the pool merely to change quantity.

## 4. Neutral architecture contract

### 4.1 Base versus live target

`TunnelPoolConfig.num_inbound` and `num_outbound` remain immutable configured/base authority.

`TunnelPool` gains separate current desired active targets, conceptually:

- `desired_inbound_quantity`;
- `desired_outbound_quantity`.

Names may differ, but the distinction must remain explicit. Do not mutate the stored base config as the mechanism for runtime reduction.

Initial desired targets equal the configured/base values.

### 4.2 Reconfiguration operation

Expose a narrow destination-scoped operation equivalent to:

- set desired inbound/outbound quantities to bounded values;
- restore desired quantities to configured/base values.

The API must be generic and contain no `Reduce`, `Close`, Proposal 170 or I2PControl vocabulary.

Preferred shape is one latest-state/coalescing target update carrying both quantities atomically. If separate inbound/outbound updates are used, tests must prove intermediate mismatch cannot cause incorrect LeaseSet/build behavior.

### 4.3 Pool behavior

The current desired target controls:

- active inbound build deficit;
- active outbound build deficit;
- whether standby inbound/outbound tunnels are promoted when active capacity drops;
- whether expired/failed tunnels are replaced;
- metrics that explicitly represent desired active capacity, if any.

It does **not**:

- delete otherwise-valid excess active tunnels immediately;
- hide otherwise-valid excess active tunnels from ordinary selection solely because the desired target was lowered;
- reinterpret backup quantity;
- change hop length/variance/peer selection;
- affect exploratory or participating pools.

The normal tunnel expiry/failure path is responsible for convergence down toward the new target.

### 4.4 Pending builds

A target decrease may race with already-started builds. M135 must define and test a bounded reference-compatible rule.

Preferred rule:

- already-dispatched builds may complete;
- completed excess tunnels may remain active until normal lifecycle removal;
- no additional replacement/build is initiated while current usable+pending capacity is at/above the desired target;
- restore may resume building immediately through the normal maintenance wake path.

Do not add cancellation plumbing to remote tunnel build exchanges solely to make the count drop immediately.

### 4.5 LeaseSet desired count

`LeaseSetManager` currently stores a fixed `num_inbound` used to decide when enough inbound tunnels exist for a new LeaseSet.

M135 authorizes a generic update of that **desired inbound count**. On target change:

- the desired count becomes the new current inbound target;
- existing lease/tunnel records are not fabricated or synchronously deleted merely because target decreased;
- normal inbound lifecycle events rebuild/publish using current real leases and the current desired count;
- restore may not claim/publish missing tunnels before they are actually built;
- unpublished destinations remain unpublished.

The exact local wake/state transition required after a desired-count change must be bounded and deterministic. If a target increase means the manager should await additional tunnels, it must enter the existing await-tunnels state or a semantically equivalent existing-owner state rather than publish prematurely.

### 4.6 Destination bridge

`Destination` is the canonical owner joining its `TunnelPoolHandle` and `LeaseSetManager`. If coordination is required, add one narrow generic method there so callers cannot independently mutate the pool target and LeaseSet target into inconsistent states.

Preferred authority:

`Destination::set_tunnel_quantity_target(inbound, outbound)` (or neutral equivalent) performs/coordinates both lower-layer desired-target changes.

Do not expose mutable pool maps or mutable `LeaseSetManager` to SAM/I2PControl.

## 5. Control delivery and boundedness

M135 must not create an unbounded command/event channel.

Acceptable designs:

- actor-local latest-state/watch/coalescing control;
- bounded command channel with deterministic newest-state preservation;
- direct owner-local mutation if the current task architecture proves single-owner access without crossing async ownership boundaries.

Whichever design is chosen must prove:

- one destination cannot alter another destination's target;
- stale generation updates cannot reach replacement pools;
- shutdown wins cleanly;
- target restore cannot be permanently lost to queue saturation;
- no lock is held across tunnel build/network I/O;
- no unbounded task is spawned per target update.

## 6. Authorized production path budget

M135 is an explicit neutral lower-layer exception. Production edits are limited to the following exact areas unless the plan is amended and re-registered before expansion.

### Tunnel pool

- `emissary-core/src/tunnel/pool/handle.rs` — narrow generic target-control handle if required;
- `emissary-core/src/tunnel/pool/context.rs` — only if bounded/coalescing control transport is required;
- `emissary-core/src/tunnel/pool/mod.rs` — desired targets and quantity-sensitive maintenance decisions.

### Destination / LeaseSet

- `emissary-core/src/destination/mod.rs` — atomic/narrow coordination bridge;
- `emissary-core/src/destination/lease_set.rs` — dynamic desired inbound count and existing-state wake/reconciliation only.

### Tests/evidence/planning

- focused unit/integration tests colocated with the above owners;
- M061/M062 containment evidence;
- roadmap/registry/closure documents.

M135 authorizes **no** production change to:

- `emissary-core/src/sam/**`;
- `emissary-cli/src/i2pcontrol/**`;
- Yosemite;
- Cargo manifests/lockfile;
- NetDb protocol/crypto;
- transport/router peer selection;
- frontend/startup/config surfaces.

If the primitive cannot be implemented truthfully inside this budget, stop and record the specific missing owner rather than expanding by convenience.

## 7. Ordered work packages

### WP1 — Re-freeze reference vectors

Record direct source locations and deterministic behavior vectors for:

- quantity decrease with excess live tunnels;
- quantity decrease with pending builds;
- no replacement above target;
- restore to configured quantity;
- LeaseSet wanted-count behavior.

No production edit before this table exists in work notes/tests.

### WP2 — Pool desired-target state

Add desired inbound/outbound quantities initialized from base config. Replace only quantity-decision reads that represent **current desired active capacity**.

Do not globally replace every `config.num_*` use: hop/build parameters and configured/base reporting must remain base-owned where appropriate.

### WP3 — Bounded control seam

Add the smallest generic owner-to-pool target update mechanism. Deterministically test coalescing/saturation/generation behavior before destination wiring.

### WP4 — LeaseSet desired-count state

Make `LeaseSetManager`'s publication readiness use a current desired inbound count that can change without reconstructing the manager.

### WP5 — Destination atomic coordination

Expose one narrow destination-scoped reconfiguration method that keeps pool and LeaseSet desired targets coherent.

Define failure semantics. If one side cannot accept an update, do not leave a silently divergent target pair. Prefer owner-local sequencing that cannot partially fail; otherwise provide explicit rollback.

### WP6 — End-to-end neutral tests

Exercise one destination through decrease, normal excess expiry, target convergence, restore and rebuild. Verify LeaseSet creation requests never reference nonexistent tunnels and do not wait for the old base count after reduction.

### WP7 — Containment and closure

Update M062 to the exact realized production diff, run verification, write M135 closure and explicitly decide whether M136 is dependency-ready.

M135 closure MUST NOT modify M095 support cells.

## 8. Required focused tests

At minimum:

1. desired targets initialize to configured inbound/outbound quantities;
2. lowering target changes future build deficit without mutating base config;
3. lowering target does not synchronously remove valid excess inbound tunnels;
4. lowering target does not synchronously remove valid excess outbound tunnels;
5. excess active tunnels remain ordinary selectable tunnels until normal expiry/failure;
6. no replacement is built while live/pending capacity is at/above desired target;
7. pending build completion above target does not trigger another replacement;
8. restore resumes normal build deficit toward base quantities;
9. backup target/count is unchanged across target changes;
10. standby promotion uses desired active target rather than base target;
11. exploratory pool cannot be controlled through the client-destination seam;
12. another destination's target is unaffected;
13. LeaseSet desired inbound count follows target decrease;
14. LeaseSet desired inbound count follows restore;
15. target increase waits for real tunnels before full-count LeaseSet creation;
16. target decrease does not fabricate lease removal or new leases;
17. unpublished destination behavior is unchanged;
18. stale-generation/closed control is rejected or harmlessly dropped;
19. bounded/coalescing control preserves the latest restore target;
20. no Proposal/I2PControl vocabulary appears in changed core source.

## 9. Failure, cancellation, restart and contention

- Desired targets are generation-local and never persisted.
- Process/session restart initializes desired targets from base config.
- Pool shutdown invalidates target updates and wakes any bounded control waiter.
- Target updates do not hold locks across network/build I/O.
- A failed target update is explicit; no caller may assume convergence from fire-and-forget delivery without an authoritative owner result/state.
- If the implementation uses a channel, overload behavior is deterministic and finite.
- Existing excess tunnel expiry/failure remains idempotent under target changes.
- LeaseSet state transitions remain owned by `LeaseSetManager`; no second publisher/timer is created.

## 10. Compatibility

When no runtime reconfiguration method is called, behavior must remain equivalent to the current M130-qualified runtime:

- same configured quantities;
- same backup quantities;
- same tunnel lengths/variance;
- same build/selection behavior;
- same LeaseSet publication behavior.

No durable-state migration, public API version, SAM command, I2PControl method or Proposal matrix change is permitted.

## 11. Security review

Closure must explicitly verify:

- per-destination ownership and no cross-destination control;
- exploratory/participating pools remain unreachable;
- target reduction cannot force zero-hop/direct-clearnet behavior;
- peer/hop selection is unchanged;
- backup capacity is not silently promoted beyond the desired active target;
- LeaseSet never advertises a nonexistent/unusable tunnel due solely to target control;
- no key/secret/path data is added to logs/debug/events;
- no new unbounded queue/task/timer exists;
- changed core source contains no Proposal/I2PControl policy vocabulary.

## 12. Broad verification

Run at minimum:

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Record known pre-existing stable/nightly rustfmt drift rather than normalizing unrelated source.

## 13. Matrix accounting

Starting and required closing matrix for M135 is exactly:

- `284 apply`;
- `88 blocked_primitive`;
- `468 not_applicable`.

M135 is infrastructure only. **Any M095 cell promotion or applicability change under M135 is a plan violation.**

## 14. Closure evidence

M135 closure must include:

- exact changed production paths;
- direct Java reference source table;
- before/after base and desired target traces;
- excess-tunnel lifecycle trace;
- LeaseSet desired-count/convergence trace;
- bounded control/generation evidence;
- containment and broad verification results;
- unchanged matrix hash/count evidence;
- explicit M136 readiness decision.

## 15. Stop conditions

Close as blocked without partial production churn if any of the following proves necessary:

- immediate forced retirement is required for reference equivalence;
- truthful LeaseSet convergence requires NetDb protocol/crypto redesign;
- target control requires exposing arbitrary pool mutation outside the destination owner;
- implementation requires Yosemite/SAM/I2PControl changes;
- destination isolation cannot be proven;
- required control cannot be bounded/generation-local;
- path expansion outside §6 is materially necessary and the plan is not amended first.

## 16. Success criterion

M135 succeeds only when Emissary has an independently correct, neutral, destination-scoped live tunnel-quantity/LeaseSet desired-count reconfiguration primitive with deterministic reference-equivalent lower-layer behavior and **zero Proposal support promotion**.

A successful M135 closure may register M136 as the sole next handoff. It does not itself make `Reduce*`, `Close*` or `NewDest` operational.