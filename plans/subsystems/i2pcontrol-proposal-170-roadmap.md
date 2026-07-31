# I2PControl Proposal 170 Roadmap

Status: active internal corrective work

Current corrective baseline: `db5e0679369dcefe61eb24bd10079dbf98086cea`

Pinned source:

- I2P Proposal 170, `I2PControl Expansion`
- status: `Open`
- created and last updated: `2026-05-20`
- `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`

Canonical references:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/closure/i2pcontrol-proposal-170/017-closure-invalidation.md`
- `plans/closure/i2pcontrol-proposal-170/018-implementation-disposition.md`
- `plans/implementation/i2pcontrol-proposal-170/018a-wire-semantics-and-internal-only-corrective-pass.md`
- `plans/implementation/i2pcontrol-proposal-170/019a-internal-pinned-revision-reclosure.md`

## 1. Purpose and boundary

This subsystem owns internal compatibility of `eggstack/emissary` with the pinned Proposal 170 I2PControl wire contract:

- exact method, parameter, action, selector, and response vocabulary;
- exact JSON types and parameter-presence semantics;
- truthful source-unavailable and runtime-unsupported behavior;
- compatibility with already-shipped Emissary extensions;
- accurate internal documentation of wire, source, runtime, and evidence status.

This roadmap does not authorize an upstream contribution.

It does not own:

- upstream review, pull requests, merge requests, issues, discussions, patches, adoption, or maintainer outreach;
- router algorithms;
- transport or NetDB inspection architecture;
- peer selection;
- missing tunnel data planes;
- frontend behavior;
- runtime address-book precedence;
- CI, release, packaging, publishing, or version policy;
- broad security, dependency, or formatting work.

## 2. Internal-only policy

All work is internal to `eggstack/emissary`.

Agents and reviewers MUST NOT:

- write to an upstream repository;
- open or modify upstream issues, pull requests, merge requests, discussions, reviews, or proposals;
- request upstream review, feedback, adoption, approval, or merge;
- push branches, commits, tags, patches, or artifacts to an upstream remote;
- contact upstream maintainers on behalf of this workstream;
- prepare an upstream contribution package, patch series, or merge plan.

Read-only inspection of the official proposal and reference implementations is allowed solely for internal verification.

A future upstream contribution requires a separate explicit maintainer directive that supersedes this roadmap and `plans/003-planning-process.md`. No current milestone or closure record grants that authority.

Violation is a stop condition and invalidates closure evidence.

## 3. Retained implementation

The following remain accepted absent direct regression:

- the exact 43-key Proposal 170 RouterInfo manifest and direct request form;
- canonical AddressBook direct modes and response adjudication;
- seven lowercase TunnelManager actions and structured success responses;
- direct ClientServicesInfo service parameters;
- bounded SAM session observation and explicit overflow behavior;
- atomic service-generation fencing;
- pre-spawn TLS connection bounds;
- live metric and log sources;
- durable administrative tunnel and address-book stores;
- truthful unavailable sources and explicit unsupported tunnel runtimes;
- compatibility-path separation in production handlers.

The initial M018 implementation at `ea35de9` is retained but not sufficient for closure.

## 4. Current findings

| ID | Finding | Severity | Owner |
|---|---|---|---|
| M018A-F1 | Transit total uses received plus sent rather than forwarded/transmitted transit bytes | high | M018A |
| M018A-F2 | Valid canonical TunnelManager operational failures do not consistently use structured `result.status` | high | M018A |
| M018A-F3 | Base and compatibility inventory remains labeled as canonical Proposal 170 coverage in the static manifest | medium | M018A |
| M018A-F4 | TunnelManager canonical examples and `Name`/`All` wording require correction | low | M018A |
| M018A-F5 | Internal-only/no-upstream authority needed normative codification | governance | codified; M018A/M019A must verify |
| M018-F6 | No true live-session-to-production-ClientServicesInfo SAM test | medium evidence decision | M019A |

No new router, transport, NetDB, tunnel-runtime, or frontend finding is activated.

## 5. Current sequence

```text
M018 initial exact-wire implementation (retained, corrective disposition)
    |
    v
M018A wire semantics and internal-only corrective pass
    |
    v
