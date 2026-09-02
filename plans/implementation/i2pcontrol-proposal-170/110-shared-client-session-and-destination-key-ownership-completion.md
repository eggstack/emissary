# M110 — Shared Client Session and Destination-Key Ownership Completion

Status: **closed** — closure: `plans/closure/i2pcontrol-proposal-170/110-closure.md`

Class: capability / lifecycle / secret ownership

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Source evidence:

- M097 closure: `plans/closure/i2pcontrol-proposal-170/097-closure.md`
- M104 closure: `plans/closure/i2pcontrol-proposal-170/104-closure.md`
- M105 audit: `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`
- M109 plan/closure when available.

Pinned authority: I2P Proposal 170 revision `2026-05-20`, status Open.

All external evidence is read-only. Repository writes are internal to `eggstack/emissary` only.

## 1. Objective

Implement the Proposal 170 client identity/session ownership cells that cannot be made truthful by raw option persistence:

- `Shared` — 7 client-family cells;
- `NewDest` and `PersistentClientKey` — 14 client-family cells;
- `PrivKeyFile` — 10 applicable client/server-family cells in the M105 ledger.

The target is an I2PControl-owned, bounded, generation-safe session/key owner that uses existing Yosemite public session primitives and confined I2PControl secret storage. The plan must not add a router-global destination/key subsystem merely for Proposal 170.

M110 is intentionally separate from SAM session-wire options (`UseSSL`, tunnel variance/backups, `SigType`, `CustomOptions`) because those require a different dependency capability and are owned by M111.

## 2. Readiness gates

M105 classifies these cells as architecture/ownership blocked rather than simple parser gaps. The following readiness gates are now satisfied by the M115 closure and registry reconciliation:

1. M109 and M115 are closed and startup lifecycle ownership is stable;
2. the M115 closure explicitly accepts a bounded I2PControl-local shared-session and client-secret owner rather than a router-global owner;
3. accepted Yosemite 0.7.0 public APIs expose `DestinationKind::Persistent` and `SessionOptions`, sufficient to create a client streaming session from generated/imported persistent destination material without a dependency fork/vendor or parallel SAM stack;
4. exact applicable cells are frozen from the current M095/M105 artifacts.

If item 3 is false, split the dependency-blocked portion rather than changing Yosemite under this plan.

## 3. Required ownership model

Preferred ownership is entirely under `emissary-cli/src/i2pcontrol/**`:

```text
TunnelDefinition
      |
      v
validated identity/session policy
      |
      +--> ClientSecretStore (confined, atomic, owner-only)
      |
      +--> SharedClientSessionRegistry
                |
                v
        existing Yosemite Session<style::Stream>
```

The shared-session registry is a control-plane/application owner, not a router protocol subsystem. It may share sessions only among explicitly compatible I2PControl-owned client definitions.

Compatibility keys must include every option that changes session identity/security/transport semantics. At minimum evaluate encryption/signing identity material, inbound/outbound tunnel settings already supported, target publication behavior, and any later M111 session-wire fields. Unknown or blocked fields make definitions incompatible rather than being ignored.

## 4. Invariants

M110 MUST preserve:

- no secret material in `Debug`, logs, RPC errors, `raw_config`, RouterInfo, or ClientServicesInfo;
- private key files/state owner-only on Unix and fail-closed on unsafe file types;
- one exact owner for each persisted client private destination;
- deterministic session compatibility; never share across incompatible security/session settings;
- bounded number of shared sessions and bounded subscribers per session by the existing tunnel inventory ceiling;
- reference-counted or lease-based teardown only after the last compatible owner releases a shared session;
- edit/restart/delete generation isolation;
- no lock across Yosemite/network I/O, sleeps, cancellation waits, or filesystem sync;
- failed edit/start preserves the prior running/durable generation;
- `NewDest` never silently reuses an old identity when a new identity is required;
- `PersistentClientKey` never silently rotates an identity that is required to persist;
- `PrivKeyFile` cannot escape a confined administrative root and cannot make arbitrary host files part of router state;
- no feature-disabled/default behavior change;
- M093 server anonymity/secret boundaries remain intact.

## 5. Explicit non-goals

M110 MUST NOT:

- implement M111 session-wire fields;
- implement proxy/plugin/lifecycle residuals owned by M112;
- implement server presentation/LeaseSet residuals owned by M113;
- add a router-global keyring or destination service;
- rewrite startup configuration;
- accept arbitrary absolute/traversal `PrivKeyFile` paths;
- vendor/fork/patch Yosemite or implement SAM manually;
- add Cargo dependencies solely for parity;
- modify `emissary-core`, `emissary-util`, frontend, workflow, or release paths;
- weaken private-key permissions or symlink/special-file checks;
- prepare upstream submission/review/contact.

## 6. Expected production paths

Preferred paths:

- `emissary-cli/src/i2pcontrol/domain/tunnel.rs` — typed option semantics only;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs` — validation/action integration;
- `emissary-cli/src/i2pcontrol/backends/options.rs` — capability validation;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` and/or a new narrowly named runtime owner under the same directory;
- a new secret store under `emissary-cli/src/i2pcontrol/**` if required;
- affected I2PControl client backends only;
- M095/M105 matrix/audit artifacts and focused tests after real runtime evidence exists.

No non-I2PControl production path is authorized by this proposed plan. If a neutral existing-owner seam becomes necessary, stop and amend planning with exact paths before implementation.

## 7. Work packages

### WP1 — Freeze exact cell inventory and Yosemite capability

Re-read M095/M105 at the implementation baseline and enumerate exactly the 31 targeted cells. Verify the current Yosemite public session API can consume generated/imported destination material for every applicable client use case. Record the exact API/type used; no private cargo-registry path may become production coupling.

