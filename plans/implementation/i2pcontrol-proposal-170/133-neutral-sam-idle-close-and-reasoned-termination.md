# M133 — Neutral SAM Idle Close and Reasoned Termination

Status: **deferred / unregistered; hard-depends on M132 closure**

Class: capability / neutral lower-layer exception / Proposal-170 vertical slice

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

Hard dependency:

- M132 `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md` must close with a stable SAM activity/idle state machine and generation-local pool/session lifecycle contract.

Authority inherited from M132:

- M061/M062 containment;
- M093 tunnel security;
- M110/M116 shared-session ownership;
- M121 semantic truthfulness;
- M123 cancellation atomicity;
- M131 primitive map path budget `PB-SESSION-IDLE-CLOSE-01`.

Pinned Proposal authority remains revision `2026-05-20`. External reference sources remain read-only. This plan authorizes no execution until it is explicitly registered after M132 closure.

## 1. Objective

Extend the neutral M132 SAM-session idle state machine to implement exact generic close-on-idle behavior and consume it from I2PControl for Proposal fields:

- `Close`;
- `CloseTime`.

M133 must also expose a minimal **neutral, in-process termination fact** that distinguishes a session intentionally destroyed by the idle policy from ordinary/manual termination. That fact is required for the separately planned M134 `NewDest` consumer.

M133 does not rotate destination keys and does not implement `NewDest`.

## 2. Readiness gate

Before registration, M132 closure must prove all of the following stable interfaces:

- one canonical activity timestamp/state machine in `SamSession`;
- no local-TCP-handler heuristic;
- generation-local monotonic timer behavior;
- clean session/pool shutdown and replacement-generation isolation;
- a stable way to parse standard I2CP session options without Yosemite changes;
- no unresolved high/medium correctness issue in the M132 session owner.

If M132 closes blocked or materially changes the owner/path contract, this plan must be amended before registration.

## 3. Reference semantic contract

M133 execution must re-check and freeze:

- `i2cp.closeOnIdle=true` enables idle session destruction;
- `i2cp.closeIdleTime` is milliseconds;
- Java default close time is 30 minutes;
- Java minimum is 5 minutes;
- close is evaluated against the same session activity clock used by reduction;
- when reduce and close are both enabled and close time is less than or equal to reduce time, Java suppresses reduction rather than running a useless reduction first;
- close-on-idle destroys the I2P session, which tears down its tunnel pool;
- activity before the close threshold postpones close;
- after close, a later application/tunnel owner may choose to reopen a new session; that reopen policy is outside the neutral core close primitive.

Do not infer close from lack of local TCP handlers.

## 4. Neutral lower-layer design

### 4.1 One idle state machine

M133 MUST extend the M132 state machine rather than introducing a second competing timer.

The state machine may represent:

- active;
- reduced, when reduction is configured and has occurred;
- closing/closed by idle policy.

It must compute the next relevant deadline from the same last-activity value. Timer rescheduling remains actor-local and bounded.

### 4.2 Session destruction

At the close threshold, the SAM session owner performs the same canonical teardown path used for ordinary session destruction wherever possible:

- stop accepting application commands for the generation;
- shut down the stream manager/destination/tunnel pool in the existing order;
- remove session/subsession mappings through the existing server owner;
- emit no fake success after teardown begins.

Do not create a second pool-shutdown implementation.

### 4.3 Neutral termination reason

M134 needs to know whether termination was caused by the idle policy. M133 may satisfy this by extending the existing passive SAM lifecycle observation seam or by an equivalently narrow in-process result type.

The preferred shape is a generic enum such as:

- `IdlePolicy`;
- `Requested` / `Control` where the owner can prove it;
- `Failure` where the owner can prove it;
- `Unknown`.

Exact naming is implementation-owned, but the type must contain no Proposal/I2PControl terminology and no destination/key material.

A passive observer publication failure must never prevent session teardown. The authoritative reason remains owned by the session lifecycle, not by the observer.

M133 MUST NOT invent a new SAM wire response/event solely for I2PControl.

### 4.4 Subsessions/shared sessions

Reference primary/subsession activity aggregation remains authoritative. Closing the owning primary session closes its dependent subsessions.

I2PControl shared definitions using one Yosemite session share the same close policy and activity clock. A member with incompatible close policy must not share that session; exact options must participate in the existing compatibility key.

## 5. I2PControl contract

I2PControl validates and translates:

- `Close` -> `i2cp.closeOnIdle`;
- `CloseTime` -> `i2cp.closeIdleTime` in the pinned/reference unit.

The standard option values are transported through the existing Yosemite session options/additional-options mechanism. No raw SAM encoder.

If `CloseTime` is present while close is disabled/absent, preserve the exact pinned/reference presence semantics; do not silently enable close unless the reference contract does so.

M133 continues to reject `NewDest` before allocation. M134 is the only milestone allowed to remove that gate.

## 6. Authorized production path budget

Final registration must rebase this list against the actual M132 closure diff. Expected paths are:

Neutral core:

- `emissary-core/src/sam/session.rs` — extend M132 idle state machine with close;
- `emissary-core/src/sam/mod.rs` — only if the existing lifecycle observation event/result must gain a neutral termination reason;
- existing SAM session-server context path only if required to carry the reason through authoritative removal; exact path must be named in the registration amendment;
- no tunnel-pool production change beyond what M132 already established unless exact shutdown integration requires a narrowly justified correction.

I2PControl:

- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` — Proposal Close/CloseTime validation and standard option translation;
- `emissary-cli/src/i2pcontrol/backends/options.rs` only if the common capability gate requires adjustment;
- the existing in-process SAM observation consumer under I2PControl only if it already owns lifecycle facts and M134 requires durable generation-local idle-close evidence; no new global observer subsystem.

Evidence/docs:

- M061/M062 exact-path authority/tests;
- M095/M105/M110 reconciliation;
- focused M133 tests;
- support/tunnel-manager docs;
- closure/registry/roadmaps.

No Cargo/dependency/Yosemite/frontend/NetDb/crypto/profile/transport changes are authorized.

## 7. Work packages

### WP1 — Rebase after M132

Inspect the actual M132 closure head, changed paths, state machine and unresolved findings. Amend this plan before registration if the assumed interface differs.

### WP2 — Freeze close semantics

Record exact reference timer ordering, reduce+close interaction, primary/subsession behavior and teardown sequence.

### WP3 — Extend idle state machine

Add close deadline/evaluation to the M132 owner. Reuse its activity tracking and timer. Prove no duplicate timers/tasks.

### WP4 — Reasoned termination

Add the minimum neutral termination reason through the authoritative lifecycle/removal path. Unknown conditions stay unknown.

### WP5 — I2PControl translation

Enable Close/CloseTime for only applicable client families after core support exists. Keep NewDest blocked.

### WP6 — Shared-session and cancellation evidence

Prove compatible policy sharing, incompatible policy separation, creator cancellation safety and final-member teardown behavior.

### WP7 — Matrix/docs/closure

Promote only exact cells with end-to-end evidence; mechanically reconcile current authority and decide M134 readiness.

## 8. Failure, cancellation, restart and contention

- Timer/session state is generation-local and cannot outlive the session.
- Idle close must be idempotent if shutdown is concurrently requested.
- Manual Stop/Restart racing an idle deadline must produce one teardown, never double-remove the session/pool.
- The authoritative termination reason follows the transition that wins; if the owner cannot distinguish a race safely, use `Unknown` rather than fabricating `IdlePolicy`.
- Passive observation failure cannot block teardown.
- No lock is held across network I/O, join/wait or timer await.
- Process restart begins a new session generation and is not classified as idle resume.
- Shared-session final-member release and idle close cannot leave a registry entry referring to a dead Yosemite session.

## 9. Compatibility and migration

No public API version/storage migration is expected. Core close behavior is unchanged when standard close-on-idle options are absent.

Existing definitions containing blocked Close/CloseTime values remain round-trippable; successful start becomes available only after validation and exact runtime support.

No wire extension is added to SAM.

## 10. Focused tests

At minimum:

1. absent close option leaves session lifetime unchanged;
2. close enabled does not fire before threshold;
3. qualifying activity reschedules the close deadline;
4. naming/PING/control traffic does not reschedule;
5. exact idle threshold tears down one session generation and its pool;
6. reduce+close ordering follows reference, including close <= reduce suppression;
7. manual stop racing idle close performs one teardown;
8. transport/session failure is not reported as idle close unless authoritative owner says so;
9. reason publication failure cannot prevent teardown;
10. primary/subsession teardown is coherent;
11. shared definitions with identical close policy share; different close policy does not;
12. malformed/below-minimum/overflow Proposal CloseTime fails before allocation according to the frozen contract;
13. NewDest still fails before allocation throughout M133;
14. stale reason from an old generation cannot qualify a replacement generation;
15. M061/M062 reject path expansion and core Proposal terminology.

## 11. Broad verification

Use the session-lifecycle roadmap baseline plus M132 retained tests:

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

## 12. Matrix accounting

M133 must start from the M132 closure matrix, not the historical M131 count.

The current M131 matrix contains 14 `Close`/`CloseTime` cells across seven client families. Applicability must be rechecked against the M132 Streamr decision/reference evidence. Promote no Streamr cell solely because the core primitive can technically apply.

Count changes are mechanically computed at closure.

## 13. Security/static guards

Closure must prove:

- core reason types contain no Proposal/I2PControl concepts or secrets;
- no public/general router-control endpoint is introduced;
- no new SAM wire field/status/event is added;
- termination reason cannot be spoofed by a remote peer or local unauthenticated SAM payload;
- one session cannot close another session through the new primitive;
- exact production paths are recorded in M062;
- M132 behavior remains green.

## 14. Acceptance criteria

M133 closes only when:

1. M132 is closed and its activity owner is reused;
2. exact close/reference ordering is frozen;
3. close-on-idle performs real bounded session/pool teardown;
4. a neutral authoritative idle-close reason exists in-process;
5. manual/failure teardown is not mislabeled;
6. Proposal Close/CloseTime validation/translation is fail-before-allocation and raw-SAM-free;
7. shared/cancellation/restart races are covered;
8. every promoted matrix cell has end-to-end evidence;
9. no unauthorized path/dependency changes occur;
10. M134 receives a stable, documented idle-close/reopen interface or remains blocked.

## 15. Stop conditions

Stop if:

- M132 closure changes the owner enough that this plan is stale;
- correct idle close requires a broad SAM server redesign;
- the idle-close reason can only be exposed by inventing a nonstandard SAM wire extension;
- teardown races cannot be made authoritative without weakening existing lifecycle safety;
- a new dependency/Yosemite modification is required without separate planning;
- path scope expands before amendment.

## 16. Closure evidence required

Require implementation commits, M132 dependency proof, exact changed paths, reference semantic table, focused/broad results, reason/race analysis, shared-session review, starting/final matrix, compatibility/security review, unresolved findings and an explicit M134 readiness decision.