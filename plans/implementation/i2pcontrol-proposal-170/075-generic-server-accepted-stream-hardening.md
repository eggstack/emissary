# M075 — Generic Server Accepted-Stream Hardening

Status: blocked — hard dependencies M073 and M074

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Corrective context:

- M072 identified generic option truthfulness gaps;
- M073 owns apply-or-reject repair;
- M074 owns the shared peer-aware accepted-server admission boundary.

Planning production baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

## 1. Objective

Move the control-plane generic Proposal 170 `server` backend from blind SAM `STREAM FORWARD` to the existing application-visible accepted-stream model, then relay admitted streams byte-for-byte to the fixed local target.

This is not a protocol filter. It is a security-boundary correction so generic servers receive the same authenticated peer admission, concurrency, and rate-limit protections as the specialized accepted-stream servers.

## 2. Confirmed defect

The current generic backend delegates to `emissary-cli/src/tunnel/server.rs::run_single_server`, which creates a persistent Yosemite streaming session and issues `session.forward(local_port)`. Once forwarding is installed, I2PControl does not see individual remote peers and cannot enforce its own per-peer admission/fairness semantics.

Consequences:

- generic `server` bypasses M074's peer-aware admission state;
- Proposal 170 server rate/security options cannot be enforced at the application boundary;
- the control plane can truthfully reject such fields under M073, but the running generic service still has a weaker resource-exhaustion boundary than the new filtered servers.

M075 corrects the runtime shape without adding application-layer protocol semantics.

## 3. Hard invariants

- no `emissary-core/**` production change;
- no new router/SAM protocol extension;
- reuse M065/M074 accepted-server runtime; do not create a second accept loop;
- persistent server destination identity remains the existing backend-owned stored identity;
- public destination remains stable across stop/start/restart when the stored identity is unchanged;
- remote identity comes from accepted Yosemite stream only;
- local target host/port are administrator-selected and fixed before accepting traffic;
- local target remains loopback-only under the current data-plane policy;
- after admission and local connect, payload bytes are relayed without HTTP/IRC/SOCKS interpretation or rewriting;
- no local target connection for a denied peer;
- no lock held across target connect or relay;
- startup-managed generic servers remain outside TunnelManager ownership;
- no secret material in errors/logs/Debug/status.

## 4. Target data path

```text
remote I2P peer
    -> persistent Yosemite accepted-stream session
    -> TrustedPeerIdentity
    -> M074 ServerAdmissionState
    -> fixed loopback TCP target
    -> raw bidirectional relay
```

This path replaces only the control-plane-owned generic `server` runtime. Existing startup server managers remain untouched.

## 5. Runtime configuration

Create/use a generic-server runtime configuration that contains only:

- tunnel name;
- SAM TCP port;
- persistent destination secret wrapper;
- fixed target host/port;
- M074 admission policy;
- only I2CP/session options demonstrably consumed by Yosemite.

Do not pass arbitrary raw configuration through to Yosemite.

M073 remains the truthfulness authority for typed/raw option validation. M075 may expand the accepted generic-server set only for options it now genuinely implements through the accepted-stream/admission path.

## 6. Proposal 170 server controls

After M074 exists, generic `server` SHOULD support the same connection admission controls where semantics are protocol-independent:

- `MaxConcurrentConns`;
- `ClientPerMinute`;
- `ClientPerHour`;
- `ClientPerDay`;
- `TotalInPerMinute`;
- `TotalInPerHour`;
- `TotalInPerDay`.

An access allow/deny list may be implemented here only if the existing `AccessList`/`AccessOption` domain mapping can be applied directly to authenticated peer destinations without inventing new syntax. If implemented, it must run before local target connect and reuse a common peer-match helper. Otherwise reject it truthfully.

Do not implement in this milestone:

- `FilterFilePath` parser compatibility;
- `UniqueLocalAddressPerClient`;
- `MultiHoming`;
- TLS termination;
- HTTP/IRC filtering;
- arbitrary target host routing;
- guessed `PerClientPeriod`/`TotalPeriod`/`TotalBanTime` semantics.

