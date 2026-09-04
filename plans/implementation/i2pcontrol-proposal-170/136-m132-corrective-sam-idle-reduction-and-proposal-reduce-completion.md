# M136 — M132 Corrective: SAM Idle Reduction and Proposal Reduce Completion

Status: **closed as complete**

Closure authority: `plans/closure/i2pcontrol-proposal-170/136-closure.md`

M135 closure `plans/closure/i2pcontrol-proposal-170/135-closure.md` proves all
§2 gate items (destination-scoped update/restore, immutable base,
desired-driven deficit/standby, excess convergence, dynamic LeaseSet count,
bounded generation-local control, green containment, unchanged `284/88/468`
matrix, no high/medium defect). No production work is authorized until the
registration step flips this status.

Class: corrective implementation / neutral SAM policy + I2PControl consumer

Corrects:

- M132 `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`;
- M132 closure `plans/closure/i2pcontrol-proposal-170/132-closure.md`.

Planning baseline:

- M133 closure/current pre-M135 baseline `517decf733352dfc2bf24ad349c5ab4cf9315742`;
- matrix `284 apply / 88 blocked_primitive / 468 not_applicable`;
- M135 must close successfully and provide the neutral live-quantity/LeaseSet primitive described by `135-neutral-live-tunnel-quantity-and-leaseset-reconfiguration-primitive.md`.

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

Pinned authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- read-only Java I2P reference snapshot `i2p/i2p.i2p@2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`.

Architecture/security authority:

- canonical plans 000-003;
- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- M110/M116 shared-session ownership;
- M121 semantic correction;
- M123 cancellation atomicity;
- M130 runtime/security qualification;
- M131 residual primitive authority;
- successful M135 closure.

External repositories remain read-only. Writes are limited to `eggstack/emissary`.

## 1. Objective

After M135 proves the neutral lower-layer reconfiguration primitive, implement one generic generation-local SAM/I2P-session idle activity owner and consume it for:

- standard `i2cp.reduceOnIdle`;
- standard `i2cp.reduceIdleTime`;
- standard `i2cp.reduceQuantity`;
- Proposal `Reduce`;
- Proposal `ReduceTime`;
- Proposal `ReduceCount`.

Reduction must alter real destination tunnel targets through M135 and restore the configured/base targets on qualifying activity.

M136 does **not** implement `Close`, `CloseTime` or `NewDest`. M137 is the only planned close-on-idle successor.

## 2. Registration/readiness gate

M136 MUST NOT be registered until M135 closure proves all of the following:

1. one destination-scoped generic target update changes current desired inbound/outbound quantities;
2. base configuration remains immutable;
3. future build deficit and standby promotion use the desired target;
4. excess live tunnels remain valid until normal lifecycle removal;
5. LeaseSet desired count follows current inbound target without fabricated leases;
6. restore to base target is deterministic;
7. target control is bounded and generation-local;
8. M061/M062 containment is green;
9. M135 matrix remains exactly `284/88/468`;
10. no unresolved high/medium correctness defect remains in the primitive.

If any gate fails, keep M136 deferred and amend/replan rather than registering around the dependency.

## 3. Corrective reference freeze

M132 stopped because direct evidence for the session/pool behavior was unavailable during that execution. M136 freezes the now-resolved reference behavior before production edits.

### 3.1 Generic session owner

Java `I2PSessionImpl.startIdleMonitor()` reads:

- `i2cp.reduceOnIdle`;
- `i2cp.closeOnIdle`.

If either is enabled, it initializes activity and schedules `SessionIdleTimer`.

Reference:

- `core/java/src/net/i2p/client/impl/I2PSessionImpl.java`.

### 3.2 Reduction defaults and bounds

Java `SessionIdleTimer` defines:

- minimum idle time: 5 minutes / 300000 ms;
- default reduction time: 20 minutes / 1200000 ms;
- reduced quantity default: 1;
- parsed reduced quantity coerced to at least 1.

Reference:

- `core/java/src/net/i2p/client/impl/SessionIdleTimer.java`.

