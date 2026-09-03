# M123 — M120 Commit-Phase Cancellation Atomicity Corrective

Status: **ready**

Class: invariant / persistence-lifecycle corrective

Repository: `eggstack/emissary`

Baseline: `045d1e8b4eba1141d2488882f99c5ce994db91a8`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Corrective targets:

- `plans/implementation/i2pcontrol-proposal-170/120-server-start-preallocation-validation-and-secret-transactionality-corrective.md`
- `plans/closure/i2pcontrol-proposal-170/120-closure.md`

Applicable architecture/guards:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- M107/M108 secret/TLS hardening principles;
- M120 deterministic-preflight and server-secret staging design.

Pinned Proposal 170 revision: `2026-05-20` (Open).

External/upstream sources are read-only. No upstream issue, PR, review, contact, submission, release, merge or adoption activity is authorized.

## 1. Objective

Make control-plane-owned **server start and restart terminalization cancellation-safe after the runtime has started**.

M120 correctly moved deterministic validation ahead of secret allocation and introduced staged secret commit/rollback. However, once `commit_server_start()` begins, it disarms `ServerStartGuard` before asynchronous secret publication and durable definition persistence. If the owning future is dropped during those awaits, no rollback path is guaranteed to run. The lifecycle lock held by the caller may also be released while the partially committed runtime/storage transition is still unresolved.

M123 must guarantee one terminal outcome for every server start generation:

1. **committed:** runtime, authoritative private destination and durable definition/public destination all describe the same generation; or
2. **rolled back:** runtime is stopped and the exact pre-start durable secret/definition state is restored.

There must be no cancellation point that can leave a third half-committed state.

M123 changes no Proposal 170 matrix cell. Current M095 remains `284 apply / 98 blocked_primitive / 458 not_applicable`.

## 2. Findings owned by M123

### F1 — guard is disarmed before cancellable durability awaits

Current `commit_server_start()` removes the drop guard before awaiting:

- `ServerDestinationStore::commit()`;
- `TunnelStore`/public-destination persistence;
- rollback `backend.stop()`/secret restoration on ordinary errors.

Dropping the future after disarm can therefore skip both commit completion and rollback.

### F2 — lifecycle exclusion does not survive caller cancellation

`start()` / `restart()` hold the per-name lifecycle lock in the caller future. If that future is cancelled while commit terminalization is incomplete, the lock is dropped and a later edit/stop/start/delete may observe or mutate the same name before the previous transaction reaches a terminal state.

### F3 — cancellation cleanup is best-effort under state-lock contention

`ServerStartGuard::Drop` calls `ServerDestinationStore::discard_sync()`, which uses `try_lock()`. If the lock is contended, staged secret state is deliberately left behind. That is not a sufficient invariant for a cancellation guard whose closure claimed deterministic cleanup.

### F4 — existing M120 cancellation evidence covers the armed staging window, not all commit boundaries

M120's abort regression demonstrates cleanup while staging is still guarded. It does not deterministically pause/abort:

- inside secret durable publication;
- after secret durability succeeds but before definition/public-destination persistence;
- during replacement-secret terminalization.

M123 adds those missing regressions.

## 3. Required terminalization model

The implementation may choose commit-to-completion or rollback-on-cancel, but it MUST satisfy all invariants below.

### 3.1 Owned terminal state machine

After backend start succeeds and a server destination is available, transaction state must have an explicit owner until terminal commit or rollback.

No code may simply "disarm the guard and await" without another owner that survives cancellation.

The preferred bounded design is an **owned terminalization future/task** that receives:

- the prepared server transaction state;
- the exact pre-start rollback snapshot for replacements;
- the backend/runtime handle needed to stop on failure;
- the server destination store;
- the durable tunnel store handle;
- the per-name owned lifecycle guard or equivalent exclusion token.

The caller may await this owner for a normal result, but dropping the caller's await must not drop the terminalization owner. If a detached Tokio task is used, the per-name lifecycle guard MUST move into that task before the first commit-phase await so later lifecycle operations cannot overtake it.

A fire-and-forget cleanup task that does **not** retain lifecycle exclusion is not acceptable.

### 3.2 Alternative rollback-on-cancel design

A different design is acceptable only if it proves that dropping the future synchronously establishes an unambiguous rollback owner capable of completing all required async restoration while retaining lifecycle exclusion.

