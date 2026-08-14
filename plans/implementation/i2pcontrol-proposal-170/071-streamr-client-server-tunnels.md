# M071 — Streamr Client and Streamr Server Tunnels

Status: blocked — hard dependency on M065 closure

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Authority:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

Hard dependency:

- M065 runtime/option-capability foundation closed.

## 1. Objective

Implement the Proposal 170 `streamrclient` and `streamrserver` tunnel types as a small bounded datagram producer/consumer system over existing SAM/Yosemite datagram support and Tokio UDP.

Streamr must remain isolated from the streaming TCP families. M071 should not introduce a generalized transport framework merely to share supervision code.

## 2. Adopted behavioral model

The reference Streamr design has two complementary roles:

- producer/server: owns a persistent I2P datagram identity, receives local UDP payloads, and forwards them to currently subscribed remote I2P consumers;
- consumer/client: owns an I2P datagram client session, periodically sends subscription refresh messages to the producer, and forwards received payloads to a configured local UDP target.

The reference control protocol uses one-byte subscription messages:

- `0` = subscribe/refresh;
- `1` = unsubscribe.

Subscriptions expire if not refreshed and the producer bounds the subscriber set. M071 independently implements these semantics with explicit bounds and lifecycle.

## 3. Classification

Primary class: capability / resource-safety.

Types promoted on closure:

- `streamrclient`;
- `streamrserver`.

## 4. Hard invariants

- production logic under `emissary-cli/src/i2pcontrol/**`;
- no `emissary-core/**` change;
- use existing SAM/Yosemite datagram APIs; if unavailable/inadequate, stop and replan rather than adding core protocol APIs;
- server identity uses backend-owned secret-store authority where persistent destination material is required;
- subscription state bounded by a fixed/configured maximum;
- subscriptions expire without refresh;
- packet sizes bounded to the actual datagram transport capability and a documented application cap;
- control packets distinguished from payload deterministically;
- local UDP targets are admin-configured only; incoming packets cannot choose arbitrary host/port;
- no reflection/amplification to unbounded subscriber sets;
- no lock held across socket/datagram send/receive waits;
- stop/restart cancellation exact and task-safe;
- relevant Proposal 170 options applied or rejected before allocation;
- no startup datagram task adoption.

## 5. Explicit non-goals

Do not:

- implement generic UDP tunneling outside Streamr semantics;
- add SOCKS UDP ASSOCIATE;
- create multicast/broadcast support beyond bounded subscribed destinations;
- implement reliability/retransmission/FEC on top of datagrams;
- build a generalized pub/sub framework;
- permit packet-driven local destination selection;
- persist ephemeral subscriber lists across restart;
- add core datagram APIs unless a separate architecture plan is approved.

## 6. Readiness audit

Before coding, confirm exact Yosemite APIs for:

- datagram session creation style(s);
- sending to destination with from/to port metadata if available;
- receiving remote destination + port metadata;
- persistent versus transient destination configuration;
- payload size/error behavior;
- cancellation/task ownership.

Map these APIs to the adopted Streamr control semantics. If remote destination identity or required port metadata is unavailable, stop M071 and document the exact gap.

## 7. `streamrserver` / producer requirements

### 7.1 Runtime topology

```text
configured local UDP source
    -> bounded UDP receive
    -> fan-out to current bounded I2P subscribers

remote I2P datagram control packet
    -> validate peer/control/from/to ports
    -> subscribe/refresh/unsubscribe bounded state
```

The server owns a persistent I2P destination identity through existing backend secret authority.

### 7.2 Subscription key

Key subscriptions by the minimum tuple required to route replies correctly, expected to include:

- remote I2P destination/public identity;
- remote/client receive port and/or source port according to Yosemite datagram API;
- producer destination port if the protocol uses it.

Do not key by untrusted textual nickname/address when trusted destination identity exists.

### 7.3 Control message validation

Control datagram requirements:

- payload length exactly one byte for control messages;
- `0` subscribes or refreshes timestamp;
- `1` removes subscription;
- other values ignored/rejected without creating state;
- malformed/null peer identity cannot create subscription;
- control datagram itself is never forwarded to local UDP source or other subscribers.

