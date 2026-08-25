# M078 — Streamr Local Boundary Hardening

Status: closed — implementation and closure accepted; merged-head integration reconciled by M084 and M085

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Inherited implementation:

- M071 Streamr client/server runtime.

Original planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Current corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Read-only reference evidence:

- Java I2P Streamr producer/subscriber behavior, including 10 subscriptions and 60-second expiry;
- current Emissary M071 bounds.

## 0. Corrective readiness gate

M078 remains technically separate from the server-admission/HTTP findings, but the planning process registers one dependency-ready handoff at a time and the tunnel-security workstream must reconcile M080-M082 before advancing to later family hardening.

M078 becomes eligible only after M080, M081, M082, and M077 close.

## 1. Objective

Tighten the Streamr local UDP trust boundary and reduce multicast amplification/state exposure while preserving the already bounded M071 control protocol.

The core correction is to make Proposal 170 Streamr local UDP ingress/egress loopback-only. There is no authenticated local publisher/consumer protocol in the pinned contract, so exposing the UDP socket on LAN/external interfaces permits unauthenticated traffic injection into an I2P-correlatable stream.

## 2. Confirmed strengths to preserve

M071 already provides:

- exact one-byte `0` subscribe/refresh and `1` unsubscribe controls;
- malformed/unknown control rejection;
- trusted remote destination identity from Yosemite;
- finite subscription state;
- 60-second expiry;
- 15-second refresh;
- 1200-byte application payload cap;
- 4095-byte Yosemite transport receive buffer;
- no remote packet-driven local target selection;
- sequential bounded fanout without unbounded send tasks;
- bounded shutdown/unsubscribe attempt.

Do not replace these with a generalized UDP framework.

## 3. Confirmed risks

### Non-loopback producer ingress

`streamrserver` currently allows an administrator-selected IP bind. If configured to `0.0.0.0`, a LAN address, or another reachable interface, another host can publish attacker-selected traffic to current I2P subscribers. This is an integrity/abuse issue and can provide a distinctive correlation marker.

### Non-loopback client target

`streamrclient` forwards remote I2P payloads to a fixed administrator-selected local UDP target. The target is not remotely selected, but a non-loopback target still lets a remote I2P peer trigger traffic toward a LAN/external UDP service.

### Fanout ceiling

Java's reference Streamr subscriber ceiling is 10; Emissary currently permits 16. Aligning downward reduces worst-case fanout without reducing the declared Proposal 170 feature.

## 4. Hard invariants

- Streamr production logic remains under I2PControl;
- no core/router UDP API change;
- no public auth/ACL field is invented;
- local producer bind is loopback-only;
- local client output target is loopback-only;
- remote payload never selects/rewrites the local target;
- subscriber identity remains authenticated remote Destination/session context;
- subscription set is finite and overflow does not evict active entries;
- expiry/refresh use monotonic time;
- packet/body buffers remain hard bounded;
- no per-packet task spawn/fanout queue;
- cancellation/restart discards ephemeral subscriber state;
- persistent server destination identity remains stable/secret-safe;
- trusted peer identity validation must remain compatible with the canonical Destination boundary established by M080.

## 5. Loopback-only policy

For both Streamr roles:

- absent local host/interface defaults to loopback;
- `127.0.0.1` and `::1` are accepted where platform support exists;
- any non-loopback `TargetHost`, `Host`, `ReachableBy`, or typed listen-interface value that would expose/target UDP outside loopback rejects before SAM session or UDP socket allocation;
- do not silently coerce non-loopback requests to loopback;
- error identifies the option/policy, not secret/raw values.

If later maintainers require LAN Streamr, that needs separate authentication/source-policy planning.

## 6. Subscriber/fanout bounds

Reduce `MAX_SUBSCRIBERS` from 16 to 10 unless a direct Proposal 170 compatibility requirement proves a larger value necessary.

Keep:

- 60-second expiry;
- 15-second refresh;
- 1200-byte application payload cap;
- 4095-byte receive buffer ceiling.

At capacity, a new Destination is rejected; refresh of an existing subscriber remains possible; active state is not evicted.

## 7. Destination/control validation

Do not introduce another arbitrary small textual Destination ceiling. Reuse the M080 trusted peer/canonical Destination validation where the Streamr API path permits it.

Requirements:

- all structurally valid current Destinations accepted by the accepted-stream/datagram layer remain usable;
- malformed/control/whitespace/non-Destination identity rejects;
- memory worst case for all 10 subscribers is documented;
- control packet remains exactly one byte;
- unknown control creates/refreshes no state;
- unsubscribe of absent subscriber is harmless.

Where full Destination text is required for Yosemite send operations, retain one bounded validated copy; use canonical fixed-size identity for lookup/accounting where practical.

## 8. Local UDP source semantics

Because producer ingress becomes loopback-only, ignore/drop any unexpectedly observed non-loopback source as defense in depth.

No local publisher authentication handshake or arbitrary rate limiter is added. Same-host process trust remains outside this protocol's scope unless a concrete unbounded queue/task defect is discovered.

## 9. Ordered work packages

### WP1 — Loopback validation

Centralize Streamr local address validation for server bind and client target; fail before allocation.

### WP2 — Fanout/reference alignment

Reduce subscriber ceiling to 10 and retain expiry/refresh/payload constants.

### WP3 — Identity/control bounds

Consume the corrected trusted Destination boundary from M080 and keep exact one-byte controls.

### WP4 — Lifecycle/adversarial tests

Test non-loopback rejection, capacity, expiry, restart, malformed controls, oversized packets, and fixed local target behavior.

### WP5 — Documentation

Update runtime/support docs with loopback-only policy and reference-aligned subscriber ceiling.

## 10. Required tests

At minimum:

- default server bind is loopback;
- explicit loopback IPv4/IPv6 accepted where supported;
- `0.0.0.0`, LAN, public, and non-loopback IPv6 addresses reject before session/socket allocation;
- client non-loopback target rejects before session allocation;
- remote payload cannot change target address/port;
- 10 subscribers accepted, 11th rejected without eviction;
- refresh at capacity succeeds for an existing subscriber;
- expiry at 60 seconds removes stale entry;
- 15-second client refresh remains;
- exact one-byte controls enforced;
- oversized >1200-byte payload not forwarded;
- unexpected non-loopback UDP source is ignored if directly testable;
- restart begins with empty subscriber state while persistent server identity is reused;
- valid current trusted Destinations larger than legacy assumptions are not rejected solely by text length;
- private destination remains absent from diagnostics.

## 11. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Use focused paused-time Streamr tests for expiry/refresh closure evidence.

## 12. Acceptance criteria

M078 may close only when:

1. M080-M082 and M077 are independently closed;
2. Streamr local UDP ingress and output target are loopback-only;
3. non-loopback requests fail before allocation and are not silently rewritten;
4. subscriber maximum is 10 and overflow never evicts active state;
5. 60s expiry, 15s refresh, 1200-byte payload, and bounded transport buffer remain intact;
6. trusted identity/control memory bounds are explicit and compatible with M080;
7. no unbounded task/queue/state path is introduced;
8. production changes remain under I2PControl;
9. no high/medium Streamr anonymity/integrity/resource finding remains.

## 13. Stop conditions

If Proposal 170 compatibility is proven to require non-loopback Streamr local exposure, stop rather than weaken the loopback policy. A safe LAN mode requires separate authentication/source-policy planning and explicit maintainer authorization.

Closure must attest external reference access was read-only and no upstream activity occurred.
