# M064 — Proposal 170 Tunnel-Runtime Baseline Corrective

Status: closed

Closure record: `plans/closure/i2pcontrol-proposal-170/064-closure.md`.

Planning baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6` — production head reviewed before the tunnel-runtime completion planning series

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/ADR authority:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

Predecessor state:

- M063 closed containment/dependency corrective work;
- current master contains a narrow feature-disabled/no-events regression in an already accepted Proposal 170 observation method;
- no missing tunnel runtime work should begin until this baseline is clean.

## 1. Objective

Repair the current feature-disabled/no-events build regression in `emissary-core/src/events.rs` with the smallest semantically neutral patch possible, then prove that the core/default/feature-enabled baseline is stable enough for M065.

This milestone adds no tunnel type, no runtime capability, no new observation, and no architecture. It exists to prevent later tunnel implementation from obscuring or accidentally carrying forward an unrelated core regression.

## 2. Current evidence and defect

At baseline `a1296b0`, the accepted reachability-testing setters are:

```rust
pub fn set_ipv4_testing(&self, testing: bool) {
    #[cfg(feature = "events")]
    self.ipv4_testing.store(testing, Ordering::Release);
}

pub fn set_ipv6_testing(&self, testing: bool) {
    #[cfg(feature = "events")]
    self.ipv6_testing.store(testing, Ordering::Release);
}
```

When `feature = "events"` is disabled, the parameter is unused. The repository's warning-as-error/no-std check therefore fails even though the enabled behavior is correct.

This is an accepted-core-path regression and must be fixed without changing the event/RouterInfo semantics.

## 3. Classification

Primary class: corrective invariant.

No external capability change.

## 4. Hard invariants

M064 must preserve:

- exact enabled `set_ipv4_testing` and `set_ipv6_testing` store semantics;
- no new field, atomic, event, callback, owner, timer, or RouterInfo source;
- accepted RouterInfo 37/1/5 matrix;
- M061 source-containment authority;
- M062/M063 dependency authority;
- no change to I2PControl public behavior;
- no change to tunnel backend registry;
- no new dependency;
- no change to startup/runtime ownership;
- no CI/release expansion;
- no upstream interaction.

## 5. Explicit non-goals

Do not:

- refactor `EventHandle`;
- change method signatures unless the semantically neutral cfg solution truly requires it;
- add `#[allow(unused_variables)]` broadly at module/crate scope;
- change network status/testing behavior;
- touch transport/session/tunnel runtime code;
- begin M065 or any missing tunnel implementation;
- clean unrelated rustfmt drift;
- update RouterInfo source claims;
- add tests unrelated to the exact feature-disabled regression.

## 6. Authorized production path

The only expected production Rust change is:

- `emissary-core/src/events.rs`.

Planning/closure records may be updated as required by repository ceremony.

If another production path appears necessary, stop and document why before expanding scope.

## 7. Required implementation

Use the smallest local cfg-safe correction. Acceptable shapes include:

```rust
#[cfg(not(feature = "events"))]
let _ = testing;
```

inside each setter, or equivalent cfg-separated parameter/body handling that:

- leaves enabled code generation/semantics unchanged;
- keeps the public method callable in feature-disabled builds;
- does not suppress unrelated warnings.

Do not remove the setters from feature-disabled builds because callers may rely on a neutral no-op interface.

## 8. Ordered work packages

### WP1 — reproduce and freeze defect

Before editing, run the focused command(s) that demonstrate the unused-parameter failure. Record exact compiler output in the closure record.

Confirm the failing lines are the two `testing` parameters and not a broader no-std regression.

If additional independent errors are present, do not absorb them silently; record them and decide whether M064 remains valid.

### WP2 — apply the neutral correction

Modify only the local setter bodies/signatures necessary to consume/rename the parameter under `not(feature = "events")`.

Review generated diff for semantic neutrality.

### WP3 — focused regression evidence

Prove:

- feature-disabled/no-events core check passes;
- normal core check passes;
- feature-enabled behavior compiles/tests;
- no RouterInfo/I2PControl source state changed;
- changed-path list contains no new runtime scope.

### WP4 — closure handoff

Create `plans/closure/i2pcontrol-proposal-170/064-closure.md` only after evidence exists.

At closure, mark M065 ready in registry/roadmap/index and M064 closed.

## 9. Failure, cancellation, restart, and contention semantics

None. M064 must not add runtime state or tasks.

Failure rule: if the regression cannot be fixed without changing runtime semantics, M064 is blocked and a separate corrective plan is required.

## 10. Compatibility, migration, security, and operations

Compatibility: no intended behavior change.

Migration/storage: none.

Security: reduces risk by restoring feature-disabled build coverage for an audited-core path; does not change security policy.

Operations: none.

## 11. Verification commands

Required local checks:

```text
cargo check -p emissary-core --no-default-features
cargo check -p emissary-core
cargo test -p emissary-core --no-default-features
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
git diff --name-only a1296b018ce98d26a019bd5064dff9f4b47e0ad6..HEAD
```

If the repository's exact no-std command differs, use the existing CI command as the source of truth and record it verbatim.

Do not add a new CI job.

## 12. Acceptance criteria

M064 may close only when all are true:

1. the original feature-disabled/no-events unused-parameter failure is reproduced and documented;
2. the fix is local to the two accepted setters or an equivalently narrow region;
3. enabled `events` semantics remain unchanged;
4. feature-disabled/no-events core check passes;
5. normal core check passes;
6. feature-enabled I2PControl CLI check passes;
7. existing M061 containment test passes;
8. existing M062/M063 dependency-containment test passes;
9. no TunnelManager/backend behavior changes;
10. no RouterInfo source/value mapping changes;
11. no dependency/lockfile change;
12. no new production path outside `emissary-core/src/events.rs`;
13. no broad lint suppression is introduced;
14. no unrelated formatting cleanup is mixed in;
15. no CI/release/fuzz/coverage machinery is added;
16. no upstream/third-party write, review, issue, PR, branch, tag, release, or contribution preparation occurs;
17. closure record identifies M065 as the next ready handoff only after the baseline checks are green.

## 13. Closure evidence required

`064-closure.md` must include:

- implementation commit(s);
- reproduced pre-fix failure;
- exact diff and changed paths;
- exact verification command outcomes;
- explicit enabled-semantics review;
- M061/M062 containment outcomes;
- confirmation that RouterInfo remains 37/1/5;
- confirmation that no tunnel-runtime implementation entered M064;
- internal-only external-interaction attestation;
- final disposition.

## 14. Stop condition

Stop rather than broaden M064 if:

- additional unrelated core failures appear;
- a signature/API redesign appears necessary;
- the fix would alter network-state semantics;
- Cargo/dependency changes become necessary;
- implementation begins touching missing tunnel families.

Those findings belong to separate work.
