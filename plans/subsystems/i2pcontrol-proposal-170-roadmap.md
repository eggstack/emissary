# I2PControl Proposal 170 RouterInfo Source-Completion Roadmap

Status: partial Proposal 170 support; RouterInfo source-completion sequence closed with accepted semantic limitation

Planning baseline: `b759038` — M044 finalized reviewed head

Corrective baseline: `bf9c2eeb` — M045 blocked after stale startup-snapshot rollback

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`;
- status: `Open`;
- revision created/updated `2026-05-20`;
- existing I2PControl authentication and JSON-RPC contract.

Canonical internal references:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/closure/i2pcontrol-proposal-170/044-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/045-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/049-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/053-closure.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`;
- `plans/implementation/i2pcontrol-proposal-170/053-m045-live-profile-storage-corrective.md`.

## 1. Purpose

M044 truthfully closed the prior corrective sequence with 43 canonical Proposal 170 RouterInfo additions classified as 16 available, 1 protocol-permitted neutral, and 26 unavailable. M045 then attempted to implement three known-peer fields using a bounded `Router::inspection_snapshot()` captured at I2PControl startup. Closure rejected that source because it was safe but stale: peer churn after composition was not visible.

M053 corrected that exact defect with the smallest neutral live core inspection seam required to expose the canonical `ProfileStorage` directory without making `ProfileStorage`, `Bucket`, NetDB authority, or mutable router state public. M045 is corrected/closed through the accepted M053 closure. M046 then added the neutral live transport inspection seam and completed active-peer inventory and finite-limit sources. M047 added bounded current active-session statistics; M048 added bounded live tunnel-pool sources; M049 added rolling transit/recent-success metrics and live queue depths; M050 added independent v4/v6 network state. The matrix is now 40 available, 1 protocol-permitted neutral, and 2 unavailable.

The architecture remains asymmetric: core/runtime code may expose only the smallest neutral bounded read-only facts that only the canonical owner can know. I2PControl owns source composition, rolling windows, aggregation, deterministic ordering, response bounds, Proposal 170 source disposition, numeric/wire mapping, JSON serialization, and failure semantics.

This roadmap does not reopen the ten unsupported tunnel data planes, `SetConfig`, base unsupported I2PControl methods, frontend work, or upstream integration.

## 2. Target fields

The 2 remaining target rows are exactly the current unavailable rows in `router_info_keys::PROPOSAL_170_CONTRACT` and `docs/i2pcontrol/router-info-source-map.md`:

1. `i2p.router.news`;
2. `i2p.router.netdb.bannedpeers`.

No additional RouterInfo/base selector work is authorized unless a direct regression is discovered and separately planned.

## 3. Current-state evidence

Existing code already provides the necessary ownership facts:

- `EventHandle` owns cumulative transport/transit byte counters, connected-router count, transit-tunnel count, cumulative tunnel-build success/failure counts, and v4/v6 firewall status;
- `RouterContext::profile_storage()` is the canonical known-router directory and provides public serialized RouterInfo inside core;
- `ProfileStorage::get_router_ids(Bucket::Any, ...)`, `get_raw()`, and `reader()` provide the internal primitives needed for a live request-time peer-directory snapshot;
- the obstacle is visibility, not missing data: `profile` is private to `emissary-core`, so `emissary-cli` cannot use `Bucket` directly;
- `emissary-core/src/inspection.rs` already establishes the correct neutral DTO boundary and is public;
- `Router::inspection_snapshot()` is intentionally one-shot and therefore cannot be reused as the M045 live source;
- `TransportManager` owns current connected peers and already has a bounded peer-ID snapshot helper for later milestones;
- tunnel pools/transit owners contain later lifecycle information but no suitable cloneable aggregate yet;
- I2PControl already owns canonical serializers and `RouterInfoControl`, so wire policy need not move into core.

Read-only i2pd reference inspection remains the semantic authority for adopted fields where Proposal 170 is terse: transit 15s is rolling, recent tunnel success is distinct from cumulative success, queue/TBM queue are instantaneous depths, and v4/v6 status/error/testing are independent state.