Do not rely on async work inside `Drop`, `try_lock()` best effort, process-exit timing, or an unowned spawned cleanup.

### 3.3 Terminal outcomes

For `Fresh`:

- committed outcome: candidate private destination durable + identity/public destination durable + runtime running;
- rollback outcome: runtime stopped, no candidate private destination durable, original definition unchanged and no orphan identity/public destination.

For `Replacement`:

- committed outcome: replacement private destination durable + matching public destination/definition durable + runtime running;
- rollback outcome: runtime stopped and exact previous private destination + previous durable definition/public destination restored.

For `ExistingUnchanged`:

- public-destination persistence and runtime state must still terminalize consistently; cancellation must not leave a newly running runtime whose durable display state is indeterminate.

Startup-managed tunnels are not owned by this transaction and remain unchanged.

## 4. Invariants

M123 MUST preserve:

- M120 validation order: load → common validation → backend pure preflight → secret staging → backend start → terminalization;
- deterministic unsupported configuration fails before secret allocation/import/generation;
- one per-name lifecycle owner for create/edit/start/stop/restart/delete interactions;
- no lock held across external/network I/O except an owned lifecycle exclusion token specifically intended to span the operation;
- no private destination/key material in logs, errors, Debug/Display, raw config or public responses;
- fixed confined import/storage paths;
- exact previous-secret restore for replacement rollback;
- startup-managed tunnel ownership untouched;
- no Proposal matrix promotion/demotion;
- no dependency changes;
- no router-core/Yosemite/frontend changes.

## 5. Authorized production scope

Preferred production paths:

- `emissary-cli/src/i2pcontrol/production.rs`;
- `emissary-cli/src/i2pcontrol/server_secret_store.rs` only for deterministic staged-state ownership/cleanup;
- existing focused tests in those modules.

Static/plan evidence may update:

- `emissary-cli/tests/m062_dependency_containment.rs`;
- M062 containment metadata;
- M123 plan/closure, registry, roadmap and implementation README.

No changes are authorized to:

- `emissary-core/**`;
- `emissary-util/**`;
- Yosemite dependency or Cargo manifests/lockfile;
- backend option semantics except where a test-only terminalization seam is strictly necessary;
- startup tunnel implementation;
- frontend/UI/workflows/releases.

If cancellation safety requires a production path outside `production.rs` and `server_secret_store.rs`, stop and record the required path before editing it.

## 6. Work packages

### WP1 — deterministic commit-boundary harness

Add test-only barriers/failpoints around the terminalization boundaries without adding production observability or a general fault-injection framework.

Required pause points:

1. candidate staged, runtime successfully started, before secret durable commit;
2. secret durable commit completed, before durable definition/public-destination persistence;
3. replacement candidate committed, before replacement definition/public destination is durable.

The harness must allow the test to abort/drop the caller future at each point and then deterministically wait for terminal state.

### WP2 — lifecycle-owner transfer

Refactor the start/restart internal flow only as much as necessary so the per-name lifecycle exclusion remains owned through terminalization after caller cancellation.

Requirements:

- no double-lock deadlock between `restart()` and `start_locked()`;
- ordinary `stop`, `delete` and `update` cannot overtake an unresolved terminalization;
- cancellation before backend start may still roll back staged state normally;
- cancellation after backend start cannot abandon the runtime/storage transaction.

If using a Tokio task, task creation is internal implementation machinery, not a new long-lived supervisor. It must terminate after one start transaction and must not create an unbounded task registry.

### WP3 — deterministic staged-secret Drop behavior

Remove `try_lock()` as the sole correctness mechanism for guard cleanup.

Acceptable approaches include:

- moving the purely in-memory pending map behind a non-async mutex that can be deterministically cleared in `Drop`, provided no blocking I/O occurs under it;
- eliminating the synchronous guard's responsibility by transferring the staged candidate into the owned terminalization state before any cancellation window;
- another bounded design with equivalent proof.

A contended lock may not silently convert required cleanup into "left for next load".

### WP4 — commit/rollback terminal state machine

Make every fallible commit transition explicit.

At each boundary, document and test what is authoritative:

- staged only;
- secret durable / definition old;
- secret + definition durable;
- rollback in progress;
- terminal committed/rolled back.

