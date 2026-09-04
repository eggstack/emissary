# Proposal 170 Session-Lifecycle Completion Roadmap

Status: **active / partial; M132 ready / registered; M133-M134 dependency-blocked**

Parent roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Current handoff:

- `plans/implementation/i2pcontrol-proposal-170/132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md` — **ready / registered**.

Future handoffs:

- M133 `133-neutral-sam-idle-close-and-reasoned-termination.md` — deferred, hard-depends M132;
- M134 `134-newdest-on-proven-idle-resume.md` — deferred, hard-depends M133.

Planning baseline:

- M131 closure head `3a829d7d3d6314ecf09e42dbf0339506f0917c96`;
- M131 matrix `284 apply / 88 blocked_primitive / 468 not_applicable`;
- M130 remains current implemented-subset runtime/security qualification authority.

Pinned Proposal authority: I2P Proposal 170 revision `2026-05-20`, status Open.

Architecture/security authority:

- canonical plans 000-003;
- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- M110/M116 shared-session and destination ownership;
- M121 semantic-truthfulness correction;
- M123 cancellation/commit atomicity;
- M131 residual primitive map/closure.

External specifications/reference implementations are read-only evidence. Repository writes remain internal to `eggstack/emissary`.

## 1. Purpose and scope

Resolve the Proposal-170 client idle-lifecycle residuals through the smallest exact neutral lower-layer mechanics, while retaining Proposal policy and destination/key decisions under `emissary-cli/src/i2pcontrol/**`.

The line is deliberately split:

1. **M132** implements generic session activity and dynamic active tunnel-pool quantity, then consumes that capability for `Reduce`, `ReduceCount`, and `ReduceTime`.
2. **M133** reuses the M132 activity/timer owner for generic close-on-idle and adds a neutral authoritative termination reason, then consumes it for `Close` and `CloseTime`.
3. **M134** keeps `NewDest` entirely I2PControl-owned and rotates only on a proven idle-close/resume transaction.

This split prevents neutral core mechanics, session termination and key mutation from being combined into one oversized milestone.

## 2. Current-state evidence

M131 established that local TCP handler count is not the reference idle predicate. Reference I2CP behavior instead tracks activity at the client session payload boundary, reduces both tunnel quantities after an idle threshold, restores configured quantities on subsequent activity, and destroys the session on close-on-idle.

Current Emissary owners are sufficient for M132 without a new subsystem:

- `emissary-core/src/sam/session.rs` owns active SAM sessions and streaming/datagram payload send/receive;
- `emissary-core/src/destination/mod.rs` owns the client destination and `TunnelPoolHandle`;
- `emissary-core/src/tunnel/pool/**` owns active/standby pool population;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` owns Proposal-to-Yosemite session option translation;
- Yosemite Y005 already provides validated additional SAM/I2CP option serialization.

Therefore M132 is dependency-ready. M133/M134 are not registered because their required stable interfaces must be proved by preceding closures.

## 3. Ownership boundary

### Neutral core

Core may own only generic mechanics:

- session activity state;
- standard I2CP/SAM idle option consumption;
- generation-local monotonic idle state/timers;
- live active inbound/outbound quantity targets;
- restoration to configured targets;
- generic close-on-idle and a neutral termination reason in M133.

No core API/type may contain Proposal field names, JSON-RPC concepts, TunnelManager policy or I2PControl key lifecycle.

### I2PControl

I2PControl owns:

- Proposal field validation/presence semantics;
- mapping to standard I2CP/SAM options;
- shared-definition compatibility policy;
- matrix/support claims;
- all `NewDest` key staging/commit/rollback and persistence conflicts.

### Yosemite

Yosemite remains the sole accepted SAM client. No raw parallel SAM encoder is authorized. Y005 remains exact-pinned behind `yosemite-i2pcontrol`; M132-M134 authorize no Yosemite change.

## 4. Frozen lifecycle semantics

The line must preserve the reference contract:

- `i2cp.reduceOnIdle` enables reduction;
- default reduce time 20 minutes; reference minimum 5 minutes;
- reduced quantity default 1;
- reduction reconfigures inbound and outbound quantity;
- later session activity restores configured quantities;
- `i2cp.closeOnIdle` enables idle close;
- default close time 30 minutes; reference minimum 5 minutes;
- if close time is <= reduce time while both are enabled, reduction is suppressed;
- close-on-idle destroys the session;
- `newDestOnResume` is higher-level identity policy and qualifies only after an actual idle close/reopen cycle.

Each milestone rechecks the exact pinned reference before production edits. Ambiguity is a stop condition.

## 5. Activity contract

The neutral activity clock follows the I2CP application-protocol payload boundary, not local listener occupancy.

It includes:

- outbound streaming packets that become I2CP payload messages, including streaming control/retransmit packets;
- accepted outbound datagram payloads, including payloads queued pending LeaseSet lookup;
- inbound successfully decoded I2CP streaming/datagram payloads delivered into the session protocol manager.

It excludes naming lookups, SAM PING/PONG/control traffic, tunnel maintenance/build traffic, unrelated NetDb work and local handler count.

Primary/subsession activity aggregates at the owning primary session. I2PControl definitions sharing one Yosemite session therefore share one activity clock and idle policy.

## 6. Target architecture

### Session state

`SamSession` owns one generation-local idle state machine. M132 adds reduction; M133 extends that same owner with close. No second timer subsystem.

### Pool live target

`TunnelPoolConfig` remains immutable configured/base authority. `TunnelPool` gains a separate live active inbound/outbound target. The pool alone owns build deficit, retirement/expiry, selection and standby promotion.

Control delivery must be bounded and latest-state-safe. Prefer a coalescing/watch-style desired target so activity-triggered restoration cannot be lost behind stale reduction commands.

Backup quantity remains separate standby capacity.

### LeaseSet correctness

Inbound quantity changes must remain truthful to the LeaseSet owner. Removed/nonusable tunnels cannot remain advertised indefinitely; restoration cannot publish leases before tunnels exist. If exact behavior requires a broad LeaseSet redesign, stop rather than approximate.

### Reasoned close

M133 may extend the existing passive SAM lifecycle observation seam or equivalent narrow in-process result with a neutral reason. It must not invent a SAM wire extension. Unknown remains unknown.

### NewDest transaction

M134 consumes only a proven M133 idle-close fact. Successor identity staging/commit/discard remains under the existing I2PControl secret owner and commit transaction. Manual stop/start, restart, process restart and non-idle failure never imply rotation.

## 7. Dependency graph

```text
M131 residual primitive re-freeze                 [CLOSED AS BLOCKED]
  |
  v
