# I2PControl Proposal 170 Corrective Roadmap

Status: corrective pass required

Current corrective baseline:

- `03a384aec495232e64468dcf61d60dd2bab5cfe0`

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`
- status: `Open`
- created: `2026-05-20`
- last updated: `2026-05-20`
- `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`
- existing I2PControl authentication and error contract at `https://i2p.net/en/docs/api/i2pcontrol`

Canonical internal references:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`

## 1. Purpose

This roadmap closes the remaining in-scope Proposal 170 defects without
expanding the project into missing tunnel data planes, broad router inspection,
or unrelated infrastructure.

The retained M020–M027 implementation substantially corrects the API contract.
The current corrective work is narrower:

- restore the M027/M020–M027 chronology after a later merge revived superseded
  M019 closure language;
- make the M022 AddressBook control-state owner active only when I2PControl is
  both compiled and enabled;
- prove that disabled/default router behavior remains independent of Proposal
  170 control state;
- independently review the final corrected head;
- retain a truthful support disposition that distinguishes exact implemented
  dimensions from unavailable sources and unsupported runtimes.

## 2. Current-state evidence

### 2.1 Retained implementation

The following work remains retained candidate evidence:

- standard `Authenticate` parameters, numeric `API`, `params.Token`, exact
  I2PControl error inventory, notification execution, and strict request IDs;
- direct base and Proposal 170 RouterInfo selector compatibility;
- seven lowercase TunnelManager actions and twelve exact types;
- canonical TunnelManager `status`, `results`, `info`, and `rawConfig` shapes;
- failure-atomic tunnel persistence and secret-safe response serialization;
- runtime-backed four-book AddressBook behavior while the control owner is
  active;
- startup-managed tunnel inventory, collision guards, and proxy exit
  observation;
- bounded recoverable SAM observation;
- exact 43-selector RouterInfo contract/source matrix;
- literal external-contract fixtures;
- explicit unsupported behavior for missing tunnel data planes.

### 2.2 Retained source disposition

The frozen RouterInfo matrix remains:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

M026 found no additional source that could be exposed without new historical
telemetry, semantic invention, broad core ownership changes, or fabricated
values. M028/M029 do not repeat that audit.

### 2.3 Current defects

At `03a384a`:

1. planning and support documents identify M019 as current closure even though
   M019 predates M020–M027 and had been superseded;
2. top-level status claims `closed against pinned revision` while M027 selected
   `partial Proposal 170 support`;
3. `AddressBookManager::new` always constructs the Proposal 170 runtime control
   owner;
4. normal startup can read `control-state.json` and rebuild ordinary lookup from
   it without an active I2PControl service;
5. normal downloader persistence calls the control owner and can create/update
   control-state files in disabled/default execution;
6. `AddressBookHandle` always carries Proposal 170 mutation/persistence state;
7. `serde_json` is unconditional in the CLI after M022.

The closure invalidation is:

- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`

## 3. Scope boundary

### 3.1 In scope

- restoring current planning/support chronology and status;
- marking M019 superseded/non-controlling;
- compile-time and runtime isolation of the existing AddressBook control owner;
- preserving enabled-mode M022 single-authority behavior;
- preserving default/disabled legacy AddressBook behavior;
- restoring optional dependency ownership where applicable;
- focused feature-disabled, runtime-disabled, enabled, restart, and transition
  regressions;
- independent final-head reclosure;
- directly affected documentation and static guards.

### 3.2 Out of scope

- missing HTTP, IRC, SOCKS-IRC, CONNECT, Streamr, bidirectional, or other tunnel
  data planes;
- new lifecycle supervisors for startup-managed tunnels;
- new RouterInfo sources, samplers, rolling windows, polling loops, peer
  classifications, NetDB inspection, or fabricated defaults;
- router, transport, peer-selection, streaming, LeaseSet, cryptographic, SAM,
  resolver, downloader-policy, frontend, or configuration redesign;
- generic owner registries, event buses, plugin systems, schema frameworks, or
  second AddressBook authorities;
- persistence schema migration hidden inside the corrective pass;
- new dependencies;
- CI, release, packaging, coverage, fuzzing, soak testing, platform matrices, or
  generated evidence bundles;
- upstream review, submission, adoption, or merge activity.

## 4. Internal-only external-interaction boundary

All writes must remain in `eggstack/emissary`.

No milestone may:

- write to an upstream or third-party repository;
- open or update upstream issues, pull requests, merge requests, reviews,
  discussions, or proposals;
