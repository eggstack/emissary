# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; tunnel-security corrective sequence active; M080 next

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

Original M064-M072 planning baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`.

Current corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, I2P/I2P+ reference implementations, Yosemite source, issues, and pull requests are read-only evidence. No plan authorizes upstream submissions, review requests, maintainer contact, contribution preparation, or repository writes outside this fork.

## Scope and containment

ADR-0003 authorizes bounded implementation of the ten previously deferred Proposal 170 tunnel types while preserving established containment and startup/control-plane ownership rules.

The implementation target remains:

- keep runtime/filter/admission policy in `emissary-cli/src/i2pcontrol/**` wherever technically possible;
- avoid new `emissary-core/**` production changes;
- treat HTTP/IRC filtering and server admission as correctness/security behavior, not optional polish;
- apply or reject every runtime-relevant option before allocation;
- keep persistent server secrets backend-owned and redacted;
- keep startup-managed tunnel behavior separate from Proposal 170 TunnelManager ownership;
- avoid hosted CI/fuzz/soak/release infrastructure for this bounded workstream.

## Current handoff

Exactly one plan is dependency-ready:

- `077-irc-server-lifetime-and-exhaustion-hardening.md` — **ready**.

The independent post-M076 review reopened the current security disposition for M074-M076. M077 is now ready after the M080-M082 corrective sequence closed.

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
| M072 | historical runtime reclosure accepted after M073 | integrated twelve-type runtime audit |
| M073 | closed; corrective history | generic client/server option truthfulness; M081 closes the M075 accepted-but-ignored regression |
| M074 | closed; corrective history | shared server admission/rate hardening; M080 owns discovered defects |
| M075 | closed | generic server accepted-stream migration; M081 closes LeaseSet regression |
| M076 | closed | HTTP fingerprint/POST hardening; M082 closes follow-up defects |

## Corrective sequence required before final closure

| Handoff | Status | Scope | Dependency |
|---|---|---|---|
| M080 | closed | transactional/bounded admission state, canonical peer keys, capacity/retention coherence | independent review findings |
| M081 | closed | generic `leaseSetEncType` apply-or-reject after accepted-stream migration | sequencing behind M080 |
| M082 | closed | structural HTTP peer identity, `Expect` rejection, canonical POST peer key | M080 identity boundary; sequencing behind M081 |
| M077 | ready | IRC post-registration idle lifetime/connect bound | M080-M082 closed |
| M078 | blocked | Streamr loopback-only local UDP + reference-aligned fanout | M077 closed |
| M079 | blocked | independent integrated tunnel-security reclosure | M077-M078 closed |

Per `plans/003-planning-process.md`, future handoffs are prewritten for continuity but only the next dependency-ready plan is registered `ready`.

## Why M080-M082 were added

### M080

The M074 admission implementation can insert a fresh peer before aggregate-rate eligibility is known. Aggregate rejection may then leave an unexpiring zero-active record, allowing fresh identities to poison the bounded peer table. The auxiliary expiry queue is also not independently bounded, the fixed peer capacity is incoherent with long retained default windows, and accounting uses an eight-byte `DefaultHasher` key rather than the canonical I2P Destination ID.

### M081

M075 correctly migrated generic control-plane `server` from blind `STREAM FORWARD` to accepted streams, but the new accepted-stream configuration no longer carries `leaseSetEncType` while the backend still accepts it. M081 must apply it in Yosemite session setup or reject it before allocation.

### M082

M076's 524-character trusted-Destination limit is based on a legacy-sized I2P Destination and can reject valid larger current key-certificate/signature forms. M082 switches to structural Destination validation and a defensible current maximum. It also rejects unsupported `Expect: 100-continue` before local target allocation and moves POST accounting to canonical Destination IDs.

## Security-critical family rules

### Accepted server admission

All accepted-stream server families must derive trusted identity from Yosemite, apply bounded transactional admission before handler/local-target work, and keep every peer/rate/expiry structure hard bounded. A denied attempt must not leave attacker-owned accounting state.

### Generic server

Control-plane generic `server` remains accepted-stream/raw-relay. It may not return to `STREAM FORWARD`. Runtime-relevant options, including `leaseSetEncType`, must be applied or rejected.

### HTTP server

`httpserver` and inbound `httpbidirserver` must use the same application-visible accepted-stream filter path. Request framing remains fail-closed, spoofed I2P/proxy identity is removed, trusted peer identity is structurally valid/bounded, backend/provider/cache/trace response fingerprints are stripped, write throttling is bounded/churn-safe, and unsupported expectations fail before local target allocation.

### IRC

`ircclient` and `socksirc` retain the common anonymity filter. `ircserver` retains bounded registration and trusted peer-derived presentation; M077 adds activity-resetting post-registration inactivity expiry without parsing/reframing normal IRC traffic.

### Streamr

Streamr remains a small bounded datagram producer/consumer subsystem. M078 makes local UDP ingress/output loopback-only and aligns the subscriber ceiling to the reference value without creating generalized UDP/auth infrastructure.

## Containment authority

M061 remains the accepted source-path authority. M062 plus M063 remain the dependency/feature-ownership authority.

M080-M082 must not add a new `emissary-core/**` production path. If a correction requires a core API, router algorithm change, Yosemite fork/protocol extension, or new I2PControl-only dependency that cannot satisfy M062/M063, stop and create separate architecture/dependency planning.

## Accepted unrelated Proposal 170 state

Tunnel security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook `SetConfig` and unrelated base-I2PControl limitations remain separate and must be documented truthfully.

## Verification discipline

Use focused deterministic local tests, structurally valid I2P Destination fixtures, fake/local SAM and local TCP/UDP services, Tokio paused-time tests, package-scoped checks, M061/M062/M063 containment tests, Clippy, scoped nightly rustfmt for touched files, and `git diff --check`.

Do not add public-network certification/deanonymization tests, broad platform matrices, hosted CI, release machinery, generalized fuzzing, or soak farms merely for this workstream.

## Final status rule

The tunnel-security line of work is not closed until M080, M081, M082, M077, and M078 are independently closed and M079 accepts the actual final repository head with no high/medium security, anonymity, correctness, lifecycle, option-truthfulness, or containment finding.

No upstream review or acceptance is implied or authorized.
