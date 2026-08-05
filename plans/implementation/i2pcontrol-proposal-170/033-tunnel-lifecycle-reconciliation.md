# M033 — TunnelManager Lifecycle Reconciliation and StartOnLoad

Status: blocked on M031 and M032

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Applicable governance and decisions:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`

Repository baseline:

- accepted M032 implementation/closure head, to be recorded before execution

Hard dependencies:

- M031 closed
- M032 closed

## 1. Bounded objective

Reconcile the real generic client/server backends with durable definition state
and complete the lifecycle semantics required for an operational TunnelManager
slice.

M033 closes `StartOnLoad`, restart, delete, edit/rename, task completion, failed
start recovery, `All`, and runtime inspection for eligible control-plane
`client` and `server` definitions. It revalidates that the remaining ten tunnel
types stay explicit unsupported backends and that startup-managed definitions
remain externally owned.

M033 does not add new tunnel types or data planes, modify core, alter the public
wire, or reopen AddressBook/RouterInfo work.

## 2. Readiness and retained evidence

M033 is dependency-ready only after M031 and M032 independently close.

Retain:

- M021 wire, validation, atomic definition persistence, and secret filtering;
- M023 startup inventory and ownership rejection;
- M031 per-name supervisor and real client backend;
- M032 real server backend and secret identity store;
- unsupported backend exhaustive registration;
- canonical and compatibility response shapes.

M033 must test the composed behavior rather than redesign the individual data
planes.

## 3. Required invariants

1. Durable definitions and runtime instances have one explicit reconciliation
   policy.
2. Runtime state is never inferred solely from persisted intent.
3. `StartOnLoad` applies only to control-plane-owned `client` and `server`
   definitions.
4. Unsupported and startup-managed definitions never auto-start.
5. One failed `StartOnLoad` definition does not block I2PControl startup or
   unrelated eligible tunnels unless a shared invariant is corrupt.
6. Start/restart never overlap two generations of the same name.
7. Stop and delete target only the exact named control-plane task.
8. Delete cannot remove a definition while its task remains active.
9. Edit/rename cannot create a durable/runtime split.
10. Running definitions may be edited only under an explicit safe policy.
11. `All` remains bounded, deterministic, and excludes startup-managed
    definitions.
12. Task completion/panic/failure releases supervisor state and does not require
    router restart or store reset.
13. Server destination identity remains coherent through all lifecycle paths.
14. Unsupported types remain resource-free and never report running.
15. No public fields/statuses/actions are added.
16. No `emissary-core/**` production changes.
17. No upstream interaction.

## 4. Scope and file budget

Primary production work remains in:

- `emissary-cli/src/i2pcontrol/backends/**`;
- `emissary-cli/src/i2pcontrol/production.rs`;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs` only where operation ordering
  must be corrected without wire changes;
- `emissary-cli/src/i2pcontrol/stores/**` for exact transactional support;
- directly affected tests and documentation.

`emissary-cli/src/main.rs` may change only to trigger the bounded post-load
reconciliation/start sequence. The original client/server runtime modules should
not require new behavior in M033 unless a direct M031/M032 defect is discovered.

Prohibited:

- core, router, SAM protocol, transport, LeaseSet, frontend, missing data-plane,
  AddressBook, RouterInfo source, CI/release, or broad persistence refactors.

## 5. Target reconciliation model

### 5.1 Startup sequence

After definition and secret stores load successfully and the runtime supervisor
is available:

1. list persisted control-plane definitions once;
2. validate the bounded inventory and duplicate/name invariants;
3. select only `client`/`server` definitions with `StartOnLoad = true`;
4. start them through the same backend path used by the RPC method;
5. record each sanitized result independently;
6. expose the service even if one definition fails, unless the store/supervisor
   invariant itself is invalid.

Do not hold initialization/store locks across starts. Do not auto-start
unsupported or startup-managed definitions.

### 5.2 Runtime state source

The supervisor/backend is authoritative for runtime state. Persisted definitions
retain configuration and intent only. `get` translates current backend
inspection plus durable options into the existing Proposal 170 response shape.

A stopped definition with `StartOnLoad = true` may exist after a failed start;
it must not be reported running.

### 5.3 Edit policy

Default policy:

- stopped eligible definition: edit/rename allowed with existing atomic store
  behavior and server-secret coordination;
- starting/running/stopping eligible definition: reject edit/rename with a
  deterministic operation error;
- startup-managed: reject;
- unsupported stopped definition: retain existing administrative edit behavior.

Do not implement implicit live reconfiguration. A caller may stop, edit, and
start explicitly.

### 5.4 Delete policy

- stopped eligible or unsupported definition: existing atomic delete behavior;
- running/starting/stopping eligible definition: perform bounded stop first,
  then delete definition and associated server secret under the M032 policy;
- stop failure: preserve definition and secret;
- startup-managed: reject.

### 5.5 Restart policy

Restart is exact stop completion followed by start using the latest durable
definition. It must not reuse stale pre-edit configuration or permit generation
overlap.

### 5.6 All operations

For canonical `All` start/stop/restart:

- snapshot the bounded eligible control-plane inventory;
- deterministic name order;
- skip startup-managed definitions;
- unsupported definitions return their explicit per-item error status without
  resource allocation;
- continue after individual failure and aggregate exact results;
- do not hold the store lock during dispatch.

## 6. Ordered work packages

### WP1 — Freeze lifecycle discrepancies

Add failing tests for:

- `StartOnLoad` currently stored but not executed;
- failed start incorrectly blocking or poisoning later recovery;
- running edit/rename ambiguity;
- delete while running;
- restart generation overlap;
- task completion cleanup;
- mixed `All` eligible/unsupported/startup inventory.

### WP2 — Implement post-load reconciliation

Add one bounded initialization call in I2PControl composition. Reuse backend
start and supervisor semantics; do not create a second startup path.

### WP3 — Make runtime inspection authoritative

Ensure `get`, ClientServicesInfo, and applicable quick-info serializers use the
same current backend/supervisor state and actual addresses. Avoid stale
persisted `runtime_state` claims.

### WP4 — Reconcile edit/rename/delete ordering

Implement the stopped-only edit policy and stop-before-delete transaction.
Coordinate server secrets without holding locks across stop.

### WP5 — Reconcile restart and All

Add exact per-name generation protection and deterministic bounded aggregation.

### WP6 — Failure recovery and cleanup

Ensure task completion, panic, bind/SAM failure, cancellation timeout, and stale
completion remove or preserve state according to the explicit model. A corrected
configuration must start without manual database deletion.

### WP7 — Documentation and disposition

Update TunnelManager, support, security, and conformance documents. Create:

- `plans/closure/i2pcontrol-proposal-170/033-implementation-disposition.md`.

## 7. Failure, cancellation, restart, and contention semantics

- Initialization validates stores before starts.
- Individual `StartOnLoad` failure is isolated and sanitized.
- Cancellation after durable definition publication but before start completion
  leaves a stopped/failed definition, not a phantom running state.
- Stop timeout preserves the definition and runtime record for further
  inspection/retry; it does not silently delete.
- Restart uses a new generation token only after prior completion.
- Stale task completion cannot overwrite newer state.
- Per-name lifecycle lock serializes start/stop/restart/edit/delete.
- Store and secret-store locks are not held across runtime awaits.
- Unrelated names may progress concurrently within global bounds.

## 8. Compatibility and migration

- Existing persisted definitions remain readable.
- `StartOnLoad` begins to have real effect only for eligible control-plane
  client/server definitions; document this behavioral activation.
- Unsupported definitions preserve current CRUD and not-implemented lifecycle.
- Startup configuration remains authoritative and unchanged.
- No public wire or persistence schema migration unless the internal runtime
  generation metadata strictly requires a backward-compatible default.

## 9. Security review requirements

Review and test:

- no auto-start of unsupported/startup definitions;
- bounded startup and `All` target count;
- no secret leakage in aggregated failures;
- server identity preservation/deletion ordering;
- exact cancellation target and stale-generation protection;
- no lock across network work;
- no core changes;
- no upstream interaction.

## 10. Focused tests

Required semantics include:

- `start_on_load_starts_eligible_client_and_server`;
- `start_on_load_skips_unsupported_and_startup_managed`;
- `failed_start_on_load_does_not_block_other_tunnels`;
- `running_edit_is_rejected_without_mutation`;
- `running_delete_stops_before_store_delete`;
- `restart_uses_latest_durable_definition`;
- `stale_completion_cannot_overwrite_new_generation`;
- `task_failure_allows_corrected_restart_without_store_reset`;
- `all_lifecycle_is_bounded_deterministic_and_truthful`;
- `runtime_inspection_does_not_use_persisted_intent_as_state`.

## 11. Verification commands

```bash
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol start_on_load
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Run targeted formatting and `git diff --check`. No CI/release/fuzz/soak
expansion.

## 12. Documentation and static guards

Add guards proving:

- exactly two real production backends and ten unsupported backends;
- only control-plane client/server definitions are auto-start eligible;
- handler/store code does not spawn data-plane tasks directly;
- startup-managed mutation/lifecycle rejection remains;
- no core changes;
- public action/type/status inventories remain exact.

## 13. Acceptance criteria

M033 may close only when:

- eligible lifecycle and `StartOnLoad` are operational and coherent;
- edit/delete/restart/All/failure behavior is deterministic;
- runtime inspection is truthful;
- unsupported/startup boundaries remain exact;
- no high/medium M033 defect remains;
- implementation disposition and frozen head are committed;
- all non-I2PControl changes are justified;
- no upstream interaction occurred.

## 14. Stop conditions

Stop and record `blocked` if:

- reconciliation requires taking ownership of startup tasks;
- safe lifecycle requires a core/router redesign;
- definition and server-secret transactions cannot remain coherent within the
  existing bounded stores;
- implicit live edit appears necessary;
- public wire extensions are required;
- missing data-plane implementation is needed;
- Proposal 170 changes materially;
- upstream action is requested without explicit authorization.