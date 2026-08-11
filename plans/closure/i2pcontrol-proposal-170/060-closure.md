# M060 Closure Record — Core Observation Seam Consolidation and Containment

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/060-core-observation-containment.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`

Repository baseline reviewed: `85677e7` (M059-close planning head)

Upstream comparison baseline: `9b43484a21d5a1291c4881cdae62a36c527f8c0f`

Implementation head: `6085eca` (`contain core observation seams for M060`)

Implementation commits or pull requests:

- `6085eca` — core observation seam reduction, focused containment guards, and
  ownership documentation. No upstream pull request or contribution artifact
  was created.

## 1. Executive finding

M060 is closed. The accepted 32-path core budget was reduced to 23 retained
paths against the pinned upstream baseline. Nine formatting-only or obsolete
paths were restored to upstream; dead aggregate snapshot scaffolding and an
unconsumed SAM recovery map were removed; and a duplicate SSU2 observation
update was corrected so each buffered send is counted once.

The remaining core changes are neutral, bounded, passive owner seams. SAM
aggregation/recovery and all public bounds remain in `emissary-cli`; transport
byte hooks remain at active I/O owners because only those owners observe exact
partial read/write and packet-send completion counts; tunnel lifecycle and
queue facts remain at their canonical pool/transit owners. No router algorithm,
protocol behavior, persistence format, wire contract, dependency, or supported
RouterInfo disposition changed.

M061 is unblocked and is registered `ready` against implementation head
`6085eca`. M051 remains independently blocked because substantive news and
banned-peer owners are still absent.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Exact M060 budget respected | `060-containment-result.toml`, M060 guard, baseline/upstream path audit | pass | No path outside the frozen 32-path budget was changed; final upstream delta has 23 core paths |
| No new core production path | `git diff --name-only 9b43484..6085eca -- emissary-core`, M060 guard | pass | All retained paths were present in M058’s budget |
| Candidate-revert paths handled | Upstream comparison and result manifest | pass | Error, I2CP, runtime, SAM parser, subsystem, SSU2 data/relay/pending/terminating formatting paths match upstream |
| Dead aggregate scaffolding removed | `inspection.rs`, `router/mod.rs`, `transport/mod.rs` diff and static guard | pass | `CoreSnapshot`, its DTOs, aggregate accessor, and orphaned peer-ID accessor are absent |
| SAM seam remains minimal | SAM source review, 149 SAM tests, M060 neutral-term guard | pass | Sanitized lifecycle events remain owner-local; observer failure is non-fatal |
| Transport deep hooks are necessary | NTCP2/SSU2 owner review and 43 NTCP2 / 253 SSU2 focused tests | pass | Exact byte counts are only visible at active I/O owners; duplicate SSU2 update removed |
| Tunnel hooks are owner-local and bounded | 138 tunnel tests, retained-path review | pass | Pool/transit owners publish bounded lifecycle/queue facts; no rolling transit sampler was added |
| Core policy neutrality | M060 static guard and source search | pass | No `I2PControl`, Proposal 170, JSON-RPC, or ClientServicesInfo terminology remains in production core sources |
| RouterInfo truthfulness unchanged | M056 accepted matrix, M060 result manifest, CLI focused suite | pass | 37 available / 1 neutral / 5 unavailable remains authoritative |
| Router/protocol behavior unchanged | diff review, broad core suite, focused protocol suites | pass with environmental flake | Focused suites passed; one broad-suite IPv6/ML-KEM relay timeout passed on isolated rerun |
| Failure/recovery semantics preserved | SAM observer code, tunnel inspection code, focused tests | pass | Publication failure remains passive; incomplete tunnel snapshots remain fail-closed |
| Compatibility and migration unchanged | manifest/lockfile/wire/persistence review | pass | No dependency, schema, config, or wire migration |
| Internal-only boundary preserved | repository and remote review | pass | Upstream was read-only; only the internal `origin` is in scope for push |

## 3. Before/after core path set

The M058 budget contained 32 core paths. The final implementation head changes
these 23 paths against the pinned upstream baseline:

```text
emissary-core/src/events.rs
emissary-core/src/inspection.rs
emissary-core/src/lib.rs
emissary-core/src/primitives/router_identity.rs
emissary-core/src/router/context.rs
emissary-core/src/router/mod.rs
emissary-core/src/sam/mod.rs
emissary-core/src/sam/pending/connection.rs
emissary-core/src/sam/protocol/streaming/listener.rs
emissary-core/src/sam/protocol/streaming/mod.rs
emissary-core/src/sam/session.rs
emissary-core/src/sam/socket.rs
emissary-core/src/transport/mod.rs
emissary-core/src/transport/ntcp2/mod.rs
emissary-core/src/transport/ntcp2/session/active.rs
emissary-core/src/transport/ntcp2/session/mod.rs
emissary-core/src/transport/ssu2/mod.rs
emissary-core/src/transport/ssu2/peer_test/mod.rs
emissary-core/src/transport/ssu2/session/active/mod.rs
emissary-core/src/transport/ssu2/socket.rs
emissary-core/src/tunnel/mod.rs
emissary-core/src/tunnel/pool/mod.rs
emissary-core/src/tunnel/transit/mod.rs
```

The following nine budget paths are fully restored to upstream:

```text
emissary-core/src/error/mod.rs
emissary-core/src/i2cp/socket.rs
emissary-core/src/runtime/mod.rs
emissary-core/src/sam/parser.rs
emissary-core/src/subsystem/mod.rs
emissary-core/src/transport/ssu2/message/data.rs
emissary-core/src/transport/ssu2/relay/mod.rs
emissary-core/src/transport/ssu2/session/pending/inbound.rs
emissary-core/src/transport/ssu2/session/terminating.rs
```

The five reduced hunks are recorded in
`plans/implementation/i2pcontrol-proposal-170/060-containment-result.toml`.

## 4. Retained deep-hook necessity review

| Retained owner | Fact | Why a higher owner cannot truthfully replace it |
|---|---|---|
| `SamServer` / `SamSession` | Session, stream, and listener lifecycle | These owners alone observe authoritative activation and terminal close/reject ordering; the event is sanitized before publication |
| NTCP2 active session | Per-peer inbound/outbound bytes | The active I/O owner sees partial reads and writes; `TransportManager` sees only connection transitions |
| SSU2 active session | Per-peer packet, ACK, retransmit, and send bytes | The active session owns packet completion and retransmission paths; lifting it would infer rather than observe counts |
| `TunnelPool` | Directional live tunnel entries and pending queue depth | The pool owns exact build/expiry transitions and both inbound/outbound pending collections |
| `TransitTunnelManager` | Participating tunnel entries and TBM queue depth | The transit owner alone sees admission/expiry and its message receiver queue; no request-driven sampler is used |

All retained publications are optional/passive, bounded, sanitized where text
exists, and have no lifecycle-control capability.

## 5. Production implementation evidence

- Removed the unused aggregate `CoreSnapshot`, `TransportSnapshot`,
  `TunnelSnapshot`, and `NetDbSnapshot` DTO family and the unused
  `Router::inspection_snapshot`/`TransportManager::connected_peer_ids` path.
- Removed an unused transport peer-list replacement helper and the unused SAM
  rejected-publication tracking map. Stream-manager close/reject events already
  carry the authoritative socket identity.
- Restored nine non-semantic core paths exactly toward the pinned upstream
  implementation.
- Removed the duplicate buffered-send `record_peer_bytes` call in SSU2 active
  sessions; no handshake, retry, congestion, or timing logic was changed.
- Kept neutral public identity and inspection handles required by the live CLI
  adapters, without exposing sockets, sessions, mutable owners, keys, payloads,
  or command channels.
- Added M060 source/budget/passivity guards and updated the inspection
  architecture documentation. The historical M037 boundary was not rewritten.

## 6. Verification executed

### Commands run

Passed:

```text
rtk cargo check -p emissary-core
rtk cargo check -p emissary-core --no-default-features --features no_std
rtk cargo check -p emissary-cli --no-default-features
rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo test -p emissary-core inspection --no-fail-fast
rtk cargo test -p emissary-core sam --no-fail-fast
rtk cargo test -p emissary-core transport::ntcp2 --no-fail-fast
rtk cargo test -p emissary-core transport::ssu2 --no-fail-fast
rtk cargo test -p emissary-core tunnel --no-fail-fast
rtk cargo test -p emissary-core transport::ssu2::tests::relay_works_ipv6_ml_kem_512 --no-fail-fast
rtk cargo test -p emissary-core --no-fail-fast (991 passed, 1 flaky timeout, 2 ignored)
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test client_services_live --test router_info_truthfulness --test production_adapter --test production_composition --test m037_containment --test m059_containment --test m060_containment
rtk cargo clippy -p emissary-core --all-targets -- -D warnings
rtk git diff --check
rtk git diff --cached --check
```

The focused CLI command passed 99 tests across seven suites. The broad core
run completed 991 passed, 1 failed, and 2 ignored; the failure was
`transport::ssu2::tests::relay_works_ipv6_ml_kem_512`, which passed immediately
when rerun alone. Focused SSU2 completed 253/253 and the isolated regression
completed 1/1.

### Known repository/toolchain results

These checks were run and did not provide clean results for reasons unrelated
to M060 production changes:

- `rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` reports the pre-existing frozen-path warning `clippy::to_string_in_format_args` at `emissary-cli/src/proxy/socks.rs:543`.
- `rtk cargo fmt --all -- --check` reports the repository’s existing stable-rustfmt/nightly-option drift across untouched files and test sources. No broad formatting rewrite was made because it would violate the frozen containment boundary.
- `rtk cargo check -p emissary-core --no-default-features` remains a pre-existing invalid feature combination: the crate requires either `std` or `no_std`; the supported no-std check above passes.

## 7. Invariant review

1. Router algorithms and network behavior: unchanged by the implementation
   diff; only owner-local publication/dead-code removal was changed.
2. Peer selection, NetDB, tunnels, transports, cryptography, LeaseSet, and
   I2NP: no algorithm or protocol path was changed; focused core tests pass.
3. Unavailable sources: no source was added; the 37/1/5 matrix is unchanged.
4. ClientServicesInfo and supported tunnel lifecycle: CLI regression suites
   pass and core SAM/tunnel owner behavior remains intact.
5. Core neutrality: static guard and source search pass.
6. Secret/live-object boundary: retained DTOs contain owned public facts only;
   SAM events contain sanitized text and scalar IDs.
7. Bounds/contention: existing bounded locks/collections remain; snapshots are
   copied before serialization/await, and no new queue or background task was
   introduced.
8. Observer failure: publication errors are logged and do not fail the owning
   SAM/session/tunnel lifecycle; complete/incomplete handling remains in the
   application adapter.
9. No upstream interaction: confirmed below.

## 8. Failure, recovery, and contention review

- SAM observer rejection is a passive error path; session creation, stream
  registration, and socket teardown continue independently.
- Tunnel inspection overflow remains fail-closed until owner recovery, with a
  fixed maximum entry bound. Pool and transit drops clear their owner facts.
- Removing the unconsumed SAM tracking map does not alter recovery: the
  authoritative stream-manager close/reject events still carry socket IDs.
- The SSU2 duplicate accounting correction reduces overcounting without
  changing packet scheduling, retransmission, handshake, or congestion state.
- No new cancellation token, task handle, durable state, or lock crossing an
  await/network/serialization boundary was introduced.

## 9. Compatibility, migration, and security review

There is no protocol, wire, persistence, configuration, dependency, or
migration change. Public core inspection constructors remain compatible through
the existing default wrappers; removed aggregate DTOs/accessors had no live
consumer.

No retained seam carries a socket, stream, mutable router/transport/tunnel
owner, command sender/receiver, private key, LeaseSet private material, or raw
message payload. The core source contains no control-plane method, selector,
JSON-RPC, or administrative support classification. No new dependency, event
bus, probe, sampler, persistent metric store, unsupported data plane, or CI/
release machinery was introduced.

## 10. Documentation and operations

- Added `060-containment-result.toml` with machine-readable reversion,
  reduction, retained-owner, and no-new-path evidence.
- Added `emissary-cli/tests/m060_containment.rs` for path-budget, neutrality,
  aggregate-snapshot, passive-failure, and single-count guards.
- Updated `docs/i2pcontrol/inspection-architecture.md` to describe narrow
  neutral handles rather than an aggregate core snapshot.
- Updated the M059 historical guard to compare its own implementation range,
  so later M060 core changes do not invalidate historical M059 evidence.
- Updated the containment roadmap, implementation README, registry, M060 plan,
  and M061 handoff status.

## 11. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | Stable rustfmt check has pre-existing repository-wide drift | Formatting command is not a clean repository gate | Preserve current narrow diff; future formatting cleanup requires its own scope |
| low | One broad SSU2 IPv6/ML-KEM relay test timed out once | Broad run had 991/992 completed tests; isolated rerun passed | M061 may independently re-run the bounded suite; no M060 corrective action is indicated |
| low | CLI clippy reports a warning in frozen M059 SOCKS code | Required `-D warnings` command is not clean on this toolchain | Do not widen M060; address only in a separately authorized corrective scope |

No medium- or high-severity finding remains. These low-severity environmental/
historical findings do not block M060 closure.

## 12. Roadmap and registry disposition

M060 is closed. `plans/registry.md` now marks M060 closed and registers M061
as the sole dependency-ready successor with planning baseline `6085eca`.
The containment roadmap and implementation README reflect the same status.
M051 remains blocked independently by absent substantive news/banned-peer
owners; no other future plan became ready.

## 13. Internal-only attestation

External specifications and the pinned upstream source/commit were accessed
read-only for comparison. No upstream repository or maintainer channel was
mutated. No upstream issue, pull request, review, discussion, merge/adoption
request, submission, contribution package, branch, tag, release, or connector
write was created or prepared. The requested push is limited to the internal
`eggstack/emissary` repository’s `origin` remote.

**Disposition: M060 closed; core observation containment accepted; M061 ready;
M051 remains independently blocked.**
