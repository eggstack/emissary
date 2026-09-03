# M120 — Server Start Preallocation Validation and Secret Transactionality Corrective

Status: **proposed / blocked on M119 closure**

Class: corrective / I2PControl lifecycle / secret transactionality

Baseline for planning: `feafc6a1d9650887015a01f87bf21b57a4e92085`

Corrects the unclosed start-ordering defect identified after M116 and still present after M111-M114:

- `ProductionTunnelManagerControl::start_locked()` calls `prepare_server_definition()` before backend capability validation;
- `prepare_server_definition()` may generate/import and persist private destination material and mutate the stored definition before a later unsupported-option/backend validation failure.

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`.

Applicable authority:

- ADR-0002 control-plane tunnel runtime ownership;
- ADR-0003 tunnel runtime/filter boundary;
- ADR-0004 pinned completion boundary;
- M093 tunnel security;
- M107/M108 secret/TLS hardening;
- M110/M116 destination-key ownership and transactional generation behavior.

## 1. Objective

Make server tunnel start fail **before private destination allocation/import/persistence** for every deterministically invalid/unsupported definition, and make any remaining server-secret mutation transactional across runtime start failure.

All production work must stay under `emissary-cli/src/i2pcontrol/**`. No core/router/Yosemite change is authorized.

## 2. Current defect

Current order is effectively:

```text
load persisted definition
  -> prepare_server_definition()
       -> possibly import/generate private destination
       -> put/replace secret in ServerDestinationStore
       -> persist internal identity into TunnelStore
  -> validate_common_options()
  -> backend.start()
       -> backend-specific raw/I2CP/typed validation
       -> runtime allocation
```

This violates the repeated Proposal 170 fail-before-allocation invariant for unsupported definitions.

For a fresh server definition, an invalid backend option can leave a generated private destination and internal identity persisted even though start fails.

For an existing identity with `PrivKeyFile`, `prepare_server_definition()` can replace the stored private destination before backend validation/runtime success. A later failure can therefore mutate identity material even though the requested start did not succeed.

The prior tests focused rollback on client staging and runtime state; no regression asserted that server option validation precedes secret generation/import or that an existing server secret is restored after failed replacement/start.

## 3. Required architecture

### 3.1 Pure backend preflight

Introduce one non-allocating backend validation capability inside the existing I2PControl backend abstraction. The exact name may be `validate_start`, `preflight`, or equivalent, but its contract must be explicit:

- no listener/session/task allocation;
- no network I/O;
- no private destination generation/import/store mutation;
- no runtime-map reservation;
- validates all deterministic definition-shape/raw-option/I2CP-option/common/typed constraints that `start()` would otherwise reject before resource lookup/allocation;
- `start()` reuses the same validation helpers so preflight and actual start cannot silently drift.

It is acceptable for preflight not to prove dynamic conditions such as port availability, SAM reachability, or secret-store I/O success. Those remain runtime failures and require rollback semantics below.

Do not implement preflight by cloning the entire backend implementation into `production.rs`.

### 3.2 Correct start order

Required conceptual order:

```text
load definition
  -> validate_common_options
  -> backend pure preflight
  -> prepare/stage server identity mutation
  -> backend.start
  -> publish public destination / commit durable mutation
```

Every unsupported Proposal/raw/backend option that can be known without allocation must fail before `prepare_server_definition()` performs generation/import/persistence.

### 3.3 Server secret transaction

After deterministic preflight passes, server preparation may still fail or backend start may fail dynamically. The owner must preserve exact rollback semantics.

For a newly allocated identity:

- failure before successful runtime/public-destination commit removes the new private destination and restores the original persisted definition without the internal identity/public-destination fields;
- no orphan secret remains.

For an existing identity with a requested `PrivKeyFile` replacement:

- retain the previous secret until the new runtime reaches the accepted commit point;
- on any import/start/public-destination persistence failure, restore the previous secret and previous durable definition exactly;
- do not expose old/new private material through errors/logs/debug.

A staged/backup representation may be used inside the I2PControl secret owner. Do not add a second generic secret subsystem or write temporary raw keys outside the existing confined store.

If the simplest safe implementation is to refactor `prepare_server_definition()` into a prepare/commit/rollback object, keep that object private to I2PControl and bounded to one start transaction.

## 4. Authorized production paths

Expected I2PControl-only paths:

- `emissary-cli/src/i2pcontrol/production.rs`;
- `emissary-cli/src/i2pcontrol/backends/mod.rs` for a pure validation trait seam;
- existing backend implementation files only as required to implement/reuse that pure validation seam;
- `emissary-cli/src/i2pcontrol/server_secret_store.rs` only if transactional staging/restore needs an owner-local primitive;
- focused existing I2PControl tests.

No `emissary-core/**`, `emissary-util/**`, `emissary-cli/src/main.rs`, ordinary startup tunnel owner, Cargo/dependency, Yosemite, frontend, workflow, or release production path is authorized.

The implementation must enumerate exact backend files actually changed in closure and update M061/M062 containment before landing them.

## 5. Invariants

M120 MUST preserve:

- only control-plane-owned definitions use this mutation path;
- startup-managed definitions remain config-owned and do not enter the persistent secret transaction;
- all server secret material remains in the existing confined `ServerDestinationStore` ownership boundary;
- private keys never enter RPC responses, RawConfig, logs, debug/display, RouterInfo, or public-destination fields;
- no store/global lock held across network I/O;
- per-name lifecycle serialization remains authoritative;
- successful existing server starts preserve identity stability unless an explicit accepted key-import operation requests replacement;
- failed start leaves runtime state and durable configuration truthful;
- no Proposal-specific validation moves into core/Yosemite;
- no upstream activity.

## 6. Explicit non-goals

M120 does not:

- implement any currently blocked M111/M112/M113 option;
- change M095 counts;
- redesign `ServerDestinationStore` file format unless a minimal backward-compatible transaction primitive requires metadata that can be represented without migration;
- create a router-global transaction manager;
- change local loopback target confinement;
- implement LeaseSet encryption/client auth;
- perform final interoperability reclosure.

## 7. Work packages

### WP1 — freeze every deterministic server validation gate

For all five server families, trace current `start()` validation in exact order and classify each check as deterministic preallocation vs dynamic resource/runtime.

Build a table proving the new pure preflight covers all deterministic checks that could otherwise reject after server-secret preparation.

### WP2 — add/reuse pure preflight

Add the smallest backend abstraction necessary. Refactor backend `start()` functions to call the same pure validation helper rather than maintaining duplicate policy.

Tests must include at least one unsupported raw/backend option for each server-family validation shape that previously could reach preparation.

### WP3 — transactional preparation

Represent newly generated/imported secret changes as a bounded transaction with enough previous-state evidence for exact rollback.

Do not persist the final internal identity/public destination earlier than required. If a temporary durable record is unavoidable for crash consistency, it must be explicitly marked non-active/recoverable and cleanup/recovery behavior must be tested; prefer avoiding such a new format.

### WP4 — runtime failure rollback

Exercise SAM unreachable, backend readiness failure, bind/target failure where applicable, and public-destination persistence failure using test doubles/fixtures. Verify no orphan/replaced secret or mutated durable definition remains.

### WP5 — success commit/restart

Verify successful start commits exactly one intended server identity, preserves it across stop/restart/reload, and public destination remains bound to the committed private identity.

### WP6 — containment/closure

Update M061/M062 exact path authority and produce M120 closure without rewriting prior M110/M116/M113 history.

## 8. Failure, cancellation, restart, contention

The per-name lifecycle lock must cover preflight through transaction commit/rollback so two concurrent starts/edits cannot interleave secret mutations for the same definition.

Do not hold `TunnelStore`, secret-store, backend runtime-map, or other mutex guards across SAM/network I/O.

Cancellation/panic/drop after preparation but before commit must cause the same rollback as an ordinary backend error. If ordinary Rust drop cannot perform async rollback, structure the owning async function so every exit path is explicit and add a guard/state machine that makes a skipped rollback detectable in tests.

Restart/recovery must never treat a failed/uncommitted replacement secret as authoritative.

## 9. Focused tests

Required regression tests include:

- unsupported common option on fresh server fails with zero secret-store mutation and zero destination generation call;
- backend-specific unsupported raw option on fresh server fails before generation/import;
- invalid I2CP/backend option fails before generation/import;
- unsupported server family option never creates `__emissary_server_destination_identity`;
- existing identity + `PrivKeyFile` + later runtime failure restores original private destination exactly;
- fresh generated identity + runtime failure removes secret and restores original stored definition;
- public-destination persistence failure stops runtime and rolls back secret/definition state;
- successful start commits one identity and preserves it across stop/start/reload;
- concurrent same-name starts cannot double-generate/double-commit;
- failures contain no raw key/import value;
- startup-managed server behavior is unchanged.

Use call-count/test-double evidence for 'fail before allocation'; checking only the final absence of a file is insufficient.

## 10. Broad verification

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

Known rustfmt toolchain drift may be dispositioned only as baseline tooling evidence.

## 11. Matrix/documentation rules

M120 changes lifecycle correctness only. M095 remains at the current closed counts unless testing discovers that an already-applied cell cannot preserve fail-before-allocation semantics. Such a finding requires an explicit matrix corrective, not an opportunistic reclassification here.

Update registry, corrective roadmap, M061/M062 containment evidence, M120 closure, and any user-facing TunnelManager documentation that currently promises transactionality contradicted by the old order.

## 12. Acceptance criteria

M120 closes only when:

1. every deterministic server start validation failure occurs before secret generation/import/persistence;
2. dynamic start failure restores exact previous server secret and durable definition state;
3. successful starts commit exactly one intended identity/public destination;
4. no secret/debug/log exposure is introduced;
5. concurrency/cancellation cannot leave an orphan or half-committed server identity;
6. changes remain confined to I2PControl production owners;
7. broad/focused verification passes or baseline-only tooling failures are dispositioned;
8. closure states M121 readiness.

## 13. Stop conditions

Stop rather than broaden scope if:

- fixing validation requires core/router changes;
- a backend cannot expose a pure validation seam without a broad runtime redesign;
- transactional rollback requires changing ordinary startup tunnel ownership;
- a new cryptographic/dependency/store subsystem is proposed;
- completing blocked Proposal options becomes entangled with this correction.

## 14. External-interaction boundary

All external sources are read-only evidence. Repository writes are internal to `eggstack/emissary` only. No upstream issue/PR/review/release/submission/merge/adoption/contact activity is authorized.

## 15. Closure evidence required

Record preflight coverage table, changed paths, allocation call-count regressions, fresh/existing secret rollback evidence, contention/cancellation tests, all verification outcomes, containment/security review, unresolved findings, implementation SHA, and M121 readiness.