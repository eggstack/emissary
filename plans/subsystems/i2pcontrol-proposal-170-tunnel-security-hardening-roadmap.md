# I2PControl Proposal 170 Tunnel Security Hardening Roadmap

Status: reopened; M090 closed, M091 blocked

Original planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Post-M076 corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Merged-head corrective baseline: `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae`.

M084 post-fix baseline: `1196a4d85cecb4f9676a8d87d27c69322816d7a8`.

M085 final reviewed head: `a6f18268b8d8724ed826f69614161b5b8d293ef5`.

M086 planning baseline: `185d43174c491a57c217c39e45555d136f40a406`.

M087-M089 corrective planning baseline: `2b01bfd11ebcd768fcd5488f18b063ac336931a2`.

Post-M089 corrective planning baseline: `f0f3fc2204318c2fac69817d347df2702c51287b`.

Source runtime roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001, ADR-0002, ADR-0003;
- M061 source-containment and M062/M063 dependency-containment authorities;
- M072 runtime-reclosure history;
- M073-M089 tunnel-security implementation/closure history.

Pinned external contract:

- I2P Proposal 170, `I2PControl Expansion`, open revision created/updated `2026-05-20`.

External I2P/I2P+/Yosemite sources remain read-only behavioral/security evidence. No upstream issue, PR, review, submission, merge request, contribution preparation, repository write, or maintainer contact is authorized.

## 1. Purpose and current disposition

The twelve registered Proposal 170 tunnel backends remain functionally implemented. The tunnel-security line is reopened after M089 for two narrow in-`i2pcontrol` corrections and one separately bounded lower-layer defense-in-depth plan.

M089 remains valid closure evidence for its pinned head `f0f3fc2204318c2fac69817d347df2702c51287b`. The post-M089 review does not rewrite or invalidate the evidence M089 actually gathered. Instead, it identified two details whose exact invariants were not directly regression-tested:

1. HTTP/IRC server target validation accepts the compatibility hostname `localhost`, but the runtime later passes that hostname to `TcpStream::connect`; therefore the loopback-only invariant remains dependent on resolver/NSS configuration rather than on a literal socket address;
2. `ircserver` post-registration relay terminates when either direction completes, unlike the M087-corrected generic `server` relay, so a one-sided EOF discards useful half-close/drain behavior and creates avoidable termination asymmetry.

These were owned by **M090**, which is now closed with a dedicated closure
record. No tunnel-security implementation handoff is currently dependency-ready.

The review also reconfirmed M088's medium residual: lower-layer inbound streaming work occurs before `ServerAdmissionState` can reject an accepted stream. The repository now has **M091** as a bounded owner for a future pre-accept stream concurrency defense. M091 is deliberately blocked because Yosemite 0.7.0, and current Yosemite `master`, do not expose a streaming-concurrency session option, while Emissary's declared `StreamConfig` admission fields are not currently carried as per-session configuration into `StreamManager`.

No new Streamr, HTTP body-limit/fairness, randomized timing, or `httpbidirserver` identity-sharing work is authorized by this reopening.

## 2. Threat model

The security review assumes a remote adversary may:

- create many valid I2P Destinations cheaply;
- open, stall, half-close, reset, and reconnect streams repeatedly;
- vary traffic timing to create service-load modulation;
- send malformed or ambiguous application framing;
- spoof application-level identity/proxy headers;
- attempt to monopolize bounded peer/subscriber state;
- attempt to trigger local target allocation or lower-layer stream state before application admission;
- observe protocol-visible rejection/termination timing available to an ordinary remote peer.

Per-Destination quotas remain fairness/amplification controls, not Sybil resistance.

Relevant properties are:

- no direct identity disclosure through local target routing, spoofable headers, diagnostics, or private server Destination handling;
- resolver-independent confinement of server-local targets;
- bounded concurrency/state/tasks/parser/payload/relay lifetime where the current boundary can enforce it;
- useful half-close behavior without indefinite zero-progress occupancy;
- no unnecessary load/timing modulation primitive created by avoidable resource retention;
- no broad router/core changes where a smaller I2PControl-owned correction suffices.

