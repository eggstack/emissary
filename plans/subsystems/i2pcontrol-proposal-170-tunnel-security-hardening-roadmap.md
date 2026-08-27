# I2PControl Proposal 170 Tunnel Security Hardening Roadmap

Status: corrective pass required; M092 closed; M093 ready

Original planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Post-M089 corrective planning baseline: `f0f3fc2204318c2fac69817d347df2702c51287b`.

Current corrective planning baseline: `944da7b887b6efbd46601e9fad1c853581f40b8e`.

Known valid pre-M091 implementation/closure baseline: `6d631d4423c7faa761b47a84e07436bbaf5d9ad4`.

Source runtime roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001, ADR-0002, ADR-0003;
- M061 source-containment and M062/M063 dependency-containment authorities;
- M072 runtime-reclosure history;
- M073-M090 tunnel-security implementation/closure history.

Pinned external contract:

- I2P Proposal 170, `I2PControl Expansion`, revision created/updated `2026-05-20`.

External I2P/I2P+/Yosemite sources remain read-only behavioral/security evidence. No upstream issue, PR, review, submission, merge request, contribution preparation, repository write, or maintainer contact is authorized.

## 1. Purpose and current disposition

The twelve registered Proposal 170 tunnel backends remain functionally implemented. M090 is valid and retained. The security line is reopened because M091's technical implementation crossed the registered authorization boundary and then rewrote its own blocker/containment history after the fact.

At planning commit `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3`, M091 was explicitly `blocked`. It stated that current Yosemite 0.7.0 exposed no supported typed transport for `i2p.streaming.maxConcurrentStreams` and that M091 MUST remain blocked until a maintainer explicitly authorized a narrow transport. It specifically said registration did not authorize vendoring/forking Yosemite, an unreviewed git dependency, or a process-global registry.

Commit `5053ce6b595351b251afb36f1f7d5278ef8f58d1` nevertheless switched the workspace to a full vendored Yosemite 0.7.0 copy, changed root dependency/lockfile state, changed three `emissary-core` SAM/streaming files, changed accepted-server session construction, and amended M060/M061/M062 containment machinery to admit those changes. Commit `944da7b887b6efbd46601e9fad1c853581f40b8e` then changed M091 from blocked to closed and described a maintainer authorization not present before implementation.

Per `plans/003-planning-process.md`, technical success does not supply retroactive authority. M091 therefore requires a corrective pass.

M092 is closed; see `plans/closure/i2pcontrol-proposal-170/092-closure.md`. M093 is the dependency-ready next security reclosure and is now the active implementation handoff.

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

Relevant properties remain:

- no direct identity disclosure through local target routing, spoofable headers, diagnostics, or private server Destination handling;
- resolver-independent confinement of server-local targets;
- bounded concurrency/state/tasks/parser/payload/relay lifetime where the approved boundary can enforce it;
- useful half-close behavior without indefinite zero-progress occupancy;
- no unnecessary load/timing modulation primitive created by avoidable resource retention;
- no broad router/core change where a smaller I2PControl-owned correction suffices.

Public-network deanonymization experiments, timing-jitter theater, padding, and adversarial traffic generation against third parties remain prohibited.

## 3. Durable security/anonymity invariants

The corrective line MUST preserve:

- exact Proposal 170 wire fields/actions/types/statuses;
- all twelve registered tunnel backends;
- authenticated remote identity from SAM/Yosemite only;
- bounded remote Destination text, exactly one supported `Destination`, zero parser remainder, canonical full-Destination text, and 32-byte cryptographic accounting ID;
- transactional bounded post-accept application admission before handler/local-target work;
- finite application global/per-peer concurrency and configured peer/aggregate rate counters;
- bounded peer map, expiry index, POST limiter, task groups, buffers, and Streamr subscriber state;
- HTTP identity/proxy spoof stripping, response fingerprint stripping, unambiguous framing, fixed `Expect` rejection, and bounded POST accounting;
- literal-loopback HTTP/IRC local targets from M090;
- IRC bounded registration, five-second target connect, ten-minute progress-resetting inactivity, and useful half-close/drain behavior;
- generic server progress-based inactivity and half-close behavior from M087;
- Streamr loopback-only local boundary and bounded fanout;
- generation-local ephemeral state and stable backend-owned persistent server identity;
- no lock across network I/O/sleeps/joins;
- no private Destination material in diagnostics;
- no startup/frontend ownership refactor;
- unsupported/underspecified runtime options fail before allocation rather than persist-and-ignore;
- no upstream interaction.

