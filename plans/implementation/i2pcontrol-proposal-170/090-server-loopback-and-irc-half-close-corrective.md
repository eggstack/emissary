# M090 — Server Loopback and IRC Half-Close Corrective

Status: closed — implementation complete; see `plans/closure/i2pcontrol-proposal-170/090-closure.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Corrective authority and predecessors:

- M076 HTTP server anonymity/POST hardening;
- M077 IRC server lifetime/exhaustion hardening;
- M087 generic-server inactivity/half-close corrective;
- M089 current-head tunnel runtime/security reclosure and `plans/closure/i2pcontrol-proposal-170/089-closure.md`.

Planning baseline: `f0f3fc2204318c2fac69817d347df2702c51287b`.

Classification: corrective security hardening / local-boundary and relay semantics.

## 1. Objective

Correct two narrow post-M089 server-family defects without reopening unrelated Proposal 170 scope:

1. make `httpserver`, inbound `httpbidirserver`, and `ircserver` local-target confinement resolver-independent by normalizing an accepted loopback target to a literal loopback `IpAddr` before runtime connection;
2. make `ircserver` preserve useful TCP half-close semantics while retaining its existing ten-minute progress-based inactivity bound, matching the already-corrected generic `server` relay behavior.

This milestone is intentionally small. It is not a general server-runtime refactor and it does not alter the Proposal 170 API contract.

## 2. Why a corrective pass is required

M089 correctly established that the server family rejects arbitrary non-loopback targets and that IRC has bounded registration, target-connect, and post-registration inactivity behavior. A subsequent source review found two details that the M089 acceptance criteria did not test precisely enough.

### 2.1 Resolver-dependent `localhost`

Current HTTP and IRC configuration accepts the literal string `localhost` as a loopback target, then later passes that hostname to `TcpStream::connect`. Under ordinary host configuration this resolves to loopback, but the security invariant is stronger: a remote I2P stream must never cause these server backends to select a non-loopback local target. That invariant should not depend on NSS, `/etc/hosts`, DNS, resolver hooks, or local hostname configuration.

The generic `server` backend already avoids this ambiguity by connecting directly to `127.0.0.1`.

### 2.2 IRC half-close asymmetry

Current `ircserver::relay_with_inactivity()` returns when either directional relay future completes. Therefore a normal EOF/half-close on one side terminates the other direction immediately even when useful bytes remain to drain.

M087 corrected this exact semantic for generic `server`: one completed direction becomes inactive, its opposite writer is shut down, and the remaining direction may continue until EOF/error or the shared inactivity deadline. IRC should use the same bounded half-close property without changing its post-registration bytes or IRC parsing semantics.

### 2.3 Why prior verification missed these details

M089 verified that configured target strings were restricted to loopback spellings but did not assert that accepted values are converted to resolver-free socket addresses before connect. Its IRC checks verified registration bounds, five-second target connect, and ten-minute progress-resetting inactivity, but did not include an EOF/half-close drain regression test.

M090 must add tests that would have caught both defects.

## 3. Scope

Primary production scope:

- `emissary-cli/src/i2pcontrol/backends/http_server.rs`;
- `emissary-cli/src/i2pcontrol/backends/irc_server.rs`.

Conditionally allowed, only if required by the existing shared HTTP handler/config seam:

- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs`;
- a small colocated helper under `emissary-cli/src/i2pcontrol/backends/**` if it demonstrably avoids duplicated loopback-normalization logic without creating a generalized networking abstraction.

Planning/test bookkeeping:

- this plan and its future closure record;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- the exact M062 planning-path guard update required to authorize the new plan/closure records.

No production path outside `emissary-cli/src/i2pcontrol/**` is authorized by M090.

## 4. Security and compatibility invariants

M090 MUST preserve:

- exact Proposal 170 tunnel types, fields, actions, option spelling, and status behavior;
- existing accepted-stream trusted peer identity and `ServerAdmissionState` ordering;
- current server Destination persistence/secret ownership;
- loopback-only local service forwarding;
- five-second HTTP/IRC target-connect timeout;
- bounded HTTP request/header/body/POST behavior;
- bounded IRC registration and trusted peer-derived hostname rewriting;
- ten-minute IRC inactivity deadline reset only by successful byte-transfer progress;
- raw IRC bytes after registration;
- generation-local task/admission ownership and bounded stop semantics;
- current `httpbidirserver` separate unpublished client-session identity decision;
- current Streamr behavior;
- no private Destination/key material in diagnostics;
- internal-only repository interaction.

## 5. Explicit non-goals

M090 MUST NOT:

- implement or change lower-layer/pre-accept stream admission; that is M091 territory;
- add arbitrary LAN/clearnet HTTP or IRC targets;
- add DNS resolution, hostname fallback, interface discovery, or service discovery;
- redesign HTTP parsing, POST throttling, response filtering, or body limits;
- redesign IRC registration or add new IRC commands/features;
- impose a maximum absolute lifetime on active IRC connections;
- change Streamr subscriber policy;
- change `httpbidirserver` to share the public server Destination with its local proxy;
- touch `emissary-core/**`, router algorithms, startup ownership, frontend state, Cargo dependencies, or `Cargo.lock`;
- add hosted CI, fuzz, soak, or public-network deanonymization/load testing;
- initiate or prepare upstream review, submission, merge, issue/PR activity, or maintainer contact.

## 6. Required production changes

### 6.1 Resolver-free local target representation

HTTP and IRC runtime configuration MUST hold or derive a literal `IpAddr` before an accepted remote stream can reach `TcpStream::connect`.

Accepted compatibility spellings remain:

- `127.0.0.1` -> IPv4 loopback;
- `::1` where the backend currently supports it -> IPv6 loopback;
- `localhost` -> normalize directly to a chosen literal loopback address, preferably `127.0.0.1` for compatibility with the existing default.

No accepted hostname may be passed to a resolver during target connection.

Implementation preference:

- use a tiny parser/normalizer that returns `IpAddr` and rejects every non-loopback value;
- carry that typed address through the config/handler seam where practical;
- if changing the shared HTTP handler type requires the composed `httpbidirserver` config to carry `IpAddr`, make only that mechanical in-`i2pcontrol` adjustment.

Do not broaden accepted address forms merely because `IpAddr::is_loopback()` could recognize additional loopback addresses. Preserve the existing external option contract unless a current test/spec already establishes broader acceptance.

### 6.2 IRC half-close-preserving progress relay

Bring IRC relay termination semantics into line with M087 generic `server`:

- each relay direction loops on bounded buffer reads and `write_all`;
- successful transferred bytes increment the shared activity sequence;
- EOF shuts down the opposite writer and marks only that direction complete;
- completion of one direction does not immediately cancel the other;
- the remaining direction may drain until it also completes/errors;
- the shared inactivity timer remains ten minutes and is reset only by successful byte-transfer progress from either active direction;
- if neither direction remains active, return success;
- error propagation remains bounded and releases the accepted-server admission lease through handler completion.

Prefer a local structural copy of the already-reviewed generic relay pattern or a very small shared `i2pcontrol` helper. Do not create a broad generic transport framework.

## 7. Failure, cancellation, restart, and contention semantics

- Invalid or non-loopback target options fail before destination/session/runtime allocation, as today.
- A failed local loopback connect remains bounded by the existing five-second connect timeout and releases the handler/admission lease.
- Resolver failure is no longer a possible runtime state for these target values because no resolver is used.
- IRC EOF on one side half-closes the corresponding opposite writer while the remaining direction stays live.
- IRC write/read errors end the handler and release admission state.
- IRC zero-progress sessions still expire after the existing ten-minute inactivity interval.
- Cancellation and stop continue through the accepted-server task group; M090 must not add detached tasks.
- No mutex may be held across socket I/O, sleep, shutdown wait, or task join.

## 8. Ordered work packages

### A. Pin and inspect the M089 baseline

Before editing, inspect the current `http_server.rs`, `http_bidir.rs`, `irc_server.rs`, M087 generic relay, and existing tests. Confirm no intervening runtime change already corrects either defect.