### 7.4 Bounds and expiry

Define explicit constants/configured caps for:

- max subscriptions (reference scale is small, e.g. ~10; choose/justify actual value);
- subscription expiry duration (reference scale ~60s; choose/justify actual value);
- local UDP packet max size;
- per-source receive rate/burst if needed to avoid obvious amplification/resource starvation;
- maximum concurrent send operations/queued datagrams.

Use periodic expiry without creating unbounded timer-per-subscriber tasks. One bounded supervisor timer/interval is sufficient.

### 7.5 Fan-out

For each local UDP payload:

- snapshot current subscribers without holding lock across network sends;
- send at most one copy per active unique subscriber key;
- enforce payload limit;
- individual subscriber send failure does not stop producer or block other subscribers;
- persistent repeated failures may be handled by expiry rather than unbounded failure state.

## 8. `streamrclient` / consumer requirements

### 8.1 Runtime topology

```text
periodic subscribe/refresh -> remote Streamr producer over I2P datagram
remote producer payload    -> configured local UDP target
```

### 8.2 Subscribe cadence

Implement bounded refresh cadence compatible with the producer expiry window.

A reasonable adopted behavior may use a short initial retry/refresh cadence followed by a stable interval materially below expiry. Exact values must be constants/documented/testable and not configurable into a busy loop.

On graceful stop, send a best-effort single-byte unsubscribe where possible, but stop must not block indefinitely waiting for it.

### 8.3 Local UDP target

Target host/port comes only from validated definition.

Default should be loopback-safe where applicable. If non-loopback local UDP target is allowed by Proposal 170, it is an explicit administrator action; remote packet contents cannot override it.

Use connected UDP socket or explicit validated destination so received I2P datagrams cannot redirect local output.

### 8.4 Payload handling

- control semantics from producer, if any, distinguished from payload as defined by adopted protocol;
- oversized datagram dropped with bounded diagnostic;
- preserve payload bytes unchanged;
- no retries/reliability layer;
- receiving malformed metadata cannot panic.

## 9. Proposal 170 option mapping

Explicitly disposition at least:

- `TargetDestination` / Streamr remote producer destination;
- `TargetHost`/local UDP host;
- `TargetPort`/local UDP port;
- listen/local source port fields used by producer;
- `StreamrTarget` or equivalent field in the existing domain model;
- destination identity/private-key semantics for server;
- I2CP/datagram session options that Yosemite supports;
- tunnel length/quantity/variance/signature/encryption options where applicable;
- custom options.

Reject recognized relevant unimplemented fields before UDP bind/session allocation.

## 10. Lifecycle and task model

Use M065 generation/cancellation patterns but keep datagram-specific loops explicit.

Server tasks:

- I2P control receive loop;
- local UDP receive loop;
- one expiry interval loop or integrated select branch;
- bounded send futures/tasks.

Client tasks:

- I2P receive loop;
- subscribe refresh interval;
- local UDP send path.

Prefer one `tokio::select!` owner per backend where practical over many detached tasks.

Start reports ready only after:

- required UDP socket bound;
- I2P datagram session established;
- server destination known/published where relevant.

Stop signals loops, optionally emits best-effort unsubscribe for client, drains/aborts boundedly, and releases sockets/session.

Restart preserves server identity but clears ephemeral subscriptions/client cadence state.

## 11. Resource/abuse tests

Required adversarial coverage:

- subscribe flood from more unique peers than cap;
- repeated refresh does not duplicate subscriber;
- unsubscribe removes exact tuple only;
- expiry removes stale subscribers;
- invalid control byte does not create state;
- oversized control/payload dropped;
- high local UDP rate cannot create unbounded send task queue;
- one failing subscriber does not block others;
- no payload sent when zero subscribers;
- remote payload cannot change client local UDP target;
- cancellation with pending UDP receive/send;
- restart clears subscriptions while retaining server destination identity.

## 12. Ordered work packages

### WP1 — Yosemite datagram capability audit + tiny adapter