## 4. Ownership architecture

### 4.1 I2PControl-owned policy

The following remain under `emissary-cli/src/i2pcontrol/**`:

- Proposal 170 key/type/source inventory;
- request assembly and source grouping;
- rolling-window policy and aggregation;
- deterministic sorting/deduplication;
- peer-directory joins and completeness policy;
- collection/serialized-size bounds;
- Base64 and numeric wire mappings;
- sanitized error translation;
- JSON-RPC and compatibility semantics;
- source availability/disposition.

### 4.2 Core/runtime exception rule

A change outside `i2pcontrol/**` is authorized only when the canonical owner is the only truthful place to observe a fact. Such changes must be neutral immutable DTOs, cloneable read-only inspection handles, passive bounded lifecycle/stat publication at existing transitions, or composition-only wiring in `main.rs`.

M053 is the first corrective exception and is deliberately narrower than the later generic budgets: only `emissary-core/src/inspection.rs` and `emissary-core/src/router/mod.rs` are authorized core production paths. `profile.rs`, `router/context.rs`, `lib.rs` re-export expansion, and NetDB are explicitly forbidden unless another corrective disposition is created.

The exact budgets are machine-readable in `045-052-routerinfo-source-boundary.toml`.

### 4.3 No parallel authorities

Do not build a second NetDB, peer, tunnel, reachability, or metrics cache when an existing owner can be inspected directly. A request-time inspection handle may retain a clone of the canonical owner privately inside core, but it must expose only owned bounded public snapshots and no mutation methods.

The one-shot `CoreSnapshot` must not become a runtime authority or be retained by I2PControl as a supposedly live source.

## 5. Cross-cutting invariants

1. Exact Proposal 170 names, casing, types, and presence semantics remain unchanged.
2. No fabricated zero, false, empty, null, or adjacent metric may replace unavailable state.
3. Default/no-feature router execution performs no background I2PControl work.
4. Core observation cannot change routing, peer selection, NetDB, transport, tunnel-building, congestion, retry, timing, cryptographic, or LeaseSet behavior.
5. No private/session key, destination private material, socket, mutable session/tunnel/router object, command channel, or message payload crosses an inspection boundary.
6. All collections and histories are bounded; handlers retain no unbounded time series.
7. No lock is held across network I/O, sleep, cancellation, `.await`, or JSON serialization.
8. Partial/incomplete observation fails closed rather than returning plausible partial data.
9. Source availability changes only after production source plus exact regression/contract fixtures exist.
10. Compatibility aliases/base nested selectors are not expanded or silently changed.
11. No tunnel data plane, `SetConfig`, frontend, workflow/release, broad refactor, or upstream activity is authorized.

## 6. Dependency graph

```text
M044 closed
   |
   v
M053 M045 live ProfileStorage corrective — closed
   |
   v
M045 known peer directory (3) corrected/closed
   |
   v
M046 active peers + limits (4) — closed
   |
   v
M047 active peer stats (1) — closed
   |
   v
M048 tunnel pool counts/details (7) — closed
   |
   v
M049 rolling metrics + queues (4)
   |
   v
M050 v4/v6 network state (5)
   |
   v
M051 news + banned-peer semantics (2)
   |
   v
M052 integration/containment reclosure
```

The sequence remains serialized so every new audited-core observation seam receives independent closure before the next one is introduced. M050 is closed, M051 is blocked with an accepted semantic disposition, and M052 is now closed after validating the final integrated head. No dependency-ready successor remains because the two M051 fields require substantive owners absent from Emissary.

## 7. Milestones

### M053 — M045 live ProfileStorage corrective — closed

Plan: `053-m045-live-profile-storage-corrective.md`.

Corrective target: the three M045 known-peer fields.

Create a neutral cloneable request-time live inspection handle in `emissary-core::inspection`, expose it through one read-only `Router` method, compose it into I2PControl, and prove the same source instance observes canonical peer-directory mutation after construction. Core production budget is exactly `inspection.rs` + `router/mod.rs`; do not modify or publicly export `ProfileStorage`/`Bucket`.

