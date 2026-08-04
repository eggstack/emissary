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
| I2PControl Proposal 170 | partial Proposal 170 support | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | M029 closed | Exact in-scope dimensions closed; unavailable sources and unsupported runtimes remain |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|
| — | None | — | — | — |

## Registered successor handoffs

| Subsystem | Handoff | Status | Implementation plan | Hard dependency |
|---|---|---|---|---|
| — | None | — | — | — |

## Active closure work

| Subsystem | Handoff | Status | Evidence | Closure record |
|---|---|---|---|---|
| — | None | — | — | — |

## Recently closed milestones

| Subsystem | Handoff | Status | Evidence |
|---|---|---|---|
| I2PControl Proposal 170 | M028 — Post-M027 status and AddressBook feature isolation | closed | `plans/closure/i2pcontrol-proposal-170/028-closure.md` |
| I2PControl Proposal 170 | M029 — In-scope Proposal 170 conformance reclosure | partial Proposal 170 support | `plans/closure/i2pcontrol-proposal-170/029-closure.md` |

## Current corrective findings

| Finding | Severity | Owner | State |
|---|---|---|---|
| Post-M027 merge revived superseded M019 as controlling closure | high claim/governance defect | M028 | resolved; M029 accepted |
| Top-level support documents overstate closed status while M027 records partial support | high claim defect | M028 | resolved; M029 accepted |
| Proposal 170 AddressBook control owner is constructed without compile-time/runtime enablement isolation | medium compatibility/scope defect | M028 | resolved; M029 accepted |
| Disabled/default address-book execution can read, publish, and persist `control-state.json` | medium behavior defect | M028 | resolved; M029 accepted |
| `serde_json` became unconditional for the CLI after the control-state bridge | low/medium dependency-footprint defect | M028 | resolved; M029 accepted |
| Final-head independent review after correction | high evidence gate | M029 | resolved; partial Proposal 170 support accepted |

Authoritative invalidation:

- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`

## Retained corrective evidence

The following implementation work remains retained candidate evidence and must not be reimplemented without a new defect:

| Milestone | Retained scope | Current evidence status |
|---|---|---|
| M020 | base I2PControl auth/token/errors, JSON-RPC notifications and IDs, base RouterInfo compatibility | retained |
| M021 | exact TunnelManager wire, validation, atomic persistence, secret boundary | retained |
| M022 | enabled-mode runtime AddressBook authority | retained but feature-isolation boundary reopened by M028 |
| M023 | startup tunnel inventory and ClientServicesInfo lifecycle/address truthfulness | retained |
| M024 | recoverable bounded SAM observation | retained |
| M025 | exact 43-selector RouterInfo contract/source matrix | retained |
| M026 | bounded-source audit; no feasible additional authoritative sources | retained |
| M027 | literal fixtures and partial-support disposition | retained evidence; final closure historically invalidated |

RouterInfo source classification remains:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

Missing tunnel data planes remain explicit unsupported runtimes and are not reopened by M028/M029.

## Historical invalidation and supersession

| Subsystem | Record | Status | Document | Disposition |
|---|---|---|---|---|
| I2PControl Proposal 170 | M029 final reclosure | closed; partial Proposal 170 support | `plans/closure/i2pcontrol-proposal-170/029-closure.md` | exact in-scope closure accepted; unavailable sources/runtime remain explicit |
| I2PControl Proposal 170 | M027 final internal reclosure | invalidated final disposition; evidence retained | `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md` | feature-isolation and post-merge status defects require M028/M029 |
| I2PControl Proposal 170 | M019 closure revived by `03a384a` | superseded/non-controlling | `plans/closure/i2pcontrol-proposal-170/019-closure.md` | historical evidence only; predates M020–M027 correction |
| I2PControl Proposal 170 | M019 original implementation plan | superseded/non-executable | `plans/implementation/i2pcontrol-proposal-170/019-pinned-revision-independent-reclosure.md` | must not be executed |
| I2PControl Proposal 170 | M019A final internal closure | invalidated | `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md` | historical evidence retained |
| I2PControl Proposal 170 | M017 broad closure | invalidated historical closure | `plans/closure/i2pcontrol-proposal-170/017-closure-invalidation.md` | component evidence only |

## Scope guard

M028/M029 own only Proposal 170 status correctness and the existing AddressBook control-state feature boundary.

Allowed production scope:

- `emissary-cli/src/address_book.rs`;
- `emissary-cli/src/main.rs` and `emissary-cli/src/lib.rs` composition only;
- `emissary-cli/Cargo.toml` optional dependency ownership;
- directly affected `emissary-cli/src/i2pcontrol/**` adapter files;
- focused tests and directly affected documentation/planning.

Prohibited:

- implementation of missing HTTP, IRC, SOCKS-IRC, CONNECT, Streamr, bidirectional, or other tunnel data planes;
- new RouterInfo sources, samplers, polling, peer classifications, NetDB inspection, or fabricated values;
- router, transport, peer-selection, cryptographic, streaming, LeaseSet, SAM, resolver, downloader-policy, or frontend redesign;
- generic event buses, task registries, plugin systems, schema frameworks, or second AddressBook authorities;
- persistence schema migration hidden inside M028;
- new dependencies without explicit maintainer direction;
- `.github/workflows/**`, CI, release, packaging, publishing, version, matrix, coverage, fuzz, soak, or generated-evidence machinery;
- repository-wide formatting;
- upstream contribution, review, submission, adoption, approval, merge, or maintainer-contact activity.

## Pinned authority

Current work is pinned to:

- proposal: `I2PControl Expansion`, Proposal 170;
- status: `Open`;
- created: `2026-05-20`;
- last updated: `2026-05-20`;
- canonical page: `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`;
- existing I2PControl authentication/error documentation: `https://i2p.net/en/docs/api/i2pcontrol`.

A changed proposal revision blocks M029 and requires a new contract-rebase plan.

## Registry maintenance rules

1. M028 is the completed implementation correction and M029 is the accepted controlling closure.
2. No dependency-ready implementation handoff remains for this workstream.
3. M019 is superseded and must never again be treated as current closure.
4. M020–M027 evidence is retained; do not reopen unrelated work.
5. Disabled/default AddressBook execution must not consult Proposal 170 control state.
6. Final expected in-scope disposition remains `partial Proposal 170 support` while 26 RouterInfo sources and missing tunnel data planes remain unavailable/unsupported.
7. Do not count compatibility aliases, unavailable sources, stored definitions, or unsupported stubs as operational coverage.
8. Verification remains local and package-scoped.
9. No upstream interaction is authorized.
