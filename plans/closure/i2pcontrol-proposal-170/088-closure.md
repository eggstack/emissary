# M088 — Pre-Accept Server Admission Boundary Corrective Closure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Corrective predecessors:

- M074/M083 application admission hardening;
- M085/M087 closure records;
- M087 generic-server inactivity corrective.

Planning baseline reviewed: `2b01bfd11ebcd768fcd5488f18b063ac336931a2`.

Final reviewed M088 implementation head: `9d4b2b1dd40765e2314f364721c3c9934d278c3f`.

Implementation commits or pull requests:

- `9d4b2b1dd40765e2314f364721c3c9934d278c3f` — prior internal M087 implementation and planning head reviewed as M088's starting implementation head; M088 itself made no production change.
- No external pull request or upstream artifact exists or was prepared.

## 1. Executive finding

M088 is closed as an evidence-only Tier 3 disposition. The earliest
enforceable accepted-stream limit in the current Emissary boundary is the
post-`Session<style::Stream>::accept()` application admission check in
`emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs`. The current
Yosemite dependency can serialize the session's I2CP options and the SAM
`STREAM ACCEPT` command, but it exposes no lower-layer streaming admission
configuration. Emissary's SAM/streaming implementation has no consuming
algorithm for the candidate connection limits either.

The similarly named fields in
`emissary-core/src/sam/protocol/streaming/config.rs` are declarations/defaults
only: source search shows no consumer for those fields, and every accepted
stream is created with `StreamConfig::default()`. Passing Java option names
through the existing SAM option map would therefore be persist-and-ignore, not
pre-accept enforcement. M088 correctly adds no production option plumbing,
dependency, fork, router change, or Proposal 170 API field.

This leaves a real residual risk: an attacker can cause Emissary's lower
streaming layer to parse and authenticate signed SYN packets, allocate pending
or active stream-manager state, bind routing paths, and perform local SAM
socket/session work before the application can reject a stream. Existing
application admission still bounds handler tasks, peer history, aggregate
rates, and local-target work after `accept()`; it does not bound that earlier
lower-layer work. This is a known resource-exhaustion and timing/load
correlation limitation, not a claim of pre-accept protection.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Earliest inbound boundary is source-mapped | `accepted_server.rs:82-145`; Emissary SAM/core flow mapped in §3 | pass | Application admission follows Yosemite `accept()`. |
| Yosemite support is established from code | Yosemite 0.7.0 source, checksum-pinned in `Cargo.lock`, `SessionOptions`, `Session::accept()`, and `SessionController::accept_stream()` | pass | Session options contain no streaming admission fields; `STREAM ACCEPT` carries only `ID`/`SILENT`. |
| Emissary candidate support is established from code | `emissary-core/src/sam/protocol/streaming/config.rs`; streaming-manager construction/use; SAM accept handling | pass | Candidate fields are unused configuration declarations, not an implemented limiter. |
| Java reference behavior was compared | Java I2P and I2P+ `ConnectionManager`, `ConnThrottler`, and `ConnectionOptions` at pinned revisions in §4 | pass | Reference rejection occurs on validated inbound SYN processing. |
| I2P+/i2pd equivalent was compared | i2pd configuration/reference documentation and pinned branch in §4 | pass | Its controls are streaming-layer options, not SAM `STREAM ACCEPT` options. |
| Existing application admission remains active | `accepted_server.rs:107-135`; `admission.rs` policy/state implementation and tests | pass | Identity, peer/global concurrency, rates, and bounded handler task ownership remain unchanged. |
| No unsupported option is persisted and ignored | No production change; no candidate Java option is emitted | pass | Truthfulness rule is preserved. |
| No Proposal 170 API surface or identity semantics changed | Changed-path review; no production paths changed | pass | No JSON-RPC fields, actions, types, or clearnet identity were added. |
| No broad dependency/core/router refactor | Changed-path review and M062 containment | pass | Yosemite remains crates.io `0.7.0`; no lockfile or core production change. |
| M089 can proceed | M087 closed and this closure accepts Tier 3 as out of scope | pass | M089 is advanced to `ready`; no separate dependency-boundary plan is required. |

## 3. Full inbound boundary/resource/limit map

The following is one inbound accepted stream through the current implementation.