Exit: M045's stale-source defect is corrected, the three selectors are live/bounded/exact, source accounting becomes 19 available + 1 neutral + 23 unavailable, and an independent closure accepts the core path audit and post-construction churn regression.

### M045 — Known-peer directory sources — corrected/closed through M053

Plan: `045-routerinfo-known-peer-directory.md`.

Fields: `netdb.peers`, `netdb.peers.list`, `netdb.peers.info`.

The zero-core-change assumption was disproven by closure. M053 is the authorized corrective expansion, and its independent closure explicitly accepts the live source and closes M045.

### M046 — Active-peer inventory and limits — closed

Plan: `046-routerinfo-active-peer-inventory-and-limits.md`.

Fields: active peer list/info and NTCP/SSU limits. M046 added the minimum neutral cloneable transport-inspection source, joined active IDs to the live public RouterInfo directory, and retained unlimited/disabled limits as unavailable rather than inventing a sentinel. Closure: `plans/closure/i2pcontrol-proposal-170/046-closure.md`.

### M047 — Active-peer statistics — closed

Plan: `047-routerinfo-active-peer-stats.md`.

Field: `netdb.activepeers.stats`. Audit each required object field to a canonical NTCP2/SSU2 owner and extend neutral observation only where passive capture is sufficient.

M046 is closed; M047's field-owner audit was accepted through `plans/closure/i2pcontrol-proposal-170/047-closure.md`. Its source is the accepted neutral transport seam plus passive counters at existing NTCP2/SSU2 byte-accounting points.

### M048 — Tunnel-pool counts/details — closed

Plan: `048-routerinfo-tunnel-pool-sources.md`.

Fields: participating detail; exploratory in/out/detail; client in/out/detail. Implemented through a neutral bounded lifecycle source shared by canonical pool/transit owners; I2PControl owns grouping, counts, deterministic row mapping, and response bounds. Closure: `plans/closure/i2pcontrol-proposal-170/048-closure.md`.

### M049 — Rolling metrics and queues — closed

Plan: `049-routerinfo-rolling-metrics-and-queues.md`.

Fields: transit 15s, recent tunnel success, queue, TBM queue. Compute rolling transit in I2PControl; match reference recent-success semantics; add only neutral core gauges that cannot be reconstructed truthfully.

Closure: `plans/closure/i2pcontrol-proposal-170/049-closure.md`.

### M050 — Network status/error/testing — closed

Plan: `050-routerinfo-network-state-sources.md`.

Fields: status.v6; error v4/v6; testing v4/v6. Track independent neutral state at existing reachability transitions and map to wire integers only in I2PControl. No new probes.

Closure: `plans/closure/i2pcontrol-proposal-170/050-closure.md`.

### M051 — News and banned-peer semantics — blocked by accepted semantic limitation

Plan: `051-routerinfo-news-and-banned-peer-semantics.md`.

Fields: router news and banned peers. The pinned proposal specifies the string
and map-of-objects wire types but does not authorize capability-empty values.
The read-only Java reference requires a NewsFeedHelper owner for news and a
Banlist owner for bans; Emissary has neither. Both fields therefore remain
unavailable. Do not add a news service or ban engine solely for telemetry.

Closure: `plans/closure/i2pcontrol-proposal-170/051-closure.md`.

### M052 — 26-source integration and reclosure — closed with accepted semantic limitation

Plan: `052-routerinfo-source-integration-and-reclosure.md`.

No production changes. M052 reviewed all 26 target rows, retained rows, child-process behavior, bounds, failure semantics, no-feature behavior, and every core changed path. M051's two semantic limitations are accepted inputs, so the RouterInfo dimension remains incomplete. Closure evidence is in `plans/closure/i2pcontrol-proposal-170/052-closure.md`; a new defect requires another corrective plan.

## 8. Failure, cancellation, restart, and contention policy

