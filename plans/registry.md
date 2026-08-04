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
| I2PControl Proposal 170 | partial Proposal 170 support | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | M030 closed | No in-scope blocker; unavailable sources/data planes remain outside this correction |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|
| I2PControl Proposal 170 | M030 — AddressBook destination and owner coherence | closed | `plans/implementation/i2pcontrol-proposal-170/030-addressbook-destination-owner-coherence.md` | M029 invalidation corrected |

## Registered successor handoffs

| Subsystem | Handoff | Status | Plan | Hard dependency |
|---|---|---|---|---|
| — | None | — | — | No successor is currently authorized or dependency-ready; remaining unavailable capabilities require separately scoped plans |

## Active closure work

| Subsystem | Handoff | Status | Evidence | Closure record |
|---|---|---|---|---|
| — | None | — | — | M030 implementation and final closure records are complete |

## Current corrective findings

| Finding | Severity | Owner | State |
|---|---|---|---|
| Active Base64 lookup reads a legacy destination file before the active control owner | high owner-coherence defect | M030 | closed |
| Published control-state seeding copies Base32 cache values as destinations | high source/type defect | M030 | closed |
| Active download `or_insert` can retain an incomplete Base32-seeded value | medium persistence/source defect | M030 | closed |
| Update/delete regressions do not check Base64 lookup with a stale legacy file | high evidence defect | M030 | closed |
| First activation and repair paths lack full-destination structural evidence | medium evidence/security defect | M030 | closed |
| Independent final-head reclosure after M030 | high evidence gate | M030 | closed |

Authoritative invalidation:

- `plans/closure/i2pcontrol-proposal-170/029-closure-invalidation.md`

## Retained corrective evidence

The following work remains retained and must not be reimplemented without a new direct defect:

| Milestone | Retained scope | Current evidence status |
|---|---|---|
| M020 | base I2PControl auth/token/errors, JSON-RPC notifications and IDs, base RouterInfo compatibility | retained |
| M021 | exact TunnelManager wire, validation, atomic persistence, secret boundary | retained |
| M022 | enabled-mode runtime AddressBook authority | retained except destination/lookup coherence reopened by M030 |
| M023 | startup tunnel inventory and ClientServicesInfo lifecycle/address truthfulness | retained |
| M024 | recoverable bounded SAM observation | retained |
| M025 | exact 43-selector RouterInfo contract/source matrix | retained |
| M026 | bounded-source audit; no feasible additional authoritative sources | retained |
| M027 | literal fixtures and partial-support disposition | retained evidence; final disposition historically invalidated |
| M028 | compile-time/runtime AddressBook feature isolation and optional dependency ownership | retained |
| M029 | independent review evidence | historical invalidated closure; non-AddressBook evidence retained |

RouterInfo source classification remains:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

Missing tunnel data planes remain explicit unsupported runtimes and are not reopened by M030.

## Historical invalidation and supersession

| Subsystem | Record | Status | Document | Disposition |
|---|---|---|---|---|
| I2PControl Proposal 170 | M029 final reclosure | invalidated final disposition; evidence retained | `plans/closure/i2pcontrol-proposal-170/029-closure-invalidation.md` | active Base64 and published-destination coherence defects require M030 |
| I2PControl Proposal 170 | M027 final reclosure | invalidated final disposition; evidence retained | `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md` | historical |
| I2PControl Proposal 170 | M019 closure revived by `03a384a` | superseded/non-controlling | `plans/closure/i2pcontrol-proposal-170/019-closure.md` | historical only |
| I2PControl Proposal 170 | M019A final closure | invalidated | `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md` | historical only |
| I2PControl Proposal 170 | M017 broad closure | invalidated | `plans/closure/i2pcontrol-proposal-170/017-closure-invalidation.md` | component evidence only |

## M030 scope guard

Primary production work should remain in:

- `emissary-cli/src/i2pcontrol/production.rs`;
- directly affected `emissary-cli/src/i2pcontrol/**` tests/adapters.

Permitted changes outside the I2PControl crate are limited to:

- `emissary-cli/src/address_book.rs` for owner-aware lookup precedence, bounded full-destination loading/validation, one purpose-specific import/repair seam, and focused tests;
- `emissary-cli/src/main.rs` only for one narrow activation input/call if required.

Prohibited:

- `emissary-core/**` changes;
- missing HTTP, IRC, SOCKS-IRC, CONNECT, Streamr, bidirectional, or other tunnel data planes;
- new RouterInfo sources, samplers, polling, peer classifications, NetDB inspection, or fabricated values;
- router, transport, streaming, LeaseSet, cryptographic, SAM, frontend, or general resolver redesign;
- a second AddressBook authority, bidirectional synchronization framework, provenance/tombstone schema, generic migration engine, event bus, or background reconciler;
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

A changed proposal revision blocks M030 and requires a contract-rebase plan.

## Registry maintenance rules

1. No implementation handoff is currently dependency-ready.
2. M029 is historical invalidated evidence and must not be represented as controlling closure.
3. M030's frozen implementation/test head and distinct final-head closure are recorded in the 030 closure records.
4. Preserve M020–M028 unrelated evidence; do not reopen method families without a demonstrated defect.
5. Active owner lookup must be coherent across administrative, RouterInfo, Base32, and Base64 views.
6. Published Proposal 170 entries must contain validated full destinations, not Base32 cache values.
7. Disabled/default behavior and M028 feature isolation must remain unchanged.
8. Final expected in-scope disposition remains `partial Proposal 170 support` after correction and independent review.
9. Verification remains local and package-scoped.
10. No upstream interaction is authorized.
