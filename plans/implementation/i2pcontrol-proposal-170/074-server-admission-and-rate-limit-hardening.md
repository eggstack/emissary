# M074 — Shared Server Admission and Rate-Limit Hardening

Status: blocked — prewritten corrective successor; hard dependency M073 must close first

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Corrective predecessors:

- M072 closure: `plans/closure/i2pcontrol-proposal-170/072-closure.md`;
- M073: `plans/implementation/i2pcontrol-proposal-170/073-generic-tunnel-option-truthfulness-corrective.md`.

Planning production baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Read-only reference evidence:

- Java I2P `TunnelController` server connection defaults at `i2p/i2p.i2p@498488b0`;
- Java streaming `ConnectionManager` peer/aggregate minute/hour/day throttlers and max streams;
- Java `I2PTunnelServer` bounded handler rejection behavior;
- Proposal 170 server option list, revision `2026-05-20`.

## 1. Objective

Add one I2PControl-owned, peer-aware admission boundary for accepted-stream server tunnels and integrate it into `httpserver`, `ircserver`, and the inbound server half of `httpbidirserver`.

The correction must prevent one authenticated remote I2P identity from consuming the whole global handler pool, add the supported Proposal 170 peer/aggregate connection-rate controls, and keep all limiter/task state hard bounded without introducing artificial timing sleeps.

## 2. Confirmed defect and threat model

At the baseline, `run_accepted_server` accepts a Yosemite stream, obtains a trusted remote destination, then attempts to spawn the handler into a semaphore-bounded task group. The global task limit prevents unbounded task creation, but there is no per-peer concurrent occupancy or peer/aggregate connection-rate state.

An attacker can therefore:

1. open enough valid streams to consume all global permits;
2. hold them near parser/application timeouts or indefinitely for protocols without an idle timeout;
3. probe the destination from another identity and observe the transition between normal handling and immediate capacity rejection;
4. repeat the induced-load pattern while testing a candidate host/service outside I2P.

This is an availability problem and a useful timing/load-correlation primitive. The goal is to make global saturation expensive across multiple identities rather than controllable by one peer.

## 3. Classification

Primary class: invariant/security infrastructure with directly consumed capability behavior.

Affected production families:

- `httpserver`;
- `ircserver`;
- inbound half of `httpbidirserver`.

Generic `server` integration is M075, not this plan.

## 4. Hard invariants

- all new production logic stays under `emissary-cli/src/i2pcontrol/**`;
- no `emissary-core/**` production change;
- remote identity key comes only from `TrustedPeerIdentity`/SAM/Yosemite;
- admission is evaluated before protocol-handler spawn and before local target connection;
- no lock is held across network I/O, sleeps, task execution, or joins;
- global and per-peer active counts are released on normal completion, parser error, panic isolation, cancellation, task abort, and shutdown;
- peer/rate state has an explicit memory bound;
- when peer-state capacity is full, an untrusted new identity is rejected rather than evicting still-live abuse/accounting state;
- counters use monotonic time;
- no private destination key material or full raw option values in errors/logs;
- no random/fixed rejection delay;
- option validation occurs before session/task allocation;
- existing HTTP/IRC sanitizers remain protocol owners; admission code does not parse application bytes.

## 5. Required production design

### 5.1 Shared admission types

Add a small backend-local component such as:

```text
ServerAdmissionPolicy
ServerAdmissionState
PeerKey
AdmissionLease
AdmissionDecision
```

Exact names are non-normative.

`PeerKey` SHOULD be a fixed-size digest/hash derived from the trusted public destination for accounting so an attacker cannot multiply memory consumption through long textual destination representations. The original trusted destination remains available to protocol handlers that genuinely require it.

`AdmissionLease` SHOULD use RAII/drop semantics or an equivalent exact lifetime mechanism so active counters cannot leak when a handler returns early or panics.

### 5.2 Global concurrency

`MaxConcurrentConns` controls the per-tunnel global active accepted-stream ceiling.

Adopt a reference-scale default of 30 when the operator has not supplied the field. Preserve the existing hard upper ceiling of 128 unless current compatibility evidence requires a lower ceiling; do not increase it in this milestone.

The task-group semaphore and admission state must describe the same global capacity. Do not keep two independently configurable limits that can diverge.

### 5.3 Per-peer concurrent fairness

Add an internal finite per-peer active-stream ceiling because rate limits alone do not prevent one peer from opening the entire global pool and holding it.

Default target: no more than 8 simultaneous accepted streams from one peer, further limited by a smaller global `MaxConcurrentConns` value.

This is an internal hardening constant, not a new Proposal 170 field. If implementation evidence demonstrates that 8 is incompatible with ordinary browser/IRC behavior, record the deviation and choose another finite conservative value; do not remove the per-peer bound or add a wire extension opportunistically.

### 5.4 Connection-rate windows

Implement the directly evidenced Proposal 170/Java-equivalent controls:

- `ClientPerMinute`;
- `ClientPerHour`;
- `ClientPerDay`;
- `TotalInPerMinute`;
- `TotalInPerHour`;
- `TotalInPerDay`.

When absent, use Java-reference server defaults unless M073/persisted compatibility demonstrates the operator explicitly selected unlimited behavior:

- peer/minute: 30;
- peer/hour: 80;
- peer/day: 200;
- total/minute: 50;
- total/hour: 0/unlimited;
- total/day: 0/unlimited.

Use fixed/rolling buckets with O(1) or amortized O(1) admission checks. Do not keep an unbounded timestamp vector per peer. The exact bucket algorithm is an implementation detail, but its edge semantics must be deterministic and testable with paused time.

### 5.5 State capacity

