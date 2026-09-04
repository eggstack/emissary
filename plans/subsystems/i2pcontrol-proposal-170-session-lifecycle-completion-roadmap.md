# Proposal 170 Session-Lifecycle Completion Roadmap

Status: **active / partial; M135/M136/M137 closed as complete, NewDest future**

Parent roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Current handoff:

- NewDest successor (historical M134 rebase or corrective M138), gated on the M137 §12 consumer contract. No NewDest execution is authorized now.

Closed correctives:

- M135 `plans/implementation/i2pcontrol-proposal-170/135-neutral-live-tunnel-quantity-and-leaseset-reconfiguration-primitive.md` — **closed as complete** (`plans/closure/i2pcontrol-proposal-170/135-closure.md`), zero promotions, matrix `284/88/468` at closure;
- M136 `plans/implementation/i2pcontrol-proposal-170/136-m132-corrective-sam-idle-reduction-and-proposal-reduce-completion.md` — **closed as complete** (`plans/closure/i2pcontrol-proposal-170/136-closure.md`), 21 promotions, matrix `305/67/468`;
- M137 `plans/implementation/i2pcontrol-proposal-170/137-m133-corrective-sam-idle-close-and-reasoned-termination.md` — **closed as complete** (`plans/closure/i2pcontrol-proposal-170/137-closure.md`), 14 promotions, matrix `319/53/468`.

Deferred corrective successors:

- M134 `134-newdest-on-proven-idle-resume.md` — historical deferred plan; must be explicitly rebased/amended against the M137 closure contract or superseded by a corrective M138.

Failed/blocked predecessor evidence:

- M132 closed as blocked at `plans/closure/i2pcontrol-proposal-170/132-closure.md`;
- M133 closed as blocked at `plans/closure/i2pcontrol-proposal-170/133-closure.md`.

Planning baseline:

- M137 closure matrix `319 apply / 53 blocked_primitive / 468 not_applicable`;
- M130 remains implemented-subset runtime/security authority;
- M131 remains residual applicability/primitive authority.

Pinned Proposal authority: I2P Proposal 170 revision `2026-05-20`, status Open.

Pinned read-only Java reference snapshot for the corrective lifecycle line:

- `i2p/i2p.i2p@2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`.

Architecture/security authority:

- canonical plans 000-003;
- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- M110/M116 shared-session/destination ownership;
- M118/M119 neutral tunnel-pool variance/backup behavior;
- M121 semantic truthfulness correction;
- M123 cancellation/commit atomicity;
- M130/M131 closure authorities.

External specifications/reference implementations are read-only evidence. Repository writes remain internal to `eggstack/emissary` unless separately authorized.

## 1. Purpose

Resolve the Proposal-170 client idle-lifecycle residuals through the smallest exact neutral lower-layer mechanics while keeping Proposal-specific validation, support claims and destination/key policy under `emissary-cli/src/i2pcontrol/**` wherever possible.

The original M132/M133 attempt was correctly stopped rather than approximated, but its decomposition combined too many independently risky mechanics into one gate. The corrective line separates those mechanics so each lower-layer primitive can be qualified before Proposal support depends on it.

## 2. What the M132/M133 closures taught us

M132 attempted all of the following together:

- reference freeze for quantity reduction;
- live tunnel-pool reconfiguration;
- LeaseSet convergence;
- SAM application-message activity ownership;
- idle timer/reduction policy;
- I2PControl `Reduce*` translation;
- matrix promotion.

It closed blocked because the execution did not obtain direct reference evidence for excess-tunnel behavior, LeaseSet behavior and Streamr applicability, and because truthful live-target/LeaseSet behavior appeared broader than the plan's path budget.

M133 then necessarily closed blocked because its hard dependency — the M132 activity/timer owner — did not exist.

The closures remain valid historical evidence. They are not overwritten or reopened.

## 3. Corrective reference findings

Direct read-only Java source now resolves the core M132 unknowns.

### 3.1 Runtime quantity reconfiguration

`I2CPMessageProducer.updateTunnels()` emits a `ReconfigureSessionMessage`; router `ClientMessageEventListener.handleReconfigureSession()` updates inbound/outbound client pool settings through `TunnelManagerFacade`.

### 3.2 Excess live tunnels