Public-network deanonymization experiments, timing-jitter theater, padding, and adversarial traffic generation against third parties remain prohibited.

## 3. Durable security/anonymity invariants

M090-M091 MUST preserve the previously accepted invariants:

- exact Proposal 170 wire fields/actions/types/statuses;
- authenticated remote identity from SAM/Yosemite only;
- bounded remote Destination text, exactly one supported `Destination`, zero parser remainder, canonical full-Destination text, and 32-byte cryptographic accounting ID;
- transactional bounded application admission before handler/local-target work;
- finite application global/per-peer concurrency and configured peer/aggregate rate counters;
- bounded peer map, expiry index, POST limiter, task groups, buffers, and Streamr subscriber state;
- HTTP identity/proxy spoof stripping, response fingerprint stripping, unambiguous framing, fixed `Expect` rejection, and bounded POST accounting;
- IRC bounded registration, five-second target connect, ten-minute progress-resetting inactivity, and raw post-registration bytes;
- Streamr loopback-only local boundary and bounded fanout;
- generation-local ephemeral state and stable backend-owned persistent server identity;
- no lock across network I/O/sleeps/joins;
- no private Destination material in diagnostics;
- no startup/frontend ownership refactor;
- unsupported/underspecified runtime options fail before allocation rather than persist-and-ignore;
- no upstream interaction.

M090 strengthens the local-target and half-close invariants without widening capabilities. M091, if unblocked, may add only earlier stream-concurrency defense in depth while retaining the complete post-accept application admission policy.

## 4. Explicit non-goals

The reopened line does not authorize:

- new tunnel types or Proposal 170 API fields;
- new RouterInfo source owners or AddressBook/base-I2PControl expansion;
- arbitrary LAN/clearnet server targets or new DNS behavior;
- process-wide cross-Destination admission budgets;
- wholesale replacement of fixed-window application admission with token bucket/GCRA;
- generalized Sybil resistance;
- randomized rejection timing/jitter/padding;
- new HTTP methods/features or speculative request-body byte caps;
- new public Streamr authentication/allowlist/fairness semantics;
- changing `httpbidirserver` to share the public server Destination with its local client proxy;
- a parallel SAM implementation in `i2pcontrol`;
- a generic callback from `emissary-core` into application policy;
- Yosemite vendoring/forking/patching or a git dependency without explicit later authorization;
- broad router/streaming rewrites;
- hosted CI/fuzz/soak/release machinery;
- public-network deanonymization/load tests;
- upstream contribution preparation or review requests.

## 5. Why HTTP/IRC receive M090 but Streamr does not

### HTTP local-target detail

The HTTP server family already has bounded request/header parsing, trusted peer identity, fail-closed ambiguous framing, `Expect` rejection, response fingerprint stripping, finite body relay deadlines, and bounded POST state. M090 does not revisit those controls.

The correction is narrower: accepted target spellings must become literal loopback addresses before any runtime connect. This removes resolver/NSS behavior from a security invariant without broadening HTTP capability.

### IRC relay detail

IRC already has bounded registration, trusted peer-derived presentation, a five-second local connect, and a ten-minute post-registration progress deadline. M090 changes only the EOF state machine so one completed direction half-closes the opposite writer while the other direction may drain, as the generic M087 relay already does.

### Streamr remains reference-aligned

The current ten-subscriber / 60-second expiry behavior aligns with Java I2P and I2P+ reference implementations. An attacker with enough Destinations can refresh and monopolize the finite subscriber set, so the model is not Sybil-resistant.

That remains an explicit specialized availability limitation, not a fork regression. Stronger allowlist/auth/fairness semantics require separate compatibility evidence and are not authorized by M090 or M091.

## 6. Lower-layer admission architecture

M088 established the current boundary precisely:

```text
remote signed streaming SYN
  -> Emissary streaming parse/signature/replay work
  -> pending/active stream-manager and routing/SAM work
  -> Yosemite Session<style::Stream>::accept()
  -> TrustedPeerIdentity
  -> ServerAdmissionState
  -> bounded application handler/local target
```

Java I2P applies connection limits in its streaming manager after authenticated SYN validation and before normal connection creation. Emissary currently lacks an equivalent consumed per-session configuration path.

