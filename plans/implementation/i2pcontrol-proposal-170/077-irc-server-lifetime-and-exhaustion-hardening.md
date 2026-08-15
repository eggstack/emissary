# M077 — IRC Server Lifetime and Exhaustion Hardening

Status: blocked — hard dependencies M073 and M074

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Inherited implementation:

- M066 IRC client/server family;
- M074 shared server admission/rate hardening.

Planning production baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Read-only reference evidence:

- Java I2P `I2PTunnelIRCServer` at `i2p/i2p.i2p@498488b0`, including bounded registration and `DEFAULT_IRC_READ_TIMEOUT = 10 minutes` after registration.

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

A remote peer can therefore register correctly, stop transmitting, and hold one accepted-server task indefinitely. Before M074, 128 such streams could consume the full pool; after M074, peer fairness limits the damage from one destination, but idle resource retention remains unnecessary and still aids distributed/Sybil exhaustion.

## 3. Hard invariants

- preserve the M066 registration parser/rewrite contract;
- continue deriving presented hostname solely from trusted I2P destination;
- local IRCd connection still occurs only after valid registration;
- use M074 admission; do not create an IRC-specific connection limiter;
- post-registration timeout is inactivity-based and resets on traffic, not a fixed maximum connection lifetime;
- normal IRC PING/PONG/traffic keeps a connection alive;
- no per-message post-registration IRC parsing is added to the server side merely for timeout tracking;
- raw post-registration byte semantics remain intact;
- no lock across local connect or relay;
- no new core API/dependency unless already owned by i2pcontrol;
- stop/restart cancels exact generation/child relay;
- local target errors are sanitized.

## 4. Inactivity policy

Adopt a 10-minute post-registration inactivity ceiling, matching the Java reference scale.

Required semantics:

- the deadline begins after the sanitized registration is written to the local IRCd;
- any successful read/forward of bytes in either direction resets activity;
- the timeout must not be implemented as `timeout(10m, entire_connection_future)` because that would terminate active long-lived IRC sessions after 10 minutes;
- on inactivity expiry, close both relay directions and release the M074 admission lease promptly;
- cancellation wins over inactivity and uses the existing bounded stop path.

Implementation may use small relay loops with an activity notification/deadline, or an equivalent bounded helper local to I2PControl. Avoid a generalized network relay framework unless another existing i2pcontrol path genuinely consumes it.

## 5. Local target connection

Wrap `TcpStream::connect` in a bounded timeout; target 5 seconds for consistency with `httpserver`.

On timeout/refusal/error:

- do not expose target host, port, OS error, or filesystem detail to the remote peer;
- release the admission lease;
- keep the server runtime alive for unrelated future peers;
- do not mark the whole tunnel failed for one backend connection failure.

Whether a bounded IRC 499-style local error is sent or the stream is simply closed should follow the existing independently authored Emissary policy. Do not copy Java error text.

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

The 10-minute inactivity bound is a resource-ownership policy, not an attempt at timing obfuscation.

Do not add randomized idle expiry. Deterministic timeout is preferable because:

- clients/IRCd can maintain sessions through normal activity;
- randomness complicates testing and does not hide application traffic patterns;
- the main correlation defense is M074 peer fairness plus finite idle occupancy.

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
- no target host/port/OS detail appears in externally visible errors.

## 10. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Run focused paused-time IRC tests independently for closure evidence.

## 11. Acceptance criteria

M077 may close only when:

1. post-registration IRC occupancy has a 10-minute inactivity bound;
2. the bound resets on traffic and does not cap total active session lifetime;
3. local target connect is bounded;
4. all completion/error/cancellation paths release admission state;
5. the M066 registration anonymity filter remains non-bypassable;
6. no new protocol features or public fields are added;
7. production changes remain under `i2pcontrol`;
8. no high/medium IRC exhaustion/anonymity finding remains.

## 12. Stop conditions

Stop if implementing an actual inactivity timer would require parsing/reframing post-registration IRC bytes or changing router-core streaming behavior. A fixed total connection lifetime is not an acceptable substitute.

Closure must attest reference access was read-only and no upstream interaction occurred.
