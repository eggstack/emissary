# I2PControl Proposal 170 RouterInfo Source-Completion Roadmap

Status: partial Proposal 170 support; M057 post-M056 planning-record consistency corrective closed

Planning baseline: `b759038` — M044 finalized reviewed head

Corrective baselines:

- `bf9c2eeb` — M045 blocked after stale startup-snapshot rollback;
- `970252c` — merged M053–M052 head reviewed after source-completion closure;
- `cdbc3a4` — merged M054–M056 corrective implementation/reclosure head and M057 planning baseline.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`;
- status: `Open`;
- revision created/updated `2026-05-20`;
- existing I2PControl authentication and JSON-RPC contract;
- read-only i2pd/reference implementation where Proposal 170 explicitly adopts or leaves semantics terse.

Canonical internal references:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/closure/i2pcontrol-proposal-170/044-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/045-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/049-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/050-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/052-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/053-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/054-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/055-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/056-closure.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`;
- `plans/implementation/i2pcontrol-proposal-170/057-post-m056-planning-record-consistency-corrective.md`.

## 1. Purpose

M044 closed the prior corrective sequence with 43 canonical Proposal 170 RouterInfo additions classified as 16 available, 1 protocol-permitted neutral, and 26 unavailable. M045 then failed because its known-peer source retained a startup `CoreSnapshot`. M053 corrected that exact defect with a narrow live `ProfileStorage` inspection seam. M046–M048 subsequently added bounded live active-peer and tunnel observations. M049 added rolling/recent metrics and queue observations; M050 added v4/v6 network state; M051 retained news and banned peers unavailable; M052 accepted an integrated 40 available / 1 neutral / 2 unavailable matrix, which is now historical only.

Post-M052 review at `970252c` found two material semantic defects in that closure:

1. `i2p.router.net.bw.transit.15s` was sampled only when the RouterInfo getter was called, so the claimed 15-second router metric depended on API request history rather than router traffic history.
2. `i2p.router.net.error` and `.error.v6` were marked available even though M050 itself recorded that Emissary had no canonical network-error owner; unset state was mapped to wire code `0` (`No error`), fabricating a positive semantic claim from source absence.

M054 and M055 corrected those findings by demoting the three unsupported rows rather than broadening audited core behavior. M056 accepted the integrated final RouterInfo matrix as 37 available, 1 protocol-permitted neutral, and 5 unavailable: news, banned peers, transit 15s, and the two network-error rows.

M057 is a planning-record-only follow-up. It does not reopen production behavior. It exists because a small number of active planning statements remained stale after M056, including a dependency-graph M055 readiness label and historical/current baseline wording. M057 must reconcile those records without changing the accepted 37/1/5 disposition.

The architecture remains asymmetric: core/runtime code may expose only the smallest neutral bounded read-only facts that only a canonical owner can know. I2PControl owns field/source disposition, numeric/wire mapping, JSON serialization, compatibility semantics, aggregate bounds, and sanitized errors.

This roadmap does not reopen unsupported tunnel data planes, `SetConfig`, unrelated base methods, frontend work, release/CI machinery, or upstream integration.

## 2. Corrective target fields and records

The production corrective sequence touched exactly three previously promoted RouterInfo rows:

1. `i2p.router.net.bw.transit.15s`;
2. `i2p.router.net.error`;
3. `i2p.router.net.error.v6`.

M051's two retained unavailable rows remain unchanged:

4. `i2p.router.news`;
5. `i2p.router.netdb.bannedpeers`.

M057 touches no RouterInfo field. Its target is active planning-record consistency only:

- milestone lifecycle/status wording after accepted M054–M056 closure;
- historical `970252c` 40/1/2 wording versus final post-M056 37/1/5 wording;
- stale current-state lifecycle references for M056, if any remain.

No other RouterInfo row or runtime is reopened unless a direct new defect is independently demonstrated and separately planned.

## 3. Current-state evidence

Established owners and boundaries:

- `EventHandle` owns cumulative transport/transit counters, connected-router count, tunnel-build counters, firewall status, testing observations, and recent tunnel-build success state;
- `ProfileStorage` remains the canonical known-router directory, exposed through the accepted M053 read-only inspection handle;
- `TransportInspection` and tunnel inspection sources remain accepted M046–M048 neutral observations;
- SSU2 peer-test activity is a real production owner for v4/v6 testing state;
- no canonical production owner exists for the adopted v4/v6 network-error reason state;
- no request-independent 15-second transit-bandwidth owner fits the accepted bounded scope;
- I2PControl owns the direct Proposal 170 serializers and source inventory;
- M056 independently reconciled the machine-readable contract to 43 total / 37 available / 1 neutral / 5 unavailable.

Read-only reference evidence remains authoritative where adopted: transit 15s is a router-maintained recent bandwidth value, not a request-history measurement; i2pd network error code `0` means `No error` from explicit router state, not `source unavailable`.

The remaining M057 evidence defect is not production state. It is active planning metadata that did not fully converge on the accepted M056 lifecycle/history.

## 4. Ownership architecture

### 4.1 I2PControl-owned policy

The following remain under `emissary-cli/src/i2pcontrol/**`:

- Proposal 170 key/type/source inventory;
- request grouping/assembly;
- source disposition and unavailable behavior;
- deterministic sorting/deduplication;
- collection/serialized-size bounds;
- Base64/numeric wire mapping;
- JSON-RPC compatibility and direct-presence semantics;
- sanitized error translation.

M057 has no authority to modify any of these.

### 4.2 Core/runtime exception rule

A change outside I2PControl is authorized only when the canonical owner is the only truthful place to observe a fact. It must be neutral, bounded, passive, and unable to mutate router behavior.

Machine-readable budgets are in `045-052-routerinfo-source-boundary.toml`:

- M054 may touch only `emissary-core/src/events.rs` in core;
- M055 may touch only `events.rs` and `inspection.rs` for removal of unowned error-only scaffolding;
- M056 authorizes no production code;
- M057 authorizes no production code or core path.

### 4.3 No parallel authorities

Do not create a second NetDB, transport owner, tunnel owner, reachability engine, news service, ban engine, or persistent metrics service for I2PControl. Do not retain stale one-shot snapshots as live authority. Do not add a dedicated polling daemon or background sampler task merely to raise RouterInfo source counts.

M057 specifically must not turn a planning cleanup into a production follow-up.

## 5. Cross-cutting invariants

1. Exact Proposal 170 names, casing, JSON types, and direct-presence semantics remain unchanged.
2. No fabricated zero, false, empty, null, or adjacent metric replaces unavailable state.
3. A serializer is not evidence of a production source owner.
4. Request frequency must not determine a router-owned rolling metric.
5. Missing network-error authority must not serialize as `No error`.
6. Default/no-I2PControl router execution gains no I2PControl-specific task, poller, probe, or persistent state.
7. Core observation cannot change routing, peer selection, NetDB, transport, tunnel building, congestion, retry, timing, cryptographic, or LeaseSet behavior.
8. No private/session key, destination private material, socket, mutable session/tunnel/router object, command channel, or message payload crosses an inspection boundary.
9. Collections/histories are bounded and locks do not cross `.await`, sleep, network I/O, or JSON serialization.
10. Partial/incomplete observation fails closed.
11. Compatibility aliases/base selectors are not expanded by these corrective passes.
12. No tunnel data plane, AddressBook/`SetConfig`, frontend, workflow/release, or upstream activity is authorized.
13. M057 may change planning/control-surface records only; the final 37/1/5 matrix is invariant under M057.

## 6. Dependency graph

```text
M044 closed
   |
   v
M053 / corrected M045 — closed
   |
   v
M046 — closed
   |
   v
M047 — closed
   |
   v
M048 — closed
   |
   v
M049 — partially invalidated: transit 15s only
   |
   v
M050 — partially invalidated: error v4/v6 only
   |
   v
M051 — blocked with accepted news/ban semantic limitation
   |
   v
M052 — final source-accounting closure invalidated by post-closure review
   |
   v
M054 — M049 transit-15s corrective — CLOSED
   |
   v
M055 — M050 network-error truthfulness corrective — CLOSED
   |
   v
M056 — corrective integration reclosure — CLOSED
   |
   v
M057 — post-M056 planning-record consistency corrective — CLOSED; no dependency-ready successor
```

