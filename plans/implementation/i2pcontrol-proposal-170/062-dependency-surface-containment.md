# M062 — I2PControl Dependency-Surface Containment Corrective

Status: ready

Planning baseline: `a70dd3ac82f12fbea1f8fba51e30a9e2e516650a` — merged M061 containment reclosure head

Pinned upstream comparison baseline: `eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f`

Predecessor:

- M061 closed: `plans/closure/i2pcontrol-proposal-170/061-closure.md`.

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`.

Current containment authorities retained as historical accepted evidence:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`;
- `emissary-cli/tests/m061_containment.rs`.

## 1. Objective

Close the remaining dependency-surface containment gap without reopening any Proposal 170 runtime, source, protocol, or core behavior.

The M061 source boundary is good but intentionally covers only changed production source paths under `emissary-cli/src` and `emissary-core/src`. The current fork still has one I2PControl-specific direct dependency, `subtle`, declared in workspace/package dependency surfaces in a way that makes it an unconditional direct dependency of `emissary-cli` even when the `i2pcontrol` feature is disabled.

`subtle` is used by `emissary-cli/src/i2pcontrol/auth.rs` for reviewed constant-time password comparison. That use is valid and must be preserved when I2PControl is enabled. The containment defect is dependency ownership, not the primitive itself.

M062 must:

1. make every dependency introduced solely for I2PControl optional at the package boundary;
2. remove the I2PControl-only `subtle` declaration from the workspace dependency surface when no non-I2PControl workspace consumer requires it;
3. make `subtle` activation explicit through the `i2pcontrol` feature;
4. add a current machine-readable dependency-surface authority and static guard covering the relevant Cargo manifests and lockfile invariants;
5. preserve M061 source containment unchanged;
6. make no runtime/core/source behavior change.

This is a small containment corrective, not a new Proposal 170 implementation phase.

## 2. Problem statement and current evidence

At planning baseline `a70dd3ac`:

- root `Cargo.toml` differs from the pinned upstream workspace manifest by the added workspace dependency:

  ```toml
  subtle = { version = "2.6.1", default-features = false }
  ```

- `emissary-cli/Cargo.toml` currently declares:

  ```toml
  subtle = { workspace = true }
  ```

  as a non-optional dependency;

- `emissary-cli/src/i2pcontrol/auth.rs` imports `subtle::ConstantTimeEq` and uses it only for I2PControl password comparison;
- the `i2pcontrol` feature currently enables the other I2PControl-specific optional dependencies but does not name `subtle` because `subtle` is unconditional;
- M061's exact changed-path guard covers `emissary-cli/src` and `emissary-core/src`, not `Cargo.toml`, `Cargo.lock`, or `emissary-cli/Cargo.toml`.

The result is a containment mismatch: feature-disabled Emissary does not need I2PControl authentication code, yet the package manifest carries its direct dependency unconditionally. The existing static containment authority would not detect a future recurrence of the same class of dependency leak.

## 3. Hard scope boundary

### 3.1 Authorized production files

M062 may modify only:

- `Cargo.toml`;
- `emissary-cli/Cargo.toml`.

`Cargo.lock` is **not** an authorized semantic-change surface. The expected dependency relocation uses the same crate/version and should leave the resolved lock graph stable. If Cargo requires a lockfile change, implementation must inspect and explain it before proceeding. A broad or unrelated lockfile rewrite is a scope failure.

### 3.2 Authorized test/planning files

M062 may add or modify only the following non-production support files as needed:

