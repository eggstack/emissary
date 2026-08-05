# M031 — Control-Plane Runtime Supervisor and Generic Client Backend

Status: implemented

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Applicable governance and decisions:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`

Repository baseline:

- `415213a72be1afecb4925a25112d0f6fabcd1638`

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`, Open, updated `2026-05-20`

## 1. Bounded objective

Make control-plane-created generic `client` tunnel definitions operational
through the existing Emissary client tunnel data plane.

M031 establishes an I2PControl-owned per-name runtime supervisor and replaces
only the `client` unsupported backend with a real backend. It must support
truthful start, stop, restart, inspect, completion, bind failure, SAM failure,
and recovery without adopting startup-managed tasks or changing
`emissary-core`.

M031 does not implement the generic server backend, `StartOnLoad`, missing tunnel
families, AddressBook changes, base compatibility, or broader containment work.

## 2. Readiness and current evidence

M031 is dependency-ready from the M030 partial-support closure.

Retained evidence:

- exact seven-action and twelve-type TunnelManager wire;
- atomic definition store and secret filtering;
- exhaustive backend registry;
- startup-managed inventory and ownership rejection;
- unsupported backend zero-resource behavior;
- bounded `All` dispatch and public status translation.

Current runtime evidence:

- `emissary-cli/src/tunnel/client.rs` contains the canonical generic client data
  plane;
- it currently owns a startup-oriented shared Yosemite session and monolithic
  retrying `JoinSet`;
- it exposes no named cancellation or state handle;
- production I2PControl currently registers an unsupported backend for
  `TunnelType::Client`.

## 3. Required invariants

1. Only `TunnelOwnership::ControlPlane` definitions may enter the runtime
   supervisor.
2. Startup-managed definitions remain externally owned and lifecycle-rejected.
3. Only `client` changes from unsupported to real in M031.
4. Every other tunnel type retains its current backend and zero-resource
   unsupported behavior.
5. Existing startup client manager behavior remains unchanged.
6. No `emissary-core/**` production file changes.
7. No JSON-RPC, persistence, or Proposal 170 type enters the original client
   data-plane module.
8. Start validates required runtime fields before binding or creating a SAM
   session.
9. Stop cancels and awaits only the exact named control-plane task.
10. Restart completes stop before allocating a replacement task.
11. A failed/panicked/completed task releases its runtime slot and permits a
    corrected definition to start without router restart or store deletion.
12. No persistence lock is held across bind, SAM I/O, task spawn, cancellation,
    join, retry sleep, or traffic copy.
13. Runtime state is truthful and never reports running before the listener and
    session path are ready according to the chosen lifecycle contract.
14. No new public fields, statuses, methods, aliases, or tunnel types.
15. No upstream interaction.

## 4. Scope and production file budget

### Primary production scope

Prefer changes inside:

- `emissary-cli/src/i2pcontrol/backends/`;
- `emissary-cli/src/i2pcontrol/production.rs`;
- `emissary-cli/src/i2pcontrol/domain/tunnel.rs` only for internal state helpers
  without public wire changes;
- `emissary-cli/src/i2pcontrol/server.rs` only for composition inputs;
- directly affected I2PControl tests.

### Permitted original CLI seam

`emissary-cli/src/tunnel/client.rs` may change only to expose a
purpose-specific single-client runtime primitive reusable by both startup and
control-plane ownership.

The seam may accept a plain runtime configuration and cancellation/lifecycle
callback. It must not import I2PControl modules or persistence types.

### Conditional composition scope

`emissary-cli/src/main.rs` may pass the existing SAM TCP endpoint and one
I2PControl runtime-supervisor handle into server initialization. Do not
restructure router startup or startup tunnel ownership.

### Hard exclusions

- `emissary-core/**`;
- server tunnel implementation;
- HTTP/SOCKS/IRC/CONNECT/Streamr/bidirectional data planes;
- AddressBook, RouterInfo source, SAM observer, frontend, CI, release, packaging,
  version, fuzz, soak, or unrelated refactors.

