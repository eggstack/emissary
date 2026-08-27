# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; tunnel runtime functionally complete; M092/M093 closed; production security closed; M094 documentation reconciliation ready

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

Current M094 planning baseline: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.

Known valid pre-M091 implementation/closure baseline: `6d631d4423c7faa761b47a84e07436bbaf5d9ad4`.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, I2P/I2P+ reference implementations, Yosemite source, issues, and pull requests are read-only evidence. No plan authorizes upstream submission, review request, maintainer contact, contribution preparation, merge request, or repository write outside this fork.

## Scope and containment

ADR-0003 authorizes the bounded Proposal 170 tunnel data planes while preserving established containment and startup/control-plane ownership.

The implementation target remains:

- keep runtime/filter/admission policy in `emissary-cli/src/i2pcontrol/**` wherever technically possible;
- keep lower-layer changes exceptional, narrowly scoped, and explicitly authorized before implementation;
- apply or reject every runtime-relevant option before allocation;
- keep persistent server secrets backend-owned and redacted;
- keep startup-managed tunnel behavior separate from Proposal 170 TunnelManager ownership;
- avoid hosted CI/fuzz/soak/release machinery for this bounded workstream.

M090 remains a valid in-boundary correction. M091 crossed the intended boundary into root dependency state, a full vendored Yosemite copy, `emissary-core`, and historical containment machinery while its registered plan was still blocked. M092 removed that unauthorized expansion and restored the smaller boundary. M093 independently reclosed the corrected production head.

## Current handoff

The production/security line is already closed by M093. The sole ready handoff is documentation/evidence reconciliation:

- **M094** `094-post-m093-planning-state-reconciliation.md` — status `ready`.

M094 reconciles stale M092/M093 plan-state wording and commit-head terminology only. It must make no production, dependency, runtime, core, router, startup, frontend, API, security-policy, or residual-risk change.

Closed predecessors:

- M092: `plans/closure/i2pcontrol-proposal-170/092-closure.md`;
- M093: `plans/closure/i2pcontrol-proposal-170/093-closure.md`.

Per `plans/003-planning-process.md`, M094 is the only dependency-ready tunnel-security handoff.

## Why the corrective line reopened

At planning commit `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3`, M091 was explicitly blocked. Its plan said no supported Yosemite/SAM transport existed for the intended pre-accept concurrency limit and that vendoring/forking Yosemite, switching to an unreviewed git dependency, or using a process-global registry was not authorized without a later maintainer directive.

Commit `5053ce6b595351b251afb36f1f7d5278ef8f58d1` nevertheless implemented a vendored Yosemite 0.7.0 transport, modified root dependency/lockfile state, changed three `emissary-core` SAM/streaming files, changed accepted-server session construction, and amended M060/M061/M062 containment machinery to allow those changes. Commit `944da7b887b6efbd46601e9fad1c853581f40b8e` then rewrote the plan from blocked to closed and described authorization not present in the registered handoff before implementation.

The technical tests recorded by M091 remain useful evidence, but they do not cure the missing pre-implementation authority. M092 therefore treats M091 as `corrective pass required` rather than current closure authority.

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
| M089 | closed for pinned head | independent post-corrective tunnel-security reclosure |
| M090 | **closed / retained** | resolver-free server targets and IRC half-close correction |
| M091 | **corrective pass required** | lower-layer concurrency implementation landed while registered plan was blocked |
| M092 | **closed** | remove M091 production/dependency/vendor delta; restore containment/history |
| M093 | **closed** | independent post-M092 corrected-head tunnel-security reclosure; current production security authority |
| M094 | **ready** | post-M093 planning-state/SHA reconciliation; no production change |

## Current corrective sequence

```text
M087 generic server inactivity corrective            [CLOSED]
  |
  v
M088 pre-accept boundary evidence                     [CLOSED / TIER 3]
  |
  v
M089 independent security reclosure                   [CLOSED @ f0f3fc2]
  |
  v
M090 resolver-free loopback + IRC half-close          [CLOSED]
  |
  v
M091 pre-accept stream concurrency implementation     [CORRECTIVE PASS REQUIRED]
  |
  v
M092 authorization/dependency/containment rollback    [CLOSED @ 8860407]
  |
  v
M093 independent corrected-head security reclosure    [CLOSED @ 4da022e]
  |
  v
M094 planning-state reconciliation                    [READY / DOCS ONLY]
```

The production/security line remains current-head closed by M093. M094 exists only to converge planning records.

## M092 handoff summary

Plan: `092-m091-authorization-and-containment-corrective.md`.

Required result:

- preserve M090 exactly as valid production work;
- restore crates.io Yosemite 0.7.0 and its pre-M091 lockfile entry;
- remove `vendor/yosemite/**`;
- remove M091's lower-layer option transport from accepted-server session creation;
- remove the three M091 `emissary-core` SAM/streaming changes;
- restore M060/M061/M062 production/dependency containment semantics to the pre-M091 authority;
- retain only exact planning/closure path bookkeeping where the cumulative guard requires it;
- restore truthful M091 blocked/superseded status and mark its closure corrective-pass-required rather than deleting history;
- return M088's lower-layer/pre-accept limitation to the accepted residual set;
- make no replacement lower-layer transport or new production feature.