M054, M055, and M056 are accepted closed milestones. M057 was the bounded planning-record corrective handoff and is now closed; no dependency-ready successor is registered. M051 remains blocked with its accepted semantic limitation and has no substantive owner-specific successor plan.

## 7. Milestones

### M053 — M045 live ProfileStorage corrective — closed

Plan: `053-m045-live-profile-storage-corrective.md`.

M053 remains accepted. It created the bounded live known-peer inspection seam and the required post-construction churn regression. No current corrective finding reopens it.

### M045 — Known-peer directory — corrected/closed through M053

Fields: `netdb.peers`, `netdb.peers.list`, `netdb.peers.info`. Retained.

### M046 — Active-peer inventory and finite limits — closed

Fields: active peer list/info and finite NTCP/SSU limits. Retained.

### M047 — Active-peer statistics — closed

Field: `netdb.activepeers.stats`. Retained. Richer unstable reference-map schemas remain an interoperability observation, not a current formal Proposal 170 violation.

### M048 — Tunnel-pool counts/details — closed

Seven participating/exploratory/client count/detail fields remain accepted. Minimal map member shape remains an interoperability observation because the pinned proposal defines list-of-map types but not stable member names.

### M049 — Rolling metrics and queues — corrected/closed through M054 and M056

Original plan: `049-routerinfo-rolling-metrics-and-queues.md`.

Retained as accepted:

- recent tunnel success rate;
- tunnel queue depth;
- TBM queue depth.

The invalidated transit-15s claim is corrected by M054 to explicit unavailability.

### M050 — Network status/error/testing — corrected/closed through M055 and M056

Original plan: `050-routerinfo-network-state-sources.md`.

Retained as accepted:

- `i2p.router.net.status.v6`;
- `i2p.router.net.testing`;
- `i2p.router.net.testing.v6`.

The invalidated v4/v6 error claims are corrected by M055 to explicit unavailability.

### M051 — News and banned-peer semantics — blocked with accepted limitation

Plan: `051-routerinfo-news-and-banned-peer-semantics.md`.

News and banned peers remain unavailable because Emissary has no authoritative news-feed or ban-list owner. Do not create either subsystem solely for telemetry.

### M052 — Integration/containment reclosure — corrected/closed through M056

Original closure remains historical evidence, but its `40/1/2` final matrix is superseded by the M056 `37/1/5` integrated reclosure. No M052 production code is implicated.

### M054 — M049 request-independent transit 15s corrective — closed

Plan: `054-m049-transit-15s-corrective.md`.

The feasibility audit found that the configurable existing event cadence cannot provide the pinned request-independent semantics without a new timer or data-plane instrumentation outside scope. The request-local sampler was removed and the field was demoted to explicit unavailable.

### M055 — M050 network-error truthfulness corrective — closed

Plan: `055-m050-network-error-truthfulness-corrective.md`.

The production-writer audit found no canonical owner. Both error rows were demoted unavailable and the unused neutral core error scaffolding was removed. Status.v6 and testing v4/v6 remain accepted unchanged.

### M056 — Corrective integration reclosure — closed

Plan: `056-m049-m050-corrective-reclosure.md`.

No production changes. M056 validated the accepted M054/M055 dispositions, reconciled all 43 rows, and superseded only the invalidated M049/M050/M052 findings. Final matrix: 37 available / 1 protocol-permitted neutral / 5 unavailable.

### M057 — Post-M056 planning-record consistency corrective — closed

Plan: `057-post-m056-planning-record-consistency-corrective.md`.

Correct only active planning/control-surface drift after accepted M056 closure. Required targets include the stale M055 readiness label in this roadmap dependency graph and any historical/current source-count wording that conflates `970252c`'s pre-corrective 40/1/2 claim with the final post-M056 37/1/5 disposition.

M057 authorizes no production Rust, runtime, test-behavior, workflow, release, or source-disposition change. Closure is based on changed-path review, targeted planning-status/baseline searches, `git diff --check`, preservation of accepted closure records, and explicit internal-only attestation.

