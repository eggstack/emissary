# I2PControl Proposal 170 Milestone M047 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/047-routerinfo-active-peer-stats.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md#milestone-047--active-peer-statistics--closed`

Repository baseline reviewed: `b759038` planning baseline; implementation head recorded in the final commit below.

Implementation commit:

- `fc7d067` — bounded RouterInfo active-peer statistics from the neutral transport inspection seam

Pinned Proposal 170 revision: `2026-05-20`, Open.

## 1. Executive finding

M047 is closed. `i2p.router.netdb.activepeers.stats` now returns bounded exact
objects for the current established transport population. The existing
I2PControl `ActivePeerStats` DTO and serializer remain the wire owner. Core
exposes only owned peer ID, inbound/outbound fact, connected-membership fact,
and saturating per-session byte counters.

Latency, IP/port, version, capabilities, cryptographic/session state, and
other absent facts remain unavailable and are not represented by defaults.

## 2. Field-owner matrix

| Wire field | Canonical owner/update point | Neutral observation | Result |
|---|---|---|---|
| `peerId` | `TransportManager::routers` / existing connection-establishment transition | Base64 peer ID in `TransportPeerInspection` | pass |
| `direction` | `TransportEvent::ConnectionEstablished::direction` | `inbound: bool`, captured at the manager transition | pass |
| `state` | `TransportManager::routers` membership and existing close transition | `connected: bool`, active entries only | pass |
| `bytesReceived` | NTCP2 active-session reads; SSU2 active-session packet receives | Saturating owned counter updated at existing read points | pass |
| `bytesSent` | NTCP2 active-session writes; SSU2 active-session ACK, retransmit, and packet writes | Saturating owned counter updated at existing write points | pass |
| latency/IP/port/version/capabilities | No canonical current owner for this selector | Not added and not serialized | pass |

The I2PControl adapter maps the neutral booleans to `inbound`/`outbound` and
`connected`, sorts by peer ID, and applies the existing 10,000-item and 4 MiB
response bounds.

## 3. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Exact Proposal 170 object | `proposal_active_peer_stats_returns_exact_bounded_objects`; five-field JSON fixture | pass |
| Live active-session lifecycle | `transport_peer_stats_are_live_bounded_and_removed_with_sessions`; NTCP2/SSU2 transport suites | pass |
| Deterministic ordering | Core snapshot sorts peer IDs and stats; production adapter sorts returned stats | pass |
| Empty authoritative snapshot | Default inspection snapshot returns an empty stats array; no fallback source is used | pass |
| Explicit item bound | Core inspection bound test and existing handler `MAX_ACTIVE_PEER_STATS` check | pass |
| No invented individual values | Absent latency and other fields are omitted; no plausible defaults cross the wire | pass |
| No live or secret handle crossing | `transport_inspection_handle_contains_only_owned_snapshot_state` and `transport_peer_inspection_contains_only_sanitized_facts` | pass |
| Feature isolation | No-feature CLI suite passes; composition remains inside the I2PControl setup path | pass |
| Source accounting | Contract, literal fixtures, conformance manifest, and source map agree: 24 available / 1 neutral / 18 unavailable | pass |

## 4. Production implementation evidence

The cloneable `TransportInspection` snapshot now contains bounded owned peer
facts. The transport manager updates membership and direction only at the
existing connection establish/close transitions. NTCP2 and SSU2 active
sessions mirror their existing byte-accounting points into that snapshot;
this does not add polling, probes, timers, network traffic, or persistence.

`LiveActivePeerSource` maps the neutral snapshot to `ActivePeerStats`, and the
existing handler serializes the exact five-field object for both the canonical
Proposal 170 selector and the retained compatibility selector.

## 5. Verification executed

All commands passed unless noted:

```text
cargo check -p emissary-core
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core inspection --no-fail-fast
cargo test -p emissary-core transport::ntcp2 --no-fail-fast
cargo test -p emissary-core transport::ssu2 --no-fail-fast
cargo test -p emissary-cli --no-default-features --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --test router_info_truthfulness
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards --test conformance_manifest --test router_info_truthfulness
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo fmt --all
git diff --check
```

The installed stable formatter emitted the repository-known warnings for
nightly-only options. Formatter-only changes outside the touched paths were
removed; no unrelated formatting churn is included.

## 6. Invariant review

1. No socket, cryptographic state, session key, Noise state, channel, mutable
   connection object, or transport command handle crosses the inspection
   boundary: pass; only owned `TransportPeerInspection` values are exposed.
2. Observation does not alter transport behavior: pass; updates are passive
   copies at existing transitions and byte-accounting points.
3. Per-peer memory is bounded by active sessions and snapshots by the existing
   10,000-item / 4 MiB response controls: pass.
4. I2PControl owns labels, ordering, bounds, serialization, and error mapping:
   pass.
5. Unsupported facts are not plausible defaults: pass; latency and unrelated
   peer metadata remain absent.

## 7. Failure, recovery, and contention review

Snapshot locks are held only while cloning owned state; no lock is held across
awaits or serialization. A peer disappearing during a snapshot is removed at
the existing close transition and is absent from a later coherent snapshot.
An empty snapshot is authoritative zero active peers. Counters are process
local, reconstructed with active sessions, and are not persisted or migrated.
Counter overflow saturates rather than wrapping.

## 8. Compatibility and security review

The canonical selector remains direct-presence and retains its declared
`array<object>` type. The existing compatibility serializer keeps its exact
five labels. No router algorithm, transport admission, handshake,
retransmission, congestion, or protocol-message behavior changed. No private
keys, session keys, addresses, live handles, or control authority are exposed.

## 9. Changed-path containment

Production changes are limited to the authorized I2PControl paths and these
neutral observation paths:

- `emissary-core/src/inspection.rs`;
- `emissary-core/src/transport/mod.rs`;
- `emissary-core/src/transport/ntcp2/**`;
- `emissary-core/src/transport/ssu2/**`.

No crypto, I2NP, NetDB, tunnel, router-algorithm, frontend, proxy,
AddressBook, workflow, release, or external/upstream path changed.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| — | None | — | — |

## 11. Roadmap disposition

M047 is formally closed. Its hard dependency M046 was closed before
implementation. M048 is dependency-ready; M049–M052 remain blocked behind
their named hard dependencies. The RouterInfo source matrix is now 24
available, 1 protocol-permitted neutral, and 18 unavailable.

## 12. Registry updates

- M047 implementation plan moved from `blocked` to `closed`.
- M047 closure was added here.
- M048 moved from `blocked` to `ready` as the sole dependency-ready successor.
- M049–M052 remain blocked.
- Roadmap and source documentation now record 24/1/18.

## 13. Internal-only attestation

Proposal 170 and reference material were accessed read-only. No upstream
repository or maintainer channel was mutated; no upstream issue, pull request,
review, submission, adoption, merge, maintainer contact, or contribution
artifact was created or requested.

**Disposition: M047 closed; M048 ready; RouterInfo source matrix 24/1/18.**
