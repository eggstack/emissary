# M092 — M091 Authorization, Dependency, and Containment Corrective

Status: ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Corrective predecessors:

- M090 server loopback and IRC half-close corrective and `plans/closure/i2pcontrol-proposal-170/090-closure.md`;
- M091 pre-accept stream concurrency plan and `plans/closure/i2pcontrol-proposal-170/091-closure.md`;
- `plans/003-planning-process.md` §§6–8 and §11.

Planning baseline: `944da7b887b6efbd46601e9fad1c853581f40b8e`.

Known valid pre-M091 implementation/closure baseline: `6d631d4423c7faa761b47a84e07436bbaf5d9ad4`.

M091 implementation commit: `5053ce6b595351b251afb36f1f7d5278ef8f58d1`.

Classification: corrective pass / security-containment governance / dependency rollback.

## 1. Objective

Restore the Proposal 170 tunnel-security line to a truthful, minimally invasive state after M091 implemented a Yosemite vendor/core transport while its registered implementation plan was still blocked and explicitly prohibited that dependency strategy without a later maintainer authorization.

M092 MUST preserve M090 exactly as valid production work. It MUST remove the M091 production/dependency/core/vendor delta, restore the pre-M091 containment semantics, and record the M091 closure as superseded/invalid for current-head authority rather than silently deleting history.

The result is intentionally conservative: the M088 pre-accept/lower-layer concurrency limitation becomes an explicitly accepted residual again. This plan does not authorize a new Yosemite fork, vendoring strategy, git dependency, parallel SAM stack, broader core hook, or alternative lower-layer transport.

The maintainer request that opened M092 authorizes this corrective planning and rollback line only. It does **not** retroactively authorize M091's vendored Yosemite dependency strategy.

## 2. Corrective trigger

The registered M091 plan at planning commit `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3` stated:

- `Status: blocked`;
- no supported in-repository configuration path existed from `i2pcontrol` through Yosemite/SAM into the streaming manager before `accept()`;
- M091 MUST remain blocked until a maintainer explicitly authorized one narrow internal transport;
- registration did not authorize vendoring/forking Yosemite, an unreviewed git dependency, or a process-global registry.

Commit `5053ce6b595351b251afb36f1f7d5278ef8f58d1` nevertheless implemented the blocked design by:

- switching workspace Yosemite from crates.io 0.7.0 to `vendor/yosemite`;
- adding the full vendored Yosemite crate;
- adding a typed `max_concurrent_streams` Yosemite option;
- changing `emissary-core` SAM/streaming configuration and manager behavior;
- changing accepted-server session construction;
- modifying M060/M061/M062 historical containment machinery to admit those changes.

The following closure commit `944da7b887b6efbd46601e9fad1c853581f40b8e` then rewrote the M091 plan from blocked to closed and described a maintainer authorization that was not present in the registered handoff before implementation.

This violates the planning authority order and corrective-pass rules. A closure record cannot create retroactive implementation authority.

## 3. Why prior verification missed the defect

M091's local tests primarily proved that the implemented lower-layer algorithm behaved as intended. They did not prove that implementation was authorized by the registered plan before the production/dependency changes landed.

The containment tests were also modified by M091 itself to accept the new core/vendor paths and lockfile delta. Therefore passing those amended guards could not independently establish that the expansion was permitted by the earlier authority.

M092 adds explicit baseline/diff checks that compare the corrected tree to the last valid M090 closure state and distinguish planning bookkeeping from production/dependency authority.

## 4. Required end state

After M092 implementation:

1. M090's resolver-free server loopback targets and IRC half-close behavior remain present and unchanged.
2. Root `Cargo.toml` again consumes crates.io Yosemite 0.7.0 exactly as before M091.
3. `Cargo.lock` again contains the crates.io Yosemite 0.7.0 source/checksum entry and has no M091 vendor-only Yosemite dependency expansion.
4. `vendor/yosemite/**` is absent.
5. The three M091 `emissary-core` production changes are removed.
6. The M091 lower-layer session option translation in `accepted_server.rs` is removed while the pre-existing M090/application admission behavior remains.
7. M060/M061/M062 containment semantics are restored to their pre-M091 authority.
8. M091 is truthfully represented as a blocked, unexecuted-by-authority design whose implementation was later rolled back; its closure record remains in history but is marked superseded/invalidated by M092 rather than deleted.
9. M088's lower-layer/pre-accept residual limitation is again the current accepted disposition.
10. No new production behavior outside the rollback is introduced.
11. M093 becomes the next dependency-ready independent security reclosure only after M092 has an accepted closure.

## 5. Exact production/dependency rollback boundary