M019A internal pinned-revision independent reclosure
```

The original M019 plan is superseded and non-executable.

## 6. M018A — Wire semantics and internal-only corrective pass

Plan:

- `plans/implementation/i2pcontrol-proposal-170/018a-wire-semantics-and-internal-only-corrective-pass.md`

Status: ready

Objective:

- return the existing transmitted/forwarded transit counter for `i2p.router.net.total.transit.bytes`;
- ensure valid canonical TunnelManager operation failures use structured `result.status`;
- separate base, canonical Proposal 170, and Emissary compatibility manifests;
- correct directly affected documentation;
- verify and attest the internal-only/no-upstream boundary.

Required regressions:

- distinct transit received/sent values prove no double counting;
- missing/owned/rejected canonical tunnel operations remain inside structured result envelopes;
- malformed requests still return JSON-RPC validation errors;
- compatibility action responses remain unchanged;
- canonical and compatibility inventory cannot be conflated.

Exit conditions:

- all M018A findings resolved or explicitly blocked;
- targeted package verification passes;
- no prohibited scope or external write occurs;
- new frozen implementation/test head recorded;
- `plans/closure/i2pcontrol-proposal-170/018a-implementation-disposition.md` exists;
- M018A moves to `closing`;
- M019A becomes `ready`.

## 7. M019A — Internal pinned-revision independent reclosure

Plan:

- `plans/implementation/i2pcontrol-proposal-170/019a-internal-pinned-revision-reclosure.md`

Status: blocked

Activation requires a complete frozen M018A head and a distinct auditable internal reviewer.

Review duties:

- read-only refetch and pin verification of Proposal 170;
- exact 43-key and transit-semantic audit;
- AddressBook mode/envelope audit;
- TunnelManager success and operational-failure envelope audit;
- ClientServicesInfo and SAM evidence audit;
- canonical/base/compatibility manifest audit;
- wire/source/runtime/evidence claim audit;
- changed-file and targeted command review;
- internal-only/no-upstream compliance attestation.

M019A must reject closure for any unresolved high or medium finding. It must not modify production code and close in the same pass.

## 8. Canonical, compatibility, and evidence policy

### Canonical Proposal 170

Canonical forms:

- use exact official names and casing;
- use exact direct parameter-presence behavior where specified;
- preserve exact response keys and JSON types;
- pass literal internal fixtures derived from the pinned source;
- do not depend on an Emissary compatibility path.

### Compatibility

Compatibility forms:

- may remain to avoid breaking internal/existing Emissary clients;
- must be separately named and counted;
- must not substitute for canonical behavior;
- must reject ambiguous mixing;
- do not imply upstream compatibility certification.

### Coverage dimensions

| Dimension | Meaning |
|---|---|
| Wire implemented | exact request and response contract is recognized |
| Source available | a truthful current Emissary source exists |
| Runtime implemented | a real operational backend exists |
| Evidence qualified | implementation is present but closure evidence has a named limitation |

Unavailable, unsupported, compatibility-only, or qualified items are not full operational coverage.

## 9. Scope guard

Allowed production scope:

- the existing RouterInfo transit mapping and smallest existing metric DTO/adapter seam;
- existing TunnelManager handler result-envelope logic;
- existing static conformance manifest;
- focused tests and directly affected I2PControl documentation.

Prohibited:

- upstream write, submission, review, adoption, or merge activity;
- `.github/workflows/**`;
- CI, release, packaging, publishing, version, matrix, coverage, or generated-evidence machinery;
- tunnel data-plane implementation;
- broad router, transport, NetDB, peer, tunnel, crypto, resolver, frontend, SAM, or I2CP redesign;
- generic protocol/schema/fixture frameworks;
- dependencies;
- repository-wide formatting;
- fabricated defaults.

## 10. Verification policy

Use local package-scoped verification:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Focused runs:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol transit
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest
```

Use touched-file configured formatting checks when the known workspace baseline blocks full stable formatting. Do not reformat unrelated files.

No remote or upstream CI, release check, platform matrix, coverage gate, fuzzing, network farm, submission check, or evidence bundle is required.

## 11. Milestone status

| Milestone/handoff | Status | Current disposition |
|---|---|---|
| 001–014 | historical implementation evidence | retained as recorded |
| 015 | invalid historical closure | retained as history |
| 016 | bounded SAM implementation | retained |
| 017 | invalidated broad closure | component evidence retained |
| 018 | corrective pass required | initial implementation retained at `ea35de9` |
| 018A | ready | sole executable implementation handoff |
| 019 | superseded | non-executable |
| 019A | blocked | final internal revision-bound closure gate |

## 12. Completion definition

The workstream may become `closed internally against pinned revision` only when M019A confirms:

- exact current proposal revision;
- exact 43 RouterInfo additions and forwarded/transmitted transit semantics;
- exact AddressBook canonical modes and adjudicated results;
- exact lowercase TunnelManager actions and structured success/failure operation results;
- exact direct ClientServicesInfo form;
- base and compatibility inventory excluded from canonical totals;
- truthful unavailable and unsupported classifications;
- accepted or explicitly blocked SAM evidence;
- zero unresolved high/medium findings;
- no scope expansion;
- complete internal-only compliance attestation.

Internal closure must not imply upstream review, acceptance, adoption, certification, or merge. Any future change to the Open proposal requires a new internal comparison only.