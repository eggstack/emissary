# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies and interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency or evidence requirement prevents progress.
- **closing** — implementation landed and independent closure evidence is being gathered.
- **closed** — closure record accepted for the pinned implementation head.
- **closed internally against pinned revision** — internal closure accepted against an explicitly named revision of an open external specification; does not imply upstream review or acceptance.
- **partial Proposal 170 support** — exact supported dimensions are closed, but one or more pinned source/runtime capabilities remain truthfully unavailable.
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, merge-integration, security, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 source/truthfulness | partial Proposal 170 support; M057 closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | no source-completion handoff | M051 remains blocked by absent substantive news/ban owners; accepted RouterInfo matrix remains 37/1/5 |
| I2PControl Proposal 170 containment | accepted/closed authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | exact-path bookkeeping only as later plans are registered | M061/M062/M063 semantics remain authoritative; no production containment corrective is open |
| I2PControl Proposal 170 tunnel runtime completion | runtime complete; security corrective line reopened | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no runtime-feature handoff | M072 historical runtime evidence remains; M087-M089 are security/lifetime corrections, not new tunnel-type work |
| I2PControl Proposal 170 tunnel security hardening | corrective pass in progress after post-M086 active-adversary review | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | **M088 ready** | M087 closed; M089 requires accepted M088 closure |

## Canonical scope amendment for tunnel runtimes

ADR-0003 remains the controlling scope amendment for the ten Proposal 170 tunnel data planes. ADR-0001/ADR-0002 remain controlling for contract spelling, startup/control-plane separation, server secret ownership, and internal-only scope except where ADR-0003 explicitly superseded earlier data-plane deferment.

The preferred production boundary remains `emissary-cli/src/i2pcontrol/**`. The M087-M089 corrective sequence does not authorize broad `emissary-core/**`, router, startup, frontend, dependency, or Proposal 170 wire/API expansion.

No upstream review, merge, submission, contribution preparation, issue/PR mutation, or maintainer contact is authorized. External I2P/I2P+/Yosemite repositories and specifications are read-only evidence only. Repository writes remain internal to `eggstack/emissary`.

## Corrective trigger after M086

M085 remains valid closure evidence for its pinned post-M084 head, and M086 remains valid documentation/evidence reconciliation for its scope. A later active-adversary review identified two additional server-side hardening concerns that were not fully addressed by those milestones:

1. generic `server` accepted streams use a five-second local-target connect timeout but then an unbounded raw `copy_bidirectional` relay, so several Sybil Destinations can hold the finite shared admission pool indefinitely without useful byte progress;
2. common accepted-server admission is applied only after Yosemite/SAM `session.accept()` returns, so application concurrency/rate denial does not necessarily bound lower-layer stream-establishment work.

The same review did **not** justify immediate new HTTP or Streamr production semantics:

- HTTP already has bounded request/header parsing, a finite request-body relay deadline, bounded fail-closed POST accounting, trusted peer identity, and spoof/fingerprint stripping;
- Streamr's ten-subscriber / 60-second expiry model matches Java I2P and I2P+ reference behavior, although Sybil monopolization remains a documented specialized availability limitation.

Those families are therefore rechecked in M089 rather than changed preemptively.

## Dependency-ready implementation plan

Exactly one tunnel-security implementation handoff is registered ready:

- **M088 — Pre-Accept Server Admission Boundary Corrective**: `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md`.

M089 remains future/blocked until M088 closes so the repository does not expose multiple competing executable handoffs.

## Current tunnel-security sequence

| Handoff | Current disposition | Plan | Dependency / note |
|---|---|---|---|
| M074 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md` | M080/M083 own later application-admission corrections |
| M075 | closed; later security lifetime corrective opened | `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md` | M087 is the separate compatibility-evidenced inactivity correction anticipated by M075 |
| M076 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md` | HTTP residuals rechecked by M089; no production change currently authorized |
| M077 | closed | `plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md` | retained by M085; rechecked by M089 |
| M078 | closed | `plans/implementation/i2pcontrol-proposal-170/078-streamr-local-boundary-hardening.md` | reference-aligned bounded Streamr behavior retained; rechecked by M089 |
| M079 | closed historical record; superseded for current-head certification | `plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md` | retained historical evidence |
| M080 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/080-server-admission-transactionality-and-cardinality-corrective.md` | retained by M083/M085 |
| M081 | closed | `plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md` | retained |
| M082 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md` | retained by M083/M085 |
| M083 | closed | `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md` | current application-admission and trusted-Destination authority |
| M084 | closed | `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md` | merged-head integration corrective accepted |
| M085 | closed; historical current-head authority for its pinned head | `plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md` | superseded as current-head authority only if M089 later closes |
| M086 | closed; documentation/evidence only | `plans/implementation/i2pcontrol-proposal-170/086-post-m085-documentation-and-evidence-reconciliation-corrective.md` | record reconciliation; no runtime change |
| M087 | closed | `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md` | progress-based inactivity bound implemented and independently tested; see M087 closure |
| M088 | **ready** | `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md` | M087 administrative gate satisfied; feasibility-gated lower-layer admission mapping/hardening |
| M089 | future/blocked | `plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md` | requires accepted M088 closure; verification only |

