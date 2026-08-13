# I2PControl Proposal 170 Containment Corrective Roadmap

Status: closed; source containment closed by M061; dependency-surface containment closed by M062

Original planning baseline: `adb2f52543764b267b2bcb282d093111001ae4b2` — merged M057 closure head

M062 planning baseline: `a70dd3ac82f12fbea1f8fba51e30a9e2e516650a` — merged M061 containment reclosure head

M062 closure commit: `a0d9f2dcc15fdeb5fcbe6658c0399ff9c8c9575b` — accepted dependency-surface closure head

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
- `plans/closure/i2pcontrol-proposal-170/062-closure.md`.

Pinned external contract:

- I2P Proposal 170 `I2PControl Expansion`, revision `2026-05-20` as already pinned by the accepted workstream;
- external sources and upstream repository remain read-only and do not authorize upstream interaction.

## 1. Purpose and ownership boundary

The Proposal 170 implementation is operational for its supported surface and keeps control-plane policy under `emissary-cli/src/i2pcontrol/**`. M058–M061 closed the main physical source-containment problem by reducing and then statically enforcing the minimum justified non-`i2pcontrol` source delta.

A post-M061 review found one residual containment class that M061 intentionally did not govern: direct Cargo dependency ownership. `subtle`, used by I2PControl authentication for constant-time comparison, was declared at workspace scope and consumed by `emissary-cli` unconditionally even when the `i2pcontrol` feature was disabled. M062 closed that dependency-surface gap.

M062 did **not** reopen the accepted source boundary, RouterInfo source completeness, unsupported tunnel types, authentication behavior, or runtime/core architecture.

The governing ownership model is now two-layered:

- **source containment:** M061 is authoritative for changed source paths outside `emissary-cli/src/i2pcontrol/**`;
- **dependency containment:** M062 ensures direct dependencies used solely by I2PControl are optional and feature-owned, and installs a complementary manifest/lockfile guard.

The target is **minimum justified delta**, now including both source and direct dependency surfaces.

## 2. Work classification

### Invariants

- Supported Proposal 170 behavior, exact wire spelling/types, authentication/TLS behavior, persistence semantics, AddressBook owner coherence, ClientServicesInfo truthfulness, and supported client/server tunnel lifecycle remain unchanged.
- The accepted RouterInfo matrix remains 37 available / 1 neutral / 5 unavailable.
- Unsupported tunnel data planes remain unsupported and resource-free.
- M061 source-boundary files and accepted source-path disposition remain unchanged by M062 implementation.
- Default/no-I2PControl execution must not carry an unconditional direct dependency solely for I2PControl code.
- No router, peer-selection, NetDB, tunnel construction/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP algorithm change is authorized.
- No upstream write/review/submission activity is authorized.

### Infrastructure

Completed through M061:

- complete non-`i2pcontrol` fork-delta ledger;
- narrowed original CLI adapters;
- consolidated neutral core observation boundary;
- exact current source manifest and static guard.

M062 adds:

- machine-readable Cargo dependency ownership authority;
- focused semantic manifest guard for I2PControl-only direct dependencies;
- lockfile/no-resolution-churn evidence.

### Polish

M062 may restore root workspace dependency scope and package feature ownership only. It is not a general dependency-slimming pass.

### Capabilities

No new external capability is created. Authentication continues to use the reviewed `subtle` constant-time primitive when I2PControl is enabled.

## 3. Explicit non-goals

This work must not:

- implement news, banned-peer ownership, transit 15-second sampling, v4/v6 network-error ownership, or any other currently unavailable RouterInfo source;
- implement HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, or any other missing tunnel data plane;
- add new I2PControl methods, aliases, selectors, statuses, fields, or compatibility extensions;
- change `emissary-cli/src/**`, `emissary-core/**`, `emissary-util/**`, examples, runtime configuration, or persistence code;
- replace the `subtle` primitive with hand-written constant-time logic;
- perform a repository-wide dependency cleanup or version refresh;
- extract the implementation into a new Cargo crate merely for aesthetic isolation;
- add CI, release automation, coverage, fuzz, soak, platform matrices, or generated evidence bundles;
- prepare or seek an upstream merge, review, issue, pull request, proposal update, or maintainer contact.

## 4. Current state

### 4.1 Accepted source containment

M058 inventoried 47 non-I2PControl production/example paths. M059 moved AddressBook administrative policy into `emissary-cli/src/i2pcontrol/**` and restored unnecessary original-CLI differences without touching core. M060 reduced the accepted 32-path core budget to 23 retained neutral owner/inspection paths and restored nine paths to upstream. M061 independently accepted nine original CLI/runtime source paths plus those 23 core paths and installed an exact-path static guard.

That source disposition remains accepted and is not reopened by M062.

### 4.2 Residual dependency gap (closed)

At M062 baseline `a70dd3ac`:

