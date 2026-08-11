# I2PControl Proposal 170 RouterInfo Source-Completion Roadmap

Status: partial Proposal 170 support; corrective RouterInfo truthfulness sequence closed

Planning baseline: `b759038` — M044 finalized reviewed head

Corrective baselines:

- `bf9c2eeb` — M045 blocked after stale startup-snapshot rollback;
- `970252c` — merged M053–M052 head reviewed after source-completion closure.

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
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`.

## 1. Purpose

M044 closed the prior corrective sequence with 43 canonical Proposal 170 RouterInfo additions classified as 16 available, 1 protocol-permitted neutral, and 26 unavailable. M045 then failed because its known-peer source retained a startup `CoreSnapshot`. M053 corrected that exact defect with a narrow live `ProfileStorage` inspection seam. M046–M048 subsequently added bounded live active-peer and tunnel observations. M049 added rolling/recent metrics and queue observations; M050 added v4/v6 network state; M051 retained news and banned peers unavailable; M052 accepted an integrated 40 available / 1 neutral / 2 unavailable matrix, which is now historical only.

Post-M052 review at `970252c` found two material semantic defects in that closure:

1. `i2p.router.net.bw.transit.15s` is currently sampled only when the RouterInfo getter is called, so the claimed 15-second router metric depends on API request history rather than router traffic history.
2. `i2p.router.net.error` and `.error.v6` are marked available even though M050 itself records that Emissary has no canonical network-error owner; unset state is mapped to wire code `0` (`No error`), which fabricates a positive semantic claim from source absence.

The review-corrected matrix before implementation of M054/M055 is therefore 37 available, 1 protocol-permitted neutral, and 5 unavailable: news, banned peers, transit 15s, and the two network-error rows. This is a review disposition, not yet the machine-readable contract state; M054/M055 must reconcile code, tests, docs, and source accounting.

The architecture remains asymmetric: core/runtime code may expose only the smallest neutral bounded read-only facts that only a canonical owner can know. I2PControl owns field/source disposition, numeric/wire mapping, JSON serialization, compatibility semantics, aggregate bounds, and sanitized errors.

This roadmap does not reopen unsupported tunnel data planes, `SetConfig`, unrelated base methods, frontend work, release/CI machinery, or upstream integration.

## 2. Corrective target fields

The active corrective sequence touches exactly three previously promoted rows:

1. `i2p.router.net.bw.transit.15s`;
2. `i2p.router.net.error`;
3. `i2p.router.net.error.v6`.

M051's two retained unavailable rows remain unchanged:

4. `i2p.router.news`;
5. `i2p.router.netdb.bannedpeers`.

No other RouterInfo row is reopened unless a direct new defect is independently demonstrated.

## 3. Current-state evidence

Established owners and boundaries:

- `EventHandle` owns cumulative transport/transit counters, connected-router count, tunnel-build counters, firewall status, testing observations, and recent tunnel-build success state;
- `EventManager` already runs as part of `Router`, owns a periodic timer, and reads cumulative transit counters, but its `refresh_interval` is configurable and cannot be assumed to be exactly one second;
- `ProfileStorage` remains the canonical known-router directory, exposed through the accepted M053 read-only inspection handle;
- `TransportInspection` and tunnel inspection sources remain accepted M046–M048 neutral observations;
- SSU2 peer-test activity is a real production owner for v4/v6 testing state;
- no canonical production owner has been identified for the i2pd-style v4/v6 network-error reason state;
- I2PControl already owns the direct Proposal 170 serializers and source inventory.

Read-only reference evidence remains authoritative where adopted: transit 15s is a router-maintained recent bandwidth value, not a request-history measurement; i2pd network error code `0` means `No error` from explicit router state, not `source unavailable`.

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

Rolling-window policy may remain in I2PControl only when the underlying observation is genuinely request-independent. A RouterInfo getter may not create the only history used to answer itself.

### 4.2 Core/runtime exception rule

A change outside I2PControl is authorized only when the canonical owner is the only truthful place to observe a fact. It must be neutral, bounded, passive, and unable to mutate router behavior.

Machine-readable budgets are in `045-052-routerinfo-source-boundary.toml`:

- M054 may touch only `emissary-core/src/events.rs` in core;
- M055 may touch only `events.rs` and `inspection.rs` for removal of unowned error-only scaffolding;
- M056 authorizes no production code.

### 4.3 No parallel authorities

Do not create a second NetDB, transport owner, tunnel owner, reachability engine, news service, ban engine, or persistent metrics service for I2PControl. Do not retain stale one-shot snapshots as live authority. Do not add a dedicated polling daemon or background sampler task merely to raise RouterInfo source counts.

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
M055 — M050 network-error truthfulness corrective — READY
   |
   v
M056 — corrective integration reclosure — CLOSED; no successor currently ready
```

The corrective sequence is serialized to keep one dependency-ready handoff in the registry and to preserve independent closure evidence for each semantic defect.

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

Invalidated finding:

- `i2p.router.net.bw.transit.15s` uses request-driven sampling and therefore lacks request-independent router history.

Corrective plan: `054-m049-transit-15s-corrective.md`. M054 closed the transit
row as explicitly unavailable because the existing configurable event cadence
cannot provide the pinned request-independent semantics within scope.

### M050 — Network status/error/testing — corrected/closed through M055 and M056

Original plan: `050-routerinfo-network-state-sources.md`.

Retained as accepted:

- `i2p.router.net.status.v6`;
- `i2p.router.net.testing`;
- `i2p.router.net.testing.v6`.

Invalidated findings:

