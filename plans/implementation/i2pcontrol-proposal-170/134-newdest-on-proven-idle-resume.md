# M134 — NewDest on Proven Idle Resume

Status: **closed as complete**

Closure authority: `plans/closure/i2pcontrol-proposal-170/134-closure.md`

Rebased on M137: the hard M133 dependency is satisfied by corrective M137
closure (`plans/closure/i2pcontrol-proposal-170/137-closure.md` §12 consumer
contract: `idle_generation`, `SamTerminationReason`, `IdlePolicy` boundary,
new-generation rule, no replay). Historical M134 design material is preserved;
assumptions were mechanically rebased against the proven M137 reason/reopen
interface before code, so no corrective M138 was required.

Class: capability / I2PControl-owned identity lifecycle

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

Hard dependency:

- M133 `133-neutral-sam-idle-close-and-reasoned-termination.md` must close with an authoritative generation-local idle-close reason and stable reopen boundary.

Architecture/security authority:

- M093 tunnel security;
- M110 client destination/key owner;
- M116 shared-session/NewDest corrective authority;
- M120/M123 secret commit/cancellation transactionality;
- M121 semantic demotion of approximate NewDest;
- M131 Streamr `NewDest` not-applicable correction;
- M132/M133 session-lifecycle closure evidence when available.

Pinned Proposal authority remains revision `2026-05-20`. External references remain read-only. This plan authorizes no execution until explicitly registered after M133 closure.

## 1. Objective

Implement exact Proposal `NewDest` semantics for the six non-Streamr client families:

- `client`;
- `httpclient`;
- `ircclient`;
- `socks`;
- `socksirc`;
- `connectclient`.

A fresh client destination/key generation is used **only when reopening after the immediately preceding owning SAM session was authoritatively closed by the configured idle-close policy**.

M134 must not rotate identity on:

- ordinary manual Stop -> Start;
- explicit Restart;
- process restart;
- SAM transport failure;
- router/network failure;
- failed/cancelled start/resume;
- edits unrelated to a qualifying idle close.

`NewDest:streamrclient` remains `not_applicable` under M131 authority and is out of scope.

## 2. Readiness gate

Before registration, M133 closure must provide a stable neutral contract that can answer, without inference:

- which exact session generation terminated;
- whether termination reason was idle policy;
- that the session/pool teardown completed far enough for a later start to be a resume rather than a concurrent duplicate;
- that stale reasons from older generations are distinguishable from the current generation.

If M133 can only report generic disconnect/unknown, M134 remains blocked. Do not reconstruct idle close from elapsed time, local handler count, EOF timing or error strings.

## 3. Ownership

M134 is intentionally I2PControl-owned. No new core identity API is required.

Canonical owners expected at registration:

- `emissary-cli/src/i2pcontrol/client_secret_store.rs` — existing staged/committed client destination identity owner;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` — shared Yosemite session acquisition and compatibility identity;
- `emissary-cli/src/i2pcontrol/production.rs` — existing TunnelManager start/edit/commit transaction boundary if lifecycle disposition is coordinated there;
- the existing I2PControl-local SAM lifecycle observation adapter/state, if M133 exposes the idle-close reason through that accepted passive seam.

Core owns only the neutral reason produced by M133. Core MUST NOT know `NewDest`, persistent-client-key policy, tunnel names as administrative identities, or I2PControl secret stores.

## 4. Exact lifecycle model

For each control-plane client tunnel/session generation, I2PControl may track a bounded internal disposition equivalent to:

- no qualifying idle close;
- generation `G` closed by idle policy and is eligible for one resume decision;
- successor identity staged for resume generation `G+1`;
- successor identity committed for successful resume;
- eligibility consumed/cleared.

The actual type/naming may differ, but the model must make one-shot generation ownership explicit.

A qualifying idle-close fact is consumed by at most one successful resume transaction. Concurrent starts must serialize through the existing per-name/shared-session lifecycle owner and cannot create two successors.

## 5. Proposal interactions

### 5.1 `Close`

`NewDest=true` requires the exact reference prerequisite that idle close is enabled. If the pinned/reference behavior requires `Close=true`, validation fails before allocation when NewDest is supplied without it.

### 5.2 `PersistentClientKey`

Reference configuration states that `newDestOnResume=true` is incompatible with persistent-client-key behavior. M134 must preserve the exact pinned/reference conflict:

- reject incompatible configuration before allocation/secret staging;
- never rotate a persistent identity behind an option that promises persistence;
- never silently coerce one option off.

### 5.3 `PrivKeyFile`

If imported/persistent identity plus NewDest is incompatible under the reference contract, reject before allocation. Do not overwrite or mutate the imported source file.

### 5.4 `Shared`

A shared Yosemite session has one destination identity and one idle-close reason. Therefore all sharing members must have compatible NewDest/identity lifecycle policy.

A qualifying idle close of the shared session creates at most one successor shared identity, not one identity per member. Membership changes during idle/resume must remain bounded and deterministic.

## 6. Identity transaction

Use the existing destination-secret stage/commit/discard pattern established by M120/M123.

Required sequence for a qualifying resume:

1. validate current definition and the unconsumed idle-close generation fact;
2. reserve/serialize the start generation through the existing lifecycle owner;
3. stage a fresh transient successor destination only if NewDest is enabled and the reason qualifies;
4. construct the Yosemite session outside persistent-state locks;
5. if creation/start fails or is cancelled, discard the staged successor and leave the last committed identity/definition unchanged;
6. once the new session is irreversibly accepted by the existing commit boundary, atomically commit the successor identity and consume the idle-close eligibility;
7. publish running state only after the identity/session transaction is coherent.

A failed attempt may retry the same logical resume without consuming eligibility. Whether the same staged key may be reused across a retry or a new staged key must be frozen to the existing secret-store transaction model; no leaked orphan keys.

## 7. Manual lifecycle semantics

Manual actions remain distinct from idle resume:

- `Stop` clears/renders ineligible any stale idle-resume disposition for that definition/session generation unless the exact reference says otherwise;
- `Start` after manual Stop uses the existing committed identity behavior;
- explicit `Restart` preserves identity unless another existing option independently requires identity replacement;
- process restart reconstructs state from durable configuration/committed secret ownership only and never assumes the prior process ended idle;
- Delete removes any pending eligibility/staged successor with the existing bounded cleanup guarantees;
- Edit that changes NewDest/Close/PersistentClientKey/PrivKeyFile/Shared must validate and reconcile lifecycle state transactionally before a later start.

## 8. Authorized production path budget

Final registration must rebase exact paths against M133 closure. Expected I2PControl-only production paths are:

- `emissary-cli/src/i2pcontrol/client_secret_store.rs` — only if a small generation-aware stage/commit API is required;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` — identity selection/shared compatibility/resume acquisition;
- `emissary-cli/src/i2pcontrol/production.rs` — only where the existing start/commit owner must coordinate idle-resume eligibility;
- the existing I2PControl-local SAM observation/lifecycle adapter path established/used by M133, if needed;
- `emissary-cli/src/i2pcontrol/backends/options.rs` — validation gate for exact NewDest conflicts.

No `emissary-core/**` production change is expected or pre-authorized by M134. If M134 requires a new core behavior beyond the stable neutral M133 termination fact, stop and amend/re-plan rather than moving Proposal identity policy into core.

No Cargo/dependency/Yosemite/startup/frontend/NetDb/crypto change is authorized.

## 9. Work packages

### WP1 — Rebase and freeze reference contract

Read M133 closure, mechanically list the six current NewDest blocked cells, and recheck pinned/reference requirements for close-on-idle, persistent key and shared behavior.

### WP2 — Generation eligibility owner

Add bounded I2PControl-local state connecting an authoritative M133 idle-close reason to exactly one future resume transaction. No wall-clock heuristic.

### WP3 — Validation conflicts

Implement fail-before-allocation NewDest prerequisites/conflicts with Close, PersistentClientKey, PrivKeyFile and Shared compatibility.

### WP4 — Successor secret transaction

Reuse/extend the existing staged secret lifecycle so a qualifying resume gets a fresh destination only at the correct transaction point, with cancellation rollback.

### WP5 — Shared-session successor semantics

Prove one shared session -> one successor identity, compatibility equality includes NewDest/close policy, and concurrent members cannot double-rotate.

### WP6 — Manual/edit/restart cleanup

Clear or preserve eligibility exactly across Stop/Start/Restart/Edit/Delete/process restart according to the frozen contract.

### WP7 — Matrix/docs/closure

Promote only the six M131-blocked NewDest cells with end-to-end evidence. Streamr remains not applicable. Reconcile M095/M105/M110 and decide whether the session-lifecycle roadmap is closed.

## 10. Failure, cancellation, restart and contention

