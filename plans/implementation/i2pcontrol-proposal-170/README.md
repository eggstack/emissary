# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; tunnel-security corrective sequence active; M083 next

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

Post-M076 corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Current M083 planning baseline: `a35d2bc333ff0e8b9889cd133d8ef75a98faa049`.

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

- `083-admission-capacity-and-trusted-destination-exactness-corrective.md` — **ready**.

M077 is re-blocked until M083 closes because IRC consumes the same shared admission/trusted-peer boundary.

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
| M073 | closed; corrective history | generic client/server option truthfulness; M081 closed the M075 accepted-but-ignored regression |
| M074 | closed; corrective history | shared server admission/rate hardening; M080 corrected its original transactionality/cardinality defects |
| M075 | closed | generic server accepted-stream migration; M081 closed LeaseSet regression |
| M076 | closed; corrective history | HTTP fingerprint/POST hardening; M082 closed direct identity-length/`Expect`/POST-key defects |

## Current corrective sequence before final closure

| Handoff | Status | Scope | Dependency |
|---|---|---|---|
| M080 | corrective pass required at current head; historical closure retained | transactional admission, canonical 32-byte peer keys, bounded expiry direction; M083 closes remaining capacity/expiry semantics | post-M082 review -> M083 |
| M081 | closed | generic `leaseSetEncType` apply-or-reject after accepted-stream migration | complete |
| M082 | corrective pass required only for inherited trusted-Destination exactness; direct HTTP fixes retained | common trusted identity, fixed `Expect` rejection, canonical POST peer key | post-M082 review -> M083 |
| M083 | ready | minute/no-history representability, tightest aggregate capacity bound, active-peer expiry consistency, exact/canonical trusted Destination | current handoff |
| M077 | blocked | IRC post-registration idle lifetime/connect bound | M083 closed |
| M078 | blocked | Streamr loopback-only local UDP + reference-aligned fanout | M083 + M077 closed |
| M079 | blocked | independent integrated tunnel-security reclosure | M083 + M077-M078 closed |

Per `plans/003-planning-process.md`, only the next dependency-ready plan is registered `ready`.

## Why M083 was added

### Capacity/history semantics

M080 correctly moved peer accounting to canonical 32-byte Destination IDs, made denial transactional, and bounded peer/expiry state. Its remaining capacity gate uses `retention > SHORT_RETENTION`, but minute peer history and short cleanup are both 60 seconds. Minute-only history can therefore bypass the check that rejects fully unlimited aggregate arrival even though fresh identities must remain retained for the minute window.

When no per-peer rate history is enabled, inactive records also do not need an arbitrary 60-second retention. M083 separates historical rate state from active connection ownership so no-history inactive churn cannot fill the table.

### Tightest aggregate bound

The current helper selects the first non-zero aggregate field. The runtime actually enforces every enabled minute/hour/day limit. M083 computes a conservative retained-event bound for each enabled aggregate window, includes fixed-window boundary overlap, and selects the smallest safe bound. This prevents both under-budgeting and false rejection when hour/day is tighter than minute.

### Expiry-index state

M080's bounded `BTreeMap` design is retained, but an expired active peer can currently lose its expiry entry while remaining in the peer map. M083 defines one explicit active/inactive indexing invariant and proves it across acquire/reap/drop.

### Trusted Destination exactness

The common peer helper currently accepts the core convenience parser without requiring zero unconsumed bytes. M083 requires the decoded payload to be exactly one supported Destination and derives downstream full-Destination text from canonical Base64 encoding of the parsed bytes. The 32-byte accounting ID remains unchanged.

## Security-critical family rules

### Accepted server admission

All accepted-stream server families derive trusted identity from Yosemite, require exactly one canonicalizable supported Destination, apply bounded transactional admission before handler/local-target work, and keep every peer/rate/expiry structure hard bounded. A denied attempt must not leave attacker-owned accounting state.

Historical peer-rate state must be distinguishable from active-only connection state. Capacity must be proven against all enabled aggregate windows, including fixed-window overlap.

### Generic server

Control-plane generic `server` remains accepted-stream/raw-relay. It may not return to `STREAM FORWARD`. Runtime-relevant options, including `leaseSetEncType`, must be applied or rejected.

### HTTP server

`httpserver` and inbound `httpbidirserver` use the same accepted-stream filter path. Request framing remains fail-closed, spoofed I2P/proxy identity is removed, trusted full-Destination text is canonical, backend/provider/cache/trace fingerprints are stripped, write throttling is bounded/churn-safe, and unsupported expectations fail before local target allocation.

### IRC

`ircclient` and `socksirc` retain the common anonymity filter. `ircserver` retains bounded registration and trusted peer-derived presentation. M077 adds activity-resetting post-registration inactivity expiry only after M083 closes the shared admission/identity prerequisite.

### Streamr

Streamr remains a small bounded datagram producer/consumer subsystem. M078 makes local UDP ingress/output loopback-only and aligns the subscriber ceiling to the reference value without creating generalized UDP/auth infrastructure.

## Containment authority

M061 remains the accepted source-path authority. M062 plus M063 remain the dependency/feature-ownership authority.

M083 must not add a new `emissary-core/**` production path or dependency. If a correction requires a core API, router algorithm change, Yosemite fork/protocol extension, or new dependency that cannot satisfy M062/M063, stop and create separate planning.

## Accepted unrelated Proposal 170 state

Tunnel security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook `SetConfig` and unrelated base-I2PControl limitations remain separate and must be documented truthfully.

## Verification discipline

Use focused deterministic local tests, structurally valid I2P Destination fixtures plus trailing-byte negatives, fake/local SAM and local TCP/UDP services, Tokio paused-time capacity/window/reap tests, package-scoped checks, M061/M062/M063 containment tests, Clippy, scoped nightly rustfmt for touched files, and `git diff --check`.

Do not add public-network certification/deanonymization tests, broad platform matrices, hosted CI, release machinery, generalized fuzzing, or soak farms merely for this workstream.

## Final status rule

The tunnel-security line of work is not closed until M083, M077, and M078 are independently closed and M079 accepts the actual final repository head with no high/medium security, anonymity, correctness, lifecycle, option-truthfulness, or containment finding.

No upstream review or acceptance is implied or authorized.
