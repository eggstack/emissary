# M063 — M062 Closure Consistency and Indirect Feature-Activation Guard Corrective

Status: closed

Planning baseline: `fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c` — current `master`, containing the M062 dependency-surface implementation and closure record

Pinned upstream comparison baseline: `eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f` — read-only comparison authority

Predecessors:

- M061 source containment closed: `plans/closure/i2pcontrol-proposal-170/061-closure.md`;
- M062 dependency-surface implementation landed at `fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c` and its production dependency correction is accepted;
- M062 closure record: `plans/closure/i2pcontrol-proposal-170/062-closure.md`.

Closure record: `plans/closure/i2pcontrol-proposal-170/063-closure.md`.

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`.

Governance:

- `plans/003-planning-process.md`, especially §7 corrective passes and §10 Proposal 170-specific guards.

## 1. Objective

Close the remaining M062 closure/evidence defects without reopening the accepted dependency implementation, source containment, runtime behavior, or Proposal 170 capability scope.

M062 correctly fixed the production dependency edge: `subtle` is locally owned by `emissary-cli`, optional, default-features-disabled, and explicitly activated by `i2pcontrol`; the root workspace declaration is gone; `Cargo.lock` and production Rust source were not changed. M063 must preserve that state exactly.

M063 has only two implementation purposes:

1. reconcile stale/inconsistent M062 planning and closure records with the repository state that actually landed; and
2. strengthen the existing M062 dependency-containment test so unrelated Cargo features cannot activate `subtle` indirectly through another local feature.

This is a closure-corrective and invariant-test pass, not a dependency, runtime, protocol, or architecture pass.

## 2. Why a corrective pass is required

The post-M062 review identified four bounded defects.

### D1 — M062 implementation plan retains stale readiness status

`plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md` still says `Status: ready` even though the implementation and closure record landed.

This conflicts with the implementation index, registry, and M062 closure record and violates M062 acceptance criterion 18 requiring planning records to agree on final status.

### D2 — stale M062 closure-head identity

The containment roadmap records `a0d9f2dcc15fdeb5fcbe6658c0399ff9c8c9575b` as the M062 closure commit. The registry likewise describes `a0d9f2d` as the post-M062 current master.

That SHA is the M062 planning head. The actual commit containing the M062 implementation and closure record is:

`fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c`.

M063 must correct the historical labels without rewriting the substantive M062 evidence.

### D3 — registry/roadmap lifecycle mismatch

The containment roadmap says closed while the registry table says `active; M062 closed`. The current control surface therefore disagrees about whether a containment handoff is active.

While M063 is ready/active, both registry and roadmap must describe M063 as the sole containment corrective handoff. At M063 closure, both return to closed with no ready successor.

### D4 — M062 guard checks direct but not transitive feature activation

`emissary-cli/tests/m062_dependency_containment.rs::only_i2pcontrol_activates_subtle` currently checks whether `default`, `ui`, or `metrics` directly contain `subtle` or `dep:subtle`.

That catches a direct regression such as:

```toml
ui = ["dep:subtle"]
```

but does not catch an indirect regression such as:

```toml
ui = ["i2pcontrol"]
i2pcontrol = ["dep:subtle"]
```

The latter violates the M062 invariant because enabling the unrelated `ui` feature activates the I2PControl-only direct dependency.

The M062 closure reports direct activation regression injection and `cargo metadata` diagnostics, but the durable static guard does not compute transitive local-feature reachability. The prior verification therefore proved the current graph but did not fully enforce the future invariant.

## 3. Classification

Primary class: invariant/corrective closure.

No external capability is added or changed.

No production dependency correction is required; the accepted M062 Cargo manifest state is frozen.

## 4. Hard invariants

M063 must preserve all of the following:

- root `Cargo.toml` has no `subtle` workspace dependency;
- `emissary-cli/Cargo.toml` keeps `subtle = { version = "2.6.1", default-features = false, optional = true }`;
- `i2pcontrol` remains the only local feature permitted to activate the direct `subtle` dependency;
- `Cargo.lock` remains byte-identical to `fac2a0c`;
- `emissary-cli/src/**` is unchanged;
- `emissary-core/**` is unchanged;
- `emissary-util/**` is unchanged;
- M061 source-boundary authority remains unchanged;
- M062 dependency manifest remains semantically authoritative unless a minimal test-only clarification is strictly required; changing dependency policy is forbidden;
- the reviewed `subtle::ConstantTimeEq` authentication implementation is unchanged;
- accepted RouterInfo disposition remains 43 total / 37 available / 1 protocol-permitted neutral / 5 unavailable;
- M051 remains blocked by absent substantive news/banned-peer owners;
- unsupported tunnel data planes remain unsupported and resource-free;
- no CI/release/fuzz/coverage/platform expansion;
- no upstream or third-party write/review/submission interaction.

## 5. Explicit non-goals

Do not:

- edit `Cargo.toml`, `emissary-cli/Cargo.toml`, or `Cargo.lock`;
- edit any production Rust source;
- refactor authentication or replace `subtle`;
- perform dependency slimming, upgrades, deduplication, workspace cleanup, or lockfile refresh;
- reopen M058–M061 source containment;
- change Proposal 170 selectors, methods, wire types, persistence, compatibility, RouterInfo sources, ClientServicesInfo, AddressBook, TunnelManager, SAM observation, or tunnel behavior;
- implement any unavailable RouterInfo source or missing tunnel data plane;
- add a generalized feature-analysis framework or new build-time tooling;
- add hosted CI or release verification;
- rewrite historical closure records merely for style;
- seek or prepare upstream review/merge/submission.

## 6. Authorized implementation paths

The implementation agent may modify only:

- `emissary-cli/tests/m062_dependency_containment.rs`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md` — status/closure pointer only;
- `plans/implementation/i2pcontrol-proposal-170/063-m062-closure-and-feature-guard-corrective.md` — status only at closure if repository ceremony requires it;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`;
- `plans/registry.md`;
- `plans/closure/i2pcontrol-proposal-170/063-closure.md` — new closure record.

The implementation should not modify `062-closure.md`; preserve it as the historical record that motivated M063. M063 closure should identify and correct its stale claims externally rather than rewriting prior evidence.

If any Cargo manifest, lockfile, production source, workflow, example, runtime configuration, or release file appears in the M063 diff, stop and re-evaluate. Such a change is outside this plan.

## 7. Ordered work packages

### WP1 — freeze the accepted M062 production state

Before editing:

1. record current head `fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c` as the M063 planning baseline;
2. verify the M062 manifest diff remains exactly the accepted dependency correction;
3. verify `Cargo.lock` is unchanged from the M062 planning baseline and is not modified by M063;
4. verify M061 boundary files are unchanged;
5. verify no current production source defect requires reopening M062.

If the production dependency state differs materially from these assumptions, stop M063 and record the discrepancy rather than broadening scope.

### WP2 — harden indirect Cargo feature activation detection

Modify only `emissary-cli/tests/m062_dependency_containment.rs`.

Replace or extend the current direct-string check with a small semantic local-feature reachability check over `emissary-cli/Cargo.toml`.

Required behavior:

- parse `[features]` from TOML;
- for each forbidden root feature (`default`, `ui`, `metrics`), compute the transitive closure of local feature references;
- detect a violation if any reachable feature entry independently activates the direct `subtle` dependency;
- detect the canonical direct activation syntax `dep:subtle`;
- treat a dependency feature edge such as `subtle/<feature>` as activating `subtle`;
- do not treat weak dependency-feature syntax `subtle?/<feature>` as independently activating `subtle` unless another reachable edge activates it;
- recurse through local feature names, including `ui -> i2pcontrol -> dep:subtle`;
- handle feature cycles deterministically with a visited set rather than recursion without bounds;
- fail closed if a forbidden feature can reach `i2pcontrol` or another local feature that activates `subtle`;
- continue permitting transitive crate presence from unrelated package dependencies; this guard is about Cargo feature activation of the direct `emissary-cli` dependency, not crate-name absence from the resolved graph.

Keep this helper local to the test. Do not introduce a new crate, build script, generalized parser package, or production abstraction.

Required regression evidence must include at least:

1. current manifest passes;
2. direct forbidden activation is rejected;
3. indirect `ui -> i2pcontrol -> dep:subtle` activation is rejected;
4. an unrelated local feature chain that does not reach `subtle` passes;
5. cycle handling terminates and still detects a reachable activation;
6. weak `subtle?/feature` alone is not falsely treated as direct activation.

These may be table-driven unit tests against an in-memory feature map/TOML fixture inside the existing test target. Do not mutate repository manifests during the test suite.

### WP3 — reconcile M062 planning records

Make only factual/lifecycle corrections:

- change the M062 implementation plan top-level status from `ready` to `closed` and add a concise pointer that M063 corrected closure-record consistency/guard completeness;
- preserve the M062 plan's original requirements and acceptance criteria;
- correct roadmap text so `a0d9f2d` is identified as the M062 planning head, not closure commit;
- identify `fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c` as the actual M062 implementation/closure commit;
- correct registry text that calls `a0d9f2d` current master after M062 closure;
- while M063 is in progress, registry/roadmap/index identify M063 as the sole dependency-ready/active containment corrective;
- at M063 closure, registry/roadmap/index agree that M063 is closed and no containment successor is ready.

Do not alter the accepted 37/1/5 matrix, M051 blocker, M061 path authority, M062 dependency rule, or internal-only boundary.

### WP4 — focused verification and closure

Create `plans/closure/i2pcontrol-proposal-170/063-closure.md` only after implementation evidence exists.

The closure record must explicitly distinguish:

- M062 production dependency implementation: accepted and unchanged;
- M062 historical closure-record defects: corrected by M063;
- M062 static guard weakness: corrected and regression-tested by M063.

Do not claim Proposal 170 source completeness.

## 8. Failure, cancellation, restart, and contention semantics

M063 creates no runtime task, channel, lock, service, persistence, or network behavior. Therefore runtime cancellation/restart/contention semantics are unchanged by construction.

Implementation failure rules:

- if indirect-feature detection requires Cargo manifest changes, stop;
- if strengthening the test reveals the current manifest actually permits a forbidden indirect activation, stop and create a separate manifest corrective rather than editing manifests under M063;
- if any M061 source-boundary test fails for reasons unrelated to M063 test code, record the failure and do not expand scope;
- if planning records reveal another materially different containment defect, record it as a separate finding rather than absorbing it into M063;
- if tests require network access or new CI infrastructure, redesign the test to remain local and deterministic.

The test helper must have explicit cycle detection and bounded traversal over the finite manifest feature map.

## 9. Compatibility, migration, security, and operations

Compatibility: none. No wire/API/configuration behavior changes.

Migration/storage: none. No persisted state changes.

Security: strengthens the durable proof that feature-disabled/default CLI configurations cannot acquire an I2PControl-only direct dependency through feature composition. It must not change authentication cryptography.

Operations: none. No daemon/runtime/deployment behavior changes.

Dependency resolution: frozen. No lockfile or version resolution changes.

## 10. Verification commands

Keep verification local and package-scoped.

Required:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
git diff --check
git diff --name-only fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c..HEAD
```

Also verify by exact diff that M063 did not modify:

```text
Cargo.toml
emissary-cli/Cargo.toml
Cargo.lock
emissary-cli/src/**
emissary-core/**
emissary-util/**
examples/**
.github/**
plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml
```

`cargo metadata` or `cargo tree` may be used diagnostically, but the persistent regression proof must live in the deterministic test and must exercise indirect feature reachability.

No full workspace test matrix, CI expansion, fuzzing, soak testing, or release workflow is required.

## 11. Acceptance criteria

M063 may close only when all are true:

1. M062's production manifest state is byte/semantically unchanged from `fac2a0c`.
2. `Cargo.lock` is unchanged.
3. No production Rust source changed.
4. M061 boundary files remain unchanged and `m061_containment` passes.
5. The M062 dependency guard still rejects direct forbidden activation.
6. The guard rejects indirect `ui -> i2pcontrol -> dep:subtle` activation.
7. The guard computes transitive local-feature reachability with cycle protection.
8. Weak dependency-feature syntax does not produce a false positive when it does not independently activate `subtle`.
9. Current `default`, `ui`, and `metrics` feature roots cannot transitively activate the direct `subtle` dependency.
10. Feature-off and feature-on `emissary-cli` checks pass.
11. M062 implementation plan status is no longer stale `ready`.
12. Roadmap identifies `a0d9f2d` as planning head and `fac2a0c` as the actual M062 implementation/closure commit.
13. Registry no longer describes `a0d9f2d` as current post-M062 master.
14. Registry, roadmap, implementation index, M062 plan status, and M063 closure record agree on final containment lifecycle state.
15. M058–M061 accepted closure evidence is not rewritten.
16. M062 closure record remains historical and is not rewritten; M063 closure records the correction.
17. RouterInfo remains 37/1/5 and M051 remains blocked.
18. Unsupported tunnel data planes remain unchanged.
19. No dependency/version/lockfile cleanup or runtime/core change occurs.
20. No CI/release/fuzz/coverage/platform machinery is added or expanded.
21. No upstream/third-party write, issue, pull request, review, merge, maintainer contact, contribution preparation, branch/tag/release, or submission occurs.
22. After M063 closure, no containment successor is registered unless a new concrete defect is independently identified.

## 12. Closure evidence required

`063-closure.md` must include:

- exact implementation commit(s);
- exact changed-path list relative to `fac2a0c`;
- requirement-to-evidence matrix for all 22 acceptance criteria;
- test counts/outcomes for M062 and M061 containment targets;
- explicit regression evidence for direct and indirect activation cases;
- confirmation that Cargo manifests and lockfile are unchanged;
- confirmation that production source paths are unchanged;
- record-consistency review naming corrected stale SHA/status text;
- security review confirming authentication implementation and dependency version unchanged;
- unresolved findings with severity;
- internal-only attestation for read-only upstream evidence and no upstream mutation;
- final disposition.

## 13. Internal-only rule

All work is internal to `eggstack/emissary`.

The I2P specification and `eepnet/emissary` are read-only evidence only. M063 does not authorize any upstream issue, pull request, review, merge, discussion, submission, adoption request, maintainer outreach, contribution package, branch, tag, release, or connector write.

**Handoff: execute M063 only. Correct closure records and indirect feature-activation enforcement; do not touch Cargo manifests, production source, runtime/core behavior, or Proposal 170 capability scope.**
