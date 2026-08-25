# M077 — IRC Server Lifetime and Exhaustion Hardening

Status: closed — implementation and closure accepted; merged-head integration reconciled by M084 and M085

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Inherited implementation:

- M066 IRC client/server family;
- M074 shared server admission/rate hardening as corrected by M080 and pending M083;
- M080 canonical 32-byte trusted peer identity/admission boundary;
- M081 generic-server option truthfulness and M082 direct HTTP corrections are already closed and do not alter this IRC objective.

Planning production baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Current prerequisite baseline: `a35d2bc333ff0e8b9889cd133d8ef75a98faa049`.

Read-only reference evidence:

- Java I2P `I2PTunnelIRCServer`, including bounded registration and `DEFAULT_IRC_READ_TIMEOUT = 10 minutes` after registration.

## 0. Corrective readiness gate

M077 was previously advanced to `ready` after M082. A post-M082 review found remaining defects in the same shared admission/trusted-peer boundary consumed by `ircserver`:

- minute/no-history peer-state representability is not correctly distinguished;
- aggregate capacity derives from field precedence rather than all enabled limits;
- expired active peers can lose the expiry-index representation claimed by M080;
- trusted peer text may contain a valid parsed Destination plus unconsumed trailing bytes.

M083 owns those defects. M077 must remain blocked until M083 closes and the registry explicitly advances M077 again.

This gate does not change M077's IRC objective or expand its scope.

## 1. Objective

Prevent a peer that successfully completes IRC registration from pinning an accepted-server slot indefinitely while preserving normal long-lived IRC sessions.

M077 adds post-registration inactivity expiry, bounded local target connection, and explicit cancellation/error semantics. It does not alter the common IRC client-side anonymity filter, add WEBIRC, or add DCC support.

## 2. Confirmed IRC defect

The current `ircserver` registration path is already bounded:

- 5-second line timeout;
- 15-second total registration timeout;
- 12-line maximum;
- 1 KiB registration line bound;
- wrong-protocol rejection;
- authenticated peer-derived hostname rewrite before local connect.

After registration, however, the code opens the local IRCd and relays with raw bidirectional copies until one side closes. There is no inactivity deadline and no target-connect timeout.

A remote peer can register correctly, stop transmitting, and hold one accepted-server task indefinitely. M083 must first make the inherited admission/identity prerequisite valid; M077 then removes the IRC-specific idle occupancy.

## 3. Hard invariants

- M083 is closed before implementation starts;
- preserve the M066 registration parser/rewrite contract;
- continue deriving presented hostname solely from the M083-corrected trusted I2P Destination identity;
- consume the shared accepted-server admission boundary; do not create an IRC-specific connection limiter;
- local IRCd connection occurs only after valid registration;
- post-registration timeout is inactivity-based and resets on traffic, not a fixed maximum connection lifetime;
- normal IRC PING/PONG/traffic keeps a connection alive;
- no per-message post-registration IRC parsing is added merely for timeout tracking;
- raw post-registration byte semantics remain intact;
- no lock across local connect or relay;
- no new core API/dependency unless already owned by I2PControl;
- stop/restart cancels exact generation/child relay;
- local target errors are sanitized;
- no regression to M083 admission capacity, expiry-index, or canonical trusted-peer identity invariants;
- no upstream interaction.

## 4. Inactivity policy

Adopt a 10-minute post-registration inactivity ceiling, matching the Java reference scale.

Required semantics:

- deadline begins after sanitized registration is written to the local IRCd;
- any successful read/forward of bytes in either direction resets activity;
- do not implement `timeout(10m, entire_connection_future)` because that would terminate healthy long-lived sessions after 10 minutes;
- on inactivity expiry, close both relay directions and release the shared admission lease promptly;
- cancellation wins over inactivity and uses the existing bounded stop path.

Implementation may use small relay loops with an activity notification/deadline or an equivalent helper local to I2PControl. Do not create a generalized network relay framework unless an existing I2PControl path already requires the same abstraction.

## 5. Local target connection

Wrap `TcpStream::connect` in a bounded timeout; target 5 seconds for consistency with `httpserver`.

On timeout/refusal/error:

