# I2PControl Proposal 170 RouterInfo Source-Completion Roadmap

Status: partial Proposal 170 support; RouterInfo source-completion sequence blocked; M045 corrective seam required

Planning baseline: `b759038` — M044 finalized reviewed head

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
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`.

## 1. Purpose

M044 truthfully closed the prior corrective sequence with 43 canonical Proposal 170 RouterInfo additions classified as 16 available, 1 protocol-permitted neutral, and 26 unavailable. A prior M045 attempt added a bounded startup inspection-snapshot adapter, but closure review rejected it because it is not a live ProfileStorage source. The three known-peer fields therefore remain unavailable, leaving 16 available, 1 protocol-permitted neutral, and 26 unavailable. This roadmap remains limited to creating truthful sources for the remaining additions while minimizing changes outside `emissary-cli/src/i2pcontrol/**`.

The architecture is intentionally asymmetric: core/runtime code may expose only the smallest neutral bounded read-only facts that only the canonical owner can know. I2PControl owns source composition, rolling windows, aggregation, deterministic ordering, response bounds, Proposal 170 source disposition, numeric/wire mapping, JSON serialization, and failure semantics.

This roadmap does not reopen the ten unsupported tunnel data planes, `SetConfig`, base unsupported I2PControl methods, frontend work, or upstream integration.

## 2. Target fields

The 26 target rows are exactly the current unavailable rows in `router_info_keys::PROPOSAL_170_CONTRACT` and `docs/i2pcontrol/router-info-source-map.md`:

1. `i2p.router.news`;
2. `i2p.router.net.bw.transit.15s`;
3. `i2p.router.net.tunnels.participating.info`;
4. `i2p.router.net.tunnels.exploratory.inbound`;
5. `i2p.router.net.tunnels.exploratory.outbound`;
6. `i2p.router.net.tunnels.exploratory.info.list`;
7. `i2p.router.net.tunnels.client.inbound`;
8. `i2p.router.net.tunnels.client.outbound`;
9. `i2p.router.net.tunnels.client.info.list`;
10. `i2p.router.net.status.v6`;
11. `i2p.router.net.error`;
12. `i2p.router.net.error.v6`;
13. `i2p.router.net.testing`;
14. `i2p.router.net.testing.v6`;
15. `i2p.router.net.tunnels.successrate`;
16. `i2p.router.net.tunnels.queue`;
17. `i2p.router.net.tunnels.tbmqueue`;
18. `i2p.router.netdb.peers`;
19. `i2p.router.netdb.activepeers.info`;
20. `i2p.router.netdb.ntcp.limit`;
21. `i2p.router.netdb.ssu.limit`;
22. `i2p.router.netdb.bannedpeers`;
23. `i2p.router.netdb.activepeers.list`;
24. `i2p.router.netdb.peers.list`;
25. `i2p.router.netdb.peers.info`;
26. `i2p.router.netdb.activepeers.stats`.

No additional RouterInfo/base selector work is authorized unless a direct regression is discovered and separately planned.

## 3. Current-state evidence

Existing code already provides useful but incomplete neutral primitives:

- `EventHandle` owns cumulative transport/transit byte counters, connected-router count, transit-tunnel count, cumulative tunnel-build success/failure counts, and v4/v6 firewall status;
- `RouterContext::profile_storage()` is the canonical known-router directory and can provide public serialized RouterInfo;
- `TransportManager` owns current connected peers and already has a bounded peer-ID snapshot helper;
- NTCP2/SSU2 configs retain `max_connections` information;
- `Router::inspection_snapshot()` and `emissary-core/src/inspection.rs` establish a neutral DTO vocabulary, but the snapshot is one-shot and contains placeholder zeros for currently unavailable tunnel fields; it is not a live I2PControl source;
- tunnel pools/transit owners contain the lifecycle information needed for count/detail fields, but no suitable cloneable read-only aggregate exists;
- I2PControl already owns all canonical serializers and a `RouterInfoControl` abstraction, so source creation need not move wire policy into core.

Read-only i2pd reference inspection confirms that the adopted fields have distinct semantics: transit 15s is a rolling rate, recent tunnel success is a recent/EWMA-style metric distinct from total success, queue/TBM queue are instantaneous depths, and network status/error/testing are independent v4/v6 state.

## 4. Ownership architecture

### 4.1 I2PControl-owned policy

The following remain under `emissary-cli/src/i2pcontrol/**`:

- Proposal 170 key/type/source inventory;
- source grouping and request assembly;
- rolling transit sampling and time-window policy;
- recent-success wire mapping/reference conformance;
- deterministic sorting/deduplication;
- peer-directory joins;
- collection and serialized-size bounds;
- numeric network status/error mapping;
- sanitized error translation;
- all JSON-RPC and compatibility semantics;
- complete/incomplete observation policy and fail-closed behavior.

### 4.2 Core/runtime exception rule

A change outside `i2pcontrol/**` is authorized only when the current canonical owner is the only truthful place to observe the fact. Such changes must be one of:

- neutral immutable snapshot DTO;
- cloneable read-only inspection handle;
- passive bounded lifecycle/stat publication at an existing transition;
- composition-only wiring in `main.rs`.

They must never become administrative/control handles or contain Proposal 170/wire terminology.

The exact milestone budgets are machine-readable in `045-052-routerinfo-source-boundary.toml`.

### 4.3 No parallel authorities

Do not build a second NetDB, peer, tunnel, reachability, or metrics cache when an existing owner can be snapshotted. A bounded I2PControl aggregation map is acceptable only when it consumes authoritative lifecycle facts and has explicit incomplete/recovery semantics. The current one-shot `CoreSnapshot` must not become a stale parallel runtime authority.

## 5. Cross-cutting invariants

1. Exact Proposal 170 names, casing, types, and presence semantics remain unchanged.
2. No fabricated zero, false, empty, null, or adjacent metric may replace unavailable state.
3. Default/no-feature router execution performs no new I2PControl work.
4. Core observation cannot change routing, peer selection, NetDB, transport, tunnel-building, congestion, retry, timing, cryptographic, or LeaseSet behavior.
5. No private key, session key, destination private material, socket, mutable session/tunnel object, command channel, or message payload crosses an inspection boundary.
6. All collections and histories are bounded; API handlers do not retain unbounded time series.
7. No lock is held across network I/O, sleep, cancellation, `.await`, or JSON serialization.
8. A partial/incomplete observation fails closed rather than returning a plausible partial result.
9. Source availability in `PROPOSAL_170_CONTRACT` changes only after the production source and exact fixture exist.
10. Compatibility aliases/base nested selectors are not expanded or silently changed.
11. No tunnel data plane, `SetConfig`, frontend, workflow/release, broad refactor, or upstream activity is authorized.

## 6. Dependency graph

```text
M044 closed
   |
   v
M045 known peer directory (3) — ready
   |
   v
M046 active peers + limits (4)
   |
   v
M047 active peer stats (1)
   |
   v
M048 tunnel pool counts/details (7)
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

The sequence is deliberately serialized. Several source families are technically separable, but serialization gives each non-I2PControl seam an independent closure review before another audited-core exception is added. M045 is blocked pending a separately authorized neutral ProfileStorage enumeration seam; M046–M052 remain blocked roadmap successors until their hard dependencies close.

## 7. Milestones

### M045 — Known-peer directory sources — blocked

Plan: `045-routerinfo-known-peer-directory.md`

Fields: `netdb.peers`, `netdb.peers.list`, `netdb.peers.info`.

Use existing `ProfileStorage` read-only APIs. Expected core production changes: zero. The current
public API does not expose the required directory enumeration type to `emissary-cli`, so a
separate corrective plan is required before implementation can resume. Exit: all three fields
live/bounded/exact; no core production diff for M045 itself.

### M046 — Active-peer inventory and limits — blocked on M045 corrective live-source seam

Plan: `046-routerinfo-active-peer-inventory-and-limits.md`

Fields: active peer list/info and NTCP/SSU limits.

Add the minimum neutral cloneable transport-inspection source. Resolve finite/unlimited limit semantics exactly; never invent a sentinel.

### M047 — Active-peer statistics — blocked on M046

Plan: `047-routerinfo-active-peer-stats.md`

Field: `netdb.activepeers.stats`.

Audit each required object field to a canonical NTCP2/SSU2 owner. Extend neutral observation only for facts that can be captured passively without protocol behavior changes.

### M048 — Tunnel-pool counts/details — blocked on M047

Plan: `048-routerinfo-tunnel-pool-sources.md`

Fields: participating detail; exploratory in/out/detail; client in/out/detail.

Use a neutral bounded passive tunnel observation source. Do not expose pool handles or promote current placeholder zero fields.

### M049 — Rolling metrics and queues — blocked on M048

Plan: `049-routerinfo-rolling-metrics-and-queues.md`

Fields: transit 15s, recent tunnel success, queue, TBM queue.

Compute rolling transit in I2PControl from cumulative counters. Match reference recent-success semantics; add a neutral core gauge only if event-order semantics cannot be reconstructed. Read queues from actual owners.

### M050 — Network status/error/testing — blocked on M049

Plan: `050-routerinfo-network-state-sources.md`

Fields: status.v6; error v4/v6; testing v4/v6.

Track neutral independent state at existing reachability transitions and map to wire integers only in I2PControl. No new network probes.

### M051 — News and banned-peer semantics — blocked on M050

Plan: `051-routerinfo-news-and-banned-peer-semantics.md`

Fields: router news and banned peers.

These require explicit contract adjudication because Emissary has no identified news or ban subsystem. An authoritative capability-empty value is acceptable only with pinned/reference evidence. Adding news/ban behavior merely for telemetry is prohibited; unresolved semantics remain a truthful blocker.

### M052 — 26-source integration and reclosure — blocked on M045–M051

Plan: `052-routerinfo-source-integration-and-reclosure.md`

No production changes. Review all 26 target rows, retained 17 rows, production child-process behavior, bounds, failure semantics, no-feature behavior, and every core changed path. A defect requires a new corrective plan.

## 8. Failure, cancellation, restart, and contention policy

Read-only snapshots are request-scoped and owned. Passive observers use bounded state and must not block the producer. Observation publication failure never changes router behavior; if it makes administrative state incomplete, I2PControl fails the affected request until authoritative recovery. Rolling metrics are process-local and reset safely on counter rollback/restart. No new persisted state is required by this roadmap.

Peer/tunnel churn is expected. Each source family must define a coherent snapshot boundary and deterministic handling of an object disappearing during collection. Partial cross-owner joins are errors or documented omissions only when contract-correct; never fabricated rows.

## 9. Verification policy

Each implementation milestone runs focused tests first, then affected core package tests and the bounded CLI feature/no-feature matrix. M052 reruns the broad matrix and live child-process test. No new remote CI, coverage, fuzz, soak, network farm, release automation, or generated evidence bundle is required.

Static guards should enforce:

- no wire/Proposal 170 terminology in neutral core DTOs;
- no secret/live handle types in inspection contracts;
- changed paths remain within milestone budget;
- no resource allocation or mutable control methods in inspection handles;
- no unavailable source is marked available before its production evidence lands.

## 10. Security and compatibility

The work is read-only observability. Authentication, TLS, tokens, AddressBook authority, tunnel administrative persistence, server secrets, and compatibility aliases remain untouched. No schema migration is expected. Response-size and request bounds remain mandatory and may be tightened per source without changing wire shape.

Because upstream Emissary is treated as heavily security-reviewed, duplicate small observation adapters inside I2PControl are preferable to broad refactors of existing router data planes. Code reuse is not a sufficient reason to alter audited core paths.

## 11. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Core inspection grows into a general management API | cloneable read-only owned snapshots only; machine-readable path budget |
| One-shot snapshot becomes stale authority | request-time live handles/passive state; no second persisted cache |
| Recent success implemented as cumulative success | reference fixture requires distinct recent/EWMA semantics |
| Tunnel placeholders become claimed values | explicit prohibition and M048 lifecycle evidence |
| Active peer stats leak live/session secrets | sanitized neutral DTO + static type guards |
| Peer joins race with connection churn | coherent snapshots, deterministic failure/omission policy |
| Unlimited transport limits get invented sentinel | M046 stop rule until exact semantics are established |
| Network testing/error inferred from firewall state | independent neutral state required |
| Empty news/ban values used as fake completion | M051 contract adjudication gate |
| Scope expands into router behavior | milestone path budgets + independent closure after each slice |

## 12. Explicit non-goals

- the ten unsupported Proposal 170 tunnel data planes;
- control/adoption of startup-managed tunnel tasks;
- AddressBook or `SetConfig` changes;
- base I2PControl unsupported methods;
- router/transport/tunnel/NetDB algorithm redesign;
- new reachability probes, news downloader, or ban engine solely for telemetry;
- UI/frontend work;
- crate-wide extraction/refactor;
- CI/release/publishing expansion;
- upstream issues, PRs, reviews, submissions, adoption, merge requests, maintainer outreach, or contribution preparation.

## 13. Final status rule

If M045–M051 make all 26 fields operational and M052 accepts the final head, the RouterInfo dimension may be recorded as 42 available + 1 protocol-permitted neutral + 0 unavailable and `RouterInfo source completion closed internally against pinned revision`.

That does not by itself make the entire Proposal 170 implementation complete. The subsystem remains `partial Proposal 170 support` while unrelated accepted unsupported/runtime dimensions remain.

If M051 proves a field requires an absent substantive subsystem that this roadmap refuses to add, retain that field as unavailable and record RouterInfo source completion as incomplete rather than broadening scope or fabricating support.

## 14. Historical corrective sequence

M040–M044 remain closed historical/retained evidence. M039 remains historical-invalidated as documented. This roadmap supersedes only the prior statement that the 26 RouterInfo sources were outside authorized scope; it does not rewrite or invalidate the accepted M044 closure for the earlier head.
