# I2PControl Proposal 170 Containment Corrective Roadmap

Status: active; M063 ready; M061 source containment closed; M062 production dependency correction accepted with closure/evidence corrective required

Original planning baseline: `adb2f52543764b267b2bcb282d093111001ae4b2` — merged M057 closure head

M062 planning head: `a0d9f2dcc15fdeb5fcbe6658c0399ff9c8c9575b`

M062 implementation/closure commit and M063 planning baseline: `fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c`

Upstream comparison baseline: `eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f` — pinned fork merge base/read-only comparison authority

Source workstream:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` — source-completion/truthfulness work through M057;
- accepted RouterInfo disposition remains 43 total / 37 available / 1 protocol-permitted neutral / 5 unavailable.

Canonical and governance references:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/closure/i2pcontrol-proposal-170/037-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/056-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/057-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/058-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/059-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/060-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/061-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/062-closure.md`;
- `plans/implementation/i2pcontrol-proposal-170/063-m062-closure-and-feature-guard-corrective.md`.

Pinned external contract:

- I2P Proposal 170 `I2PControl Expansion`, revision `2026-05-20` as already pinned by the accepted workstream;
- external sources and upstream repository remain read-only and do not authorize upstream interaction.

## 1. Purpose and ownership boundary

The Proposal 170 implementation is operational for its supported surface and keeps control-plane policy under `emissary-cli/src/i2pcontrol/**`. M058–M061 closed the main physical source-containment problem by reducing and then statically enforcing the minimum justified non-`i2pcontrol` source delta.

M062 correctly closed one residual production containment class: direct Cargo dependency ownership. `subtle`, used by I2PControl authentication for constant-time comparison, is now a local optional `emissary-cli` dependency activated explicitly by `i2pcontrol`; the root workspace declaration is gone; no production Rust source or lockfile changed.

A post-M062 review found bounded closure/evidence defects rather than a production dependency regression:

- the M062 implementation plan retained `Status: ready`;
- roadmap/registry text mislabeled the M062 planning head `a0d9f2d` as the closure/current head;
- lifecycle status text disagreed between planning control surfaces;
- the durable M062 guard rejects direct forbidden activation of `subtle` but does not compute transitive local Cargo feature reachability, so `ui -> i2pcontrol -> dep:subtle` is not currently rejected by the persistent test.

M063 is the sole corrective for those items. It may modify planning records and the existing dependency-containment test only. It does **not** reopen the accepted source boundary, dependency manifests, RouterInfo source completeness, unsupported tunnel types, authentication behavior, or runtime/core architecture.

The governing containment model remains two-layered:

- **source containment:** M061 is authoritative for changed source paths outside `emissary-cli/src/i2pcontrol/**`;
- **dependency containment:** M062 defines direct dependency ownership, with M063 strengthening the durable guard to enforce that ownership across transitive local-feature composition.

The target remains **minimum justified delta**, including both source and direct dependency surfaces.

## 2. Work classification

### Invariants

- Supported Proposal 170 behavior, exact wire spelling/types, authentication/TLS behavior, persistence semantics, AddressBook owner coherence, ClientServicesInfo truthfulness, and supported client/server tunnel lifecycle remain unchanged.
- The accepted RouterInfo matrix remains 37 available / 1 neutral / 5 unavailable.
- Unsupported tunnel data planes remain unsupported and resource-free.
- M061 source-boundary files and accepted source-path disposition remain unchanged.
- Root/package Cargo manifests and `Cargo.lock` remain unchanged by M063.
- Default/no-I2PControl execution must not activate an I2PControl-only direct dependency, including indirectly through another local feature.
- No router, peer-selection, NetDB, tunnel construction/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP algorithm change is authorized.
- No upstream write/review/submission activity is authorized.

### Infrastructure

Completed through M061:

- complete non-`i2pcontrol` fork-delta ledger;
- narrowed original CLI adapters;
- consolidated neutral core observation boundary;
- exact current source manifest and static guard.

M062 production result:

- machine-readable Cargo dependency ownership authority;
- optional/local ownership of the I2PControl-only `subtle` CLI dependency;
- lockfile/no-resolution-churn evidence.

