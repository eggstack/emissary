# M039 — Proposal 170 Operational Final-Head Reclosure

Status: ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Applicable governance and decisions:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`

Repository baseline:

- accepted M038 implementation/evidence head: `a5864d2`

Hard dependency:

- M038 closed; implementation disposition and closure accepted

## 1. Bounded objective

Perform an independent final-head review of the complete M031–M038 workstream
and select the truthful Proposal 170 subsystem disposition.

M039 is review and closure only. It does not implement production fixes. Any
new high/medium correctness, security, compatibility, ownership, containment,
or evidence defect requires a distinct corrective plan and invalidates closure.

## 2. Review dimensions

Review each dimension separately:

- wire — exact methods, selectors, actions, types, casing, presence semantics,
  response shapes, and errors;
- source — truthful current source for each claimed field;
- runtime — real backend behavior for generic client/server and explicit
  unsupported behavior elsewhere;
- persistence — definitions, server identity, AddressBook state, recovery, and
  documented durability;
- lifecycle — start/stop/restart/StartOnLoad/edit/delete/rename/All/failure;
- compatibility — direct Proposal 170 versus nested/base behavior;
- feature isolation — no-feature and runtime-disabled behavior;
- security — authentication, throttling, TLS, secrets, paths, bounds, locks;
- containment — production changes outside `i2pcontrol/**` and core passive seam;
- evidence — unit, fixtures, production composition, and live-runtime run;
- governance — scope, registry chronology, and no-upstream compliance.

No aggregate test count substitutes for dimension-specific evidence.

## 3. Required invariants

1. M020–M030 retained evidence remains valid or is explicitly superseded by
   stronger evidence.
2. Generic `client` and `server` are the only real control-plane tunnel
   backends unless a separately authorized plan changed that fact.
3. Remaining tunnel types are exhaustive unsupported backends and resource-free.
4. Startup-managed tunnels remain externally owned.
5. AddressBook successful setters have real runtime effect; unsupported config
   fields fail truthfully.
6. RouterInfo remains 16 available / 1 neutral / 26 unavailable unless a
   separately authorized source plan changed the matrix.
7. Direct and compatibility request modes are distinct and literal fixtures
   pass.
8. Authentication and persistence guarantees match documentation.
9. Proposal 170 policy outside `i2pcontrol/**` is minimal and justified.
10. Live-runtime evidence uses production composition.
11. No upstream interaction occurred.

## 4. Required review inputs

- M031–M038 implementation dispositions and closure records;
- frozen implementation/evidence heads;
- changed-file comparison from M030 baseline to final head;
- current ADRs, roadmap, registry, support/conformance/source maps;
- exact local verification output;
- live-runtime M038 evidence;
- unresolved findings from every milestone;
- external Proposal 170 page rechecked read-only for revision/status changes.

If the proposal changed after the pinned revision, M039 is blocked pending a
contract-rebase plan.

## 5. Review procedure

### WP1 — Freeze and compare the final head

Record the exact final commit and verify a clean working tree in the execution
environment. Compare production/test/docs/plans against M030 baseline and
classify every changed path.

### WP2 — Revalidate wire and compatibility

Run literal contract fixtures and independently inspect the canonical and base
inventories, exact overlaps, and dispatcher methods.

### WP3 — Revalidate runtime and lifecycle

Review real backend registration, startup ownership, supervisor state,
server-secret ownership, StartOnLoad, edit/delete/restart/All, failure cleanup,
and unsupported resource guards.

### WP4 — Revalidate AddressBook and RouterInfo

Confirm M028/M030 owner/isolation behavior, M034 setter truthfulness, exact
source matrix, and unavailable-selector failure semantics.

### WP5 — Revalidate security and persistence

Review authentication primitive/throttle, request/connection bounds, secret
redaction, path confinement, publication/recovery, cancellation, and lock
boundaries.

### WP6 — Revalidate containment

List every production file outside `emissary-cli/src/i2pcontrol/**` changed by
the workstream. For each, identify the narrow runtime hook and verify no Proposal
170 policy remains without direct runtime-owner necessity.

### WP7 — Revalidate live evidence

Rerun or independently inspect the M038 child-process scenario. A fake-only
substitute is not accepted.

### WP8 — Select disposition

Allowed dispositions:

- `partial Proposal 170 support` — every claimed implemented dimension is exact
  and evidenced, but unavailable RouterInfo sources and/or unsupported tunnel
  families remain;
- `corrective pass required` — unresolved high/medium defect in a claimed
  dimension;
- `blocked` — external revision changed or required evidence cannot be obtained.

`closed internally against pinned revision` is allowed only if every proposal
source/runtime dimension is actually operational and evidenced; it is not
expected under this roadmap.

## 6. Required verification matrix

At minimum:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test golden_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

If M037 retains or changes a core SAM hook:

```bash
cargo check -p emissary-core
cargo test -p emissary-core sam
cargo clippy -p emissary-core --all-targets -- -D warnings
```

Use targeted formatting and `git diff --check`. Record exact command outcomes,
including pre-existing failures. Do not add remote CI or generated evidence.

## 7. Requirement-to-evidence matrix

The final closure record must include one row for at least:

- authentication and token behavior;
- failed-login throttling;
- direct/base compatibility;
- each Proposal 170 method family;
- generic client backend;
- generic server backend and secret identity;
- unsupported tunnel families;
- startup-managed ownership;
- StartOnLoad/restart/delete/edit/All/failure recovery;
- AddressBook entry owner coherence;
- AddressBook setter truthfulness;
- RouterInfo 43-field source classification;
- ClientServicesInfo actual sources;
- persistence/recovery/durability;
- feature isolation;
- containment boundary;
- live-runtime validation;
- no-upstream compliance.

Each row names exact code/tests/commands and result. Documentation-only claims
are insufficient.

## 8. Failure, compatibility, migration, and security review

The closure must explicitly state:

- known failure recovery and whether router restart/store deletion is ever
  required;
- cancellation and stale-generation behavior;
- existing data-file compatibility/migrations;
- activation of `StartOnLoad` behavior;
- unsupported base methods and tunnel families;
- path/private-key/password/token handling;
- platform durability qualifications;
- changed-path/core impact;
- residual risk with severity.

## 9. Closure artifacts

Create:

- `plans/closure/i2pcontrol-proposal-170/039-closure.md`.

Update:

- `plans/registry.md`;
- subsystem roadmap status;
- implementation README;
- support/conformance/security documentation;
- any source/runtime matrices directly affected.

The closure must include an internal-only attestation:

- external sources were read-only;
- no upstream repository/channel was mutated;
- no review/merge/adoption/submission was requested;
- no contribution artifact was prepared.

## 10. Acceptance criteria

M039 may select `partial Proposal 170 support` only when:

- every implemented/claimed dimension passes independent review;
- generic client/server are operational and isolated;
- unsupported/unavailable dimensions remain explicit;
- no high/medium defect remains;
- documentation and registry chronology are consistent;
- live-runtime evidence passes or contains only clearly external, non-claimed
  blockers;
- changed paths satisfy the containment policy;
- no upstream interaction occurred.

## 11. Stop conditions

M039 must stop and select `corrective pass required` or `blocked` if:

- any claimed runtime is stubbed/fabricated;
- a successful setter is inert;
- startup ownership is ambiguous;
- task failure requires router restart/store deletion to recover;
- secrets or arbitrary paths cross the API boundary;
- direct/base compatibility regresses;
- live-runtime evidence cannot be obtained;
- containment changes add core behavior;
- proposal authority changed materially;
- upstream interaction occurred.
