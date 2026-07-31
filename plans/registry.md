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
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 | active internal corrective work | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | M018A ready | M018 initial disposition reopened; M019 superseded; M019A blocked until corrected frozen head |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|
| I2PControl Proposal 170 | M018A — wire semantics and internal-only corrective pass | ready | `plans/implementation/i2pcontrol-proposal-170/018a-wire-semantics-and-internal-only-corrective-pass.md` | existing M018 implementation at `ea35de9`; bounded corrective scope only |

## Active closure work

| Subsystem | Handoff | Status | Evidence | Closure record |
|---|---|---|---|---|
| I2PControl Proposal 170 | M018 initial implementation | corrective pass required | `ea35de9` | `plans/closure/i2pcontrol-proposal-170/018-implementation-disposition.md` |

## Blocked and superseded handoffs

| Subsystem | Handoff | Status | Plan | Blocker or disposition |
|---|---|---|---|---|
| I2PControl Proposal 170 | M019 — original pinned reclosure | superseded | `plans/implementation/i2pcontrol-proposal-170/019-pinned-revision-independent-reclosure.md` | replaced by M019A after post-M018 findings and policy correction |
| I2PControl Proposal 170 | M019A — internal pinned-revision reclosure | blocked | `plans/implementation/i2pcontrol-proposal-170/019a-internal-pinned-revision-reclosure.md` | M018A must land on a frozen complete head with disposition; distinct internal reviewer required |

## Current corrective findings

| Finding | Severity | Owner | State |
|---|---|---|---|
| Transit total uses received plus sent rather than forwarded/transmitted transit bytes | high | M018A | ready |
| Valid canonical TunnelManager operational failures do not consistently use structured `result.status` | high | M018A | ready |
| Base and compatibility surfaces remain counted as canonical Proposal 170 inventory in the conformance manifest | medium | M018A | ready |
| TunnelManager canonical examples and `Name`/`All` wording need correction | low | M018A | ready |
| No true live-session SAM-to-I2PControl end-to-end test | medium evidence decision | M019A | blocked pending M018A; qualified evidence retained |
| Initial active planning lacked an absolute no-upstream rule | governance | M018A/M019A | normative rule now codified; implementation and closure must attest compliance |

## Internal-only upstream boundary

The Proposal 170 workstream is internal to `eggstack/emissary`.

No active or historical plan authorizes:

- upstream issues, pull requests, merge requests, discussions, review requests, or patch submissions;
- upstream review, feedback, adoption, approval, or merge solicitation;
- pushes of branches, commits, tags, patches, or artifacts to an upstream remote;
- upstream maintainer outreach;
- preparation of an upstream contribution package, patch series, submission checklist, or merge plan;
- connector/API write actions against an upstream repository.

External specifications, source trees, pull requests, commits, and discussions may be inspected read-only for internal verification and cited internally.

All writes for this workstream must target `eggstack/emissary`. A future upstream contribution requires a new explicit maintainer directive that supersedes `plans/003-planning-process.md`; no current plan grants that authority.

Any upstream write, submission, solicitation, or merge-preparation action is a stop condition and invalidates affected closure evidence.

## Pinned Proposal 170 authority

Current internal work is pinned to:

- proposal: `I2PControl Expansion`, Proposal 170;
- status: `Open`;
- created: `2026-05-20`;
- last updated: `2026-05-20`;
- canonical page: `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`.

Because the proposal is Open, final status may only be `closed internally against pinned revision`. A changed proposal revision blocks internal closure until the manifest and fixtures are reconciled. It does not trigger or authorize upstream contact.

## M018A scope guard

M018A owns only:

- forwarded/transmitted semantics for `i2p.router.net.total.transit.bytes`;
- canonical TunnelManager operational-failure `result.status` envelopes;
- canonical/base/compatibility manifest separation;
- directly affected tests and documentation;
- internal-only policy verification.

M018A must not add:

- `.github/workflows/**`, CI, platform, coverage, or evidence machinery;
- release, publishing, packaging, or version automation;
- upstream contribution or review activity;
- missing tunnel data planes;
- broad router, transport, NetDB, peer, tunnel, cryptographic, resolver, frontend, SAM, or I2CP redesign;
- generic protocol/schema/fixture frameworks;
- dependencies;
- repository-wide formatting;
- fabricated values.

## Historical records and current authority

M001–M004 remain historical implementation foundations. M005–M007 are superseded. M008–M014 and M016 contain retained implementation evidence. M015 remains an invalid historical closure. M017's component evidence is retained, but its broad closure is invalidated. The initial M018 implementation is retained but requires M018A correction.

| Record | Current disposition |
|---|---|
| `plans/closure/i2pcontrol-proposal-170/014-closure.md` | implementation evidence retained |
| `plans/closure/i2pcontrol-proposal-170/015-closure.md` | invalid historical closure |
| `plans/closure/i2pcontrol-proposal-170/016-implementation-disposition.md` | bounded SAM implementation retained |
| `plans/closure/i2pcontrol-proposal-170/017-closure.md` plus invalidation | historical component evidence only |
| `plans/closure/i2pcontrol-proposal-170/018-implementation-disposition.md` | corrective pass required |
| M018A plan | sole ready implementation handoff |
| original M019 plan | superseded and non-executable |
| M019A plan | blocked final internal gate |

## Registry maintenance rules

1. Execute only M018A.
2. Keep M019A blocked until M018A freezes a complete implementation/test head and records `018a-implementation-disposition.md`.
3. Never execute the superseded M019 plan.
4. Do not count base protocol, compatibility aliases, unavailable sources, or unsupported runtimes as canonical operational coverage.
5. Preserve historical closure and invalidation records rather than rewriting them into passing evidence.
6. If the Open proposal changes, rebase the internal contract manifest without contacting or submitting to upstream.
7. M019A must use a distinct auditable internal reviewer and perform read-only source verification.
8. Any high/medium M019A finding returns to M018A when it fits that boundary.
9. Final status may be `closed internally against pinned revision` only with zero unresolved high/medium findings and a no-upstream compliance attestation.
10. Verification remains local and package-scoped; remote CI is not required.
11. No plan may initiate, prepare, request, or imply upstream submission, review, adoption, or merge.