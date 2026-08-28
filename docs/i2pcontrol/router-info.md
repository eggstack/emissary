# RouterInfo Method

Status: exact-wire implementation accepted against the pinned revision

This document describes the Proposal 170 `RouterInfo` JSON-RPC method implementation in Emissary.

## Overview

The `RouterInfo` method allows authenticated callers to request specific router state data using exact selector-by-presence behavior. Only requested selector keys appear in the response.

## Selector registry

`rpc.rs` contains 121 unique legacy/base selectors plus the exact 43-key
Proposal 170 addition manifest. The 121-key catalog is not counted as Proposal
170 coverage. The machine-readable manifest declares exact nested JSON types,
direct-presence semantics, mutation, owner, serializer, fixture, bound, and
source disposition for every addition: 41 available, 1 protocol-permitted
neutral, and 1 unavailable. Known-peer, active-peer, transport, tunnel, and
v4/v6 status/testing/error fields use bounded live owners; router news uses the
I2PControl-owned signed XML_GZ feed, while banned peers remain unavailable
because no canonical owner exists
for those facts. The 15-second transit-bandwidth field uses a bounded
I2PControl-owned sampler over the authoritative cumulative transit counter.
M051's semantic adjudication confirmed that the pinned proposal does not
authorize empty-string or empty-map capability placeholders for routers that
lack those owners.

Canonical Proposal 170 additions are selected by direct parameter presence and
are returned under the exact same key. Values are ignored. All four address-book
list additions and the subscription/config objects consume the shared runtime
address-book owner established by M022.

### Base compatibility boundary

The handler has two exclusive request modes:

- Direct mode accepts the retained base inventory plus the 43 Proposal 170
  additions, selects every key by presence, and applies the Proposal 170 source
  and response contract.
- Nested compatibility mode accepts only the historical base inventories and
  selects truthy boolean values in `Selector` (with the retained peer lookup
  string exception). It uses the historical base serializers.

The exact overlaps are `i2p.router.news`,
`i2p.router.addressbook.subscriptions`, and
`i2p.router.addressbook.config`. Nested requests preserve the legacy news
source and array/map metadata shapes; direct requests use the Proposal 170
disposition and `{path, entries}` metadata shape. Mixing modes fails before
any source query. The machine-readable inventories and overlap table are in
`rpc.rs::router_info_keys`.

### Legacy/base selector groups

| Group | Prefix | Count | Source |
|---|---|---|---|
| Identity/static | `i2p.router.identity`, `i2p.router.version`, `i2p.router.uptime` | 3 | Startup-retained values |
| Router news | `i2p.router.news` | 1 | I2PControl-owned signed XML_GZ feed; bounded 7-day cache |
| Clock skew | `i2p.router.clock.skew` | 1 | Compatibility alias; RouterInfo control source |
| Network status | `i2p.router.net.bw.*` | 2 | EventMetrics firewall status |
| Share ratio | `i2p.router.shareRatio` | 1 | Retained configuration |
| Configured BW | `i2p.router.configuredbw.*` | 2 | Retained configuration |
| UDP transport | `i2p.router.udp.*` | 7 | unsupported-inspection (no transport-specific source) |
| TCP transport | `i2p.router.tcp.*` | 7 | unsupported-inspection |
| NetDB | `i2p.router.netdb.*` | 10 | unsupported-inspection |
| Bandwidth | `i2p.router.bw.*` | 14 | MetricsSnapshot + RollingWindow |
| Tunnels | `i2p.router.tunnels.*` | 7 | EventMetrics (participating, live); administrative-store (configured); unsupported-inspection (exploratory, client, queue) |
| I2PTunnel | `i2p.router.iptunnels` | 1 | TunnelManager administrative store |
| Peers | `i2p.router.peers.*` | 10 | unsupported-inspection (no live source) |
| Logs | `i2p.router.log.*` | 2 | LogRing (tracing layer) |
| Address book | `i2p.router.addressbook.*` | 6 | AddressBook administrative store |

### Selector behavior

- Canonical additions: direct key presence selects, including `false`, `null`,
  and non-boolean values
- Compatibility nested `Selector`: only truthy boolean values select
- Direct and nested forms cannot be mixed
- Unknown selector keys return an error (`INVALID_PARAMS`)
- Only requested keys appear in the response (no unrelated keys)

## Startup-retained values