- `emissary-cli/tests/m062_dependency_containment.rs`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`;
- this plan;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- the eventual `plans/closure/i2pcontrol-proposal-170/062-closure.md`.

### 3.3 Explicitly prohibited production files

Do not modify:

- any `emissary-cli/src/**` file, including `emissary-cli/src/i2pcontrol/**`;
- any `emissary-core/**` file;
- any `emissary-util/**` file;
- examples;
- `.github/**`;
- runtime configuration formats;
- release/publishing files.

If the dependency correction appears to require source changes, stop and re-evaluate rather than expanding M062.

## 4. Required implementation

### 4.1 Restore the root workspace dependency boundary

Determine whether any current workspace package outside the I2PControl feature requires a direct workspace-level `subtle` declaration.

Use source/manifest inspection, not assumption. The expected current answer is no: the direct consumer is I2PControl authentication.

If no independent direct workspace consumer exists:

- remove the `subtle` entry from root `[workspace.dependencies]`;
- make root `Cargo.toml` byte/semantic-equivalent to the pinned upstream manifest for this dependency;
- do not alter unrelated workspace dependency versions or ordering.

If a real independent direct workspace consumer is discovered, record it as a blocker because that contradicts the planning evidence; do not silently retain the workspace entry and declare success.

### 4.2 Make the package dependency optional and locally owned

Declare `subtle` directly in `emissary-cli/Cargo.toml` as an optional dependency with the already reviewed version/features, expected shape:

```toml
subtle = { version = "2.6.1", default-features = false, optional = true }
```

The exact equivalent Cargo syntax is acceptable, but all of these invariants are mandatory:

- `optional = true`;
- no default features are introduced;
- version remains compatible with the current lock/resolution and does not widen scope;
- ownership is in `emissary-cli`, next to the feature that consumes it;
- no dependency is promoted to workspace scope solely for I2PControl convenience.

### 4.3 Activate `subtle` only from `i2pcontrol`

Add explicit feature linkage using Cargo's dependency feature syntax, preferably:

```toml
i2pcontrol = [
  ...,
  "dep:subtle",
]
```

Do not activate `subtle` from `default`, `ui`, `metrics`, or any unrelated feature.

Do not change the existing authentication implementation. Constant-time comparison remains required when `i2pcontrol` is enabled.

### 4.4 Add machine-readable dependency containment authority

Create:

`plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`

It must record at minimum:

- version;
- planning/fork baseline `a70dd3ac82f12fbea1f8fba51e30a9e2e516650a`;
- pinned upstream baseline `9b43484a21d5a1291c4881cdae62a36c527f8c0f`;
- retained M061 source-boundary authority path;
- root workspace manifest expectation for `subtle` (`absent` unless a separately authorized non-I2PControl owner exists);
- `emissary-cli` direct dependency expectation (`subtle`, optional, default-features false);
- owning feature (`i2pcontrol`);
- forbidden activating features (`default`, `ui`, `metrics` and any other unrelated current feature);
- expected lockfile disposition;
- exact allowed M062 production path set.

Do not rewrite `061-containment-boundary.toml`. M061 remains historical accepted evidence for source paths. M062 adds a complementary dependency authority.

### 4.5 Add focused dependency containment guard

Create `emissary-cli/tests/m062_dependency_containment.rs` under `#![cfg(feature = "i2pcontrol")]` or an equivalent test configuration that can parse the workspace/package manifests.

The guard must verify semantically, not by fragile comment placement:

1. root `[workspace.dependencies]` does not directly declare `subtle` after the correction;
2. `emissary-cli` declares `subtle` with `optional = true`;
3. `subtle` retains `default-features = false`;
4. `i2pcontrol` activates `dep:subtle` (or exact Cargo-equivalent optional dependency activation);
5. unrelated package features do not activate `subtle`;
6. the dependency-containment manifest and test agree on the exact allowed production files;
7. the M061 source-boundary files remain present and are not modified by the M062 implementation range;
8. `Cargo.lock` did not receive an unrelated rewrite.

Prefer TOML parsing using the repository's existing `toml` test dependency rather than substring-only assertions.

The guard should fail closed if a future I2PControl-only dependency is made unconditional without updating the dependency authority.

## 5. Dependency-boundary design rule

M062 establishes this durable rule:

> A dependency whose only direct consumer is code gated by `feature = "i2pcontrol"` must itself be optional and activated by the `i2pcontrol` feature. It must not become an unconditional dependency of the default Emissary CLI or a workspace-level dependency unless an independently justified non-I2PControl workspace consumer exists.

This rule concerns direct dependency ownership. It does not claim that a crate name can never appear transitively in the feature-disabled resolved graph. For common cryptographic crates such as `subtle`, transitive use by unrelated dependencies may legitimately exist.

Therefore acceptance must not use an invalid criterion such as "`cargo tree` must contain no crate named subtle". The required property is that **the I2PControl direct dependency edge is optional and feature-owned**.

## 6. Behavior invariants

M062 must not change:

- Proposal 170 wire behavior;
- authentication semantics or password comparison implementation;
- token issuance/validation;
- authentication throttle behavior;
- TLS behavior;
- RouterInfo source disposition;
- AddressBook semantics/persistence;
- TunnelManager behavior or supported backend inventory;
- ClientServicesInfo behavior;
- SAM observation;
- router/core/runtime behavior;
- default feature set;
- runtime configuration keys;
- persistence formats.

The accepted RouterInfo matrix remains exactly:

- 43 total;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. Missing tunnel data planes remain out of scope.

## 7. Implementation sequence

### Step A — Reconfirm direct ownership

Before editing, inspect all workspace manifests/source references for a direct `subtle` consumer.

Acceptance:

- the direct-use inventory is recorded in implementation/closure evidence;
- if a non-I2PControl direct consumer exists, M062 stops and is replanned rather than broadening.

### Step B — Correct Cargo ownership

Perform only the root/package manifest edits in §4.1–4.3.

Acceptance:

- root workspace declaration is removed;
- package dependency is optional/local;
- `i2pcontrol` explicitly activates it;
- no unrelated dependency/version/feature changes.

### Step C — Preserve lock resolution

Run Cargo metadata/check commands without intentionally updating dependency versions.

Acceptance:

- `Cargo.lock` remains byte-identical to baseline if Cargo does not require a change;
- if it changes, inspect exact diff and reject unrelated churn;
- no `cargo update` or dependency refresh campaign.

### Step D — Install dependency containment authority

Add the M062 TOML authority and focused static test.

Acceptance:

- machine-readable policy matches actual manifests;
- M061 source boundary remains untouched;
- guard is semantic and fail-closed.

### Step E — Focused regression validation

Run only the bounded checks below. Do not expand CI or create a new broad verification apparatus.

### Step F — Independent closure

Create `062-closure.md`, reconcile registry/roadmap/index, and close M062 only if every acceptance criterion is satisfied.

## 8. Required verification

Run from the implementation head:

```bash
# manifest/dependency ownership
cargo metadata --format-version 1 --no-deps

# default/no-I2PControl package still builds
cargo check -p emissary-cli --no-default-features

# I2PControl build still includes working auth implementation
cargo check -p emissary-cli --no-default-features --features i2pcontrol

# focused dependency guard
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment

# authentication behavior regression
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib i2pcontrol::auth

# current source-boundary guard remains valid
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment

# no whitespace/path drift
git diff --check
git diff --name-only a70dd3ac82f12fbea1f8fba51e30a9e2e516650a
```

If repository/toolchain behavior makes the exact test filter syntax invalid, use the narrow equivalent and record the actual command.

Optional diagnostic, not an acceptance gate by crate-name absence:

```bash
cargo tree -p emissary-cli --no-default-features -e features
cargo tree -p emissary-cli --no-default-features --features i2pcontrol -e features
```

Use these to understand feature edges only. Do not fail M062 merely because `subtle` appears transitively through another dependency.

## 9. Explicit acceptance criteria

M062 may close only when all of the following are true:

1. `subtle` is no longer an unconditional direct dependency of `emissary-cli`.
2. `subtle` is declared optional at the package boundary with `default-features = false`.
3. `i2pcontrol` explicitly activates the optional `subtle` dependency.
4. No unrelated feature activates the direct `subtle` dependency.
5. Root `Cargo.toml` no longer carries an I2PControl-only `subtle` workspace declaration, absent a separately evidenced non-I2PControl direct consumer.
6. Root manifest changes are limited to restoring that dependency boundary; no dependency refresh or version churn occurs.
7. `Cargo.lock` is unchanged, or any unavoidable change is narrowly explained and contains no unrelated resolution churn. A broad lockfile rewrite fails closure.
8. No `emissary-cli/src/**`, `emissary-core/**`, `emissary-util/**`, example, workflow, runtime config, or release file changes.
9. `m062_dependency_containment` passes and checks semantic TOML ownership rather than only text comments.
10. The M062 machine-readable authority records the exact dependency rule and allowed production files.
11. M061's source manifest/test remain byte-identical across the M062 production implementation range and still pass.
12. Feature-disabled `emissary-cli` check passes.
13. Feature-enabled I2PControl check passes.
14. Focused authentication tests pass with constant-time comparison unchanged.
15. Proposal 170 accepted behavior, 37/1/5 RouterInfo disposition, M051 blocker, and unsupported tunnel-data-plane scope remain unchanged.
16. No CI/release/fuzz/coverage/platform workflow is added or expanded.
17. No upstream issue, pull request, review, merge request, maintainer contact, contribution preparation, or write occurs.
18. Registry, roadmap, implementation index, and M062 closure record agree on final status.

## 10. Failure and rollback rules

Stop and do not broaden the plan if:

- a non-I2PControl direct consumer of the workspace `subtle` declaration is found;
- making `subtle` optional requires source-code changes;
- Cargo dependency resolution unexpectedly changes unrelated versions;
- M061 source containment fails after manifest-only changes;
- authentication behavior changes;
- default/no-feature build changes for reasons beyond dependency ownership.

Rollback is simple because M062 has no runtime migration or persistence effect: restore the two Cargo manifests to the planning baseline and remove the new M062 test/authority artifacts.

## 11. Security review

The security objective is to reduce the feature-disabled trusted/dependency surface while preserving the reviewed constant-time primitive in enabled I2PControl.

Do not replace `subtle` with hand-written constant-time logic merely to remove a dependency. That would trade a small dependency-containment issue for a worse cryptographic-review issue.

Do not move authentication into core or a shared utility crate merely to justify workspace-level dependency ownership.

Do not generalize M062 into a repository-wide dependency slimming exercise. Other dependencies are out of scope unless the M062 edits directly reveal an I2PControl-only unconditional edge of the same class; if more than a trivial manifest-only correction is required, record and separately plan it.

## 12. Compatibility, migration, and operations

No protocol, persistence, configuration, runtime, or migration change is expected.

The `i2pcontrol` feature remains opt-in. Default/no-feature execution should behave exactly as at M061 closure. Enabled builds continue to use the same `subtle` version and constant-time comparison semantics.

No operator documentation update is required unless implementation discovers that the feature/dependency contract is currently documented inaccurately.

## 13. Planning and closure disposition

At plan creation:

- M058–M061 remain closed and are not invalidated;
- M062 is the sole dependency-ready containment corrective;
- M051 remains independently blocked;
- overall Proposal 170 remains partial at 37/1/5;
- no other implementation plan becomes ready.

At successful closure:

- the source-containment authority remains M061;
- dependency-surface authority is added by M062;
- the containment roadmap returns to closed with no ready successor;
- future changes to I2PControl direct dependencies must satisfy the M062 guard.

## 14. Internal-only rule

All work is internal to `eggstack/emissary`.

The pinned upstream repository/specification is read-only evidence. This plan does not authorize any upstream issue, pull request, review, merge, discussion, submission, adoption request, maintainer outreach, contribution package, branch, tag, release, or connector write.

**Handoff: implement M062 only. Do not modify runtime/core source or reopen Proposal 170 source completeness.**