- root `Cargo.toml` had an added workspace dependency `subtle = { version = "2.6.1", default-features = false }` relative to the pinned upstream manifest;
- `emissary-cli/Cargo.toml` declared `subtle = { workspace = true }` without `optional = true`;
- `emissary-cli/src/i2pcontrol/auth.rs` was the identified direct consumer and uses `subtle::ConstantTimeEq` for password comparison;
- `i2pcontrol` activated the other optional service dependencies but did not explicitly activate `subtle` because the direct dependency was unconditional;
- M061's path guard intentionally scoped its git-diff check to `emissary-cli/src` and `emissary-core/src`, so Cargo manifests were not part of the exact source path authority.

This was a dependency ownership defect, not an authentication or runtime defect. M062 corrected it without altering any source file. The current state at `a0d9f2d`:

- root `Cargo.toml` no longer declares `subtle`;
- `emissary-cli/Cargo.toml` declares `subtle = { version = "2.6.1", default-features = false, optional = true }`;
- `i2pcontrol = [..., "dep:subtle"]` explicitly activates the optional dependency;
- `emissary-core` continues to declare `subtle` with a literal version (independent non-I2PControl direct consumer in `emissary-core/src/crypto/dsa.rs`; not a workspace reference);
- `Cargo.lock` is byte-identical to the M062 planning baseline.

## 5. Target architecture

### 5.1 I2PControl source owner

All Proposal 170 and I2PControl administrative/wire/security policy remains under `emissary-cli/src/i2pcontrol/**`, as accepted by M061.

### 5.2 Original CLI/core source boundary

The M061 source boundary remains unchanged. M062 is prohibited from changing production Rust source.

### 5.3 Dependency boundary

A direct dependency whose only direct consumer is I2PControl feature-gated code must:

1. be declared optional at the package that owns the feature;
2. be activated explicitly by the `i2pcontrol` feature;
3. not be activated by `default`, `ui`, `metrics`, or another unrelated feature;
4. not be promoted to workspace scope solely for I2PControl convenience unless another independently justified direct workspace consumer exists.

For `subtle`, the expected target is local optional ownership in `emissary-cli/Cargo.toml`, with `dep:subtle` in `i2pcontrol`, and no direct root workspace declaration.

This rule concerns **direct dependency ownership**, not transitive crate-name presence. `subtle` may legitimately appear transitively through unrelated cryptographic dependencies; that does not violate M062.

### 5.4 Combined containment authority

After M062 closure:

- source-path authority remains `061-containment-boundary.toml` plus `m061_containment.rs`;
- dependency authority is `062-dependency-containment.toml` plus `m062_dependency_containment.rs`.

Do not rewrite M061 historical accepted evidence merely to combine these into one file.

## 6. Dependency graph

```text
M057 closed source/truthfulness planning state
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
M062 — dependency-surface containment corrective — CLOSED
```

M062 is closed. The containment roadmap returns to closed with M061 governing
source paths and M062 governing direct dependency ownership. M051 remains
independently blocked in the source-completion roadmap. No future
implementation plan becomes ready as a result of M062.

## 7. Milestones

### M058 — Non-I2PControl fork-delta inventory and containment ledger — closed

Class: infrastructure/corrective audit.

Result: complete 47-path inventory, exact M059/M060 budgets, zero production behavior changes.

### M059 — Original CLI/runtime adapter containment — closed

Class: corrective implementation.

Result: AddressBook administrative policy contained under I2PControl; original CLI/runtime reduced to composition/neutral adapters; no core changes.

### M060 — Core observation seam consolidation — closed

Class: corrective implementation/security containment.

Result: core budget reduced from 32 to 23 retained paths; nine paths restored to upstream; only bounded neutral owner-local observations retained.

### M061 — Independent source containment reclosure — closed

Class: invariant/closure.

Result: exact current source boundary accepted and enforced with no production changes.

### M062 — I2PControl dependency-surface containment corrective — closed

Class: corrective manifest/security containment.

Plan: `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md`.

Closure: `plans/closure/i2pcontrol-proposal-170/062-closure.md`.

