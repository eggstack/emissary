# Proposal 170 Implementation Handoffs

Status: corrective pass required

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`

Pinned external authority:

- Proposal 170 `I2PControl Expansion`, Open, created/updated `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

## Internal-only rule

These handoffs are internal to `eggstack/emissary`.

No plan authorizes:

- an upstream issue, pull request, merge request, discussion, review request, or patch submission;
- upstream review, feedback, approval, adoption, or merge solicitation;
- pushing branches, commits, tags, patches, artifacts, or releases to an upstream remote;
- upstream maintainer outreach;
- preparation of an upstream contribution package, patch series, submission checklist, or merge plan;
- connector/API writes against any upstream or third-party repository.

External specifications and reference implementations may be inspected read-only for internal correctness. All writes must remain in `eggstack/emissary` unless a future explicit maintainer directive supersedes the normative planning policy.

Violation is a stop condition and invalidates affected evidence.

## Scope rule

The current corrective sequence owns only:

- post-M027 status and chronology repair;
- compile-time/runtime isolation of the existing Proposal 170 AddressBook control owner;
- restoration of disabled/default legacy AddressBook behavior;
- preservation of enabled-mode M022 semantics;
- optional dependency ownership where directly affected;
- focused regression evidence and independent final-head reclosure.

It must not implement missing tunnel data planes. HTTP, IRC, SOCKS-IRC, CONNECT, Streamr, bidirectional, and other missing listener/destination/LeaseSet/traffic implementations remain separate security-focused work.

It also must not add RouterInfo telemetry, polling, peer classifications, NetDB inspection, core algorithms, lifecycle supervisors, generic frameworks, CI/release machinery, or upstream contribution activity.

## Current handoffs

| Handoff | Status | Plan | Dependency |
|---|---|---|---|
| M028 — Post-M027 status and AddressBook feature isolation | ready | `028-post-m027-status-and-addressbook-feature-isolation.md` | none |
| M029 — In-scope Proposal 170 conformance reclosure | blocked | `029-in-scope-conformance-reclosure.md` | M028 closed with frozen head |

## Execution order

```text
M028 status and AddressBook feature-isolation corrective pass
    |
    v
M029 independent final-head reclosure
```

Only M028 is dependency-ready.

## Retained history

The following work remains candidate evidence and is not reopened unless M028 exposes a direct regression:

| Milestone | Retained result |
|---|---|
| M020 | base I2PControl authentication/token/error and JSON-RPC correctness |
| M021 | exact TunnelManager wire, validation, atomic persistence, secret boundary |
| M022 | enabled-mode runtime AddressBook authority; feature boundary reopened by M028 |
| M023 | startup tunnel inventory and ClientServicesInfo lifecycle/address truthfulness |
| M024 | recoverable bounded SAM observation |
| M025 | exact 43-selector RouterInfo contract/source matrix |
| M026 | no feasible additional bounded authoritative RouterInfo sources |
| M027 | literal conformance evidence and partial-support disposition; final closure invalidated pending M029 |

Current source matrix:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

Missing tunnel types remain explicit unsupported runtimes under ADR-0001.

## Closure chronology

- M017 and M019A are invalidated historical closures.
- M019 is superseded and non-executable; the post-M027 merge that revived it does not make it controlling.
- M020–M027 contain the retained corrective implementation and evidence.
- M027's final subsystem disposition is invalidated by `027-closure-invalidation.md` because the AddressBook feature boundary and post-merge status state require correction.
- M028 owns the implementation correction.
- M029 will be the controlling final-head review if accepted.

Historical files are retained for traceability and must not be rewritten into current authority.

## M028 handoff rule

M028 owns one bounded corrective objective:

- restore M027/M020–M027 planning authority;
- ensure no-feature and runtime-disabled execution never reads, writes, migrates, or consults Proposal 170 AddressBook control state;
- preserve legacy `addresses`, `destinations/`, subscription download, and modified-time behavior in disabled/default mode;
- preserve one coherent control owner and immediate lookup visibility in enabled mode;
- preserve control-state files while disabled and restore them on re-enable;
- restore optional `serde_json` feature ownership if no independent unconditional consumer requires it;
- add focused no-feature, runtime-disabled, enabled, restart, and transition regressions;
- produce M028 disposition/closure and freeze the implementation/test head.

M028 must not change canonical Proposal 170 wire behavior, source counts, SAM behavior, tunnel runtime support, or control-state schema.

## M029 closure rule

M029 is a distinct review only. It must:

- refetch the still-open external contract read-only;
- identify the reviewer as distinct from the M028 implementation executor;
- review the actual final M028 head and all later diffs;
- verify all four AddressBook execution states;
- rerun focused retained M020–M027 evidence;
- classify the final changed-file scope;
- verify no secret/path/resource regression;
- reconcile registry, roadmap, support docs, and closure chronology;
- choose an honest final disposition.

Expected status under current scope is `partial Proposal 170 support` because 26 RouterInfo selectors remain unavailable and missing tunnel data planes remain explicit unsupported runtimes.

`closed internally against pinned revision` is allowed only if every source/runtime dimension is actually available and evidenced; M028/M029 do not authorize the implementation work required to reach that state.

## Handoff discipline

Each implementation plan must produce an implementation disposition containing:

- implementation commits;
- exact changed files;
- requirement-to-evidence matrix;
- focused and broad command outcomes;
- failure/restart/cancellation/contention evidence;
- compatibility and migration effects;
- security review;
- unresolved findings with severity;
- scope/no-upstream attestation;
- frozen implementation/test head.

A successful commit or broad test count is not closure by itself.

## Verification rule

M028 default/disabled scope:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features address_book
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings
```

M028 enabled scope:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

M029 adds the focused TunnelManager, ClientServicesInfo, RouterInfo, production-composition, conformance-manifest, literal-fixture, and core SAM commands named in its plan.

Use touched-file formatting checks. Remote CI, upstream CI, release checks, platform matrices, coverage gates, fuzz campaigns, network farms, soak tests, submission checks, and generated evidence bundles are not required.

## Final-status rule

Possible outcomes after M029:

- `partial Proposal 170 support` when every implemented/claimed dimension is exact and evidenced but one or more sources/runtimes remain unavailable;
- `closed internally against pinned revision` only with actual evidence for every source/runtime dimension;
- `corrective pass required` for unresolved high/medium defects;
- `blocked` when the proposal changed or required evidence cannot be obtained.

No final status implies upstream review, acceptance, certification, adoption, or merge.