M063 adds only:

- transitive local-feature reachability enforcement in the existing M062 test;
- corrected lifecycle/head records for M062/M063.

### Polish

M063 corrects stale status/SHA text only. It is not a general planning cleanup.

### Capabilities

No new external capability is created. Authentication continues to use the reviewed `subtle` constant-time primitive when I2PControl is enabled.

## 3. Explicit non-goals

M063 must not:

- implement news, banned-peer ownership, transit 15-second sampling, v4/v6 network-error ownership, or any other currently unavailable RouterInfo source;
- implement HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, or any other missing tunnel data plane;
- add new I2PControl methods, aliases, selectors, statuses, fields, or compatibility extensions;
- change `Cargo.toml`, `emissary-cli/Cargo.toml`, `Cargo.lock`, `emissary-cli/src/**`, `emissary-core/**`, `emissary-util/**`, examples, runtime configuration, or persistence code;
- replace the `subtle` primitive with hand-written constant-time logic;
- perform a repository-wide dependency cleanup or version refresh;
- extract the implementation into a new Cargo crate merely for aesthetic isolation;
- add a generalized feature-analysis framework or production/build-time dependency;
- add CI, release automation, coverage, fuzz, soak, platform matrices, or generated evidence bundles;
- prepare or seek an upstream merge, review, issue, pull request, proposal update, or maintainer contact.

## 4. Current state

### 4.1 Accepted source containment

M058 inventoried 47 non-I2PControl production/example paths. M059 moved AddressBook administrative policy into `emissary-cli/src/i2pcontrol/**` and restored unnecessary original-CLI differences without touching core. M060 reduced the accepted 32-path core budget to 23 retained neutral owner/inspection paths and restored nine paths to upstream. M061 independently accepted nine original CLI/runtime source paths plus those 23 core paths and installed an exact-path static guard.

That source disposition remains accepted and is not reopened by M063.

### 4.2 Accepted M062 production dependency correction

At M062 baseline `a70dd3ac`, root `Cargo.toml` carried a workspace `subtle` declaration and `emissary-cli/Cargo.toml` consumed it unconditionally. M062 corrected that state at `fac2a0c`:

- root `Cargo.toml` no longer declares `subtle`;
- `emissary-cli/Cargo.toml` declares `subtle = { version = "2.6.1", default-features = false, optional = true }`;
- `i2pcontrol = [..., "dep:subtle"]` explicitly activates the optional dependency;
- `emissary-core` continues to declare `subtle` with a literal version for its independent DSA consumer;
- `Cargo.lock` remained byte-identical to the M062 planning baseline;
- no production Rust source changed.

This production disposition is frozen under M063.

### 4.3 M062 closure/evidence defects

The post-M062 review found:

1. stale `Status: ready` in the M062 implementation plan;
2. stale/mislabeled `a0d9f2d` closure-head text in roadmap/registry;
3. lifecycle status disagreement between planning records;
4. incomplete durable feature-activation guard: direct forbidden activation is checked, but transitive local-feature reachability is not.

M063 exists only to close these four defects.

## 5. Target architecture and durable boundary

### 5.1 Source owner

All Proposal 170/I2PControl administrative, wire, validation, persistence, security, support, and aggregation policy remains under `emissary-cli/src/i2pcontrol/**`, as accepted by M061.

### 5.2 Source boundary

The M061 source boundary remains unchanged. M063 may not change production Rust source.

### 5.3 Dependency boundary

A direct dependency whose only direct consumer is I2PControl feature-gated code must:

1. be optional at the package that owns the feature;
2. be activated explicitly by `i2pcontrol`;
3. not be directly or indirectly activated by `default`, `ui`, `metrics`, or another unrelated local feature;
4. not be promoted to workspace scope solely for I2PControl convenience unless another independently justified direct workspace consumer exists.

The persistent guard must compute local Cargo feature reachability rather than checking only literal membership in each root feature list.