| Layer | Resource allocated or retained | Remote identity known? | Limit and scope | Before/after stream establishment | Rejection/reset/drop behavior | Attacker-distinguishable behavior |
|---|---|---|---|---|---|---|
| Remote I2P peer / streaming SYN | Router-delivered I2NP payload; parsed `Packet`; signature/destination/replay-validation work | Yes after signed SYN parsing; the source is the authenticated Destination in the packet | No M088 global, per-peer, or rate limiter in Emissary streaming manager | During lower-layer stream establishment | Malformed/signature/replay failure returns streaming error; valid SYN with no listener enters pending state and receives the existing pending-stream response/retry behavior | Protocol-required reset/retry/drop and normal timing; no custom M088 response |
| Emissary streaming manager | `pending_inbound`, destination-to-stream indexes, active stream map/channel, stream task, and routing-path binding | Yes for a valid SYN | Existing pending-stream expiry/pruning and lifecycle behavior; no candidate admission bound is consumed | Before Yosemite returns from `accept()` | Pending entries expire through existing stream lifecycle; active stream closes through existing stream task/drop paths | Normal streaming ACK/SYN-ACK/close behavior |
| Emissary SAM TCP listener | Local TCP socket and bounded pending SAM connection task | No | SAM listener/pending-connection keep-alive is 10 seconds; no streaming admission limit | Before stream establishment | Local SAM protocol/socket failure closes the connection | Normal local SAM failure |
| Yosemite `Session::accept()` | A new local TCP connection to SAM, stream handshake state, and `STREAM ACCEPT` request/response exchange | Not until the SAM response includes the peer Destination | Yosemite exposes no lower-layer stream limit or rate control | Spans establishment; the returned `Stream` already represents the accepted stream | I/O/protocol error returns an accept error; no admission policy is consulted | Normal SAM/Yosemite error or connection close |
| Emissary SAM `STREAM ACCEPT` handling | Parsed option map and an ephemeral `StreamListener` socket | Not from the command; remote identity arrives later in the streaming SYN | Option map is accepted, but `on_stream_accept` consumes only stream support and `SILENT`; no candidate limiter | Before remote stream is attached | Unsupported session kind/listener mismatch drops/rejects the SAM socket | Existing SAM status/rejection behavior |
| Trusted peer boundary | `TrustedPeerIdentity` parsed from Yosemite's authenticated remote Destination; canonical identity text and 32-byte key | Yes | Text bound, exact Destination parse/remainder check, and canonicalization | After stream establishment and immediately after `accept()` | Malformed/trailing identity drops the accepted stream before handler admission | No peer/private-Destination material is emitted |
| `ServerAdmissionState` | Bounded peer record/expiry state and an admission lease | Yes | Per-peer, aggregate, rate-window, and global handler concurrency policy; all are application-level | After `accept()` and after the lower-layer stream exists | Denied stream is dropped; no handler/local target is started; lease drop releases state | Existing fail-closed drop, without custom jitter |
| Accepted handler/local target | Bounded task group, protocol parser/relay, and only then loopback target connection where applicable | Yes | Handler task/global bound plus protocol-specific bounds already established by M074-M087 | After application admission | Panic/cancellation/error releases lease and task resources; target connection ordering remains protocol-owned | Existing protocol failure/reset behavior |

The common path is shared by generic `server`, `httpserver`, inbound
`httpbidirserver`, and `ircserver` through `run_accepted_server()`. Streamr is
not on this path and is unchanged.

## 4. Pinned external and dependency evidence

Evidence was read-only and collected on 2026-08-25.

### Yosemite

- Dependency declaration: workspace `yosemite = { version = "0.7.0", features = ["async-extra"] }`.
- Lockfile source: crates.io registry, version `0.7.0`, checksum `c6bf3692263d7a9258016f5468c5cf5301b06189d7bc4c97b014b69022659871`.
- Source package: `https://github.com/eepnet/yosemite/releases/tag/v0.7.0` (tag commit `d0fe71da214b212790773be12a93162ae71f3e03`; the registry package is the lockfile authority).
- `src/options.rs`: `SessionOptions` contains I2CP/session and tunnel-pool controls only; `StreamOptions` contains source/destination ports only.
- `src/proto/session.rs`: `create_session()` serializes I2CP options; `accept_stream()` emits `STREAM ACCEPT ID=... SILENT=false`.
- `src/asynchronous/session/mod.rs:334-356`: `Session::accept()` opens a local SAM TCP connection, sends the accept command, waits for SAM status, then reads the remote Destination before returning `Stream`.

### Java I2P

