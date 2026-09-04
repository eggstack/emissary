# M137 — M133 Corrective: SAM Idle Close and Reasoned Termination

Status: **deferred / unregistered; hard-depends on successful M136 closure**

Class: corrective implementation / neutral lifecycle close + I2PControl consumer

Corrects:

- M133 `133-neutral-sam-idle-close-and-reasoned-termination.md`;
- M133 closure `plans/closure/i2pcontrol-proposal-170/133-closure.md`.

Planning baseline:

- current pre-M135/M136 matrix authority `284 apply / 88 blocked_primitive / 468 not_applicable`;
- M135 must establish neutral live quantity/LeaseSet reconfiguration;
- M136 must establish the canonical generation-local SAM activity/timer owner and close successfully before M137 registration.

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
- successful M135 and M136 closures.

External repositories remain read-only. Writes are limited to `eggstack/emissary`.

## 1. Objective

Extend the **same** session activity/timer owner proven by M136 with exact standard I2CP close-on-idle semantics, then consume that capability for Proposal:

- `Close`;
- `CloseTime`.

M137 must also expose one neutral authoritative in-process termination reason sufficient for a later `NewDest` implementation to distinguish:

- idle-policy close;
- explicit/manual/requested close;
- failure/transport shutdown where authoritative;
- unknown when the winning cause cannot be proven.

M137 does **not** rotate destination keys and does not implement `NewDest`.

## 2. Registration/readiness gate

M137 MUST remain unregistered until M136 closure proves:

1. one canonical session activity timestamp/state machine;
2. one monotonic generation-local timer owner;
3. real reduction and restore through M135;
4. no local-TCP-handler heuristic;
5. shared-session activity aggregation;
6. deterministic shutdown/replacement-generation isolation;
7. stable standard I2CP option parsing;
8. no unresolved high/medium lifecycle correctness issue;
9. M136 closure explicitly marks M137 dependency-ready.

A blocked/partial M136 does not automatically unblock M137.

## 3. Direct reference freeze

### 3.1 Close defaults and bounds

Java `SessionIdleTimer` defines:

- default close time: 30 minutes / 1800000 ms;
- minimum close time: 5 minutes / 300000 ms.

At each timer firing it checks close before reduction.

Reference:

- `core/java/src/net/i2p/client/impl/SessionIdleTimer.java`.

### 3.2 Reduce/close interaction

When both policies are enabled and `closeIdleTime <= reduceIdleTime`, Java disables reduction for that session timer. Close remains enabled.

M137 must preserve this ordering using the M136 state owner rather than running two independent timers.

### 3.3 Close action

When idle age reaches the close threshold Java calls:

`session.destroySession()`

and returns without rescheduling the idle timer.

The close is therefore authoritative session teardown, not merely local listener closure or Yosemite-client object release.

### 3.4 Shared/subsession semantics

M136 must already have frozen primary/subsession activity aggregation. M137 uses the same owner. It must not create a per-front-end or per-shared-member close timer.

### 3.5 Streamr/datagram applicability

The same Java generic `I2PSessionImpl` idle monitor is used by Streamr's normal `I2PSession`; direct reference therefore supports session-level `Close`/`CloseTime` applicability to Streamr as well. Promotion still requires Emissary runtime evidence.

## 4. State-machine extension

M137 extends the M136 owner. Do not create a second idle scheduler.

Conceptually the session lifecycle becomes:

- `Active`;
- `Reduced` where reduction remains enabled and threshold reached;
- `Closing { cause }` terminal transition for the generation.

Required ordering per timer evaluation:

1. if session generation is already closing/closed, stop;
2. compute idle age from the same M136 last-activity value;
3. if close enabled and idle age >= close threshold, atomically win/record idle-policy close and enter canonical teardown;
4. otherwise evaluate/reuse M136 reduction behavior if reduction is enabled;
5. reschedule only if the generation remains active/reduced.

If close threshold is <= reduce threshold, reduction must be suppressed from initialization/frozen policy state rather than momentarily reducing immediately before close.

## 5. Canonical teardown contract

M137 must trigger the existing authoritative SAM session-generation teardown path. It must not duplicate destination/tunnel-pool shutdown logic inside the idle timer.

The implementation agent must identify and record the existing canonical removal path before edits, including:

- transition that stops accepting commands for the generation;
- stream/datagram manager shutdown/termination;
- destination/tunnel pool shutdown;
- SAM server/session map removal;
- passive observation publication;
- owner wake/removal behavior.