Per `plans/003-planning-process.md`, only the next dependency-ready implementation plan is registered `ready`.

## M087 scope guard

M087 may change only the generic accepted-server relay lifetime semantics needed to prevent indefinite zero-progress slot pinning. It must use an inactivity/progress timeout, not an absolute connection lifetime, and should stay in `emissary-cli/src/i2pcontrol/backends/server.rs` plus focused tests/planning bookkeeping.

It does not authorize a new Proposal 170 field, dependency, core/router/startup/frontend change, new protocol parser, or upstream work.

## M088 scope guard

M088 first maps the actual Emissary/Yosemite/SAM lower-layer boundary. Java I2P streaming option names are evidence, not assumed Emissary capabilities.

If a supported lower-layer bound is already exposable through the current dependency/API, M088 may wire the smallest translation from existing server admission policy. If progress requires vendoring/forking/patching Yosemite, broad `emissary-core/**` changes, a new router/streaming algorithm, or a parallel SAM implementation, M088 stops and records the limitation or launches a separately approved narrow dependency-boundary plan. It must not widen itself automatically.

The existing post-accept `ServerAdmissionState` remains mandatory defense in depth regardless of M088 outcome.

## M089 scope guard

M089 is an independent reclosure and authorizes no production code changes. It rechecks all twelve tunnel types and explicitly dispositions:

- generic-server lifetime hardening;
- lower-layer/pre-accept admission status;
- HTTP bounded body/POST behavior without inventing a byte cap or fairness scheme absent a concrete defect;
- Streamr reference parity and known Sybil subscriber-monopolization limitation;
- containment of all corrective changes.

Any new high/medium production defect creates another numbered plan and keeps M089 blocked.

## Durable tunnel-security invariants retained

The prior M074-M086 work remains authoritative for:

- exact Proposal 170 contract spelling/types/actions;
- authenticated SAM/Yosemite accepted-peer identity;
- exactly one parsed supported Destination with zero remainder and canonical downstream B64 text;
- 32-byte cryptographic Destination ID for admission/POST accounting;
- transactional, bounded application admission with explicit peer-history semantics;
- no-history inactive peer reclamation and coherent inactive-only expiry index;
- HTTP spoof/fingerprint/framing/Expect/POST protections;
- IRC bounded registration, five-second target connect, ten-minute activity-resetting idle expiry, raw post-registration relay;
- Streamr loopback-only local boundary, ten subscribers, bounded expiry/refresh/control/payload/fanout;
- generation-local ephemeral state and bounded stop/restart ownership;
- no private destination material in diagnostics;
- no timing-jitter theater or public-network deanonymization testing;
- no upstream interaction.

M087-M089 add or re-evaluate only the corrective properties explicitly named above.

## Containment authority

Accepted authorities remain:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` plus `m061_containment.rs`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` plus the M063-strengthened `m062_dependency_containment.rs`.

Registering M087-M089 requires only exact implementation/closure path bookkeeping in M062. Do not broaden production globs or dependency/feature ownership as part of planning.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate and must be documented truthfully.

## Registry maintenance rules

1. M087 is the sole ready tunnel-security handoff.
2. M088 remains future until M087 closes; this is administrative sequencing, not a claim of technical dependency.
3. M089 remains future/blocked until M087 and M088 have accepted closure records and any M088 dependency-boundary blocker is resolved or explicitly accepted.
4. M085 remains valid historical reclosure evidence for its pinned head until M089 closes.
5. Preserve RouterInfo 37/1/5 and M051 unless separate source-owner work changes them.
6. Preserve ADR-0003 and the preferred `emissary-cli/src/i2pcontrol/**` production boundary.
7. Unsupported/underspecified runtime options fail before allocation; persist-and-ignore is forbidden.
8. External sources remain read-only; no upstream interaction is authorized.
9. All writes remain internal to `eggstack/emissary`.
