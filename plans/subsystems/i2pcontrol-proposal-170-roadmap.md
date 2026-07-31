# I2PControl Proposal 170 Corrective Roadmap

Status: corrective pass required

Corrective baseline: `a3c4f469f4877e5ff4a0bb4230da298f0b367ed2`

Pinned authority:

- I2P Proposal 170, `I2PControl Expansion`
- status: `Open`
- created: `2026-05-20`
- last updated: `2026-05-20`
- `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`
- existing I2PControl authentication and error contract documented at `https://i2p.net/en/docs/api/i2pcontrol`

Canonical internal references:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`

## 1. Purpose

This roadmap corrects the current `eggstack/emissary` Proposal 170 implementation without expanding the workstream into missing tunnel data planes or broad router redesign.

The target is exact and truthful I2PControl behavior:

- preserve the existing I2PControl authentication and JSON-RPC contract;
- implement Proposal 170 methods, parameters, selector semantics, response fields, and JSON types exactly against the pinned revision;
- connect administrative methods to the actual Emissary owners when the method claims to manage running-router state;
- report unsupported tunnel runtimes and unavailable telemetry explicitly rather than fabricating success or values;
- preserve existing Emissary compatibility extensions only when they remain unambiguous and separately documented;
- keep production changes outside `emissary-cli/src/i2pcontrol/**` to the smallest bounded adapters or passive observation hooks required for truthful data.

## 2. Non-negotiable scope boundary

### 2.1 In scope

- base I2PControl authentication/token/error interoperability needed by Proposal 170 clients;
- JSON-RPC request, notification, and request-ID correctness;
- exact Proposal 170 RouterInfo, AddressBook, TunnelManager, and ClientServicesInfo wire behavior;
- durable and atomic administrative persistence;
- secret-safe logging, persistence, and response serialization;
- a narrow adapter to the actual runtime address-book owner;
- truthful read-only import of startup-managed tunnel definitions;
- lifecycle adapters only for already-existing generic Emissary client/server tunnel facilities when ownership can be proven safe and narrow;
- passive proxy/listener/session observation required by ClientServicesInfo;
- bounded read-only RouterInfo snapshots adjacent to existing state owners where they can be added without changing router algorithms;
- documentation, focused regressions, static guards, and independent internal closure.

### 2.2 Out of scope

- implementation of missing `httpclient`, `ircclient`, `socksirc`, `connectclient`, `streamrclient`, `httpserver`, `httpbidirserver`, `ircserver`, `streamrserver`, or any other missing tunnel data plane;
- changes to streaming, LeaseSet publication, tunnel construction, routing, transport negotiation, peer selection, NetDB algorithms, cryptography, or protocol behavior to make an API field easier to source;
- making I2PControl dependent on frontend or UI state;
- replacing the router's existing address-book precedence policy;
- generic observability frameworks, event buses, schema generators, plugin systems, or cross-router abstractions;
- new dependencies unless a later maintainer directive explicitly authorizes one;
- `.github/workflows/**`, release automation, coverage gates, fuzz farms, platform matrices, or generated evidence bundles;
- upstream contribution, review, submission, adoption, or merge activity.

## 3. Internal-only external-interaction rule

All writes must target `eggstack/emissary`.

No milestone may:

- write to an upstream or third-party repository;
- open or modify upstream issues, pull requests, merge requests, discussions, reviews, or proposals;
- request upstream review, feedback, approval, adoption, or merge;
- push branches, tags, patches, artifacts, or releases to an upstream remote;
- contact upstream maintainers;
- prepare an upstream contribution package, patch series, submission checklist, or merge plan.

Read-only specification and reference-source inspection is allowed for internal correctness. Violation is a stop condition and invalidates affected evidence.

## 4. Retained implementation

The corrective sequence should preserve these components unless direct evidence shows a defect:

- the feature-gated HTTPS I2PControl service and bounded connection/request handling;
- typed Proposal 170 tunnel types and exhaustive backend registration;
- explicit unsupported tunnel backends that do not bind listeners or report running;
- versioned generation-store publication and corruption fallback;
- direct Proposal 170 parameter-presence forms;
- fixed-size service registry with generation fencing;
- bounded read-only SAM observation ownership;
- live event counters and log ring already exposed through narrow adapters;
- compatibility aliases that remain separate from canonical contract accounting.

Retained code is not automatically retained evidence. Each milestone must add the regression that would have caught its named finding.

## 5. Corrective architecture

The primary implementation boundary remains:

```text
emissary-cli/src/i2pcontrol/**
```

Permitted external seams are deliberately narrow:

```text
emissary-cli/src/main.rs
    composition only: pass existing handles, startup definitions, and proxy exit observations

emissary-core/src/router/** or existing owner modules
    read-only bounded snapshot/handle only when no truthful source exists at the CLI layer

emissary-core/src/sam/**
    correction to the already-introduced bounded observation publisher only
```

No external seam may grant I2PControl generic mutation authority over router internals. Mutation must remain behind a purpose-specific trait whose implementation is owned by the existing subsystem.

## 6. Capability and evidence dimensions

Every Proposal 170 item is classified independently:

| Dimension | Meaning |
|---|---|
| Wire | exact request names, casing, presence rules, response fields, and JSON types |
| Source | truthful current Emissary data source exists |
| Runtime | requested operation controls a real backend |
| Persistence | mutation is durable and failure-atomic where required |
| Evidence | literal fixture, failure, restart, and production-composition proof exists |

Unsupported runtime and unavailable source are valid truthful states but are not counted as implemented runtime/source capability.

## 7. Dependency sequence

```text
M020 base protocol interoperability
    |
    v
M021 TunnelManager exact contract and storage safety
    |
    +--------------------------+
    |                          |
    v                          v
M022 AddressBook runtime bridge   M023 startup tunnel inventory and client-service truthfulness
    |                          |
    +-------------+------------+
                  v
M024 recoverable bounded SAM observation
                  |
                  v
M025 RouterInfo contract/source reconciliation
                  |
                  v
M026 bounded core inspection for feasible remaining selectors
                  |
                  v
M027 conformance and independent reclosure
```

M020 is ready. M021 depends hard on M020 because standard token removal and request parsing affect TunnelManager fixtures. M022 and M023 depend hard on M021's corrected shared tunnel/address model only where they consume it; they may otherwise execute in parallel after M021. M024 depends on M023's final ClientServicesInfo ownership. M025 depends on M020, M022, and M023. M026 depends on M025's exact source matrix. M027 depends on every implementation milestone reaching a frozen disposition.

## 8. Milestones

### M020 — Base I2PControl and JSON-RPC interoperability

Plan: `020-base-i2pcontrol-and-jsonrpc-interoperability.md`

Status: ready

Objective:

- restore standard `Authenticate` parameters and numeric response API;
- accept the standard `Token` parameter on protected calls and distinguish missing/unknown token errors;
- preserve the current header token only as a documented compatibility extension;
- execute JSON-RPC notifications while suppressing their responses;
- reject invalid request IDs without coercion;
- preserve the existing direct RouterInfo selector surface alongside Proposal 170 additions.

Primary boundary: `emissary-cli/src/i2pcontrol/**` only.

Exit: literal existing-I2PControl fixtures and Proposal 170 requests work through the same dispatcher.

### M021 — TunnelManager exact wire, atomic persistence, and secret boundary

Plan: `021-tunnelmanager-wire-atomicity-and-secrets.md`

Status: blocked on M020

Objective:

- implement the exact pinned `result.status`, `result.results`, and `result.info` shapes;
- emit exact `info` and nested `rawConfig` names/types;
- validate all defined canonical fields and enum/range constraints without accepting arbitrary top-level extensions;
- publish edit/rename as one generation;
- prevent secret duplication, logging, and accidental response disclosure while retaining only the storage necessary for future backends;
- enforce persistence permissions or fail closed where the platform supports them.

Primary boundary: `i2pcontrol` handler/domain/store/backend modules.

Exit: all seven actions have literal success/failure fixtures, rename failure injection preserves the original definition, and unsupported runtimes remain explicit stubs.

### M022 — AddressBook runtime bridge and canonical source reconciliation

Plan: `022-addressbook-runtime-bridge.md`

Status: blocked on M020 and M021

Objective:

- replace the disconnected administrative-only success claim with a narrow adapter to the actual runtime address-book owner;
- preserve four Proposal 170 administrative book identities without changing resolver precedence;
- ensure add/delete/set operations affect the source the API claims to manage or fail explicitly;
- define restart/import behavior and avoid two contradictory authoritative stores;
- correct subscription/config RouterInfo shapes and path/entry provenance without allowing arbitrary path writes.

External changes: one purpose-specific runtime address-book control handle and composition wiring only.

Exit: successful mutations are observable through both the API and normal runtime lookup according to documented book precedence, with restart evidence.

### M023 — Startup tunnel inventory and ClientServicesInfo truthfulness

Plan: `023-startup-tunnel-inventory-and-client-services.md`

Status: blocked on M021

Objective:

- import startup-configured client/server tunnels as read-only `StartupManaged` inventory;
- prevent duplicate administrative definitions from contradicting startup-owned names;
- add real lifecycle adapters only where existing manager ownership permits safe targeted control without redesign;
- publish proxy `Stopped` on task exit;
- derive ClientServicesInfo addresses only from actual bound/listening or destination sources;
- never substitute local target hosts for I2P destinations.

External changes: composition-time definition mapping and two passive proxy-exit observation calls; existing manager adapter only if ownership is demonstrably targetable.

Exit: current startup inventory is visible and immutable, task exit clears enabled state, and unsupported/missing destination data is explicit.

### M024 — Recoverable bounded SAM observation

Plan: `024-recoverable-bounded-sam-observation.md`

Status: blocked on M023

Objective:

- retain the existing read-only bounded SAM handle;
- replace process-lifetime sticky overflow with recoverable, generation-aware incomplete-state semantics;
- preserve active-session/socket freshness across create, close, accept, connect, and forward paths;
- fail explicitly only while the authoritative bounded snapshot is incomplete;
- avoid a second SAM registry or lifecycle controller.

External changes: only the existing SAM observation publisher/handle and its call sites.

Exit: transient bound pressure can recover without router restart, no private key/session material is exposed, and ClientServicesInfo remains bounded.

### M025 — RouterInfo contract and source reconciliation

Plan: `025-routerinfo-contract-and-source-reconciliation.md`

Status: blocked on M020, M022, M023, and M024

Objective:

- rebuild the exact 43-selector manifest and JSON types from the pinned proposal;
- map every selector to one current source owner or explicit unavailable reason;
- correct current contradictions, especially address-book subscription/config objects and service/tunnel inventory fields;
- remove completion claims for unavailable sources;
- identify the smallest feasible read-only snapshots required by M026.

This milestone is principally `i2pcontrol` code, tests, and documentation. It must not add broad core inspection itself.

Exit: one machine-readable matrix drives validation, dispatch, documentation, and successor scope; no selector is mislabeled available.

### M026 — Bounded core inspection for feasible remaining selectors

Plan: `026-bounded-router-inspection-sources.md`

Status: blocked on M025

Objective:

- add only bounded read-only snapshots for Proposal 170 fields whose current authoritative state already exists in Emissary;
- group work by existing owner: event/network status, tunnel pools/build queues, NetDB peers/RouterInfo, connection limits, and bans if an owner exists;
- do not add new historical samplers, peer classifications, algorithms, or fabricated defaults;
- leave fields explicitly unavailable when the authoritative state does not exist or would require invasive redesign.

External changes: adjacent snapshot DTO/handle methods only; no mutation, no single-owner receiver consumption, no task spawning for polling.

Exit: every newly available selector has bounded source and failure tests; remaining unavailable fields have precise reasons and block unqualified full-completion claims.

### M027 — Exact conformance, documentation, and independent reclosure

Plan: `027-proposal-170-conformance-and-reclosure.md`

Status: blocked on M020–M026

Objective:

- run literal wire fixtures for base I2PControl plus all Proposal 170 methods and selectors;
- verify production composition, restart, failure atomicity, unavailable/unsupported behavior, and secret non-disclosure;
- reconcile all support documents and static manifests;
- perform a distinct internal review against the pinned revision;
- choose an honest final disposition.

Possible final dispositions:

- `closed internally against pinned revision` only when exact wire behavior and every claimed source/runtime dimension are supported;
- `partial Proposal 170 support` when one or more pinned source fields remain truthfully unavailable;
- `corrective pass required` for any unresolved high/medium defect.

## 9. Cross-cutting invariants

1. Existing I2PControl clients continue to authenticate and submit `Token` in `params` without modification.
2. Canonical Proposal 170 names, casing, parameter-presence rules, response fields, and JSON types are exact.
3. Compatibility aliases are isolated and never counted as canonical coverage.
4. Protected operations authenticate before expensive parsing or mutation.
5. No unsupported tunnel type binds a listener, creates a destination, publishes a LeaseSet, or reports running.
6. Missing tunnel data planes remain outside this workstream.
7. Startup-owned tunnel definitions cannot be edited or shadowed by control-plane definitions.
8. Successful persistent mutation is durable before response and failure-atomic.
9. AddressBook success corresponds to the actual managed source, not a disconnected shadow.
10. I2PControl never consumes a single-owner event receiver or changes router algorithms.
11. ClientServicesInfo enabled state reflects actual bound/listening lifecycle and clears on exit.
12. No secret, private key, credential, full destination, or sensitive path appears in logs or unintended responses.
13. Unavailable telemetry fails or reports the exact protocol-permitted neutral value; it is never fabricated.
14. External read-only observation is bounded and non-authoritative for lifecycle.
15. No plan creates upstream interaction or broad CI/release machinery.

## 10. Failure, restart, and contention policy

- All store mutation occurs under the existing serialized store owner and publishes one complete generation per logical operation.
- A failed create/edit/delete/set operation leaves the previous durable and in-memory state unchanged.
- Runtime adapter failure returns an explicit operation error and must not commit contradictory administrative state.
- Startup import is deterministic and repeatable; restart cannot duplicate startup-managed entries.
- Name collision between startup-owned and control-plane definitions fails deterministically before persistence.
- Observation overflow or source unavailability never returns a partial snapshot as complete.
- Locks are not held across unrelated runtime awaits; no new global lock is introduced.
- Shutdown and task exit publish inactive state where the owner already has a lifecycle signal.

## 11. Compatibility and migration

- Existing persisted generation files remain readable where structurally safe.
- Any persistence schema change requires versioned migration or explicit fail-closed incompatibility with a documented recovery path; silent reinterpretation is prohibited.
- Header token support may remain for existing Emissary clients but is secondary to standard `params.Token`.
- Capitalized action aliases and nested selector compatibility forms may remain if they do not weaken canonical parsing.
- Startup configuration remains authoritative for startup-managed tunnels.
- No router configuration file is rewritten by Proposal 170 handlers unless M022's actual address-book owner already owns that exact file and the adapter preserves its established atomic semantics.

## 12. Verification policy

Required package-scoped commands for implementation milestones:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Run `emissary-core` checks/tests only for milestones that touch the bounded runtime/SAM snapshot seams:

```bash
cargo check -p emissary-core
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
```

Use focused test filters named in each plan before broad package tests. Use touched-file formatting when unrelated repository-wide formatting differences remain. Do not add remote CI, platform matrices, coverage gates, fuzz campaigns, or generated evidence infrastructure.

## 13. Milestone status

| Milestone | Status | Disposition |
|---|---|---|
| M001–M018A | historical implementation evidence | retained only as referenced |
| M019 | superseded | non-executable |
| M019A | invalidated closure | historical evidence; see `019a-closure-invalidation.md` |
| M020 | ready | next executable handoff |
| M021 | blocked | hard dependency M020 |
| M022 | blocked | hard dependencies M020 and M021 |
| M023 | blocked | hard dependency M021 |
| M024 | blocked | hard dependency M023 |
| M025 | blocked | hard dependencies M020, M022, M023, M024 |
| M026 | blocked | hard dependency M025 |
| M027 | blocked | hard dependencies M020–M026 |

## 14. Completion definition

The corrective workstream is complete only when M027 records a disposition supported by:

- standard I2PControl authentication and token interoperability;
- correct JSON-RPC execution and request-ID behavior;
- exact Proposal 170 wire fixtures for all four method families;
- atomic tunnel mutation and secret-safe handling;
- actual AddressBook ownership integration;
- truthful startup tunnel and client-service inventory;
- recoverable bounded SAM observation;
- exact 43-selector source matrix with no fabricated data;
- bounded source evidence for every field claimed available;
- explicit unsupported runtime evidence for deferred tunnel types;
- restart, failure, contention, and production-composition tests;
- accurate documentation and static manifests;
- zero unresolved high/medium findings for the selected final disposition;
- internal-only/no-upstream compliance attestation.

The roadmap does not require unqualified full Proposal 170 support when Emissary lacks a non-invasive authoritative source. It requires an honest final status and prohibits claiming completion from parser coverage or administrative shadow state alone.