M137 may add a narrow internal trigger to request that existing teardown with an explicit neutral cause.

Do not:

- close only a local TCP listener;
- infer session destruction from Yosemite handle drop;
- create a second pool shutdown path;
- invent a SAM wire protocol extension.

## 6. Neutral termination reason

The reason must be recorded at the authoritative winning lifecycle transition, not inferred after the session disappears.

Acceptable vocabulary is generic, e.g.:

- `IdlePolicy`;
- `Requested`;
- `Failure`;
- `Unknown`.

Exact names may differ.

Requirements:

- reason is generation-local;
- idle-policy reason is recorded only if the idle transition actually wins the teardown race;
- manual stop/restart cannot be labeled idle;
- transport/network failure cannot be labeled idle;
- simultaneous causes have deterministic winner semantics or resolve to `Unknown` when the implementation genuinely cannot prove precedence;
- reason does not contain secrets, addresses or Proposal fields;
- observation publication failure never blocks authoritative teardown;
- the reason is available in-process to the I2PControl session owner for a later `NewDest` plan;
- no persistence across process restart.

Preferred seam is an extension of the existing neutral SAM observation/lifecycle result path rather than a new I2PControl callback inside core.

## 7. Standard option consumption

M137 consumes standard:

- `i2cp.closeOnIdle`;
- `i2cp.closeIdleTime`.

Use the same generic option owner/parsing style established by M136.

Reference behavior:

- absent/false closeOnIdle -> no idle close;
- default close time 1800000 ms;
- minimum close time 300000 ms;
- reduce suppression when close time <= reduce time and both enabled.

Malformed non-I2PControl SAM input must fail safely. I2PControl must validate Proposal-specific constraints before allocation.

## 8. I2PControl mapping

After the generic capability is proven, map:

- `Close` -> `i2cp.closeOnIdle`;
- `CloseTime` -> `i2cp.closeIdleTime` in the pinned Proposal/reference unit.

Use Yosemite's existing validated generic additional-session-option path. No Yosemite source change and no raw SAM command construction.

Presence semantics must be frozen against M095/M105 and Proposal 170 before code. `CloseTime` must not silently enable close unless the pinned contract says it does.

Unsupported/malformed values fail before allocation.

## 9. Shared-session behavior

Because idle close belongs to the underlying session generation:

- shared definitions must have compatible close/reduce policy to share;
- activity from any member resets the common idle clock;
- idle close tears down the shared session generation once;
- all members observe loss of the same underlying session;
- one member's manual release does not idle-close remaining members;
- final explicit member release is `Requested`, not `IdlePolicy`;
- creator cancellation cannot fabricate an idle reason.

Existing M116/M123 ownership/cancellation rules remain authoritative.

## 10. Authorized production path budget

### Neutral core

- `emissary-core/src/sam/session.rs` — extend M136 activity/timer state with close policy and canonical teardown request;
- `emissary-core/src/sam/mod.rs` — only if authoritative session removal/observation reason must be carried by the existing SAM server lifecycle seam;
- exact existing SAM lifecycle helper modules touched by the canonical teardown path only if required and explicitly recorded before edit;
- M135/M136 lower-layer files only for corrective defects discovered while consuming already-closed interfaces; semantic expansion requires amendment.

### I2PControl

- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` — `Close`/`CloseTime` validation/translation;
- existing I2PControl session/lifecycle owner files needed to consume the neutral termination reason;
- backend option allowlists only where fail-before-allocation requires known-field updates.

### Evidence

- focused tests;
- M061/M062 containment;
- M095/M105/M110 evidence;
- support docs;
- closure/registry/roadmaps.

No Cargo/lockfile/Yosemite/NetDb/crypto/frontend/transport/peer-selection change is authorized.

## 11. Ordered work packages

### WP1 — Rebase on M136

Freeze the exact activity/timer API and successful M136 lifecycle behavior. Amend M137 before code if the interface materially differs.

### WP2 — Canonical teardown map

Document the actual winning transition and removal sequence in current Emissary. Identify where a termination cause can be authoritatively attached without duplicate teardown.

### WP3 — Close policy state

Add close settings to the M136 state machine and implement close-before-reduce ordering/suppression.

### WP4 — Reasoned teardown

Add the neutral termination-cause fact at the authoritative transition. Cover idle/manual/failure races deterministically.

### WP5 — Generic runtime tests

Prove idle close tears down the real SAM session/destination/pool generation and stops rescheduling.

### WP6 — I2PControl mapping

Enable `Close`/`CloseTime` only after generic close works. Serialize via Yosemite generic standard options.

### WP7 — Shared/cancellation tests

Prove one common clock/reason/teardown under sharing and no mislabeling under manual stop/restart/failure/cancellation.

### WP8 — Matrix/docs/closure

Promote only cells with end-to-end evidence. Explicitly decide the successor for `NewDest`: either amend/rebase historical M134 against M137 closure or create a new M138 corrective. Do not register either automatically without the authoritative reason/reopen contract.

## 12. Required focused tests

At minimum:

1. no close option -> M136 behavior unchanged;
2. close enabled -> no teardown before threshold;
3. exact threshold -> canonical session teardown occurs;
4. close timer uses the same activity clock as reduction;
5. activity resets both effective close and reduce idle age;
6. close <= reduce suppresses reduction;
7. close > reduce allows reduce then later close if no activity;
8. reduced session activity restores and postpones close;
9. idle-close timer does not reschedule after teardown;
10. idle cause is emitted/recorded only when idle transition wins;
11. manual stop reason is not idle;
12. explicit restart reason is not idle;
13. network/SAM failure reason is not idle;
14. simultaneous idle/manual race has deterministic tested disposition;
15. stale generation idle timer cannot close replacement generation;
16. shared members use one close clock;
17. one member release does not close remaining shared members;
18. final explicit release is not idle;
19. Streamr/datagram close uses the same generic owner;
20. malformed `Close`/`CloseTime` fail before allocation;
21. Yosemite serialization uses exact standard keys and no raw injection;
22. every promoted client family has an end-to-end teardown trace;
23. M061/M062 containment rejects unauthorized expansion.

## 13. Failure/cancellation/restart

- Idle-close state is generation-local and not persisted.
- Process restart cannot reconstruct an old idle reason.
- Once canonical teardown wins, pending reduction/restore work becomes stale and harmless.
- Manual/restart/failure transitions can race with idle close only through one authoritative owner.
- No lock spans network, pool shutdown, join, timer or filesystem I/O.
- Observation/reason publication is passive; failure to publish does not prevent teardown.
- A replacement generation starts without inherited idle/reduced/closing state.

## 14. Security requirements

Closure must prove:

- no cross-destination teardown;
- no general router shutdown capability exposed;
- exploratory/participating pools remain unrelated;
- idle policy cannot weaken proxy/DNS/loopback confinement;
- no secret/key/path/address data leaks through termination reason;
- no user-controlled string becomes a core lifecycle reason;
- unsupported inputs fail before allocation;
- core contains no Proposal/I2PControl policy names.

## 15. Verification

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

Record known pre-existing stable/nightly rustfmt drift without unrelated normalization.

## 16. Matrix accounting

At M131 authority there are 14 blocked `Close`/`CloseTime` cells across the seven client families.

If M136 successfully promoted all 21 `Reduce*` cells, M137 would start at `305 apply / 67 blocked / 468 not_applicable`; otherwise closure must use the actual mechanically recomputed M136 matrix.

M137 may promote at most 14 cells:

- 7 `Close`;
- 7 `CloseTime`.

Direct reference supports Streamr applicability, but end-to-end Emissary evidence remains mandatory.

Do not use target counts as acceptance proof.

## 17. NewDest successor gate

M137 does not implement `NewDest`.

Closure must describe a stable consumer contract containing at least:

- session generation identifier or equivalent stale-generation protection;
- authoritative termination cause;
- whether the generation ended by `IdlePolicy`;
- the boundary at which a future resume is considered a new qualifying generation;
- no persistent/replayed idle reason across process restart.

Only after that contract is proven may the registry activate a NewDest plan.

Historical M134 may be amended/rebased if its assumptions exactly match the M137 closure. Otherwise create a corrective M138 rather than silently executing stale M134 authority.

## 18. Stop conditions

Stop rather than approximate if:

- M136 does not provide one canonical clock/state owner;
- canonical session teardown cannot be invoked without duplicating shutdown logic;
- authoritative cause cannot be attached at the winning transition;
- manual/failure races can only be guessed;
- a new SAM wire extension appears necessary;
- Yosemite changes are required;
- path expansion outside §10 is required without amendment;
- support would be accept-inert or based on local handler count.

## 19. Success criterion

M137 succeeds when `Close`/`CloseTime` drive real canonical idle session teardown through the same activity owner as M136, a neutral authoritative termination reason exists for later identity policy, shared/race semantics are proven, and only evidence-backed matrix cells are promoted.