# M123 — M120 Commit-Phase Cancellation Atomicity Corrective Closure

Status: **closed**

Implementation commit: `7e58457`

Plan: `plans/implementation/i2pcontrol-proposal-170/123-m120-commit-phase-cancellation-atomicity-corrective.md`

## Disposition

M123 is closed internally. It supersedes only M120's stronger commit-phase
cancellation-completeness claim; the historical M120 closure remains unchanged.
No Proposal 170 matrix cell changed. M095 remains exactly `284 apply / 98
blocked_primitive / 458 not_applicable`.

## Requirement and finding evidence

| Requirement/finding | Evidence | Disposition |
|---|---|---|
| F1 — guard disarmed before durability awaits | `terminalize_server_start` transfers `PreparedServerStart`, including `ServerStartGuard`, to a spawned transaction owner before `commit_server_start` reaches secret or definition persistence. The guard is no longer disarmed. | Closed |
| F2 — caller cancellation releases lifecycle exclusion | `start` and `restart` pass the owned per-name `OwnedMutexGuard` into the terminalizer. The task drops it only after commit/rollback and result publication. M123 fresh and restart contention tests prove competing `stop` waits. | Closed |
| F3 — best-effort staged cleanup | `ServerDestinationStore` in-memory state uses a short-lived synchronous mutex. `discard_sync` cannot silently skip cleanup because of `try_lock` contention; no file I/O occurs under that mutex. Existing staged cleanup tests remain green. | Closed |
| F4 — missing commit-boundary regressions | Test-only commit barriers pause before secret commit, after fresh secret commit, after replacement secret commit, and during existing-identity public persistence. Caller abort is exercised at the paused boundaries. | Closed |

## Terminal-state model

The server start transaction has one owner from staged preparation through a
terminal result:

1. `PreparedServerStart` owns staged fresh/replacement state and the exact
   replacement secret snapshot.
2. Backend start succeeds and publishes a public destination.
3. A bounded Tokio terminalizer owns the prepared transaction and the
   per-name lifecycle guard before the first commit-phase await.
4. Secret durability is committed before definition/public-destination
   durability.
5. On persistence failure, runtime stop and fresh-secret removal or exact
   replacement-secret restoration run within the same owner.
6. The owner marks terminalization complete, releases lifecycle exclusion, and
   then sends the result to the caller.

Thus caller cancellation can discard only the result wait. It cannot discard
the transaction owner, staged guard, runtime stop/rollback path, or lifecycle
exclusion. Fresh, replacement, and existing-unchanged paths all retain one
coherent committed or rolled-back outcome.

## Cancellation-boundary tests

| Test | Paused boundary | Assertions |
|---|---|---|
| `m123_abort_before_fresh_secret_commit_terminalizes_and_holds_lifecycle` | Fresh candidate staged, runtime started, before secret commit | Caller abort; secret is not yet durable; competing stop cannot pass; terminalizer commits, clears staging, persists identity/public destination, and competing stop then succeeds. Reload confirms the same secret and definition. |
| `m123_abort_after_fresh_secret_commit_finishes_definition_persistence` | Fresh secret durable, before definition persistence | Caller abort; definition still lacks the candidate identity while secret is durable; terminalizer completes definition/public persistence; staging and reload state are coherent. |
| `m123_abort_after_replacement_secret_commit_finishes_matching_definition` | Replacement secret durable, before replacement definition/public persistence | Caller abort; candidate secret is visible while old public definition remains; terminalizer completes the matching replacement definition, preserves the identity, and reload confirms the replacement secret. |
| `m123_abort_existing_unchanged_start_finishes_public_persistence` | Existing unchanged identity during public-destination persistence | Caller abort; terminalizer completes public persistence without changing the existing secret; identity, public destination, runtime stop, and secret remain coherent. |
| `m123_abort_restart_start_phase_terminalizes_before_competing_stop` | Restart's start half before secret commit | Caller abort; competing stop waits for the terminalizer, then observes/stops the coherent committed runtime. |

