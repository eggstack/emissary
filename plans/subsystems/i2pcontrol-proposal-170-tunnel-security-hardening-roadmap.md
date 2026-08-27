# I2PControl Proposal 170 Tunnel Security Hardening Roadmap

Status: production/security current-head closed after M093; M094 documentation/evidence reconciliation ready

Original planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Post-M089 corrective planning baseline: `f0f3fc2204318c2fac69817d347df2702c51287b`.

M091 corrective planning baseline: `944da7b887b6efbd46601e9fad1c853581f40b8e`.

M094 planning baseline: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.

Known valid pre-M091 implementation/closure baseline: `6d631d4423c7faa761b47a84e07436bbaf5d9ad4`.

M092 implementation head: `8860407a79347ce925603821cdb231e47a680623`.

M093 closure/planning head: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.

Source runtime roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001, ADR-0002, ADR-0003;
- M061 source-containment and M062/M063 dependency-containment authorities;
- M072 runtime-reclosure history;
- M073-M090 tunnel-security implementation/closure history;
- M092 rollback/containment corrective authority;
- M093 corrected-head security reclosure authority.

Pinned external contract:

- I2P Proposal 170, `I2PControl Expansion`, revision created/updated `2026-05-20`.

External I2P/I2P+/Yosemite sources remain read-only behavioral/security evidence. No upstream issue, PR, review, submission, merge request, contribution preparation, repository write, or maintainer contact is authorized.

## 1. Purpose and current disposition

The twelve registered Proposal 170 tunnel backends remain functionally implemented. M090 is valid and retained. M092 removed M091's unauthorized root-dependency/Yosemite-vendor/core/containment expansion, and M093 independently reclosed the corrected production head.

At planning commit `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3`, M091 was explicitly `blocked`. It stated that current Yosemite 0.7.0 exposed no supported typed transport for `i2p.streaming.maxConcurrentStreams` and that M091 MUST remain blocked until a maintainer explicitly authorized a narrow transport. It specifically said registration did not authorize vendoring/forking Yosemite, an unreviewed git dependency, or a process-global registry.

Commit `5053ce6b595351b251afb36f1f7d5278ef8f58d1` nevertheless switched the workspace to a full vendored Yosemite 0.7.0 copy, changed root dependency/lockfile state, changed three `emissary-core` SAM/streaming files, changed accepted-server session construction, and amended M060/M061/M062 containment machinery to admit those changes. Commit `944da7b887b6efbd46601e9fad1c853581f40b8e` then changed M091 from blocked to closed and described a maintainer authorization not present before implementation.

Per `plans/003-planning-process.md`, technical success does not supply retroactive authority. M092 corrected this by restoring the M090 production/dependency boundary. M093 then found no high-, medium-, or low-severity production security/anonymity defect inside the approved Proposal 170 boundary.

The remaining work is administrative only. M094 reconciles stale M092/M093 status wording and ambiguous SHA-role terminology across the active planning records. Production/runtime security remains closed by M093 throughout M094.

Current ready handoff:

- `plans/implementation/i2pcontrol-proposal-170/094-post-m093-planning-state-reconciliation.md` — **ready / documentation only**.

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

Documentation must state truthfully that common application admission is post-accept and that lower-layer signed-SYN/streaming work can occur before `ServerAdmissionState` runs. M088 remains the accepted lower-layer residual disposition.

## 4. Explicit non-goals

The active line does not authorize:

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

A future lower-layer transport can only be reconsidered under a new explicit maintainer directive that names and accepts its dependency/core maintenance cost. M094 grants no such authority.

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

M091 attempted to add a concurrency decision between replay validation and normal pending/active allocation. M092 intentionally removed that attempt because its transport was not authorized under the registered handoff.

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
M092 authorization/dependency/containment rollback    [CLOSED @ 8860407]
          |
          v
M093 independent corrected-head security reclosure    [CLOSED @ 4da022e]
          |
          v
