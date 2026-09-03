# M121 — M111/M112 Semantic Truthfulness Corrective

Status: **closed** — both areas demoted (Outcome C + §5.2); matrix is 284 apply / 98 blocked / 458 not_applicable; see `plans/closure/i2pcontrol-proposal-170/121-closure.md`

Class: corrective / Proposal conformance / lifecycle truthfulness

Planning baseline: `feafc6a1d9650887015a01f87bf21b57a4e92085`

Corrects/re-audits claims from:

- `plans/closure/i2pcontrol-proposal-170/111-closure.md` (`SigType` support classification);
- `plans/closure/i2pcontrol-proposal-170/112-closure.md` (`Close`, `CloseTime`, `NewDest` idle-session semantics).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`.

Pinned Proposal authority remains revision `2026-05-20`.

## 1. Objective

Independently re-freeze and either prove, correct, or truthfully demote two already-applied semantic areas:

1. whether Proposal `SigType` may be classified `apply` when the I2PControl-facing runtime accepts exactly canonical signing type `7` and rejects all other values;
2. whether M112's `Close` / `CloseTime` / `NewDest` behavior—currently driven by the absence of active local TCP handler tasks—is equivalent to the pinned I2PTunnel/I2CP definition of an idle I2P session and resume-with-new-destination behavior.

This pass must prefer truthfulness over matrix preservation. It may reduce the current `312 apply / 70 blocked / 458 not_applicable` count if exact semantics cannot be implemented inside accepted ownership boundaries.

## 2. Why prior verification was insufficient

### F1 — `SigType` API-to-wire capability vs accepted value domain

Yosemite Y001/Y002 and M117 can serialize/generate arbitrary numeric SAM `SIGNATURE_TYPE` values. M111 nevertheless constrains actual Proposal-facing validation to the exact string `"7"` because Emissary only has the required private signing-key support for that type.

M111 promoted ten `SigType` cells based on real wire plumbing and fail-closed rejection of unsupported values, but did not independently settle whether Proposal 170 considers 'one fixed supported signing type' a valid implementation of a configurable `SigType` field.

### F2 — local connection count may not equal I2P session idle

M112's `ConnectionActivity` counts accepted local TCP handler tasks. `run_idle_closer()` closes the Yosemite session only after that count stays zero for `CloseTime`.

The reference `i2cp.closeOnIdle` behavior is an I2P-session idle policy. A local TCP socket may remain open while carrying no I2P traffic, and an I2P session may have activity patterns not represented by the local handler count. M112 tests verified its local lifecycle state machine but did not prove semantic equivalence to reference session-idle behavior.

`NewDest` is coupled to the reference close-on-idle resume path, so any `Close` semantic mismatch also affects the 6 applied `NewDest` cells.

## 3. Scope

Expected production changes, if semantics can be corrected locally, are limited to:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs`;
- existing six TCP client backend composition files only if needed to carry a corrected lifecycle/activity signal;
- I2PControl matrix/audit/ledger tests and documentation.

No `emissary-core/**`, `emissary-util/**`, Cargo/dependency, Yosemite, frontend, workflow, release, or startup tunnel production change is authorized by M121.

If exact semantics require a neutral lower-layer primitive outside these owners, M121 must demote the affected cells to `blocked_primitive`, record the missing primitive precisely, and stop. A new neutral-owner plan may then be written separately.

## 4. SigType semantic freeze

Before changing code or matrix state, inspect the pinned Proposal 170 revision, Java I2PTunnel configuration/reference behavior, SAM signing-type rules, and Emissary's actual destination signing capabilities.

Answer explicitly:

1. Is Proposal `SigType` a best-effort selection constrained to router-supported algorithms, or is configurable support across a defined reference value set required?
2. Does accepting only type 7 constitute a truthful supported value domain, or is that equivalent to an inert/fixed field?
3. Which signing types can Emissary actually generate and use end-to-end for both transient and persistent destinations today?
4. Is the value encoded as numeric string only, or does reference I2PControl accept names/aliases that Proposal 170 requires?