For an optional dependency target such as `subtle`, the guard must recognize independent activation through `dep:subtle` and dependency-feature syntax such as `subtle/feature`; weak `subtle?/feature` alone does not independently activate the dependency. Local feature cycles must terminate through a visited set.

This rule concerns **direct dependency activation**, not transitive crate-name presence in the resolved dependency graph.

### 5.4 Combined containment authority

After M063 closure:

- source-path authority remains `061-containment-boundary.toml` plus `m061_containment.rs`;
- dependency authority remains `062-dependency-containment.toml` plus the strengthened `m062_dependency_containment.rs`;
- M063 closure records the correction without rewriting M062 historical closure evidence.

## 6. Dependency graph

```text
M057 — source/truthfulness planning closure
   |
   v
M058 — non-i2pcontrol delta inventory — CLOSED
   |
   v
M059 — original CLI/runtime containment — CLOSED
   |
   v
M060 — core observation containment — CLOSED
   |
   v
M061 — source containment reclosure/static guard — CLOSED
   |
   v
M062 — dependency-surface production corrective — PRODUCTION ACCEPTED; CLOSURE/EVIDENCE CORRECTIVE REQUIRED
   |
   v
M063 — closure consistency + indirect feature-activation guard — READY
```

M051 remains independently blocked in the source-completion roadmap and is not a dependency of M063.

## 7. Milestones

### M058 — Non-I2PControl fork-delta inventory and containment ledger — closed

Result: complete 47-path inventory, exact M059/M060 budgets, zero production behavior changes.

### M059 — Original CLI/runtime adapter containment — closed

Result: AddressBook administrative policy contained under I2PControl; original CLI/runtime reduced to composition/neutral adapters; no core changes.

### M060 — Core observation seam consolidation — closed

Result: core budget reduced from 32 to 23 retained paths; nine paths restored to upstream; only bounded neutral owner-local observations retained.

### M061 — Independent source containment reclosure — closed

Result: exact current source boundary accepted and enforced with no production changes.

### M062 — I2PControl dependency-surface containment corrective — production accepted; closure/evidence corrective required

Plan: `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md`.

Historical closure: `plans/closure/i2pcontrol-proposal-170/062-closure.md`.

Production result at `fac2a0c`: dependency ownership corrected exactly as intended, with no production source or lockfile changes.

Remaining closure defects are delegated exclusively to M063.

### M063 — M062 closure consistency and indirect feature-activation guard corrective — ready

Class: invariant/corrective closure.

Plan: `plans/implementation/i2pcontrol-proposal-170/063-m062-closure-and-feature-guard-corrective.md`.

Objective:

- reconcile M062 status/head records with the actual landed commit;
- strengthen `m062_dependency_containment.rs` so forbidden root features cannot transitively reach the I2PControl-only direct `subtle` activation;
- preserve all Cargo manifests, lockfile, production source, runtime/core behavior, and Proposal 170 capability state.

Authorized implementation paths are limited to the existing M062 test and planning/closure records named by the M063 plan.

Exit conditions:

- direct and indirect forbidden activation regressions fail the guard;
- cycle/weak-feature semantics are covered without false positives;
- M061 guard still passes unchanged;
- Cargo manifests, lockfile, and production source are untouched;
- M062/M063 planning records agree on exact status/head identities;
- registry and roadmap return to closed with no ready containment successor;
- 37/1/5, M051 blocker, unsupported tunnel scope, CI/release scope, and internal-only boundary remain unchanged.

## 8. Cross-cutting requirements

### Storage and migration

No persistence schema or storage change is authorized.

### Protocol and compatibility

No wire contract change. Exact Proposal 170/base compatibility semantics remain those accepted before M063.

### Security and authorization

The reviewed constant-time password comparison primitive remains unchanged. M063 strengthens only the evidence that unrelated Cargo features cannot activate its I2PControl-only direct dependency edge.

### Concurrency, cancellation, restart, and recovery

No runtime task/lock/channel/lifecycle change is authorized. The test's feature graph traversal must be finite and cycle-safe.

### Observability and audit

M061 remains source-audit authority. M062 remains dependency-policy authority. M063 adds regression completeness and record consistency only.