If the client backend cannot be implemented without a broader shared service or
core change, stop and record the blocker.

## 5. Target design

### 5.1 Runtime supervisor

Add a bounded control-plane runtime supervisor keyed by exact tunnel name.

Each entry contains only:

- internal lifecycle phase;
- cancellation primitive;
- join/completion handle or equivalent bounded task record;
- sanitized last-failure metadata if required for inspection;
- generation/token needed to prevent stale completion from overwriting a newer
  start.

Use per-name serialization. A global map lock may protect entry lookup and
reservation, but must be released before await-heavy operations.

### 5.2 Client adapter

Extract a single-client runner from the existing startup manager behavior.

Required input mapping:

- `Name` -> diagnostic nickname only;
- `ReachableBy` -> listen interface, default loopback where the existing runtime
  defaults;
- `Port` -> local listen port;
- `TargetDestination` or `Destination` -> required remote I2P destination;
- `TargetPort` -> destination port, default existing behavior;
- supported tunnel/I2CP options only where already consumed by the existing
  generic client runtime.

Fields preserved only for round-trip must not be silently treated as runtime
behavior.

The control-plane instance may own an independent Yosemite streaming session to
make lifecycle independently cancellable. Bound the number of simultaneous
instances by the existing inventory limit.

### 5.3 Start transition

1. Load and clone the durable definition without retaining the store lock.
2. Reject startup ownership or non-client type.
3. Validate runtime-required fields and address/port bounds.
4. Reserve the name in `Starting` state.
5. Spawn/enter the single-instance runner.
6. Publish `Running` only at the documented readiness point.
7. On failure, publish sanitized failed/stopped state and release task resources.

A second start while starting/running returns deterministic invalid-state status.

### 5.4 Stop transition

- absent/stopped is safe and idempotent;
- signal cancellation once;
- await exact task completion with a bounded deadline;
- remove the matching generation only;
- timeout/failure returns a sanitized operation error and does not target any
  other task.

Do not use task abortion as the normal first mechanism if graceful cancellation
can release listener/session resources. A bounded abort fallback may be used
only if documented and tested.

### 5.5 Completion and retry

Preserve the existing behavior that a client tunnel remains available after a
connection completes or a transient data-path error. The reusable runner may
contain its own bounded retry loop, but the supervisor must still be able to
cancel it promptly.

A permanent configuration error must not loop forever. Classify bind/validation
errors as terminal for that start attempt; transient SAM/traffic errors may use
the existing bounded retry delay.

## 6. Ordered work packages

### WP1 — Freeze ownership and failure cases

Add failing tests for:

- control-plane client start currently returns not implemented;
- startup-managed client lifecycle remains rejected;
- duplicate start is rejected;
- stop absent is safe;
- bind failure does not poison future corrected start;
- one client failure does not affect another definition;
- unsupported types remain zero-resource.

Record the expected initial failures in the implementation disposition.

### WP2 — Extract the single-client runtime primitive

Refactor the original client module minimally:

- preserve `ClientTunnelManager::new(...).run()` behavior;
- factor one plain configuration conversion and one cancellation-aware
  single-instance runner;
- retain existing default bind, Yosemite stream, destination-port, traffic-copy,
  retry, and logging semantics unless a direct lifecycle requirement demands a
  bounded adjustment;
- keep the module independent of I2PControl.

Add focused no-feature tests or compile checks proving startup behavior still
builds without the `i2pcontrol` feature.

### WP3 — Implement the supervisor

Create the bounded per-name runtime map, lifecycle reservations, cancellation,
completion cleanup, and inspection path inside I2PControl.

Test stale completion generation handling and lock release around awaits.

### WP4 — Implement and register the real client backend

Add a backend implementing `TunnelBackend` for `TunnelType::Client` and register
it in the production registry. Keep test/fake registry injection available.

Do not alter handler response shapes.

### WP5 — Wire production composition

Pass only the existing runtime inputs required by the client backend. Avoid
passing the Router, SAM owner, EventSubscriber, private keys, or broad config
objects.

