# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies and interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency or evidence requirement prevents progress.
- **closing** — implementation landed and independent closure evidence is being gathered.
- **closed** — closure record accepted.
- **closed internally against pinned revision** — internal closure accepted against an explicitly named revision of an open external specification; does not imply upstream review or acceptance.
- **partial Proposal 170 support** — exact supported dimensions are closed, but one or more pinned source/runtime capabilities remain truthfully unavailable.
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 | partial Proposal 170 support; operational corrective work active | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | M037 ready | M030–M036 closed; M037 is the only dependency-ready plan |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|
| I2PControl Proposal 170 | M037 — Containment boundary reduction | ready | `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md` | M036 closed; ADR-0002 accepted |

## Registered successor handoffs

| Subsystem | Handoff | Status | Plan | Hard dependency |
|---|---|---|---|---|
| I2PControl Proposal 170 | M032 — Generic server backend and destination identity | closed | `plans/implementation/i2pcontrol-proposal-170/032-server-tunnel-runtime-backend.md` | M031 closed |
| I2PControl Proposal 170 | M033 — Lifecycle reconciliation and StartOnLoad | closed | `plans/implementation/i2pcontrol-proposal-170/033-tunnel-lifecycle-reconciliation.md` | M031 and M032 closed |
| I2PControl Proposal 170 | M034 — AddressBook setter truthfulness | closed | `plans/implementation/i2pcontrol-proposal-170/034-addressbook-setter-truthfulness.md` | M033 closed |
| I2PControl Proposal 170 | M035 — Base compatibility and selector overlap | closed | `plans/implementation/i2pcontrol-proposal-170/035-base-compatibility-and-selector-overlap.md` | M034 closed |
| I2PControl Proposal 170 | M036 — Authentication and publication hardening | closed | `plans/implementation/i2pcontrol-proposal-170/036-auth-and-publication-hardening.md` | M035 closed |
| I2PControl Proposal 170 | M037 — Containment boundary reduction | ready | `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md` | M036 closed |
| I2PControl Proposal 170 | M038 — Live-runtime interoperability | blocked | `plans/implementation/i2pcontrol-proposal-170/038-live-runtime-interoperability.md` | M031–M037 closed |
| I2PControl Proposal 170 | M039 — Operational final-head reclosure | blocked | `plans/implementation/i2pcontrol-proposal-170/039-operational-reclosure.md` | M038 closed |

## Active closure work

| Subsystem | Handoff | Status | Evidence | Closure record |
|---|---|---|---|---|
| — | None | — | M036 closure accepted; M037 is ready for handoff | — |

## Recently closed milestones

| Subsystem | Handoff | Status | Closure | Implementation disposition |
|---|---|---|---|---|
| I2PControl Proposal 170 | M031 — Runtime supervisor and generic client backend | closed | `plans/closure/i2pcontrol-proposal-170/031-closure.md` | `plans/closure/i2pcontrol-proposal-170/031-implementation-disposition.md` |
| I2PControl Proposal 170 | M032 — Generic server backend and destination identity | closed | `plans/closure/i2pcontrol-proposal-170/032-closure.md` | `plans/closure/i2pcontrol-proposal-170/032-implementation-disposition.md` |
| I2PControl Proposal 170 | M033 — Lifecycle reconciliation and StartOnLoad | closed | `plans/closure/i2pcontrol-proposal-170/033-closure.md` | `plans/closure/i2pcontrol-proposal-170/033-implementation-disposition.md` |
| I2PControl Proposal 170 | M034 — AddressBook setter truthfulness | closed | `plans/closure/i2pcontrol-proposal-170/034-closure.md` | `plans/closure/i2pcontrol-proposal-170/034-implementation-disposition.md` |
| I2PControl Proposal 170 | M035 — Base compatibility and selector overlap | closed | `plans/closure/i2pcontrol-proposal-170/035-closure.md` | `plans/closure/i2pcontrol-proposal-170/035-implementation-disposition.md` |
| I2PControl Proposal 170 | M036 — Authentication and publication hardening | closed | `plans/closure/i2pcontrol-proposal-170/036-closure.md` | `plans/closure/i2pcontrol-proposal-170/036-implementation-disposition.md` |

## Current corrective findings

| Finding | Severity | Owner | State |
|---|---|---|---|
| Ten non-client/server tunnel families remain unsupported | high runtime capability gap | roadmap future work | retained explicit disposition |
| Base I2PControl dispatcher/overlapping RouterInfo names require explicit compatibility boundary | medium compatibility gap | M035 | closed; mode-specific inventory and tests added |
| Hand-written password comparison and no failed-auth throttle | medium security gap | M036 | closed; reviewed primitive and bounded peer throttle |
| Publication documentation may overstate power-loss durability | medium persistence-claim gap | M036 | closed; claims qualified and directory sync added |
| Proposal 170 AddressBook/SAM policy remains broader than desired outside i2pcontrol | medium containment gap | M037 | ready on M036 closure |
| No bounded live production-composition interoperability run | medium evidence gap | M038 | blocked on M031–M037 |
| Independent final-head operational review | high evidence gate | M039 | blocked on M038 |

