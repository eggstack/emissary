# I2PControl Proposal 170 Milestone 019A — Internal Pinned-Revision Reclosure

Status: blocked

Planning baseline: `db5e0679369dcefe61eb24bd10079dbf98086cea`

Supersedes:

- `plans/implementation/i2pcontrol-proposal-170/019-pinned-revision-independent-reclosure.md`

Activation dependency:

- M018A is complete on a frozen implementation/test head;
- `plans/closure/i2pcontrol-proposal-170/018a-implementation-disposition.md` exists;
- the registry marks M018A `closing` and M019A `ready`;
- the M019A reviewer is distinct from the final M018A implementation executor and identifies the separate internal review run.

Primary class: internal independent protocol-conformance closure

## 1. Objective

Independently determine whether the actual internal `eggstack/emissary` final head conforms to the pinned Proposal 170 revision, with unavailable sources, unsupported runtimes, and compatibility extensions classified truthfully.

M019A is an internal review and closure gate only. It does not authorize or prepare any upstream submission, review request, pull request, merge request, issue, discussion, patch, maintainer outreach, or adoption proposal.

The bounded closure statement is:

> The internal eggstack/emissary implementation matches the Proposal 170 wire contract as pinned to the reviewed open revision, with separately documented unavailable data sources, unsupported tunnel runtimes, compatibility extensions, and evidence limitations.

No closure statement may claim upstream acceptance, upstream compatibility certification, intended upstream merge, or permanent conformance to future revisions.

## 2. Absolute no-upstream rule

The reviewer MUST NOT:

- open, draft, update, or comment on an upstream issue, pull request, merge request, discussion, review, or proposal;
- request upstream review, approval, merge, adoption, or feedback;
- push branches, commits, tags, patches, or artifacts to an upstream remote;
- contact upstream maintainers on behalf of this review;
- generate a submission package, upstream patch series, contribution checklist, or merge plan;
- use any connector or API write action against an upstream repository.

The reviewer MAY perform read-only source verification against:

- the official Proposal 170 page;
- linked reference implementations;
- upstream commits and discussions needed to interpret the pinned contract.

All review artifacts and repository writes MUST remain in `eggstack/emissary`.

Any upstream write, submission, solicitation, or merge-preparation action invalidates the review and requires an internal incident note before work resumes.

## 3. Review boundary

Review only:

- the M018 and M018A implementation diffs;
- current I2PControl request dispatch and serialization paths;
- the canonical/base/compatibility manifests and literal fixtures;
- directly affected Proposal 170 documentation;
- retained bounded SAM observation evidence;
- final changed-file scope and targeted local verification;
- compliance with the internal-only policy.

Do not reopen:

- router algorithms;
- transport, NetDB, peer-selection, tunnel construction, cryptography, resolver, SAM, I2CP, or frontend architecture;
- missing tunnel data planes;
- CI, release, packaging, publishing, or distribution policy;
- unrelated workspace formatting or historical code quality;
- any upstream contribution or merge path.

## 4. Required source and semantic audit

### 4.1 Pinned source

At review time:

1. fetch the official Proposal 170 page read-only;
2. record status, created date, last-updated date, and review date;
3. compare the current revision against the repository manifest;
4. stop if the proposal changed materially after the implementation pin;
5. do not contact upstream to resolve ambiguity; record the ambiguity internally and return it to M018A or an explicit architecture-owner decision.

### 4.2 RouterInfo

Verify:

- exactly 43 canonical additions with exact spelling and type;
- direct parameter-by-presence behavior;
- canonical response key preservation;
- unavailable fields fail truthfully without partial or fabricated results;
- nullable fields use `null` only where allowed;
- `i2p.router.logs.clear` returns `"success"`;
- `i2p.router.net.total.transit.bytes` returns transmitted/forwarded transit bytes only, not received plus sent;
- base and compatibility keys are not counted as Proposal 170 additions.

The transit check must use a fixture with distinct received and sent values.

### 4.3 AddressBook

Verify:

- exact canonical `Type`, `Hostname`, `Destination`, optional presence-selected `Delete`;
- in-method `SetSubscriptions` and `SetConfig` modes;
- exact response-envelope adjudication;
- mixed canonical modes and canonical/compatibility forms rejected;
- persistence, bounds, validation, and redaction retained;
- action-style and standalone methods classified as compatibility/base implementation surfaces, not canonical additions.

### 4.4 TunnelManager

Verify:

- exactly seven lowercase canonical actions;
- `List` and capitalized actions remain compatibility-only;
- canonical success and expected operational failure outcomes use structured `result.status`;
- `get` uses structured `status` and `info`;
- malformed requests use appropriate JSON-RPC errors;
- absent definitions, ownership rejection, duplicate/collision outcomes, and backend rejection use the adopted canonical operation-status policy;
- unsupported runtimes never report running;
- option inventory remains truthful and does not imply data-plane implementation.