### Performance and resource use

No runtime performance behavior changes. The new test helper must remain local, bounded, and deterministic.

### Documentation and operations

No deployment/release-process change. Planning records must distinguish the M062 planning head `a0d9f2d` from the actual M062 implementation/closure commit `fac2a0c`.

## 9. Verification strategy

M063 verification is local and proportional:

- focused `m062_dependency_containment` tests including direct/indirect/cycle/weak-edge fixtures;
- retained `m061_containment` source guard;
- `cargo check -p emissary-cli --no-default-features`;
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`;
- exact changed-path review from `fac2a0c`;
- exact proof that Cargo manifests, lockfile, M061 authority, and production source did not change;
- `git diff --check`.

No full workspace matrix, hosted CI, coverage, fuzz, soak, release, or platform expansion is required.

## 10. Risks and decision points

| Risk | Decision/mitigation |
|---|---|
| Guard remains literal-only and misses feature composition | compute transitive local-feature closure with cycle protection |
| Weak dependency-feature syntax is misclassified | treat `dep?/feature` as non-activating unless another reachable edge activates the dependency |
| Test becomes a generalized Cargo implementation | keep a small local helper scoped to the feature forms required by this invariant |
| Current manifests unexpectedly violate indirect activation rule | stop and separately plan a manifest corrective; M063 cannot edit manifests |
| Closure cleanup rewrites historical evidence | preserve `062-closure.md`; record correction in M063 closure |
| Work drifts into production dependency cleanup | Cargo manifests/lockfile are prohibited paths |
| M063 accidentally reopens M061 source boundary | M061 manifest/test remain unchanged and must pass |
| Work drifts into unavailable Prop 170 rows | 37/1/5 matrix is invariant |
| Upstream interaction is inferred from comparison | upstream access remains read-only only |

## 11. Completion definition

This containment roadmap returns to closed only after M063 demonstrates:

- M058–M061 remain accepted closed;
- M062 production dependency correction remains unchanged and accepted;
- direct and transitive local-feature activation of the I2PControl-only direct dependency is correctly guarded;
- M062 plan status and planning/closure head labels are factually reconciled;
- M062 historical closure record is preserved and M063 records the corrective evidence;
- no Cargo manifest, lockfile, production source, runtime/core, unsupported capability, CI/release, or upstream-interaction change occurred;
- accepted Proposal 170 behavior and 37/1/5 RouterInfo disposition remain unchanged;
- no containment successor is dependency-ready.

The completion criterion remains **minimum justified source and dependency delta with durable evidence**, not a predetermined changed-file count.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M058 | closed | `plans/implementation/i2pcontrol-proposal-170/058-non-i2pcontrol-delta-inventory.md` | `plans/closure/i2pcontrol-proposal-170/058-closure.md` | audit-only; 47 paths classified; no production changes |
| M059 | closed | `plans/implementation/i2pcontrol-proposal-170/059-cli-runtime-containment.md` | `plans/closure/i2pcontrol-proposal-170/059-closure.md` | exact original-CLI budget implemented; no core changes |
| M060 | closed | `plans/implementation/i2pcontrol-proposal-170/060-core-observation-containment.md` | `plans/closure/i2pcontrol-proposal-170/060-closure.md` | 23 retained core paths; 9 restored; no new core path |
| M061 | closed | `plans/implementation/i2pcontrol-proposal-170/061-containment-reclosure.md` | `plans/closure/i2pcontrol-proposal-170/061-closure.md` | exact source boundary accepted and enforced |
| M062 | corrective pass required for closure/evidence only; production fix accepted | `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md` | `plans/closure/i2pcontrol-proposal-170/062-closure.md` | M063 corrects stale records and incomplete indirect feature-activation guard |
| M063 | ready | `plans/implementation/i2pcontrol-proposal-170/063-m062-closure-and-feature-guard-corrective.md` | pending | M062 production implementation landed at `fac2a0c`; no other hard dependency |

M051 from the source-completion roadmap remains independently blocked by absent substantive news/ban owners and is not a dependency of M063.
