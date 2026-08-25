# I2PControl Proposal 170 Tunnel Security Hardening Roadmap

Status: corrective pass in progress after post-M086 active-adversary review; M087 closed, M088 ready

Original planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Post-M076 corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Merged-head corrective baseline: `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae`.

M084 post-fix baseline: `1196a4d85cecb4f9676a8d87d27c69322816d7a8`.

M085 final reviewed head: `a6f18268b8d8724ed826f69614161b5b8d293ef5`.

M086 planning baseline: `185d43174c491a57c217c39e45555d136f40a406`.

M087-M089 corrective planning baseline: `2b01bfd11ebcd768fcd5488f18b063ac336931a2`.

Source runtime roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001, ADR-0002, ADR-0003;
- M061 source-containment and M062/M063 dependency-containment authorities;
- M072 runtime-reclosure history;
- M073-M086 tunnel-security implementation/closure history.

Pinned external contract:

- I2P Proposal 170, `I2PControl Expansion`, open revision created/updated `2026-05-20`.

External I2P/I2P+/Yosemite sources remain read-only behavioral/security evidence. No upstream issue, PR, review, submission, merge request, contribution preparation, repository write, or maintainer contact is authorized.

## 1. Purpose and current disposition

The tunnel runtime remains functionally complete for the twelve registered Proposal 170 tunnel types, but the tunnel-security line is reopened for a narrow post-M086 corrective sequence.

M085 remains valid final runtime/security evidence for its pinned post-M084 head. M086 remains valid documentation/evidence reconciliation for its scope. Neither record is being rewritten as invalid history.

A later active-adversary review found two server-side hardening issues not fully captured by the M085 threat model:

1. generic `server` relays can remain open indefinitely after local-target connect when no bytes make progress, allowing a small collection of Sybil Destinations to pin the finite shared admission pool;
2. common accepted-stream admission executes after Yosemite/SAM `session.accept()` returns, so application-level rejection does not necessarily bound work already spent in lower-layer stream establishment.

The corrective sequence is deliberately narrow:

- **M087** — add a progress-based inactivity bound to generic `server` raw relay;
- **M088** — map and, only if narrowly supported, harden the lower-layer/pre-accept stream admission boundary;
- **M089** — independently reclose the complete tunnel-security line and disposition HTTP/Streamr residual questions.

M087 is closed. M088 is currently `ready`; M089 remains future/blocked pending M088 closure.

## 2. Corrective threat model

The corrective review assumes a remote adversary may create many valid I2P Destinations and may repeatedly open, stall, half-close, reset, and reconnect streams while observing ordinary protocol-visible outcomes.

Per-Destination quotas are treated as fairness/amplification controls, not Sybil resistance.

The relevant security properties are:

- no direct identity disclosure through local target routing, spoofable headers, diagnostics, or private server Destination handling;
- bounded attacker-controlled concurrency, state, task, parser, payload, and relay lifetime where the current repository boundary can enforce it;
- no unnecessary load/timing modulation primitive created by indefinitely attacker-owned resources;
- no scope expansion into router algorithms merely to claim defense against a threat the Proposal 170 control-plane implementation cannot safely own.

Public-network deanonymization experiments, timing-jitter theater, padding, and adversarial traffic generation against third parties remain prohibited.

## 3. Security/anonymity invariants retained

M087-M089 MUST preserve the previously accepted invariants:

- exact Proposal 170 wire fields/actions/types/statuses;
- authenticated remote identity from SAM/Yosemite only;
- bounded Base64 peer text, exactly one supported `Destination`, zero parser remainder, canonical full-Destination text, and 32-byte cryptographic accounting ID;
- transactional application admission denial before handler/local-target work;
- finite global/per-peer concurrency and configured peer/aggregate rate counters;
- historical peer state only when configured semantics require it;
- no-history inactive peer reclamation;
- bounded peer map, expiry index, POST limiter, task groups, buffers, Streamr subscribers, and stop waits;
- HTTP identity/proxy spoof stripping, response fingerprint stripping, unambiguous framing, fixed `Expect` rejection, and bounded POST accounting;
- IRC bounded registration/connect/idle occupancy with raw post-registration relay;
- Streamr loopback-only local boundary and bounded fanout;
- generation-local ephemeral state and stable backend-owned persistent server identity;
- no lock across network I/O/sleeps/joins;
- no private destination material in diagnostics;
- no local DNS/LAN routing expansion;
- no startup/router/frontend ownership refactor;
- unsupported/underspecified runtime options fail before allocation rather than persist-and-ignore;
- no upstream interaction.