The existing M120 tests also remain green for preflight ordering, ordinary
runtime/public-persistence rollback, secret redaction, concurrent starts,
startup-managed ownership, and successful stop/restart/reload behavior.

## Restart, contention, and reload evidence

- `restart` retains its original lifecycle guard while stopping the old
  generation and passes that same guard to the new start transaction; no nested
  lifecycle lock is acquired.
- Same-name `stop` is blocked at both the fresh-start and restart commit
  barriers. Different-name operations still use independent lifecycle entries;
  no global start lock or task registry was introduced.
- Reload assertions cover fresh and replacement durable secret/definition
  state. The existing unchanged test verifies the retained identity and secret
  after terminalization, and all M120 reload tests pass.
- Startup-managed definitions never enter the transaction path and the
  startup-managed regression remains green.

## Changed-path and containment audit

Production changes are limited to the authorized paths:

- `emissary-cli/src/i2pcontrol/production.rs` — terminalizer ownership,
  lifecycle transfer, explicit commit-boundary test seam, and M123 regressions.
- `emissary-cli/src/i2pcontrol/server_secret_store.rs` — deterministic
  in-memory staged-state cleanup mutex.

Planning evidence changes are limited to the M123 plan, closure, registry,
implementation README, and post-M114 corrective roadmap. No core, util,
frontend, startup tunnel, manifest, lockfile, dependency, Yosemite, workflow,
or release path changed. M061/M062 containment tests pass with the exact
changed-path allowlist.

## Verification

Successful commands:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Results:

- all three checks passed;
- the I2PControl library suite passed: 709 tests;
- the full I2PControl package suite passed: 752 unit/integration tests plus
  all listed integration suites and doc-tests;
- M061 containment: 7 passed;
- M062 dependency containment: 23 passed;
- M095 matrix: 2 passed;
- M105 residual audit: 1 passed;
- Clippy passed with `-D warnings`;
- the focused M123 suite passed all 5 tests.

`cargo fmt --all -- --check` remains non-zero because the installed stable and
nightly rustfmt toolchains disagree with the repository's committed formatting
and unstable rustfmt settings; the command reports pre-existing repository-wide
drift across unrelated files. Formatter-only output was discarded after
confirming it was unrelated repository-wide churn; the M123-added lines are
formatted and `git diff --check` is clean. This is recorded as the existing
low-severity tooling finding rather than introducing formatter churn into the
M123 commit.

## Security and secret review

- Private destination material remains confined to `ServerDestinationStore`
  and immediate backend construction.
- `StoredDestination` redaction and all M120 no-leak assertions remain intact;
  M123 tests assert neither old nor new private material appears in results.
- No secret enters raw public configuration, RPC responses, logs, Debug, or
  Display output.
- The synchronous state mutex protects only bounded in-memory map operations;
  durable publication and runtime/network awaits remain outside it.
- No new dependency, persistence format, external service, upstream source,
  or administrative authority was introduced.

## Matrix and findings

The authoritative M095 file remains byte/count equivalent at:

```text
apply = 284
blocked_primitive = 98
not_applicable = 458
```

No high or medium M123 finding remains open. The only unresolved item is the
pre-existing low-severity rustfmt toolchain drift documented above.

## Next handoff

M124 remains **blocked on Yosemite Y005 closure**. M123 closure removes the
Emissary-side blocker, but Y005 is still marked ready rather than closed in its
own repository. No future M113/LeaseSet capability audit or Proposal mapping is
unblocked by this closure. The registry and roadmap now reflect that M123 is
closed and M124 is the next blocked handoff.

## Internal-only external-interaction attestation

All writes in this closure were limited to the authorized internal
`eggstack/emissary` repository. External/upstream sources and repositories were
not mutated. No upstream issue, pull request, review, contact, submission,
release, merge, adoption request, or contribution artifact was created or
requested.