At startup, the following values are retained and never re-read from disk:

- `router_id`: Local router identity in Base64
- `router_info_bytes`: Serialized local RouterInfo bytes
- `router_info_b64`: Base64 encoding of serialized RouterInfo
- `startup_time`: Server startup instant for uptime calculation

These are set via `I2pControlState::set_startup_values()` during router initialization.

## Bounded metrics

### Cumulative counters (MetricsSnapshot)

- `total_transport_received`: Cumulative inbound transport bytes
- `total_transport_sent`: Cumulative outbound transport bytes
- `total_transit_received`: Cumulative inbound transit bytes
- `total_transit_sent`: Cumulative outbound transit bytes
- `connected_routers`: Number of connected routers
- `participating_tunnels`: Number of transit tunnels
- `tunnel_build_successes`: Cumulative tunnel build successes
- `tunnel_build_failures`: Cumulative tunnel build failures

Counters are monotonic except process restart. Reads are non-destructive.

The canonical `i2p.router.net.total.transit.bytes` selector reports the
forwarded/transmitted transit counter (`total_transit_sent`). It does not add
the received and sent counters together; those counters remain distinct source
metrics.

### Rolling window (RollingWindow)

1-second buckets covering multiple intervals:

| Interval | Buckets | Memory |
|---|---|---|
| 1 second | 1 | ~24 bytes |
| 15 seconds | 15 | ~360 bytes |
| 1 minute | 60 | ~1.4 KB |
| 1 hour | 3600 | ~86 KB |
| 1 day | 86400 | ~2 MB |

Rolling window resets on process restart. No historical data is fabricated.

## Bounded log buffer (LogRing)

- Fixed maximum entries (default: 1000) and total bytes (default: 512 KB)
- Deterministic oldest-entry eviction
- Redaction of Base64 private keys (>=40 chars), `password=`, and `token=` patterns
- Clear affects only the I2PControl ring; terminal/file sinks unchanged
- Concurrent readers receive coherent snapshot
- Wired as `tracing_subscriber::Layer` for automatic event capture

## Response budgets

Pre-query budget estimation prevents oversized responses:

| Selector | Limit |
|---|---|
| Peer identities (known/active) | 10,000 |
| Peer RouterInfo bytes | 4 MB |
| Active peer stats | 10,000 |
| Banned peers | 10,000 |
| Log entries | 10,000 |
| Total response | 10 MB |

If estimated response exceeds bounds, the request fails with an explicit error before any expensive queries are issued.

Per-selector item bounds enforce limits on returned collections.

## Canonical source status

Available canonical fields include retained identity/info, cumulative byte
counters, share ratio, I2PTunnel controller info, total success rate, logs,
log clear, and all six address-book objects/lists. Clock skew is the one
protocol-permitted neutral field and returns `null` when no estimate exists.
Router news and banned-peer fields without an authoritative bounded source
return the established JSON-RPC unavailable error; M026 found no adjacent
  authoritative bounded owner to add, and Emissary never substitutes
zero, false, or an empty collection.

## Null/unavailable behavior

- Clock skew: `null` when not yet determined (protocol-permitted nullable)
- Router news: signed XML_GZ feed rendered to bounded HTML; unavailable before a
  validated generation or after seven days without a successful refresh
- Banned peers: unavailable because Emissary has no authoritative ban-list owner;
  an empty map is not an authorized capability-empty value
- Peer RouterInfo: `null` when no peer ID specified
- Network status: exact string compatibility codes and Proposal 170 numeric
  status/error/testing mappings from independent v4/v6 state
- Share ratio: from retained configuration
- Unavailable non-null selectors: return JSON-RPC error with no partial result
- Available-zero selectors: return successful zero/empty values
- Source failure: distinguished from unavailable and from empty

## Read-only architecture

- No mutation of router state from inspection requests
- No consumption of `EventSubscriber` (frontend events preserved)
- No mutable core handles exposed
- No private keys, session keys, or authentication tokens in responses
- Log redaction applied before ring insertion
- Core remains free of HTTP/JSON-RPC dependencies

## Limitations (permanent unsupported-inspection)

- NetDB, TCP transport, UDP transport, peer list/lookup/stats: `unsupported-inspection`
- Tunnel exploratory/client inbound/outbound, queue depth: `unsupported-inspection`
- UDP/TCP active/firewalled: no transport-specific canonical source in `EventMetrics`
