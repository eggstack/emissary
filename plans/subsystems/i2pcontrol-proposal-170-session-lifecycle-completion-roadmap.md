# Proposal 170 Session-Lifecycle Completion Roadmap

Status: **active / partial; M132 ready for registration, M133-M134 dependency-blocked**

Parent roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Planning baseline:

- M131 closure head `3a829d7d3d6314ecf09e42dbf0339506f0917c96`;
- M131 support authority `284 apply / 88 blocked_primitive / 468 not_applicable`;
- M130 remains the implemented-subset runtime/security qualification authority.

Pinned Proposal authority:

- I2P Proposal 170, revision `2026-05-20`, status Open.

Architecture/security authority:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/003-planning-process.md`;
- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- M110/M116 shared-session and destination ownership;
- M121 semantic-truthfulness correction;
- M123 commit/cancellation atomicity;
- M131 residual primitive map and closure.

All external specifications and reference implementations are read-only evidence. All writes authorized by this roadmap remain internal to `eggstack/emissary`. No upstream issue, pull request, review, contact, release, submission, or adoption activity is authorized.

## 1. Purpose

Complete the Proposal-170 client idle-lifecycle residuals through the smallest neutral lower-layer capability that matches I2P session semantics, while keeping Proposal policy and identity decisions inside `emissary-cli/src/i2pcontrol/**`.

The line is intentionally split into three milestones:

1. **M132** — generic SAM/I2P-session activity plus dynamic tunnel-pool quantity control, consumed by Proposal `Reduce`, `ReduceCount`, and `ReduceTime`;
2. **M133** — generic close-on-idle plus a reasoned session termination fact, consumed by Proposal `Close` and `CloseTime`;
3. **M134** — I2PControl-owned `NewDest` rotation exactly on resume after a proven idle close.

The split prevents destination/key mutation from being mixed into the neutral tunnel-pool primitive and gives each milestone an independently testable contract.

## 2. Current state and why this line is ready

M131 established that local accepted TCP handler count is not the correct idle predicate. Direct reference evidence instead places idle reduction/close at the I2P client-session boundary:

- Java `SessionIdleTimer` observes `I2PSessionImpl.lastActivity()`;
- outbound I2CP message send calls update activity;
- inbound delivered payload calls update activity;
- subsessions delegate activity to their primary session;
- reduction reconfigures both inbound and outbound tunnel quantity;
- activity after reduction restores the original configured quantity;
- close-on-idle destroys the session;
- `newDestOnResume` is a higher application/I2PTunnel identity policy layered on that close/resume lifecycle.

Current Emissary already has the matching canonical owners:

- `emissary-core/src/sam/session.rs` owns the active SAM session and the actual streaming/datagram I2CP payload send/receive boundary;
- `emissary-core/src/destination/mod.rs` owns the session destination and its `TunnelPoolHandle`;
- `emissary-core/src/tunnel/pool/**` owns active/standby tunnel population and maintenance;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` is the existing Proposal-to-Yosemite session-option boundary;
- Yosemite Y005 can carry validated additional SAM/I2CP session options without a fork change.

Therefore no new subsystem, raw SAM stack, router-global policy framework, or Yosemite change is required to begin M132.

## 3. Ownership boundary

### 3.1 Neutral lower-layer owner

Core may own only generic session/pool mechanics:

- monotonic session activity;
- generic idle-reduction configuration consumed from standard I2CP/SAM option names;
- dynamic active inbound/outbound tunnel quantity target;
- restoration to the configured target;
- generic idle session termination and a neutral termination reason in M133.

Core APIs and state MUST NOT contain `TunnelManager`, Proposal-170 field names such as `ReduceCount`, or I2PControl administrative concepts.

### 3.2 I2PControl owner

I2PControl continues to own:

- Proposal field validation and presence semantics;
- mapping `Reduce`/`ReduceCount`/`ReduceTime` and later `Close`/`CloseTime` to standard I2CP/SAM options;
- fail-before-allocation behavior for unsupported/invalid combinations;
- support-matrix promotion only after end-to-end evidence;
- persistent `NewDest` policy, successor-key creation, rollback and restart semantics;
- shared-definition membership/compatibility policy.

### 3.3 Yosemite boundary

Yosemite remains the sole accepted SAM client. M132-M134 MUST NOT add a parallel SAM encoder. The exact Y005 alias remains unchanged unless a later separately registered Yosemite plan proves a missing generic serialization capability. Current evidence shows `SessionOptions::additional_options` is sufficient for the standard idle options.

## 4. Canonical semantic contract

Reference contract frozen for this line:

- `i2cp.reduceOnIdle` enables idle reduction;
- `i2cp.reduceIdleTime` is milliseconds; Java default 20 minutes and minimum 5 minutes;
- `i2cp.reduceQuantity` defaults to 1 and is at least 1 in the Java reference;
- reduction applies to both inbound and outbound active tunnel quantity;
- configured base quantities remain the restore target;
- subsequent session activity restores the original configured quantities;
- `i2cp.closeOnIdle` enables idle close;
- `i2cp.closeIdleTime` is milliseconds; Java default 30 minutes and minimum 5 minutes;
- when both are enabled and close time is less than or equal to reduce time, the Java reference suppresses reduction;
- close-on-idle destroys the session;
- `newDestOnResume` is only meaningful after a close-on-idle/reopen cycle and must not rotate identity on ordinary manual stop/start or unrelated failure.

Each implementation milestone MUST re-check the exact pinned reference snapshot before production edits. A reference disagreement is a stop condition, not permission to approximate.

## 5. Activity semantics

The neutral activity predicate is the SAM/I2P application-protocol payload boundary, not local listener occupancy.

M132 must count the events equivalent to reference I2CP send/deliver activity:

- outbound streaming protocol packets emitted through the SAM session, including control/retransmit packets that become I2CP payload messages;
- accepted outbound datagram payload requests, including a payload queued while a LeaseSet lookup is pending;
- inbound successfully decoded I2CP streaming/datagram payloads delivered into the session protocol manager.

The following do not by themselves reset the idle clock:

- naming lookups;
- SAM PING/PONG/control commands;
- tunnel maintenance/build traffic;
- NetDb lookups unrelated to an application payload;
- local TCP listener existence or handler count.

Primary/subsession activity must aggregate at the owning primary session as in the reference. A shared Yosemite session naturally aggregates activity for all I2PControl members using that one session.

## 6. Target architecture

### 6.1 Session idle state

`SamSession` owns generation-local idle state and runtime timers. No global timer registry is introduced.

M132 adds only reduction state. M133 extends the same state machine with close-on-idle; it must not create an independent competing timer.

### 6.2 Dynamic tunnel-pool target

`TunnelPoolConfig` remains the immutable configured/base target. The live pool maintains a separate actor-local active quantity target for inbound and outbound tunnels.

A neutral control path may change only the live active target and restore it to the configured target. Backup quantity remains separately configured standby capacity.

The control path must be bounded and latest-state-safe: a restoration request after activity must not be silently lost behind a full queue. Prefer a coalescing/watch-style target or another bounded actor-local mechanism over an unbounded command queue. If implementation chooses a finite command channel, the plan must prove deterministic saturation/retry semantics and that restore cannot be lost.

The pool remains the only owner that decides how excess active tunnels retire and how capacity is replenished. M132 must freeze the reference router behavior before choosing immediate retirement versus natural expiry/no-rebuild.

### 6.3 LeaseSet correctness

Inbound quantity changes must remain truthful to the LeaseSet owner. A reduction must not leave permanently advertised leases for tunnels the pool has removed, and restoration must not publish nonexistent leases. Existing LeaseSet creation/update behavior is reused where possible.

If exact reduced-pool LeaseSet maintenance requires a broad LeaseSet redesign outside the named destination/pool owner, M132 stops and closes the affected slice blocked.

### 6.4 Reasoned idle termination

M133 may extend existing neutral SAM lifecycle observation with a bounded termination reason, or add an equivalently narrow in-process fact owned by the SAM session. It MUST NOT invent a SAM wire extension merely so I2PControl can distinguish idle close.

The reason must distinguish at least:

- idle-policy close;
- ordinary control/manual termination;
- transport/session failure when the owner can identify it without inference.

Unknown remains unknown; no reason is fabricated.

### 6.5 New destination on resume

M134 consumes only a proven idle-close fact. Identity rotation stays in the existing I2PControl destination/key owner and shared-session registry.

A new identity is staged/committed exactly once for the successful resume generation. Failed/cancelled resume does not consume or publish a successor identity. Manual stop/start and non-idle failure do not rotate.

## 7. Milestone sequence

```text
M131 residual primitive re-freeze                 [CLOSED AS BLOCKED]
  |
  v
M132 SAM idle reduction + dynamic pool target     [READY / NEXT HANDOFF]
  |
  v
M133 SAM idle close + reasoned termination        [DEFERRED; HARD-DEPENDS M132]
  |
  v
M134 NewDest on proven idle-resume                [DEFERRED; HARD-DEPENDS M133]
```

Only M132 is dependency-ready and should be registered now.

## 8. M132 exit

M132 closes only when:

- the exact activity predicate is implemented at the real SAM/I2CP payload boundary;
- idle reduction changes real active inbound/outbound pool targets;
- configured quantities restore on subsequent activity;
- backup capacity and unrelated pools remain unchanged;
- shared-session activity is aggregated correctly;
- invalid Proposal values fail before allocation;
- end-to-end tests prove the runtime effect rather than only SAM serialization;
- M061/M062 path containment is amended and green;
- the matrix is mechanically updated only for cells with proven reference applicability;
- M133 readiness is re-audited from the actual M132 closure head.

## 9. M133 exit

M133 closes only when:

- close-on-idle uses the M132 activity clock/state machine;
- idle close tears down the correct SAM session and pool with bounded cleanup;
- manual stop/network/session failure cannot be mislabeled as idle close;
- a neutral reason is available to an in-process consumer without a new wire extension;
- `Close`/`CloseTime` cells are promoted only where exact applicability is proven;
- M134 can consume a stable idle-close/reopen contract.

## 10. M134 exit

M134 closes only when:

- `NewDest` rotates only on successful resume after a proven idle close;
- exactly one successor identity is committed per qualifying resume generation;
- failed/cancelled resumes roll back staged identity state;
- manual stop/start and unrelated failure preserve identity;
- persistent-key and shared-session interactions are explicit and fail-closed;
- the six non-Streamr `NewDest` cells are promoted only with end-to-end evidence; Streamr remains not applicable under M131 authority.

## 11. Failure, cancellation, restart and contention

All three milestones preserve:

- one owner for each mutable state machine;
- no lock across network I/O, timer waits, joins or filesystem synchronization;
- generation-local timers/control state;
- bounded queues/state with explicit overload behavior;
- no orphan pool/session task after stop/restart;
- edit/start/restart transactionality and last-known-good behavior;
- creator-cancellation safety from M116/M123;
- shared-session member bounds and compatibility equality;
- no destination secret in logs, events, Debug or matrix fixtures.

Process restart does not resume an old idle timer. A restarted tunnel starts a new session generation from persisted configuration. `NewDest` rotation is never inferred from process restart.

## 12. Compatibility and migration

No public API version, tunnel type, action, or storage format change is required by the roadmap itself.

M132/M133 may consume residual raw Proposal configuration through the existing I2PControl parser/definition model rather than adding public persistence fields solely for implementation convenience. If a typed field is needed to preserve validation/round-trip semantics, it must remain I2PControl-local and backwards-compatible with existing stored definitions.

Core behavior is unchanged when idle options are absent. Non-I2PControl SAM clients gain only standard generic I2CP idle semantics when they explicitly supply those options.

## 13. Security and anonymity

This line MUST NOT:

- weaken proxy routing, DNS, local-target or loopback confinement;
- expose mutable `TunnelPoolHandle` internals outside the neutral owner;
- let an unauthenticated external caller control another session's pool;
- add router-global administrative APIs;
- change tunnel cryptography, hop selection, peer profiling or build protocol;
- log session keys, private destinations or raw secret-bearing options;
- silently fall back when idle policy cannot be honored.

Reducing tunnel count is an operator-selected privacy/performance tradeoff already represented by the pinned contract. The implementation must never reduce an unrelated destination or exploratory/participating pool.

## 14. Verification policy

Each milestone adds focused deterministic tests and runs, at minimum:

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Known repository-wide stable/nightly rustfmt drift is recorded rather than normalized through unrelated churn.

## 15. Deferred work

This roadmap does not authorize or solve:

- Streamr `ConnectDelay` if still semantically ambiguous after this line;
- `Profile` / streaming max-window;
- presentation `UseSSL`;
- HTTP `SSLProxies`/`JumpList`;
- outproxy plugin/provider integration;
- `UniqueLocalAddressPerClient`;
- `MultiHoming`/`shouldBundleReplyInfo`;
- `SigType`;
- encrypted/authenticated LeaseSets.

Those remain separate M131 primitive clusters.

## 16. Final rule

This roadmap is complete only after M134 closure or a truthful blocked disposition for any remaining cells in this lifecycle cluster. It does not by itself make Proposal 170 complete. Full support remains governed by the parent full-support roadmap and a future whole-surface requalification after all applicable residuals are resolved.