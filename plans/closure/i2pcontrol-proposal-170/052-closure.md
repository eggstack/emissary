# I2PControl Proposal 170 Milestone M052 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/052-routerinfo-source-integration-and-reclosure.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline reviewed: `1b4b6e5`

Final implementation/evidence head reviewed: `3887106`

Implementation commits:

- `09a46cb` — wire the live M045 peer-directory inspection source;
- `fca7a5f` — expose active-peer inventory and finite transport limits;
- `fc7d067` — expose active-peer statistics;
- `0c50a21` — expose live tunnel-pool sources;
- `cd9ee99` — expose rolling metrics and queue sources;
- `11b1d33` — expose v4/v6 network state sources;
- `3887106` — exercise all 24 newly available target selectors and representative multi-group composition through the real child process.

Pinned Proposal 170 revision: `2026-05-20`, Open.

## 1. Executive finding

M052 is closed as an internal validation and reclosure milestone. The final
integrated head truthfully serves 40 of the 43 canonical RouterInfo additions,
with one protocol-permitted neutral value and two accepted unavailable rows.
The 24 operational fields added by M045–M050 passed focused production
composition and real TLS/authenticated child-process evidence. M051's
`i2p.router.news` and `i2p.router.netdb.bannedpeers` rows remain unavailable
because Emissary has no authoritative news-feed or ban-list owner; M052 does
not claim RouterInfo source completion or broaden scope to create either
subsystem. Overall Proposal 170 status remains `partial Proposal 170 support`.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| `i2p.router.netdb.peers` | `LivePeerDirectorySource`; `p170.netdb.peer_hashes`; production child-process request | pass | Current bounded known-peer IDs |
| `i2p.router.netdb.peers.list` | `LivePeerDirectorySource`; `p170.netdb.peer_list`; production child-process request | pass | Same live source, exact list shape |
| `i2p.router.netdb.peers.info` | `LivePeerDirectorySource`; `p170.netdb.peer_info`; production child-process request | pass | Public RouterInfo bytes only |
| `i2p.router.netdb.activepeers.list` | `TransportInspectionSource`; `p170.netdb.active_peers`; production child-process request | pass | Current connected peer IDs |
| `i2p.router.netdb.activepeers.info` | Transport-to-live-directory join; `p170.netdb.active_peer_info`; production child-process request | pass | Missing public joins fail closed |
| `i2p.router.netdb.ntcp.limit` | Authoritative finite NTCP2 configuration; `p170.netdb.ntcp_limit`; child fixture sets 64 | pass | Unlimited configuration remains unavailable rather than sentinel-mapped |
| `i2p.router.netdb.ssu.limit` | Authoritative finite SSU2 configuration; `p170.netdb.ssu_limit`; child fixture sets 64 | pass | Unlimited configuration remains unavailable rather than sentinel-mapped |
| `i2p.router.netdb.activepeers.stats` | Bounded neutral active-session DTO; `p170.netdb.active_peer_stats`; production child-process request | pass | No sockets, keys, or mutable sessions cross the seam |
| `i2p.router.net.tunnels.participating.info` | Neutral tunnel-pool inspection; `p170.participating_info.rows`; production child-process request | pass | Bounded live participating rows |
| `i2p.router.net.tunnels.exploratory.inbound` | Neutral tunnel-pool inspection; `p170.exploratory_inbound.count`; production child-process request | pass | Exact count serializer |
| `i2p.router.net.tunnels.exploratory.outbound` | Neutral tunnel-pool inspection; `p170.exploratory_outbound.count`; production child-process request | pass | Exact count serializer |
| `i2p.router.net.tunnels.exploratory.info.list` | Neutral tunnel-pool inspection; `p170.exploratory_info.rows`; production child-process request | pass | Bounded deterministic rows |
| `i2p.router.net.tunnels.client.inbound` | Neutral tunnel-pool inspection; `p170.client_inbound.count`; production child-process request | pass | Exact count serializer |
| `i2p.router.net.tunnels.client.outbound` | Neutral tunnel-pool inspection; `p170.client_outbound.count`; production child-process request | pass | Exact count serializer |
| `i2p.router.net.tunnels.client.info.list` | Neutral tunnel-pool inspection; `p170.client_info.rows`; production child-process request | pass | Bounded deterministic rows |
| `i2p.router.net.bw.transit.15s` | Bounded rolling transit sampler; `p170.transit_15s.bytes_per_second`; production child-process request | pass | Process-local bounded observation |
| `i2p.router.net.tunnels.successrate` | Ordered recent-build EWMA; `p170.success_rate.recent.percent`; production child-process request | pass | Distinct from cumulative success |
| `i2p.router.net.tunnels.queue` | Live pending-build depth; `p170.tunnel_queue.depth`; production child-process request | pass | Bounded current queue observation |
| `i2p.router.net.tunnels.tbmqueue` | Live transit build-message depth; `p170.tbm_queue.depth`; production child-process request | pass | Bounded current queue observation |
| `i2p.router.net.status.v6` | Cached v6 reachability state; `p170.status_v6.integer`; production child-process request | pass | No new probe or network task |
| `i2p.router.net.error` | Independent v4 error state; `p170.error_v4.integer`; production child-process request | pass | Unknown reason maps to the contract neutral code |
| `i2p.router.net.error.v6` | Independent v6 error state; `p170.error_v6.integer`; production child-process request | pass | Not inferred from firewall status |
| `i2p.router.net.testing` | Existing v4 reachability-test state; `p170.testing_v4.integer`; production child-process request | pass | Passive observation only |
| `i2p.router.net.testing.v6` | Existing v6 reachability-test state; `p170.testing_v6.integer`; production child-process request | pass | Passive observation only |
| `i2p.router.news` | M051 semantic adjudication; `p170.router_news.string`; unavailable fixture | retained unavailable | No authoritative news-feed owner; empty string is not fabricated |
| `i2p.router.netdb.bannedpeers` | M051 semantic adjudication; `p170.netdb.banned_peers.unavailable`; unavailable fixture | retained unavailable | No authoritative ban-list owner; empty map is not fabricated |

