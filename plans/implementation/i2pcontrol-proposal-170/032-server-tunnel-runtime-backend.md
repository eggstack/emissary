# M032 — Generic Server Backend and Persistent Destination Identity

Status: ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Applicable governance and decisions:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`

Repository baseline:

- `8f635616c174e8681ba86de79f80ca3fff2cccee` — M031 implementation head

Hard dependency:

- M031 closed

## 1. Bounded objective

Make control-plane-created generic `server` tunnel definitions operational
through the existing Emissary server tunnel data plane.

M032 replaces only the `server` unsupported backend with a real backend and adds
a backend-owned, path-confined persistent destination identity store. It reuses
the M031 runtime supervisor and the existing Yosemite persistent server-session
behavior.

M032 does not implement HTTP, bidirectional HTTP, IRC, Streamr, or other server
types; it does not accept `PrivKeyFile`; it does not adopt startup-managed server
tasks; and it does not modify `emissary-core`.

## 2. Readiness and current evidence

M032 is dependency-ready now that M031 has closed the supervisor and client
backend boundary at the recorded implementation head.

Current server runtime:

- `emissary-cli/src/tunnel/server.rs` loads or generates a persistent
  destination, creates one Yosemite streaming session per startup tunnel,
  publishes the actual destination through an optional callback, issues
  `STREAM FORWARD`, and then remains alive;
- the startup manager spawns tasks but retains no named cancellation or join
  handle;
- destination paths come from startup configuration and are not suitable as
  control-plane request input;
- I2PControl currently registers an unsupported backend for
  `TunnelType::Server`.

Retain M031 supervisor semantics rather than creating a second runtime registry.

## 3. Required invariants

1. Only control-plane-owned generic `server` definitions may use the real
   backend.
2. Startup-managed server definitions remain externally owned, read-only, and
   lifecycle-rejected.
3. Only `server` changes backend disposition in M032.
4. No arbitrary request-selected filesystem path is accepted.
5. `PrivKeyFile` remains rejected.
6. Server destination private material is never stored in generic `rawConfig`,
   returned through `get`, logged, or included in errors.
7. Destination identity survives stop/restart and router restart.
8. Definition rename preserves identity or fails atomically without split state.
9. Definition delete cannot leave a running task or orphan an uncontrolled
   secret; deletion semantics are explicit and tested.
10. Existing startup server behavior and destination files remain unchanged.
11. No `emissary-core/**` production file changes.
12. No public Proposal 170 extension fields/statuses.
13. No persistence/supervisor lock is held across key generation, filesystem
   sync, SAM I/O, cancellation, or join.
14. Unsupported server families remain resource-free and inactive.
15. No upstream interaction.

## 4. Scope and file budget

### Primary production scope

- `emissary-cli/src/i2pcontrol/backends/`;
- I2PControl-owned secret/persistence modules under
  `emissary-cli/src/i2pcontrol/`;
- `emissary-cli/src/i2pcontrol/production.rs`;
- directly affected TunnelManager/domain/store tests.

### Permitted original CLI seam

`emissary-cli/src/tunnel/server.rs` may expose a purpose-specific
cancellation-aware single-server runtime primitive. It must remain independent
of I2PControl, JSON-RPC, TunnelStore, and Proposal 170 DTOs.

### Conditional composition scope

`emissary-cli/src/main.rs` may pass the existing SAM endpoint, I2PControl base
state path, and supervisor inputs already established by M031. No new broad
runtime object may cross the seam.

### Hard exclusions

- `emissary-core/**`;
- startup server task adoption;
- arbitrary destination/key paths;
- missing server data planes;
- general key-management service, HSM support, encrypted vault redesign, or
  repository-wide secret framework;
- AddressBook, RouterInfo source, auth, frontend, CI/release, or unrelated work.

## 5. Target design

### 5.1 Server runtime adapter

Extract a reusable single-server runner preserving the existing behavior:

- create a Yosemite streaming session with a persistent destination;
- use the configured SAM TCP endpoint;
- map `TargetHost`/`Host` and `TargetPort` according to the existing generic
  server semantics;
- issue forward and retry transient failure with the existing bounded policy;
- publish actual destination metadata through a narrow callback;
- terminate promptly and cleanly on cancellation.

The adapter accepts plain runtime configuration and private destination contents
from the backend; it does not open arbitrary paths.

### 5.2 Backend-owned secret store

Use a fixed directory beneath the existing I2PControl state root, for example a
purpose-specific `server-destinations/` directory. The exact path is internal
and must not be request-controlled.

Requirements:

- path confinement and safe filename derivation;
- no raw tunnel name as an unchecked path component;
- restrictive file permissions where supported;
- temp/write/sync/rename publication with current/backup or equivalent recovery;
- bounded file size and destination validation;
- secret omission from debug/display/serialization;
- deterministic lookup by a stable internal identity.

If the current tunnel-definition schema lacks a stable internal identity,
introduce the smallest internal persistence field or companion mapping required.
It must not change the public JSON response or accept a client-selected path.

### 5.3 Create/start behavior

Creating a server definition does not need to allocate the destination unless
Proposal 170 response semantics require persistent-key storage information.
Choose and document one deterministic point:

- create-time identity allocation committed atomically with the definition; or
- first-start allocation committed before reporting start success.

Whichever point is selected, a crash or failure cannot leave a definition
claiming an identity that cannot be recovered.

### 5.4 Rename behavior

A stopped control-plane server rename must preserve its destination identity.
Definition and secret mapping rename/publication must be atomic enough that
recovery sees either the old complete name/mapping or the new complete
name/mapping.

A running server rename is rejected in M032 unless M033 later establishes an
explicit stop/rename/start transaction. Do not silently run under the old name
while persisting the new one.

### 5.5 Delete behavior

Delete of a running server must stop and await the exact task before deleting
durable definition/secret state, or fail without deleting either. Decide whether
private destination identity is deleted with the definition or retained as a
bounded recovery artifact; the plan default is delete with the definition after
successful stop, with no public path disclosure.

### 5.6 Inspection

`get` and ClientServicesInfo must use actual destination metadata only when the
backend has a real validated destination. Never fabricate an address or expose
the private destination.

## 6. Ordered work packages

### WP1 — Freeze secret and lifecycle defects

Add failing tests for:

- generic server start currently returns not implemented;
- arbitrary `PrivKeyFile` remains rejected;
- startup server lifecycle remains external;
- generated identity survives stop/restart;
- rename preserves identity;
- delete cannot orphan a running task;
- bind/SAM/forward failure releases the runtime slot;
- private destination never appears in response/log/debug output.

### WP2 — Extract the single-server runtime primitive

Refactor the original server module minimally while preserving startup behavior.
Add cancellation and readiness/failure callbacks without importing I2PControl.

Prove the no-feature startup path still builds and focused server tests pass.

### WP3 — Implement the secret identity store

Add bounded typed storage, recovery, validation, permission handling, and atomic
publication under I2PControl ownership.

Test corrupt current/valid backup, failed publication, unsafe path/symlink,
rename, and delete behavior.

### WP4 — Implement and register the real server backend

Use the M031 supervisor and register exactly one real backend for
`TunnelType::Server`. Preserve unsupported registration for the other ten types.

### WP5 — Reconcile create/edit/delete transactions

Coordinate definition persistence and secret mapping with explicit order and
rollback/fail-closed semantics. Do not hold store locks across runtime stop/start.

### WP6 — Documentation and disposition

Update TunnelManager/support/security documentation and create:

- `plans/closure/i2pcontrol-proposal-170/032-implementation-disposition.md`.

## 7. Failure, cancellation, restart, and contention semantics

- Key generation failure allocates no active task and publishes no incomplete
  mapping.
- Secret persistence failure leaves the prior definition/identity coherent.
- SAM/session/forward failure is local and sanitized.
- Stop cancellation closes only the target server task.
- Restart reuses the same validated destination identity.
- Rename/delete serialize with lifecycle for the same name.
- A stale completion from the pre-rename generation cannot overwrite the new
  state.
- Different server names may operate concurrently within global bounds.
- Secret-store and TunnelStore locks are not held across runtime awaits.

## 8. Compatibility and migration

- Existing control-plane server definitions remain readable.
- Existing startup destination paths/files remain untouched.
- A control-plane server receives an internal backend-owned identity without a
  public schema change.
- Existing unsupported server-family definitions remain unsupported.
- `StartOnLoad` remains deferred to M033.
- `PrivKeyFile` remains a documented security deviation from the proposal input
  inventory.

## 9. Security review requirements

Review and test:

- path confinement, symlink/irregular-file handling, permissions, and bounds;
- private destination redaction in all output channels;
- no client-selected path or filename;
- no secret material in raw configuration or generic store snapshots;
- exact task cancellation and startup ownership isolation;
- no key reuse across unrelated definitions unless explicitly preserved by
  rename;
- failed delete/rename leaves recoverable coherent state;
- no core changes and no unsupported resource allocation;
- no upstream interaction.

## 10. Focused tests

Required semantic coverage includes:

- `control_plane_server_start_uses_real_backend`;
- `startup_server_lifecycle_remains_external`;
- `server_identity_survives_stop_restart`;
- `server_identity_survives_definition_rename`;
- `server_delete_stops_before_secret_removal`;
- `server_secret_store_recovers_backup`;
- `server_secret_path_is_confined`;
- `server_private_destination_is_never_serialized`;
- `server_failure_releases_runtime_slot`;
- `unsupported_server_types_remain_resource_free`.

## 11. Verification commands

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features server_tunnel
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol server
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Use targeted formatting and `git diff --check`. No remote CI, fuzz, soak,
release, packaging, or generated evidence expansion.

## 12. Documentation and static guards

Add guards proving:

- production registry has real `client` and `server` backends only;
- original server module has no I2PControl dependency;
- private destination fields are excluded from serialization/debug;
- no arbitrary path input reaches the secret store;
- no `emissary-core/**` production changes;
- startup ownership remains rejected.

## 13. Acceptance criteria

M032 may move to closing only when:

- generic server lifecycle is real and truthful;
- destination identity persistence/rename/delete/recovery is evidenced;
- startup behavior is unchanged;
- secret material is confined and redacted;
- no high/medium M032 defect remains;
- every non-I2PControl production change is minimal and justified;
- implementation disposition and frozen head are committed;
- no upstream interaction occurred.

## 14. Stop conditions

Stop and record `blocked` if:

- safe server cancellation requires a core change;
- identity cannot be made durable without arbitrary path exposure or a broad key
  management redesign;
- definition/identity rename cannot be made coherent within the existing store
  boundary;
- startup task adoption becomes necessary;
- missing server data-plane work is required;
- public protocol extensions appear necessary;
- external authority changes materially;
- upstream activity is requested without explicit new authorization.