Confirm APIs and implement only the minimal I2PControl-local session wrapper needed for testability/cancellation.

### WP2 — producer subscription state machine

Implement pure bounded subscription map/control parser/expiry tests before network fan-out.

### WP3 — `streamrserver` runtime/backend

Wire persistent destination, I2P control receive, local UDP source, fan-out, lifecycle, and option validation.

### WP4 — `streamrclient` runtime/backend

Wire remote target, subscribe refresh/unsubscribe, I2P payload receive, local UDP sink, lifecycle, and option validation.

### WP5 — adversarial/e2e tests

Use fake/local datagram fixtures; no public network required.

### WP6 — registry/docs/closure

Promote both types and document bounds/control semantics.

## 13. Failure, cancellation, restart, contention semantics

- invalid control packet: drop, no state;
- local UDP receive error: classify fatal/transient explicitly; fatal socket error marks backend failed;
- one I2P send failure: per-recipient error, does not kill server unless session itself failed;
- client producer unreachable: refresh/send failures remain bounded and runtime state reflects actual session/target behavior without busy retry;
- stop cancels interval/receive/send loops exactly;
- restart server retains destination identity and starts empty subscriber set;
- duplicate start rejected;
- stale generation cannot report running/failed after restart.

## 14. Compatibility and migration

No public/persistence schema migration.

Persisted Streamr definitions become startable only if mapped fields/options fit implemented datagram capability.

No effect on generic TCP tunnel families.

## 15. Verification commands

Minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
git diff --check
```

No external/public I2P network required for local control/fan-out semantics. If a bounded live datagram smoke test is available locally, it is useful but not a new CI requirement.

## 16. Acceptance criteria

M071 may close only when:

1. M065 is closed;
2. exact Yosemite datagram APIs are documented in closure evidence and no core API is added;
3. `streamrserver` owns persistent I2P identity and local UDP source;
4. control message `0` subscribe/refresh and `1` unsubscribe semantics implemented deterministically;
5. malformed/unknown control cannot create state;
6. subscription key uses trusted remote I2P identity plus required port tuple;
7. subscription count is hard bounded;
8. subscriptions expire without refresh;
9. refresh does not duplicate subscribers;
10. fan-out snapshots state without holding lock across sends;
11. one subscriber failure does not block others;
12. local UDP receive/send task queue is bounded;
13. payload size is explicitly bounded;
14. `streamrclient` sends bounded periodic subscribe refresh;
15. graceful client stop attempts bounded unsubscribe without delaying stop indefinitely;
16. client forwards remote payload only to configured local UDP target;
17. remote packet cannot choose local host/port;
18. restart server clears subscribers but retains server destination identity;
19. lifecycle stop/restart/cancellation is exact and generation-safe;
20. relevant unimplemented options reject before UDP/session allocation;
21. both Streamr types replace only their own unsupported backends;
22. no `emissary-core/**` production change;
23. no unjustified non-I2PControl production change;
24. feature-disabled/default and containment checks pass;
25. docs record control protocol, subscriber/expiry/packet bounds, and option matrix;
26. no CI/release/fuzz/coverage/platform expansion;
27. no upstream/third-party write/review/submission/contribution preparation;
28. no high/medium unbounded-state, amplification, target-redirection, task-leak, or identity finding remains.

## 17. Closure evidence required

`071-closure.md` must include:

- implementation commits/paths;
- Yosemite datagram API mapping;
- subscription state-machine matrix;
- explicit bounds/cadence values;
- flood/expiry/failure evidence;
- producer/client e2e local UDP evidence;
- restart identity/subscriber-reset evidence;
- option-capability matrix;
- registry/docs;
- containment/default-build results;
- security/resource review;
- internal-only attestation;
- disposition.

## 18. Stop conditions

Stop/replan if:

- Yosemite lacks required peer identity/from-to port datagram metadata;
- correct semantics require a new router/core datagram API;
- subscription state cannot be bounded without breaking adopted protocol;
- implementation pressure creates generic UDP tunnel scope;
- a relevant security/resource option would have to be silently ignored.