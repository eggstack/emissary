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
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 source/truthfulness | partial Proposal 170 support; M057 closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | no source-completion handoff | M051 remains blocked by absent substantive news/ban owners; accepted RouterInfo matrix remains 37/1/5 |
| I2PControl Proposal 170 containment | closed | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | no containment corrective handoff | M061 source containment and M062/M063 dependency containment remain accepted authorities |
| I2PControl Proposal 170 tunnel runtime completion | historical runtime completion accepted; current security closure reopened | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no separate runtime handoff | M072 historical runtime closure remains; M081 restored the M073 generic-server option truthfulness invariant |
| I2PControl Proposal 170 tunnel security hardening | corrective pass required | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | M083 — admission capacity semantics and trusted Destination exactness | post-M082 review found remaining M080 capacity/expiry defects plus non-exact trusted-Destination acceptance inherited by HTTP |

## Canonical scope amendment for tunnel runtimes

ADR-0003 remains the controlling scope amendment for the ten Proposal 170 tunnel data planes. ADR-0001/ADR-0002 remain historical/controlling for contract spelling, startup/control-plane separation, server secret ownership, and internal-only scope except where ADR-0003 explicitly superseded the earlier deferment of those data planes.

The preferred production boundary is `emissary-cli/src/i2pcontrol/**`. M065-M083 do not authorize a new `emissary-core/**` production path. M064 remains the narrow historical exception for an already accepted event-observation setter repair.

No upstream review, merge, submission, contribution preparation, issue/PR mutation, or maintainer contact is authorized. External I2P/I2P+ repositories and specifications are read-only evidence only. Repository writes remain internal to `eggstack/emissary`.

## Dependency-ready implementation plan

No implementation plan is currently registered as dependency-ready:

| Handoff | Status | Plan | Objective |
|---|---|---|---|
| M083 — admission capacity semantics and trusted Destination exactness corrective | ready | `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md` | repair minute/no-history representability, choose the true tightest aggregate capacity bound with fixed-window overlap, preserve expiry-index invariants for active peers, and require exactly one canonical trusted Destination |

Per `plans/003-planning-process.md`, only the next dependency-ready implementation plan is registered `ready`.

## Current tunnel-security sequence

| Handoff | Status | Plan | Dependency / blocker |
|---|---|---|---|
| M074 — shared server admission/rate hardening | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md` | M080 corrected its original transactionality/cardinality defects; M083 addresses remaining M080 capacity semantics |
| M075 — generic server accepted-stream hardening | closed | `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md` | M081 closed the `leaseSetEncType` regression |
| M076 — HTTP anonymity/POST hardening | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md` | M082 closed the original valid-Destination-bound/`Expect`/POST-key defects; M083 revalidates exact canonical trusted identity inherited by HTTP |
| M080 — admission transactionality/cardinality corrective | corrective pass required at current head; historical closure retained for pinned commit `f07bf14` | `plans/implementation/i2pcontrol-proposal-170/080-server-admission-transactionality-and-cardinality-corrective.md` | M083 owns minute/no-history capacity semantics, tightest aggregate-bound derivation, and active-peer expiry-index consistency |
| M081 — generic server LeaseSet option truthfulness corrective | closed | `plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md` | closure accepted; no M083 finding reopens this invariant |
| M082 — HTTP peer identity and Expect-framing corrective | corrective pass required only for inherited trusted-Destination exactness; direct `Expect`/POST-key fixes retained | `plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md` | M083 owns zero-remainder/canonical full-Destination text; M082's fixed 417 and 32-byte POST key remain accepted |
| M083 — admission capacity and trusted Destination exactness corrective | ready | `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md` | current registered handoff |
| M077 — IRC server lifetime/exhaustion hardening | blocked | `plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md` | M083 must close first because IRC consumes the shared admission/trusted-peer boundary |
| M078 — Streamr local-boundary hardening | blocked | `plans/implementation/i2pcontrol-proposal-170/078-streamr-local-boundary-hardening.md` | M083 and M077 must close first |
| M079 — integrated tunnel-security reclosure | blocked | `plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md` | M083 plus M077-M078 must close; M079 independently re-audits the final head |

## Post-M082 corrective findings controlling M083

### Admission history/representability

M080 correctly made denial transactional, bounded the primary peer/expiry state, moved accounting to the 32-byte Destination ID, and replaced the original 4096-entry table with a documented hard memory ceiling. Those mechanisms are retained.

The remaining defect is semantic: `MINUTE` and `SHORT_RETENTION` are both 60 seconds, while capacity validation runs only for `retention > SHORT_RETENTION`. A minute-only peer-rate policy therefore skips the unrepresentable-unlimited-aggregate check. The same duration comparison also conflates real peer history with no-history cleanup.