### WP6 — Reconcile tests and documentation

Update TunnelManager/support documentation to distinguish:

- generic `client`: real lifecycle backend;
- generic `server`: still unsupported until M032;
- remaining ten types: unsupported;
- startup-managed definitions: externally managed.

Create `plans/closure/i2pcontrol-proposal-170/031-implementation-disposition.md`.

## 7. Failure, cancellation, restart, and contention semantics

- Validation failure allocates no listener/session/task.
- Bind failure releases the name reservation.
- SAM/session failure is sanitized and local to the instance.
- Cancellation while starting prevents later stale readiness publication.
- Cancellation while copying traffic closes the exact listener/session path.
- Response loss after successful start leaves the runtime active; retry observes
  current state and returns deterministic status.
- Restart uses backend stop then start; no overlap is permitted.
- Concurrent lifecycle calls for one name serialize; different names may
  progress concurrently within bounds.
- Store mutex and supervisor map mutex are never held while awaiting the runtime.

## 8. Compatibility and migration

- Existing persisted client definitions require no schema migration.
- A valid existing client definition becomes startable.
- Existing startup client configuration and task behavior remain unchanged.
- `StartOnLoad` remains stored but is not activated until M033.
- Unsupported definitions and response shapes remain unchanged.
- No base I2PControl compatibility behavior changes.

## 9. Security review requirements

Review and test:

- loopback/default bind behavior and explicit non-loopback handling;
- port/address validation;
- no destination/password/custom option leakage in logs/errors;
- bounded task count and retry behavior;
- cancellation cannot target startup or unrelated tasks;
- no arbitrary filesystem path use;
- no lock held across network work;
- no core changes;
- no unsupported backend resource allocation;
- no upstream interaction.

## 10. Focused tests

Required semantic coverage includes:

- `control_plane_client_start_uses_real_backend`;
- `startup_client_lifecycle_remains_external`;
- `client_duplicate_start_is_rejected`;
- `client_stop_is_idempotent`;
- `client_restart_does_not_overlap_generations`;
- `client_bind_failure_releases_runtime_slot`;
- `client_transient_failure_does_not_poison_backend`;
- `client_failure_isolated_from_other_tunnels`;
- `unsupported_backends_remain_resource_free`;
- `no_feature_startup_client_manager_still_builds`.

Use repository naming conventions, but retain each semantic case.

## 11. Verification commands

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features client_tunnel
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol client
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Use targeted formatting and `git diff --check`. Do not add CI, coverage, fuzz,
soak, release, or generated-evidence machinery.

## 12. Documentation and static guards

Update directly affected TunnelManager/support documents and planning status.

Add guards proving:

- production registry has exactly one real `client` backend;
- remaining eleven types have the expected current disposition;
- original client module contains no `crate::i2pcontrol` dependency;
- no `emissary-core/**` changed in the implementation commit;
- startup ownership remains rejected;
- handler contains no listener bind or task spawn.

## 13. Acceptance criteria

M031 may move to closing only when:

- generic control-plane client lifecycle is real and evidenced;
- startup client behavior is unchanged;
- failure/cancellation/recovery semantics pass;
- unsupported types remain truthful;
- all non-I2PControl production changes are minimal and justified;
- no high/medium M031 defect remains;
- the implementation disposition and frozen head are committed;
- no upstream interaction occurred.

M031 does not close the broader roadmap or authorize M032 status changes before
its own closure evidence is accepted.

## 14. Stop conditions

Stop and record `blocked` if:

- safe independent cancellation requires modifying `emissary-core`;
- implementation requires adoption of startup-managed tasks;
- the existing client data plane cannot be reused without duplication;
- a public protocol extension appears necessary;
- runtime fields cannot be validated without exposing secrets or arbitrary
  paths;
- unrelated data-plane or service redesign becomes necessary;
- external Proposal 170 authority changes materially;
- any upstream write/review/submission action is requested without a new
  explicit maintainer directive.
