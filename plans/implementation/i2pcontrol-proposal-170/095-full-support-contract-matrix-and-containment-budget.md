# M095 — Full-Support Contract Matrix and Containment Budget

Status: ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`;
- retained ADR-0001/0002/0003 ownership and security rules;
- M061/M062/M063 containment authorities;
- M093 current tunnel production/security reclosure authority.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207`.

Pinned external contract: I2P Proposal 170, status `Open`, revision `2026-05-20`.

Classification: invariant / infrastructure; no-production-behavior milestone.

## 1. Objective

Create the single authoritative, machine-readable full-support matrix and exact owner/path budget for the newly authorized completion phase before any M096-M103 production work begins.

The current repository has several truthful partial-support authorities that were created for different phases:

- RouterInfo 43 total / 37 available / 1 neutral / 5 unavailable;
- the AddressBook SetConfig thirteen-key disposition;
- TunnelManager's canonical option inventory and backend-specific apply/reject capability tables;
- all twelve real tunnel backends;
- six ClientServicesInfo selectors;
- accepted source/dependency containment manifests.

M095 must reconcile these into one current-head contract/applicability matrix against the pinned proposal. It must not infer completion from parsers, raw-config round-trip, or historical closure text.

## 2. Required output

Create:

`plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`

The matrix must be deterministic and reviewable without executing the router. It must include at least four sections.

### A. RouterInfo

For all 43 Proposal 170 additions record:

- canonical key;
- JSON return type;
- current disposition (`available`, `neutral`, `unavailable`);
- production owner/source;
- I2PControl adapter/serializer;
- evidence plan/closure;
- final target disposition;
- owning completion milestone if not final;
- whether any non-I2PControl production path is required.

The five current unavailable rows must map exactly to M100-M103 work:

- `i2p.router.net.bw.transit.15s` -> M100;
- `i2p.router.news` -> M101;
- `i2p.router.net.error` -> M102;
- `i2p.router.net.error.v6` -> M102;
- `i2p.router.netdb.bannedpeers` -> M103.

### B. AddressBook SetConfig

For all 13 pinned keys record:

- key name and expected value type/string semantics;
- current disposition;
- whether it is behaviorally meaningful, path-valued, or administrative metadata;
- active runtime/persistence consumer required for a successful SetConfig;
- confinement/migration requirement;
- owning M096 work package.

No key may remain `unknown` or `accept_inert`.

### C. TunnelManager options

For every canonical option in the pinned Proposal 170 TunnelManager inventory and each of the twelve tunnel types record one of:

- `apply` — already operational;
- `planned_apply` — applicable and assigned to M097/M098/M099;
- `not_applicable` — the option does not apply to the type, with spec/reference rationale;
- `blocked_primitive` — applicable but no supported current primitive exists; must name the exact missing primitive and blocking milestone.

Do not use `parser_supported`, `round_trip`, or `recognized` as a completion disposition.

Each row must also record whether the value is security-sensitive, secret-bearing, path-bearing, or identity/key-affecting.

The matrix must distinguish canonical Proposal 170 keys from historical compatibility aliases and internal `i2p.*` spellings.

### D. ClientServicesInfo and method inventory

Record all six Proposal 170 selectors and their production owner/evidence. Confirm no additional Proposal 170 selector is missing.

Separately record broader/base I2PControl methods as `outside_proposal_170_scope` so later agents do not turn this phase into general API completion.

## 3. Readiness/current evidence

M094 is closed. There is no active production/security corrective handoff. M093 remains current production/security authority.

ADR-0004 explicitly authorizes a new completion phase while preserving historical partial closures. Therefore M095 is dependency-ready and does not need to reopen or invalidate M051/M056/M072/M093 merely to inventory their current consequences.

## 4. Authorized path boundary

M095 may modify only planning, support documentation, and matrix/static-evidence paths:

- this plan;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` for status/evidence reconciliation only;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `docs/i2pcontrol/proposal-170-support.md` only to link the new matrix/phase without changing a partial-support claim to full;
- `docs/i2pcontrol/proposal-170-conformance.md` if needed to reconcile the matrix authority;
- one focused test/guard under `emissary-cli/tests/**` if needed to validate machine-readable matrix exhaustiveness;
- `emissary-cli/tests/m062_dependency_containment.rs` only for exact planning-path bookkeeping required by the existing cumulative containment guard;
- the M095 closure record when implementation is complete.

No `emissary-cli/src/**`, `emissary-core/**`, manifest, lockfile, runtime, dependency, workflow, or frontend change is authorized.

## 5. Invariants

1. Proposal revision remains pinned to `2026-05-20`.
2. Current partial dispositions remain truthful until their owning implementation milestone closes.
3. Historical closure evidence is cited, not rewritten.
4. An option is not complete merely because it parses or persists.
5. `not_applicable` requires explicit rationale; it cannot hide missing semantics.
6. `blocked_primitive` names an exact primitive, not a vague need for core access.
7. No new core path is authorized by placing it in the matrix.
8. Compatibility extensions do not count toward canonical completeness.
9. Broader/base I2PControl methods remain out of scope.
10. No upstream interaction occurs.

## 6. Explicit non-goals

M095 MUST NOT:

- implement any RouterInfo source;
- implement SetConfig behavior;
- change TunnelManager option behavior;
- change a backend registry/runtime/filter;
- modify AddressBook persistence/runtime;
- add timers, news fetchers, transport state, ban state, or session options;
- modify production source or dependency state;
- revise M088/M093 security conclusions;
- add CI/fuzz/coverage/release machinery;
- prepare or request upstream review/submission.

## 7. Ordered work packages

### A. Freeze the external contract

Read the official Proposal 170 source for the pinned revision and record a source hash or equivalent stable revision evidence where practical. Confirm the proposal is still Open and that the repository is intentionally implementing the `2026-05-20` text.

Do not silently incorporate later draft edits if the proposal changes during M095; record a delta and stop for maintainer disposition.

### B. Reconcile RouterInfo

Use current production code plus M045-M057 closures to populate all 43 rows. Verify no historical 40/1/2 source claim leaks into the current 37/1/5 baseline.

### C. Reconcile AddressBook

Inspect the exact current SetConfig parser/disposition and runtime command seam. Classify all thirteen keys and identify the concrete runtime/persistence owner M096 must add or update.

### D. Reconcile TunnelManager

Start from the canonical option-key inventory in `tunnel_manager.rs` and the backend option-capability tables. For each of the twelve types, classify every option cell from actual runtime behavior.

Inspect Yosemite/SAM APIs used by the current backends before marking a cell `blocked_primitive`.

### E. Reconcile ClientServicesInfo and method scope

Verify six selectors and record current evidence. Explicitly separate unrelated base methods.

### F. Produce containment budgets

For M096-M101, target `emissary-cli/src/i2pcontrol/**` only unless a current existing neutral CLI adapter is demonstrably necessary.

For M102, identify the smallest existing allowed M061 owner/inspection paths that could hold/transport neutral network-error state. The matrix is not authorization to edit them; M102's plan is the authorization once dependency-ready.

For M103, determine whether a genuine enforced ban/exclusion owner exists. Do not invent a ban subsystem in M095.

## 8. Failure/cancellation/restart/contention semantics

None are changed by M095. If matrix construction discovers a behavior defect that requires production correction, record it as a finding and assign it to the correct existing/new milestone rather than editing production under M095.

## 9. Compatibility/migration/security effects

No runtime compatibility or migration effect.

Security benefit is planning exactness: a future implementation agent receives explicit path and option/source budgets rather than broad permission to modify core.

The matrix itself must not contain real passwords, private keys, destination private material, tokens, or secret file contents.

## 10. Focused tests and static guards

If a machine-readable guard is added, it should assert at least:

- 43 unique RouterInfo rows;
- 13 unique SetConfig keys;
- exactly 12 canonical tunnel types;
- exactly 7 canonical actions;
- exactly 6 ClientServicesInfo selectors;
- every TunnelManager option/type cell has a known disposition;
- no duplicate canonical key;
- no `unknown` final disposition;
- current RouterInfo baseline totals 37/1/5 before later milestones change it.

The guard must not hard-code transient test counts or require network access.

## 11. Verification

At minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Run any new focused matrix guard directly. No broad core/runtime test matrix is required because M095 changes no production behavior.

Review changed paths against Section 4.

## 12. Acceptance and stop conditions

M095 closes only when:

- the machine-readable matrix exists and is exhaustive;
- every current partial cell has an owning later milestone or explicit accepted current evidence;
- every TunnelManager cell is classified from runtime semantics rather than parser presence;
- M096-M103 path/owner assumptions are concrete enough for bounded handoff;
- M102 core candidate paths are explicit and minimal;
- M103 has a documented source audit strategy that does not authorize a ban algorithm;
- support docs remain partial, not prematurely full;
- no production/dependency path changed;
- no upstream interaction occurred.

Stop rather than guess if:

- the official proposal has changed from the pinned revision and the delta is material;
- an option's applicability cannot be established from the proposal/reference behavior;
- a lower-layer requirement cannot be bounded to a neutral owner;
- full support appears to require a new router algorithm not authorized by ADR-0004.

## 13. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/095-closure.md` containing:

- planning baseline and implementation/closure heads;
- changed-path matrix;
- exact proposal revision evidence;
- machine-readable row counts and duplicate/unknown checks;
- RouterInfo 43-row reconciliation;
- 13-key SetConfig reconciliation;
- full option/type applicability summary and blocked primitive list;
- ClientServicesInfo/method-scope reconciliation;
- M096-M103 owner/path budgets;
- matrix/static guard outcomes;
- `m062_dependency_containment` and `git diff --check` outcomes;
- unresolved findings/severity;
- explicit no-production-change attestation;
- explicit internal-only/no-upstream attestation.

After M095 closure, registry may advance only the dependency-ready next milestone(s) allowed by the roadmap. M095 itself does not claim improved production support.

## 14. Internal-only rule

All writes remain within `eggstack/emissary`. External specification/reference repositories are read-only evidence only. No upstream issue, PR, review, submission, merge, contribution preparation, or maintainer contact is authorized.