- request upstream review, approval, feedback, adoption, or merge;
- push branches, commits, tags, patches, artifacts, or releases upstream;
- contact upstream maintainers;
- prepare a contribution package, patch series, submission checklist, or merge
  plan.

Read-only source/specification inspection is permitted for internal correctness.
Violation is a stop condition and invalidates affected evidence.

## 5. Target architecture

### 5.1 Default/disabled mode

When I2PControl is not compiled, or is compiled but runtime-disabled:

- ordinary AddressBook loading uses legacy `addresses` and destination files;
- subscription downloads update legacy sources only;
- Proposal 170 control-state files are not read, written, migrated, or
  consulted;
- stale control state cannot affect lookup;
- no Proposal 170 mutation handle exists;
- control-state files from earlier enabled use remain untouched on disk.

### 5.2 Enabled mode

When I2PControl is compiled and runtime-enabled:

- one purpose-specific control owner is constructed;
- it shares the live lookup maps used by the router;
- it loads current/backup control state and applies the existing migration
  rules;
- downloads merge through that owner;
- successful Proposal 170 mutations are durable before response and immediately
  visible to normal lookup;
- I2PControl receives a dedicated control handle;
- no shadow store or second authority exists.

### 5.3 Feature ownership

Proposal 170-only serialization, persistence, and control types should compile
only with `i2pcontrol` where practical. `serde_json` should return to optional
feature ownership unless an independently required unconditional CLI consumer
exists.

## 6. Capability and evidence dimensions

Every claim remains classified independently:

| Dimension | Meaning |
|---|---|
| Wire | exact names, casing, presence rules, response fields, and JSON types |
| Source | truthful current Emissary owner exists |
| Runtime | a real backend/service performs the operation |
| Persistence | mutation is durable and failure-atomic |
| Feature isolation | disabled/default execution is unaffected by the administrative feature |
| Evidence | literal fixtures plus failure/restart/composition/transition proof |

Compatibility aliases, unavailable sources, stored definitions, and unsupported
runtime stubs are not operational coverage.

## 7. Dependency sequence

```text
M020–M027 retained corrective implementation/evidence
                    |
                    v
M028 post-M027 status and AddressBook feature isolation
                    |
                    v
M029 independent in-scope conformance reclosure
```

M028 is ready. M029 is blocked until M028 closes with a frozen implementation
and test head.

## 8. Milestones

### M020–M027 — Retained corrective sequence

Status: retained evidence; M027 final disposition invalidated pending M029

Summary:

- M020: base I2PControl/JSON-RPC interoperability;
- M021: TunnelManager wire, validation, atomicity, and secrets;
- M022: enabled-mode runtime AddressBook authority;
- M023: startup tunnel inventory and ClientServicesInfo truthfulness;
- M024: recoverable bounded SAM observation;
- M025: exact RouterInfo contract/source matrix;
- M026: bounded-source audit with no feasible additional sources;
- M027: literal conformance evidence and partial-support disposition.

Do not reopen these areas without a newly demonstrated defect.

### M028 — Post-M027 status and AddressBook feature isolation

Plan:

- `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`

Status: ready

Objective:

- restore M027/M020–M027 chronology and supersede M019;
- isolate control state behind compile-time and runtime enablement;
- restore default/disabled legacy AddressBook behavior;
- retain enabled-mode M022 single-authority semantics;
- restore optional dependency ownership where applicable;
- add focused disabled/enabled/transition evidence;
- freeze a corrected implementation/test head.

Exit conditions:

- disabled/default execution never consults Proposal 170 control state;
- enabled execution retains durable coherent four-book behavior;
- disable/re-enable transitions are documented and tested;
- top-level status is no longer overstated;
- no unrelated Proposal 170 behavior changes;
- M028 closure is accepted and M029 becomes ready.

### M029 — In-scope Proposal 170 conformance reclosure

Plan:

- `plans/implementation/i2pcontrol-proposal-170/029-in-scope-conformance-reclosure.md`

Status: blocked on M028

Objective:

- independently refetch the pinned external contract;
- review the actual final M028 head;
- revalidate feature isolation and retained M020–M027 behavior;
- classify all changed files against scope;
- run focused and package-scoped verification;
- reconcile documentation and registry chronology;
- choose the truthful final status.

Expected final disposition under current scope:

- `partial Proposal 170 support` with zero unresolved high/medium defects.