`TunnelPool.setSettings()` replaces current desired pool settings and wakes maintenance. Existing live tunnels remain in the pool/selection set; the build algorithm uses the new quantity to stop replacing/building above the desired target. Quantity reduction therefore converges through normal expiration/failure rather than synchronous purge.

### 3.3 LeaseSet desired count

Java `TunnelPool.locked_buildNewLeaseSet()` derives the wanted lease count from the current inbound pool quantity. This establishes a small dynamic desired-count responsibility in Emissary's existing `LeaseSetManager`; it does not require encrypted-LeaseSet or NetDb protocol redesign.

### 3.4 Session idle semantics

Java `SessionIdleTimer` provides:

- minimum idle time 5 minutes;
- default reduce time 20 minutes;
- default close time 30 minutes;
- reduced quantity default 1;
- close evaluated before reduction;
- reduction suppressed when close time <= reduce time;
- reduction via quantity reconfiguration;
- close via `destroySession()`.

`I2PSessionImpl.updateActivity()` restores original quantities after reduction.

### 3.5 Shared/subsession aggregation

Java `SubSession` delegates activity, last-activity and reduced state to the primary session.

### 3.6 Streamr/datagram applicability

Java Streamr client uses `I2PTunnelUDPClientBase`, which creates a normal generic `I2PSession` from the tunnel's client options. The generic `I2PSessionImpl` idle monitor consumes `i2cp.reduce*`/`close*`. Streamr is therefore governed by the same session-level idle policy, subject to Emissary end-to-end proof before matrix promotion.

## 4. Corrective decomposition

The line is now:

```text
M131 residual primitive re-freeze                         [CLOSED AS BLOCKED — 284/88/468]
  |
  +--> M132 combined reduction vertical slice             [CLOSED AS BLOCKED]
  |      |
  |      x
  |    M133 combined close vertical slice                 [CLOSED AS BLOCKED]
  |
  v
M135 neutral live quantity + LeaseSet desired-count       [CLOSED AS COMPLETE — ZERO PROMOTIONS]
  |
  v
M136 M132 corrective: activity + Reduce*                  [CLOSED AS COMPLETE — 305/67/468]
  |
  v
M137 M133 corrective: Close* + reasoned termination       [CLOSED AS COMPLETE — 319/53/468]
  |
  v
M134 amended/rebased OR M138 corrective NewDest           [FUTURE — NOT REGISTERED]
```

No active implementation handoff; NewDest registers only against the M137 §12 contract.

## 5. M135 — lower-layer primitive only

M135 exists specifically to remove the circularity that blocked M132.

It owns only:

- dynamic desired inbound/outbound quantities in the existing client tunnel pool;
- bounded destination-scoped target control;
- quantity-sensitive build/standby decisions;
- dynamic desired inbound count in `LeaseSetManager`;
- destination-level coordination between pool target and LeaseSet target.

It does **not** touch SAM activity, I2PControl or the support matrix.

Starting and required closing matrix is exactly `284 / 88 / 468`.

M135 must preserve the direct reference lifecycle shape:

- base configuration remains immutable;
- lowering desired quantity does not synchronously kill valid excess tunnels;
- excess tunnels remain usable until normal expiry/failure;
- no new replacement is built above the desired target;
- restoring the target resumes normal build deficit;
- LeaseSet desired count tracks current inbound target without fabricated leases.

M135 is the only current lower-core authorization.

## 6. M136 — reduction corrective (closed)

M136 closed as complete (`136-closure.md`, 21 promotions, `305/67/468`). It
added one generation-local SAM session activity/timer owner and consumed
standard `i2cp.reduce*`; I2PControl mapped Proposal `Reduce*` through
Yosemite's generic path.

## 7. M137 — close corrective (closed)

M137 closed as complete (`137-closure.md`, 14 promotions, `319/53/468`). It
extended the same owner with standard `i2cp.close*`, exact
close-before-reduce/suppression ordering, canonical session teardown, and one
neutral authoritative in-process termination cause; I2PControl mapped Proposal
`Close`/`CloseTime`.

M137 did not implement `NewDest`.

## 8. NewDest successor rule

Historical M134 remains useful design material but its dependency statement references the failed M133 line.

After successful M137 closure:

- if M134's assumptions exactly match the proven M137 reason/reopen contract, amend/rebase M134 and register it;
- otherwise create M138 as a corrective NewDest plan.

No NewDest plan may execute before an authoritative generation-local `IdlePolicy` close fact exists.