### 4.1 Allowed outcomes

**Outcome A — retain apply:** only if affirmative reference/spec evidence says an implementation may expose the option with a supported-domain subset and rejecting unsupported algorithms is conformant. Closure must document the supported domain as `{7}` and tests must prove no fallback.

**Outcome B — expand locally:** only if Emissary already has accepted lower-layer signing primitives for additional required types and I2PControl merely failed to expose them. Do not add new cryptographic algorithms in M121.

**Outcome C — demote:** if configurable semantics require values Emissary cannot actually generate/sign with, reclassify the ten `SigType` cells to `blocked_primitive` with the exact missing signing-key primitive. Do not preserve `apply` merely because Yosemite can put a number on the wire.

No `accept_inert`, silent coercion, or fallback to 7 is permitted.

## 5. Close/CloseTime/NewDest semantic freeze

Trace the Java/reference lifecycle for:

- what event resets `closeOnIdle` idle time;
- whether open but inactive streaming connections keep the I2P session non-idle;
- whether the policy is based on bytes/messages/session activity rather than socket/task presence;
- the exact transition that causes `newDestOnResume` to allocate a new identity;
- interaction with `delayOpen`, persistent client key, shared session, multiple local connections, and restart/manual start.

Use implementation/reference evidence, not option names alone.

### 5.1 Preferred local correction

If exact reference semantics can be represented entirely inside the existing I2PControl generation owner, implement a bounded session-activity signal derived from the actual I2P stream/session use that the owner controls.

Properties:

- activity timer resets on reference-equivalent I2P traffic/use, not merely accepted local socket lifetime;
- an open but idle local connection behaves according to reference evidence;
- multiple concurrent connections contribute correctly without unbounded per-byte bookkeeping;
- idle close remains generation-local and cancellable;
- no lock is held across stream I/O;
- after an actual idle close, `NewDest=true` creates a transient successor identity only on resume;
- manual stop/start and failed connection attempts do not spuriously rotate identity;
- `Shared=true` close remains rejected unless exact shared-session semantics can be proven without one member terminating another.

Use a coarse atomic timestamp/counter/watch signal if sufficient; do not add a router-global monitor or per-packet logging subsystem.

### 5.2 Required demotion if no exact local signal exists

If Yosemite/Emissary does not expose enough information to implement reference session-idle semantics locally without changing core/router/session internals, reclassify:

- `Close` × six TCP client families;
- `CloseTime` × six TCP client families;
- `NewDest` × six TCP client families;

to `blocked_primitive` and record the exact missing observation/control primitive.

`ConnectDelay` remains outside this corrective unless the audit finds a direct semantic dependency.

Do not implement an approximation and keep the cells applied.

## 6. Invariants

M121 MUST preserve:

- M110/M116 shared-session compatibility/cancellation ownership;
- M112 generation-local lifecycle and no-lock-across-I/O structure;
- destination rotation only after the accepted close/resume trigger;
- persistent-client-key and NewDest mutual exclusion;
- server/startup behavior unchanged;
- no direct clearnet fallback or proxy-boundary weakening;
- no secret or destination private-key diagnostics;
- matrix has no `accept_inert`, unknown, or silent unsupported state;
- no upstream activity.

## 7. Explicit non-goals

M121 does not:

- implement M112's 45 already-blocked proxy/plugin/profile/reduce/Streamr cells;
- implement M111's four `UseSSL` cells;
- add cryptographic algorithms to core;
- modify Yosemite;
- implement encrypted LeaseSets;
- change tunnel variance/backup behavior;
- perform final M114-equivalent interoperability closure.

## 8. Work packages

### WP1 — independent semantic evidence table

