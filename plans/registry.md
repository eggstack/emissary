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
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 | closed against pinned revision | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | M019 accepted at `db5e067` | Proposal 170 remains Open; closure is bound to the 2026-05-20 revision |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|
| — | — | — | — | — |

## Recently closed milestones

| Subsystem | Milestone | Status | Implementation plan | Evidence commit | Closure record |
|---|---|---|---|---|---|
| I2PControl Proposal 170 | 018 — exact wire-contract reconciliation | closed | `plans/implementation/i2pcontrol-proposal-170/018-exact-wire-contract-reconciliation.md` | `ea35de9` | `plans/closure/i2pcontrol-proposal-170/018-implementation-disposition.md` |
| I2PControl Proposal 170 | 019 — pinned-revision independent reclosure | closed against pinned revision | `plans/implementation/i2pcontrol-proposal-170/019-pinned-revision-independent-reclosure.md` | `db5e067` | `plans/closure/i2pcontrol-proposal-170/019-closure.md` |

## Blocked plans

| Subsystem | Milestone | Status | Plan | Blocker |
|---|---|---|---|---|
| — | None | — | — | M019A is invalidated; no closure work is active |

## Recently closed milestones

| Subsystem | Milestone | Status | Closure record |
|---|---|---|---|
| Exact Proposal 170 RouterInfo keys are absent or renamed, and a 121-key legacy catalog is mislabeled as Proposal 170 | high | M018 | resolved and independently accepted by M019 at `db5e067` |
| AddressBook canonical `Type`/`Hostname`/`Destination`/optional `Delete` and in-method `SetSubscriptions`/`SetConfig` modes are missing | high | M018 | resolved and independently accepted by M019 at `db5e067` |
| TunnelManager lowercase actions and structured result envelopes are missing; `List` and capitalized actions are extensions | high | M018 | resolved and independently accepted by M019 at `db5e067` |
| ClientServicesInfo direct parameter-by-presence form is missing | high | M018 | resolved and independently accepted by M019 at `db5e067` |
| Real-session-to-production-ClientServicesInfo SAM evidence is incomplete | informational evidence limitation | M018/M019 | accepted bounded environmental limitation; closest-production composition and retained lifecycle evidence recorded |
| Wire, source, and runtime support claims are conflated | medium documentation | M018 | resolved and independently accepted by M019 at `db5e067` |
| M017 claimed zero unresolved high/medium findings | closure defect | M019 | superseded by independent M019 closure; historical record preserved |

## Historical invalidation and supersession

| Subsystem | Record | Status | Document | Disposition |
|---|---|---|---|---|
| I2PControl Proposal 170 | M019A final internal closure | invalidated | `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md` | historical evidence retained; completeness conclusion revoked |
| I2PControl Proposal 170 | M019 original pinned reclosure | superseded | `plans/implementation/i2pcontrol-proposal-170/019-pinned-revision-independent-reclosure.md` | non-executable |
| I2PControl Proposal 170 | M017 broad closure | invalidated historical closure | `plans/closure/i2pcontrol-proposal-170/017-closure-invalidation.md` | component evidence only |

## Current corrective findings

| Finding group | Severity | Owner | State |
|---|---|---|---|
| Standard I2PControl auth/token/error incompatibility | high | M020 | resolved |
| JSON-RPC notification and request-ID correctness | high | M020 | resolved |
| Direct base RouterInfo compatibility after token removal | high | M020 | resolved |
| Canonical TunnelManager `get` schema and validation | high | M021 | resolved; M023 owns truthful startup sources |
| Non-atomic tunnel rename and secret handling | high | M021 | resolved; M022/M023 consume the corrected primitives |
| AddressBook disconnected administrative shadow | high | M022 | resolved; M025 owns final selector/source matrix |
| Startup tunnel inventory and stale proxy state | high/medium | M023 | resolved; M024 consumes final service-source evidence |
| Sticky SAM observation overflow | medium | M024 | resolved; bounded incomplete state recovers from lifecycle events |
| RouterInfo 43-selector source/claim contradictions | high claim defect | M025 | resolved; exact matrix and truthful counts frozen |
| Feasible bounded core inspection sources | medium | M026 | resolved; frozen matrix has no feasible fields and 26 explicit deferred/out-of-scope fields |
| Literal external conformance and honest reclosure | high evidence defect | M027 | resolved; final status is partial Proposal 170 support |