### 6.1 M091 target

M091's preferred end state is deliberately minimal:

```text
remote signed streaming SYN
  -> parse + authenticate + replay validation
  -> per-session inbound stream concurrency check   [new]
  -> pending/active stream allocation
  -> Yosemite accept
  -> TrustedPeerIdentity
  -> existing ServerAdmissionState                  [retained]
  -> handler/local target
```

`i2pcontrol` remains the Proposal 170 policy owner. The lower layer receives only the already-validated concurrent-stream ceiling for the dedicated accepted-server session. Per-peer/rate/history semantics remain application-owned unless a separately demonstrated defect justifies further work.

### 6.2 Why M091 is blocked

At the post-M089 planning baseline:

- workspace Yosemite remains crates.io `0.7.0`;
- current Yosemite `master` is still the v0.7.0 release commit;
- Yosemite `SessionOptions` contains no `i2p.streaming.maxConcurrentStreams`-style field;
- `StreamOptions` contains only stream source/destination port controls;
- Emissary core declares `StreamConfig::max_concurrent_streams` and rate fields but current `StreamManager`/`Stream::new` paths use defaults rather than a session-carried policy.

Therefore M091 cannot truthfully begin with a simple option translation. It remains blocked until an explicit supported transport/dependency strategy is authorized.

## 7. Dependency graph

Historical sequence is retained as evidence:

```text
M074-M083 application/security hardening
          |
          v
M084 merged-head integration
          |
          v
M085 independent reclosure
          |
          v
M086 documentation/evidence reconciliation
          |
          v
M087 generic relay inactivity/half-close
          |
          v
M088 lower-layer boundary evidence
          |
          v
M089 independent current-head reclosure
```

Active sequence:

```text
M089 closed baseline @ f0f3fc2
          |
          v
M090 resolver-free loopback + IRC half-close        [CLOSED @ 172a4e8]
          |
          v
M091 pre-accept stream concurrency boundary         [BLOCKED]
          |
          v
future independent security reclosure               [UNREGISTERED]
```

M090's hard dependency is satisfied. M091 remains blocked by the
architecture/dependency blocker described in §6.2. The future reclosure remains
unregistered until M091 also has accepted closure evidence.

## 8. Milestone summary

### M074-M086 — Historical security foundation

Closed. These milestones remain authoritative for shared application admission, protocol filters, trusted peer identity, option truthfulness, merged-head integration, and prior closure evidence. Their individual plan/closure records are preserved and are not rewritten by this roadmap update.

### M087 — Generic Server Inactivity Timeout Corrective

Status: **closed**.

Established the reference relay behavior reused by M090's IRC correction: successful progress resets a ten-minute inactivity timer; one-sided EOF half-closes the opposite writer and permits the remaining direction to drain; no absolute active-connection lifetime is imposed.

Plan: `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/087-closure.md`.

### M088 — Pre-Accept Server Admission Boundary Corrective

Status: **closed; evidence-only Tier 3**.

Established that the lower-layer limitation is real and that current Yosemite/SAM does not expose the required streaming-admission semantic. M091 owns any future correction rather than rewriting M088 history.

Plan: `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/088-closure.md`.

### M089 — Post-Corrective Tunnel Security Reclosure

Status: **closed for pinned head `f0f3fc2`**.

M089 remains valid historical/current accepted evidence for that exact reviewed head. The later findings open new plans; they do not retroactively modify its closure record.

Plan: `plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/089-closure.md`.

### M090 — Server Loopback and IRC Half-Close Corrective

Status: **closed**.

Required outcome:

- normalize existing accepted HTTP/IRC local-target spellings to literal loopback `IpAddr` before runtime connection;
- retain compatibility spelling `localhost` without resolver use;
- preserve non-loopback fail-before-allocation behavior;
- change IRC EOF handling to allow the remaining direction to drain;
- retain ten-minute progress-based inactivity and five-second target connect;
- remain inside `emissary-cli/src/i2pcontrol/**` plus focused tests/planning bookkeeping;
- make no dependency/core/router/startup/frontend/Proposal 170 wire change.

Plan: `plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/090-closure.md`.

