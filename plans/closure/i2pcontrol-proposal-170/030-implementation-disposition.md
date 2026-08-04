# M030 Implementation Disposition — AddressBook Destination and Owner Coherence

Status: closed for implementation; M030 final-head closure accepted

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/030-addressbook-destination-owner-coherence.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline: `9c35e7f3a09613bd63b51ad12b7832fe75724ab4`

Implementation commit:

- `29b42f29fdd98914ef95d44f80f9353175019ee0` — `fix: make I2PControl address book owner coherent`

Frozen implementation/test head: `29b42f29fdd98914ef95d44f80f9353175019ee0`

## Disposition

M030 is implemented within its bounded corrective scope. Enabled AddressBook
state now has one full-destination owner for administrative, RouterInfo,
Base32, and Base64 views. First activation imports validated destination files,
historical Base32-seeded published entries are repaired only from matching
files, and unrepairable state fails closed before I2PControl startup.

Disabled/default behavior remains the legacy path and ignores control state.
Established control state remains authoritative across re-enable, so stale
legacy files cannot resurrect a deleted published entry. No schema, second
authority, background reconciler, or unrelated Proposal 170 method family was
introduced.

## Changed-file classification

Production and shared owner seam:

- `emissary-cli/src/address_book.rs` — owner-first Base64 lookup, owner-derived
  Base32 lookup, bounded filename-confined full-destination loading, validation,
  import/repair, and active download seed repair.
- `emissary-cli/src/i2pcontrol/production.rs` — activation-time destination
  snapshot import/repair and fail-closed startup integration.

Focused regression fixture:

- `emissary-cli/tests/production_adapter.rs` — existing production CRUD and
  persistence fixtures now use structurally valid destinations required by the
  active owner.
- Focused unit regressions remain in `emissary-cli/src/address_book.rs` and
  `emissary-cli/src/i2pcontrol/production.rs`.

No file outside this list was part of the implementation commit. In particular,
`emissary-core/**`, router/transport/NetDB/SAM/frontend code, CI, release,
packaging, and dependency manifests were unchanged.

## Before/after regression evidence

Before M030, inspection of the frozen baseline showed:

1. `AddressBookHandle::resolve_base64` read the legacy destination file before
   the active owner, so update/delete could return a stale destination.
2. `RuntimeAddressBookOwner::new` copied `addresses` Base32 values into
   published `destination` fields on first activation.
3. `merge_downloaded` used `or_insert`, so a persisted Base32 seed survived a
   full download.

The new regressions directly exercise those baseline failures and pass after
M030: `active_owner_update_and_delete_override_legacy_destination_file`,
`active_download_repairs_a_persisted_base32_seed`,
`first_activation_imports_full_destinations_for_api_and_router_info`,
`persisted_base32_seed_is_repaired_from_matching_destination_file`,
`unrepairable_published_seed_fails_without_mutating_state`, and
`reenable_does_not_resurrect_deleted_published_entry`.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Active owner is authoritative for Base32 and Base64 | owner-first lookup plus update/delete stale-file regression | pass |
| First activation stores full destinations | bounded destination loader and API/RouterInfo regression | pass |
| Historical Base32 seeds repair or fail closed | matching repair and unrepairable-state regressions | pass |
| Active downloads retain full destinations | download seed-repair regression and validated merge | pass |
| Disabled mode remains legacy-only | no-feature suite and retained runtime-disabled isolation tests | pass |
| Re-enable does not resurrect deletions | restart/re-enable regression | pass |
| Import is bounded and path-confined | direct regular-file enumeration, filename/size bounds, symlink rejection, sanitized errors | pass |
| Publication remains failure-atomic | validation before commit; current/backup persistence and existing failure tests | pass |
| No invalid published value is serialized | runtime entry/snapshot validation and full enabled suite | pass |
| No second authority or schema | existing snapshot schema and one owner handle retained | pass |
| No unrelated Proposal 170 scope changed | changed-file inventory and diff review | pass |

## Verification outcomes

All commands used the repository `rtk` wrapper; outcomes were:

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo test -p emissary-cli --no-default-features address_book` | pass, 18 |
| `cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book` | pass, 237 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter` | pass, 20 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` | pass, 8 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest` | pass, 58 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures` | pass, 7 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass, 1,231 |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `cargo fmt --all` | pass with stable-toolchain warnings for nightly-only rustfmt options; generated unrelated changes were reverted |
| `git diff --check` | pass |

## Failure, restart, cancellation, and contention review

Validation and repair complete before publication. A failed import or repair
does not modify the live state or prior current/backup generation. Runtime
mutations retain the existing serialized owner lock and publish durable state
before rebuilding live indexes. Lookup readers observe the old or new complete
state. No network await or new task occurs under the owner mutation lock.

Enabled restart uses current/backup control state; first activation imports once;
disabled restart ignores control state; re-enable restores established state
without merging stale legacy files. Cancellation before publication leaves the
prior generation, while cancellation/response loss after publication leaves the
committed generation. No new contention or cancellation surface was added.

## Compatibility, migration, and security review

Existing valid M022 snapshots remain readable without a schema change. Historical
Base32-seeded published values are repaired only with an exact matching,
validated destination file. Arbitrary disabled-period edits are not imported
over an established authority. Legacy lookup/download behavior is unchanged
when the owner is absent.

Destination files are read only as bounded regular files directly under the
existing directory. Hostnames are filename-confined; symlinked/irregular files
are not imported; entry count, per-file, aggregate, and serialized-state bounds
remain enforced. Errors do not include destination contents or private paths.
Published output is structurally validated, and no mutation authority leaks
through the ordinary lookup handle.

## Findings and attestation

No unresolved M030 high, medium, or low correctness, security, compatibility,
or scope finding remains. Unavailable RouterInfo sources and missing tunnel
data planes remain explicit out-of-scope limitations under ADR-0001.

All work remained inside `eggstack/emissary`. External specifications were
accessed read-only. No upstream repository or maintainer channel was mutated;
no upstream review, merge, adoption, submission, or contribution artifact was
requested or prepared.
