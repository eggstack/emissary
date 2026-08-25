# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; tunnel runtime/security closed by M085; M086 documentation/evidence reconciliation ready

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

Current M086 planning baseline: `185d43174c491a57c217c39e45555d136f40a406`.

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

M086 is stricter: it authorizes no production-source or Cargo/dependency change at all.

## Current handoff

Exactly one implementation plan is dependency-ready:

- `086-post-m085-documentation-and-evidence-reconciliation-corrective.md` — **ready**.

M086 corrects stale current-state planning/support text and closure-record errata after M085. It does **not** reopen M085 or authorize another runtime/security pass.

After M086 closes, no tunnel-security successor should be registered unless new production evidence appears.

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
| M077 | implementation/closure present; merged-head integration reconciled by M084 | IRC connect/idle hardening |
| M078 | implementation/closure present; merged-head integration reconciled by M084 | Streamr loopback/fanout hardening |
| M079 | historical closure only for older branch lineage | integrated tunnel-security reclosure before later M083 merge |
| M080 | closed; corrective history | admission transactionality/cardinality |
| M081 | closed | generic `leaseSetEncType` apply-or-reject |
| M082 | closed; corrective history | HTTP peer identity / `Expect` / POST key |
| M083 | closed | admission capacity semantics + exact/canonical trusted Destination |
| M084 | closed | merged-head integration/planning corrective |
| M085 | closed; current-head final runtime/security authority | independent merged-head tunnel-security reclosure |
| M086 | ready; documentation/evidence only | post-M085 record reconciliation; no production runtime change |

## Why M084-M086 exist

The repository merged two independently verified histories: the M077/M078/M079 line and the later M080-M083 admission/identity line.

M084 repaired the actual merge-composition failures, including stale test integration, dropped HTTP identity-header helper definitions, and exact M062 planning-path bookkeeping. M085 then independently audited the post-M084 head and closed the runtime/security workstream with no high/medium finding.

A later documentation audit found only residual record-quality defects:

- stale pending/reopened status wording in active planning documents;
- stale trusted-peer parser/canonicalization wording in user-facing support documentation;
- a closure-document arithmetic error for `MAX_PEER_ENTRIES`;
- a need to clarify that M084's HTTP helper restoration changed a production source file only to restore the already-intended merged behavior.

M086 owns those record-quality defects. It does not reopen runtime/security closure.

## Current corrective sequence

| Handoff | Status | Scope | Dependency |
|---|---|---|---|
| M084 | closed | repair merged-head integration/planning failures | merged-head findings |
| M085 | closed | independent actual post-M084 runtime/security reclosure | M084 closed |
| M086 | ready | reconcile stale planning/support text and transparent closure errata; exact M062 planning-path bookkeeping only | M085 closed |

Per `plans/003-planning-process.md`, only the next dependency-ready plan is registered `ready`.

## Security-critical family rules retained

### Accepted server admission

Accepted-stream server families derive trusted identity from Yosemite, require exactly one canonicalizable supported Destination, apply bounded transactional admission before handler/local-target work, and keep peer/rate/expiry structures hard bounded.

Historical peer-rate state exists only when configured semantics require it. Capacity is proven against all enabled aggregate windows with fixed-window overlap. No-history inactive peers are removed on final lease drop.

### Trusted peer identity

The current M083/M085 boundary is:

- bounded Base64 text input;
- one decode;
- `Destination::parse_frame`;
- empty parser remainder required;
- 32-byte `parsed.id()` for accounting;
- canonical full-Destination text from Base64-encoding `parsed.serialize()`.

M086 updates user-facing documentation to state this exact behavior; it does not change implementation.

### Generic server

Control-plane generic `server` remains accepted-stream/raw-relay. `leaseSetEncType` is applied when accepted; other runtime-relevant fields are applied or rejected before allocation.

### HTTP server

`httpserver` and inbound `httpbidirserver` use the shared accepted-stream identity/admission path. Request framing stays fail-closed, spoofed I2P/proxy identity is stripped, trusted full-Destination text is canonical, response fingerprints are stripped, POST accounting uses the canonical Destination ID, and unsupported `Expect` requests fail before local target allocation.

### IRC

`ircserver` retains bounded registration and trusted peer-derived presentation, five-second local-target connect, and ten-minute activity-resetting post-registration idle expiry. Post-registration bytes remain raw.

### Streamr

Streamr remains a bounded datagram subsystem with loopback-only local endpoints, ten subscribers, 60-second expiry, 15-second refresh, one-byte controls, 1200-byte application payload cap, 4095-byte receive bound, and bounded sequential fanout.

## Containment authority

M061 remains the source-path authority. M062 plus M063 remain the dependency/feature-ownership authority.

M086 may add only its exact implementation/closure paths to the M062 test allowlist. It may not broaden production path globs or dependency/feature ownership.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate.

## Verification discipline for M086

Use only proportional evidence:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
git diff --name-only <M086-baseline>..HEAD
```

No full runtime/security rerun is required if M086 obeys its no-production-change scope. If a production change becomes necessary, stop M086 and create separate corrective planning.

## Final status rule

M085 remains the final runtime/security closure authority throughout M086.

M086 may close only the residual documentation/evidence-integrity corrective. After M086 closes, the tunnel runtime/security line remains closed and no active tunnel-security handoff should remain.

Proposal 170 remains separately partial for the accepted source/truthfulness limitations, RouterInfo 37/1/5, M051, and unrelated AddressBook/base-I2PControl gaps.

No upstream review or acceptance is implied or authorized.
