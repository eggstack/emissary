# I2PControl Proposal 170 Milestone M064 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/064-proposal-170-tunnel-runtime-baseline-corrective.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`

Production planning baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`.

M064 implementation baseline: `fe16cd1` — current `master` before the M064 implementation commit.

Implementation commit:

- `2be451827a85671f1003ac55cd33574b15531fe6` — consume feature-disabled setter arguments with cfg-gated local no-ops; extend the retained M062 path guard for the explicitly authorized M064 core path and approved runtime-planning records.

## 1. Executive finding

M064 is closed. The two accepted `EventHandle` reachability-testing setters now retain their public no-op interface when `events` is disabled without triggering unused-parameter warnings. The `events`-enabled atomic stores, ordering, method signatures, and RouterInfo observation behavior are unchanged.

The literal `cargo check -p emissary-core --no-default-features` command remains independently invalid for this workspace because it enables neither `std` nor `no_std` and exposes pre-existing `RwLock` import errors. The repository CI commands are the authoritative feature-disabled checks: `--features no_std` for no-std and `--features=std` for no-events. Both pass after M064, including the no-events warning-as-error check. The unrelated no-default core test configuration and one existing ML-KEM test failure are recorded below and do not belong to this corrective pass.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Reproduce the accepted feature-disabled regression | Pre-fix `RUSTFLAGS="-D warnings" cargo check -p emissary-core --no-default-features --features=std` failed on exactly the two `testing` parameters at `events.rs:477` and `events.rs:484` | pass | The ordinary CI no-events command emitted the same two warnings with exit status 0; `-D warnings` made the defect explicit. |
| Keep the correction local and semantically neutral | `git show --stat --oneline 2be4518`; production diff contains only `emissary-core/src/events.rs` | pass | Two cfg-gated `let _ = testing;` statements; no signature or field change. |
| Preserve enabled `events` behavior | Enabled branch remains `self.ipv4_testing.store(testing, Ordering::Release)` and `self.ipv6_testing.store(testing, Ordering::Release)` | pass | The stores and release ordering are byte-for-byte unchanged. |
| Restore no-std coverage | `cargo check -p emissary-core --no-default-features --features no_std` | pass | CI-authoritative no-std command succeeds. |
| Restore no-events warning-as-error coverage | `RUSTFLAGS="-D warnings" cargo check -p emissary-core --no-default-features --features=std` | pass | Both previously failing unused-parameter diagnostics are gone. |
| Preserve normal core compilation | `cargo check -p emissary-core` | pass | Default `std` + `events` build succeeds. |
| Exercise enabled event behavior | `cargo test -p emissary-core events:: -- --nocapture` | pass | 3 focused tests passed; 1,061 tests were filtered. |
| Preserve feature-enabled CLI compilation | `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass | I2PControl feature build succeeds. |
| Preserve M061 source containment | `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment` | pass | 7 tests passed. |
| Preserve M062/M063 dependency containment | `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | pass | 19 tests passed after the exact M064 path exception and approved planning-path allowlist were recorded. |
| Avoid runtime/backend scope expansion | `git show --name-only 2be4518`; no `emissary-cli/src/i2pcontrol/**` or backend registry changes | pass | No tunnel type, listener, session, task, callback, or runtime owner was added. |
| Preserve RouterInfo/source state | Registry and roadmap remain 43 total / 37 available / 1 neutral / 5 unavailable; M051 remains blocked | pass | No RouterInfo source/value mapping changed. |
| Preserve dependencies and lockfile | Implementation commit does not touch `Cargo.toml`, `emissary-cli/Cargo.toml`, or `Cargo.lock` | pass | No dependency or feature ownership change. |
| Avoid broad lint/format/CI expansion | Implementation diff has no broad lint suppression, CI file, release, fuzz, coverage, or platform change | pass | Existing formatter drift was not mixed into M064. |
| Keep external interaction internal-only | Repository writes were limited to `eggstack/emissary`; no upstream/third-party issue, review, merge, submission, or contribution artifact was prepared | pass | No external source access was needed for M064. |
| Identify the next ready handoff | Registry, runtime roadmap, and implementation index now mark M064 closed and M065 ready | pass | M066/M067/M068/M071 remain blocked on M065; M069/M070/M072 remain transitively blocked. |

## 3. Production implementation evidence

The only production source change is in `emissary-core/src/events.rs`:

```rust
#[cfg(not(feature = "events"))]
let _ = testing;
#[cfg(feature = "events")]
self.ipv4_testing.store(testing, Ordering::Release);
```

The equivalent correction appears in the IPv6 setter. When `events` is enabled, the no-op statements are not compiled. When it is disabled, the public setters remain callable and discard their arguments without adding state or behavior.

The M062 test guard also changed in the implementation commit. It recognizes only the exact M064 core path as an authorized corrective exception and enumerates the approved Proposal 170 runtime planning records already present relative to the M062 fork baseline. It does not alter the M062 direct-dependency rule, source boundary, lockfile rule, or forbidden feature activation checks.

## 4. Verification executed

### Commands run

```text
# pre-fix reproduction
RUSTFLAGS="-D warnings" cargo check -p emissary-core --no-default-features --features=std

# authoritative baseline checks
cargo check -p emissary-core --no-default-features --features no_std
cargo check -p emissary-core
RUSTFLAGS="-D warnings" cargo check -p emissary-core --no-default-features --features=std
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core events:: -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

### Results

- Pre-fix no-events warning-as-error check failed with exactly two unused `testing` parameters in the accepted setters.
- Post-fix no-std, no-events `-D warnings`, normal core, default CLI, I2PControl CLI, focused event tests, M061, M062, and `git diff --check` passed.
- `cargo check -p emissary-core --no-default-features` was also run as written in the implementation plan. It failed before reaching the targeted warning with pre-existing `RwLock` errors because neither `std` nor `no_std` is selected. This is not the repository’s CI no-std command and was not absorbed into M064.
- `cargo test -p emissary-core --no-default-features` was run as written in the implementation plan. It failed with 105 pre-existing test/configuration errors, including missing event-only test symbols and mismatched `EventManager::new` calls. The focused event tests and authoritative feature checks pass; no unrelated test repair was added.
- `cargo test -p emissary-core` compiled the enabled suite but reported one unrelated existing ML-KEM failure (`client_ml_kem_1024_server_ml_kem_512`) amid otherwise passing test groups. This does not involve `events.rs` or M064 behavior.
- `cargo fmt --all -- --check` was run and reported extensive pre-existing stable-rustfmt drift across unrelated files, including `emissary-cli/src/address_book.rs` and many core/util paths. No unrelated formatting cleanup was mixed into this corrective pass.

## 5. Invariant review

- No field, atomic, event, callback, owner, timer, RouterInfo source, or runtime task was added.
- The enabled stores retain exact `Release` ordering and the existing boolean values.
- The feature-disabled setter signatures remain public and callable.
- The accepted RouterInfo 37/1/5 matrix is unchanged.
- M061 source-containment authority and M062/M063 dependency authority remain in force.
- No I2PControl public behavior, tunnel backend registry, startup/runtime ownership, or transport/session code changed.
- No new dependency, lockfile entry, CI job, release machinery, fuzz target, or coverage machinery was introduced.

## 6. Failure, recovery, and contention review

M064 adds no runtime state or tasks, so there are no new duplicate-request, cancellation, restart, persistence, lock, contention, resource-release, or stale-generation behaviors. The feature-disabled setters are deterministic no-ops and remain safe to call repeatedly. Enabled behavior remains the pre-existing atomic publication path.

The unrelated no-default test failures and ML-KEM failure were isolated from the targeted setter path; no recovery or scope expansion was attempted for them.

## 7. Migration and compatibility review

There is no schema, storage, configuration, protocol, wire, or migration change. Public method signatures and feature-disabled call compatibility are preserved. The patch is rollback-safe as a local source change, subject to the unrelated baseline failures documented above.

## 8. Security review

The change restores warning-as-error coverage for an accepted core observation path without changing network-state semantics, reachability behavior, event publication, authentication, authorization, secrets, or address handling. No new input, output, privilege boundary, or resource allocation exists.

## 9. Documentation and operations

Updated planning lifecycle records:

- M064 implementation plan status and closure pointer;
- runtime implementation index;
- active registry and dependency-ready handoff;
- tunnel-runtime subsystem roadmap;
- this closure record.

No operational procedure, deployment behavior, CI job, or release artifact changed.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| medium | `cargo test -p emissary-core --no-default-features` is independently broken by existing event/test configuration errors | The literal plan command cannot serve as a no-events test command | Keep using the CI-authoritative `--features no_std` check and `--features=std` no-events check; repair the broader test configuration only under a separate plan. |
| low | Stable `cargo fmt --all -- --check` reports broad pre-existing formatting drift | Full repository formatter gate is not green on this toolchain | Do not fold unrelated formatting cleanup into M064; address separately if maintainer-authorized. |
| low | Full enabled core test run has an unrelated ML-KEM failure | Broad suite is not fully green on this run | Triage separately; M064’s focused event tests and compilation checks pass. |

These findings do not invalidate the narrow M064 corrective invariant or block the M065 handoff.

## 11. Roadmap disposition

M064 is closed and the next dependency may proceed. M065 is ready because its only hard dependency, M064, is now closed. M066, M067, M068, and M071 remain blocked on M065; M069 remains blocked on M065 and M066; M070 remains blocked on M067 and M068; M072 remains blocked on M066–M071. M051 remains independently blocked by the accepted RouterInfo news/ban semantic limitation.

## 12. Registry updates

- `plans/registry.md`: M064 moved to recently closed; M065 is the sole dependency-ready handoff; later tunnel families retain their declared blockers.
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`: status reconciled to M064 closed / M065 ready, with the M064 closure link.
- `plans/implementation/i2pcontrol-proposal-170/README.md`: current handoff and sequence reconciled to M064 closed / M065 ready.
- `plans/implementation/i2pcontrol-proposal-170/064-proposal-170-tunnel-runtime-baseline-corrective.md`: status moved from ready to closed and linked to this record.

Internal-only attestation: all repository writes, including the later internal push, are scoped to `eggstack/emissary`. No upstream or third-party repository or maintainer channel was mutated, and no upstream review, merge, adoption, submission, or contribution package was requested or prepared.

Final disposition: **closed**.
