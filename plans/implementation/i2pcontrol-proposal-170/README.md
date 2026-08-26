# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; tunnel runtime functionally complete; M087-M090 tunnel-security corrective sequence closed; M091 blocked

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

Current M090 planning baseline: `f0f3fc2204318c2fac69817d347df2702c51287b`.

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

M087-M090 do not authorize a Proposal 170 wire expansion, broad router/core refactor, parallel SAM implementation, Yosemite fork/vendor/patch, or dependency widening. If M088 proves a dependency-boundary change is genuinely required, it must stop and produce a separate explicit plan.

## Current handoff

No implementation plan is currently dependency-ready in this tunnel-security
sequence. M090 is closed with
`plans/closure/i2pcontrol-proposal-170/090-closure.md`.

Future/dependency-gated handoffs:

- M091 remains blocked by the absent Yosemite/SAM transport for pre-accept streaming concurrency policy; its M090 dependency is now satisfied.

Per `plans/003-planning-process.md`, only the next executable handoff is registered `ready`.

## Why the security line reopened after M086

M085 remains valid runtime/security closure evidence for its pinned post-M084 head, and M086 remains valid documentation/evidence reconciliation. A later active-adversary review found two additional server-side hardening concerns:

1. generic `server` uses a bounded five-second target connect but then an unbounded raw relay, so several Sybil Destinations can hold all finite shared admission slots indefinitely without useful byte progress;
2. shared accepted-server admission is evaluated only after Yosemite/SAM `session.accept()` returns, leaving lower-layer stream-establishment work outside the application admission gate.

These findings do not imply a direct clearnet-address leak, but they are relevant to availability and active load/timing correlation resistance.

The same review found no current reason to add speculative HTTP or Streamr production semantics. HTTP already has bounded request/header/body handling and bounded fail-closed POST state; Streamr's ten-subscriber/60-second expiry model matches Java I2P and I2P+ reference behavior. M089 rechecked and recorded those residual risks; M090 changes only the two named server-boundary details.

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
| M074 | closed; corrective history | shared application server admission/rate hardening |
| M075 | closed; later lifetime corrective opened as M087 | generic accepted-stream migration |
| M076 | closed; corrective history | HTTP anonymity/POST hardening |
| M077 | closed | IRC connect/idle hardening |
| M078 | closed | Streamr loopback/fanout hardening |
| M079 | closed historical record | pre-M083 integrated tunnel-security reclosure |
| M080 | closed; corrective history | admission transactionality/cardinality |
| M081 | closed | generic `leaseSetEncType` apply-or-reject |
| M082 | closed; corrective history | HTTP peer identity / `Expect` / POST key |
| M083 | closed | admission capacity semantics + exact/canonical trusted Destination |
| M084 | closed | merged-head integration/planning corrective |
| M085 | closed; valid for pinned head | independent merged-head tunnel-security reclosure |
| M086 | closed; documentation/evidence only | post-M085 record reconciliation; no production runtime change |
| M087 | closed | progress-based generic-server inactivity timeout |
| M088 | closed; Tier 3 unsupported lower-layer semantic | lower-layer/pre-accept admission evidence and residual-risk disposition |
| M089 | **closed** | independent post-corrective tunnel-security reclosure; current-head authority |
| M090 | **closed** | resolver-free server targets and IRC half-close correction |

## Current corrective sequence

```text
M087 closed implementation baseline
             |
             v
M088 pre-accept/lower-layer admission    [CLOSED / TIER 3]
             |
             v
M089 independent security reclosure      [CLOSED]
             |
             v
M090 resolver-free loopback + IRC half-close [CLOSED]
```

M087 -> M088 was an administrative sequencing dependency and is now satisfied.
M088's unsupported lower-layer semantic is accepted as out of scope; M089 and
M090 are closed. M091 remains explicitly blocked on the lower-layer transport
boundary.

## M087 handoff summary

Plan: `087-generic-server-inactivity-timeout-corrective.md`.

Required result:

- replace indefinite zero-progress generic relay occupancy with a finite inactivity/progress timeout;
- do not impose a maximum absolute connection lifetime;
- successful byte progress in either direction resets the deadline;
- preserve useful half-close behavior and active long-lived raw streams;
- retain loopback target confinement, five-second target connect, trusted peer identity, and shared admission ownership;
- stay in `emissary-cli/src/i2pcontrol/**` plus focused tests/planning bookkeeping;
- no dependency/core/router/startup/frontend/Proposal-170-wire changes.

## M088 handoff summary

Plan: `088-pre-accept-server-admission-boundary-corrective.md`.

Required result:

- source-map the full inbound stream path from Emissary streaming/SAM through Yosemite `Session<style::Stream>::accept()` to `ServerAdmissionState`;
- establish from actual code whether Emissary supports Java-style lower-layer streaming connection limits;
- if already supported through the current boundary, wire only the smallest useful bound from existing admission policy;
- keep post-accept application admission as defense in depth;
- do not treat Java option names as supported merely because they can be serialized;
- if progress requires Yosemite vendoring/forking/patching, a new git dependency, broad `emissary-core/**`/router changes, or a parallel SAM stack, stop and create a separate explicit plan or record the capability as out of current scope.

An evidence-only M088 closure is valid if it precisely proves that the meaningful pre-accept semantic is unavailable within the approved containment boundary. It must not falsely claim equivalent protection from another post-accept counter.

## M089 handoff summary

Plan: `089-post-corrective-tunnel-security-reclosure.md`.

Required result:

- independent current-head audit of all twelve Proposal 170 tunnel types;
- verify M087 generic lifetime behavior;
- verify M088 lower-layer disposition without overstatement;
- recheck HTTP bounded parsing/body/POST state and identity filtering;
- explicitly document that no HTTP byte cap/fairness replacement is added absent a concrete high/medium defect;
- recheck Streamr ten-subscriber/60-second reference parity and document its Sybil-monopolization limitation;
- audit containment of M087/M088 changes;
- make no production code changes itself.

Any high/medium production finding opens a new numbered corrective and keeps M089 blocked.

## Security-critical family rules retained

### Accepted server admission

Accepted-stream server families derive trusted identity from Yosemite, require exactly one canonicalizable supported Destination, apply bounded transactional application admission before handler/local-target work, and keep peer/rate/expiry structures hard bounded.

Historical peer-rate state exists only when configured semantics require it. Capacity is proven against all enabled aggregate windows with fixed-window overlap. No-history inactive peers are removed on final lease drop.

M088 investigates whether an additional lower-layer bound can act before `session.accept()` returns. Until and unless that is proven, documentation must call the current admission boundary post-accept/application-level.

### Trusted peer identity

The current M083/M085 boundary is:

- bounded Base64 text input;
- one decode;
- `Destination::parse_frame`;
- empty parser remainder required;
- 32-byte `parsed.id()` for accounting;
- canonical full-Destination text from Base64-encoding `parsed.serialize()`.

### Generic server

Control-plane generic `server` remains accepted-stream/raw-relay. `leaseSetEncType` is applied when accepted; other runtime-relevant fields are applied or rejected before allocation.

M087 adds only a finite inactivity/progress lifetime bound; it must not change raw protocol semantics into a parsed application protocol.

### HTTP server

`httpserver` and inbound `httpbidirserver` use the shared accepted-stream identity/admission path. Request framing stays fail-closed, spoofed I2P/proxy identity is stripped, trusted full-Destination text is canonical, response fingerprints are stripped, POST accounting uses the canonical Destination ID, and unsupported `Expect` requests fail before local target allocation.

M089 rechecks the existing finite request-body relay deadline and bounded POST limiter. No speculative body byte cap or replacement fairness algorithm is currently authorized.

### IRC

`ircserver` retains bounded registration and trusted peer-derived presentation, five-second literal-loopback target connect, ten-minute activity-resetting post-registration idle expiry, and half-close drain semantics. Post-registration bytes remain raw.

### Streamr

Streamr remains a bounded datagram subsystem with loopback-only local endpoints, ten subscribers, 60-second expiry, 15-second refresh, one-byte controls, 1200-byte application payload cap, 4095-byte receive bound, and bounded sequential fanout.

The finite subscriber set is not Sybil-resistant. M089 records this as a reference-aligned specialized availability limitation unless new evidence justifies separate corrective planning.

## Containment authority

M061 remains the source-path authority. M062 plus M063 remain the dependency/feature-ownership authority.

The M062 exact planning allowlist contains the M090/M091 implementation and closure document pairs. This planning registration does not broaden production path globs or dependency/feature ownership.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate.

## Final status rule

M090 is closed. No tunnel-security implementation plan is currently executable;
M091 remains blocked until its lower-layer transport/dependency boundary is
resolved.

M085 remains valid historical reclosure authority for its pinned head. M089 is
the current-head runtime/security reclosure authority for its pinned head, and
M090's closure records the later server-boundary corrections without rewriting
M089's historical evidence.

Proposal 170 remains separately partial for accepted source/truthfulness limitations, RouterInfo 37/1/5, M051, and unrelated AddressBook/base-I2PControl gaps.

No upstream review or acceptance is implied or authorized.