### 3.3 Reduction behavior

At/after the reduction deadline, Java calls:

`session.getProducer().updateTunnels(session, reduceQuantity)`

then marks the session reduced. Later qualifying activity calls `updateTunnels(this, 0)` to restore original configured quantities.

M136 must use the M135 primitive for equivalent behavior rather than duplicating pool mechanics.

### 3.4 Primary/subsession aggregation

Java `SubSession` delegates:

- `updateActivity()`;
- `lastActivity()`;
- `setReduced()`

to its primary session. Idle policy therefore belongs to the owning shared session, not an individual front-end handler.

Reference:

- `core/java/src/net/i2p/client/impl/SubSession.java`.

### 3.5 Streamr/datagram applicability

Java Streamr client extends `I2PTunnelUDPClientBase`, which creates a normal `I2PSession` using generic tunnel client options and calls `_session.connect()`.

The generic `I2PSessionImpl` idle monitor consumes `i2cp.reduce*`/`close*`; therefore Streamr/datagram sessions are governed by the same session-level idle owner.

References:

- `apps/i2ptunnel/java/src/net/i2p/i2ptunnel/streamr/StreamrConsumer.java`;
- `apps/i2ptunnel/java/src/net/i2p/i2ptunnel/udpTunnel/I2PTunnelUDPClientBase.java`;
- `core/java/src/net/i2p/client/impl/I2PSessionImpl.java`.

M136 may therefore target all seven client families for `Reduce*`, including Streamr, but only end-to-end tests may promote the cells.

## 4. Activity contract

M136 introduces exactly one generation-local activity state owner in the existing SAM session lifecycle.

The owner must reflect the reference I2P application-message boundary, not local TCP listener occupancy.

Qualifying activity includes:

- outbound streaming payload/protocol packets accepted for I2P delivery;
- inbound streaming payload/protocol packets successfully delivered into the local streaming manager;
- outbound datagrams accepted for I2P delivery;
- inbound datagrams successfully delivered to the SAM/datagram consumer;
- activity from any member sharing the same underlying destination/session generation.

Activity excludes:

- local handler count;
- idle local TCP sockets with no I2P traffic;
- SAM PING/PONG/control chatter;
- name/address lookup control;
- tunnel build/maintenance;
- NetDb maintenance;
- I2PControl RPC traffic.

The implementation agent must identify the exact current Emissary send/receive call sites and record them in closure evidence. Do not sprinkle per-byte callbacks through applications.

## 5. Timer/state-machine contract

Use one monotonic generation-local state machine, conceptually:

- `Active`;
- `Reduced { last_activity_generation/time }`.

M137 may later extend this same owner with close-on-idle. Do not create a timer implementation that cannot be extended.

Requirements:

- fresh session generation initializes last activity at activation;
- absent `reduceOnIdle`, no reduction timer/work exists;
- deadline is computed from monotonic runtime time;
- activity before deadline resets effective idle age;
- at deadline, request M135 target `reduceQuantity` for both inbound/outbound;
- only authoritative success marks the session reduced;
- while still idle and reduced, do not repeatedly enqueue unbounded identical controls;
- first qualifying activity while reduced requests restore-to-base;
- failed restore remains a correctness-visible state and must be retried or fail the session according to a bounded explicit rule; it must not falsely mark restored;
- session teardown cancels/invalidates timer state;
- replacement generation cannot inherit prior timer/reduced state.

A single actor-local timer/reschedule loop is preferred. No global idle scheduler is required.

## 6. Standard option consumption

M136 may consume the following standard I2CP keys in Emissary's SAM session owner:

- `i2cp.reduceOnIdle`;
- `i2cp.reduceIdleTime`;
- `i2cp.reduceQuantity`.

No Proposal vocabulary belongs in core.

Parsing must be deterministic and bounded. The I2PControl caller is expected to validate before allocation, but core must fail safely if a non-I2PControl SAM client supplies malformed values.

Reference-compatible semantics:

- reduction disabled unless `reduceOnIdle` parses true;
- default time 1200000 ms;
- minimum time 300000 ms;
- default quantity 1;
- quantity lower than 1 coerces or rejects according to the frozen generic SAM compatibility decision recorded before implementation; I2PControl itself must enforce Proposal-valid values before allocation.

Do not consume `closeOnIdle` yet except to preserve/parse it as future M137 input if unavoidable. M136 must not close a session.

## 7. Yosemite/I2PControl translation

Yosemite Y005's typed reduce fields are dormant on its serializer, but its validated generic additional-session-option path can serialize non-reserved standard keys.

M136 MUST use that existing path rather than change Yosemite or create raw SAM commands.

I2PControl mapping after validation:

- `Reduce` -> `i2cp.reduceOnIdle`;
- `ReduceTime` -> `i2cp.reduceIdleTime` in the pinned Proposal/reference unit;
- `ReduceCount` -> `i2cp.reduceQuantity`.

The implementation must freeze exact Proposal type/unit/presence semantics from M095/M105 and the pinned Proposal before editing.

Invalid/unsupported combinations fail before listener/session allocation.

If `Reduce` is absent/false, supplied `ReduceTime`/`ReduceCount` behavior must match pinned Proposal/reference semantics; do not silently enable reduction unless the Proposal requires it.

## 8. Shared-session semantics

Existing I2PControl shared-session compatibility must include the exact standard reduction option identity.

Required behavior:

- definitions with equal session-relevant options may share one underlying session;
- differing `Reduce`, `ReduceTime` or `ReduceCount` must prevent sharing if they would create different session policy;
- activity from any definition sharing the same underlying session resets/restores that one session's idle state;
- releasing one member does not reset the shared activity state;
- final-member release tears down the owner exactly once;
- creator cancellation cannot strand a timer or reduced target.

Prefer existing `additional_options_identity`/`CompatibilityKey` behavior if it already provides exact equality; prove it rather than adding duplicate policy.

## 9. Authorized production path budget

M136 is a lower-layer exception plus I2PControl consumer. Exact edits are limited to:

### Neutral core

- `emissary-core/src/sam/session.rs` — activity state, standard reduce option consumption, monotonic idle policy;
- `emissary-core/src/destination/mod.rs` — call the M135 reconfiguration bridge if the current API requires a narrow forwarding method only;
- M135-established generic APIs in `emissary-core/src/tunnel/pool/**` and `destination/lease_set.rs` only for corrective defects discovered while consuming the already-closed primitive; any semantic expansion requires plan amendment.

### I2PControl

- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` — Proposal validation/translation into Yosemite session options;
- `emissary-cli/src/i2pcontrol/backends/options.rs` only if common known-field validation requires it;
- exact client backend files only where the existing option bundle must be passed through an already-existing constructor/call site.

### Evidence

- focused tests;
- M061/M062 containment;
- M095/M105/M110 matrix/ledger evidence;
- support docs;
- closure/registry/roadmaps.

No Cargo/lockfile/Yosemite/frontend/NetDb/crypto/transport/peer-selection change is authorized.

## 10. Ordered work packages

### WP1 — Rebase on M135 closure

Read M135 closure and freeze the exact generic target-control API. If M135 changed materially from this plan's assumed interface, amend M136 before code.

### WP2 — Reference/activity call-site table

Record all actual Emissary SAM streaming/datagram send/receive boundaries that qualify as session activity. Record excluded control paths.

### WP3 — Generic activity state

Add one session-generation activity owner with deterministic paused-time tests.

### WP4 — Reduction state machine

Consume standard `i2cp.reduce*`, invoke M135 reduction/restore, and test error/cancellation/restart behavior without I2PControl.

### WP5 — Streamr/datagram proof

Exercise the datagram/Streamr session path through the same owner. No Streamr cell promotion without a real runtime trace.

### WP6 — I2PControl translation

Enable fail-before-allocation validation and map Proposal fields through Yosemite generic session options. Prove exact wire/session option serialization.

### WP7 — Shared-session proof

Test policy equality, aggregated activity, cancellation and final-member release.

### WP8 — Matrix/docs/closure

Promote only evidence-backed cells. Mechanically update M095/M105/M110 and support docs. Explicitly decide M137 readiness.

## 11. Required tests

At minimum:

1. no reduce options -> no timer/control and unchanged M135 target;
2. reduce enabled -> no reduction before threshold;
3. exact threshold -> M135 desired inbound/outbound target becomes reduced quantity;
4. base config remains unchanged;
5. outbound streaming activity resets idle before reduction;
6. inbound streaming activity resets idle before reduction;
7. outbound datagram activity resets idle;
8. inbound datagram activity resets idle;
9. SAM control/PING/name lookup does not reset activity;
10. local TCP handler count does not define activity;
11. activity after reduction restores base target;
12. failed reduction does not mark reduced;
13. failed restore does not falsely mark restored;
14. repeated idle ticks do not create unbounded duplicate controls;
15. session shutdown clears timer/reduced state;
16. replacement generation ignores stale timer/control;
17. shared-session member activity aggregates;
18. differing reduction policies do not share;
19. `Reduce`/`ReduceTime`/`ReduceCount` malformed values fail before allocation;
20. exact Yosemite `SESSION CREATE` contains standard keys with no raw-command injection;
21. all seven client families, including Streamr, have end-to-end evidence before promotion;
22. server-family cells remain not applicable;
23. M061/M062 containment rejects unauthorized expansion.

## 12. Failure/cancellation/restart

- Timer state is not persisted.
- Process/session restart starts fresh active state.
- Shutdown wins over timer/reduction/restore work.
- No lock spans Yosemite/network/pool I/O.
- Stale generations cannot change replacement targets.
- A failed lower-layer control is explicit; no support claim is made while desired target is unknown/stale.
- Shared registry cancellation rules from M116/M123 remain authoritative.

## 13. Security requirements

Closure must prove:

- no destination can control another destination's pool;
- no exploratory/participating pool is reachable;
- reduction does not alter hop count, peer selection or clearnet behavior;
- backup capacity semantics remain M135/M118-owned;
- no LeaseSet advertises fabricated tunnels;
- no secret/private destination material is logged;
- malformed Proposal inputs fail before effect;
- core source contains no Proposal/I2PControl vocabulary.

## 14. Verification

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

Known pre-existing stable/nightly rustfmt drift is recorded without unrelated normalization.

## 15. Matrix accounting

Starting matrix after M135 must still be `284 / 88 / 468`.

There are 21 current blocked `Reduce*` cells:

- 7 `Reduce` client cells;
- 7 `ReduceCount` client cells;
- 7 `ReduceTime` client cells.

Direct reference evidence now supports session-level applicability for Streamr, so the maximum M136 promotion is 21 cells. The theoretical post-M136 count would be `305 apply / 67 blocked / 468 not_applicable` if and only if every cell receives end-to-end evidence.

That number is **not** an acceptance target. Partial promotion is required if any family remains unproven.

## 16. M137 readiness

M136 closure may register M137 only if it proves:

- one canonical session activity clock/state owner;
- monotonic generation-local timer behavior;
- activity aggregation across shared members;
- deterministic shutdown/restart isolation;
- stable standard option parser;
- M135 control remains correct under timer-driven use;
- no unresolved high/medium lifecycle defect.

## 17. Stop conditions

Stop and close blocked rather than approximate if:

- qualifying activity cannot be represented at the actual session payload boundary;
- M135 control cannot be driven authoritatively from the session owner;
- restore failures cannot be made truthful/bounded;
- Streamr runtime differs materially from the direct reference session-level model;
- Yosemite changes become necessary;
- path expansion outside §9 is required without amendment;
- any proposed support would be accept-inert.

## 18. Success criterion

M136 succeeds when real idle reduction/restoration is operational through the neutral M135 primitive for each promoted client family, Proposal `Reduce*` fields are validated and mapped exactly, shared-session semantics are correct, and closure provides enough stable lifecycle authority to decide M137 registration.