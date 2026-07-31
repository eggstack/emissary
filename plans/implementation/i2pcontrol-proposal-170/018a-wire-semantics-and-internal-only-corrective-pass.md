# I2PControl Proposal 170 Milestone 018A — Wire Semantics and Internal-Only Corrective Pass

Status: ready

Planning baseline: `db5e0679369dcefe61eb24bd10079dbf98086cea`

Parent implementation:

- `plans/implementation/i2pcontrol-proposal-170/018-exact-wire-contract-reconciliation.md`
- frozen M018 implementation head `ea35de9be339fa2c963f9c553cbbcf01540e3ee3`
- `plans/closure/i2pcontrol-proposal-170/018-implementation-disposition.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Primary class: narrow protocol-semantic corrective pass

## 1. Objective

Correct the remaining Proposal 170 semantic and evidence defects found after the initial M018 implementation without reopening router internals, tunnel data planes, CI, release, or broad validation scope.

M018A owns only:

1. the exact meaning of `i2p.router.net.total.transit.bytes`;
2. canonical TunnelManager operational-failure result envelopes;
3. separation of canonical Proposal 170 inventory from base-protocol and Emissary compatibility inventory;
4. directly affected focused tests and documentation;
5. codification and enforcement of the internal-only, no-upstream-submission boundary.

M018A is the sole dependency-ready implementation handoff. M019A remains blocked until M018A lands on a frozen complete head.

## 2. Absolute internal-only boundary

This workstream is internal to `eggstack/emissary`.

The implementation agent and every later reviewer MUST NOT:

- open or draft an issue, pull request, merge request, discussion, review request, or patch submission in any upstream repository;
- request upstream review, approval, acceptance, or merge;
- push commits, branches, tags, patches, or generated artifacts to an upstream remote;
- comment on upstream issues, pull requests, commits, proposals, or discussions on behalf of this workstream;
- contact upstream maintainers to solicit review, adoption, or merge;
- represent this work as intended for upstream submission;
- add roadmap, release, documentation, or automation steps that prepare or propose an upstream contribution;
- use GitHub or another connector to mutate any upstream repository.

Allowed external interaction is read-only research:

- fetch and inspect the official Proposal 170 text;
- fetch and inspect linked reference implementations, commits, and discussions;
- cite those sources in internal documentation;
- compare internal behavior against those sources.

All repository writes for this handoff MUST target `eggstack/emissary`. A future upstream contribution would require a separate explicit maintainer directive that supersedes this policy; no current plan or closure record grants that authority.

Violation of this boundary is a stop condition and invalidates closure evidence.

## 3. Corrective findings

| ID | Finding | Severity | Required correction |
|---|---|---|---|
| M018A-F1 | `i2p.router.net.total.transit.bytes` returns received plus sent transit bytes | high protocol semantic defect | return the forwarded/transmitted transit-byte counter only, matching the pinned proposal and adopted i2pd semantic |
| M018A-F2 | Some valid canonical TunnelManager operations return JSON-RPC application errors instead of `result.status = "error - ..."` | high canonical-envelope defect | keep malformed requests as JSON-RPC errors, but return structured operation status for valid canonical actions that fail during lookup, ownership, persistence, or runtime dispatch |
| M018A-F3 | `conformance_manifest.rs` still describes base methods and Emissary action-style AddressBook forms as Proposal 170 canonical inventory | medium coverage defect | split canonical additions/modes from base protocol and compatibility-extension manifests |
| M018A-F4 | TunnelManager documentation contains capitalized canonical examples and overstates the `Name` requirement when `All: true` is valid | low documentation defect | correct examples and requirement wording |
| M018A-F5 | Active planning did not explicitly prohibit upstream submission or review solicitation | governance defect | apply the internal-only boundary in normative governance and all active Proposal 170 handoffs |

The qualified SAM evidence remains for M019A adjudication. M018A does not build a new SAM harness unless a direct regression is found.

## 4. Scope boundary

### Allowed production files

- `emissary-cli/src/i2pcontrol/router_info_handler.rs`
- the smallest existing metric adapter or DTO file required to expose the already-existing transmitted transit counter correctly
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`

