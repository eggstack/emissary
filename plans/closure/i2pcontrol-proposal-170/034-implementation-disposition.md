# M034 Implementation Disposition — AddressBook Setter Truthfulness

Status: implemented; closure accepted

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/034-addressbook-setter-truthfulness.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Implementation commit:

- `be7bc16` — `feat: make I2PControl address book setters truthful`

Frozen implementation/test head: `be7bc16`

## Disposition

M034 is implemented within its bounded AddressBook scope. The enabled runtime
AddressBook manager now exposes one capacity-one typed subscription command
seam. Commands replace the complete active source set through the existing
downloader path, publish the accepted set durably from the manager, and use one
manager-owned refresh worker with one in-flight and one newest pending
generation. Requests fail explicitly while the downloader is unavailable.

`SetConfig` now has an exhaustive pinned-key disposition. Request-selected path
keys return invalid parameters; all other non-empty keys return unsupported
operation errors. Empty configuration is a successful no-op only. Legacy inert
configuration is not promoted during migration and is cleared from an existing
enabled authority.

No core, tunnel, RouterInfo, frontend, CI/release, or upstream files changed.

## Changed-file classification

Production and tests:

- `emissary-cli/src/address_book.rs` — bounded runtime subscription command,
  active-source snapshot, manager-owned refresh processing, URL/size bounds,
  accepted-generation persistence, restart source restoration, and focused
  runtime tests.
- `emissary-cli/src/i2pcontrol/address_book.rs` — exhaustive configuration
  disposition, URL validation, truthful setter responses, and negative tests.
- `emissary-cli/src/i2pcontrol/control_plane.rs` — setter ownership and success
  semantics documentation.
- `emissary-cli/src/i2pcontrol/production.rs` — live runtime subscription
  adapter, inert-configuration cleanup, migration exclusion, and production
  rejection tests.

Documentation:

- `docs/i2pcontrol/address-book.md`
- `docs/i2pcontrol/proposal-170-conformance.md`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/security.md`

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Active `SetSubscriptions` changes live sources | `address_book::tests::set_subscriptions_updates_active_runtime_sources`; manager command path and `runtime_active_subscriptions` | pass |
| Accepted sources survive restart | `address_book::tests::set_subscriptions_restart_restores_accepted_sources`; owner snapshot load | pass |
| Queue/unavailable failure preserves prior state | `address_book::tests::set_subscriptions_queue_failure_preserves_prior_state`; production unavailable-setter test | pass |
| Complete generations under contention | `address_book::tests::concurrent_subscription_replacements_publish_complete_generation` | pass |
| One bounded refresh worker and coalesced pending generation | capacity-one command/refresh channels, manager-owned worker, pending-generation slot; clippy/static review | pass |
| Existing proxy/download/parse/merge path retained | `AddressBookManager::download_with_retries`, shared parse helper, refresh context, existing destination-owner merge tests | pass |
| Full-destination owner coherence remains intact | retained M030 suite plus full feature package suite and `active_download_repairs_a_persisted_base32_seed` | pass |
| Downloader unavailable fails explicitly | started-state gate and queue-failure test | pass |
| Path configuration keys rejected before persistence | `set_config_path_keys_are_rejected`; exhaustive key table | pass |
| Unsupported/unknown configuration keys do not persist | `set_config_unsupported_keys_do_not_persist`; production setter test | pass |
| Legacy inert config is not treated as operational | `legacy_configuration_is_not_promoted_into_runtime_owner`; enabled-authority cleanup | pass |
| Disabled/no-feature isolation | retained M028 tests; no-feature package suite; control seam is feature-gated and not constructed by legacy constructors | pass |
| Public wire remains unchanged | conformance manifest, golden fixtures, full feature package suite | pass |
| No core/tunnel/frontend expansion | frozen changed-path inventory; no `emissary-core/**` changes in implementation commit | pass |

## Verification outcomes

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features` | pass, 54 tests |
| `cargo test -p emissary-cli --no-default-features address_book` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass, 468 library tests plus all integration suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter` | pass, 20 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` | pass, 8 |
| `cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings` | pass |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `git diff --check` | pass |
| `cargo fmt --all -- --check` | fails on inherited stable/nightly formatter differences outside M034 and pre-existing match-arm style in touched files; no formatter spillover retained |

## Failure, recovery, and contention review

- Validation happens before command submission or persistence.
- The unavailable/startup gate and closed-channel path leave the owner state
  unchanged.
- Manager cancellation can leave only a durably accepted complete generation;
  no partial vector is published.
- Network refresh failure is isolated to the manager-owned worker and does not
  roll back an accepted source set or crash I2PControl.
- Concurrent callers serialize complete vectors through the bounded manager
  command channel. Refresh work is limited to one active and one newest pending
  generation.
- Restart initializes the active snapshot from the accepted owner state.
- No persistence or owner lock is held across network I/O.

## Compatibility, migration, and security review

- Existing address-book entry state and M030 full-destination validation remain
  unchanged.
- Existing legacy subscription state remains readable and is used as the
  accepted startup source set when enabled.
- Legacy configuration metadata is ignored on migration; existing enabled
  state is cleared rather than silently treated as an active setting.
- URL, count, per-item, and aggregate bounds are enforced.
- No request-selected path reaches filesystem operations.
- Errors expose neither subscription values, configuration values, destinations,
  tokens, nor state paths.
- The optional command seam is compiled and composed only for runtime-enabled
  I2PControl; disabled/default AddressBook execution remains isolated.

## Unresolved findings

No unresolved M034 high or medium correctness, security, compatibility,
ownership, or scope finding remains. The stable repository-wide formatter
failure is a low tooling finding inherited from the baseline and does not alter
the implementation head. Unsupported tunnel families and unavailable RouterInfo
sources remain intentional roadmap limitations, not M034 defects.

## Internal-only attestation

The implementation and closure evidence are internal repository records.
The pinned Proposal 170 page was accessed read-only for the configuration-key
inventory. No upstream or third-party issue, pull request, review, submission,
adoption request, maintainer contact, or connector write was created. The
maintainer directive to commit and push authorizes publication of this internal
repository branch only.