## 7. Local relay semantics

Target connection requirements:

- resolve no attacker-controlled hostname;
- accept only loopback literals/accepted localhost spelling under current policy;
- use a bounded target-connect timeout (target 5 seconds, matching the specialized server pattern);
- on connect failure, close/reset the I2P stream without exposing local IP/path/OS error detail;
- perform raw bidirectional relay after connect;
- propagate EOF/shutdown cleanly;
- cancellation stops the exact running generation and drains/aborts child relays within the existing bounded stop policy.

Do not impose a short generic application idle timeout merely to simplify resource accounting. Generic server protocols may legitimately be idle. The M074 per-peer concurrent ceiling exists specifically so one peer cannot consume the whole service with long-lived generic streams. A generic inactivity policy would require separate compatibility research.

## 8. Migration/lifecycle

This is an in-memory runtime implementation change only. No durable definition or destination-store migration is required.

Start/restart sequence:

1. validate all effective options;
2. load the existing persistent destination secret;
3. reserve one exact runtime generation;
4. create accepted Yosemite session;
5. publish actual public destination;
6. report running only after session readiness;
7. accept/admit/relay individual streams;
8. on stop, cancel the exact generation and bounded-drain active relays.

A failed start must leave the durable definition and persistent identity available for later retry.

## 9. Ordered work packages

### WP1 — Generic accepted handler

Add the smallest raw accepted-stream handler that performs fixed local target connection and byte relay.

### WP2 — Backend migration

Change only the control-plane generic `server` backend to compose `run_accepted_server` + M074 admission + raw handler rather than `run_single_server`/`STREAM FORWARD`.

### WP3 — Option application

Expose supported protocol-independent server admission fields after M073 validation. Keep all other fields explicit apply-or-reject.

### WP4 — Lifecycle/identity regression tests

Prove start/stop/restart, stable destination, failed target connect, capacity denial, and cancellation behavior.

### WP5 — Containment/documentation

Update the backend/support docs to distinguish startup server forwarding from the control-plane accepted-stream runtime. Do not refactor startup code.

## 10. Required tests

At minimum:

- fake SAM proves generic control-plane `server` issues accepted-stream operations and does not issue `STREAM FORWARD`;
- admitted payload reaches the fixed local TCP target byte-for-byte in both directions;
- rejected peer never causes local target connect;
- one peer at M074 peer concurrency cap cannot block an unrelated peer while global capacity remains;
- rate-limit rejection follows M074 behavior;
- target host outside loopback rejects before session allocation;
- target connect timeout/failure returns sanitized failure and leaves runtime alive for later peers;
- persistent destination/public identity is stable across stop/start;
- stop/restart cannot let an old generation mutate a new generation;
- child relay panic/error does not kill unrelated streams;
- private destination never appears in Debug/error/status;
- startup-managed server code/path is unchanged;
- M061 path-containment tests remain green.

## 11. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Run focused fake-SAM generic-server tests separately so the closure record can show that `STREAM FORWARD` is absent from the control-plane path.

## 12. Acceptance criteria

M075 may close only when:

1. control-plane generic `server` no longer uses blind SAM forwarding;
2. it reuses M074 admission rather than duplicating limiter state;
3. raw payload semantics are preserved after admission;
4. fixed loopback target ownership is preserved;
5. destination identity remains stable and secret-safe;
6. no unsupported runtime option is silently accepted;
7. startup server code remains untouched;
8. no production change lands outside `emissary-cli/src/i2pcontrol/**`;
9. all focused/package/containment checks pass;
10. no high/medium finding remains in the generic-server scope.

## 13. Stop conditions

If Yosemite's existing accepted-stream API cannot support a transparent generic relay without a new core API or materially different stream semantics, stop M075 and record the exact limitation. Do not broaden `emissary-core`, SAM, or startup server ownership inside this plan.

Closure must attest external source access was read-only and no upstream activity occurred.
