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
| I2PControl Proposal 170 | partial Proposal 170 support; RouterInfo source completion active | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | M045 closing | M045 has a named live-source condition; later plans remain blocked |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|
| I2PControl Proposal 170 | — | — | — | — |

## Blocked roadmap successors

Per `plans/003-planning-process.md`, these plans exist for deterministic handoff but are not registered as executable until hard dependencies close.

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M046 — active-peer inventory and transport limits | blocked | `plans/implementation/i2pcontrol-proposal-170/046-routerinfo-active-peer-inventory-and-limits.md` | M045 live-source closure condition |
| M047 — active-peer statistics | blocked | `plans/implementation/i2pcontrol-proposal-170/047-routerinfo-active-peer-stats.md` | M046 closure |
| M048 — tunnel-pool counts and details | blocked | `plans/implementation/i2pcontrol-proposal-170/048-routerinfo-tunnel-pool-sources.md` | M047 closure |
| M049 — rolling transit/build metrics and queues | blocked | `plans/implementation/i2pcontrol-proposal-170/049-routerinfo-rolling-metrics-and-queues.md` | M048 closure |
| M050 — v4/v6 network status/error/testing | blocked | `plans/implementation/i2pcontrol-proposal-170/050-routerinfo-network-state-sources.md` | M049 closure |
| M051 — router news and banned-peer semantics | blocked | `plans/implementation/i2pcontrol-proposal-170/051-routerinfo-news-and-banned-peer-semantics.md` | M050 closure |
| M052 — 26-source integration and containment reclosure | blocked | `plans/implementation/i2pcontrol-proposal-170/052-routerinfo-source-integration-and-reclosure.md` | M045–M051 accepted |

## Active closure work

| Subsystem | Handoff | Status | Evidence | Closure record |
|---|---|---|---|---|
| I2PControl Proposal 170 | M045 | conditionally closed | `plans/closure/i2pcontrol-proposal-170/045-closure.md` | live-source evidence outstanding; M046 remains blocked |

## Current authorized finding/work scope

M044's accepted source matrix, plus the conditionally closed M045 implementation disposition, remains current repository reality:

- 43 canonical Proposal 170 RouterInfo additions;
- 19 available;
- 1 protocol-permitted neutral;
- 23 unavailable.

The maintainer has explicitly authorized work to create truthful sources for the remaining 23 unavailable rows while keeping modifications outside `emissary-cli/src/i2pcontrol/**` minimal.

The source-completion decomposition is:

- M045 known public peer directory: 3;
- M046 active peer inventory + NTCP/SSU limits: 4;
- M047 active peer stats: 1;
- M048 participating/exploratory/client tunnel counts/details: 7;
- M049 transit 15s/recent success/queue/TBM queue: 4;
- M050 v4/v6 status/error/testing: 5;
- M051 news/banned peers: 2;
- M052 final integration/containment review.

## RouterInfo source containment guard

Machine-readable authority:

- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`

General rule: Proposal 170 source policy, aggregation, rolling windows, joins, bounds, wire mappings, serialization, compatibility behavior, and errors remain in `emissary-cli/src/i2pcontrol/**`. Outside changes may only expose neutral bounded read-only facts from canonical owners and must remain inside the active milestone's explicit path budget.

M045 production budget is I2PControl + `main.rs` composition only; no `emissary-core/**` production change is authorized.

Later core exceptions are narrow and milestone-specific. They do not authorize algorithm changes, general management handles, mutable subsystem authority, network probes, or broad refactors.

## Prohibited scope throughout M045–M052

- new HTTP, HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, or other tunnel data planes;
- startup task adoption/control;
- router/peer selection, NetDB protocol/discovery, tunnel selection/build/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP behavior changes;
- fabricated RouterInfo values or placeholder promotion;
- sockets, keys, mutable session/tunnel/transport handles, channels, or message payloads crossing the inspection boundary;
- new network probes, polling daemons, persistent metric stores, news downloader/feed, or ban engine solely for observability;
- AddressBook/`SetConfig`, proxy/UI, frontend, or broad crate/service refactors;
- `.github/workflows/**`, remote CI, release/publishing, coverage, fuzz, soak, platform matrices, or generated evidence bundles;
- upstream issues, pull requests, reviews, submissions, adoption, merge, maintainer contact, or contribution preparation.

## Retained closed evidence

| Milestone | Retained scope | Current qualification |
|---|---|---|
| M020–M030 | wire/auth/base behavior, persistence, AddressBook owner/isolation, original RouterInfo matrix | retained |
| M031–M033 | generic client/server runtimes and lifecycle | retained; outside current roadmap |
| M034–M037 | setter truthfulness, compatibility, auth/publication hardening, containment | retained |
| M038 | bounded live child-process evidence | retained |
| M039 | final review record | historical-invalidated |
| M040–M043 | corrective implementation/regression sequence | closed retained evidence |
| M044 | accepted corrected final-head reclosure | closed; baseline for M045 |

The new roadmap supersedes only the earlier planning statement that the 26 RouterInfo sources were deferred/out of scope. It does not invalidate M044's correctness judgment for the head it reviewed.

## Pinned authority

Current work is pinned to Proposal 170 `I2PControl Expansion`, status Open, revision `2026-05-20`. A changed external revision blocks affected implementation/closure and requires an explicit contract-rebase plan.

## Registry maintenance rules

1. No current implementation plan is dependency-ready; M045's live-source condition must be resolved first.
2. Register M046 only after M045 implementation and independent closure are accepted; continue serially through M052.
3. Preserve M020–M044 history/evidence unless a direct new defect is demonstrated.
4. Keep all Proposal 170 policy under I2PControl; core exceptions are neutral observation only and milestone-budgeted.
5. Do not mark a source available before its production owner, exact fixture, bounds, and failure semantics are evidenced.
6. Keep verification local/package-scoped; do not add CI/release infrastructure.
7. Overall Proposal 170 status remains partial unless separately authorized work closes unrelated unsupported dimensions.
8. No upstream interaction is authorized.