After M092 rollback, documentation MUST state truthfully that common application admission is post-accept and that lower-layer signed-SYN/streaming work can occur before `ServerAdmissionState` runs.

## 4. Explicit non-goals

The corrective line does not authorize:

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
- retaining or replacing M091's Yosemite vendoring/forking/path/git dependency transport;
- broad router/streaming rewrites;
- hosted CI/fuzz/soak/release machinery;
- public-network deanonymization/load tests;
- upstream contribution preparation or review requests.

A future lower-layer transport can only be reconsidered under a new explicit maintainer directive that names and accepts its dependency/core maintenance cost. M092 itself grants no such authority.

## 5. Accepted server security architecture after M092

M088 established the lower-layer boundary:

```text
remote signed streaming SYN
  -> Emissary streaming parse/signature/replay work
  -> pending/active stream-manager and routing/SAM work
  -> Yosemite Session<style::Stream>::accept()
  -> TrustedPeerIdentity
  -> ServerAdmissionState
  -> bounded application handler/local target
```

M091 attempted to add a concurrency decision between replay validation and normal pending/active allocation. M092 intentionally removes that attempt because its transport was not authorized under the registered handoff.

The post-M092 architecture therefore returns to the M088 boundary. This is a known availability/timing residual, not evidence of a direct clearnet identity leak. Existing application admission remains defense in depth for peer/global concurrency and configured rate/cardinality limits after accept.

## 6. Family-specific retained dispositions

### Generic server

M087 remains authoritative: finite progress-based inactivity, useful half-close drain, bounded local target connect, and no absolute lifetime cap on an active stream.

### HTTP server family

M090 remains authoritative for literal-loopback target normalization. Earlier HTTP hardening remains authoritative for trusted peer identity, spoof/fingerprint stripping, fail-closed framing, `Expect` rejection before local target allocation, finite body/relay deadlines, and bounded POST state.

No speculative body byte cap or replacement fairness algorithm is authorized merely for closure.

### IRC server

M090 remains authoritative for literal-loopback target normalization and useful half-close/drain. Existing bounded registration, five-second target connect, ten-minute progress-resetting inactivity, and raw post-registration behavior remain unchanged.

### Streamr

The ten-subscriber / 60-second expiry model remains a bounded, reference-aligned specialized service. An attacker with enough Destinations may monopolize the finite subscriber set. This remains an accepted availability limitation, not a reason to invent new public authentication/fairness semantics in this line.

### HTTP bidirectional server

The fork-local separate unpublished outbound client session remains privacy-positive isolation. Do not change it to Java I2P's same-manager/public-Destination sharing without separate authorization.

## 7. Dependency graph

Historical sequence:

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
M089 independent reclosure @ f0f3fc2
          |
          v
M090 resolver-free loopback + IRC half-close
```

Current corrective sequence:

```text
M090 valid closed baseline
          |
          v
M091 implementation landed while blocked             [CORRECTIVE PASS REQUIRED]
          |
          v
M092 authorization/dependency/containment rollback    [CLOSED]
  |
  v