- do not expose target host, port, OS error, or filesystem detail to the remote peer;
- release the admission lease;
- keep the server runtime alive for unrelated future peers;
- do not mark the whole tunnel failed for one backend connection failure.

Whether the remote stream receives a small sanitized IRC-specific error or simply closes should follow existing independently authored Emissary policy. Do not copy Java error text.

## 6. Registration boundary revalidation

While touching the handler, revalidate but do not broaden:

- first-line wrong-protocol signatures;
- total registration deadline;
- maximum line count/length;
- USER rewrite from trusted canonical Destination identity;
- CAP/PASS/AUTHENTICATE/PING/PONG registration compatibility;
- malformed registration fails before local connect;
- WEBIRC remains unsupported/fail-before-allocation if configured;
- DCC remains outside this server-side milestone.

Do not loosen registration to improve compatibility without a separate finding and plan update.

## 7. Timing/correlation policy

The 10-minute inactivity bound is a resource-ownership policy, not timing obfuscation.

Do not add randomized idle expiry. Deterministic timeout is preferable because clients/IRCd can maintain sessions through normal activity and randomness does not hide application traffic patterns. The relevant anonymity/resource defense is finite occupancy plus M083-corrected peer-fair admission.

## 8. Ordered work packages

### WP1 — Idle-aware relay helper

Implement the smallest I2PControl-local bidirectional relay that resets an inactivity deadline on successful traffic.

### WP2 — IRC integration

Replace post-registration raw copy pair with the idle-aware relay and add bounded target connect.

### WP3 — Cancellation/error cleanup

Prove task/admission release on timeout, EOF, local error, panic isolation, and tunnel stop.

### WP4 — M083 prerequisite regression

Ensure IRC integration continues to consume canonical trusted identity and shared admission without adding independent capacity/identity state.

### WP5 — Regression tests/docs

Use paused Tokio time for deterministic idle tests and update IRC server runtime documentation.

## 9. Required tests

At minimum:

- registered peer with no traffic is closed after 10 minutes of inactivity;
- activity at 9 minutes resets the deadline and connection remains alive past the original 10-minute point;
- traffic in either direction resets inactivity;
- continuously active connection is not killed by a fixed total-lifetime timer;
- local target connect timeout is bounded and does not fail the whole server runtime;
- malformed/incomplete registration still never connects local IRCd;
- admission lease is released on idle expiry;
- admission lease is released on remote EOF;
- admission lease is released on local EOF/error;
- tunnel stop cancels an active relay within bounded shutdown;
- a timed-out peer does not affect unrelated peer capacity;
- M083 admission-state sizes/index invariants return to expected state after idle expiry;
- trusted peer presentation uses canonical M083 Destination text and the 32-byte accounting ID remains shared;
- trailing-byte/non-exact peer identity remains rejected before IRC handler/local target;
- no target host/port/OS detail appears in externally visible errors.

## 10. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc
cargo test -p emissary-cli --no-default-features --features i2pcontrol admission
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Run focused paused-time IRC tests independently for closure evidence. Keep verification local/package-scoped.

## 11. Acceptance criteria

M077 may close only when:

1. M083 is already independently closed;
2. post-registration IRC occupancy has a 10-minute inactivity bound;
3. the bound resets on traffic and does not cap total active session lifetime;
4. local target connect is bounded;
5. all completion/error/cancellation paths release M083-corrected admission state;
6. the M066 registration anonymity filter remains non-bypassable;
7. trusted peer presentation remains canonical/exact per M083 and no independent identity model is introduced;
8. no new protocol features or public fields are added;
9. production changes remain under I2PControl;
10. M061/M062/M063 containment remains green;
11. no high/medium IRC exhaustion/anonymity finding remains.

After M077 closes, registry sequencing may advance M078. M079 remains the independent final tunnel-security reclosure authority.

## 12. Stop conditions

Stop if implementing an actual inactivity timer would require parsing/reframing post-registration IRC bytes, changing router-core streaming behavior, weakening the M083 admission/trusted-identity boundary, or adding a generalized dependency/framework. A fixed total connection lifetime is not an acceptable substitute.

Closure must attest reference access was read-only and no upstream interaction occurred.