M083 separates these concepts explicitly. When peer-rate history is required, an unbounded aggregate policy must fail unless exact bounded representation is otherwise proven. When no peer-rate history is required, inactive peer records must not linger solely for an arbitrary cleanup interval.

### Aggregate capacity solver

All enabled aggregate minute/hour/day counters are enforced conjunctively, but the current capacity helper selects the first non-zero field. M083 instead computes a conservative retained-event bound for every enabled aggregate window, includes fixed-window boundary overlap, and selects the smallest safe bound. This prevents both under-budgeting and false rejection of configurations where hour/day is tighter than minute.

### Expiry-index invariant

M080's `BTreeMap<(Instant, PeerKey), ()>` remains the correct bounded direction. M083 repairs the transition where reaping an expired **active** peer can remove its authoritative expiry entry while retaining the peer record. The implementation must document whether all peers or only inactive peers are indexed and enforce that invariant across acquire/reap/drop.

### Trusted Destination exactness

The shared peer-identity helper currently calls the core convenience `Destination::parse`, which discards `parse_frame` remainder. A valid Destination followed by trailing bytes can therefore yield a trusted identity while the original textual representation survives into HTTP full-Destination metadata/access matching.

M083 stays I2PControl-local: require exactly one parsed Destination with no remainder, keep the 32-byte canonical ID, and derive downstream full-Destination text by canonical Base64 encoding of the parsed serialized Destination.

## Durable tunnel security boundary

- accepted server families use authenticated Yosemite peer identity before application handler/local-target work;
- trusted identity must decode to exactly one supported Destination; downstream text is canonical and accounting uses the 32-byte Destination ID;
- global and per-peer concurrency plus peer/aggregate rate state must be bounded and denial must not leave attacker-owned state;
- historical peer-rate state exists only as long as enabled semantics require it;
- no-history inactive peers do not accumulate attacker-controlled table occupancy;
- capacity proof considers all enabled aggregate windows and fixed-window boundary overlap;
- every attacker-influenced auxiliary collection, including expiry indexes, is hard bounded with an explicit state invariant;
- generic control-plane `server` remains accepted-stream/raw-relay, not blind `STREAM FORWARD`;
- every runtime-relevant option is applied or rejected before allocation;
- HTTP spoofed identity/proxy metadata is stripped; response clock/server/provider/cache/trace fingerprints remain stripped;
- unsupported HTTP expectations fail before local allocation;
- IRC registration filtering remains before local connect; M077 adds activity-resetting post-registration idle expiry only after M083;
- Streamr remains bounded; M078 later makes local UDP ingress/output loopback-only with reference-aligned fanout;
- no timing-jitter theater, local-routing expansion, private secret leakage, core production widening, or upstream interaction.

## Containment authority

Accepted containment authorities remain:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` plus `m061_containment.rs` for source paths;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` plus the M063-strengthened `m062_dependency_containment.rs` for direct dependency ownership and transitive local-feature activation.

M083 requires no new dependency and authorizes no new `emissary-core/**` production path.

## Accepted unrelated Proposal 170 state

The RouterInfo source matrix remains exactly:

- 43 canonical Proposal 170 additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable: transit 15s, news, banned peers, and both v4/v6 network-error rows.

M051 remains blocked by absent substantive news/ban owners. Tunnel security work does not create those owners or reopen unrelated AddressBook/base-I2PControl limitations.

## Verification policy

Keep verification local/package-scoped and proportional:

- focused admission/trusted-peer/HTTP tests;
- structurally valid I2P Destination fixtures including trailing-byte negative cases;
- Tokio paused-time fixed-window/capacity/reap tests;
- fake/local SAM and local TCP capture services where needed;
- feature-disabled and feature-enabled `cargo check`;
- M061/M062/M063 containment suites;
- Clippy and scoped repository-accepted nightly rustfmt for touched files;
- `git diff --check`.

Do not add hosted CI jobs, release machinery, generalized fuzz infrastructure, soak farms, broad platform matrices, public-network deanonymization experiments, or upstream contribution machinery.

## Registry maintenance rules

1. M083 is the sole dependency-ready handoff.
2. M077 must remain blocked until M083 closes and reconciles M080/M082 current dispositions.
3. M078 remains behind M083 and M077.
4. M079 remains behind M083, M077, and M078 and is the final independent security reclosure authority.
5. Any high/medium finding discovered before or during M079 creates another narrow corrective; it may not be hidden inside closure.
6. Preserve ADR-0003 scope and the preferred `emissary-cli/src/i2pcontrol/**` production boundary.
7. Preserve RouterInfo 37/1/5 and the M051 blocker unless separate source-owner work changes them.
8. Unsupported/underspecified runtime options fail before allocation; persist-and-ignore is forbidden.
9. No artificial response jitter/fixed delays substitute for bounded resource ownership.
10. External sources are read-only only; no upstream interaction is authorized.
11. All repository writes remain internal to `eggstack/emissary`.
