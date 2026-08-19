# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; tunnel-security merge corrective active; M084 next

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative planning references:

- `plans/000-long-term-specification.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`

Pinned Proposal 170 revision: `2026-05-20`.

Current merged-head planning baseline: `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae`.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, I2P/I2P+ reference implementations, Yosemite source, issues, and pull requests are read-only evidence. No plan authorizes upstream submission, review request, maintainer contact, contribution preparation, merge request, or repository write outside this fork.

## Scope and containment

ADR-0003 authorizes the bounded Proposal 170 tunnel data planes while preserving established containment and startup/control-plane ownership.

The implementation target remains:

- keep runtime/filter/admission policy in `emissary-cli/src/i2pcontrol/**` wherever technically possible;
- avoid new `emissary-core/**` production changes;
- apply or reject every runtime-relevant option before allocation;
- keep persistent server secrets backend-owned and redacted;
- keep startup-managed tunnel behavior separate from Proposal 170 TunnelManager ownership;
- avoid hosted CI/fuzz/soak/release machinery for this bounded workstream.

## Current handoff

Exactly one plan is dependency-ready:

- `084-merged-head-integration-and-planning-corrective.md` — **ready**.

The final independent reclosure plan is prewritten but blocked:

- `085-merged-head-tunnel-security-reclosure.md` — **blocked on M084**.

## Why the final corrective sequence exists

Current `master` combines two independently verified histories:

- the M077 IRC / M078 Streamr / M079 final-reclosure lineage; and
- the later M083 admission/trusted-Destination corrective lineage.

M079 did not audit the later merged composition. The merge also retained:

- an IRC test that still calls removed `TrustedPeerIdentity::for_test` instead of an M083 structurally valid Destination fixture;
- M062 planning-path bookkeeping that omits the merged M077/M078/M079 closure files;
- contradictory registry/roadmap/support status about which milestones are ready, blocked, or closed.

M084 repairs only those integration/status defects. M085 then independently audits the actual post-M084 head.

## Historical runtime/security sequence

| Handoff | Current disposition | Scope |
|---|---|---|
| M064 | closed | baseline feature-disabled corrective |
| M065 | closed | I2PControl client/accepted-server runtime primitives |
| M066 | closed | IRC client/server family |
| M067 | closed | HTTP server family |
| M068 | closed | HTTP client + CONNECT |
| M069 | closed | SOCKS + SOCKS-IRC |
| M070 | closed | HTTP bidirectional server composition |
| M071 | closed | Streamr client/server |
| M072 | historical runtime reclosure | integrated twelve-type runtime audit |
| M073 | closed; corrective history | generic option truthfulness |
| M074 | closed; corrective history | shared server admission/rate hardening |
| M075 | closed | generic accepted-stream migration |
| M076 | closed; corrective history | HTTP anonymity/POST hardening |
| M080 | closed; corrective history | admission transactionality/cardinality |
| M081 | closed | generic `leaseSetEncType` apply-or-reject |
| M082 | closed; corrective history | HTTP peer identity / `Expect` / POST key |
| M083 | closed | admission capacity semantics + exact/canonical trusted Destination |
| M077 | implementation/closure present; merged-head integration pending | IRC connect/idle hardening |
| M078 | implementation/closure present; merged-head integration pending | Streamr loopback/fanout hardening |
| M079 | historical closure only for older branch lineage | integrated tunnel-security reclosure before later M083 merge |

## Current corrective sequence

| Handoff | Status | Scope | Dependency |
|---|---|---|---|
| M084 | ready | repair stale IRC test API usage, exact M062 planning/closure bookkeeping, and contradictory status docs; no runtime semantic changes | current merged-head findings |
| M085 | blocked | independently re-audit actual post-M084 merged head and close only with no high/medium finding | M084 closed |

Per `plans/003-planning-process.md`, only the next dependency-ready plan is registered `ready`.

## Security-critical family rules retained

### Accepted server admission

Accepted-stream server families must derive trusted identity from Yosemite, require exactly one canonicalizable supported Destination, apply bounded transactional admission before handler/local-target work, and keep every peer/rate/expiry structure hard bounded.

Historical peer-rate state must exist only when enabled semantics require it. Capacity must be proven against all enabled aggregate windows, including fixed-window overlap. No-history inactive peers are removed on final lease drop.

### Generic server

Control-plane generic `server` remains accepted-stream/raw-relay. It may not return to `STREAM FORWARD`. `leaseSetEncType` must be applied when accepted; other runtime-relevant fields are applied or rejected before allocation.

### HTTP server

`httpserver` and inbound `httpbidirserver` use the shared accepted-stream identity/admission path. Request framing stays fail-closed, spoofed I2P/proxy identity is stripped, trusted full-Destination text is canonical, response fingerprints are stripped, POST accounting uses the canonical Destination ID, and unsupported `Expect` requests fail before local target allocation.

### IRC

`ircserver` retains bounded registration and trusted peer-derived presentation, five-second local-target connect, and ten-minute activity-resetting post-registration idle expiry. Post-registration bytes remain raw. M084 changes only the stale test fixture wiring.

### Streamr

Streamr remains a small bounded datagram subsystem with loopback-only local endpoints, ten subscribers, 60-second expiry, 15-second refresh, one-byte controls, 1200-byte application payload cap, 4095-byte receive bound, and bounded sequential fanout.

## Containment authority

M061 remains the source-path authority. M062 plus M063 remain the dependency/feature-ownership authority.

M084 may make exact-path planning/closure bookkeeping updates in the M062 test but may not broaden production path globs. M085 independently verifies the final containment state.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate.

## Verification discipline

Use focused deterministic local tests, structurally valid I2P Destination fixtures, fake/local SAM/TCP/UDP services, Tokio paused-time tests, package-scoped checks, M061/M062 containment tests, strict Clippy, scoped nightly rustfmt, and `git diff --check`.

Do not add public-network certification/deanonymization tests, broad platform matrices, hosted CI, release machinery, generalized fuzzing, or soak farms merely for this workstream.

## Final status rule

The tunnel runtime/security line is not closed until:

1. M084 closes the current merged-head integration/planning defects; and
2. M085 independently accepts the actual post-M084 repository head with no high/medium security, anonymity, correctness, lifecycle, option-truthfulness, or containment finding.

If M085 closes, the tunnel runtime/security line is complete. Proposal 170 remains separately partial for the accepted source/truthfulness limitations.

No upstream review or acceptance is implied or authorized.