For `SigType`, `Close`, `CloseTime`, and `NewDest`, record Proposal text, Java/I2PTunnel behavior, Yosemite capabilities, Emissary capabilities, and current implementation behavior side-by-side.

### WP2 — SigType disposition

Choose Outcome A/B/C from §4.1 and add exact tests/matrix evidence. If demoted, calculate exact matrix counts mechanically; do not bake an expected number into the plan.

### WP3 — session idle disposition

Build a deterministic reference-oriented test fixture for active bytes, open-but-idle socket, zero local connections, concurrent connections, failed connect, cancellation, and actual resume.

Implement exact local activity behavior if possible; otherwise demote all 18 coupled cells together unless reference evidence supports a narrower split.

### WP4 — compatibility/identity tests

Prove NewDest rotates only on accepted resume, never manual staging/start, and PersistentClientKey/Shared constraints remain fail-closed.

### WP5 — matrix/ledger reconciliation

Update M095, M105, M110 ledger post-corrective metadata, user-facing support docs, registry, and corrective roadmap only after runtime evidence fixes the final disposition.

### WP6 — containment/closure

Update exact M061/M062 paths and write M121 closure. Historical M111/M112 closures remain untouched except references stating they were later corrected by M121.

## 9. Failure, cancellation, restart, contention

Any lifecycle timer/activity mechanism must be generation-local and terminated/awaited on generation exit. Cancellation wins over delayed close/resume.

Concurrent connection activity must not race an idle closer into dropping the session after reference-defined activity has resumed. Use ordering/atomics/watch semantics that permit deterministic tests at the idle-boundary race.

A failed successor session creation after idle close must report failure truthfully and must not resurrect the old closed generation or commit an unintended destination identity.

## 10. Focused tests

Required tests depend on disposition but must include:

- `SigType=7` exact wire + actual generated identity path;
- unsupported/noncanonical SigType never falls back;
- at least one reference-required non-7 value used to prove either real support or truthful blocked disposition;
- open local socket with no I2P traffic across `CloseTime`;
- active I2P transfer around the idle deadline resets/prevents close according to reference;
- zero local connections but recent I2P activity behavior;
- two concurrent connections with one active/one idle;
- idle close followed by NewDest resume produces exactly one new identity;
- manual start/restart does not count as NewDest resume;
- failed resume is bounded and secret-safe;
- cancellation at idle deadline cannot close a successor generation;
- matrix exact-count test matches reconciled artifact.

## 11. Broad verification

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

## 12. Acceptance criteria

M121 closes only when:

1. `SigType` support status is backed by explicit Proposal/reference and actual signing capability evidence;
2. `Close`/`CloseTime`/`NewDest` are either reference-equivalent in runtime behavior or truthfully returned to blocked state;
3. no approximate local-socket heuristic remains classified `apply` without equivalence evidence;
4. all matrix/ledger/docs counts mechanically match the chosen disposition;
5. M110/M116 lifecycle/secret/shared-session invariants remain green;
6. no core/Yosemite/crypto scope was smuggled into the pass;
7. broad/focused verification is recorded;
8. closure states whether M122 may proceed independently of Y004 or remains blocked on it.

## 13. Stop conditions

Stop and demote rather than broaden scope if:

- exact idle semantics require router/core instrumentation;
- additional signature algorithms require new cryptographic implementation;
- compatibility would require accepting but ignoring a requested value;
- lifecycle correction requires a global timer/session owner;
- a proxy/TLS/reduction residual becomes entangled with this pass.

## 14. External-interaction boundary

Reference sources are read-only. Writes are internal to `eggstack/emissary` only. No upstream issue, PR, review, submission, release, merge/adoption request, contribution package, or maintainer contact is authorized.

## 15. Closure evidence required

Record semantic evidence table, exact chosen dispositions, changed paths, deterministic lifecycle boundary tests, actual signing-path evidence, matrix deltas and hashes, broad verification, security/containment review, unresolved findings, implementation SHA, and M122 readiness.