M094 post-M093 planning-state reconciliation          [READY / DOCS ONLY]
```

M094 is dependency-ready because M092 and M093 are both closed. It does not reopen their production/security work.

## 8. Milestone summary

The tunnel-security corrective history covers M087–M094. Production/security is closed after M093; M094 is the sole open documentation/evidence reconciliation milestone.

### M087 — Generic Server Inactivity Timeout Corrective

Status: **closed**.

Plan: `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/087-closure.md`.

### M088 — Pre-Accept Server Admission Boundary Corrective

Status: **closed; evidence-only Tier 3**.

Plan: `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/088-closure.md`.

M088 is the current lower-layer residual disposition after M092 removed the M091 transport.

### M089 — Post-Corrective Tunnel Security Reclosure

Status: **closed for pinned head `f0f3fc2`**.

Plan: `plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/089-closure.md`.

M089 remains valid historical evidence for its reviewed head but is not sufficient current-head authority after M090/M091/M092 changes.

### M090 — Server Loopback and IRC Half-Close Corrective

Status: **closed / retained**.

Plan: `plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/090-closure.md`.

M090 MUST NOT be reverted by later reconciliation work.

### M091 — Pre-Accept Stream Concurrency Boundary Hardening

Status: **corrective pass required / superseded by M092**.

Plan: `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md`.

Closure record: `plans/closure/i2pcontrol-proposal-170/091-closure.md`.

The technical implementation/test evidence is retained as history, but the closure is not current authority because the implementation dependency strategy was not authorized by the registered blocked handoff before it landed.

### M092 — M091 Authorization, Dependency, and Containment Corrective

Status: **closed**.

Plan: `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/092-closure.md`.

Implementation head: `8860407a79347ce925603821cdb231e47a680623`.

M092 preserved M090, restored crates.io Yosemite 0.7.0, removed `vendor/yosemite/**`, removed M091's accepted-server/core streaming seam, restored M060/M061/M062 containment semantics, retained M091 as non-authoritative history, and returned M088's lower-layer residual to current accepted status.

### M093 — Post-M092 Tunnel Security Reclosure

Status: **closed**.

Plan: `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/093-closure.md`.

Reviewed production head: `8860407a79347ce925603821cdb231e47a680623`.

Closure/planning commit: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.

M093 independently audited all twelve tunnel backends at the M092-corrected production head, verified M090 retention, verified M091 production/dependency/vendor removal, verified M060/M061/M062 containment restoration, rechecked generic/HTTP/IRC/Streamr security and lifetime behavior, and recorded the post-accept application-admission boundary plus the accepted lower-layer residual. No high-, medium-, or low-severity production security/anonymity defect was found.

### M094 — Post-M093 Planning-State Reconciliation

Status: **ready**.

Plan: `plans/implementation/i2pcontrol-proposal-170/094-post-m093-planning-state-reconciliation.md`.

Closure target: `plans/closure/i2pcontrol-proposal-170/094-closure.md`.

M094 is documentation/evidence reconciliation only. It must:

- change the stale M092 plan header from `ready` to `closed` and link its closure;
- rewrite stale live-readiness wording in the closed M093 plan as historical sequencing evidence;
- pin M092 implementation head `8860407a79347ce925603821cdb231e47a680623` in the M092 closure;
- distinguish M093 reviewed production head `8860407a79347ce925603821cdb231e47a680623` from closure/planning commit `4da022ec874e9915e2d38fe63c609bff537ee8ff` in the M093 closure;
- converge registry/README/roadmap active-state wording;
- add only exact M094 plan/closure paths to the M062 planning allowlist;
- make no production/dependency change and preserve M088/M090/M091/M092/M093 technical dispositions.

## 9. Containment policy

Preferred production boundary remains `emissary-cli/src/i2pcontrol/**`.

M090 is a valid in-boundary correction.

M092 was a bounded rollback exception whose production/dependency purpose was solely to remove M091's unauthorized expansion and restore the smaller boundary.

M093 had no production authority. M094 likewise has no production authority.

The only M094 change permitted outside `plans/**` is adding the exact M094 plan and closure strings to `emissary-cli/tests/m062_dependency_containment.rs::is_authorized_planning_path`. No other predicate, path, lockfile assertion, dependency rule, production allowance, or test semantic may change.

## 10. Verification discipline

### M092

Historical closure verification includes the full core/I2PControl suites, M060/M061/M062 containment tests, clippy, and diff review against the M090 closure baseline.

### M093

Historical closure verification includes the full I2PControl suite, relevant core tests, all containment guards, focused generic/HTTP/IRC/Streamr/persistent-key tests, source audit, and diff review. M093 added no production code.

### M094

At minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Also inspect:

```text
git diff --name-only 4da022ec874e9915e2d38fe63c609bff537ee8ff..HEAD
```

Every changed path must be one of the exact planning/evidence paths in the M094 plan. The diff to `emissary-cli/tests/m062_dependency_containment.rs` must consist only of the exact M094 plan/closure allowlist entries.

A full runtime/core suite is not required solely for M094 because production/runtime changes are a stop condition, not authorized work.

## 11. Stop conditions

Stop rather than widen M094 if:

- any production/runtime/core/router/startup/frontend source change is proposed;
- any dependency or lockfile change is proposed;
- M090 behavior would change;
- M091 lower-layer transport would be reinstated or replaced;
- M088's residual-risk disposition would change;
- M093's production security findings would be rewritten rather than only its SHA-role/status terminology;
- HTTP/IRC/Streamr or other tunnel semantics would change;
- a new Proposal 170 API field/type/action is proposed;
- public-network deanonymization/load testing is proposed;
- upstream contribution/review/contact activity is proposed.

If M094 review uncovers a production/security defect, open a separate numbered corrective rather than changing production code under M094.

## 12. Closure rule

Production/runtime tunnel security remains **current-head closed by M093** while M094 is open.

M094 closes only the planning/evidence reconciliation. Its closure may return the planning line to `closed / no ready handoff` after all stale state/SHA wording is reconciled and the M062 exact planning-path registration is verified.

M090 remains valid closed production work. M091 remains technical history but is corrective-pass-required/superseded. M092 remains the rollback/containment authority. M093 remains the corrected-head production security reclosure authority. M088 remains the accepted lower-layer/pre-accept residual disposition.

Proposal 170 remains separately partial for accepted source/truthfulness limitations, RouterInfo 37/1/5 disposition, M051 blocker, and unrelated AddressBook/base-I2PControl limitations.

No upstream review, acceptance, merge, adoption, or submission is implied or authorized.