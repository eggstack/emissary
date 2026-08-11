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
| I2PControl Proposal 170 | partial Proposal 170 support; RouterInfo corrective truthfulness sequence closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | none | M051 remains blocked by absent news/ban owners; no current successor |

## Dependency-ready implementation plans

None. The final M056 integrated reclosure is accepted and no future plan is
newly dependency-ready.

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|

## Blocked roadmap successors

Per `plans/003-planning-process.md`, these plans exist for deterministic handoff but are not registered as executable until hard dependencies close.

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M051 — router news and banned peers | blocked with accepted semantic limitation | `plans/implementation/i2pcontrol-proposal-170/051-routerinfo-news-and-banned-peer-semantics.md` | substantive news/ban owners absent; no current owner-specific plan authorized |

## Retained and corrective milestone disposition

| Handoff | Current status | Evidence / correction |
|---|---|---|
| M045 / M053 — known-peer directory corrective | closed | live `ProfileStorage` source accepted by `053-closure.md` |
| M046 — active-peer inventory and limits | closed | `046-closure.md` |
| M047 — active-peer statistics | closed | `047-closure.md` |
| M048 — tunnel-pool counts/details | closed | `048-closure.md` |
| M049 — rolling metrics/queues | corrected/closed through M054 and M056 | recent success + queue/TBM retained; transit 15s explicitly unavailable |
| M050 — v4/v6 network state | corrected/closed through M055 and M056 | status.v6 + testing v4/v6 retained; error rows unavailable with no canonical owner |
| M051 — news/banned peers | blocked with accepted limitation | `051-closure.md`; both rows remain unavailable |
| M052 — integration reclosure | corrected/closed through M056 | historical `052-closure.md` `40/1/2` matrix superseded by accepted `37/1/5` audit |

## Current review finding/work scope

Repository implementation at review baseline `970252c` still declares:

- 43 canonical Proposal 170 RouterInfo additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

Post-closure source-truthfulness review invalidated three of those `available` claims:

- `i2p.router.net.bw.transit.15s` — current history is created only by RouterInfo requests;
- `i2p.router.net.error` — no canonical error owner; unset state maps to `0` / `No error`;
- `i2p.router.net.error.v6` — same defect.

The accepted M054/M055 dispositions leave the machine-readable implementation matrix at **37 available / 1 neutral / 5 unavailable**: transit 15s, news, banned peers, and both network-error rows are unavailable. The M052 `40/1/2` count remains historical evidence only; M056 accepted the integrated reclosure at that final count.

M054 demoted transit-15s because no request-independent owner fit its bounded
budget. M055 demoted both error rows because no canonical error owner exists.
M056 accepted the final integrated source accounting.

## Corrective containment guard

Machine-readable authority:

- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`

### M054

Authorized core production path:

- `emissary-core/src/events.rs` only.

Authorized non-core production work:

- `emissary-cli/src/i2pcontrol/**`;
- `emissary-cli/src/main.rs` composition only if an already-existing handle needs wiring.

Explicitly forbidden: tunnel/transport/router/NetDB production paths, a new I2PControl-specific sampler task/poller, request-driven history presented as router state.

Required regression: traffic/source history advances with zero RouterInfo transit-15s reads; a later first read and a read after a >15-second API gap still reflect the accepted current rolling semantics.

### M055

Authorized core production paths only for cleanup:

- `emissary-core/src/events.rs`;
- `emissary-core/src/inspection.rs`.

Explicitly forbidden: transport/SSU2 changes to retained status/testing, new error-detection logic/probes, adjacent-signal inference.

Required regression: direct and combined v4/v6 error requests fail unavailable and never serialize code `0` solely because internal state is unset.

### M056

Production changes: none. It is independent corrective reclosure only.

## Prohibited scope throughout the RouterInfo roadmap

- new HTTP, HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, or other unsupported tunnel data planes;
- startup task adoption/control;
- router/peer selection, NetDB protocol/discovery, tunnel selection/build/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP behavior changes;
- fabricated RouterInfo values or placeholder promotion;
- public export of mutable ProfileStorage/NetDB/router authority for inspection convenience;
- sockets, keys, mutable session/tunnel/transport handles, channels, or message payloads crossing the inspection boundary;
- new network probes, polling daemons, persistent metric stores, new background sampler tasks solely for I2PControl, news downloader/feed, or ban engine solely for observability;
- AddressBook/`SetConfig`, proxy/UI, frontend, or broad crate/service refactors;
- `.github/workflows/**`, remote CI, release/publishing, coverage, fuzz, soak, platform matrices, or generated evidence bundles;
- upstream issues, pull requests, reviews, submissions, adoption, merge, maintainer contact, or contribution preparation.

## Retained closed evidence

| Milestone | Retained scope | Current qualification |
|---|---|---|
| M020–M030 | wire/auth/base behavior, persistence, AddressBook owner/isolation, original RouterInfo matrix | retained |
| M031–M033 | generic client/server runtimes and lifecycle | retained; outside current corrective scope |
| M034–M037 | setter truthfulness, compatibility, auth/publication hardening, containment | retained |
| M038 | bounded live child-process evidence | retained |
| M039 | final review record | historical-invalidated |
| M040–M043 | corrective implementation/regression sequence | closed retained evidence |
| M044 | accepted corrected final-head reclosure | closed baseline for RouterInfo source work |
| M045 stale attempt | bounded startup peer snapshot | rejected historical evidence |
| M053 / corrected M045 | live known-peer source | closed retained evidence |
| M046–M048 | active-peer/tunnel source work | closed retained evidence |
| M049 closure | four-field closure | partially invalidated for transit 15s only |
| M050 closure | five-field closure | partially invalidated for error v4/v6 only |
| M051 closure | news/ban semantic limitation | retained |
| M052 closure | integrated `40/1/2` matrix | final source-count finding invalidated pending M056 |

## Pinned authority

Current work is pinned to Proposal 170 `I2PControl Expansion`, status Open, revision `2026-05-20`. A changed external revision blocks affected implementation/closure and requires an explicit contract-rebase plan.

## Registry maintenance rules

1. M056 is closed; no current dependency-ready plan remains.
2. M055 has an accepted closure disposition.
3. M054, M055, and M056 are accepted; no successor is registered as ready.
4. Preserve M053/M045 and M046–M048 closure history unless a direct new defect is demonstrated.
5. Preserve M049/M050 historical records but mark only their named invalidated findings as corrective.
6. Keep all Proposal 170 policy under I2PControl; core exceptions are neutral observation only and milestone-budgeted.
7. Do not mark a source available before its production owner, exact fixture, bounds, live/request-independence behavior, and failure semantics are evidenced.
8. Keep verification local/package-scoped; do not add CI/release infrastructure.
9. Overall Proposal 170 remains partial after this sequence unless separately authorized work closes unrelated unsupported dimensions.
10. No upstream interaction is authorized.