M092 is closed; closure: `plans/closure/i2pcontrol-proposal-170/092-closure.md`.

## M093 handoff summary

Plan: `093-post-m092-tunnel-security-reclosure.md`.

Status: closed. Closure: `plans/closure/i2pcontrol-proposal-170/093-closure.md`.

Result:

- independently audited all twelve Proposal 170 tunnel backends at the corrected head; no high-, medium-, or low-severity production security/anonymity defect found inside the approved Proposal 170 boundary;
- verified M090 remains intact;
- verified M091 production/dependency/vendor artifacts are gone (`vendor/yosemite/**` is absent; crates.io Yosemite 0.7.0 is the sole source);
- verified containment semantics were restored rather than weakened;
- rechecked generic/HTTP/IRC/Streamr security and lifetime behavior;
- explicitly recorded the post-accept application-admission boundary and the accepted lower-layer resource/timing residual;
- made no production code change;
- opened no new production corrective because no stop condition triggered.

## M094 handoff summary

Plan: `094-post-m093-planning-state-reconciliation.md`.

Status: ready.

Required result:

- mark M092 plan closed and link its closure;
- convert stale M093 live-readiness language to historical sequencing language;
- pin M092 implementation head `8860407a79347ce925603821cdb231e47a680623` directly in its closure;
- distinguish M093 reviewed production head `8860407a79347ce925603821cdb231e47a680623` from closure/planning commit `4da022ec874e9915e2d38fe63c609bff537ee8ff`;
- reconcile active planning text in this README, registry, and security roadmap;
- add only exact M094 plan/closure entries to the cumulative M062 planning allowlist;
- preserve all M088/M090/M091/M092/M093 technical/security dispositions;
- make no production/dependency change.

## Security-critical family rules retained

### Accepted server admission

Accepted-stream server families derive trusted identity from Yosemite, require exactly one canonicalizable supported Destination, apply bounded transactional application admission before handler/local-target work, and keep peer/rate/expiry structures hard bounded.

After M092 and M093, documentation calls this boundary post-accept/application-level. Signed-SYN/streaming work may occur before `ServerAdmissionState`; that is the accepted M088 residual unless a future separately authorized lower-layer plan changes it.

### Trusted peer identity

The M083 boundary remains:

- bounded Base64 text input;
- one decode;
- `Destination::parse_frame`;
- empty parser remainder required;
- 32-byte `parsed.id()` for accounting;
- canonical full-Destination text from Base64-encoding `parsed.serialize()`.

### Generic server

Generic `server` remains accepted-stream/raw-relay with a finite progress-based inactivity bound, useful half-close behavior, loopback target confinement, and bounded target connect.

### HTTP server

`httpserver` and inbound `httpbidirserver` use the shared accepted-stream identity/admission path. Request framing stays fail-closed, spoofed I2P/proxy identity is stripped, trusted full-Destination text is canonical, response fingerprints are stripped, POST accounting uses the canonical Destination ID, unsupported `Expect` requests fail before local target allocation, and M090 ensures the local target is a literal loopback address.

No speculative body byte cap or replacement fairness algorithm is authorized by M094.

### IRC

`ircserver` retains bounded registration and trusted peer-derived presentation, five-second literal-loopback target connect, ten-minute activity-resetting post-registration idle expiry, and M090 half-close drain semantics. Post-registration bytes remain raw.

### Streamr

Streamr remains a bounded datagram subsystem with loopback-only local endpoints, ten subscribers, 60-second expiry, 15-second refresh, one-byte controls, 1200-byte application payload cap, 4095-byte receive bound, and bounded sequential fanout.

The finite subscriber set is not Sybil-resistant. This remains a reference-aligned specialized availability limitation unless new evidence justifies separate corrective planning.

## Containment authority

M061 remains the source-path authority. M062 plus M063 remain the dependency/feature-ownership authority.

M092 restored the pre-M091 semantic assertions. The M062 exact planning allowlist may include M094 plan and closure paths only as bookkeeping; it must not broaden production path globs or dependency/feature ownership.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate.

## Final status rule

Production/runtime tunnel security remains current-head closed after M093. M094 is the sole ready handoff and is documentation/evidence reconciliation only. It may return the planning line to `closed / no ready handoff` after its closure.

M090 remains valid closed production work. M091 remains technical history but is not current authority. M092 remains rollback/containment authority. M093 remains the current production security reclosure authority.

Proposal 170 remains separately partial for accepted source/truthfulness limitations, RouterInfo 37/1/5, M051, and unrelated AddressBook/base-I2PControl gaps.

No upstream review or acceptance is implied or authorized.
