# M078 — Streamr Local Boundary Hardening

Status: blocked — prewritten successor; M073 must close before registration advances

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Inherited implementation:

- M071 Streamr client/server runtime.

Planning production baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Read-only reference evidence:

- Java I2P `streamr/StreamrProducer.java`;
- Java I2P `streamr/Subscriber.java` at `i2p/i2p.i2p@498488b0` (10 subscriptions, 60-second expiry);
- Java UDP source behavior;
- current Emissary M071 bounds (16 subscribers, 60-second expiry, 15-second refresh, 1200-byte payload, 4095-byte receive buffer).

## 1. Objective

Tighten the Streamr local UDP trust boundary and reduce multicast amplification/state exposure while preserving the already bounded M071 control protocol.

The core correction is to make Proposal 170 Streamr local UDP ingress/egress loopback-only. There is no authenticated local publisher/consumer protocol in the pinned contract, so exposing the UDP socket on LAN/external interfaces permits unauthenticated traffic injection into an I2P-correlatable stream.

## 2. Confirmed strengths to preserve

M071 already provides:

- exact one-byte `0` subscribe/refresh and `1` unsubscribe controls;
- malformed/unknown control rejection;
- trusted remote destination identity from Yosemite;
- finite subscription set;
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

`streamrserver` currently allows an administrator-selected IP bind. If configured to `0.0.0.0`, a LAN address, or another reachable interface, any host capable of sending UDP to that port can publish attacker-selected traffic to all current I2P subscribers.

That is an integrity/abuse problem and can provide a distinctive traffic marker for correlation.

### Non-loopback client target

`streamrclient` forwards remote I2P payloads to a fixed administrator-selected local UDP target. The target is not remotely selected, which is good, but a non-loopback target still turns a remote I2P peer into a trigger for traffic toward a LAN/external UDP service.

### Fanout ceiling

Java's reference Streamr subscriber ceiling is 10. Emissary currently permits 16. Both are bounded, but aligning downward reduces worst-case fanout/amplification without reducing the declared Proposal 170 feature.

## 4. Hard invariants

- Streamr remains under `emissary-cli/src/i2pcontrol/backends/streamr.rs` or an equivalently local I2PControl module;
- no core/router UDP API change;
- no public auth/ACL field is invented;
- local producer bind must be loopback;
- local client output target must be loopback;
- remote payload never selects/rewrites the local target;
- subscriber identity remains authenticated remote destination plus the fixed configured session-port context available through Yosemite;
- subscription set is finite and no active entry is evicted to admit overflow;
- expiry/refresh use monotonic time;
- packet/body buffers remain hard bounded;
- no per-packet task spawn/fanout queue;
- cancellation/restart discards ephemeral subscriber state;
- persistent server destination identity remains stable/secret-safe.

## 5. Loopback-only policy

For both Streamr roles:

- absent local host/interface defaults to loopback;
- `127.0.0.1` and `::1` are accepted;
- any non-loopback `TargetHost`, `Host`, `ReachableBy`, or typed listen-interface value that would expose/target UDP outside loopback rejects start before SAM session or UDP socket allocation;
- do not silently coerce a requested non-loopback address back to loopback;
- error reports the option name/policy, not a secret/value dump.

This deliberately chooses safe truthful rejection over unauthenticated network exposure. If later maintainers require LAN Streamr, that needs a separately authorized authentication/source-ACL design rather than an exception in M078.

## 6. Subscriber/fanout bounds

Reduce `MAX_SUBSCRIBERS` from 16 to 10 to match the Java reference ceiling unless a compatibility fixture demonstrates a Proposal 170 requirement for a larger count. No such requirement is currently identified.

Keep:

- 60-second expiry;
- 15-second refresh;
- 1200-byte application payload cap;
- 4095-byte receive buffer ceiling.

Do not increase packet size or subscriber count in this milestone.

At subscription capacity, a new destination is rejected; existing subscriber state is not evicted.

## 7. Destination/control validation

Revisit the current 64 KiB textual destination acceptance bound. Establish a bound from the actual Yosemite/I2P destination representation where possible rather than retaining an arbitrary large string solely for convenience.

Requirements:

- ordinary valid reference destinations must remain accepted;
- malformed/control/whitespace-containing identities reject;
- memory worst case for all 10 subscribers is documented;
- control packet must remain exactly one byte;
- unknown control does not create/refresh state;
- unsubscribe of an absent subscriber is harmless and creates no state.

Do not hash away the only copy of the destination required for reply sends; fixed-size hashes may be used as map/accounting helpers only if the full destination remains bounded and necessary for Yosemite send operations.

## 8. Local UDP source semantics

Because the socket is loopback-only after this plan, ignore/drop any packet whose observed source address is unexpectedly non-loopback as defense in depth.

No local-publisher authentication handshake is added. Local processes remain within the host trust boundary; protecting against malicious same-host processes is outside this tunnel protocol's scope.

No arbitrary rate limiter is added unless deterministic tests demonstrate an unbounded queue/task/memory path. The existing receive loop and sequential maximum-10 fanout already provide backpressure/drop behavior at the UDP/socket boundary. Avoid inventing throughput limits absent a contract or demonstrated resource defect.

## 9. Ordered work packages

### WP1 — Loopback validation

Centralize Streamr local address validation for server bind and client target; fail before allocation.

### WP2 — Fanout/reference alignment

Reduce subscriber ceiling to 10 and retain existing expiry/refresh/payload constants.

### WP3 — Identity/control bounds

Derive/document a realistic destination-text bound and keep exact one-byte control semantics.

### WP4 — Lifecycle/adversarial tests

Test non-loopback rejection, capacity, expiry, restart, malformed controls, oversized packets, and fixed local target behavior.

### WP5 — Documentation

Update Streamr runtime boundary documentation to state loopback-only local UDP policy and reference-aligned subscriber ceiling.

## 10. Required tests

At minimum:

- default server bind is loopback;
- explicit `127.0.0.1` and `::1` accepted where platform/test fixture permits;
- `0.0.0.0`, LAN, public, and non-loopback IPv6 addresses reject before session/socket allocation;
- client non-loopback local target rejects before session allocation;
- remote payload cannot change target address/port;
- 10 subscribers accepted, 11th rejected without eviction;
- refresh of existing subscriber at capacity succeeds;
- expiry at 60 seconds removes stale entry;
- 15-second client refresh remains;
- exact one-byte controls enforced;
- oversized >1200-byte payload not forwarded;
- unexpected non-loopback local UDP source is ignored if directly testable;
- restart starts with empty subscriber state while reusing persistent server identity;
- private destination remains absent from logs/errors/Debug.

## 11. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Use focused paused-time Streamr tests for expiry/refresh closure evidence.

## 12. Acceptance criteria

M078 may close only when:

1. Streamr local UDP ingress and output target are loopback-only;
2. non-loopback requests fail before allocation and are not silently rewritten;
3. subscriber maximum is 10 and overflow never evicts active state;
4. 60s expiry, 15s refresh, 1200-byte payload, and bounded transport buffer remain intact;
5. destination/control memory/input bounds are explicit and tested;
6. no unbounded task/queue/state path is introduced;
7. all production changes remain under `i2pcontrol`;
8. no high/medium Streamr anonymity/integrity/resource finding remains.

## 13. Stop conditions

If Proposal 170 compatibility is proven to require non-loopback Streamr local exposure, stop rather than weaken the loopback policy. A safe LAN mode requires separate authentication/source-policy planning and explicit maintainer authorization.

Closure must attest external reference access was read-only and no upstream activity occurred.
