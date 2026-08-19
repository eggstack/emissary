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
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, merge-integration, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 source/truthfulness | partial Proposal 170 support; M057 closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | no source-completion handoff | M051 remains blocked by absent substantive news/ban owners; accepted RouterInfo matrix remains 37/1/5 |
| I2PControl Proposal 170 containment | accepted authority; current merged-head bookkeeping corrective pending | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | consumed by M084 | M061/M062/M063 semantics remain authoritative; M084 repairs exact merged planning-path bookkeeping without broadening production scope |
| I2PControl Proposal 170 tunnel runtime completion | historical runtime completion accepted; security reclosure reopened by merge composition | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no separate runtime feature handoff | M072 historical runtime closure remains; M081/M083 retain current option/identity/admission corrections |
| I2PControl Proposal 170 tunnel security hardening | corrective pass required | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | M084 — merged-head integration/planning corrective | M079 closed an older branch lineage; current `master` later merged M083 and requires M084 then independent M085 reclosure |

## Canonical scope amendment for tunnel runtimes

ADR-0003 remains the controlling scope amendment for the ten Proposal 170 tunnel data planes. ADR-0001/ADR-0002 remain controlling for contract spelling, startup/control-plane separation, server secret ownership, and internal-only scope except where ADR-0003 explicitly superseded earlier data-plane deferment.

The preferred production boundary remains `emissary-cli/src/i2pcontrol/**`. M084-M085 do not authorize a new `emissary-core/**` production path, new dependency, router/startup refactor, or Proposal 170 wire expansion.

No upstream review, merge, submission, contribution preparation, issue/PR mutation, or maintainer contact is authorized. External I2P/I2P+/Yosemite repositories and specifications are read-only evidence only. Repository writes remain internal to `eggstack/emissary`.

## Current merged-head evidence problem

Current planning baseline: `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae`.

That head merges:

- the older M077/M078/M079 tunnel-security lineage, where M079 independently closed its then-final head; and
- the later M083 admission/trusted-Destination corrective lineage.

The merge created a repository state that no final closure independently audited. It also retained two concrete integration defects:

- M077's IRC admission-release test still references the removed arbitrary-string `TrustedPeerIdentity::for_test` helper instead of an M083 structurally valid fixture;
- M062's exact planning-path allowlist contains M083 closure bookkeeping but not the merged M077/M078/M079 closure paths.

Planning/status documents also disagree about whether M083, M077, M078, and M079 are ready/blocked/closed.

Historical M079 evidence is retained, but it is not current-head certification.

## Dependency-ready implementation plan

Exactly one plan is currently registered as dependency-ready:

| Handoff | Status | Plan | Objective |
|---|---|---|---|
| M084 — merged-head integration and planning corrective | ready | `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md` | repair stale IRC test fixture integration, exact M062 planning-path bookkeeping, and contradictory status documents without changing runtime semantics |

Per `plans/003-planning-process.md`, only the next dependency-ready implementation plan is registered `ready`.

## Current tunnel-security sequence

| Handoff | Current disposition | Plan | Dependency / note |
|---|---|---|---|
| M074 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md` | M080/M083 own later admission corrections |
| M075 | closed | `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md` | M081 repaired `leaseSetEncType` truthfulness |
| M076 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md` | M082/M083 own later peer-identity/Expect corrections |
| M080 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/080-server-admission-transactionality-and-cardinality-corrective.md` | M083 closes remaining current-head capacity/expiry semantics |
| M081 | closed | `plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md` | retained at current head |
| M082 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md` | M083 closes inherited exact-Destination gap |
| M083 | closed | `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md` | accepted for its own implementation lineage and present in current `master` |
| M077 | implementation/closure present; merged-head integration pending | `plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md` | runtime behavior present, but stale test helper must be reconciled by M084 |
| M078 | implementation/closure present; merged-head integration pending | `plans/implementation/i2pcontrol-proposal-170/078-streamr-local-boundary-hardening.md` | runtime behavior present; closure-path containment bookkeeping repaired by M084 |
| M079 | historical closure only; current-head certification invalidated by later merge | `plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md` | historical evidence retained; M085 supersedes it as current-head final reclosure authority |
| M084 | ready | `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md` | current sole ready handoff |
| M085 | blocked | `plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md` | hard dependency: M084 closed; then independently audit actual post-M084 merged head |

## Durable tunnel-security invariants

The remaining M084-M085 work must preserve:

- exact Proposal 170 contract spelling/types/actions;
- authenticated SAM/Yosemite accepted-peer identity;
- exactly one parsed supported Destination with zero remainder and canonical downstream B64 text;
- 32-byte cryptographic Destination ID for admission/POST accounting;
- transactional, bounded admission with explicit peer-history semantics and tightest safe aggregate capacity proof;
- no-history inactive peer reclamation and coherent inactive-only expiry index;
- generic accepted-stream raw relay and `leaseSetEncType` apply-or-reject;
- HTTP spoof/fingerprint/framing/Expect/POST protections;
- IRC bounded registration, five-second target connect, ten-minute activity-resetting idle expiry, raw post-registration relay;
- Streamr loopback-only local boundary, ten subscribers, bounded expiry/refresh/control/payload/fanout;
- generation-local ephemeral state and bounded stop/restart ownership;
- no core/router/startup/frontend scope expansion;
- no timing-jitter theater or public-network deanonymization testing;
- no upstream interaction.

## Containment authority

Accepted authorities remain:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` plus `m061_containment.rs`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` plus the M063-strengthened `m062_dependency_containment.rs`.

M084 may update only exact planning-path bookkeeping in M062. It may not broaden production globs or ownership. M085 must independently verify the final containment state.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate and must be documented truthfully.

## Verification policy

Keep verification local/package-scoped and proportional:

- focused admission/trusted-peer/generic-server/HTTP/IRC/Streamr tests;
- structurally valid I2P Destination fixtures and trailing-byte negatives;
- Tokio paused-time capacity/expiry/idle tests;
- fake/local SAM, TCP, and UDP fixtures;
- full feature-enabled I2PControl package tests;
- feature-disabled/enabled checks;
- M061/M062 containment;
- strict package Clippy;
- scoped nightly rustfmt for touched files;
- `git diff --check`.

Do not add hosted CI, release machinery, generalized fuzz/soak infrastructure, benchmark gates, or public-network certification/deanonymization tests.

## Registry maintenance rules

1. M084 is the sole ready handoff now.
2. M085 remains blocked until M084 independently closes.
3. M079 remains historical evidence and must not be rewritten to claim it audited M083/current merged head.
4. After M084 closure, M085 becomes the sole ready handoff.
5. M085 must audit the actual post-M084 head and may not trust branch-local closure assertions without current-head evidence.
6. Any high/medium finding in M084 outside integration scope or in M085 requires a new narrow corrective; do not hide it in closure.
7. Preserve ADR-0003 and the preferred `emissary-cli/src/i2pcontrol/**` production boundary.
8. Preserve RouterInfo 37/1/5 and M051 unless separate source-owner work changes them.
9. Unsupported/underspecified runtime options fail before allocation; persist-and-ignore is forbidden.
10. External sources remain read-only; no upstream interaction is authorized.
11. All writes remain internal to `eggstack/emissary`.
