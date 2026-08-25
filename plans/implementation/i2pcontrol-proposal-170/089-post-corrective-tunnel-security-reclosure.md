# M089 — Post-Corrective Tunnel Security Reclosure

Status: closed — see `plans/closure/i2pcontrol-proposal-170/089-closure.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Controlling predecessors:

- M085 final prior runtime/security reclosure;
- M086 record reconciliation;
- M087 generic-server inactivity corrective;
- M088 pre-accept admission-boundary corrective.

Planning baseline: `2b01bfd11ebcd768fcd5488f18b063ac336931a2`.

Classification: independent corrective verification / security reclosure.

## 1. Objective

Independently re-audit the Proposal 170 tunnel runtime/security implementation after M087 and M088, certify the corrected current head if the evidence supports it, and explicitly dispose of the residual HTTP and Streamr resource-exhaustion questions without turning them into unsupported feature work.

M089 is a verification and closure milestone. It is not permission to make runtime changes opportunistically. Any newly confirmed production defect that requires code creates a new narrowly scoped numbered corrective plan.

## 2. Why a separate reclosure is required

M085 was valid for its pinned post-M084 head, but the later active-adversary review identified two additional concerns that were not part of M085's accepted threat analysis:

1. generic `server` relay lifetime was bounded only by shared admission concurrency, not by inactivity/progress;
2. common accepted-stream admission occurs after Yosemite/SAM `session.accept()` returns, leaving lower-layer stream-establishment work outside the application admission gate.

M087 and M088 own those findings. Once they are resolved or truthfully dispositioned, M089 provides an independent current-head check rather than allowing the implementing milestone to self-certify the entire server family.

## 3. Scope

Re-review all Proposal 170 tunnel types, with deeper attention to server-side anonymity and resource exhaustion:

- `client`;
- `httpclient`;
- `ircclient`;
- `socks`;
- `socksirc`;
- `connectclient`;
- `streamrclient`;
- `server`;
- `httpserver`;
- `httpbidirserver`;
- `ircserver`;
- `streamrserver`.

The production implementation remains expected to be concentrated under `emissary-cli/src/i2pcontrol/**`.

M089 itself authorizes planning/closure/test execution only. It MUST NOT modify production runtime code.

## 4. Reclosure threat model

Review from the perspective of an active remote I2P adversary who may:

- create many valid Destinations cheaply;
- open, stall, half-close, reset, and reconnect streams repeatedly;
- vary traffic timing to create service-load modulation;
- send malformed or ambiguous application framing;
- spoof application-level identity/proxy headers;
- attempt to monopolize bounded peer/subscriber state;
- attempt to force local target allocation before authentication/admission/filter decisions;
- observe protocol-visible rejection timing available to a normal remote peer.

Do not assume per-Destination quotas provide Sybil resistance.

Do not perform public-network deanonymization experiments. The review is source-level plus deterministic local tests.

## 5. Required server-family verification

### 5.1 Shared accepted-stream admission

Verify current head preserves:

- peer identity derived only from authenticated Yosemite/SAM remote Destination;
- bounded input text, exact one-Destination parsing, zero remainder, canonical full-Destination text, and 32-byte cryptographic accounting ID;
- transactional bounded `ServerAdmissionState`;
- global/per-peer concurrency and configured rate bounds;
- bounded peer-state cardinality/expiry semantics;
- no attacker-controlled peer-state mutation on denied admission;
- generation-local limiter state and bounded shutdown ownership.

Then verify M088's exact disposition:

- if lower-layer/pre-accept enforcement was implemented, confirm it acts earlier than application handler admission and application admission remains defense in depth;
- if lower-layer enforcement was proven unavailable without out-of-scope changes, confirm closure says so plainly and no document claims otherwise.

### 5.2 Generic `server`

Verify M087:

- retains loopback-only target confinement;
- retains five-second target connect bound;
- preserves raw payload relay semantics;
- uses inactivity/progress timeout rather than absolute lifetime;
- releases admission ownership on idle expiry/error/EOF;
- preserves useful half-close behavior;
- does not leak peer/private server Destination material.

### 5.3 HTTP server family

Re-evaluate the current implementation rather than presuming a new HTTP corrective is necessary.

At minimum confirm:

- request line/header parsing remains bounded;
- ambiguous `Content-Length` / `Transfer-Encoding` framing fails closed;
- unsupported `Expect`/upgrade paths fail before local-target allocation where intended;
- spoofable `X-I2P-*`, `Forwarded`, `X-Forwarded-*`, and related proxy identity headers are stripped according to the existing exact/prefix policy;
- trusted I2P identity is injected only from the authenticated peer object;
- response fingerprint headers remain stripped;
- request-body relay has a finite progress/lifetime bound (currently expected to be the existing approximately 60-second relay deadline);
- POST limiter state remains hard bounded and fails closed when capacity is exhausted;
- peer keys use canonical Destination identity rather than attacker-selected text.

The absence of a request-body byte cap is **not automatically a defect** if the finite relay deadline and global admission already bound occupancy/memory and no new evidence demonstrates an unbounded allocation path. Likewise, Sybil pressure against the bounded POST map is not automatically grounds for a new fairness algorithm.

If M089 finds a concrete high/medium HTTP defect, open a new plan. Do not add a byte cap, token bucket, randomized delay, or replacement limiter inside M089.

### 5.4 IRC server

Verify:

- bounded registration line/count/deadline behavior;
- trusted peer-derived presentation;
- no local target allocation before required registration validation;
- five-second target connect bound;
- ten-minute activity-resetting established-session inactivity expiry;
- unsupported DCC/CTCP filtering remains bounded and fail-closed where intended;
- post-registration relay remains raw.

### 5.5 Streamr server

Re-evaluate the known bounded subscriber model against the pinned Java I2P and I2P+ reference implementations.

Expected current behavior to verify:

- loopback-only UDP local boundary;
- maximum ten subscribers;
- 60-second subscriber expiry;
- approximately 15-second refresh behavior;
- bounded control/payload/receive sizes;
- bounded sequential fanout.

Explicitly record the Sybil-monopolization property: multiple attacker Destinations can occupy/refesh the finite subscriber set. Because the Java I2P and I2P+ reference implementations use the same ten-subscriber/60-second model, this is currently a reference-aligned specialized availability limitation, not by itself a fork regression.

Do not invent random eviction or a new public authentication/allowlist mechanism in M089. If the intended deployment threat model requires stronger public Streamr admission, create a separate plan with compatibility evidence.

## 6. Cross-cutting anonymity checks

Verify no tunnel family introduces:

- arbitrary non-loopback server targets where the contract expects local service forwarding;
- trust in spoofable application headers for remote identity;
- clearnet IP/address substitution for I2P Destination identity;
- private Destination/key material in `Debug`, `Display`, status, or sanitized runtime errors;
- request-controlled filesystem paths for persistent server identity;
- cross-generation stale peer/admission state;
- locks held across network I/O/sleeps/joins;
- unbounded handler/task spawning;
- hidden persist-and-ignore runtime options.

For `httpbidirserver`, retain the established fork-local security decision around its local proxy/session composition unless Proposal 170 or a pinned interoperability requirement proves it wrong. Do not change identity-sharing semantics inside M089.

## 7. Explicitly deferred/non-blocking hardening questions

Unless new implementation evidence materially changes their severity, record rather than implement:

- process-wide admission budgets shared across multiple independently configured server tunnels;
- replacing fixed minute/hour/day windows with token-bucket or GCRA accounting;
- randomized rejection timing/jitter/padding;
- global Sybil-resistant identity economics;
- generic zeroization work unrelated to a demonstrated secret-lifetime defect;
- portability hardening outside the Proposal 170 runtime path.

These are not M089 closure blockers merely because they are theoretically possible improvements.

## 8. Containment review

Compare the M087/M088 changed paths against M061/M062/M063 authority.

The reclosure MUST identify every production path outside `emissary-cli/src/i2pcontrol/**` touched by either corrective, explain why it was strictly required, and treat any unexplained broad change as a closure blocker.

Expected result:

- M087 entirely inside `i2pcontrol` plus planning/test bookkeeping;
- M088 either entirely inside `i2pcontrol`, or evidence-only if the necessary lower-layer capability would require an unapproved dependency/core change.

Any dependency, root manifest, lockfile, `emissary-core/**`, router, startup, or frontend change must have its own explicit authority and cannot be silently blessed by M089.

## 9. Required verification

Use proportional but complete current-head verification.

At minimum run:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Also perform targeted source inspection for:

- accepted-server order and M088 disposition;
- generic relay inactivity semantics;
- HTTP request/body/POST bounds;
- IRC lifetime bounds;
- Streamr subscriber/fanout bounds;
- server-secret redaction/persistence safety;
- changed-path containment.

Do not add hosted CI/fuzz/soak/public-network machinery merely to close M089.

## 10. Acceptance criteria

M089 may close only when:

1. M087 and M088 have accepted closure records;
2. generic server indefinite zero-progress slot pinning is corrected;
3. M088's pre-accept/lower-layer status is technically accurate and not overstated;
4. shared application admission remains bounded and transactional;
5. HTTP server family retains bounded parsing/body relay/POST state and trusted identity filtering, with no newly demonstrated high/medium defect;
6. IRC retains bounded registration/connect/inactivity behavior;
7. Streamr bounded behavior matches the pinned reference model, with Sybil monopolization explicitly documented rather than hidden;
8. all twelve registered tunnel types remain truthful to their supported runtime behavior;
9. no direct identity leak or new high/medium active-correlation/resource-exhaustion defect remains within the approved Proposal 170 boundary;
10. containment review finds no unexplained production changes outside `i2pcontrol`;
11. the full `emissary-cli` i2pcontrol test command and M062 pass;
12. `git diff --check` passes;
13. no upstream interaction occurred.

If any high/medium production defect is found, M089 does not fix it. Open a new numbered corrective and keep M089 blocked.

## 11. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/089-closure.md` containing:

- baseline and reviewed head SHA;
- M087/M088 closure references;
- server-family threat/evidence matrix;
- exact M088 lower-layer disposition;
- HTTP residual-risk disposition;
- Streamr reference-parity/Sybil-risk disposition;
- changed-path containment matrix;
- full test commands and results;
- unresolved findings with severity and owner/disposition;
- statement of which low-priority hardening questions remain intentionally deferred;
- final registry/roadmap disposition;
- explicit internal-only/no-upstream-interaction attestation.

## 12. Final disposition

If M089 closes without a new high/medium finding, it supersedes M085 only as the **current-head tunnel runtime/security reclosure authority**. M085 remains valid historical evidence for its pinned head.

Proposal 170 may still remain partial for the separately tracked source/truthfulness limitations, RouterInfo 37/1/5 disposition, M051 blocker, and unrelated AddressBook/base-I2PControl gaps. M089 does not reopen those workstreams.
