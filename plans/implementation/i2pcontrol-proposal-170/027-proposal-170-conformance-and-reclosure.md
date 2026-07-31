# M027 — Proposal 170 Exact Conformance and Independent Reclosure

Status: blocked

Primary class: evidence/closure gate

Hard dependencies:

- M020 implementation disposition accepted
- M021 implementation disposition accepted
- M022 implementation disposition accepted or explicitly blocked with AddressBook support downgraded
- M023 implementation disposition accepted
- M024 implementation disposition accepted
- M025 exact contract/source matrix frozen
- M026 bounded-source disposition accepted

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Invalidated prior closure:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

## 1. Bounded objective

Perform a distinct internal review of the corrected implementation against:

- the existing I2PControl authentication and JSON-RPC contract;
- the pinned 2026-05-20 Proposal 170 revision;
- the internal explicit-unsupported tunnel boundary;
- the final source/runtime/persistence matrices produced by M020–M026.

M027 owns conformance fixtures, claim reconciliation, focused final verification, and the new closure record. It may correct documentation and tests. It must not implement production features unless a small evidence-only defect is trivial and separately recorded; any material production defect returns to a new corrective implementation plan.

## 2. Reviewer independence

The closure review must be performed by a distinct auditable internal reviewer or agent from the final implementation author for M026.

The reviewer must:

- refetch external specifications read-only;
- record the exact proposal status/date/revision evidence;
- inspect production code rather than relying on implementation dispositions alone;
- run or independently verify the required commands;
- report failed evidence and residual limitations;
- attest that no upstream write, submission, review, adoption, or merge solicitation occurred.

No upstream or third-party review is authorized or required.

## 3. Exact conformance inventory

The closure record must evaluate these independent surfaces.

### 3.1 Base I2PControl/JSON-RPC

- `Authenticate` canonical parameters and numeric API response;
- standard `params.Token` on protected calls;
- missing/unknown/version/password error codes;
- header compatibility path and conflict handling;
- notification execution/no-response behavior;
- request-ID validation and preservation;
- direct base RouterInfo compatibility.

### 3.2 AddressBook

- exact canonical entry add/replace/delete presence semantics;
- exact `SetSubscriptions` and `SetConfig` modes;
- result envelope adjudication;
- full destination validation;
- one coherent actual runtime/durable authority;
- restart and failure atomicity;
- four book identities and unchanged precedence;
- exact RouterInfo list/subscription/config shapes;
- compatibility paths separately identified.

### 3.3 TunnelManager

- seven exact lowercase canonical actions;
- twelve exact tunnel types;
- action-specific required/prohibited parameters;
- all option types/ranges/enums;
- exact status/results/info/rawConfig response shapes;
- one-generation atomic mutation;
- rename failure/restart evidence;
- secret handling;
- startup-managed ownership;
- explicit unsupported missing data planes;
- any accepted existing generic client/server lifecycle adapter;
- compatibility paths separately identified.

### 3.4 ClientServicesInfo

- six direct selector keys selected by presence;
- HTTP/SOCKS actual listener lifecycle including exit;
- I2PTunnel shared inventory and truthful address provenance;
- SAM bounded active session shape, incomplete failure, and recovery;
- I2CP actual listener state;
- BOB exact false value;
- response bounds and source failure behavior.

### 3.5 RouterInfo

- exact 43-key canonical addition set and JSON types;
- direct existing/base selector compatibility;
- requested-key-only behavior;
- exact source/neutral/unavailable disposition for every key;
- bounded source evidence for each available field;
- no fabricated default for unavailable fields;
- exact logs/log-clear/transit/rate/address-book/I2PTunnel semantics;
- final available/unavailable counts.

## 4. Required literal fixtures

Fixtures must be literal external-contract examples or minimal exact variants, not serializers compared against themselves.

Required fixture sets:

1. Existing I2PControl Authenticate and protected RouterInfo flow.
2. All I2PControl-specific authentication/version errors.
3. JSON-RPC request, notification, and invalid-ID matrix.
4. Canonical AddressBook add/delete/subscription/config requests and responses.
5. Canonical TunnelManager success/failure for every action.
6. Canonical `get` exact nested info/rawConfig object.
7. All twelve tunnel types on CRUD and unsupported lifecycle paths.
8. ClientServicesInfo exact key/shape fixtures.
9. Exact 43 RouterInfo selector/type manifest.
10. Representative available, neutral, unavailable, mixed, oversized, and source-failure RouterInfo requests.

Compatibility fixtures must live in a separately named section/test module and must not satisfy canonical fixture counts.

## 5. Requirement-to-evidence matrix

The closure record must include one row per material requirement with:

- requirement identifier;
- source specification/ADR/roadmap section;
- production file/function;
- focused test/fixture;
- command and outcome;
- wire/source/runtime/persistence classification;
- residual limitation;
- disposition.

A broad package test count is supporting evidence only; it cannot replace row-level evidence.

## 6. Failure, restart, cancellation, and contention review

The reviewer must inspect and cite evidence for:

- authentication failure before handler work;
- notification execution under normal resource bounds;
- tunnel create/edit/rename/delete failure atomicity;
- AddressBook durable/runtime consistency;
- startup collision behavior;
- proxy exit/stale generation handling;
- SAM overflow/incomplete/recovery;
- RouterInfo source query failure and no partial response;
- oversized result behavior;
- concurrent readers observing before/after coherent state;
- restart loading the same accepted state;
- corrupted generation fallback;
- response-lost-after-commit edge documentation.

## 7. Security review

The reviewer must search code, tests, and representative serialized files/responses for:

- passwords;
- tokens;
- proxy/outproxy/IRC credentials;
- private keys and key files;
- full private destinations where not explicitly required;
- filesystem roots and temporary paths;
- debug/tracing of raw configuration.

Required conclusions:

- protected methods require valid auth;
- conflicts fail closed;
- no unsupported tunnel opens resources;
- persistent sensitive data has enforced permissions where supported;
- errors/logs are sanitized;
- read-only snapshots expose only contract-required public state;
- no owner mutation/event-consumption authority leaked through observation handles.

## 8. Scope audit

Compare the final corrected implementation head against the pre-corrective baseline and classify every changed production file as:

- `i2pcontrol local`;
- `composition-only seam`;
- `address-book owner seam`;
- `existing tunnel manager seam`;
- `SAM observation seam`;
- `bounded RouterInfo owner snapshot`;
- `unexpected`.

Any unexpected file requires explicit justification. Broad router, protocol, crypto, frontend, workflow, release, dependency, or missing tunnel changes reject closure.

Verify no writes occurred outside `eggstack/emissary` and no upstream contribution artifact was prepared.

## 9. Verification commands

Run focused milestone filters first, then:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

If M022/M024/M026 touched `emissary-core`:

```bash
cargo check -p emissary-core
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
```

Run directly affected workspace packages only. Use touched-file formatting if unrelated repository-wide formatting remains dirty.

No remote CI, upstream CI, release gate, platform matrix, coverage threshold, fuzz campaign, network farm, or long soak run is required.

## 10. Documentation reconciliation

Update and cross-check:

- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/proposal-170-conformance.md`;
- method-specific AddressBook/TunnelManager/ClientServicesInfo/RouterInfo docs;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`;
- implementation README;
- new implementation dispositions and final closure record.

Historical closure/invalidation records must remain intact.

Every support claim must state the dimension: wire, source, runtime, persistence, or evidence.

## 11. Final disposition rules

### 11.1 Closed internally against pinned revision

Allowed only when:

- standard base I2PControl interoperability is exact;
- all Proposal 170 wire surfaces are exact;
- every field/action claimed source/runtime-supported has production evidence;
- any protocol-permitted neutral value is used exactly;
- no pinned field remains falsely labeled implemented;
- zero unresolved high/medium findings remain;
- all required atomicity/security/restart evidence passes;
- scope and no-upstream attestations pass.

This status may still state that missing tunnel data planes are runtime-unsupported under ADR-0001 if the wire/CRUD contract and operation failures are exact.

### 11.2 Partial Proposal 170 support

Required when:

- exact wire behavior is present, but one or more pinned RouterInfo/AddressBook/runtime sources remain truthfully unavailable;
- AddressBook cannot be connected to actual runtime authority without broad redesign;
- startup lifecycle authority remains unavailable;
- another non-fabricated source limitation persists.

The closure record must list exact unsupported/unavailable items and must not use unqualified `complete`, `implemented`, or `closed against Proposal 170` wording.

### 11.3 Corrective pass required

Required for any unresolved high/medium defect in wire behavior, authentication, persistence, runtime truthfulness, secret handling, source classification, or scope.

### 11.4 Blocked

Required when external evidence or an environment constraint prevents a necessary closure decision. A blocked state cannot be converted to passing by accepting implementation-agent assertions.

## 12. Acceptance criteria

M027 is complete only when:

- a distinct reviewer performs the source/code/evidence audit;
- literal fixture sets pass or failures are recorded;
- requirement-to-evidence matrix is complete;
- changed-file scope is classified;
- final source/runtime counts are exact;
- documentation and registry agree;
- security, restart, failure, and contention reviews are recorded;
- one final disposition from Section 11 is selected honestly;
- no production feature is silently added during closure;
- no upstream write, review, submission, adoption, merge request, or contribution package occurs.

## 13. Stop conditions

Reject closure and stop if:

- fixtures derive expectations from current serializers instead of the external contract;
- base/compatibility tests are counted as canonical Proposal 170 evidence;
- unavailable fields are represented by zero/empty defaults without protocol authority;
- missing tunnel data planes are implemented to improve closure counts;
- a material defect is patched without a new implementation disposition;
- repository scope expands unexpectedly;
- any upstream interaction is proposed or performed.

## 14. Required closure output

Create a new closure record under:

```text
plans/closure/i2pcontrol-proposal-170/027-closure.md
```

It must include:

- pinned external source evidence;
- corrected implementation/test head;
- implementation disposition references;
- requirement-to-evidence matrix;
- exact commands/outcomes;
- source/runtime availability tables;
- security and scope audit;
- unresolved findings;
- selected final disposition;
- internal-only/no-upstream attestation.

M027 is the only milestone authorized to restore a final subsystem status.