M087 adds a generic-server inactivity property. M088 may add a lower-layer connection bound only if that semantic already exists at a narrowly exposable boundary.

## 4. Explicit non-goals

The corrective line does not authorize:

- new tunnel types or Proposal 170 API fields;
- new RouterInfo source owners or AddressBook/base-I2PControl expansion;
- process-wide admission budgets shared across multiple configured tunnels;
- replacement of existing fixed-window admission with token bucket/GCRA;
- generalized Sybil resistance;
- randomized rejection timing/jitter/padding;
- new HTTP methods/features or informational-response state machines;
- new public Streamr authentication/allowlist semantics without a separate plan;
- a parallel SAM implementation in `i2pcontrol`;
- Yosemite vendoring/forking/patching without separate explicit dependency-boundary planning;
- new router/streaming algorithms;
- hosted CI/fuzz/soak/release machinery;
- public-network deanonymization/load tests;
- upstream contribution preparation or review requests.

## 5. Why HTTP and Streamr are not separate production plans now

### HTTP

Current HTTP server handling already has the security properties that matter for the reviewed resource path:

- bounded request-line/header parsing;
- fail-closed ambiguous framing and unsupported `Expect` handling;
- trusted peer identity derived from Yosemite/SAM rather than request headers;
- spoofable proxy/I2P identity header stripping;
- response fingerprint stripping;
- finite request-body relay deadline;
- bounded fail-closed POST limiter state.

A body byte cap could be a compatibility-affecting new policy rather than a correction, and the bounded POST map can still be pressured by Sybil Destinations despite being memory-safe. Neither observation by itself establishes a high/medium fork defect requiring production change.

M089 therefore rechecks these properties. A concrete new high/medium defect must open a separate numbered plan rather than expanding M089.

### Streamr

The current ten-subscriber / 60-second expiry behavior aligns with Java I2P and I2P+ reference implementations. An attacker with enough Destinations can refresh and monopolize the finite subscriber set, so the model is not Sybil-resistant.

That remains an explicit specialized availability limitation, not presently a fork regression. M089 must record it. Stronger allowlist/auth/fairness semantics require separate compatibility evidence and are not pre-authorized.

## 6. Dependency graph

Historical sequence remains preserved:

```text
M080 -> M081 -> M082 -> M083
                    \
                     +-- merged into current head
M077 -> M078 -> M079/
             |
             v
            M084 merged-head integration corrective
             |
             v
            M085 independent reclosure
             |
             v
            M086 documentation/evidence reconciliation
```

New corrective sequence:

```text
M087 closed implementation baseline
             |
             v
M088 pre-accept/lower-layer admission    [READY]
             |
             v
M089 independent security reclosure      [FUTURE/BLOCKED]
```

M087 -> M088 was administrative sequencing and is now satisfied; M088 remains feasibility-gated by the actual lower-layer capability.

M089 requires accepted M087 and M088 closure. If M088 discovers that a separate dependency-boundary plan is required, M089 remains blocked until that plan is resolved or the residual limitation is explicitly accepted as out of scope.

## 7. Milestone summary

### M074 — Shared application admission hardening

Closed with corrective history; M080/M083 own later admission corrections. M088 now examines only the earlier lower-layer boundary, not the correctness of the existing application admission algorithm.

### M075 — Generic accepted-stream migration

Closed. It intentionally retained raw `copy_bidirectional` semantics and did not invent an idle timeout without separate compatibility/security evidence. M087 is the separate corrective anticipated by that stop condition.

### M076 — HTTP anonymity/POST hardening

Closed with corrective history. M082/M083/M084/M085 retain later identity/framing/integration corrections. M089 rechecks current behavior but no new HTTP production plan is currently registered.

### M077 — IRC lifetime/exhaustion hardening

Closed and retained. M089 rechecks the five-second target connect and ten-minute activity-resetting idle expiry.

### M078 — Streamr local-boundary hardening

Closed and retained. M089 rechecks bounded fanout and documents reference-aligned Sybil monopolization risk.

### M079 — Historical integrated reclosure

Historical older-lineage closure only.

### M080 — Admission transactionality/cardinality corrective

Closed with corrective history; retained by M083/M085.

### M081 — Generic server LeaseSet option truthfulness

Closed and retained.

### M082 — HTTP peer identity / Expect / POST corrective

Closed with corrective history; retained by M083/M085.

### M083 — Admission capacity and trusted Destination exactness

Closed and retained. Current trusted identity uses exact framed parsing, requires zero remainder, derives the cryptographic Destination ID, and canonicalizes full-Destination text.

### M084 — Merged-head integration and planning corrective

Closed. Historical merged-head repair authority.