- Read-only source revision: `fda1ced99c3b1e8513b88c543bca3aeb668330a8` (`i2p/i2p.i2p` master, retrieved 2026-08-25).
- `apps/streaming/java/src/net/i2p/client/streaming/impl/ConnectionOptions.java` defines `i2p.streaming.maxConcurrentStreams`, per-peer minute/hour/day limits, aggregate minute/hour/day limits, and `limitAction`.
- `ConnectionManager.java` owns the `IncomingConnectionFilter`, `ConnThrottler` instances, and the inbound SYN decision. Its `receiveNewConnection()` path checks the concurrent-stream ceiling and `shouldRejectConnection()` before constructing `ConnectionOptions`/`Connection` state; rejection sends the configured reset/drop/HTTP/custom response.
- `ConnThrottler.java` counts per-peer and total arrivals in fixed periods and explicitly describes the controls as DOS protection, not a complete solution.
- Reference links: [Java I2P repository](https://github.com/i2p/i2p.i2p), [streaming option specification](https://www.i2p.net/en/docs/api/streaming/).

### I2P+

- Read-only source revision: `0c47b7ea1369d661ea08f7109d153c7df51e5c52` (`i2p.plus/I2P.Plus` master, retrieved 2026-08-25).
- Its fork retains the same `ConnectionOptions`, `ConnThrottler`, `IncomingConnectionFilter`, and `ConnectionManager` placement and option matrix. It therefore confirms the semantic is a streaming-manager algorithm, not a SAM session option.
- Reference documentation: [I2P+](https://i2pplus.github.io/) and [I2P+ TunnelController API](https://i2pplus.github.io/javadoc/net/i2p/i2ptunnel/TunnelController.html).

### i2pd / C++ equivalent

- Read-only branch heads retrieved 2026-08-25: `PurpleI2P/i2pd` `openssl` `3f413d322dc97481468b9840842674cec5b1dfe4`; `master` `a1c7e608116c23bfd513f85ff297ab763b5cbd6`.
- Its documentation describes `i2p.streaming.maxConcurrentStreams` and `i2p.streaming.maxConnsPerMinute` as streaming tunnel controls, reinforcing that textual parity does not imply SAM/Yosemite support.
- Reference: [i2pd tunnel configuration](https://docs.i2pd.website/en/latest/user-guide/tunnels/).

## 5. Candidate option support matrix and disposition

| Candidate | Java/I2P+ meaning | Yosemite 0.7.0 | Emissary consumer | M088 disposition |
|---|---|---|---|---|
| `i2p.streaming.maxConcurrentStreams` | Total concurrent inbound/outbound streams | No `SessionOptions`/`StreamOptions` field or serializer | `StreamConfig::max_concurrent_streams` is defaulted but never consumed; no enforcement in `StreamManager`/`Stream` | Unsupported; do not emit |
| `i2p.streaming.maxConnsPerMinute` | Per-peer incoming connection rate | Not exposed | `StreamConfig::max_conns_per_minute` is declaration/default only | Unsupported; do not emit |
| `i2p.streaming.maxConnsPerHour` | Per-peer incoming connection rate | Not exposed | Declaration/default only | Unsupported; do not emit |
| `i2p.streaming.maxConnsPerDay` | Per-peer incoming connection rate | Not exposed | Declaration/default only | Unsupported; do not emit |
| `i2p.streaming.maxTotalConnsPerMinute` | Aggregate incoming connection rate | Not exposed | Declaration/default only | Unsupported; do not emit |
| `i2p.streaming.maxTotalConnsPerHour` | Aggregate incoming connection rate | Not exposed | Declaration/default only | Unsupported; do not emit |
| `i2p.streaming.maxTotalConnsPerDay` | Aggregate incoming connection rate | Not exposed | Declaration/default only | Unsupported; do not emit |
| `i2p.streaming.limitAction` | Reset/drop/HTTP/custom rejection response | Not exposed | `LimitAction` declaration is unused | Unsupported; do not emit |

Selected branch: **Tier 3 — unsupported without broad router/core/streaming
changes**. Tier 1 fails because Yosemite has no expressible option. Tier 2
fails because Emissary does not already implement the receiving semantic; a
minimal plumbing change cannot expose behavior that does not exist. Implementing
it would require a new lower-layer streaming admission algorithm and likely
dependency/API boundary work, which M088 explicitly forbids.

## 6. Production implementation evidence

No production code changed for M088. In particular, the following were
intentionally not changed:

- `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs` — retains
  `session.accept()`, trusted identity validation, `ServerAdmissionState`, and
  bounded handler spawning in that order;
- `emissary-cli/src/i2pcontrol/backends/runtime/admission.rs` — remains the
  application admission authority;
- `emissary-core/**`, Yosemite dependency declarations, `Cargo.lock`, and the
  Proposal 170 JSON-RPC/API surface.

The generic, HTTP, inbound HTTP-bidirectional, and IRC families continue to use
the common accepted-server runtime. No Streamr path was touched.

## 7. Verification executed

### Commands run

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
git diff --name-only 2b01bfd11ebcd768fcd5488f18b063ac336931a2
git diff --stat 2b01bfd11ebcd768fcd5488f18b063ac336931a2
```

### Results

| Command | Outcome |
|---|---|
| M062 dependency containment | pass; exact dependency/feature/lockfile/source-boundary guards green |
| `git diff --check` | pass |
| changed-path review | pass; M088 adds closure/status bookkeeping only; no production path or dependency changed |
| focused production tests | not run for M088; no production code changed, and the plan's evidence-only branch requires source/revision evidence plus M062 and diff-check |

The existing accepted-server/admission tests remain applicable evidence for the
post-accept defense-in-depth path and were not invalidated or modified by this
milestone. M087's focused tests and closure remain the authority for the
generic inactivity correction.

## 8. Invariant, failure, recovery, and contention review

- `ServerAdmissionState` remains mandatory and authoritative for application
  handler/local-target work. Its peer identity remains the authenticated I2P
  Destination-derived key, never IP/clearnet identity or attacker-controlled
  application data.
- Admission denial remains after stream establishment but before handler and
  local-target allocation. No lower-layer counter was added under a misleading
  name.
- Session cancellation/restart remains generation-local in the accepted-server
  runtime; no new limiter state exists that could cross generations.
- SAM/Yosemite I/O is not held under the application admission mutex. Existing
  bounded task drain and lease-drop behavior remains unchanged.
- Lower-layer malformed packets, failed authentication/replay checks, missing
  listeners, socket errors, stream errors, cancellation, and task drops retain
  their existing reset/drop/expiry behavior. M088 does not introduce custom
  delay, jitter, padding, or public-network test machinery.
- Private server Destination material and full peer Destination text remain
  absent from M088 diagnostics and errors; no option/error logging was added.

## 9. Migration, compatibility, security, and operations

No schema, configuration persistence, dependency, default-feature, wire/API,
router, startup, or frontend migration occurred. Existing accepted-server
configuration remains truthful: the Proposal 170 admission options describe
application admission only, and no hidden lower-layer values are persisted.

The residual risk is explicit and material. Before application admission,
remote attempts can consume streaming-manager CPU, packet-processing work,
pending/active map entries, routing-path state, stream task/channel state, and
local SAM socket/session work. Per-Destination application limits cannot provide
Sybil resistance at that earlier boundary. M087 bounds zero-progress generic
handler occupancy after admission, but it does not remove this pre-accept
amplification window. Closing it requires implementing and owning the streaming
algorithm at the Emissary lower layer (or a separately approved dependency
boundary), which is outside M088's containment budget.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| medium | No Emissary lower-layer pre-accept concurrent/rate admission equivalent to Java I2P/I2P+ exists at the current Yosemite/core boundary. | Lower-layer stream-establishment CPU, state, routing, and SAM work remains attacker-influenceable before application rejection; timing/load correlation remains possible. | Treat as an accepted residual limitation in M089. Any implementation requires a new narrowly approved dependency-boundary or core/streaming plan. |

This is not a M088 closure blocker because the plan explicitly provides the
Tier 3 unsupported-capability branch and forbids broad implementation work.
There are no unclosed high-severity findings within M088's authorized scope.

## 11. Roadmap disposition

M088 is closed with a precise unsupported-capability finding. The application
post-accept admission boundary remains the earliest enforceable in-scope bound.
M089 is unblocked and ready as an independent verification-only reclosure. It
must record this limitation rather than claim pre-accept enforcement. No
separate Yosemite dependency-boundary plan is opened because the lower-layer
limitation is explicitly accepted as out of scope for M089.

## 12. Registry updates

Updated in the same planning transition:

- M088 implementation plan status: `closed` with Tier 3 disposition;
- M089 implementation plan status: `ready`;
- `plans/registry.md`: M088 closed, M089 sole ready tunnel-security handoff;
- tunnel-security roadmap: M088 closed and M089 ready;
- this closure record added at `plans/closure/i2pcontrol-proposal-170/088-closure.md`.

## 13. Internal-only boundary

All external specifications and repositories were accessed read-only for
behavioral/source evidence. No upstream repository, issue, pull request,
review, merge request, discussion, maintainer channel, submission,
contribution artifact, or external write was opened, drafted, mutated, or
requested. Repository writes remain internal to `eggstack/emissary`.
