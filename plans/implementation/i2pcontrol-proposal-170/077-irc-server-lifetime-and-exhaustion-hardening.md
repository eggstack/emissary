# M077 — IRC Server Lifetime and Exhaustion Hardening

Status: blocked — post-M076 corrective gate; M080, M081, and M082 must close before M077 becomes dependency-ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Inherited implementation:

- M066 IRC client/server family;
- M074 shared server admission/rate hardening as corrected by M080.

Planning production baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Current corrective prerequisite baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Read-only reference evidence:

- Java I2P `I2PTunnelIRCServer`, including bounded registration and `DEFAULT_IRC_READ_TIMEOUT = 10 minutes` after registration.

## 0. Corrective readiness gate

M077 was previously marked ready after M076 closure. An independent review later found material defects in the shared M074 admission boundary and related M075/M076 security invariants. Because this IRC server consumes the same accepted-server admission infrastructure, M077 must not begin against the invalidated prerequisite state.

M077 becomes dependency-ready only after:

- M080 closes transactional/bounded server admission and canonical trusted peer accounting;
- M081 closes generic-server option truthfulness so the tunnel-security workstream has no known persist-and-ignore regression;
- M082 closes the HTTP trusted-peer/Expect/accounting follow-up so the earlier security closure sequence is reconciled before advancing.

This gate does not change M077's IRC objective or expand its scope.

## 1. Objective

Prevent a peer that successfully completes IRC registration from pinning an accepted-server slot indefinitely while preserving normal long-lived IRC sessions.

M077 adds post-registration inactivity expiry, bounded local target connection, and explicit cancellation/error semantics. It does not alter the common IRC client-side anonymity filter, add WEBIRC, or add DCC support.

## 2. Confirmed defect

The current `ircserver` registration path is strong:

- 5-second line timeout;
- 15-second total registration timeout;
- 12-line maximum;
- 1 KiB registration line bound;
- wrong-protocol rejection;
- authenticated peer-derived hostname rewrite before local connect.

After registration, however, the code opens the local IRCd and uses two raw `io::copy` futures until one side closes. There is no inactivity deadline and no target-connect timeout.

A remote peer can therefore register correctly, stop transmitting, and hold one accepted-server task indefinitely. M080 will restore the intended peer-fair admission boundary, but idle resource retention remains unnecessary and still aids distributed/Sybil exhaustion.

## 3. Hard invariants

- preserve the M066 registration parser/rewrite contract;
- continue deriving presented hostname solely from trusted I2P destination;
- consume the M080-corrected admission boundary; do not create an IRC-specific connection limiter;
- local IRCd connection still occurs only after valid registration;
- post-registration timeout is inactivity-based and resets on traffic, not a fixed maximum connection lifetime;
- normal IRC PING/PONG/traffic keeps a connection alive;
- no per-message post-registration IRC parsing is added merely for timeout tracking;
- raw post-registration byte semantics remain intact;
- no lock across local connect or relay;
- no new core API/dependency unless already owned by I2PControl;
- stop/restart cancels exact generation/child relay;
- local target errors are sanitized;
- no regression to M080 canonical peer identity/admission state;
- no upstream interaction.

## 4. Inactivity policy

Adopt a 10-minute post-registration inactivity ceiling, matching the Java reference scale.

Required semantics:

- the deadline begins after sanitized registration is written to the local IRCd;
- any successful read/forward of bytes in either direction resets activity;
- the timeout must not be implemented as `timeout(10m, entire_connection_future)` because that would terminate active long-lived IRC sessions after 10 minutes;
- on inactivity expiry, close both relay directions and release the M080 admission lease promptly;
- cancellation wins over inactivity and uses the existing bounded stop path.

Implementation may use small relay loops with an activity notification/deadline, or an equivalent bounded helper local to I2PControl. Avoid a generalized network relay framework unless another existing I2PControl path genuinely consumes it.

## 5. Local target connection

Wrap `TcpStream::connect` in a bounded timeout; target 5 seconds for consistency with `httpserver`.

On timeout/refusal/error:

- do not expose target host, port, OS error, or filesystem detail to the remote peer;
- release the admission lease;
- keep the server runtime alive for unrelated future peers;
- do not mark the whole tunnel failed for one backend connection failure.

Whether a bounded IRC-specific local error is sent or the stream is simply closed should follow the existing independently authored Emissary policy. Do not copy Java error text.

## 6. Registration boundary revalidation

While touching the handler, revalidate but do not broaden:

- first-line wrong-protocol signatures;
- total registration deadline;
- maximum line count/length;
- USER rewrite from trusted destination;
- CAP/PASS/AUTHENTICATE/PING/PONG registration compatibility;
- malformed registration fails before local connect;
- WEBIRC remains unsupported/fail-before-allocation if configured;
- DCC remains outside this server-side milestone.

Do not loosen registration to improve compatibility without a separate finding and plan update.

## 7. Timing/correlation policy

The 10-minute inactivity bound is a resource-ownership policy, not timing obfuscation.

Do not add randomized idle expiry. Deterministic timeout is preferable because clients/IRCd can maintain sessions through normal activity and randomness does not hide application traffic patterns. The relevant anonymity/resource defense is finite occupancy plus the M080 peer-fair admission state.

## 8. Ordered work packages

### WP1 — Idle-aware relay helper

Implement the smallest I2PControl-local bidirectional relay that resets an inactivity deadline on successful traffic.

### WP2 — IRC integration

Replace post-registration raw copy pair with the idle-aware relay and add bounded target connect.

### WP3 — Cancellation/error cleanup

Prove task/admission release on timeout, EOF, local error, panic isolation, and tunnel stop.

### WP4 — Regression tests/docs

Use paused Tokio time for deterministic idle tests and update the IRC server runtime documentation.

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
- M080 admission-state sizes return to expected values after idle expiry;
- no target host/port/OS detail appears in externally visible errors.

## 10. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Run focused paused-time IRC tests independently for closure evidence.

## 11. Acceptance criteria

M077 may close only when:

1. M080-M082 are already independently closed;
2. post-registration IRC occupancy has a 10-minute inactivity bound;
3. the bound resets on traffic and does not cap total active session lifetime;
4. local target connect is bounded;
5. all completion/error/cancellation paths release corrected admission state;
6. the M066 registration anonymity filter remains non-bypassable;
7. no new protocol features or public fields are added;
8. production changes remain under I2PControl;
9. no high/medium IRC exhaustion/anonymity finding remains.

## 12. Stop conditions

Stop if implementing an actual inactivity timer would require parsing/reframing post-registration IRC bytes, changing router-core streaming behavior, or weakening the M080 admission boundary. A fixed total connection lifetime is not an acceptable substitute.

Closure must attest reference access was read-only and no upstream interaction occurred.