The complete finding inventory and rationale are in `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`.

## Scope guard

The corrective sequence owns Proposal 170 and the existing I2PControl contract only.

Allowed production scope:

- `emissary-cli/src/i2pcontrol/**`;
- one purpose-specific runtime AddressBook handle and composition wiring;
- composition-time startup tunnel inventory;
- existing generic client/server lifecycle handle only if already safely targetable without redesign;
- passive HTTP/SOCKS task-exit observations;
- correction to the existing bounded SAM observation seam;
- bounded read-only RouterInfo snapshots adjacent to authoritative owners identified by M025.

Prohibited:

- implementation of missing HTTP, IRC, SOCKS-IRC, CONNECT, Streamr, bidirectional, or other tunnel data planes;
- router, transport, NetDB, peer-selection, cryptographic, streaming, LeaseSet, resolver, or frontend redesign;
- new historical telemetry samplers, polling loops, generic event buses, introspection frameworks, or task registries;
- fabricated values for unavailable selectors;
- new dependencies without explicit maintainer direction;
- `.github/workflows/**`, CI, release, packaging, publishing, version, matrix, coverage, fuzz, soak, or generated-evidence machinery;
- repository-wide formatting;
- upstream contribution, review, submission, adoption, approval, merge, or maintainer-contact activity.

## Internal-only upstream boundary

The Proposal 170 workstream is internal to `eggstack/emissary`.

No active or historical plan authorizes:

- upstream issues, pull requests, merge requests, discussions, review requests, or patch submissions;
- upstream review, feedback, adoption, approval, or merge solicitation;
- pushes of branches, commits, tags, patches, artifacts, or releases to an upstream remote;
- upstream maintainer outreach;
- preparation of an upstream contribution package, patch series, submission checklist, or merge plan;
- connector/API write actions against an upstream or third-party repository.

External specifications and source trees may be inspected read-only and cited internally.

All writes for this workstream must target `eggstack/emissary`. A future upstream contribution requires a new explicit maintainer directive that supersedes `plans/003-planning-process.md`; no current plan grants that authority.

Any upstream write, solicitation, or contribution preparation is a stop condition and invalidates affected evidence.

## Pinned authority

Current internal work is pinned to:

- proposal: `I2PControl Expansion`, Proposal 170;
- status: `Open`;
- created: `2026-05-20`;
- last updated: `2026-05-20`;
- canonical page: `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`;
- existing I2PControl authentication/error documentation: `https://i2p.net/en/docs/api/i2pcontrol`.

A changed proposal revision blocks M027 until the contract matrix and fixtures are reconciled. It does not authorize upstream contact.

## Registry maintenance rules

1. M018 implementation is frozen and accepted; M019 is the completed independent review.
2. Final Proposal 170 subsystem closure is recorded against the pinned revision.
3. Do not count compatibility aliases, legacy/base keys, unavailable sources, or unsupported runtimes as canonical operational coverage.
4. Preserve M017 and its invalidation as history; do not rewrite M017 into passing evidence.
5. If the open Proposal 170 source changes, rebase the contract manifest before closure.
6. M018 is `closed` and M019 is `closed against pinned revision` after all applicable acceptance criteria passed.
7. M019 used a distinct auditable review run and independently compared literal fixtures to the pinned source.
8. Any future high/medium finding within this boundary returns to M018; no duplicate corrective milestone is created.
9. Final status is `closed against pinned revision` with zero unresolved high/medium findings.
10. Verification remains local and package-scoped; remote CI is not required.
11. Do not expand this corrective line into CI, release, broad security, missing tunnel runtime, or generic framework work.