The retained 16 available rows and the one protocol-permitted clock-skew
neutral row were rerun through the existing conformance, literal-fixture,
truthfulness, production-composition, and live-runtime suites. The final
source inventory remains exactly 43 rows: 40 available, 1 neutral, 2
unavailable.

## 3. Production implementation evidence

M052 made no production implementation changes. The reviewed production path
is the already integrated M045–M051 work:

- I2PControl owns Proposal 170 policy, exact selectors, joins, bounds,
  serialization, direct-presence behavior, and sanitized errors.
- `emissary-core/src/inspection.rs` and `router/mod.rs` expose only neutral,
  bounded, owned read-only observations.
- `events.rs`, the NTCP2/SSU2 observation paths, and tunnel pool/transit paths
  add only minimal passive observations at existing authoritative owners.
- `emissary-cli/src/main.rs` composes the live sources only in the enabled
  I2PControl path.

The new evidence commit changes only
`emissary-cli/tests/i2pcontrol_live_runtime.rs`: it supplies finite transport
limits in the local fixture, requests every one of the 24 newly available
target selectors individually, and requests a representative combined set
spanning peer, active-statistics, tunnel-queue, rolling-metric, and network
source groups.

## 4. Verification executed

### Commands run

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
git diff --check
cargo fmt --all -- --check
```

### Results

- Production composition: pass, 9 tests.
- Live child process: pass, 1 test in 20.45 seconds. The process exercised
  TLS, authentication, all 24 newly available selectors individually, and a
  representative multi-source request.
- No-feature CLI: check pass; 56 tests pass; clippy pass with `-D warnings`.
- Feature-enabled CLI: check pass; 1,369 tests pass; clippy pass with
  `-D warnings`.
- Core: check pass; 1,062 tests pass, 2 ignored; clippy pass with
  `-D warnings`.
- `git diff --check`: pass.
- `cargo fmt --all -- --check`: qualified failure. The installed stable
  rustfmt cannot apply this repository's nightly-only options and reports
  unrelated pre-existing formatting churn. The new test additions are
  syntactically valid and the focused formatter check showed no diff in the
  added blocks; no unrelated formatter churn was retained.

## 5. Invariant review

All M052 invariants pass:

1. Every available field has a named owner, exact JSON shape, bound, and
   fixture in `PROPOSAL_170_CONTRACT` and the source-map documentation.
2. No placeholder zero, false, empty, or null promotes an unavailable field.
   Empty/zero values succeed only after a successful authoritative query;
   clock skew is the sole protocol-permitted neutral.
3. Core observation remains passive and neutral. No Proposal 170 wire names,
   JSON-RPC types, mutable control authority, or private material cross the
   boundary.
4. Default/no-feature behavior is unchanged, confirmed by the no-feature
   package matrix.
5. No upstream interaction occurred.

Changed-path containment against M044 is clean. Production changes classify
only as I2PControl adapter/policy, CLI composition, neutral core inspection,
or minimal passive observation in existing event, transport, and tunnel-pool
owners. No crypto, I2NP, routing, tunnel selection/build algorithm, transport
state machine, NetDB protocol, proxy/UI, AddressBook, workflow, CI, release,
or frontend path was changed by M045–M051.

## 6. Failure, recovery, and contention review

Inspection sources return owned bounded snapshots and release synchronization
before async serialization. Peer-directory churn and incomplete joins fail
closed; no adjacent or empty RouterInfo is fabricated. Unlimited transport
limits remain unavailable. Unavailable news/ban requests fail before partial
assembly. Existing tests cover malformed input, auth failure, bounds, source
failure, selector overlap, concurrent reads, cancellation-safe server
shutdown, restart, and durable control-plane recovery. The live process test
also exercised restart and malformed-request isolation.

## 7. Migration and compatibility review

No schema, persistence, or configuration migration was introduced. Direct
Proposal 170 selector presence, nested compatibility mode, exact key spelling,
JSON types, and unavailable/error behavior remain unchanged. The live fixture's
finite NTCP2/SSU2 limits are test-only and do not alter defaults.

## 8. Security review

RouterInfo remains authenticated and served over the real TLS endpoint in the
operational test. Static guards and source review confirm that no socket,
session, channel, private key, session key, mutable router/transport/tunnel
handle, or control command crosses into I2PControl. Bounds apply before wire
serialization and aggregate response limits remain enforced. No secret or
password appeared in child diagnostics during the live restart path.

## 9. Documentation and operations

The source map, RouterInfo documentation, conformance documentation, roadmap,
implementation README, registry, and this closure record reconcile to
40 available / 1 protocol-permitted neutral / 2 unavailable. No operator
migration or external operational action is required. The exact live command
and its finite-limit fixture are recorded above for repeatability.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| accepted limitation | News and banned-peer rows have no authoritative Emissary owners | RouterInfo source completion cannot be claimed; overall support remains partial | Keep both rows unavailable. Create a new owner-specific plan only if the router later gains those substantive capabilities. |
| low | Stable rustfmt cannot satisfy the repository's nightly-only configuration without unrelated churn | Formatting evidence remains qualified | Use the documented nightly formatter when available; do not retain unrelated churn. |

No critical, high, or unaccepted medium correctness, security, containment,
compatibility, or source-truthfulness finding remains.

## 11. Roadmap disposition

M052 is closed as a validation/reclosure milestone with the accepted M051
semantic limitation. RouterInfo source completion is incomplete at 40/1/2;
the broader subsystem remains `partial Proposal 170 support`. M051 remains
blocked, and no future implementation plan can be unblocked because the two
remaining rows require substantive owners explicitly outside this roadmap.

## 12. Registry updates

- Mark M052 `closed` and add this closure record.
- Remove M052 from dependency-ready handoffs.
- Keep M051 `blocked` with its accepted semantic disposition and closure.
- Record that no dependency-ready successor remains.
- Keep the subsystem and RouterInfo source matrix at truthful partial status:
  43 total, 40 available, 1 protocol-permitted neutral, 2 unavailable.

## 13. Internal-only attestation

The pinned Proposal 170 specification and read-only reference material were
used only as external authority. No upstream repository or maintainer channel
was mutated; no upstream issue, pull request, review, merge, adoption request,
submission, maintainer contact, or contribution artifact was created or
prepared. All repository writes remain internal to `eggstack/emissary`.

**Disposition: M052 closed; RouterInfo source completion remains incomplete at
40 available / 1 protocol-permitted neutral / 2 unavailable; M051 remains
blocked; no future plan unblocked.**
