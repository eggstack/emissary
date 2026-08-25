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
| I2PControl Proposal 170 containment | accepted/closed authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | consumed by M086 only for exact planning-path bookkeeping | M061/M062/M063 semantics remain authoritative; no production containment corrective is open |
| I2PControl Proposal 170 tunnel runtime completion | runtime complete; M085 security reclosure accepted | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no runtime handoff | M072 historical runtime evidence remains; M080-M085 corrections/reclosure are accepted at current head |
| I2PControl Proposal 170 tunnel security hardening | runtime/security closed by M085; documentation/evidence corrective closed by M086 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | no active tunnel-security handoff | M085 remains final runtime/security closure authority; M086 closed the record-quality corrective without reopening runtime behavior |

## Canonical scope amendment for tunnel runtimes

ADR-0003 remains the controlling scope amendment for the ten Proposal 170 tunnel data planes. ADR-0001/ADR-0002 remain controlling for contract spelling, startup/control-plane separation, server secret ownership, and internal-only scope except where ADR-0003 explicitly superseded earlier data-plane deferment.

The preferred production boundary remains `emissary-cli/src/i2pcontrol/**`. M086 authorizes no production-source change, no new `emissary-core/**` path, no dependency, no feature widening, no router/startup refactor, and no Proposal 170 wire expansion.

No upstream review, merge, submission, contribution preparation, issue/PR mutation, or maintainer contact is authorized. External I2P/I2P+/Yosemite repositories and specifications are read-only evidence only. Repository writes remain internal to `eggstack/emissary`.

## Resolved merged-head security history

The two-parent merge at `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae` combined the older M077/M078/M079 lineage with the later M083 admission/trusted-Destination lineage.

M084 repaired the merge-integration defects and M085 independently audited the actual post-M084 head. M085 closed the tunnel runtime/security workstream with no high/medium security, anonymity, correctness, lifecycle, option-truthfulness, or containment finding remaining.

M079 remains historical older-lineage evidence only. M085 supersedes it as current-head final runtime/security closure authority.

A later documentation audit found only record-quality defects: stale status text, a stale trusted-peer parser description, one M085 capacity-arithmetic transcription error, and a need to clarify M084's bounded production HTTP-helper merge restoration. These are owned by M086 and do not invalidate M085.

## Dependency-ready implementation plan

No implementation plan is currently registered as dependency-ready under this
closed tunnel-security cleanup line:

M086 — post-M085 documentation and evidence reconciliation is **closed** in
`plans/closure/i2pcontrol-proposal-170/086-closure.md`. It reconciled stale
current-state planning/support text and closure errata without changing
production runtime behavior or reopening M085.

Per `plans/003-planning-process.md`, only the next dependency-ready implementation plan is registered `ready`.

## Current tunnel-security sequence

| Handoff | Current disposition | Plan | Dependency / note |
|---|---|---|---|
| M074 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md` | M080/M083 own later admission corrections |
| M075 | closed | `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md` | M081 repaired `leaseSetEncType` truthfulness |
| M076 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md` | M082/M083 own later peer-identity/Expect corrections |
| M077 | implementation/closure present; merged-head integration reconciled | `plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md` | M084 repaired stale merged test integration |
| M078 | implementation/closure present; merged-head integration reconciled | `plans/implementation/i2pcontrol-proposal-170/078-streamr-local-boundary-hardening.md` | M084 repaired merged containment bookkeeping |
| M079 | historical closure only; superseded for current-head certification | `plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md` | retained historical evidence; M085 is current-head authority |
| M080 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/080-server-admission-transactionality-and-cardinality-corrective.md` | M083 closes remaining capacity/expiry semantics |
| M081 | closed | `plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md` | retained at current head |
| M082 | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md` | M083 closes inherited exact-Destination gap |
| M083 | closed | `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md` | exact/canonical trusted identity and admission capacity semantics accepted |
| M084 | closed | `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md` | merged-head integration corrective accepted |
| M085 | closed | `plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md` | final current-head runtime/security reclosure authority |
| M086 | closed; documentation/evidence only | `plans/implementation/i2pcontrol-proposal-170/086-post-m085-documentation-and-evidence-reconciliation-corrective.md` | corrected stale planning/support text and closure errata; no runtime change; see `plans/closure/i2pcontrol-proposal-170/086-closure.md` |

## Durable tunnel-security invariants

The closed M074-M085 runtime/security workstream establishes and preserves:

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

M086 may correct only the records describing these invariants.

## Containment authority

Accepted authorities remain:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` plus `m061_containment.rs`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` plus the M063-strengthened `m062_dependency_containment.rs`.

M086 may add only its exact implementation/closure paths to the M062 planning allowlist. It may not broaden production globs or ownership.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate and must be documented truthfully.

## Verification policy for M086

Keep verification proportional to its documentation/evidence-only scope:

- M062 exact-path containment test;
- changed-path review proving no production source/manifests/lockfile changed;
- targeted stale-text inspection;
- `git diff --check`.

Do not rerun the full runtime/security suite merely to correct documentation. Any need for a production-source change stops M086 and requires a new narrow corrective.

## Registry maintenance rules

1. M086 was the sole ready handoff during implementation; it is now closed.
2. M085 remains the final runtime/security closure authority throughout M086.
3. M086 does not reopen M077-M085 production behavior.
4. After M086 closes, no tunnel-security successor should be registered unless new implementation evidence appears.
5. Preserve RouterInfo 37/1/5 and M051 unless separate source-owner work changes them.
6. Preserve ADR-0003 and the preferred `emissary-cli/src/i2pcontrol/**` production boundary.
7. Unsupported/underspecified runtime options fail before allocation; persist-and-ignore is forbidden.
8. External sources remain read-only; no upstream interaction is authorized.
9. All writes remain internal to `eggstack/emissary`.