- An idle-close reason is generation-scoped and one-shot.
- Start reservations remain cancellation-safe as established by M116/M123.
- No filesystem/persistent-store lock spans Yosemite session construction/network I/O.
- A staged successor is discarded on every pre-commit failure/cancellation path.
- A committed successor cannot be rolled back after running state/publication crosses the existing irreversible boundary; later failure is handled as the new generation's failure.
- Concurrent Start calls cannot consume the same eligibility twice.
- A manual Stop/Restart racing idle-close reason delivery must resolve under one lifecycle serialization owner; ambiguous stale reason becomes ineligible rather than causing surprise rotation.
- Process restart clears volatile idle-resume eligibility.
- Shared member release/acquisition cannot orphan the shared successor identity.

## 11. Compatibility and migration

No public wire/API change. No durable migration should be required for idle-resume eligibility because it is intentionally process/generation-local.

Committed client destination storage remains in its existing format unless the current secret-store contract strictly requires a bounded metadata addition. Any such change requires an explicit backward-compatible migration section in the registration amendment.

Existing definitions with `NewDest` remain round-trippable while blocked; after M134 they become startable only when the full exact prerequisite configuration is valid.

## 12. Focused tests

At minimum:

1. NewDest without qualifying idle close -> no rotation;
2. qualifying idle close + NewDest -> exactly one fresh identity on successful resume;
3. second ordinary start after that resume -> no second rotation;
4. manual Stop -> Start -> identity preserved;
5. explicit Restart -> identity preserved;
6. process restart -> no inferred NewDest rotation;
7. network/SAM failure -> no rotation;
8. stale idle-close reason from generation G cannot rotate G+2;
9. failed resume discards staged successor and eligibility remains correctly retryable;
10. cancelled resume at every M123 commit boundary does not leak or double-commit key material;
11. concurrent starts consume eligibility once;
12. NewDest + PersistentClientKey conflict rejects before allocation;
13. NewDest + incompatible PrivKeyFile behavior rejects before allocation;
14. NewDest prerequisite on Close is enforced exactly;
15. shared session creates one successor identity across all members;
16. incompatible shared NewDest/close policies do not share;
17. edit/delete clears/reconciles pending eligibility safely;
18. no raw private destination/key appears in Debug/log/RPC/error/matrix evidence;
19. Streamr NewDest remains not applicable and is not reintroduced;
20. M061/M062 containment confirms I2PControl-only production changes unless an explicit amendment says otherwise.

## 13. Broad verification

Run the session-lifecycle roadmap baseline and all M123/M132/M133 lifecycle/cancellation regressions, including:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-core --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 14. Matrix accounting

M134 starts from the M133 closure matrix.

M131 authority contains exactly six blocked non-Streamr `NewDest` cells and marks `NewDest:streamrclient` not applicable. M134's maximum promotion is therefore six cells.

No count is frozen before implementation. Closure mechanically recomputes authority.

## 15. Security/static guards

Closure must prove:

- no NewDest/Proposal policy entered core;
- only the existing I2PControl secret owner can stage/commit successors;
- remote peers/SAM payloads cannot forge the idle-close reason;
- no secret material enters lifecycle observation;
- restart/manual action cannot unexpectedly rotate identity;
- path confinement/import semantics are unchanged;
- no new dependency/Yosemite patch exists;
- M132/M133 semantics remain green.

## 16. Acceptance criteria

M134 closes only when:

1. M133 exposes a stable authoritative idle-close reason;
2. eligibility is generation-local, one-shot and non-persistent;
3. NewDest rotates exactly once on successful qualifying resume;
4. manual/restart/failure paths preserve identity;
5. failed/cancelled resume rolls back staged successor state;
6. persistent/import/shared conflicts are exact and fail-before-allocation;
7. six promoted cells have end-to-end evidence and Streamr stays N/A;
8. production changes remain I2PControl-local unless explicitly amended;
9. broad/cancellation/security verification is green;
10. closure decides the next residual cluster without broadening M134.

## 17. Stop conditions

Stop if:

- M133 cannot provide an authoritative reason without heuristics;
- exact reference conflict rules cannot be established;
- identity rotation would require core Proposal policy;
- secret transaction cannot be made cancellation-safe within existing owner;
- shared semantics would require multiple identities for one shared session;
- a new dependency/storage migration/path expansion is required without amendment.

## 18. Closure evidence required

Require M133 dependency proof, exact changed paths, reference conflict table, generation-state model, commit/rollback traces, focused and broad verification, six-cell mechanical matrix accounting, secret/security review, shared/manual/restart race evidence, unresolved findings and internal-only/read-only-upstream attestation.