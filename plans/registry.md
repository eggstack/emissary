# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies and interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency or evidence requirement prevents progress.
- **closing** — implementation landed and independent closure evidence is being gathered.
- **closed** — closure record accepted.
- **closed internally against pinned revision** — internal closure accepted against an explicitly named revision of an open external specification; does not imply upstream review or acceptance.
- **partial Proposal 170 support** — exact supported dimensions are closed, but one or more pinned source/runtime capabilities remain truthfully unavailable.
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 source/truthfulness | partial Proposal 170 support; M057 closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | no source-completion handoff | M051 remains blocked by absent substantive news/ban owners; accepted RouterInfo matrix remains 37/1/5 |
| I2PControl Proposal 170 containment | active; M062 closed | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | no dependency-ready handoff | M061 source containment closed; M062 dependency-surface containment closed; M051 remains independently blocked |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|

No implementation plan is currently dependency-ready. M058–M062 are closed and
their accepted closure records were not rewritten. M051 remains blocked with
its accepted semantic limitation; no successor plan becomes ready.

## Recently closed milestones

| Subsystem | Handoff | Status | Implementation plan | Closure record |
|---|---|---|---|---|
| I2PControl Proposal 170 containment | M062 — I2PControl dependency-surface containment corrective | closed | `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md` | `plans/closure/i2pcontrol-proposal-170/062-closure.md` |
| I2PControl Proposal 170 containment | M061 — independent containment reclosure | closed | `plans/implementation/i2pcontrol-proposal-170/061-containment-reclosure.md` | `plans/closure/i2pcontrol-proposal-170/061-closure.md` |

## Blocked roadmap successors

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M051 — router news and banned peers | blocked with accepted semantic limitation | `plans/implementation/i2pcontrol-proposal-170/051-routerinfo-news-and-banned-peer-semantics.md` | substantive news/ban owners absent; no current owner-specific plan authorized |

M062 is not a source-completion successor and does not alter the M051 blocker.

## Current containment corrective scope

Original containment planning baseline: `adb2f52543764b267b2bcb282d093111001ae4b2` — merged M057 closure head.

M062 planning baseline: `a70dd3ac82f12fbea1f8fba51e30a9e2e516650a` — merged M061 containment reclosure head.

Pinned upstream compare baseline/merge base: `eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f`.

M058–M061 established and enforced the minimum justified non-`i2pcontrol` **source** delta. M062 closes the remaining dependency-surface gap that the M061 source-path guard intentionally did not cover.

Closed evidence at `a0d9f2d` planning head (current `master` after M062 closure):

- root `Cargo.toml` no longer carries an I2PControl-only `subtle` workspace declaration;
- `emissary-cli/Cargo.toml` declares `subtle = { version = "2.6.1", default-features = false, optional = true }` locally;
- `i2pcontrol = [..., "dep:subtle"]` explicitly activates the optional dependency;
- `emissary-core` continues to declare `subtle` with a literal version (independent non-I2PControl direct consumer in `emissary-core/src/crypto/dsa.rs`; not a workspace reference);
- `Cargo.lock` is byte-identical to the M062 planning baseline.

### M062 production authority

Authorized production files (closed):

- `Cargo.toml`;
- `emissary-cli/Cargo.toml`.

`Cargo.lock` is unchanged relative to the planning baseline. No `emissary-cli/src/**`, `emissary-core/**`, `emissary-util/**`, example, workflow, runtime configuration, or release file change was authorized or made.

### Durable dependency rule

A direct dependency whose only direct consumer is `feature = "i2pcontrol"` code must be optional and feature-owned by `i2pcontrol`. It must not be an unconditional default-CLI dependency or a workspace-level dependency absent an independently justified non-I2PControl direct consumer.

This rule concerns direct dependency edges. A crate may still legitimately appear transitively in a feature-disabled graph; crate-name absence from `cargo tree` is explicitly not an acceptance criterion.

The dependency authority is `062-dependency-containment.toml`, enforced by `m062_dependency_containment.rs`. Future changes to I2PControl direct dependencies must satisfy this guard.

## Retained and corrective milestone disposition