- `i2p.router.net.error`;
- `i2p.router.net.error.v6`.

The current implementation has no canonical error owner and maps unset internal state to `0`/`No error`. Corrective plan: `055-m050-network-error-truthfulness-corrective.md`.

### M051 — News and banned-peer semantics — blocked with accepted limitation

Plan: `051-routerinfo-news-and-banned-peer-semantics.md`.

News and banned peers remain unavailable because Emissary has no authoritative news-feed or ban-list owner. Do not create either subsystem solely for telemetry.

### M052 — Integration/containment reclosure — corrected/closed through M056

Original closure remains historical evidence, but its `40/1/2` final matrix is invalidated by the three post-closure source-truthfulness findings. No M052 production code is implicated.

Corrective reclosure: `056-m049-m050-corrective-reclosure.md` after M054 and M055.

### M054 — M049 request-independent transit 15s corrective — closed

Plan: `054-m049-transit-15s-corrective.md`.

The feasibility audit found that the configurable existing event cadence cannot
provide the pinned request-independent semantics without a new timer or
data-plane instrumentation outside scope. The request-local sampler was
removed and the field was demoted to explicit unavailable. No I2PControl-specific
sampler task or tunnel/transport data-plane path was added.

Exit is truthful unavailability with a precise missing-owner reason, direct and
combined no-partial-result regressions, and a static guard against restoring the
request-local sampler.

### M055 — M050 network-error truthfulness corrective — closed

Plan: `055-m050-network-error-truthfulness-corrective.md`.

Audit production writers, then demote both error selectors unless a real existing canonical owner is found. Remove dead error-only atomics/enums/setters where safe, while leaving status/testing observations untouched.

M055's accepted closure demoted both error rows to unavailable because the
production-writer audit found no canonical owner. It also removed the neutral
core error enum, fields, setters, and mapper scaffolding that had no retained
production consumer. Status.v6 and testing v4/v6 remain accepted unchanged.

### M056 — Corrective integration reclosure — closed

Plan: `056-m049-m050-corrective-reclosure.md`.

No production changes. M056 validated the accepted M054/M055 dispositions,
reran retained source/child-process evidence, reconciled all 43 rows, and
superseded only the invalidated M049/M050/M052 findings. The final matrix is 37
available / 1 protocol-permitted neutral / 5 unavailable.

## 8. Failure, cancellation, restart, and contention policy

Read-only snapshots remain bounded and owned. Observation failure never changes router behavior. Peer/tunnel lifecycle semantics from M053/M046–M048 remain unchanged.

For M054, request cancellation or request inactivity must not reset/advance the underlying rolling history. Restart resets process-local state according to pinned reference startup semantics. If exact startup behavior cannot be established, unavailable is preferred to fabricated zero.

For M055, unavailable network-error requests fail before source acquisition and produce no partial result. No new runtime state is required.

## 9. Verification policy

Each corrective milestone runs its semantic regression first, then affected core and CLI feature/no-feature suites. M054 must include no-prior-query and >15-second query-gap tests. M055 must include zero-without-owner and combined-request no-partial-result tests. M056 reruns the broad matrix and existing live child-process path without constructing a new network certification harness.

No new remote CI, coverage, fuzz, soak, network farm, release automation, or generated evidence bundle is required.

## 10. Security and compatibility

Authentication, TLS, tokens, AddressBook authority, tunnel administrative persistence, secrets, and compatibility aliases remain untouched. No schema migration is expected.

Because upstream Emissary is treated as heavily security-reviewed, corrective work should reduce or preserve the current core observation footprint. M055 should remove error-only core scaffolding if it has no production writer rather than leaving speculative telemetry infrastructure behind.

## 11. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Transit metric silently remains API-history dependent | M054 no-prior-query and long-query-gap regressions |
| Existing configurable event interval is assumed to be one second | require real elapsed-time/reference-window proof or demotion |
| A new sampler task is added only for I2PControl | explicit M054 prohibition/path budget |
| Missing error owner still maps to `0` | M055 unavailable disposition + zero-without-owner regression |
| Dead error scaffolding expands audited core | remove it when production-writer audit proves it unused |
| M050 valid status/testing behavior regresses | M055 explicit retained-field fixtures |
| M052 history is rewritten instead of corrected | M056 supersedes only invalidated findings |
| Empty news/ban values used as fake completion | M051 retained semantic gate |
| Core path budget expands | machine-readable M054/M055/M056 budgets and stop conditions |

## 12. Explicit non-goals

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

The pre-review `40 available + 1 neutral + 2 unavailable` matrix is historical
only and is no longer accepted as truthful. M054 and M055 are accepted, and
M056 independently reclosed the integrated disposition as `37 available + 1
neutral + 5 unavailable`: transit 15s, news, banned peers, and both
network-error rows.

M056 derived the final count directly from the machine-readable contract and
production evidence; it did not force either target outcome.

After M056, RouterInfo source completion remains partial because news, banned
peers, transit-15s, and both network-error rows remain unavailable under current
owners. The broader Proposal 170 implementation also retains unrelated
previously accepted unsupported dimensions. No future plan is newly unblocked:
M051 remains blocked until separately authorized substantive news/ban owners
exist.

## 14. Historical corrective sequence

M040–M044 remain closed retained evidence. M039 remains historical-invalidated.
The failed M045 startup-snapshot attempt remains retained evidence explaining
M053. M053/M045 and M046–M048 remain accepted. M049 and M050 retain their
unaffected field closures but are partially superseded as described above.
M052's final source-count claim is superseded by the accepted M056 closure.