M132 SAM idle reduction + dynamic pool target     [READY / REGISTERED]
  |
  v
M133 SAM idle close + reasoned termination        [DEFERRED / UNREGISTERED]
  |
  v
M134 NewDest on proven idle resume                [DEFERRED / UNREGISTERED]
```

Only M132 is authorized for execution.

## 8. M132 exit conditions

M132 closes only when:

- the exact activity predicate is implemented at real SAM/I2CP payload boundaries;
- reduction changes real active inbound/outbound pool targets;
- configured targets restore on qualifying activity;
- backup capacity and unrelated pools remain unchanged;
- LeaseSet behavior remains truthful;
- shared-session activity/policy is exact;
- malformed Proposal inputs fail before allocation;
- matrix promotions have end-to-end evidence;
- M061/M062 exact-path containment is green;
- closure explicitly decides M133 readiness.

## 9. M133 exit conditions

M133 closes only when:

- it reuses M132 activity/timer state;
- idle close performs bounded authoritative session/pool teardown;
- manual/failure teardown is not mislabeled idle;
- a neutral in-process termination reason is stable for M134;
- Close/CloseTime promotions have exact evidence;
- closure explicitly decides M134 readiness.

## 10. M134 exit conditions

M134 closes only when:

- exactly one fresh identity is committed per successful qualifying idle resume;
- failed/cancelled resume rolls back staged successor state;
- manual stop/start, explicit restart, process restart and unrelated failure preserve identity;
- persistent/import/shared conflicts are exact and fail before allocation;
- the six non-Streamr NewDest cells have end-to-end evidence;
- Streamr NewDest remains not applicable under M131 authority.

## 11. Failure, cancellation, restart and contention

Across the line:

- timers/control/reasons are generation-local;
- state/queues are bounded with explicit overload behavior;
- no lock spans network I/O, timer waits, joins or filesystem synchronization;
- pool/session shutdown is idempotent and leaves no orphan task/control state;
- stale generation controls/reasons cannot affect replacement generations;
- shared-session creation/release remains cancellation-safe under M116/M123;
- process restart never persists or reconstructs an old idle timer/reason;
- key material remains redacted and confined.

## 12. Compatibility and migration

No public API version, method, action or tunnel type change is required. Core behavior is unchanged when standard idle options are absent.

Residual Proposal fields may continue using the existing raw-definition round-trip representation unless a typed I2PControl-local field is needed for exact validation. Any such addition must deserialize old state without migration or provide an explicit bounded migration plan.

No Yosemite/Cargo dependency change is authorized.

## 13. Security requirements

The line must not:

- expose general router/pool control;
- permit one destination to modify another's pool;
- affect exploratory/participating pools;
- alter tunnel cryptography, hop selection or peer profiling;
- weaken proxy/DNS/local-target confinement;
- expose secrets in events/logs/Debug/RPC;
- use passive observation failure as a reason to block authoritative teardown;
- silently downgrade unsupported idle/key semantics.

## 14. Verification policy

Each milestone runs focused deterministic tests plus:

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

Known repository-wide stable/nightly rustfmt drift is recorded, not normalized through unrelated churn.

## 15. Deferred work

This roadmap does not authorize:

- unresolved Streamr `ConnectDelay`;
- `Profile`;
- presentation `UseSSL`;
- HTTP `SSLProxies`/`JumpList`;
- outproxy provider/plugin integration;
- `UniqueLocalAddressPerClient`;
- `MultiHoming`/`shouldBundleReplyInfo`;
- `SigType`;
- encrypted/authenticated LeaseSets.

Those remain separate M131 primitive clusters.

## 16. Completion rule

This roadmap closes after M134 closure or a truthful blocked disposition for remaining lifecycle cells. It does not itself establish full Proposal 170 support. Whole-surface completion remains governed by the parent roadmap and future requalification after all applicable residual clusters are resolved.