| Handoff | Current status | Evidence / correction |
|---|---|---|
| M037 — earlier containment boundary reduction | closed historical containment evidence | `037-closure.md`; later RouterInfo source work expanded observation paths, so its manifest is not the final current authority |
| M045 / M053 — known-peer directory corrective | closed | live `ProfileStorage` source accepted by `053-closure.md` |
| M046 — active-peer inventory and limits | closed | `046-closure.md` |
| M047 — active-peer statistics | closed | `047-closure.md` |
| M048 — tunnel-pool counts/details | closed | `048-closure.md` |
| M049 — rolling metrics/queues | corrected/closed through M054 and M056 | recent success + queue/TBM retained; transit 15s explicitly unavailable |
| M050 — v4/v6 network state | corrected/closed through M055 and M056 | status.v6 + testing v4/v6 retained; error rows unavailable with no canonical owner |
| M051 — news/banned peers | blocked with accepted limitation | `051-closure.md`; both rows remain unavailable |
| M052 — integration reclosure | corrected/closed through M056 | historical `40/1/2` matrix superseded by accepted `37/1/5` audit |
| M054–M056 | closed | transit/error truthfulness and integrated source reclosure accepted |
| M057 | closed | planning-record consistency accepted; no production changes |
| M058 | closed | `058-closure.md`; 47-path ledger and M059/M060 budgets frozen |
| M059 | closed | `059-closure.md`; exact original-CLI budget implemented with no core changes |
| M060 | closed | `060-closure.md`; 23 retained core paths, 9 formatting-only paths reverted, no new core path |
| M061 | closed | `061-closure.md`; exact current source boundary accepted and enforced; no production changes |
| M062 | closed | `062-closure.md`; dependency-surface containment corrective complete; source/runtime/core behavior frozen |

## Accepted Proposal 170 support state

The accepted RouterInfo source matrix remains exactly:

- 43 canonical Proposal 170 RouterInfo additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable: transit 15s, news, banned peers, and both v4/v6 network-error rows.

The historical `970252c` M052-era 40/1/2 claim remains historical only. M054/M055 corrected three overclaims and M056 accepted 37/1/5. M062 is not authorized to change this matrix.

Unsupported Emissary tunnel data planes remain out of scope. Do not implement them to improve Proposal 170 completeness.

## Prohibited scope throughout M062

- any production Rust source change;
- any router/core/runtime behavior change;
- any authentication algorithm change or replacement of the reviewed `subtle` primitive;
- any dependency upgrade/refresh campaign or unrelated lockfile churn;
- any new Proposal 170 source, selector, method, alias, or compatibility behavior;
- any unsupported tunnel data plane;
- `.github/workflows/**`, remote CI expansion, release/publishing, coverage, fuzz, soak, or platform matrices;
- upstream issues, pull requests, reviews, submissions, adoption, merge, maintainer contact, contribution preparation, branches/tags/releases, or connector writes against upstream/third-party repositories.

M062 closed without performing any of the prohibited actions.

## Pinned authority

Proposal 170 remains pinned internally to revision `2026-05-20` for this workstream. A materially changed external revision blocks affected implementation/closure and requires an explicit contract-rebase plan.

Upstream `eepnet/emissary` is accessed read-only for comparison. The containment compare remains pinned to `9b43484a`; upstream advancement does not silently change the audit baseline.

## Registry maintenance rules

1. M058–M062 remain closed and their accepted closure records are not rewritten.
2. `061-containment-boundary.toml` plus `m061_containment.rs` remain the accepted source-boundary authority.
3. `062-dependency-containment.toml` plus `m062_dependency_containment.rs` are the accepted dependency-boundary authority. Future changes to I2PControl direct dependencies must satisfy this guard.
4. Preserve the accepted 37/1/5 RouterInfo disposition and M051 blocker.
5. Keep Proposal 170 policy under I2PControl; outside source changes remain neutral owner/composition seams only.
6. I2PControl-only direct dependencies must be optional and owned by the `i2pcontrol` feature.
7. Keep verification local/package-scoped and proportional; do not expand CI/release apparatus.
8. Overall Proposal 170 remains partial; dependency containment completion does not imply source completeness.
9. No upstream interaction is authorized.