`closed internally against pinned revision` is allowed only if every source and
runtime dimension is actually available and evidenced. M028/M029 do not
authorize work needed to make the 26 unavailable RouterInfo selectors or missing
tunnel data planes operational.

## 9. Cross-cutting invariants

1. Existing I2PControl clients continue to work without modification.
2. Canonical Proposal 170 names, types, presence rules, and response shapes are
   exact.
3. Compatibility forms remain isolated from canonical accounting.
4. Protected work authenticates before mutation or expensive source assembly.
5. Unsupported tunnel types remain resource-free and never report running.
6. Missing tunnel data planes remain out of scope.
7. Startup-owned tunnel definitions remain read-only and unshadowable.
8. Persistent mutation is durable before success and failure-atomic.
9. Enabled AddressBook success affects the actual managed lookup source.
10. Disabled/default AddressBook execution ignores Proposal 170 control state.
11. No control-only mutation authority leaks through ordinary runtime traits.
12. ClientServicesInfo state remains tied to actual lifecycle/provenance.
13. SAM observation remains bounded, passive, and recoverable.
14. RouterInfo unavailable data remains explicit and unfabricated.
15. No new core algorithm, polling task, framework, dependency, CI, release, or
    upstream activity is introduced.

## 10. Failure, restart, cancellation, and contention policy

- Disabled mode does not fail on corrupt/stale control-state files because it
  does not read them.
- Enabled mode retains current/backup recovery and fails explicit activation if
  both are unusable.
- Failed AddressBook or TunnelManager mutation leaves prior durable/live state.
- Download failure does not erase prior legacy or control state.
- Disabling preserves but ignores control state; re-enabling reloads it.
- No lock is held across network download awaits.
- Existing mutation serialization remains bounded and single-owner.
- Cancellation before publication leaves prior state; response loss after
  publication may expose the committed state on retry.
- Observation/source/response-bound failures return no partial success.

## 11. Compatibility and migration

- Existing router configuration remains valid.
- Existing legacy address files remain authoritative in disabled mode.
- Existing M022 control-state files remain readable in enabled mode.
- No control-state schema change is authorized.
- Disabling I2PControl intentionally removes control-only entries from active
  lookup for that run without deleting their files.
- Re-enabling restores retained control state under the documented precedence.
- Header token and other retained compatibility forms remain unchanged.
- Startup configuration remains authoritative for startup-managed tunnels.

A required persistence schema migration blocks M028 and requires a separate
maintainer-authorized plan.

## 12. Verification policy

M028 required local package scope:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features address_book
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

M029 additionally revalidates focused TunnelManager, ClientServicesInfo,
RouterInfo, production-composition, conformance-manifest, literal-fixture, and
core SAM tests listed in its plan.

Use touched-file formatting checks. Do not add remote CI, platform matrices,
release checks, coverage gates, fuzzing, network farms, soak tests, or generated
evidence infrastructure.

## 13. Milestone status

| Milestone | Status | Disposition |
|---|---|---|
| 001–019A | historical/superseded/invalidated as recorded | retained history only |
| 020 | retained closed evidence | base interoperability |
| 021 | retained closed evidence | TunnelManager correction |
| 022 | retained implementation; feature boundary reopened | enabled-mode AddressBook authority |
| 023 | retained closed evidence | startup/service truthfulness |
| 024 | retained closed evidence | SAM recovery |
| 025 | retained closed evidence | RouterInfo matrix |
| 026 | retained closed evidence | no feasible additional sources |
| 027 | final disposition invalidated; evidence retained | literal fixtures and partial-support review |
| 028 | ready | status and AddressBook feature-isolation corrective pass |
| 029 | blocked | independent final-head reclosure |

## 14. Completion definition

This workstream is complete only when M029 records that:

- M028 is correctly implemented and reviewed at the actual final head;
- disabled/default AddressBook behavior is independent of control state;
- enabled AddressBook behavior remains one coherent durable authority;
- standard I2PControl and Proposal 170 wire behavior remains exact;
- TunnelManager, ClientServicesInfo, SAM, and RouterInfo retained evidence passes;
- the 16/1/26 source matrix is truthful;
- missing tunnel data planes remain explicit unsupported behavior;
- no high/medium correctness, security, compatibility, or claim defect remains;
- documentation and planning chronology is consistent;
- no scope expansion or upstream interaction occurred.

Under the current authorized scope, the expected honest completion label is
`partial Proposal 170 support`. The roadmap does not equate parser coverage,
explicit errors, administrative definitions, or unsupported stubs with full
operational implementation.