If the required primitive is absent, mark affected cells dependency-blocked and stop that sub-slice.

### WP2 — Confined client secret store

Add a bounded I2PControl-owned store only if persistent client keys are required.

Requirements:

- state beneath the existing I2PControl/tunnel administrative root;
- canonical name-to-secret mapping with path-safe derived filenames, not request-provided arbitrary paths;
- atomic same-directory write/rename and prior-generation recovery consistent with existing stores;
- owner-only private material on Unix before/at creation;
- symlink/non-regular paths fail closed;
- bounded secret size and structural parsing before activation;
- secret values never appear in serialized TunnelDefinition/raw config.

### WP3 — `NewDest` / `PersistentClientKey`

Define exact state transitions for create/start/edit/restart:

- `NewDest=true` generates a new destination only at the contract-defined transition and commits it transactionally with the generation that will use it;
- persistent identity reuses the same validated secret across restart when required;
- failed session allocation after key generation must not ambiguously publish a half-applied generation;
- edit from persistent→new or new→persistent must have explicit identity consequences and rollback semantics.

### WP4 — `PrivKeyFile`

Treat the request value as an administrative import reference, not an unconstrained filesystem capability.

- resolve beneath one documented I2PControl-controlled import root;
- reject absolute paths, `..`, backslash/control abuse, symlink components, special files, and oversized input;
- parse/validate private destination material before copying it into the I2PControl secret owner;
- after successful import, runtime must depend on the owned copy, not a mutable external file;
- RPC get/logging may return only the configured safe reference if the contract requires it, never key bytes.

If literal Proposal/reference semantics require arbitrary host paths, closure must record the security-preserving divergence rather than relaxing confinement silently.

### WP5 — `Shared`

Implement a bounded `SharedClientSessionRegistry` under I2PControl.

- derive a deterministic compatibility key from all session-affecting settings;
- on `Shared=true`, acquire an existing compatible session or create one;
- on `Shared=false`, allocate an independent session even if settings match;
- maintain bounded subscriber/name membership and generation IDs;
- release membership on stop/delete/edit/restart cancellation;
- tear down the Yosemite session only when the final member releases it;
- do not permit one member's edit/restart to mutate the session contract for other members;
- start failure for a new member must not tear down a healthy shared session used elsewhere.

### WP6 — Matrix and wire truthfulness

Move a cell from `blocked_primitive` to `apply` only after a canonical request demonstrably changes runtime identity/session behavior. Parser acceptance, persisted flags, generated-but-unused keys, or a registry that is never consumed are not support.

## 8. Failure, cancellation, restart, contention

Key generation/import, secret publication, session allocation and runtime publication must form an ordered transaction with explicit rollback. No successful RPC result may precede the durable/active point promised by the operation.

Shared-session membership changes must be serialized without holding the registry lock through session construction or network I/O. Use reservation/generation tokens so concurrent starts for equivalent definitions cannot create unbounded duplicate sessions.

Process restart restores only identities whose contract requires persistence. Ephemeral/new identities must not become accidentally persistent due to store leftovers.

## 9. Compatibility and migration

Existing definitions with these options currently fail before allocation. M110 must not reinterpret previously accepted inert state because none is authoritative. Only successfully validated new/edited definitions may activate the new semantics.

No router.toml migration or core key-store migration is permitted.

## 10. Focused tests

At minimum cover:

- two compatible `Shared=true` definitions use one session owner while two `Shared=false` definitions do not;
- incompatible settings never share;
- stopping one shared member keeps the session alive for another; final release tears it down;
- concurrent acquisition is bounded and duplicate-safe;
- `NewDest` rotates only at the defined transition;
- persistent client identity survives process/store restart fixtures;
- failed start/edit does not lose the last-known-good persistent identity;
- confined `PrivKeyFile` import rejects traversal, absolute paths, symlinks, special files, malformed/oversized secrets;
- imported secret is copied into owned storage and remains secret-safe;
- matrix cells change only when runtime evidence exists.

## 11. Broad verification

Use the standard Proposal 170 feature, containment, matrix/audit, live-runtime, check, clippy, fmt-attempt and `git diff --check` commands from the active roadmap. Add no CI farm.

Security-sensitive secret-store tests must be included in the feature-gated suite.

## 12. Acceptance criteria

M110 closes only if:

1. every targeted cell is either `apply` with runtime evidence or remains explicitly blocked with a named primitive; no inert acceptance exists;
2. shared-session ownership is bounded, deterministic, compatible and generation-safe;
3. persistent/new/imported identity semantics are real and secret-safe;
4. no arbitrary filesystem capability is exposed;
5. no dependency/core/util/startup/frontend expansion occurred unless separately re-planned before implementation;
6. M061/M062/M093 invariants remain passing;
7. closure records exact matrix deltas and residual count;
8. a dependency decision names the next registry-ready handoff.

## 13. Stop conditions

Stop rather than widen scope if:

- Yosemite cannot consume the required destination/key material through accepted public APIs;
- correct sharing requires modifying Yosemite or router core;
- persistent identity would require a router-global key service;
- a request-selected path cannot be safely confined without violating the pinned contract;
- any secret would need to enter `raw_config`, logs, or ordinary definition persistence;
- a shared session cannot be cancelled/released without affecting unrelated owners.

## 14. Closure evidence

Require exact implementation commits/paths, cell-by-cell before/after matrix, secret-store permission/type evidence, shared-session contention/cancellation tests, restart identity evidence, full verification outcomes, unresolved findings, containment/security review, next-handoff decision, and internal-only attestation.

## 15. Internal-only boundary

No upstream issue/PR/review/submission/merge/adoption request, dependency contribution preparation, branch/tag push, release, or maintainer contact is authorized. External sources are read-only evidence only.