### B. Add typed resolver-free loopback normalization

Implement the smallest loopback normalizer and thread it through HTTP/IRC configuration. Add focused option/config tests proving:

- default target is a literal loopback address;
- `localhost` normalizes without remaining a hostname;
- supported IPv4/IPv6 loopback literals remain accepted as applicable;
- a representative non-loopback address fails before runtime/session/destination allocation;
- public/persisted server Destination material never selects the local target.

### C. Correct IRC relay half-close behavior

Adapt the generic-server M087 relay state machine to IRC while retaining `POST_REGISTRATION_INACTIVITY` and existing buffer size. Add deterministic paused-time/duplex tests for half-close in both directions.

### D. Containment and documentation

Update only the planning/control surfaces needed for M090. Do not update broad docs merely to restate unchanged behavior.

## 9. Focused regression tests

At minimum add/retain tests showing:

1. HTTP `localhost` configuration results in a literal loopback socket target before connect;
2. HTTP non-loopback target still fails closed;
3. IRC `localhost` configuration results in a literal loopback socket target before connect;
4. IRC non-loopback target still fails closed;
5. remote->local IRC EOF allows local->remote bytes to drain before final completion;
6. local->remote IRC EOF allows remote->local bytes to drain before final completion;
7. successful progress in the remaining direction resets the existing inactivity deadline;
8. zero-progress remaining direction expires at `POST_REGISTRATION_INACTIVITY`;
9. admission ownership is released after half-close completion/expiry;
10. no private Destination text appears in new debug/error paths.

Use deterministic `tokio::io::duplex` and paused-time tests where possible. No public I2P traffic is required.

## 10. Broad verification

Run at minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Also inspect:

```text
git diff --name-only f0f3fc2204318c2fac69817d347df2702c51287b
git diff --stat f0f3fc2204318c2fac69817d347df2702c51287b
```

The changed-path list must contain no unexplained production file outside `emissary-cli/src/i2pcontrol/**`.

## 11. Documentation and static guards

- Update the M062 exact planning-path allowlist for the M090 plan/closure pair and the separately created M091 plan/closure pair; do not broaden it to a directory glob.
- Registry/roadmap/README changes must describe M090 as the only dependency-ready tunnel-security implementation handoff.
- Do not rewrite M089 closure as though it had originally included these later regression tests; preserve historical pinned-head evidence.

## 12. Acceptance criteria

M090 may move to closure only when:

1. HTTP and IRC accepted target values are converted to literal loopback addresses before runtime connect;
2. no `localhost` hostname reaches `TcpStream::connect` in the corrected server paths;
3. non-loopback target rejection remains fail-before-allocation;
4. IRC preserves two-direction drain across a one-sided EOF;
5. IRC ten-minute progress-based inactivity behavior remains unchanged;
6. accepted-server admission/peer identity ordering is unchanged;
7. HTTP parsing/filtering/POST behavior is unchanged apart from target representation;
8. `httpbidirserver` identity/session semantics are unchanged;
9. no production path outside `emissary-cli/src/i2pcontrol/**` changed;
10. full i2pcontrol tests, M062, and `git diff --check` pass;
11. no upstream interaction occurred.

## 13. Stop conditions

Stop and create a new plan rather than widening M090 if:

- resolver-free confinement requires changing generic router networking behavior;
- a broad shared socket-address abstraction outside `i2pcontrol` is proposed;
- IRC half-close correction requires protocol parsing changes;
- a dependency or lockfile change is proposed;
- lower-layer/pre-accept admission work is encountered;
- Streamr or bidirectional identity semantics would change;
- a new Proposal 170 field/type/action is proposed.

## 14. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/090-closure.md` containing:

- planning baseline and implementation head;
- exact files changed;
- requirement-to-evidence matrix;
- loopback normalization test evidence;
- IRC half-close/inactivity test evidence;
- failure/cancellation/contention review;
- source-containment review;
- full verification commands and outcomes;
- unresolved findings with severity;
- explicit internal-only/no-upstream-interaction attestation;
- disposition for M091 readiness/blocker status after M090.