### Allowed tests

- existing I2PControl unit tests in the touched modules;
- `emissary-cli/tests/conformance_manifest.rs`;
- one existing focused fixture/integration file only when required by the exact corrected path.

### Allowed documentation and planning

- `docs/i2pcontrol/**` directly affected by the corrections;
- `plans/003-planning-process.md`;
- Proposal 170 roadmap, registry, implementation handoff index, disposition, and closure records.

### Prohibited

- `.github/workflows/**`;
- CI, release, packaging, publishing, version, or coverage changes;
- upstream repository writes or contribution preparation;
- new tunnel data planes;
- router, transport, NetDB, peer, SAM, I2CP, cryptographic, resolver, or frontend redesign;
- generic protocol/schema/fixture frameworks;
- new dependencies;
- repository-wide formatting;
- changes to unrelated compatibility behavior;
- fabricated values for unavailable sources.

## 5. Work package A — Correct transit-byte semantics

The canonical selector is:

```text
i2p.router.net.total.transit.bytes
```

Required behavior:

- return total transit bytes forwarded/transmitted by the router;
- do not sum transit received and transit sent counters;
- preserve an integer JSON value;
- preserve direct parameter-by-presence selection;
- do not change unrelated total received or total sent selectors.

Preferred implementation:

- use the existing transmitted/sent member of the transit-byte snapshot when it already represents forwarded transit traffic;
- otherwise make the smallest read-only DTO/adapter correction needed to expose that existing counter truthfully.

Required regression:

1. construct a source snapshot with distinct received and sent values;
2. request only `i2p.router.net.total.transit.bytes` using the canonical direct form;
3. assert the response equals the sent/transmitted value, not the sum;
4. assert the canonical response key and integer type are unchanged.

## 6. Work package B — Correct canonical TunnelManager failure envelopes

Canonical action validation errors remain JSON-RPC errors, including:

- missing or invalid `Action`;
- missing required canonical parameters;
- invalid field types or ranges;
- unknown tunnel type;
- invalid `All` use.

Once a syntactically valid canonical operation has been selected, operational outcomes MUST use a JSON-RPC success envelope containing a structured result object:

```json
{
  "result": {
    "status": "error - ..."
  }
}
```

Apply this to canonical failures such as:

- edit/get/lifecycle lookup of an absent tunnel;
- startup-managed ownership rejection;
- duplicate or rename collision outcomes;
- persistence/control-plane operation rejection;
- backend/runtime operation rejection.

Internal failures that cannot be represented as a normal operation outcome may remain sanitized JSON-RPC internal errors. The implementation disposition must identify any retained internal-error case and why it is not a Proposal 170 operation status.

Compatibility requests retain their established response behavior unless changing them is required to avoid shared-path corruption.

Required regressions:

- canonical `edit` of a missing tunnel returns `result.status` beginning with `error -`;
- canonical `get` of a missing tunnel returns structured `result.status` and no bare application-error envelope;
- canonical lifecycle action on a missing or startup-managed tunnel returns structured `result.status`;
- canonical persistence/backend rejection returns structured status when it is an operation outcome;
- malformed canonical requests still return `INVALID_PARAMS`;
- compatibility capitalized actions retain their prior shapes.

## 7. Work package C — Correct conformance classification

Refactor the existing static manifest without adding a new framework.

The test inventory must distinguish:

1. base JSON-RPC/I2PControl methods and error codes;
2. canonical Proposal 170 additions and operation modes;
3. Emissary compatibility extensions.

Required classification:

- standalone `SetConfig` and `SetSubscriptions` methods are compatibility/base implementation surfaces, not canonical Proposal 170 AddressBook modes;
- action-style AddressBook `List`, `Lookup`, `Add`, `Update`, and `Delete` are compatibility extensions;
- canonical AddressBook inventory is the direct entry mode plus in-method `SetSubscriptions` and `SetConfig` modes;
- the seven lowercase TunnelManager actions are canonical;
- capitalized actions and `List` are compatibility extensions;
- the exact 43 RouterInfo additions remain canonical and separate from the 121-key legacy/base registry;
- base JSON-RPC error codes are not described as Proposal 170 additions.

Required tests:

- canonical and compatibility sets are separately named and unique;
- no compatibility-only method, mode, action, or selector is counted in canonical totals;
- production constants remain covered without relabeling them as canonical Proposal 170 additions;
- the existing 43-key and seven-action exact-set assertions remain.

## 8. Work package D — Documentation and policy correction

Correct directly affected documents so they state:

- transit bytes mean forwarded/transmitted transit traffic;
- canonical TunnelManager operational failures use `result.status`;
- `Name` is not required when a canonical lifecycle request uses `All: true`;
- examples use lowercase canonical action values;
- compatibility inventories are not canonical Proposal 170 coverage;
- the workstream is internal-only and must not be submitted upstream.

No document may imply:

- planned upstream review;
- planned upstream merge;
- a contribution or adoption path;
- maintainer outreach;
- release or publication through an upstream project.

References to upstream proposals and implementations are read-only normative/research references only.

## 9. Verification

Run locally and proportionally:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Run focused tests during implementation:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol transit
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest
```

Check formatting only for touched Rust files with the repository-configured formatter when the known workspace baseline blocks full `cargo fmt --all -- --check`. Do not format unrelated files.

No remote CI, platform matrix, coverage gate, fuzzing, release check, upstream check, or generated evidence bundle is required.

## 10. Acceptance criteria

M018A is complete only when:

1. transit total returns the forwarded/transmitted counter only;
2. a distinct received/sent regression proves no double counting;
3. all valid canonical TunnelManager operation failures use structured `result.status` unless a specifically justified internal failure is recorded;
4. malformed requests continue to use appropriate JSON-RPC errors;
5. compatibility TunnelManager response behavior is preserved;
6. canonical, base, and compatibility manifests are separately named and counted;
7. standalone AddressBook methods and action-style modes are not counted as canonical Proposal 170 additions;
8. documentation examples and `Name`/`All` wording are corrected;
9. normative planning governance and active handoffs contain the internal-only no-upstream rule;
10. no upstream repository, issue, pull request, discussion, or maintainer channel was mutated or contacted;
11. no CI, release, data-plane, broad-core, dependency, or unrelated formatting scope entered;
12. targeted check, test, and clippy pass, with exact baseline limitations recorded;
13. an M018A implementation disposition records the frozen head and requirement-to-evidence matrix;
14. registry moves M018A to `closing` and M019A to `ready` only after all criteria pass.

## 11. Stop conditions

Stop rather than guess when:

- the proposal and the pinned reference implementation disagree materially on an affected operation envelope;
- correcting transit semantics requires inventing a new metric rather than exposing an existing transmitted counter;
- an operational failure cannot be classified safely as validation, expected operation status, or internal failure;
- a new dependency, generic framework, broad router/runtime change, or tunnel data plane is proposed;
- any action would submit, propose, request review for, or merge work upstream.

## 12. Handoff

1. inspect the exact current implementations and tests;
2. correct transit semantics and add the distinct-counter regression;
3. correct canonical TunnelManager operational-failure envelopes and tests;
4. split canonical/base/compatibility manifest classification;
5. correct directly affected documentation;
6. verify the internal-only boundary is present in all active planning surfaces;
7. run targeted local verification;
8. freeze the implementation head;
9. create `plans/closure/i2pcontrol-proposal-170/018a-implementation-disposition.md`;
10. move M018A to `closing` and M019A to `ready`.

Do not perform upstream submission, review solicitation, or merge preparation at any step.