Read-only snapshots are request-scoped and owned. Passive observers use bounded state and must not block producers. Observation failure never changes router behavior; if it makes an API source incomplete, I2PControl fails the affected request until authoritative recovery.

M053 specifically requires live peer-directory snapshots from the same retained inspection source after canonical storage mutation. Peer churn between enumeration and raw-RouterInfo copy must be handled fail-closed when a complete join cannot be constructed. No lock escapes core or survives into async JSON-RPC work.

Rolling metrics are process-local and reset safely on counter rollback/restart. No new persisted state is required by this roadmap.

## 9. Verification policy

Each implementation milestone runs focused regression/contract tests first, then affected core tests and the bounded CLI feature/no-feature matrix. M053 must include a failing-before/passing-after post-construction peer-churn regression. M052 reruns the broad matrix and live child-process test.

No new remote CI, coverage, fuzz, soak, network farm, release automation, or generated evidence bundle is required.

Static guards should enforce no Proposal 170 terminology in core DTOs, no sensitive/mutable handles in inspection contracts, changed-path compliance, no background resource allocation in read-only handles, and no source promotion before production evidence.

## 10. Security and compatibility

The work is read-only observability. Authentication, TLS, tokens, AddressBook authority, tunnel administrative persistence, server secrets, and compatibility aliases remain untouched. No schema migration is expected.

Because upstream Emissary is treated as heavily security-reviewed, a small purpose-specific neutral inspection seam is preferable to making `ProfileStorage`, `Bucket`, NetDB handles, or broad router internals public. Code reuse is not sufficient reason to alter audited data-plane logic.

## 11. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Live inspection becomes a general core management API | M053 exposes only bounded owned snapshots; no mutable methods or public ProfileStorage |
| Startup snapshot is accidentally retained again | required source-then-mutate-then-resnapshot regression |
| Peer churn yields fabricated complete join | fail closed on missing required raw RouterInfo |
| Core path budget expands | M053 exact two-file budget + explicit stop condition |
| Recent success implemented as cumulative success | M049 reference fixture requires distinct recent semantics |
| Tunnel placeholders become claimed values | M048 lifecycle evidence required |
| Active peer stats leak session details | sanitized neutral DTO + static type review |
| Unlimited transport limits get invented sentinel | M046 stop rule |
| Network testing/error inferred from firewall status | M050 requires independent state |
| Empty news/ban values used as fake completion | M051 semantic adjudication gate |

## 12. Explicit non-goals

- the ten unsupported Proposal 170 tunnel data planes;
- startup tunnel task adoption/control;
- AddressBook or `SetConfig` changes;
- base I2PControl unsupported methods;
- router/transport/tunnel/NetDB algorithm redesign;
- public export of `ProfileStorage`/`Bucket` merely for M045;
- new reachability probes, news downloader, or ban engine solely for telemetry;
- UI/frontend work;
- crate-wide extraction/refactor;
- CI/release/publishing expansion;
- upstream issues, PRs, reviews, submissions, adoption, merge requests, maintainer outreach, or contribution preparation.

## 13. Final status rule

If M046–M051 make all remaining fields operational and M052 accepts the final head, the RouterInfo dimension may be recorded as 42 available + 1 protocol-permitted neutral + 0 unavailable and `RouterInfo source completion closed internally against pinned revision`. That condition was not met: the accepted M051 limitation leaves the final matrix at 40 available + 1 protocol-permitted neutral + 2 unavailable.

That does not by itself make the entire Proposal 170 implementation complete. Unrelated accepted unsupported/runtime dimensions remain outside this roadmap.

M051 proved both remaining fields require absent substantive subsystems that this
roadmap refuses to add; retain them unavailable rather than broadening scope or
fabricating support.

## 14. Historical corrective sequence

M040–M044 remain closed retained evidence. M039 remains historical-invalidated. The failed M045 startup-snapshot attempt and blocked closure remain retained evidence explaining M053. M053 supersedes only M045's disproven zero-core-change assumption; it does not rewrite earlier closure history.
