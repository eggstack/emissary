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
| I2PControl Proposal 170 | partial Proposal 170 support; RouterInfo source completion active | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | M052 ready | M051 accepted semantic limitation; M052 follows M045–M051 |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|
| I2PControl Proposal 170 | M048 — tunnel-pool counts and details | closed | `plans/implementation/i2pcontrol-proposal-170/048-routerinfo-tunnel-pool-sources.md` | `048-closure.md` |
| I2PControl Proposal 170 | M052 — 26-source integration and containment reclosure | ready | `plans/implementation/i2pcontrol-proposal-170/052-routerinfo-source-integration-and-reclosure.md` | M045–M051 accepted or semantically blocked |

## Blocked roadmap successors

Per `plans/003-planning-process.md`, these plans exist for deterministic handoff but are not registered as executable until hard dependencies close.

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M045 — known-peer directory | closed | `plans/implementation/i2pcontrol-proposal-170/045-routerinfo-known-peer-directory.md` | corrected through accepted M053 closure |
| M046 — active-peer inventory and transport limits | closed | `plans/implementation/i2pcontrol-proposal-170/046-routerinfo-active-peer-inventory-and-limits.md` | `046-closure.md` |
| M047 — active-peer statistics | closed | `plans/implementation/i2pcontrol-proposal-170/047-routerinfo-active-peer-stats.md` | M046 closure; `047-closure.md` |
| M048 — tunnel-pool counts and details | closed | `plans/implementation/i2pcontrol-proposal-170/048-routerinfo-tunnel-pool-sources.md` | `048-closure.md` |
| M049 — rolling transit/build metrics and queues | closed | `plans/implementation/i2pcontrol-proposal-170/049-routerinfo-rolling-metrics-and-queues.md` | `049-closure.md` |
| M050 — v4/v6 network status/error/testing | closed | `plans/implementation/i2pcontrol-proposal-170/050-routerinfo-network-state-sources.md` | `050-closure.md` |
| M051 — router news and banned-peer semantics | blocked | `plans/implementation/i2pcontrol-proposal-170/051-routerinfo-news-and-banned-peer-semantics.md` | accepted semantic limitation; `051-closure.md` |

## Active closure work

| Subsystem | Handoff | Status | Evidence | Closure record |
|---|---|---|---|---|
| I2PControl Proposal 170 | M046 | closed | implementation `fca7a5f`; finite-limit and active-peer join evidence | `plans/closure/i2pcontrol-proposal-170/046-closure.md` |
| I2PControl Proposal 170 | M047 | closed | bounded current active-session statistics and passive byte observations | `plans/closure/i2pcontrol-proposal-170/047-closure.md` |
| I2PControl Proposal 170 | M048 | closed | bounded live participating/exploratory/client tunnel observations and exact Proposal 170 fixtures | `plans/closure/i2pcontrol-proposal-170/048-closure.md` |
| I2PControl Proposal 170 | M053 / M045 | closed | live-source implementation `09a46cb`; stale attempt `5ae0477` corrected | `plans/closure/i2pcontrol-proposal-170/053-closure.md`; blocked M045 record retained historically |
| I2PControl Proposal 170 | M050 | closed | independent v4/v6 status/error/testing sources and exact integer fixtures | `plans/closure/i2pcontrol-proposal-170/050-closure.md` |
| I2PControl Proposal 170 | M051 | blocked | semantic adjudication retained news and banned peers unavailable; no source owners exist | `plans/closure/i2pcontrol-proposal-170/051-closure.md` |

## Current authorized finding/work scope

Current truthful RouterInfo source matrix:

- 43 canonical Proposal 170 RouterInfo additions;
- 40 available;
- 1 protocol-permitted neutral;
- 2 unavailable.

M045 initially failed because its source retained a one-shot `Router::inspection_snapshot()` from I2PControl startup. M053 corrected that defect with a live canonical `ProfileStorage` inspection handle, and the three known-peer fields are now promoted with post-construction churn evidence.

M053 corrected only the M045 stale-source defect and completed the original three-field capability. M046 added the neutral cloneable current transport inspection source and completed the four active-peer/finite-limit fields. M047 completed the active-peer statistics object from that seam. M048 completed the seven live tunnel-pool sources. M049 completed four rolling/queue sources and unblocked M050. M050 completed the five independently sourced v4/v6 network-state fields and unblocked M051. M051 confirmed that news and banned peers require absent substantive owners, so M052 is ready for final integration/reclosure with the two fields explicitly retained unavailable. Proposal 170 policy remains in I2PControl; core carries only sanitized owned observations.

## M053 containment guard

Machine-readable authority:

- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`

Detailed handoff:

- `plans/implementation/i2pcontrol-proposal-170/053-m045-live-profile-storage-corrective.md`

Authorized core production paths:

- `emissary-core/src/inspection.rs`;
- `emissary-core/src/router/mod.rs`.

Authorized non-core production work:

- `emissary-cli/src/main.rs` composition only;
- `emissary-cli/src/i2pcontrol/**` for adapter/contract/handler behavior.

M053 explicitly does not authorize changes to `emissary-core/src/profile.rs`, `router/context.rs`, NetDB, or `lib.rs` public re-exports. It also does not authorize M046 fields.

Required regression: construct the inspection source, mutate canonical `ProfileStorage` afterward through its existing normal owner/test path, and prove a subsequent snapshot/request using the same source instance observes the new/current public peer data. Startup snapshot fixtures alone are insufficient.

## Prohibited scope throughout the RouterInfo roadmap

- new HTTP, HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, or other tunnel data planes;
- startup task adoption/control;
- router/peer selection, NetDB protocol/discovery, tunnel selection/build/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP behavior changes;
- fabricated RouterInfo values or placeholder promotion;
- public export of mutable ProfileStorage/NetDB/router authority for inspection convenience;
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
| M044 | accepted corrected final-head reclosure | closed; baseline for RouterInfo source work |
| M045 attempt | bounded startup peer snapshot | rejected as stale; retained corrective evidence only |

## Pinned authority

Current work is pinned to Proposal 170 `I2PControl Expansion`, status Open, revision `2026-05-20`. A changed external revision blocks affected implementation/closure and requires an explicit contract-rebase plan.

## Registry maintenance rules

1. M052 is the only current dependency-ready plan.
2. M051 is retained as blocked with an accepted semantic disposition; M052 may validate that disposition.
3. Preserve M020–M045 history/evidence unless a direct new defect is demonstrated.
4. Keep all Proposal 170 policy under I2PControl; core exceptions are neutral observation only and milestone-budgeted.
5. Do not mark a source available before its production owner, exact fixture, bounds, live/churn behavior, and failure semantics are evidenced.
6. Keep verification local/package-scoped; do not add CI/release infrastructure.
7. Overall Proposal 170 status remains partial unless separately authorized work closes unrelated unsupported dimensions.
8. No upstream interaction is authorized.
