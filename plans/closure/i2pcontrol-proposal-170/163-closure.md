# M163 Closure — Blocked LeaseSet-Security State Persistence Corrective

Status: **closed as complete; zero Proposal promotions; M164 unblocked**

Date: `2026-09-10`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/163-blocked-leaseset-state-persistence-corrective.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m152-blocked-state-corrective-roadmap.md`

Repository baseline reviewed: `4345258a` (M163 registered / dependency-ready)

Implementation commit:

- This closure commit — three-file I2PControl corrective, zero Proposal promotions.

Promotion budget: **zero Proposal cells**. M095 remains `336 apply / 29 blocked_primitive / 475 not_applicable`.

## 1. Executive finding

M163 is complete. The control plane now rejects every supplied blocked
`EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` value before
Create/Edit reaches durable mutation, including compatibility-shaped requests.
Existing runtime/backend rejection remains in place.

Historical tunnel-store state is repaired by a crash-resumable migration. Legacy
or contaminated payloads are sanitized, two clean pending generations are
published, older generation history is removed through a confined fail-closed
purge with directory sync, and only then is `scrub_complete` published. Load
fails before startup reconciliation when any stage fails; a subsequent load
resumes from the pending marker.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Create rejects each blocked field and all three together before persistence | M163 `tunnel_manager` tests; INVALID_PARAMS responses; no fake-store definition | pass | Error responses contain field names only |
| Edit rejects blocked fields without changing definition | M163 edit test | pass | Existing definition remains byte/semantic-equivalent |
| Compatibility and malformed requests have no durable effect | Request gate plus existing parsing tests | pass | Typed fields and blocked raw keys are excluded from new state |
| Legacy state scrub is crash-resumable | M163 tunnel-store migration tests | pass | Legacy and pending markers both resume through purge |
| Two clean fallbacks precede contaminated-history deletion | Migration test and explicit two-publish sequence | pass | Final retained set is clean |
| Purge is narrow, confined, fail-closed, and directory-synced | Generation-store purge tests | pass | Symlinks, deletion failure, and sync failure are rejected |
| `scrub_complete` cannot mask reintroduced contamination | Marker re-entry test | pass | Payload content takes precedence over marker |
| Ordinary generation cleanup semantics remain unchanged | Existing retention suite plus separate purge API | pass | No global retention constant or cleanup behavior changed |
| Proposal matrix and promotion state remain unchanged | M095/M062 planning guards and M163 zero-promotion budget | pass | `336/29/475` retained |

## 3. Production implementation evidence

Exactly these production files changed:

1. `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
2. `emissary-cli/src/i2pcontrol/stores/tunnel_store.rs`
3. `emissary-cli/src/i2pcontrol/stores/generation_store.rs`

No core, util, backend/session-runtime, server-secret-store, Yosemite,
dependency, manifest, lockfile, or Proposal-matrix source changed. No new
secret store or publication owner was introduced.

## 4. Verification executed

### Commands run

```bash
cargo fmt --all
cargo test -p emissary-cli --no-default-features --features i2pcontrol m163 -- --nocapture
rustup run nightly cargo fmt --all -- --check
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings

# Focused clean clippy command with two unrelated repository-baseline lints
# disabled for the installed Clippy version.
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- \
  -D warnings -A clippy::chunks-exact-to-as-chunks \
  -A clippy::field-reassign-with-default

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix
```

### Results

- Targeted M163 run: pass, 8 tests in each of the library and binary targets
  (16 executions total).
- Full I2PControl test suite: pass, `2303 passed` across 40 suites.
- `cargo check`: pass, 3 crates compiled.
- M062 containment/registration suite: pass, 26 tests.
- M095 matrix suite: pass, 3 tests; `336/29/475` unchanged.
- Focused clippy command: pass with no issues.
- The strict installed-Clippy command is not a clean repository baseline: it
  also rejects an unrelated `chunks_exact` call and an unrelated M144 test
  initializer. Those pre-existing lints are excluded only in the focused
  command above; no M163 lint is suppressed.
- Repository-wide formatter check is not clean under the installed formatter:
  the checked-in baseline has extensive unrelated style differences. `cargo
  fmt --all` was run during implementation, and only the three authorized
  production files were retained after discarding formatter-only churn in
  unrelated paths. No M163-specific formatting error remains in the compiled
  or tested code.

## 5. Invariant review

- Blocked fields cannot be represented as successful new or edited durable
  configuration through the manager or tunnel store.
- All three fields are cleared together from every loaded definition, with
  blocked raw-config copies removed as well.
- Unrelated name, type, ownership, start intent, ordinary options, and safe raw
  configuration survive sanitization.
- Startup reconciliation is reached only after scrub completion or a clean
  already-complete payload.
- A completion marker with blocked payload content is untrusted and re-enters
  the pending scrub.

## 6. Failure and recovery review

Publication, purge deletion, and purge directory-sync failure all return an
error before `TunnelStore::load` returns. The pending sanitized generations are
restartable, and the next load repeats the two-clean-generation sequence before
retrying history removal. Purge candidates are regular, non-symlink generation
files confined to the store directory; no broad deletion is used.

## 7. Migration and compatibility review

The scrub marker is serde-defaulted, so pre-M163 payloads deterministically
enter `legacy_unstarted`. Ordinary operations preserve the marker state. The
migration intentionally discards only older generation rollback files after two
clean fallbacks exist; it does not delete tunnel definitions or alter ordinary
generation retention.

## 8. Security review

- No rejected Create/Edit response or diagnostic includes secret values, names,
  or key material.
- No blocked Create/Edit performs a store publication.
- Sanitization does not create a second secret-bearing persistence location.
- After successful scrub, every retained generation is free of the blocked
  fields and associated secret bytes.
- Failed or interrupted scrub cannot proceed to StartOnLoad reconciliation.
- Path confinement, symlink rejection, deletion errors, and directory sync are
  fail-closed in the explicit purge operation.
- No runtime capability was promoted, no downgrade was added, and M146/M147/
  M148/M149-M151 remain untouched.

## 9. Documentation and operations

Planning authority, the M062 dependency record, the implementation README,
registry, roadmap, and this closure now record M163 as closed and M164 as the
sole registered dependency-ready successor. M165 remains deferred until M164
closes.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | Malformed SAM command content is still handled by the separate M164 plan | Outside M163 exact budget; current roadmap remains non-terminal | M164 removes raw rejected-command logging |

No high or medium M163 finding remains.

## 11. Roadmap disposition

M163 is closed and its dependency is satisfied. M164 is now registered and
dependency-ready as the sole implementation handoff. M165 remains deferred and
cannot be registered until M164 closes; the post-M152 roadmap remains active
until the corrective chain and final requalification complete.

## 12. Registry updates

- M163 implementation plan: `registered / dependency-ready` → `closed as
  complete` with this closure link.
- M062 records the exact realized three-file M163 budget and promotes M164 to
  current registration with its exact one-file budget.
- Registry and implementation README record M163 closed, M164 registered, and
  M165 deferred.
- No M095 cell disposition or completion-ledger entry changes.