M093 independent corrected-head security reclosure    [READY]
```

M093 is executable now and is the active ready handoff.

## 8. Milestone summary

### M087 — Generic Server Inactivity Timeout Corrective

Status: **closed**.

Plan: `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/087-closure.md`.

### M088 — Pre-Accept Server Admission Boundary Corrective

Status: **closed; evidence-only Tier 3**.

Plan: `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/088-closure.md`.

M088 becomes the current lower-layer residual disposition again after M092 removes the M091 transport.

### M089 — Post-Corrective Tunnel Security Reclosure

Status: **closed for pinned head `f0f3fc2`**.

Plan: `plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/089-closure.md`.

M089 remains valid historical evidence for its reviewed head but is not sufficient current-head authority after M090/M091/M092 changes.

### M090 — Server Loopback and IRC Half-Close Corrective

Status: **closed / retained**.

Plan: `plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/090-closure.md`.

M092 MUST NOT revert M090.

### M091 — Pre-Accept Stream Concurrency Boundary Hardening

Status: **corrective pass required**.

Plan: `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md`.

Closure record: `plans/closure/i2pcontrol-proposal-170/091-closure.md`.

The technical implementation/test evidence is retained as history, but the closure is not current authority because the implementation dependency strategy was not authorized by the registered blocked handoff before it landed. M092 owns the corrective disposition.

### M092 — M091 Authorization, Dependency, and Containment Corrective

Status: **closed**.

Plan: `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/092-closure.md`.

Required outcome:

- preserve M090;
- restore crates.io Yosemite 0.7.0 and the pre-M091 lockfile entry;
- remove `vendor/yosemite/**`;
- remove M091's accepted-server lower-layer option seam and three core SAM/streaming changes;
- restore M060/M061/M062 production/dependency containment semantics to their pre-M091 form;
- retain only exact M092/M093 planning/closure path bookkeeping if required;
- restore truthful M091 blocked/superseded status and mark its closure corrective-pass-required;
- explicitly return the M088 lower-layer residual to accepted status;
- make no new production feature.

### M093 — Post-M092 Tunnel Security Reclosure

Status: **ready** after M092 closure.

Plan: `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md`.

M093 is a no-production-change independent review of all twelve tunnel backends, M090 retention, M092 rollback/containment correctness, and the accepted residual-risk set. Any high/medium production defect opens a new numbered corrective and keeps M093 blocked.

Closure target: `plans/closure/i2pcontrol-proposal-170/093-closure.md`.

## 9. Containment policy

Preferred production boundary remains `emissary-cli/src/i2pcontrol/**`.

M090 is a valid in-boundary correction.

M092 is a bounded rollback exception. Its only production/dependency purpose is to remove M091's unauthorized expansion and restore the smaller boundary. It may touch only the exact M091 production/dependency/containment files listed in the M092 plan.

M062 planning bookkeeping may add exact M092/M093 plan/closure paths. It must not retain M091 core/vendor/lockfile production allowances after M092.

M093 has no production authority.

## 10. Verification discipline

### M092

At minimum:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m060_containment --test m061_containment --test m062_dependency_containment
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Also compare every M091-touched production/dependency file to M090 closure head `6d631d4423c7faa761b47a84e07436bbaf5d9ad4` and verify M090 HTTP/IRC focused regressions remain green.

### M093

Run the full I2PControl suite, relevant core tests, all containment guards, focused generic/HTTP/IRC/Streamr/persistent-key tests, source audit, and diff review. M093 does not add production code or hosted/public-network test infrastructure.

## 11. Stop conditions

Stop rather than widen the active milestone if:

- M092 cannot remove the M091 vendor/core/dependency delta without retaining a new lower-layer transport;
- rollback would require broad unrelated core/router changes;
- M090 would need to be reverted;
- unrelated dependencies or streaming defaults would change;
- HTTP/Streamr semantics would change solely because a theoretically stronger policy exists;
- a new Proposal 170 API field/type/action is proposed;
- public-network deanonymization/load testing is proposed;
- upstream contribution/review/contact activity is proposed.

If M093 finds a new high/medium production defect, create a new numbered corrective instead of modifying production code under M093.

## 12. Closure rule

The tunnel-security line remains **open** until M092 and then M093 have accepted closures.

M090 remains valid closed production work. M091 remains technical history but is corrective-pass-required. M092 restores the intended dependency/core/containment boundary; M093 then decides current-head tunnel-security closure.

Proposal 170 remains separately partial for accepted source/truthfulness limitations, RouterInfo 37/1/5 disposition, M051 blocker, and unrelated AddressBook/base-I2PControl limitations.

No upstream review, acceptance, merge, adoption, or submission is implied or authorized.