Any canonical operation outcome that incorrectly escapes into a JSON-RPC application error is a high finding.

### 4.5 ClientServicesInfo and SAM

Verify:

- direct service parameters select by presence with any value;
- nested `Selector` remains compatibility-only;
- mixed forms rejected;
- bounded SAM observations remain current, secret-free, and explicitly fail on overflow/missing source;
- the closest-production composition evidence is described accurately;
- no true end-to-end SAM claim is made unless a real activation/removal test exists.

The reviewer must explicitly accept or reject the qualified SAM evidence. An evidence limitation may not be silently converted to full end-to-end coverage.

## 5. Manifest and documentation audit

Verify that current tests and documents distinguish:

| Class | Required treatment |
|---|---|
| Base protocol | implemented foundation, not counted as Proposal 170 additions |
| Canonical Proposal 170 | exact additions, modes, actions, keys, and result fields |
| Emissary compatibility | retained extension, separately named and excluded from canonical totals |

Documentation must also distinguish:

- wire implemented;
- source available;
- runtime implemented;
- evidence qualified.

Correct low-severity documentation errors only after findings are recorded and only when they do not alter production behavior.

## 6. Internal-only compliance audit

The closure record must state:

- all repository writes targeted `eggstack/emissary`;
- no upstream issue, pull request, merge request, discussion, review request, patch, branch, tag, or comment was created or modified;
- no upstream maintainer was contacted for review, adoption, or merge;
- no submission package or upstream contribution plan was produced;
- external sources were used read-only.

Any failure of this policy rejects closure regardless of protocol test outcomes.

## 7. Verification

Run locally against the frozen final head:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Run focused checks for:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol transit
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol client_services
```

Use touched-file configured formatting checks if the known workspace baseline blocks full stable formatting. Do not format unrelated files.

No remote CI, upstream CI, platform matrix, coverage gate, fuzzing, release check, or submission validation is required.

## 8. Finding policy

- High: wrong canonical key/type/semantic, wrong operation envelope, false operational success, sensitive exposure, or violation of the no-upstream rule.
- Medium: compatibility counted as canonical, incomplete option/evidence classification, unrecorded ambiguity, misleading coverage claim, or inadequate SAM evidence.
- Low: localized documentation defect that cannot mislead client behavior or closure status.

Closure requires zero unresolved high and medium findings.

Any high or medium finding returns to M018A when it fits that boundary. Do not create another milestone for the same defect class.

## 9. Required closure artifact

Create:

- `plans/closure/i2pcontrol-proposal-170/019a-closure.md`

It must include:

1. status;
2. pinned proposal metadata and internal review date;
3. frozen M018A implementation head and final reviewed head;
4. implementation executor and distinct internal reviewer identifiers;
5. changed-file classification;
6. exact 43-key and transit-semantic result;
7. AddressBook result;
8. TunnelManager success/failure envelope result;
9. ClientServicesInfo and SAM evidence result;
10. canonical/base/compatibility manifest result;
11. wire/source/runtime/evidence table;
12. verification commands and outcomes;
13. internal-only compliance attestation;
14. unresolved findings;
15. bounded internal closure statement.

Preserve M017, M018, and their corrective records as history. Do not rewrite them into passing final evidence.

## 10. Acceptance criteria

M019A may close only when:

1. the official source revision is pinned and unchanged or the internal implementation is rebased;
2. the exact 43-key manifest and JSON types pass;
3. transit bytes use forwarded/transmitted semantics;
4. all canonical TunnelManager expected operation failures use structured `result.status`;
5. malformed requests retain correct JSON-RPC validation errors;
6. canonical AddressBook modes and adjudicated results pass;
7. ClientServicesInfo direct presence passes with any value;
8. base and compatibility inventory is excluded from canonical totals;
9. unavailable sources and unsupported runtimes remain truthful;
10. SAM evidence is explicitly accepted or a medium blocker is recorded;
11. documentation separates wire, source, runtime, and evidence claims;
12. no scope expansion entered;
13. targeted local verification passes or exact pre-existing limitations are recorded;
14. reviewer independence is auditable;
15. the no-upstream compliance attestation is complete;
16. zero unresolved high/medium findings remain;
17. final status is `closed internally against pinned revision` or equivalent wording that cannot imply upstream review or acceptance.

## 11. Activation rule

M019A remains blocked until M018A freezes a complete implementation/test head and records its disposition.

Once activated:

1. perform read-only source verification;
2. record findings before closure edits;
3. do not modify production code in the same review pass;
4. reject and return high/medium defects to M018A;
5. create `019a-closure.md` only after the final internal head passes;
6. update registry and roadmap only after closure acceptance;
7. never initiate upstream submission, review, or merge activity.