Destination/key staging/commit/rollback remains I2PControl-owned. Manual stop/start, explicit restart, process restart and non-idle failure must never imply rotation.

## 9. Ownership boundary

### Neutral core

Core may own only generic mechanics:

- live client-pool desired quantities;
- dynamic LeaseSet desired inbound count;
- destination coordination of those two owners;
- session application activity;
- standard I2CP idle policy;
- canonical session teardown cause.

Core source must contain no Proposal field names, JSON-RPC policy or TunnelManager administrative concepts.

### I2PControl

I2PControl owns:

- Proposal field validation/presence semantics;
- mapping to standard I2CP/SAM options;
- shared-definition compatibility policy;
- matrix/support claims;
- all NewDest identity/key policy.

### Yosemite

Yosemite remains the sole accepted SAM client. Exact Y005 stays pinned behind `yosemite-i2pcontrol`. M135-M137 authorize no Yosemite source/dependency change; generic additional-session-option serialization is the intended transport in M136/M137.

## 10. Cross-cutting lifecycle invariants

- no local TCP handler-count idle heuristic;
- one activity clock per underlying session generation;
- primary/shared-member activity aggregates at that owner;
- timer/control/reason state is generation-local and not persisted;
- stale generation work cannot affect replacements;
- base tunnel configuration remains distinct from live desired target;
- backup quantity remains separate standby capacity;
- no immediate excess-tunnel purge merely to satisfy a lower desired count;
- no LeaseSet fabricated from nonexistent tunnels;
- no lock spans network/build/join/timer/filesystem I/O;
- all state/queues/tasks are bounded;
- unsupported Proposal input fails before allocation/effect;
- no raw parallel SAM implementation;
- no secret/key material in logs/events/planning evidence.

## 11. Dependency readiness

### M135 exit gate for M136

M136 may register only if M135 closure proves:

- real desired target update and restore;
- reference-compatible excess-tunnel convergence;
- correct dynamic LeaseSet desired count;
- destination isolation;
- bounded generation-local control;
- unchanged matrix;
- no unresolved high/medium primitive defect.

### M136 exit gate for M137

M137 may register only if M136 closure proves:

- one canonical application-message activity owner;
- monotonic generation-local timer;
- shared-member aggregation;
- real reduction/restore through M135;
- deterministic shutdown/restart behavior;
- stable standard option parsing;
- no unresolved high/medium lifecycle defect.

### M137 exit gate for NewDest

A future NewDest plan may register only if M137 closure proves:

- canonical real session teardown;
- authoritative generation-local termination cause;
- `IdlePolicy` distinguishable from manual/failure causes;
- stable reopen/new-generation boundary;
- no persisted/replayed idle reason.

## 12. Failure, cancellation, restart and contention

Across the line:

- owner shutdown invalidates pending timers/controls;
- control overload is finite and deterministic;
- failed target/reduction/restore operations cannot be reported as successful state transitions;
- canonical session teardown is not duplicated by idle policy;
- observation publication failure never blocks authoritative teardown;
- process restart starts fresh base target/activity/reason state;
- shared-session creation/release remains governed by M116/M123 cancellation rules.

## 13. Compatibility

Without the new standard idle options and without invoking the M135 runtime target seam, behavior remains equivalent to the M130-qualified implemented subset.

No durable migration, public API version, Proposal method/action/tunnel-type change, Cargo dependency change or Yosemite pin change is authorized by this roadmap.

## 14. Verification policy

Each milestone runs its focused deterministic tests plus applicable:

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Known repository-wide stable/nightly rustfmt drift is recorded rather than normalized through unrelated churn.

## 15. Deferred residual clusters

This roadmap does not authorize:

- `Profile`;
- presentation `UseSSL`;
- HTTP `SSLProxies`/`JumpList`;
- `UseOutproxyPlugin`;
- `UniqueLocalAddressPerClient`;
- `MultiHoming`/`shouldBundleReplyInfo`;
- `SigType`;
- encrypted/authenticated LeaseSets;
- unrelated Streamr residuals such as `ConnectDelay`.

These remain separate M131 primitive clusters.

## 16. Completion rule

This focused roadmap completes only when the lifecycle cells are either operational with evidence or truthfully retained blocked after the corrective chain.

M135 alone cannot change Proposal support. M136/M137 may promote their own cells only after their prerequisites close. Whole-surface Proposal 170 completion remains governed by the parent roadmap and a future final requalification.