The implementation need not add a persisted transaction journal if an in-process owned terminalizer can make the transitions cancellation-safe. Do not add journaling unless the simpler owner model cannot satisfy crash/restart invariants.

### WP5 — restart and contention regressions

Cover both direct `start()` and the start half of `restart()`.

At a paused commit boundary, launch a competing same-name `stop`, `delete` or `update` and prove it cannot pass lifecycle exclusion until the prior transaction terminalizes.

After terminalization, the competing action must observe one coherent committed or rolled-back state.

### WP6 — closure/reconciliation

- matrix remains byte/count equivalent at `284 / 98 / 458`;
- M120 historical closure remains unchanged;
- M123 closure supersedes only M120's cancellation-atomicity claim;
- no LeaseSet/M111/M112 residual feature work occurs here.

## 7. Failure, cancellation, restart and crash semantics

### Ordinary errors

Retain M120 behavior: backend/runtime error before commit rolls back staging; secret publication or definition persistence errors stop runtime and restore exact previous state.

### Caller cancellation

Dropping/aborting the RPC/start future is **not** permitted to abandon terminalization.

The final disposition may be commit-to-completion or rollback, but must be deterministic from the transaction state and must finish while lifecycle exclusion remains owned.

The caller need not receive a result after it has cancelled; repository/runtime state still must converge to a terminal state.

### Process crash

M123 does not promise cross-process distributed transactions. Preserve the existing durable ordering so a crash never makes an uncommitted staged in-memory candidate authoritative. If the new owner design reveals a crash state that can leave durable secret/definition disagreement after restart, stop and either add bounded load-time repair evidence within I2PControl or return the plan for a separately scoped persistence-recovery milestone.

### Contention

Same-name lifecycle operations remain serialized. Different tunnel names may proceed independently; do not introduce a global start lock.

## 8. Focused tests

Required deterministic tests:

1. abort fresh start before secret commit → terminal coherent state, no staged leak;
2. abort fresh start after secret commit but before definition persistence → terminal coherent state with no orphan secret/runtime;
3. abort replacement start before secret commit → exact previous secret/definition retained;
4. abort replacement start after replacement secret commit but before definition persistence → exact coherent commit or exact previous-state rollback;
5. abort `ExistingUnchanged` start during public-destination persistence → runtime/durable state terminalizes coherently;
6. abort restart during its start/commit phase → no half-stopped/half-new state;
7. lifecycle contention cannot overtake terminalization;
8. staged cleanup succeeds even when the old `try_lock()` failure condition is deterministically simulated/replaced;
9. no secret material appears in error/debug/log-capture fixtures;
10. successful start/restart behavior remains unchanged.

Tests must assert both in-memory state and reload-from-disk state where durability is involved.

## 9. Broad verification

Run from repository root:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Known repository rustfmt toolchain drift may be dispositioned only if no M123-added line introduces new drift.

## 10. Acceptance criteria

M123 closes only when:

1. no server commit-phase await can be abandoned without a surviving terminalization owner;
2. per-name lifecycle exclusion survives caller cancellation until terminal state;
3. fresh/replacement/existing server starts have explicit committed-or-rolled-back outcomes at every tested boundary;
4. `ServerStartGuard`/pending-state cleanup is deterministic rather than best effort;
5. restart shares the same invariant;
6. reload-from-disk assertions match the terminal in-memory state;
7. no secret leakage or containment regression occurs;
8. matrix remains `284 / 98 / 458`;
9. no high/medium M123 finding remains open.

## 11. Stop conditions

Stop and return for replanning if:

- correctness requires a router-core lifecycle API;
- correctness requires a general durable transaction framework shared outside I2PControl;
- a solution would hold a broad store mutex across network I/O;
- a solution would spawn unbounded/unowned cleanup tasks;
- a dependency/Cargo/Yosemite change is proposed;
- Proposal support cells would be changed;
- upstream interaction is proposed.

## 12. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/123-closure.md` with:

- implementation commits;
- F1-F4 requirement/evidence mapping;
- exact terminal-state model;
- cancellation-boundary test table;
- restart/contention/reload evidence;
- changed-path/containment audit;
- exact verification commands/outcomes;
- security/secret review;
- matrix unchanged evidence;
- unresolved findings and severity;
- next-handoff decision;
- internal-only external-interaction attestation.
