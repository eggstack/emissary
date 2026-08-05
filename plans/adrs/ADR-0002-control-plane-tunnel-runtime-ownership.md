# ADR-0002: Control-Plane Tunnel Runtime Ownership and Minimal Adapter Boundary

Status: accepted

Date: 2026-08-05

Decision owners: project maintainers

Related governance and decisions:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`
- status: `Open`
- created and last updated: `2026-05-20`

## Context

ADR-0001 established a contract-complete TunnelManager API with exhaustive real
or explicitly unsupported backends. The current production registry maps every
Proposal 170 tunnel type to an unsupported backend. Tunnel definitions are
validated, persisted, edited, retrieved, and deleted, but no control-plane
created tunnel can start.

Emissary already contains generic client and server tunnel data planes in
`emissary-cli/src/tunnel/client.rs` and
`emissary-cli/src/tunnel/server.rs`. Those managers are startup-oriented:

- the client manager owns one shared SAM session and a monolithic retrying
  `JoinSet` for all startup definitions;
- the server manager spawns independent long-running tasks but retains no named
  cancellation or inspection handles;
- neither manager exposes safe per-name start, stop, restart, or adoption;
- startup configuration remains authoritative for those tasks.

Directly attaching I2PControl to those manager-owned task sets would create
ambiguous ownership and unsafe cancellation. Reimplementing the data planes in
I2PControl would duplicate networking and destination behavior. Moving the
runtime into `emissary-core` would contaminate the security-audited router core
for an administrative API concern.

## Decision drivers

- Make Proposal 170 TunnelManager operational for capabilities Emissary already
  has.
- Keep public Proposal 170 wire and persistence unchanged.
- Keep startup-managed tunnels externally owned and read-only.
- Avoid modifying `emissary-core/**`.
- Avoid implementing missing HTTP, IRC, SOCKS-IRC, CONNECT, Streamr,
  bidirectional, or other tunnel data planes.
- Reuse existing client/server behavior through narrow purpose-specific
  adapters rather than duplication.
- Make cancellation, restart, state inspection, and failure isolation explicit.
- Keep changes outside `emissary-cli/src/i2pcontrol/**` minimal and reviewable.

## Considered options

### Option A — Control the existing startup managers directly

I2PControl would retain handles into the current startup manager task sets.

Rejected because the managers do not expose stable per-name ownership,
cancellation, or state. Retrofitting those task sets into a shared authority
would couple startup configuration and administrative persistence and make
stop/delete behavior unsafe.

### Option B — Move tunnel supervision into `emissary-core`

Rejected because generic CLI tunnels are not router-core protocol machinery.
This would broaden the audited core surface and violate the requested
containment boundary.

### Option C — Duplicate client/server data planes inside I2PControl

Rejected because it would create parallel implementations of listener,
SAM-session, forwarding, retry, and destination behavior with divergent fixes.

### Option D — Extract narrow single-instance runners and let I2PControl own a
separate control-plane supervisor

Accepted.

The existing CLI tunnel modules retain the data-plane implementation. They may
expose purpose-specific single-instance runners or factories with explicit
configuration, cancellation, and lifecycle reporting. The I2PControl subsystem
owns only control-plane-created task supervision and backend state.

## Decision

### Runtime eligibility

The first real backends are limited to:

- `client` — using the existing generic streaming client tunnel behavior;
- `server` — using the existing generic streaming server tunnel behavior.

All other Proposal 170 tunnel types remain exhaustive unsupported backends under
ADR-0001. The existence of an HTTP or SOCKS startup service does not by itself
make the corresponding Proposal 170 I2PTunnel type eligible: their configuration,
identity, ownership, and lifecycle semantics differ and require separate plans.

### Ownership split

- Startup-configured tunnels remain `StartupManaged`, externally owned, and
  read-only through I2PControl.
- Control-plane-created definitions remain `ControlPlane` and are the only
  definitions a real backend may start, stop, restart, or delete.
- A control-plane runtime supervisor is keyed by canonical tunnel name and owns
  cancellation and task completion for those instances only.
- No task is adopted from startup configuration and no startup task is
  cancelled through the control plane.

### Minimal shared adapter seam

Permitted non-I2PControl production changes are limited to:

- `emissary-cli/src/tunnel/client.rs` — expose a bounded single-client runtime
  primitive while preserving existing startup behavior;
- `emissary-cli/src/tunnel/server.rs` — expose a bounded single-server runtime
  primitive and backend-owned destination callback/identity seam while
  preserving startup behavior;
- `emissary-cli/src/main.rs` — pass only the existing SAM endpoint/base path and
  one control-plane runtime handle into I2PControl composition;
- directly affected configuration conversion helpers when unavoidable.

These modules must not depend on JSON-RPC, Proposal 170 DTOs, the TunnelStore,
or I2PControl handler types. They expose runtime primitives, not administrative
policy.

No `emissary-core/**` file is authorized for generic client/server backend work.

### Supervisor semantics

The I2PControl-owned supervisor must:

- serialize lifecycle transitions per tunnel name;
- reject duplicate start while starting or running;
- make stop idempotent for absent/stopped instances;
- cancel and await the exact named task before reporting stopped;
- implement restart as completed stop followed by a new start;
- publish truthful `starting`, `running`, `stopping`, `stopped`, `failed`, or
  unsupported internal state without adding public protocol fields;
- remove completed task handles and permit recovery without router restart;
- isolate one tunnel failure from the server and other tunnels;
- bound retained instances and completion records;
- never hold a persistence lock across listener bind, SAM I/O, cancellation, or
  task join.

### Client runtime choice

Independent lifecycle requires an independently cancellable runtime instance.
A control-plane client tunnel may therefore own its own SAM streaming session
rather than sharing the startup manager's session. This resource difference is
accepted only for control-plane-created definitions and must be bounded by the
existing maximum tunnel inventory.

### Server destination ownership

The control plane must not accept arbitrary private-key filesystem paths.
Server destinations are generated or loaded through a backend-owned,
path-confined secret store under the I2PControl state directory. Secret identity
must survive stop/restart and definition rename without appearing in generic
responses, logs, or raw configuration. Creation, rename, deletion, and recovery
must be atomic with the definition operation or fail without contradictory
state.

## Consequences

### Positive

- Generic client and server TunnelManager lifecycle becomes real without public
  API redesign.
- Startup and control-plane ownership remain unambiguous.
- Existing data-plane implementations remain canonical.
- The audited router core is not modified.
- Unsupported tunnel families remain truthful and resource-free.
- Future real backends remain local registry replacements.

### Negative

- The CLI tunnel modules gain small reusable runtime seams.
- Control-plane client tunnels use independent sessions and may consume more
  resources than startup tunnels.
- Server destination identity requires a private backend-owned secret store and
  careful rename/delete semantics.
- Full Proposal 170 runtime support remains partial because unsupported tunnel
  families stay out of scope.

## Compatibility and migration

- Existing startup configuration and task behavior remain unchanged.
- Existing persisted control-plane definitions remain readable.
- Definitions of type `client` and `server` become startable after the real
  backends land; no public schema migration is required.
- Persisted unsupported definitions remain unsupported.
- `StartOnLoad` applies only to control-plane-owned definitions after the
  lifecycle-reconciliation milestone explicitly enables restart behavior.
- No startup definition is silently copied, adopted, or converted.

## Security and reliability implications

- Backend start validates all runtime-required fields before allocating a task.
- Listener addresses and ports are bounded and collision failures remain local.
- Server key material is path-confined, permission-restricted where supported,
  durable, and omitted from diagnostics and API responses.
- Cancellation targets one exact task and cannot affect startup-managed or
  unrelated tunnels.
- A failed backend must not poison the registry or require database deletion or
  router restart before a corrected definition can start.
- Unsupported backends retain zero-resource behavior.
- No arbitrary filesystem path from `PrivKeyFile`, `FilterFilePath`, or raw
  configuration crosses into runtime ownership.

## Verification

Implementation evidence must prove:

1. the default registry contains real `client` and `server` backends and
   unsupported backends for the remaining ten types;
2. startup-managed definitions remain mutation- and lifecycle-rejected;
3. a control-plane client starts, accepts traffic through the existing runtime
   primitive, stops, and starts again;
4. a control-plane server starts, retains destination identity, stops, and
   starts again;
5. concurrent start/stop/restart is deterministic per name;
6. task panic, bind failure, SAM failure, and cancellation affect only the
   target tunnel;
7. delete cannot orphan a running task or secret identity;
8. restart and `StartOnLoad` behavior are explicit and tested;
9. no `emissary-core/**` production file changes;
10. every non-I2PControl production change is individually justified;
11. no unsupported type opens a resource or reports running;
12. no upstream interaction occurs.

## Supersession

None. ADR-0001 remains controlling for exhaustive unsupported behavior; this ADR
defines when and how an existing capability may replace a stub.