## Runtime tunnel decision

Accepted decision:

- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`

Under ADR-0002:

- only generic `client` and `server` are eligible for real backends in this roadmap;
- existing startup-managed definitions remain externally owned and read-only;
- I2PControl owns a separate supervisor only for control-plane-created definitions;
- existing CLI client/server data planes are reused through narrow single-instance adapters;
- `emissary-core/**` changes are prohibited for M031–M036;
- HTTP, HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, and other missing types remain explicit unsupported backends;
- existing HTTP/SOCKS startup services are not automatically Proposal 170 tunnel backends.

## M031 scope guard

Primary production work should remain in:

- `emissary-cli/src/i2pcontrol/backends/**`;
- `emissary-cli/src/i2pcontrol/production.rs`;
- directly affected I2PControl domain/server/tests.

Permitted changes outside `i2pcontrol/**` are limited to:

- `emissary-cli/src/tunnel/client.rs` for one purpose-specific cancellation-aware single-client runtime primitive while preserving startup behavior;
- `emissary-cli/src/main.rs` for one narrow composition input if required.

Prohibited in M031:

- `emissary-core/**`;
- generic server backend or destination identity work;
- missing HTTP/SOCKS/IRC/CONNECT/Streamr/bidirectional data planes;
- startup task adoption/control;
- AddressBook, RouterInfo source, SAM observer, frontend, CI/release, packaging, version, fuzz, soak, or unrelated refactors;
- public protocol fields, methods, aliases, statuses, or tunnel types;
- upstream contribution, review, submission, adoption, merge, or maintainer contact.

## Retained closed evidence

| Milestone | Retained scope | Current evidence status |
|---|---|---|
| M020 | authentication/token/errors, JSON-RPC notifications/IDs, retained base behavior | retained |
| M021 | exact TunnelManager wire, validation, atomic definition persistence, secret boundary | retained |
| M022 | enabled-mode AddressBook authority | retained |
| M023 | startup inventory and ClientServicesInfo lifecycle/address truthfulness | retained |
| M024 | recoverable bounded SAM observation | retained |
| M025 | exact 43-selector RouterInfo contract/source matrix | retained |
| M026 | bounded-source audit; no additional source authorized | retained |
| M027 | literal fixtures | retained evidence; historical final disposition invalidated |
| M028 | compile-time/runtime AddressBook feature isolation | retained |
| M029 | independent review evidence | historical invalidated closure; non-AddressBook evidence retained |
| M030 | full-destination owner coherence and partial-support closure | controlling baseline |

RouterInfo source classification remains:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

## Historical invalidation and supersession

| Subsystem | Record | Status | Document | Disposition |
|---|---|---|---|---|
| I2PControl Proposal 170 | M029 final reclosure | invalidated final disposition; evidence retained | `plans/closure/i2pcontrol-proposal-170/029-closure-invalidation.md` | corrected by M030 |
| I2PControl Proposal 170 | M027 final reclosure | invalidated final disposition; evidence retained | `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md` | historical |
| I2PControl Proposal 170 | M019 closure revived by `03a384a` | superseded/non-controlling | `plans/closure/i2pcontrol-proposal-170/019-closure.md` | historical only |
| I2PControl Proposal 170 | M019A final closure | invalidated | `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md` | historical only |
| I2PControl Proposal 170 | M017 broad closure | invalidated | `plans/closure/i2pcontrol-proposal-170/017-closure-invalidation.md` | component evidence only |

## Pinned authority

Current work is pinned to:

- proposal: `I2PControl Expansion`, Proposal 170;
- status: `Open`;
- created: `2026-05-20`;
- last updated: `2026-05-20`;
- canonical page: `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`;
- existing I2PControl authentication/error documentation: `https://i2p.net/en/docs/api/i2pcontrol`.

A changed proposal revision blocks the affected implementation/closure and
requires a contract-rebase plan.

## Registry maintenance rules

1. M037 is the only dependency-ready implementation handoff.
2. Do not advance M038 until M037 implementation disposition and closure are accepted.
3. Preserve M020–M030 evidence unless a new direct defect is demonstrated.
4. Keep startup and control-plane runtime ownership separate.
5. Keep production changes outside `i2pcontrol/**` minimal and individually justified.
6. No core changes are authorized before M037; M037 may only reduce existing coupling through a passive hook.
7. Unsupported tunnel families and unavailable RouterInfo sources remain explicit.
8. Verification remains local and package-scoped; no CI/release expansion.
9. M038 requires real production-composition evidence, not fake-only substitution.
10. M039 is the distinct final-head review.
11. No upstream interaction is authorized.
