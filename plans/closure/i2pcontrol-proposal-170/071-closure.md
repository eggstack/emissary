# M071 Closure — Streamr Client and Server Tunnels

Status: closed against the local implementation head

M071 promotes both Proposal 170 Streamr tunnel types to bounded production
backends without changing `emissary-core` or the public tunnel schema.

## Implementation

- `emissary-cli/src/i2pcontrol/backends/streamr.rs` owns the dedicated client
  and server Yosemite repliable-datagram loops, lifecycle generations, local
  UDP endpoints, subscription state machine, and adversarial unit tests.
- `backends/options.rs` declares Streamr option capabilities and rejects I2CP,
  custom, and recognized-but-unimplemented runtime options before allocation.
- `backends/registry.rs` promotes `streamrclient` and `streamrserver` only in
  the composed production registry; the dependency-light default registry
  remains intentionally unsupported for test isolation.
- `production.rs` includes Streamr server definitions in identity persistence,
  public-destination publication, start-on-load reconciliation, runtime
  inspection, and cleanup.

## Yosemite mapping

Yosemite 0.7 `Session<style::Repliable>` is used directly. The server creates a
published persistent session with `DestinationKind::Persistent`; the client
creates a non-published transient session. `send_to_with_options` carries
control/payload datagrams and `recv_from` returns the authenticated remote
destination. Yosemite does not expose inbound from/to port metadata, so the
subscription key is the trusted remote destination and sends use the configured
fixed session port tuple. No core/router API was added.

## Bounds and control matrix

| Property | Adopted value/evidence |
|---|---|
| Subscriber cap | 16 unique trusted destinations |
| Expiry | 60 seconds without refresh; one 5-second scan interval |
| Refresh | Client sends `0` immediately and every 15 seconds |
| Shutdown | One best-effort `1` unsubscribe with a 100 ms timeout |
| Transport receive buffer | 4095 bytes, Yosemite's 0xfff ceiling |
| Application payload | 1200 bytes; oversized packets are dropped |
| Fan-out | Snapshot then sequential bounded sends; no per-packet task queue |
| Control | Exactly `[0]` subscribe/refresh, `[1]` unsubscribe; other lengths/bytes ignored |

The subscription tests prove flood rejection, refresh de-duplication,
exact-peer unsubscribe, expiry, malformed/unknown control rejection, and empty
state after removal. The single-owner runtime loops ensure one failed send does
not terminate fan-out to later peers and restart creates a fresh empty
subscription map while the server identity remains in the existing secret store.

## Local target and option disposition

`TargetDestination` or `i2p.tunnel.streamrTarget` selects the client producer.
`TargetHost`/`Host` or typed `ReachableBy` is parsed as an IP address and
defaults to loopback. `TargetPort` is required for the client local UDP target
and is the server's configured I2P destination port; `Port` is the server local
UDP source port and optional client I2P source port. Remote payloads never select
the local target. I2CP/custom maps and recognized tunnel length, quantity,
variance, signature, and encryption fields are rejected before session/UDP
allocation.

## Verification

Focused and containment checks are run locally before commit:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
git diff --check
```

The full feature-enabled package tests and clippy checks are also required by
the repository handoff. No public-network test or hosted CI expansion is part
of M071.

## Scope and security disposition

No generic UDP tunnel, multicast, reliability layer, packet-selected local
target, unbounded subscriber/task state, or core production path was added.
The implementation is internal to `eggstack/emissary`; no upstream interaction
or external contribution preparation is implied.
