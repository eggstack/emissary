# M057 — Post-M056 Planning-Record Consistency Corrective

Status: ready

Planning baseline: `cdbc3a4` — merged M054–M056 corrective implementation/reclosure head

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrects planning-record inconsistencies remaining after accepted M056 closure.

Milestone class: corrective planning/closure hygiene

Hard dependencies:

- M054 accepted closure;
- M055 accepted closure;
- M056 accepted integrated reclosure;
- no production defect is currently open within the M054–M056 corrective scope.

Pinned authority:

- `plans/003-planning-process.md`;
- `plans/closure/i2pcontrol-proposal-170/054-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/055-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/056-closure.md`;
- final machine-readable RouterInfo disposition at the accepted M056 head: 43 total / 37 available / 1 protocol-permitted neutral / 5 unavailable.

## 1. Objective

Make the active Proposal 170 planning records internally self-consistent after M056 without changing production behavior, source disposition, runtime support, test behavior, or the accepted 37/1/5 RouterInfo matrix.

This is a documentation/control-surface corrective pass only. It closes stale planning labels and historical-baseline wording that survived the M054–M056 implementation sequence.

The known defects are:

1. `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` still labels M055 as `READY` in the dependency graph even though M055 and M056 are both closed elsewhere in the same roadmap and in their accepted closure records.
2. `plans/registry.md` incorrectly states that review baseline `970252c` declares 37 available / 1 neutral / 5 unavailable. That baseline is the pre-corrective M052-era head and represented the historical 40/1/2 claim. The accepted 37/1/5 matrix is the post-M054/M055/M056 disposition.
3. Any residual phrase that describes M056 as pending, blocked, ready, or not yet reconciled must be removed from active planning records now that `056-closure.md` is accepted.
4. Any active planning sentence that conflates the historical `970252c` implementation state with the final accepted M056 state must be corrected while preserving both states as explicit history.

Do not reinterpret or rewrite the accepted production disposition while correcting these records.

## 2. Why prior closure missed this

M056 correctly focused on production-free integration reclosure and source accounting. The implementation and closure records were reconciled, but a small number of status/baseline phrases in the higher-level roadmap/registry were not normalized after the final closure commit sequence.

The prior checks validated contract counts, source dispositions, child-process behavior, changed production paths, and the closure evidence itself. They did not include a targeted planning-control-surface consistency scan for stale milestone words such as `READY`, `pending M056`, or baseline/count pairings.

M057 adds exactly that missing regression class. It must not compensate by rerunning broad product verification that M056 already accepted.

## 3. Scope and authorized paths

Production changes: **none**.

Test code changes: **none**, unless a repository-existing planning-lint test already owns these exact textual invariants and only its fixture data must be updated. Do not create a new test harness for M057.

Primary authorized files:

- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml` only to register M057's zero-production path budget;
- `plans/implementation/i2pcontrol-proposal-170/057-post-m056-planning-record-consistency-corrective.md`;
- `plans/closure/i2pcontrol-proposal-170/057-closure.md` when closing.

No file under these prefixes is authorized for behavior changes:

- `emissary-core/**`;
- `emissary-cli/src/**`;
- `emissary-cli/tests/**` except the narrow pre-existing planning-lint exception above;
- `docs/i2pcontrol/**` unless the M057 audit discovers a directly contradictory milestone-status/baseline sentence. If no such contradiction exists, do not touch product/support documentation;
- `.github/**`;
- release/publishing/configuration/runtime files.

If correcting the known planning inconsistencies appears to require a production or product-documentation change, stop and record the discrepancy instead of broadening M057.

## 4. Invariants

1. M054, M055, and M056 remain closed; M057 does not reopen their production findings.
2. The final RouterInfo matrix remains exactly 43 total / 37 available / 1 neutral / 5 unavailable.
3. The historical `970252c` state remains identifiable as the pre-corrective M052-era 40/1/2 claim.
4. Historical closure records remain immutable evidence. Do not rewrite `049-closure.md`, `050-closure.md`, `052-closure.md`, `054-closure.md`, `055-closure.md`, or `056-closure.md` merely to make chronology look cleaner.
5. M051 remains blocked with accepted semantic limitation because no authoritative news/ban owners exist.
6. Overall subsystem status remains `partial Proposal 170 support`; M057 is not authority to claim full Proposal 170 implementation.
7. No unsupported tunnel data plane or other previously excluded runtime becomes in scope.
8. No upstream issue, pull request, review, submission, adoption request, merge request, maintainer outreach, or contribution preparation is authorized.
9. No new CI, workflow, release, coverage, fuzz, soak, or generated-evidence apparatus is introduced.
10. The active registry must identify only one dependency-ready handoff while M057 is open, and none after M057 closes unless a separate new plan is explicitly created.

## 5. Work packages

### WP1 — Establish exact historical/current status table

Before editing prose, build a small review table from accepted repository evidence:

| Item | Historical/current fact |
|---|---|
| `970252c` | merged M053–M052 head; historical 40 available / 1 neutral / 2 unavailable claim before M054/M055 correction |
| M054 | closed; transit-15s demoted unavailable |
| M055 | closed; v4/v6 error selectors demoted unavailable |
| M056 | closed; accepted integrated 37 available / 1 neutral / 5 unavailable matrix |
| M051 | blocked with accepted semantic limitation; news and banned peers unavailable |
| current subsystem | partial Proposal 170 support; no production corrective successor currently open |

Use this table as the source of truth for every edited planning sentence.

### WP2 — Correct roadmap lifecycle/status drift

In `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`:

- change the stale M055 dependency-graph status from `READY` to `CLOSED`;
- ensure M056 is shown `CLOSED` consistently;
- add M057 as the planning-record corrective successor while it is active/ready;
- after M057 implementation, state that no dependency-ready successor remains;
- ensure the milestone prose and dependency graph agree exactly;
- do not modify technical source dispositions or production scope from M054–M056.

### WP3 — Correct registry historical-baseline wording

In `plans/registry.md`:

- replace the inaccurate assertion that baseline `970252c` declares 37/1/5;
- state explicitly that `970252c` carried the historical 40/1/2 source claim;
- distinguish that historical state from the accepted post-M056 37/1/5 disposition;
- remove any wording that says M056 is still pending or required;
- update the retained M052 row so its source-count claim is described as superseded by M056 rather than `pending M056`;
- while M057 is open, register M057 as the sole dependency-ready implementation plan;
- after closure, return the registry to no dependency-ready plan and retain M051 as the only blocked semantic successor.

### WP4 — Reconcile implementation index and boundary manifest

In `plans/implementation/i2pcontrol-proposal-170/README.md`:

- list M057 as the sole current handoff while ready/active;
- describe it as planning-record-only;
- preserve M054–M056 as closed;
- preserve the final 37/1/5 source matrix and overall partial Proposal 170 status.

In `045-052-routerinfo-source-boundary.toml`:

- increment the manifest version;
- extend its objective through M057;
- add `[m057]` with `core_production = []` and an explicit no-production purpose;
- record a required regression that checks active planning status/baseline consistency;
- do not expand any always-allowed production prefix.

### WP5 — Targeted consistency scan

Search active Proposal 170 planning records for stale combinations including at least:

- `M055` near `READY` or `ready`;
- `M056` near `pending`, `blocked`, or `ready` where the sentence is describing current state;
- `970252c` near `37 available`, `37/1/5`, or equivalent final-count wording;
- `40/1/2` without historical/superseded qualification in current-state sections;
- `pending M056`;
- `no current successor` while M057 is still registered ready;
- references that claim full Proposal 170 completion rather than partial support.

Review hits semantically; historical quoted material may remain when explicitly labeled historical.

Do not mass-rewrite archived or historical closure records merely because they contain old statuses that were correct at the time.

### WP6 — Closure

Create `plans/closure/i2pcontrol-proposal-170/057-closure.md` with:

- exact implementation/planning head;
- before/after table for each stale planning assertion;
- list of files changed;
- proof that no production/test behavior files changed;
- exact consistency-search commands or equivalent repository searches and outcomes;
- confirmation that final RouterInfo accounting remains 37/1/5;
- confirmation M051 remains blocked and no new production handoff is created;
- explicit internal-only/no-upstream attestation.

## 6. Verification

M057 does not justify the broad Rust matrix already accepted in M056 because it changes no production behavior.

Required verification is bounded to planning integrity:

```bash
git diff --check

git diff --name-only <M057_BASE>..HEAD

rg -n "M055.*READY|M055.*ready|pending M056|M056.*(READY|ready|blocked)|970252c.*(37 available|37/1/5)|37/1/5.*970252c" \
  plans/registry.md \
  plans/subsystems/i2pcontrol-proposal-170-roadmap.md \
  plans/implementation/i2pcontrol-proposal-170/README.md

rg -n "40 available / 1 neutral / 2 unavailable|40/1/2|37 available / 1 neutral / 5 unavailable|37/1/5" \
  plans/registry.md \
  plans/subsystems/i2pcontrol-proposal-170-roadmap.md \
  plans/implementation/i2pcontrol-proposal-170/README.md \
  plans/closure/i2pcontrol-proposal-170/056-closure.md
```

Equivalent search tooling is acceptable if the implementation environment does not provide `rg`.

If any Rust/source file changes, M057 has exceeded its normal scope and closure must stop unless the change is only the explicitly permitted pre-existing planning-lint fixture update and is independently justified.

No hosted CI run is required or authorized for this documentation-only pass.

## 7. Acceptance criteria

M057 may close only when all of the following are true:

1. The roadmap dependency graph labels M054, M055, and M056 consistently as closed.
2. The roadmap identifies M057 correctly during the corrective pass and no successor after closure.
3. The registry states that `970252c` represented the historical 40/1/2 claim, not the final 37/1/5 state.
4. The registry states that the accepted post-M056 matrix is 37/1/5.
5. No active registry/roadmap/index sentence describes M056 as pending, blocked, ready, or awaiting reconciliation.
6. M052's historical source-count claim is explicitly superseded by M056, not `pending M056`.
7. M054/M055/M056 accepted closure records are preserved rather than rewritten.
8. M051 remains blocked with accepted semantic limitation.
9. Overall Proposal 170 support remains partial and no unsupported runtime is promoted.
10. The M057 changed-file list contains no production Rust, runtime, workflow, or release file.
11. The machine-readable boundary contains `[m057]` with zero core/production authority.
12. `git diff --check` passes.
13. The targeted stale-status/baseline scan returns no unqualified active-state contradiction.
14. Closure records the internal-only/no-upstream attestation required by `plans/003-planning-process.md`.

## 8. Stop conditions

Stop and create a separate corrective disposition rather than expanding M057 if:

- a newly discovered inconsistency reflects an actual production/source defect rather than stale planning prose;
- the final 37/1/5 machine-readable contract does not match M056 closure evidence;
- correcting a planning statement would require changing runtime behavior;
- M051 appears newly implementable only by introducing a news/ban subsystem outside current scope;
- an external Proposal 170 revision changed and materially affects the accepted contract;
- any upstream write/review/submission step is proposed.

## 9. Compatibility, migration, security, and operations

There are no runtime compatibility, schema, migration, persistence, lifecycle, cancellation, contention, or network effects because M057 changes planning records only.

The security requirement is preservation of the existing containment boundary: no production surface may change under this plan. The operational requirement is simply that future implementation agents receive a truthful registry/roadmap state and do not reopen already closed corrective work because of stale labels.

## 10. Closure disposition

Expected disposition after successful execution:

- M057 `closed`;
- M054/M055/M056 remain closed;
- final RouterInfo matrix remains 37 available / 1 protocol-permitted neutral / 5 unavailable;
- overall Proposal 170 remains partial under the accepted scope;
- M051 remains blocked with accepted semantic limitation;
- no dependency-ready implementation plan remains;
- no production code changed;
- no upstream interaction occurred.