### M085 — Merged-head tunnel-security reclosure

Closed and valid for its pinned head. It remains the latest accepted full runtime/security reclosure until M089 closes, but its threat analysis is no longer the final word on the two newly identified server-side corrective findings.

### M086 — Post-M085 documentation/evidence reconciliation

Closed. Documentation/evidence-only; no runtime change.

### M087 — Generic Server Inactivity Timeout Corrective

Status: **closed**. See `plans/closure/i2pcontrol-proposal-170/087-closure.md`.

Required outcome:

- replace indefinite zero-progress generic relay occupancy with a finite inactivity/progress timeout;
- preserve active long-lived raw streams and useful half-close behavior;
- stay inside `emissary-cli/src/i2pcontrol/**` plus focused test/planning bookkeeping;
- add no Proposal 170 field/dependency/core/router/startup/frontend change.

Plan: `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md`.

### M088 — Pre-Accept Server Admission Boundary Corrective

Status: **ready**; the M087 dependency is closed.

Required outcome:

- source-map remote stream establishment through Emissary SAM/Yosemite into application admission;
- establish from code whether Emissary actually supports Java-style lower-layer streaming connection controls;
- wire the smallest supported lower-layer global bound if the current boundary permits it;
- preserve current post-accept application admission as defense in depth;
- stop rather than vendoring/forking Yosemite or broadly modifying router/core code without a separate explicit plan;
- truthfully document a residual limitation if the earliest in-scope enforceable boundary remains post-accept.

Plan: `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md`.

### M089 — Post-Corrective Tunnel Security Reclosure

Status: **future/blocked** on M087 + M088.

Required outcome:

- independently re-audit all twelve tunnel types at the corrected head;
- verify generic lifetime and lower-layer admission dispositions;
- recheck HTTP bounded body/POST behavior without adding speculative limits;
- recheck Streamr reference parity and explicitly record its Sybil limitation;
- verify source/dependency containment;
- make no production change itself.

Plan: `plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md`.

## 8. Containment policy for the corrective sequence

Preferred production boundary remains `emissary-cli/src/i2pcontrol/**`.

### M087

Expected to touch only generic server backend code and colocated tests under `i2pcontrol`, plus planning/closure bookkeeping.

### M088

Starts with source mapping. If the desired lower-layer capability is already exposed, changes should remain in accepted-server/admission session-option plumbing under `i2pcontrol`.

If Yosemite must be vendored/forked/patched, a git dependency introduced, `Cargo.lock` changed, or `emissary-core/**`/router algorithms modified, M088 does not authorize that work. Stop and create a separate explicit plan.

### M089

No production changes. Any production defect becomes a new corrective.

M062 exact-path bookkeeping may add M087-M089 implementation/closure documents without broadening production path globs.

## 9. Verification discipline

### M087

Focused generic-server tests + M062 + `git diff --check`.

### M088

Focused accepted-server/admission tests when code changes; otherwise source/revision evidence + M062 + `git diff --check` for an unsupported-capability disposition.

### M089

Full `emissary-cli` i2pcontrol test suite + M062 + targeted source audit + `git diff --check`.

Do not create hosted CI/fuzz/soak/public-network infrastructure for these bounded corrections.

## 10. Stop conditions

Stop the corrective sequence and create separate planning rather than widen the active milestone if:

- generic inactivity hardening requires a new protocol parser or non-i2pcontrol production change;
- lower-layer admission requires a new router/streaming algorithm;
- Yosemite must be forked/vendored/patched;
- a new dependency/default feature is required;
- broad `emissary-core/**`, startup, frontend, or router refactoring is proposed;
- a new Proposal 170 API field is proposed;
- HTTP/Streamr semantics would be changed solely because a theoretical stronger policy exists rather than because a concrete defect is demonstrated;
- public-network deanonymization/load testing is proposed;
- upstream contribution/review/contact activity is proposed.

## 11. Final closure rule

The tunnel-security line is now **open for the bounded M087-M089 corrective sequence**.

M085 remains valid historical full reclosure evidence for its pinned head. M086 remains valid record-reconciliation evidence. Neither should be rewritten as though it had originally evaluated the later findings.

If M087 and M088 close and M089 independently finds no remaining high/medium security/anonymity/resource-exhaustion defect inside the approved Proposal 170 boundary, M089 becomes the new current-head tunnel runtime/security reclosure authority.

Proposal 170 remains separately partial for the accepted source/truthfulness limitations, RouterInfo 37/1/5 disposition, M051 blocker, and unrelated AddressBook/base-I2PControl limitations.

No upstream review, acceptance, merge, adoption, or submission is implied or authorized.