Use the M090 closure head `6d631d4423c7faa761b47a84e07436bbaf5d9ad4` as the authoritative reference for the files touched by M091 production work.

M092 may revert M091 changes only in:

- `Cargo.toml`;
- `Cargo.lock`;
- `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs`;
- `emissary-core/src/sam/protocol/streaming/config.rs`;
- `emissary-core/src/sam/protocol/streaming/mod.rs`;
- `emissary-core/src/sam/session.rs`;
- `vendor/yosemite/**` — remove the M091-added tree;
- `emissary-cli/tests/m060_containment.rs`;
- `emissary-cli/tests/m062_dependency_containment.rs`;
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`.

Planning/status bookkeeping may additionally update:

- M091 plan and closure records;
- M092 plan/closure;
- M093 plan registration state after M092 closure;
- `plans/registry.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- the tunnel-security roadmap.

No other production, core, router, startup, frontend, dependency, feature, protocol, or tunnel path is authorized.

## 6. Containment restoration rules

M092 MUST restore the **semantics** of M060/M061/M062 as they existed at M090 closure, not preserve M091's self-authorizing exceptions.

Required restorations include:

- remove the M091-added core seam from M060/M061 authority;
- restore the M062 assertion that the retained M061 source-boundary files are unchanged relative to the M062 baseline;
- restore the M062 lockfile requirement to byte-identical-to-baseline behavior;
- remove the M091 `emissary-core` and `vendor/yosemite/**` production allowances;
- restore `062-dependency-containment.toml` lockfile expectation from the M091 vendored-Yosemite exception to the pre-M091 value.

The cumulative M062 planning-path allowlist MAY retain/add exact entries for M092/M093 planning and closure documents, because that is planning bookkeeping rather than production authority. Do not add globs or production exceptions for M092/M093.

Do not weaken a historical assertion merely to make the rollback pass. If an old assertion conflicts with current legitimate planning bookkeeping, add only the smallest exact planning-path entry needed.

## 7. M091 historical disposition

Do not delete M091 history.

Required documentation result:

- restore the M091 implementation plan's blocker/status language to the pre-implementation truth represented at `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3`, or equivalently mark it `blocked / superseded by M092` without claiming the vendor strategy was authorized before implementation;
- retain `plans/closure/i2pcontrol-proposal-170/091-closure.md` as evidence of what landed and what tests were run;
- amend the M091 closure disposition to `corrective pass required / superseded by M092`, explicitly stating that its technical evidence does not cure the missing pre-implementation authority;
- do not rewrite the technical test evidence as if it never occurred.

M092's own closure becomes the authoritative record of the rollback and governance correction.

## 8. Security invariants

M092 MUST preserve:

- exact Proposal 170 API spelling/types/actions/tunnel set;
- all twelve real tunnel backends;
- M090 literal-loopback target normalization and IRC half-close drain;
- authenticated Yosemite/SAM trusted peer identity;
- bounded post-accept `ServerAdmissionState` concurrency/rate/cardinality controls;
- HTTP framing/identity/fingerprint/POST protections;
- IRC bounded registration/connect/inactivity behavior;
- Streamr bounded local-only fanout model;
- persistent server key confinement and redaction;
- startup/control-plane ownership separation;
- no private Destination/key material in diagnostics;
- no upstream interaction.

M092 explicitly accepts the M088 residual that signed-SYN/streaming work can occur before application admission. It MUST NOT claim lower-layer concurrency protection after the M091 rollback.

## 9. Explicit non-goals

M092 MUST NOT:

- redesign the lower-layer admission mechanism;
- retain a partial Yosemite vendor copy;
- replace vendoring with a git/path dependency or local patch;
- add raw SAM command construction or a hidden registry;
- port Java I2P `ConnThrottler` behavior;
- modify tunnel lengths, crypto, router selection, NetDb, transport, or routing algorithms;
- change Streamr fairness/authentication;
- change `httpbidirserver` identity sharing;
- add new Proposal 170 fields/actions/types;
- add hosted CI, fuzz, soak, release, or public-network load machinery;
- prepare or request upstream review/merge/submission.

## 10. Ordered work packages

### A. Freeze the valid M090 boundary

Before editing, compare M090 closure head `6d631d4423c7faa761b47a84e07436bbaf5d9ad4` to current head and identify the exact M091 implementation delta. Confirm M090 files are not accidentally reverted.

### B. Revert dependency/vendor transport

Restore crates.io Yosemite 0.7.0 in `Cargo.toml` and the corresponding lockfile package entry. Remove the entire M091-added `vendor/yosemite/**` tree.

Do not change any unrelated dependency version or lockfile package.

### C. Revert lower-layer production changes

