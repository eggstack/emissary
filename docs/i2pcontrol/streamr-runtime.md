# Streamr runtime boundary

M071 implements Proposal 170 `streamrclient` and `streamrserver` as a dedicated
bounded datagram family under `emissary-cli/src/i2pcontrol/backends/streamr.rs`.
It does not generalize the existing TCP tunnel runtime.

The server is a persistent Yosemite `DATAGRAM` session plus one administrator-
bound Tokio UDP source. The source is loopback-only: absent or explicit
`127.0.0.1`/`::1` configuration is accepted, while `TargetHost`, `Host`,
`ReachableBy`, or typed `ListenInterface` values that are non-loopback are
rejected before session or socket allocation. Unexpected non-loopback packet
sources are dropped as defense in depth. A datagram with exactly one byte is
control: `0` subscribes or refreshes, and `1` removes the authenticated remote
destination. Unknown bytes, malformed lengths, empty/control-invalid
identities, and new subscriptions after the cap are ignored. A single expiry
interval removes entries older than 60 seconds. The set is capped at 10
destinations, matching the Java reference ceiling, and is snapshotted before
sequential sends, so no lock or unbounded send-task queue is held across
network I/O. Destination text is capped at 524 bytes, so all ten identity
strings retain at most 5,240 bytes of destination text, plus fixed map/storage
overhead.

The client is a non-published Yosemite datagram session. It refreshes every 15
seconds, attempts one unsubscribe during a 100 ms shutdown window, and forwards
only payloads of at most 1200 bytes to the configured loopback UDP target. The
client output socket also binds to that loopback address. Remote payloads never
select or rewrite the fixed target. The server applies the same 1200-byte
application cap; Yosemite's 4095-byte receive ceiling is used as the bounded
receive buffer.

`TargetDestination` or `i2p.tunnel.streamrTarget` selects the client producer.
`TargetHost`/`Host` (or typed `ReachableBy`) selects the local IP, defaulting to
loopback, but every supplied local-address spelling must resolve to loopback;
values are never silently coerced. `TargetPort` is the client local UDP target
port and the server's I2P destination port; `Port` is the server local UDP
source port and optional client I2P source port. Unsupported I2CP/custom maps
and recognized tunnel
length/quantity/variance/signature/encryption fields fail before allocation.

Yosemite 0.7 returns the authenticated remote destination on receive but does
not expose inbound from/to port metadata. Subscription identity therefore uses
that trusted destination plus the fixed configured session port tuple. This is
an application-layer limitation documented here; no `emissary-core` change is
needed.
