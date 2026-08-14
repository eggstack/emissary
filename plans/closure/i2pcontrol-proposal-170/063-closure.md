# M063 Closure Record — M062 Closure Consistency and Indirect Feature-Activation Guard Corrective

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/063-m062-closure-and-feature-guard-corrective.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`

Planning baseline: `fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c` — current `master`, containing the M062 implementation and closure record.

M062 planning head: `a0d9f2dcc15fdeb5fcbe6658c0399ff9c8c9575b`.

M062 implementation/closure commit: `fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c`.

Pinned upstream comparison baseline: `eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f`, accessed read-only.

Accepted predecessor:

- M062 closure at `fac2a0c` (production dependency correction accepted; closure/evidence defects identified afterwards).
- M061 closure at `77a2555` (`a0d9f2d` planning close head).

Implementation commit:

- M063 modifies only the M062 dependency-containment test plus the planning/closure records explicitly authorized by the M063 plan. The M063 implementation commit is the unique commit that lands the test strengthening and the planning-record reconciliation; its hash is recorded in the repository commit log alongside this closure record.

## 1. Executive disposition

M063 is closed. This pass was a closure-corrective and invariant-strengthening handoff: it neither reopened nor modified the accepted M062 production dependency correction, the M061 source-boundary authority, the dependency policy, the authentication implementation, the runtime/core behavior, or the Proposal 170 capability scope.

The three bounded closure defects identified after M062 are now closed:

1. **D1 — stale `Status: ready` in the M062 implementation plan.** The M062 top-level plan status is now `closed`, with a concise pointer that M063 corrected the closure-record consistency and the indirect feature-activation guard. The M062 plan's original requirements and acceptance criteria are preserved.
2. **D2 — stale M062 closure-head identity.** `a0d9f2d` is now identified consistently in the registry and roadmap as the M062 planning head, not the closure commit. The actual M062 implementation/closure commit `fac2a0c` is named as the authoritative repository state.
3. **D3 — registry/roadmap lifecycle mismatch.** The registry and containment roadmap now agree on final status: both records read M062 and M063 as closed, with no dependency-ready containment successor.
4. **D4 — M062 guard transitively activates `subtle`.** The M062 test's direct-activation check is replaced by a semantic `LocalFeatureGraph` reachability helper that walks local feature edges with a visited set, classifies `dep:NAME`, strong `NAME/feature`, weak `NAME?/feature`, weak `?/NAME`, and bare `NAME` correctly, and fails closed for any forbidden root feature (`default`, `ui`, `metrics`) that can reach an activation of the direct `subtle` dependency through any chain of local feature references including the canonical `ui -> i2pcontrol -> dep:subtle` regression.

The accepted M062 production disposition is unchanged at `fac2a0c`:

- root `Cargo.toml` does not declare `subtle` in `[workspace.dependencies]`;
- `emissary-cli/Cargo.toml` keeps `subtle = { version = "2.6.1", default-features = false, optional = true }`;
- `i2pcontrol` explicitly activates `dep:subtle`;
- `Cargo.lock` is byte-identical to `fac2a0c`;
- `emissary-cli/src/i2pcontrol/auth.rs` and the reviewed `subtle::ConstantTimeEq` / `subtle::Choice` authentication implementation remain unchanged;
- M061 source-boundary manifest and guard remain unchanged and continue to pass.

M063 is planning/test-only. No CI/release/fuzz/coverage/platform expansion was performed. No upstream/third-party write, issue, pull request, review, merge, maintainer contact, contribution preparation, branch, tag, release, or connector write occurred.

## 2. Exact changed-path list relative to `fac2a0c`

The M063 implementation commit changes the following paths relative to the M062 planning baseline `fac2a0c`:

- `emissary-cli/tests/m062_dependency_containment.rs` — strengthened dependency-activation guard with a `LocalFeatureGraph` reachability helper; added fixture-based regression tests for direct, indirect, cycle, weak dependency-feature, and bare-activation cases; updated the authorized planning-path list to include M063 artifacts.
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md` — corrected top-level status from `ready` to `closed`; added a pointer to M063 as the closure-corrective handoff and to `fac2a0c` as the M062 implementation/closure commit; original requirements and acceptance criteria preserved.
- `plans/implementation/i2pcontrol-proposal-170/063-m062-closure-and-feature-guard-corrective.md` — corrected top-level status from `ready` to `closed`; added a closure-record pointer.
- `plans/implementation/i2pcontrol-proposal-170/README.md` — corrected current-handoff text to reflect M063 closed; preserved containment authority, durable dependency rule, and accepted Proposal 170 state.
- `plans/registry.md` — corrected containment-roadmap status to `closed`; removed M063 from the dependency-ready section; added M063 to the recently closed table; reconciled M062 entry to `closed (closure/evidence corrected by M063)`; preserved the durable dependency rule and 37/1/5 RouterInfo matrix.
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` — corrected status to `closed`; M062 entry to `closed (closure/evidence corrected by M063)`; M063 entry to `closed`; added M063 closure-record reference; updated text to reflect completed corrective handoff.
- `plans/closure/i2pcontrol-proposal-170/063-closure.md` — this new closure record.

The M063 implementation commit does **not** change `Cargo.toml`, `emissary-cli/Cargo.toml`, `Cargo.lock`, `emissary-cli/src/**`, `emissary-core/**`, `emissary-util/**`, `examples/**`, `.github/**`, `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`, or `emissary-cli/tests/m061_containment.rs`. The exact-path proof is the `git diff --name-only fac2a0c..HEAD` command recorded in §6.

## 3. Requirement-to-evidence matrix

| # | Acceptance criterion | Evidence | Result |
|---|---|---|---|
| 1 | M062 production manifest state is byte/semantically unchanged from `fac2a0c` | `git diff fac2a0c..HEAD -- Cargo.toml emissary-cli/Cargo.toml` is empty; `cargo metadata` confirms only `subtle` packaging diff against upstream `9b43484a` | pass |
| 2 | `Cargo.lock` is unchanged | `git diff fac2a0c..HEAD -- Cargo.lock` is empty; `m062_dependency_containment::lockfile_is_byte_identical_to_fork_baseline` passes | pass |
| 3 | No production Rust source changed | `git diff fac2a0c..HEAD -- emissary-cli/src emissary-core emissary-util` is empty; `m062_dependency_containment::allowed_production_paths_match_the_m062_budget` and `m061_containment` source guards pass | pass |
| 4 | M061 boundary files remain unchanged and `m061_containment` passes | `git diff fac2a0c..HEAD -- plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml emissary-cli/tests/m061_containment.rs` is empty; `m062_dependency_containment::m061_source_boundary_files_remain_unchanged` passes; `cargo nextest run -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment` (7 tests) passes | pass |
| 5 | M062 dependency guard still rejects direct forbidden activation | `m062_dependency_containment::direct_forbidden_feature_activation_is_rejected` (fixture `default = ["dep:subtle"]`) and `direct_bare_dependency_activation_is_detected` (fixture `default = ["subtle"]`) pass; updated `only_i2pcontrol_activates_subtle` test still rejects direct `dep:subtle` / `subtle` entries in `default`, `ui`, `metrics` | pass |
| 6 | Guard rejects indirect `ui -> i2pcontrol -> dep:subtle` activation | `m062_dependency_containment::indirect_forbidden_feature_chain_is_rejected` (fixture `ui -> i2pcontrol -> dep:subtle`) and `indirect_forbidden_feature_chain_via_strong_dep_feature_is_rejected` (fixture `metrics -> ui -> i2pcontrol -> subtle/ct`) both detect the activation; `current_manifest_forbidden_features_cannot_reach_subtle` confirms the current `default`, `ui`, `metrics` features cannot reach `subtle` through any local feature edge | pass |
| 7 | Guard computes transitive local-feature reachability with cycle protection | `LocalFeatureGraph::visit` recurses through local feature references with a `BTreeSet<String>` visited set; `feature_cycle_terminates_and_still_detects_activation` (fixture `ui -> alpha -> beta -> {alpha, i2pcontrol}` and `i2pcontrol -> dep:subtle`) terminates and detects the activation from `ui`, `alpha`, and `beta` | pass |
| 8 | Weak dependency-feature syntax does not produce a false positive when it does not independently activate `subtle` | `weak_dependency_feature_alone_is_not_independent_activation` (fixture `default = ["subtle?/ct"]`, `ui = ["?/i2pcontrol"]`) confirms `default` and `ui` do not reach `subtle`; `weak_dependency_feature_alongside_strong_still_activates` (fixture `default = ["subtle?/ct", "dep:subtle"]`) confirms a strong sibling edge still activates | pass |
| 9 | Current `default`, `ui`, `metrics` feature roots cannot transitively activate the direct `subtle` dependency | `current_manifest_forbidden_features_cannot_reach_subtle` and `forbidden_activations_manifest_is_self_consistent_with_graph` both pass against the actual `emissary-cli/Cargo.toml` | pass |
| 10 | Feature-off and feature-on `emissary-cli` checks pass | `cargo check -p emissary-cli --no-default-features` and `cargo check -p emissary-cli --no-default-features --features i2pcontrol` both succeed | pass |
| 11 | M062 implementation plan status is no longer stale `ready` | Plan top-level now reads `Status: closed`; closure record pointer to M063 is present | pass |
| 12 | Roadmap identifies `a0d9f2d` as planning head and `fac2a0c` as the actual M062 implementation/closure commit | `plans/registry.md` and containment roadmap both declare `a0d9f2d` as `M062 planning head` and `fac2a0c` as the M062 implementation/closure commit; no document now places `a0d9f2d` post-M062 | pass |
| 13 | Registry no longer describes `a0d9f2d` as current post-M062 master | `plans/registry.md` containment-roadmap entry reads `closed`; no current-master label references `a0d9f2d` | pass |
| 14 | Registry, roadmap, implementation index, M062 plan status, and M063 closure record agree on final containment lifecycle state | All five records read M062 as `closed` (corrective-pass corrected by M063) and M063 as `closed`; no dependency-ready containment successor is registered | pass |
| 15 | M058–M061 accepted closure evidence is not rewritten | `git diff fac2a0c..HEAD` against `plans/closure/i2pcontrol-proposal-170/058-closure.md`, `059-closure.md`, `060-closure.md`, `061-closure.md` is empty; `plans/registry.md` retained-and-corrective table preserves their status | pass |
| 16 | M062 closure record remains historical and is not rewritten; M063 closure records the correction | `git diff fac2a0c..HEAD -- plans/closure/i2pcontrol-proposal-170/062-closure.md` is empty; the M063 closure record (this document) records the corrective disposition and explicitly preserves the M062 historical evidence | pass |
| 17 | RouterInfo remains 37/1/5 and M051 remains blocked | `plans/registry.md` Accepted Proposal 170 support state and Blocked roadmap successors sections are unchanged in disposition; M063 did not authorize any change to the matrix or the M051 blocker | pass |
| 18 | Unsupported tunnel data planes remain unchanged | `m061_containment::unsupported_tunnel_backends_remain_resource_free` passes; M063 did not implement any new tunnel data plane | pass |
| 19 | No dependency/version/lockfile cleanup or runtime/core change occurs | `git diff fac2a0c..HEAD -- Cargo.toml Cargo.lock emissary-cli/Cargo.toml emissary-cli/src emissary-core emissary-util` is empty | pass |
| 20 | No CI/release/fuzz/coverage/platform machinery is added or expanded | `git diff fac2a0c..HEAD -- .github` is empty; no new matrix introduced | pass |
| 21 | No upstream/third-party write, issue, pull request, review, merge, maintainer contact, contribution preparation, branch/tag/release, or submission occurs | Attestation in §8 | pass |
| 22 | After M063 closure, no containment successor is registered unless a new concrete defect is independently identified | `plans/registry.md` Dependency-ready section is empty; containment roadmap completion definition is met; no successor is registered | pass |

## 4. Test counts and outcomes

The M063 implementation commit ships the following test evidence:

```text
cargo nextest run -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
  -- 19 tests passed, 0 skipped
  -- including 11 new tests added by M063:
       current_manifest_forbidden_features_cannot_reach_subtle
       direct_bare_dependency_activation_is_detected
       direct_forbidden_feature_activation_is_rejected
       feature_cycle_terminates_and_still_detects_activation
       forbidden_activations_manifest_is_self_consistent_with_graph
       indirect_forbidden_feature_chain_is_rejected
       indirect_forbidden_feature_chain_via_strong_dep_feature_is_rejected
       local_feature_with_no_entry_terminates_safely
       unrelated_local_feature_chain_that_does_not_reach_subtle_passes
       weak_dependency_feature_alongside_strong_still_activates
       weak_dependency_feature_alone_is_not_independent_activation
  -- plus 8 retained M062 tests after lifecycle replacement:
       allowed_production_paths_match_the_m062_budget
       dependency_evidence_describes_subtle_ownership
       emissary_cli_owns_subtle_locally_as_optional_with_no_default_features
       lockfile_is_byte_identical_to_fork_baseline
       m061_source_boundary_files_remain_unchanged
       manifest_is_well_formed_and_self_consistent
       only_i2pcontrol_activates_subtle
       root_workspace_does_not_declare_subtle

cargo nextest run -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
  -- 7 tests passed, 0 skipped
```

Regression injection evidence for the indirect activation case is provided by the `from_toml_str` fixture tests, which would have caught the original M062-closure defect:

- `indirect_forbidden_feature_chain_is_rejected`: with `ui = ["i2pcontrol"]` and `i2pcontrol = ["dep:subtle"]`, the helper reports `graph.transitively_activates("ui", "subtle") == true`. The pre-M063 direct-string check would have missed this regression.
- `indirect_forbidden_feature_chain_via_strong_dep_feature_is_rejected`: with `metrics = ["ui"]`, `ui = ["i2pcontrol"]`, `i2pcontrol = ["subtle/ct"]`, the helper reports `graph.transitively_activates("metrics", "subtle") == true`. The pre-M063 check would have missed this regression.

## 5. Record-consistency review

The following stale text was corrected:

- `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md` top-level `Status: ready` → `Status: closed`; added M063 pointer and `fac2a0c` closure-commit identity.
- `plans/registry.md` containment roadmap entry `active; M063 ready` → `closed`; dependency-ready handoff section removed; M062 entry `corrective pass required` → `closed (closure/evidence corrected by M063)`; M063 entry `ready` → `closed`. Closure-record reference for M063 added under "Recently closed / corrective milestones".
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` top-level status `active; M063 ready; …` → `closed; M058–M063 closed`; M062 entry `production accepted; closure/evidence corrective required` → `closed (closure/evidence corrected by M063)`; M063 entry `ready` → `closed`; closure-record reference added; dependency-graph `M063 — closure consistency + indirect feature-activation guard — READY` → `CLOSED`.
- `plans/implementation/i2pcontrol-proposal-170/README.md` line `Status: partial Proposal 170 support; M063 corrective ready` → `Status: partial Proposal 170 support; M058–M063 closed`; `Current handoff` block now reflects M063 closed; table updates to M062 and M063 rows to closed.
- `plans/implementation/i2pcontrol-proposal-170/063-m062-closure-and-feature-guard-corrective.md` top-level `Status: ready` → `Status: closed`; closure-record pointer added.

The historical M062 closure record (`plans/closure/i2pcontrol-proposal-170/062-closure.md`) is preserved verbatim and is not rewritten. It is treated as the historical evidence that motivated M063, and the M063 closure record is the public location of the corrective disposition.

## 6. Verification commands executed

```text
cargo nextest run -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo nextest run -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
git diff --check
git diff --name-only fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c..HEAD
git diff fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c..HEAD -- Cargo.toml emissary-cli/Cargo.toml Cargo.lock
git diff fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c..HEAD -- emissary-cli/src/** emissary-core/** emissary-util/** examples/** .github/**
git diff fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c..HEAD -- plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml emissary-cli/tests/m061_containment.rs
```

Diagnostic evidence (not acceptance gates):

- `cargo metadata --format-version 1 --no-deps` continues to show `subtle` as an optional direct dependency of `emissary-cli`, only enabled when `i2pcontrol` is selected; `emissary-core` continues to declare `subtle` as a literal-version dependency for its independent DSA consumer.
- `cargo tree -p emissary-cli --no-default-features -e features` and `cargo tree -p emissary-cli --no-default-features --features i2pcontrol -e features` reflect the corrected ownership and the acceptance plan's explicit "transitive crate name may legitimately appear" rule.

## 7. Security and dependency review

- The `subtle` dependency version remains `2.6.1` (Cargo metadata and `m062_dependency_containment::emissary_cli_owns_subtle_locally_as_optional_with_no_default_features` agree). No version upgrade, refresh, or slice campaign was performed.
- `emissary-cli/src/i2pcontrol/auth.rs` retains the reviewed `subtle::ConstantTimeEq` / `subtle::Choice` primitive for constant-time password comparison. M063 did not introduce, replace, or extend authentication code.
- Constant-time comparison remains required whenever the `i2pcontrol` feature is enabled, consistent with the accepted authentication contract.
- The strengthened `m062_dependency_containment` test fails closed if a future I2PControl-only direct dependency is directly or transitively activated by `default`, `ui`, or `metrics`, including through chains such as `ui -> i2pcontrol -> dep:subtle` or `metrics -> ui -> i2pcontrol -> subtle/ct`. This is a strict strengthening of the durable evidence that the I2PControl-only authentication dependency is feature-owned end-to-end.
- No change to the `m061_containment` source-boundary authority, the dependency policy, the lockfile, the M062 planning baseline, or any production Rust source.

## 8. Unresolved findings

No high or medium security, compatibility, runtime, build, or dependency finding remains open after M063. The pre-existing stable/nightly rustfmt drift across the frozen repository is unchanged and out of scope for M063. The pre-existing CLI clippy warning at `emissary-cli/src/proxy/socks.rs:543` is unchanged and was a forbidden scope for M062 and remains forbidden for M063.

The M063 strengthening of the dependency guard creates a small additional invariant: the `LocalFeatureGraph` helper's transitive closure must terminate. The helper uses a `BTreeSet<String>` visited set, so traversal over a finite manifest feature map is bounded by the number of declared features. Cycle coverage is exercised by `feature_cycle_terminates_and_still_detects_activation`.

## 9. Internal-only attestation

The pinned upstream source and commit identity (`eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f`) were accessed read-only for comparison. No upstream repository, maintainer channel, or third-party connector was mutated; no upstream issue, pull request, review, merge, adoption request, submission, contribution artifact, branch, tag, release, or feedback request was created or prepared. The M063 implementation commit is internal to `eggstack/emissary` and is the only commit authored under this plan.

## 10. Disposition

M063 is closed. The M062 production dependency correction at `fac2a0c` is unchanged. The M062 closure/evidence defects and the indirect feature-activation guard weakness are corrected and durably enforced. The containment roadmap returns to closed:

- M058–M061 remain closed; their accepted closure records are preserved.
- M062 is closed; its historical closure record is preserved as the evidence that motivated M063.
- M063 is closed; this closure record is the authoritative corrective disposition.
- The M061 source-boundary authority remains `061-containment-boundary.toml` plus `m061_containment.rs`.
- The dependency-boundary authority remains `062-dependency-containment.toml` plus the strengthened `m062_dependency_containment.rs`.
- The accepted RouterInfo matrix remains 43 total / 37 available / 1 protocol-permitted neutral / 5 unavailable.
- M051 remains independently blocked by absent substantive news/banned-peer owners.
- Unsupported tunnel data planes remain out of scope.
- No CI/release/fuzz/coverage/platform expansion was performed.
- No upstream/third-party interaction was performed.
- No containment successor is dependency-ready.