Restore the M091-touched core files and accepted-server session-option seam to their M090 closure state. Preserve all unrelated code that landed outside M091 if current-head evidence shows any such change.

### D. Restore containment authority

Remove M091's core/vendor/lockfile exceptions from M060/M061/M062. Keep only exact new planning-path bookkeeping for M092/M093 if required by the cumulative guard.

### E. Correct historical/status documentation

Make M091's blocked authorization state truthful, mark its closure superseded/corrective-pass-required, register M092 as the only ready tunnel-security implementation handoff, and keep M093 dependency-blocked/unregistered until M092 closure.

### F. Verify rollback equivalence

Prove the production/dependency tree is equivalent to the M090 closure state for all M091-touched production/dependency paths, except for explicitly enumerated post-M090 planning bookkeeping.

## 11. Required regression evidence

At minimum prove:

1. `Cargo.toml` Yosemite dependency matches M090 closure state;
2. `Cargo.lock` Yosemite package entry matches M090 closure state and crates.io source/checksum is restored;
3. no `vendor/yosemite/**` path remains;
4. the three M091 core files match M090 closure state unless a separately documented unrelated post-M090 change must be preserved;
5. `accepted_server.rs` contains no M091 lower-layer session option transport while retaining existing application admission;
6. M090 HTTP/IRC literal-loopback behavior remains covered and passing;
7. M090 IRC half-close behavior remains covered and passing;
8. M060/M061/M062 containment guards again prohibit the M091 core/vendor/lockfile expansion;
9. M091 plan/closure/registry wording no longer claims pre-implementation authorization;
10. no production path outside the exact rollback set changed.

## 12. Verification

Run at minimum:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m060_containment --test m061_containment --test m062_dependency_containment
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Also run exact source comparisons for every M091 production/dependency file against `6d631d4423c7faa761b47a84e07436bbaf5d9ad4` and inspect `git diff --name-status 6d631d4423c7faa761b47a84e07436bbaf5d9ad4..HEAD` so remaining differences are explainable planning/history changes plus any unrelated pre-existing work.

`cargo fmt --all -- --check` may be recorded if runnable, but existing repository rustfmt/nightly configuration drift is not authorization for formatter-only churn.

## 13. Failure, cancellation, restart, and contention semantics

M092 adds no runtime state or concurrency mechanism.

Rollback must preserve the M090/runtime lifecycle semantics already closed. If reverting M091 exposes a compile/test failure because later unrelated work has come to depend on the M091 core/vendor API, stop and document that dependency rather than widening M092. Such a dependency requires a new explicit corrective decision.

No lock, task, socket, session, storage, or migration behavior may be newly introduced by this plan.

## 14. Compatibility and migration

- Proposal 170 JSON-RPC compatibility is unchanged.
- Server tunnel behavior remains as at M090 closure, including application-level admission.
- The M091 earlier pre-accept concurrency reset behavior is removed.
- Unrelated Emissary streaming sessions return to the pre-M091 crates.io Yosemite/core behavior.
- No persisted user data migration is required.
- Dependency provenance returns to crates.io Yosemite 0.7.0.

The security tradeoff is explicit: a medium lower-layer resource/timing residual remains, but the fork regains the smaller audited surface and truthful authorization boundary.

## 15. Acceptance and stop conditions

M092 closes only if:

- M091 production/dependency/vendor changes are fully removed;
- M090 remains intact;
- historical containment semantics are restored rather than weakened;
- M091 closure is no longer represented as valid current authority;
- all required local verification passes except explicitly pre-existing tooling-only formatting drift;
- changed paths remain inside the exact rollback/planning boundary;
- no upstream interaction occurred.

Stop and require a new maintainer decision if implementation would need to retain or replace any M091 lower-layer transport, broaden core changes, alter unrelated dependencies, or modify router/network behavior.

## 16. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/092-closure.md` containing:

- implementation baseline and head;
- exact changed/deleted-path matrix;
- proof M090 remains present;
- proof M091 production/dependency files were restored to the valid pre-M091 state;
- containment guard before/after evidence;
- M091 plan/closure disposition evidence;
- verification command outcomes;
- explicit statement that M088 lower-layer residual remains;
- unresolved findings with severity;
- internal-only/no-upstream attestation;
- dependency decision for M093.

M092 closure MUST NOT itself declare the entire tunnel-security line closed. It only makes M093 dependency-ready.

## 17. Internal-only rule

All writes are confined to `eggstack/emissary`.

External I2P, I2P+, Yosemite, specifications, issues, commits, and pull requests may be read only as correctness evidence. No upstream issue/PR/review/submission/merge/contact/contribution preparation is authorized.