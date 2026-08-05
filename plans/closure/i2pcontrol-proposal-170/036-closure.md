# I2PControl Proposal 170 Milestone M036 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/036-auth-and-publication-hardening.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md#6-milestones`

Repository baseline reviewed: `5620cb8`

Implementation commits:

- `fc54ebd` — `feat: harden I2PControl auth and publication`
- `518e05b` — `test: cover publication cancellation semantics`

Frozen implementation/test head reviewed: `518e05b`

Review date: 2026-08-05

## 1. Executive finding

M036 is complete. The authentication boundary no longer relies on an
optimizer-sensitive hand-written comparator and now has bounded peer-keyed
failed-login throttling. I2PControl-owned stores publish complete generations,
retain a recoverable prior generation, sync containing directories where
supported, and update live state only after the documented publication point.
The Proposal 170 wire contract and existing persistence formats are unchanged.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Constant-time comparison | `auth::compare_passwords`, `subtle::ConstantTimeEq`, length/oversize tests | pass | No custom byte comparator remains. |
| Bounded throttle | `AuthThrottle`, 256-entry churn test, bounded-delay test, server auth tests | pass | Deterministic oldest-entry eviction and monotonic window. |
| Successful reset/no lock-held sleep | `successful_authentication_resets_failure_state`; `invalid_password_response` | pass | Failure state is recorded after delay; lock is not held across sleep. |
| Token capacity | `token_eviction_at_capacity` | pass | One oldest token is evicted; wire token shape is unchanged. |
| Shared publication mechanics | `stores/publication.rs` consumers | pass | Fixed owner paths only; no request-selected path. |
| Directory durability qualification | file sync plus Unix directory sync and documentation | pass | Non-equivalent platforms are explicitly qualified. |
| Recovery and stale-temp behavior | generation fallback, server-secret backup, stale-temp tests | pass | Prior generation remains available. |
| Failure preserves live state | publication and directory-sync injection tests | pass | In-memory snapshots update only after publication succeeds. |
| Secret permissions/redaction | restrictive permission and `StoredDestination` tests | pass | Secrets remain absent from Debug/Display/errors. |
| Compatibility and scope | full package/conformance tests and changed-path review | pass | No protocol, core, frontend, CI, or upstream changes. |

## 3. Production implementation evidence

`emissary-cli/src/i2pcontrol/auth.rs` owns the reviewed password comparison,
bounded throttle, and deterministic token eviction. `server.rs` propagates the
accepted TCP peer address into the authentication gate after TLS termination.

`stores/publication.rs` owns the bounded shared publication primitives.
Generation stores use synced temporary files, atomic generation rename, and
directory synchronization. Fixed current/backup stores use the same mechanics;
server destination writers are serialized. The runtime AddressBook owner uses
the helper for its existing `control-state.json`/backup format and remains the
runtime authority.

## 4. Verification executed

### Commands run

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features --features i2pcontrol auth -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol persistence -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol recovery -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

### Results

- Feature and no-feature checks: pass.
- Focused authentication: 44 tests passed.
- Focused persistence: 13 tests passed; recovery-name filter had no matching
  tests, with recovery covered by the package and persistence suites.
- Adversarial: 64 passed; production adapter: 20 passed; production
  composition: 8 passed.
- No-feature AddressBook: 18 passed.
- Full I2PControl package: 1,314 passed across 18 suites, including the
  before-rename and after-directory-sync cancellation tests.
- Clippy with `-D warnings`: pass.
- `git diff --check`: pass.
- `cargo fmt --all -- --check`: not pass on the repository baseline; the
  available formatter reports the checked-in nightly/stable formatting-policy
  mismatch across unrelated pre-existing files. No formatter spillover was
  retained.

## 5. Invariant review

Authentication/error fields and standard error codes remain unchanged. Password
and token material are not logged. Authentication remains before protected
method dispatch. Tokens are random, opaque, bounded, memory-only, and restart
invalidated. Throttle state is bounded, source-keyed, monotonic, and reset on
success; loopback remains the default bind and non-loopback warnings remain.

Publication paths are fixed owner paths, payloads remain bounded, temporary and
irregular/symlink files are rejected, prior generations remain recoverable, and
live state changes follow successful publication. No router-wide storage,
protocol, core, frontend, CI/release, or upstream scope was added.

## 6. Failure and recovery review

Tests cover valid current state, corrupt current with valid backup, missing
state, stale temporary files, failed publication, failed directory-sync
injection, symlink state files, restrictive permissions, concurrent server
secret writers, cancellation before rename, and cancellation after directory
sync. Cancellation before failure recording changes no throttle state;
publication updates live state only after the selected commit point. Restart
loads the newest valid generation and falls back to a prior valid generation.

## 7. Migration and compatibility review

No migration is required. Existing token behavior, password configuration,
generation envelopes, current/backup files, AddressBook control-state files,
tunnel definitions, and server identity formats remain readable. Token state and
throttle state remain intentionally restart-invalidated. Platform qualification
narrows the durability claim without changing stored data.

## 8. Security review

The comparison uses the existing audited `subtle` primitive with explicit
different-length and size handling. The throttle has deterministic capacity,
bounded work, bounded delay, no per-failure task, and no lock-held sleep.
Secrets and source keys are excluded from logs and response data. Fixed paths,
symlink checks, regular-file checks, restrictive Unix permissions, and sanitized
errors are covered. No upstream interaction occurred.

## 9. Documentation and operations

Updated I2PControl security, administrative-state, AddressBook, support,
conformance, and README documentation. Added focused failure/recovery and
contention evidence. The next dependency-ready plan is M037; M038 remains
blocked on M031–M037 and M039 remains blocked on M038.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | Stable/nightly rustfmt policy mismatch remains in the repository baseline | Formatting check cannot be cleanly reproduced with the available default formatter | Retained as baseline tooling evidence; no M036 code defect. |

## 11. Roadmap disposition

Milestone closed and next dependency may proceed. M037 is moved from blocked to
ready. M038 and M039 remain blocked by their named hard dependencies.

## 12. Registry updates

`plans/registry.md` now records M036 closed, M037 ready, M036 findings closed,
and the M036 closure/disposition records. The subsystem roadmap and M036 plan
now record closure and the M037 handoff.

## Internal-only attestation

External sources were accessed read-only where needed. No upstream repository or
maintainer channel was mutated; no upstream review, merge, adoption, or
submission was requested; and no upstream contribution artifact was prepared.