### M091 — Pre-Accept Stream Concurrency Boundary Hardening

Status: **blocked**.

Required future outcome if unblocked:

- transport only the accepted-server concurrent-stream ceiling through an explicitly supported session configuration path;
- enforce it after signed-SYN/replay validation but before normal pending/active stream allocation;
- count lower-layer inbound pending/active states that can exist before application admission;
- preserve default behavior for unrelated sessions;
- preserve complete post-accept `ServerAdmissionState` as defense in depth;
- avoid copying Java's entire per-peer/rate throttler into core absent separate evidence.

The M090 closure dependency is satisfied. Readiness still requires an explicitly
authorized configuration/dependency strategy. The current plan does not
authorize Yosemite/core/dependency changes merely by existing.

Plan: `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md`.

### Future independent security reclosure

Status: **unregistered**.

Once M090 and M091 have accepted closures, create/register one independent current-head review of all twelve tunnel types, M090's local/half-close corrections, M091's lower-layer ordering, containment, and residual risk. Do not number/register it while M091 remains blocked.

## 9. Containment policy

Preferred production boundary remains `emissary-cli/src/i2pcontrol/**`.

### M090

Expected production scope is only:

- `emissary-cli/src/i2pcontrol/backends/http_server.rs`;
- `emissary-cli/src/i2pcontrol/backends/irc_server.rs`;
- mechanically `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` only if the shared typed target/handler seam requires it;
- a tiny colocated helper under `i2pcontrol/backends/**` only if it is smaller than duplication.

No `emissary-core/**`, dependency, lockfile, startup, frontend, router, or unrelated proxy path is authorized.

### M091

M091 is blocked and therefore authorizes no production path today.

If later unblocked, every required path outside `i2pcontrol` must be enumerated exactly in the plan/containment authority before implementation. A broad `emissary-core/**` allowance is forbidden. A Yosemite version/git/vendor/fork strategy requires explicit maintainer authorization and exact manifest/lockfile disposition.

### M062 planning bookkeeping

The exact M062 planning allowlist includes M090/M091 plan and closure files. This does not broaden production globs.

## 10. Verification discipline

### M090

At minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Add deterministic loopback-normalization and IRC half-close/inactivity tests. No public-network testing.

### M091 if unblocked

At minimum:

```text
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

If a manifest/lockfile/dependency changes, update containment with an exact approved delta rather than weakening prior assertions silently.

### Future reclosure

Run the full i2pcontrol suite, relevant core streaming tests, containment guards, targeted source audit, and diff check. Do not add hosted CI/fuzz/soak/public-network infrastructure solely for closure.

## 11. Stop conditions

Stop the active milestone and create/amend separate planning rather than widen scope if:

- M090 requires production changes outside `emissary-cli/src/i2pcontrol/**`;
- resolver-free confinement would broaden target capabilities;
- IRC half-close work would require application protocol redesign;
- M091's only transport requires an unapproved Yosemite fork/vendor/git dependency;
- M091 would require a broad streaming/router rewrite;
- a callback from core into CLI/application policy is proposed;
- application rate-limit logic would be duplicated wholesale in core;
- unrelated streaming defaults would change without separate approval;
- HTTP/Streamr semantics would change solely because a theoretically stronger policy exists;
- a new Proposal 170 API field/type/action is proposed;
- public-network deanonymization/load testing is proposed;
- upstream contribution/review/contact activity is proposed.

## 12. Closure rule

The tunnel-security line is **reopened** for M090 and the blocked M091 follow-up.

M089 remains the accepted security reclosure authority for its pinned `f0f3fc2` head until a future independent reclosure supersedes it for a later head. M088 remains accepted evidence for the pre-accept limitation; M091 is the explicit future owner of that limitation.

M090 may close independently after its in-`i2pcontrol` corrections and full verification. Its closure must state whether M091's external blocker changed; it must not mark M091 ready unless the readiness gate is actually satisfied.

Proposal 170 remains separately partial for accepted source/truthfulness limitations, RouterInfo 37/1/5 disposition, M051 blocker, and unrelated AddressBook/base-I2PControl limitations.

No upstream review, acceptance, merge, adoption, or submission is implied or authorized.
