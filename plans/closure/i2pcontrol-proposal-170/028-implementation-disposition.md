# M028 Implementation Disposition — Post-M027 Status and AddressBook Feature Isolation

Status: closed for implementation; M028 closure accepted; M029 ready

Implementation commit: `a65eecb` (`fix: isolate I2PControl address book state`)

Frozen implementation/test head: `a65eecb`

Repository baseline: `03a384aec495232e64468dcf61d60dd2bab5cfe0`

## Disposition

M028 is implemented and closed for its bounded corrective scope. The
Proposal 170 subsystem remains `corrective pass required` until the distinct
M029 final-head review. M029 is moved from `blocked` to `ready` because its
hard dependency—a closed M028 with a frozen implementation/test head—now
exists.

The correction uses one explicit composition boundary:

- `AddressBookManager::new` is the legacy/default path and does not construct
  or consult Proposal 170 control state.
- `AddressBookManager::new_with_control_owner` is feature-gated and is called
  only when I2PControl runtime configuration is enabled.
- `AddressBookHandle` remains the ordinary router lookup handle and only has a
  read-only view of active control state when enabled.
- `RuntimeAddressBookHandle` is the dedicated feature-gated Proposal 170
  mutation/administrative handle.
- downloads and legacy add/remove publication update control state only when
  the active owner exists.

No control-state schema, canonical wire behavior, resolver precedence, source
matrix, SAM behavior, tunnel runtime, or unsupported backend behavior changed.

## Exact changed files

Production and dependency boundary:

- `emissary-cli/Cargo.toml`
- `emissary-cli/src/address_book.rs`
- `emissary-cli/src/main.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/server.rs`

Focused regression/composition tests:

- `emissary-cli/tests/adversarial.rs`
- `emissary-cli/tests/production_adapter.rs`
- `emissary-cli/tests/production_composition.rs`
- unit tests in `emissary-cli/src/address_book.rs`

Directly affected documentation and planning authority:

- `docs/i2pcontrol/README.md`
- `docs/i2pcontrol/address-book.md`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/proposal-170-conformance.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`

The M028 closure records are the evidence files for this disposition.

## Requirement-to-evidence matrix

| Requirement | Evidence | Outcome |
|---|---|---|
| Restore M027 authority and supersede M019 | Registry, roadmap, implementation README, support/conformance docs; M027 invalidation retained | pass |
| No-feature build has no Proposal 170 owner or state access | `RuntimeAddressBook*` owner/handle code is `cfg(feature = "i2pcontrol")`; no-feature check/test/clippy | pass |
| Runtime-disabled feature build remains legacy-only | `AddressBookManager::new` has no owner; `inactive_address_book_ignores_and_preserves_control_state`; full feature suite | pass |
| Stale control state cannot influence disabled lookup or downloader persistence | Focused inactive-mode test proves stale lookup is absent, state bytes are unchanged, and no temporary control file is left | pass |
| Enabled mode has one coherent authority | Explicit enabled composition path, dedicated `RuntimeAddressBookHandle`, shared live maps, production composition tests | pass |
| Four-book precedence, collisions, mutation durability, and immediate lookup remain intact | Retained runtime AddressBook tests, production adapter tests, and full feature suite | pass |
| Downloads merge only while control owner is active | `save_to_disk` conditionally calls `merge_downloaded`; legacy path test and retained enabled suite | pass |
| Disable preserves and ignores; re-enable restores | `control_state_survives_disable_and_restores_on_reenable` | pass |
| Current/backup recovery and failed activation remain fail-closed | Retained runtime owner recovery tests and production initialization path; no schema change | pass |
| `serde_json` ownership is restored | Optional manifest dependency, `i2pcontrol` feature inclusion, `serde_json_is_feature_owned` | pass |
| Ordinary AddressBook trait behavior remains stable | `AddressBookHandle` keeps existing lookup/add/remove/download interfaces; no-feature suite | pass |
| RouterInfo and unsupported runtime scope is unchanged | No related production files changed; retained 16/1/26 matrix and unsupported stubs documented | pass |
| No upstream activity | Local Git history and repository-only changed-file inventory | pass |

## Verification outcomes

All commands were run against frozen head `a65eecb`.

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo test -p emissary-cli --no-default-features address_book` | pass, 18 tests |
| `cargo test -p emissary-cli --no-default-features` | pass, 54 tests |
| `cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book` | pass, 233 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` | pass, 8 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter` | pass, 20 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass, 1,219 tests |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| nightly rustfmt check on all touched Rust files | pass |
| `git diff --check` | pass |

No `emissary-core` command was needed: M028 changed only the permitted CLI
AddressBook/composition seam and its directly affected adapters/tests.

## Failure, restart, cancellation, and contention review

- Disabled/default initialization does not read current or backup control
  state, so stale or corrupt control files cannot fail startup or alter lookup.
- Enabled initialization retains current/backup recovery and reports an
  initialization error when both generations are unusable; the composition
  path fails I2PControl startup rather than substituting an empty shadow store.
- Enabled mutations remain serialized by the existing owner mutex and publish
  durable state before updating live maps.
- Legacy downloads retain their existing warning/retry and addresses/
  destinations persistence behavior. Enabled downloads merge through the one
  active owner only.
- Disabling is restart-based: it leaves control-state files untouched and uses
  legacy sources only. Re-enabling loads the retained generation.
- No network await occurs while the control mutation lock is held by the new
  composition boundary; no new event loop or hot-toggle machinery was added.
- Existing cancellation and post-publication response-loss semantics remain
  unchanged.

## Compatibility, migration, security, and dependency review

Existing router configuration and ordinary AddressBook files remain valid.
Existing M022 control-state files are read with the same snapshot schema when
I2PControl is enabled. No migration or schema rewrite was introduced; the
existing legacy administrative migration remains enabled-mode-only.

Disabled mode cannot be influenced by attacker-planted, stale, or corrupt
control-state files. Enabled mode retains path confinement, bounded state,
atomic current/backup publication, restrictive existing persistence behavior,
sanitized failures, and no raw-state or secret logging. The ordinary lookup
handle does not expose Proposal 170 mutation methods; those methods are on the
dedicated feature-gated control handle.

`serde_json` is optional and feature-owned by `i2pcontrol`; no replacement
dependency or version change was introduced.

The retained RouterInfo source classification remains exactly 16 available,
1 protocol-permitted neutral, and 26 unavailable. Missing tunnel data planes
remain explicit unsupported, resource-free stubs.

## Unresolved findings

No unresolved M028 correctness, compatibility, dependency, or security finding
remains. The high-severity independent final-head evidence gate is now owned by
M029 and is represented as `ready`, not as an M028 defect.

## Internal-only attestation

All writes were confined to `eggstack/emissary`. No upstream issue, pull
request, review request, submission, adoption request, merge activity,
maintainer contact, or third-party repository write occurred.