Define and document a per-tunnel peer-accounting memory budget. Prefer a fixed-size peer key and compact counters. A practical entry ceiling may be selected up to a few thousand entries, but closure evidence must state the worst-case memory bound.

When the table is full:

- retain peers with active connections or unexpired rate state;
- reject previously unseen peers until expiry frees capacity;
- never evict an active/throttled peer merely because a new identity arrives.

Expired inactive peers may be removed lazily/amortized without an O(n) full-map scan on every connection.

### 5.6 Underspecified fields

Re-check current Proposal 170 and reference configuration semantics for:

- `PerClientPeriod`;
- `TotalPeriod`;
- `TotalBanTime`.

If exact units, precedence, and interactions are authoritative, implement them in the same admission policy. If they remain only names without sufficient semantics, reject them before allocation and document the unsupported status. Do not infer units or invent a ban algorithm.

`FilterFilePath`, `UniqueLocalAddressPerClient`, and `MultiHoming` are not implemented by this plan. They remain apply-or-reject according to M073/current backend capability; no filter-file language, local-address allocator, or multihoming framework is authorized.

## 6. Accepted-server integration

Refactor `run_accepted_server` only enough to make admission an explicit dependency of the accepted server runtime.

Required ordering:

```text
session.accept()
  -> validate trusted peer identity
  -> admission.try_acquire(peer)
      denied: drop/reset/close stream; no handler task
      allowed: obtain AdmissionLease
  -> spawn bounded handler task carrying lease
  -> handler completion/drop releases lease
```

The exact close/reset operation must use the safe Yosemite capability available to this repository. Do not add a core API merely to produce a special overload response.

HTTP may later send protocol-level 429/503 after protocol parsing where appropriate, but the common admission layer must not require reading application bytes to make a connection admission decision.

## 7. Timing behavior

Do not add jitter, sleeps, probabilistic rejection, or deliberate response padding.

Overload rejection should be prompt and bounded. The anonymity improvement comes from peer fairness/rate controls preventing one attacker from reliably reaching the global rejection regime, not from trying to hide overload with timing theater.

No local target error, internal counter value, private destination, or raw peer destination should appear in an overload error exposed through I2PControl.

## 8. Lifecycle/failure semantics

- admission state is ephemeral and per running tunnel generation;
- stop/restart clears active/rate state after the exact generation is drained/aborted;
- a stopped generation cannot release counters into a new generation;
- handler panic still releases its lease;
- cancellation while waiting for/handling an accepted stream leaves no active count;
- a malformed peer identity is rejected before admission state insertion;
- state allocation failure fails closed for that connection, not the whole service;
- start validation failure occurs before persistent session allocation where option validation permits.

## 9. Ordered work packages

### WP1 — Policy/value mapping

Map supported Proposal 170 fields to a typed admission policy, apply safe defaults, and fail closed on invalid ranges/underspecified options.

### WP2 — Bounded admission state

Implement fixed-size peer keying, active counts, minute/hour/day counters, expiry, capacity behavior, and lease release semantics.

### WP3 — Accepted-server seam

Inject admission into the existing accepted-server runtime without creating a second server loop.

### WP4 — Family integration

Wire `httpserver`, `ircserver`, and inbound `httpbidirserver` to the shared policy. `httpbidirserver` must reuse the HTTP server policy rather than instantiate a separate rule set.

### WP5 — Regression tests and docs

Add focused concurrency/time/state tests and update tunnel backend/support docs with actual defaults and rejected fields.

## 10. Required tests

At minimum:

- default global concurrent ceiling is 30;
- explicit valid `MaxConcurrentConns` applies; zero/over-hard-max rejects before allocation;
- one peer cannot hold more than the finite peer ceiling while another peer still obtains capacity;
- 31st global connection is rejected at default capacity;
- peer minute/hour/day counters throttle only that peer when aggregate limits remain available;
- aggregate minute limit throttles new connections across identities;
- window expiry restores admission with Tokio paused time;
- active lease releases on normal return;
- releases on early parser error;
- releases on panic-isolated handler;
- releases on cancellation/abort;
- peer table at capacity denies a new identity without evicting an active/throttled entry;
- expired inactive entries are reclaimed;
- no local TCP connect occurs for common-admission rejection;
- `httpbidirserver` inbound half uses the same admission policy path as `httpserver`;
- unsupported/underspecified server options reject before SAM session allocation;
- error/debug output contains no private destination or option secret.

## 11. Verification

Run at minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Run existing M061 and M062/M063 containment tests explicitly if they are not already clearly included in the package test output. Use the repository's existing nightly rustfmt convention only for touched files; do not create a formatting/CI cleanup project.

## 12. Acceptance criteria

M074 may close only when:

1. all three accepted-stream server families consume one peer-aware admission implementation;
2. default/global and peer-specific concurrency are finite;
3. peer and aggregate minute/hour/day controls are applied or truthfully rejected;
4. no active limiter entry can be churn-evicted by a new attacker identity;
5. denial occurs before handler/local-target allocation;
6. lifecycle and panic paths cannot leak permits/counts;
7. deterministic fairness tests show one peer cannot monopolize the global pool;
8. no new production path exists outside `emissary-cli/src/i2pcontrol/**`;
9. M061/M062/M063 containment remains green;
10. no high/medium finding within this milestone remains.

## 13. Stop conditions

Stop and create a separate corrective/architecture plan if peer-aware acceptance requires a new core stream API, if safe destination identity cannot be obtained before handler allocation, or if implementing an underspecified Proposal 170 field would require guessing wire semantics.

Closure must attest that external I2P/I2P+ sources were read-only and no upstream interaction occurred.