Authority: `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
plus `emissary-cli/tests/m062_dependency_containment.rs`.

Objective: remove the unconditional I2PControl-only direct dependency edge from feature-disabled `emissary-cli` and extend containment governance to Cargo dependency ownership.

Authorized production paths:

- `Cargo.toml`;
- `emissary-cli/Cargo.toml`.

Closed changes:

- removed root workspace `subtle` declaration after confirming no independent non-I2PControl workspace consumer exists (`emissary-core` declares a literal version, not a workspace reference);
- declared `subtle` locally in `emissary-cli` as optional with `default-features = false`;
- added explicit `dep:subtle` activation to the `i2pcontrol` feature;
- `Cargo.lock` is byte-identical to the M062 planning baseline `a70dd3ac`;
- added the M062 dependency manifest, focused semantic test, and supporting planning artifacts.

Exit conditions (all met):

- direct dependency ownership is feature-correct;
- no source/runtime/core production file changed;
- M061 source guard still passes unchanged;
- M062 semantic dependency guard passes (8 tests);
- no unrelated dependency/version/lockfile churn;
- feature-off and feature-on checks pass;
- focused authentication regression passes (20 tests);
- accepted Proposal 170 state remains unchanged.

## 8. Cross-cutting requirements

### Storage and migration

No persistence schema or storage change is authorized.

### Protocol and compatibility

No wire contract change. Exact Proposal 170/base compatibility semantics remain those accepted before M062.

### Security and authorization

The reviewed constant-time password comparison primitive remains in place for enabled I2PControl. M062 reduces direct dependency exposure; it must not weaken authentication or move security policy outside I2PControl.

### Concurrency, cancellation, restart, and recovery

No runtime task/lock/channel/lifecycle change is authorized. Any such change means M062 exceeded scope.

### Observability and audit

M061 remains source-audit authority. M062 must provide semantic manifest evidence for direct dependency ownership and exact allowed production paths.

### Performance and resource use

No runtime performance behavior is changed. The intended improvement is feature-disabled dependency-surface containment, not a runtime optimization claim.

### Documentation and operations

No deployment/release-process change. Planning records should make clear that source containment was closed by M061 and dependency containment by M062.

## 9. Verification strategy

M062 verification is local and proportional:

- inspect workspace/package manifest ownership and direct source consumers;
- `cargo metadata --format-version 1 --no-deps` to inspect optional/direct feature metadata;
- `cargo check -p emissary-cli --no-default-features`;
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`;
- focused `m062_dependency_containment` test;
- focused I2PControl auth tests;
- retained `m061_containment` source guard;
- exact baseline changed-path and lockfile diff review;
- `git diff --check`.

`cargo tree` may be used diagnostically to inspect feature edges, but crate-name absence is not an acceptance gate because `subtle` may exist transitively.

No hosted CI, coverage, fuzz, soak, release, or platform expansion is required.

## 10. Risks and decision points

| Risk | Decision/mitigation |
|---|---|
| A non-I2PControl direct consumer actually needs workspace `subtle` | stop M062 and replan; do not fabricate dependency isolation |
| Making dependency optional unexpectedly requires source changes | stop; production Rust is outside M062 authority |
| Cargo rewrites unrelated lock entries | reject churn; no dependency update campaign |
| Removing workspace declaration encourages hand-written crypto | explicitly retain reviewed `subtle`; only ownership changes |
| Guard incorrectly treats transitive `subtle` as failure | guard direct manifest/feature ownership semantically, not crate-name absence |
| M062 accidentally reopens M061 source boundary | M061 manifest/test must remain unchanged and continue to pass |
| Work drifts into other dependencies | only same-class trivial manifest finding may be recorded; broader cleanup requires separate authorization |
| Work drifts into unavailable Prop 170 rows | 37/1/5 matrix is invariant |
| Upstream interaction is inferred from comparison | upstream access remains read-only only |

## 11. Completion definition

This containment roadmap returned to closed after M062 when all of the following held at the closure commit:

- M058–M061 remain accepted closed;
- root workspace no longer owns an I2PControl-only direct `subtle` dependency absent an independent direct consumer;
- `emissary-cli` owns `subtle` as an optional direct dependency;
- `i2pcontrol` explicitly activates it and unrelated features do not;
- no source/runtime/core production file changes;
- no unrelated dependency/version/lockfile churn;
- M061 source containment remains valid;
- M062 dependency containment is machine-readable and statically guarded;
- accepted Proposal 170 behavior and 37/1/5 RouterInfo disposition remain unchanged;
- no new unsupported capability, CI/release machinery, or upstream interaction is introduced.

The completion criterion is **minimum justified source and dependency delta**, not a predetermined changed-file count. All items above were independently verified before M062 closure was accepted.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M058 | closed | `plans/implementation/i2pcontrol-proposal-170/058-non-i2pcontrol-delta-inventory.md` | `plans/closure/i2pcontrol-proposal-170/058-closure.md` | audit-only; 47 paths classified; no production changes |
| M059 | closed | `plans/implementation/i2pcontrol-proposal-170/059-cli-runtime-containment.md` | `plans/closure/i2pcontrol-proposal-170/059-closure.md` | exact original-CLI budget implemented; no core changes |
| M060 | closed | `plans/implementation/i2pcontrol-proposal-170/060-core-observation-containment.md` | `plans/closure/i2pcontrol-proposal-170/060-closure.md` | 23 retained core paths; 9 restored; no new core path |
| M061 | closed | `plans/implementation/i2pcontrol-proposal-170/061-containment-reclosure.md` | `plans/closure/i2pcontrol-proposal-170/061-closure.md` | exact source boundary accepted and enforced |
| M062 | closed | `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md` | `plans/closure/i2pcontrol-proposal-170/062-closure.md` | dependency-surface containment corrective complete; manifest-only production budget executed; no source/runtime/core change |

M051 from the source-completion roadmap remains independently blocked by absent substantive news/ban owners and is not a dependency of M062.