Exit: active registry/roadmap/index agree that M054–M057 are closed, `970252c` is historical 40/1/2 evidence, the accepted current matrix is 37/1/5, M051 remains blocked, and no dependency-ready successor remains.

## 8. Failure, cancellation, restart, and contention policy

Read-only snapshots remain bounded and owned. Observation failure never changes router behavior. Peer/tunnel lifecycle semantics from M053/M046–M048 remain unchanged.

M057 has no runtime lifecycle, cancellation, restart, or contention semantics because it changes planning records only. Any implementation that introduces such semantics has exceeded scope.

## 9. Verification policy

M054/M055/M056 retain their accepted production verification evidence. M057 must not rerun or expand the broad Rust matrix solely for documentation changes.

M057 verification is limited to:

- `git diff --check`;
- changed-path audit proving no production/runtime/workflow changes;
- targeted active-planning searches for stale M055/M056 lifecycle wording;
- targeted historical/current count searches proving `970252c` is described as historical 40/1/2 and post-M056 state as 37/1/5;
- closure-record preservation review.

No new remote CI, coverage, fuzz, soak, network farm, release automation, or generated evidence bundle is required.

## 10. Security and compatibility

Authentication, TLS, tokens, AddressBook authority, tunnel administrative persistence, secrets, compatibility aliases, and every production source disposition remain untouched. No schema migration is expected.

Because upstream Emissary is treated as heavily security-reviewed, M057 has zero production authority. Its only security effect is preventing stale planning metadata from causing a future agent to reopen already closed audited-core work.

## 11. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Transit metric silently returns to API-history dependence | retained M054 closure/static guards; M057 cannot change production |
| Missing error owner returns as `0` | retained M055 closure/static guards; M057 cannot change production |
| M055/M056 lifecycle status remains contradictory in active planning | M057 targeted status scan and explicit acceptance criterion |
| Historical `970252c` state is conflated with final M056 state | M057 baseline/count reconciliation |
| Historical closure evidence is rewritten | M057 explicitly preserves accepted closure records |
| Empty news/ban values used as fake completion | M051 retained semantic gate |
| Core path budget expands | M057 machine-readable zero-production budget |

## 12. Explicit non-goals

- any production or runtime change;
- any RouterInfo source-disposition change;
- unsupported Proposal 170 tunnel data planes;
- startup tunnel task adoption/control;
- AddressBook or `SetConfig` changes;
- base I2PControl unsupported methods;
- router/transport/tunnel/NetDB algorithm redesign;
- new reachability probes or error-detection subsystem;
- news downloader/feed or ban engine solely for telemetry;
- new background sampler/polling service solely for I2PControl;
- UI/frontend work;
- crate-wide refactor;
- CI/release/publishing expansion;
- upstream issues, PRs, reviews, submissions, adoption, merge requests, maintainer outreach, or contribution preparation.

## 13. Final status rule

The pre-corrective `40 available + 1 neutral + 2 unavailable` matrix is historical only and is no longer accepted as truthful. M054 and M055 are accepted, and M056 independently reclosed the integrated disposition as `37 available + 1 neutral + 5 unavailable`: transit 15s, news, banned peers, and both network-error rows.

M057 cannot change this source matrix. Its only allowed completion is a planning-record reclosure in which active documents consistently distinguish the historical `970252c` 40/1/2 claim from the accepted post-M056 37/1/5 state and consistently label M054–M056 closed.

M057 is closed and no dependency-ready plan is registered. M051 remains blocked until separately authorized substantive news/ban owners exist.

RouterInfo source completion and broader Proposal 170 support remain partial under the accepted scope.

## 14. Historical corrective sequence

M040–M044 remain closed retained evidence. M039 remains historical-invalidated. The failed M045 startup-snapshot attempt remains retained evidence explaining M053. M053/M045 and M046–M048 remain accepted. M049 and M050 retain their unaffected field closures but are partially superseded by M054/M055/M056. M052's final source-count claim is superseded by the accepted M056 closure. M057 is the active planning-record consistency follow-up and does not alter that production history.
