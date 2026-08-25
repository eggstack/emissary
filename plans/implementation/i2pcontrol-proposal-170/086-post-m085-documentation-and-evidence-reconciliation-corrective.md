# M086 — Post-M085 Documentation and Evidence Reconciliation Corrective

Status: ready — sole dependency-ready Proposal 170 handoff

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`.

Corrective predecessors / controlling evidence:

- M084 closure: `plans/closure/i2pcontrol-proposal-170/084-closure.md`;
- M085 closure: `plans/closure/i2pcontrol-proposal-170/085-closure.md`.

Planning baseline: `185d43174c491a57c217c39e45555d136f40a406`.

Classification: corrective pass / documentation-and-evidence integrity.

## 1. Objective

Reconcile the remaining stale and inaccurate Proposal 170 planning, support, and closure-evidence text after M085, without reopening the already-closed tunnel runtime/security implementation.

M085 remains the current-head final runtime/security reclosure authority. M086 exists because a later documentation audit found a small set of record-quality defects that do not imply a production runtime defect:

1. active planning surfaces still contain stale pre-M084/pre-M085 status language;
2. the user-facing trusted-peer description still names the pre-M083 convenience parser instead of the exact current `parse_frame` + zero-remainder + canonical-reencoding boundary;
3. M085 closure contains a transcription/arithmetic error for `MAX_PEER_ENTRIES`;
4. M084 closure should explicitly record that restoring the dropped HTTP identity-header helper bodies was a production-source merge restoration, even though it introduced no new intended runtime semantics and was independently re-audited by M085.

This milestone is evidence/status cleanup only. It MUST NOT change production behavior.

## 2. Why prior verification missed this

M084 and M085 correctly repaired and independently reclosed the merged implementation. Their verification focused on runtime/security correctness, test compilation, containment, and merged-head evidence.

The remaining defects are editorial/integrity defects distributed across several planning/support documents:

- `plans/registry.md` has a detailed section saying M084/M085 are closed, while the top subsystem table still says containment bookkeeping is pending and security reclosure is reopened;
- the tunnel-security roadmap header/final sections say closed, while earlier "current findings" and M077/M078 milestone text still describe already-resolved merge defects as current;
- the implementation README says M084/M085 are closed but still labels M077/M078 merged-head integration as pending in its historical table;
- `docs/i2pcontrol/proposal-170-support.md` still describes `TrustedPeerIdentity` as using `Destination::parse` and retaining the validated input text, while current code uses `Destination::parse_frame`, rejects non-empty remainder, and stores canonical Base64 re-encoding of `parsed.serialize()`;
- M085 closure states `16 MiB / 200 = 81,920`, but the authoritative Rust expression is `(16 * 1024 * 1024) / 200`, which evaluates by integer division to **83,886**;
- M084 closure accurately lists the restored `filters/http.rs` helper bodies, but its high-level "no runtime semantics changed" wording can be misread as "no production source was changed". The distinction should be made explicit without rewriting history.

These issues were not grounds to invalidate M085 because the audited production code and test evidence remain unchanged.

## 3. Canonical authority and disposition

M086 is governed by:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001, ADR-0002, ADR-0003;
- M061/M062/M063 containment authorities;
- M084 and M085 closure records;
- pinned Proposal 170 revision `2026-05-20`.

The controlling disposition entering M086 is:

- tunnel runtime/security: **closed** by M085;
- source/truthfulness: **partial Proposal 170 support** with the accepted RouterInfo 37/1/5 disposition and M051 blocker;
- containment: accepted M061/M062/M063 authority;
- no active runtime-security corrective;
- no upstream interaction authorized.

M086 corrects records around that disposition; it does not replace it.

## 4. Invariants

M086 MUST preserve:

- exact Proposal 170 wire/API behavior;
- all twelve tunnel backend registrations;
- M083 exact trusted-Destination parsing and canonical text behavior;
- M083 admission capacity/history/expiry semantics;
- M081 generic server option truthfulness;
- M082 HTTP `Expect`/POST behavior;
- M076/M084 HTTP identity/proxy-header filtering;
- M077 IRC connect/idle behavior;
- M078 Streamr local/fanout bounds;
- M085 final runtime/security closure disposition;
- RouterInfo 37/1/5 and M051 source/truthfulness disposition;
- M061/M062/M063 containment semantics;
- internal-only repository scope.

## 5. Explicit non-goals

M086 MUST NOT:

- modify any file under `emissary-cli/src/**`, `emissary-core/**`, or other production source tree;
- change Cargo manifests, dependencies, features, or `Cargo.lock`;
- add or remove tests of runtime behavior;
- rerun/reopen M085 as a security milestone;
- change tunnel types, options, defaults, limits, filters, parsing behavior, or lifecycle semantics;
- revisit RouterInfo source ownership, AddressBook, or unrelated base-I2PControl gaps;
- add CI, fuzz, soak, benchmark, release, or public-network infrastructure;
- rewrite historical closure evidence so it appears the original author knew facts discovered later;
- perform or prepare any upstream issue/PR/review/submission/contact activity.

If implementation requires a production-source change, stop M086 and create a new narrowly scoped runtime corrective plan.

## 6. Required changes

### 6.1 Reconcile `plans/registry.md`

Make the active control surface internally consistent.

Required current-state outcomes:

- source/truthfulness remains partial with RouterInfo 37/1/5 and M051 unchanged;
- containment is recorded as accepted/closed authority, not "bookkeeping corrective pending";
- tunnel runtime completion is recorded as complete with M085 security reclosure accepted, not "security reclosure reopened";
- tunnel security remains closed by M085;
- M086 is the sole dependency-ready handoff and is explicitly documentation/evidence-only;
- no wording implies M084 or M085 is pending;
- after M086 closes, no successor under this cleanup line is expected unless new evidence appears.

Keep the registry compact; remove obsolete duplicated narrative rather than carrying stale history indefinitely.

### 6.2 Reconcile the tunnel-security roadmap

Update `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` so historical merge defects are clearly labeled resolved history.

At minimum:

- the status line should distinguish "runtime/security closed" from "M086 documentation/evidence cleanup ready";
- the former "Current repository findings" section must not describe the stale IRC fixture or M062 under-coverage as current defects;
- M077 and M078 milestone summaries must say their merged-head integration is reconciled by M084;
- M084 must be described as closed and M085 as the accepted independent final reclosure authority;
- add M086 as a documentation/evidence-only cleanup that does not reopen runtime/security closure;
- final closure rule must remain M085 for runtime/security; M086 only closes the record-quality corrective.

Do not turn the roadmap into a new runtime sequence.

### 6.3 Reconcile the implementation README

Update `plans/implementation/i2pcontrol-proposal-170/README.md` so:

- M086 is the sole current ready handoff;
- M077/M078 are no longer labeled "merged-head integration pending";
- the explanation of the M084/M085 sequence is historical/past tense;
- M085 remains the current-head final runtime/security reclosure authority;
- M086 is explicitly limited to documentation/evidence reconciliation;
- after M086 closure, there should be no active tunnel-security handoff.

### 6.4 Correct trusted-peer support documentation

Update `docs/i2pcontrol/proposal-170-support.md` to match current `TrustedPeerIdentity` implementation exactly.

The current description MUST state that trusted peer text:

1. is bounded to `MAX_TRUSTED_DESTINATION_B64_TEXT` before decode;
2. is Base64-decoded once;
3. is parsed with `Destination::parse_frame`;
4. is rejected unless the parser remainder is empty;
5. derives the 32-byte accounting ID from `parsed.id()`;
6. stores/forwards canonical full-Destination text by Base64-encoding `parsed.serialize()` rather than retaining attacker-selected textual representation.

Do not claim `Destination::parse` is the current trusted boundary.

Inspect nearby `tunnel-manager.md`, `tunnel-backends.md`, and `streamr-runtime.md` only for the same stale current-state language. Change them only if an actual contradiction remains.

### 6.5 Add an explicit M085 arithmetic erratum

Do not silently rewrite history.

In `plans/closure/i2pcontrol-proposal-170/085-closure.md`, add an explicit M086 erratum/clarification that corrects the capacity-number transcription:

```text
HARD_PEER_STATE_MEMORY_BUDGET = 16 * 1024 * 1024 = 16,777,216 bytes
WORST_CASE_BYTES_PER_PEER = 200
MAX_PEER_ENTRIES = 16,777,216 / 200 = 83,886 (integer division)
```

The existing Rust constant/expression is authoritative and does not change. State explicitly that the former `81,920` text was a closure-document arithmetic error only and did not affect policy construction, runtime behavior, or the tests M085 executed.

Where practical, make the original erroneous line point to the erratum or correct it while preserving an explicit note that M086 supplied the correction. Do not make the record look as though the original M085 closure contained the corrected arithmetic.

### 6.6 Clarify the M084 merge-restoration deviation

Append an M086 clarification/erratum to `plans/closure/i2pcontrol-proposal-170/084-closure.md` rather than erasing the historical record.

Record exactly:

- M084's implementation commit `776407f...` did modify production `emissary-cli/src/i2pcontrol/backends/filters/http.rs` by restoring two helper definitions dropped by the merge;
- the restoration reinstated the already-intended M076/M079 exact-list + `x-forwarded-*` / `x-i2p-*` prefix behavior and did not add a new Proposal 170 wire feature or broaden policy;
- therefore "no runtime semantics changed" means "no new intended runtime semantics were introduced", not "no production source file changed";
- this was a bounded deviation from M084's original expectation that only test/planning integration would be required;
- M085 subsequently independently audited the exact post-M084 head, including the restored HTTP filtering behavior, and accepted it with no high/medium finding;
- no additional runtime corrective or reclosure is required solely for this historical clarification.

### 6.7 Keep M062 exact-path bookkeeping coherent

Because M086 adds a new planning/closure pair, update only the exact planning-path allowlist in `emissary-cli/tests/m062_dependency_containment.rs` for:

- `plans/implementation/i2pcontrol-proposal-170/086-post-m085-documentation-and-evidence-reconciliation-corrective.md`;
- `plans/closure/i2pcontrol-proposal-170/086-closure.md` when created.

Do not widen a production glob, feature rule, dependency rule, or source-path exception.

This test-only exact-path bookkeeping is the only Rust file M086 may modify.

## 7. Ordered work packages

### A. Establish baseline and changed-path budget

1. Record current `master` SHA.
2. Confirm M085 closure remains present and no newer production commit supersedes it.
3. Establish an allowlist for M086 changes consisting only of planning/docs/closure files plus `emissary-cli/tests/m062_dependency_containment.rs` exact-path bookkeeping.

### B. Correct active planning state

1. Reconcile registry.
2. Reconcile tunnel-security roadmap.
3. Reconcile implementation README.
4. Ensure exactly one ready handoff: M086.

### C. Correct user-facing technical documentation

1. Fix trusted peer parsing/canonicalization wording.
2. Inspect adjacent tunnel docs for the same stale statements.
3. Do not alter unrelated support matrices.

### D. Correct closure evidence transparently

1. Add M085 arithmetic erratum.
2. Add M084 production-helper restoration clarification.
3. Preserve original closure chronology and disposition.

### E. Restore exact containment bookkeeping

1. Add only M086 plan/closure paths to M062 planning allowlist.
2. Verify no production path authorization changed.

### F. Close M086

Create `plans/closure/i2pcontrol-proposal-170/086-closure.md` with a changed-document/evidence matrix and then remove M086 from ready state. The runtime/security line remains closed by M085 throughout.

## 8. Verification discipline

Verification must be proportional to the documentation/evidence-only scope.

Required:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Also record:

```text
git diff --name-only <M086-baseline>..HEAD
```

and prove no production source, manifest, lockfile, dependency, or feature file changed.

Perform targeted text inspection for stale current-state assertions, including:

- `current merged-head bookkeeping corrective pending`;
- `security reclosure reopened by merge composition`;
- `merged-head integration pending` for M077/M078;
- trusted-boundary claims naming `Destination::parse` instead of `parse_frame` + zero remainder;
- the incorrect `81,920` M085 capacity value.

A full 1,674-test I2PControl rerun, Clippy, runtime integration suite, network test, or new CI job is NOT required when M086 obeys its no-production-change scope. If production code changes, M086 must stop rather than expand verification to justify scope creep.

## 9. Compatibility, migration, failure, and contention semantics

M086 has no runtime compatibility or migration effect.

There is no new failure/cancellation/restart/contention behavior because no runtime code may change. Historical M085 evidence remains the authority for those semantics.

The only executable code change permitted is the M062 test's exact-path planning bookkeeping.

## 10. Acceptance criteria

M086 may close only when:

1. `plans/registry.md` has no stale pending/reopened status for M084/M085 work and registers only M086 as ready during implementation;
2. tunnel-security roadmap current-state text treats the M084 merge defects as resolved history;
3. implementation README no longer labels M077/M078 merged-head integration pending;
4. support documentation describes `TrustedPeerIdentity` using `Destination::parse_frame`, empty remainder, `parsed.id()`, and canonical Base64 of `parsed.serialize()`;
5. M085 closure contains an explicit, traceable correction from `81,920` to `83,886` without pretending the original record was already correct;
6. M084 closure explicitly distinguishes the production helper restoration from a new runtime semantic change and records the bounded plan deviation;
7. M085 remains the final runtime/security reclosure authority and is not reopened;
8. M062 exact-path bookkeeping includes M086 plan/closure paths and no broader rule change;
9. no production source, manifest, dependency, lockfile, feature, or core file changes;
10. M062 containment test passes;
11. `git diff --check` passes;
12. RouterInfo 37/1/5, M051, AddressBook, and unrelated base-I2PControl dispositions remain unchanged;
13. no upstream interaction occurred.

## 11. Stop conditions

Stop and create a new narrow corrective instead of continuing M086 if:

- current code contradicts M085's runtime/security conclusions;
- correcting the trusted-peer documentation requires changing `TrustedPeerIdentity` code;
- the 83,886 correction exposes a real runtime capacity defect rather than a documentation-only arithmetic mistake;
- an M084 HTTP helper problem remains in current production code;
- M062 can pass only by widening production-path authority;
- any Cargo/core/router/startup/runtime change is required.

## 12. Closure evidence required

`plans/closure/i2pcontrol-proposal-170/086-closure.md` MUST include:

- exact baseline/final SHA;
- changed-path list proving documentation/test-guard-only scope;
- current-state planning reconciliation matrix;
- trusted-peer documentation before/after summary;
- M085 capacity arithmetic erratum with authoritative constant/expression;
- M084 merge-restoration clarification and why M085 makes a new runtime reclosure unnecessary;
- M062 exact-path diff and test outcome;
- `git diff --check` outcome;
- unresolved findings, if any, with severity;
- final registry disposition: no active tunnel-security handoff if all acceptance criteria pass;
- explicit internal-only/no-upstream-interaction attestation.

## 13. Final disposition after M086

If M086 closes cleanly:

- M085 remains the final runtime/security closure authority;
- the tunnel runtime/security workstream remains closed;
- the documentation/evidence corrective is closed;
- no successor handoff is registered for tunnel security;
- Proposal 170 remains partial only for the separately documented source/truthfulness limitations.

No upstream review, acceptance, merge, adoption, or submission is implied or authorized.