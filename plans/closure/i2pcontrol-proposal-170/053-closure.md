# I2PControl Proposal 170 Milestone M053 — M045 Live ProfileStorage Corrective Closure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/053-m045-live-profile-storage-corrective.md`

Corrected milestone and blocked closure:

- M045: `plans/implementation/i2pcontrol-proposal-170/045-routerinfo-known-peer-directory.md`;
- blocked closure: `plans/closure/i2pcontrol-proposal-170/045-closure.md`;
- stale-source implementation attempt: `5ae0477`;
- accepted blocker baseline: `bf9c2eeb`.

Implementation commit: `09a46cb` (`i2pcontrol: wire live peer directory inspection`).

Pinned Proposal 170 revision: `2026-05-20`, unchanged and still Open.

Review date: 2026-08-08

## 1. Final disposition

M053 is closed. It corrects M045's rejected startup `CoreSnapshot` source with a
neutral, cloneable, request-time `ProfileStorage` inspection handle and closes
the three M045 fields. M045 is therefore corrected and closed through this
accepted corrective record. The RouterInfo source matrix is now 19 available,
1 protocol-permitted neutral, and 23 unavailable. The broader subsystem remains
`partial Proposal 170 support`.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Neutral cloneable live core source | `emissary-core/src/inspection.rs`: `PeerDirectoryInspection<R>` privately retains `ProfileStorage<R>` and exposes only owned public entries | pass |
| Narrow Router exposure | `Router::peer_directory_inspection()` in `emissary-core/src/router/mod.rs`; no `RouterContext`, `ProfileStorage`, `Bucket`, or mutable owner crosses the API | pass |
| Post-construction churn | `inspection::tests::peer_directory_snapshot_is_live_after_construction`: snapshot A, `discover_router` mutation, snapshot B, and updated RouterInfo snapshot C use one source instance | pass |
| Oversize and incomplete joins | `inspection::tests::peer_directory_snapshot_rejects_oversize_and_incomplete_results`; source maps failures to bounded/sanitized I2PControl errors | pass |
| Production source composition | `main.rs` retains `LivePeerDirectorySource::new(router.peer_directory_inspection(), 10_000)`; static guard rejects `inspection_snapshot`/`CoreSnapshot` in composition | pass |
| Production source remains live | `production_router_info_retains_live_peer_directory_source` changes the retained source after adapter construction and observes the later value | pass |
| Exact three M045 selectors | `canonical_peer_directory_fields_return_exact_wire_values` proves direct presence, deterministic deduped ordering, and Base64 RouterInfo output for `peers`, `peers.list`, and `peers.info` | pass |
| Single request-group query | `RouterInfoControl::peer_directory()` is queried once by `resolve_proposal_peer_directory`; IDs and RouterInfo bytes come from that one owned snapshot | pass |
| Bounds and response safety | 10,000-item core/source bounds, 4 MiB RouterInfo aggregate bound, and retained final serialized-response limit | pass |
| Source accounting | `PROPOSAL_170_CONTRACT`, conformance manifest, literal fixtures, source map, and documentation agree on 19/1/23 | pass |
| No-feature behavior | no-feature check and test suite; no I2PControl source is composed without the feature | pass |
| Core path budget | `git diff` audit: only `emissary-core/src/inspection.rs` and `emissary-core/src/router/mod.rs` changed in core production; no `profile.rs`, `router/context.rs`, NetDB, or `lib.rs` change | pass |
| Security boundary | Public surface contains only router identity and public serialized RouterInfo bytes; no mutable storage, router, socket, session, transport, tunnel, channel, or private material | pass |
| Internal-only compliance | External authority was read-only; no upstream issue, PR, review, submission, adoption, merge, maintainer contact, or contribution artifact was created | pass |

## 3. Verification executed

Commands and outcomes:

- `cargo test -p emissary-core peer_directory --no-fail-fast` — pass, 2 tests;
- `cargo check -p emissary-core` — pass;
- `cargo test -p emissary-core --no-fail-fast` — pass, 1,055 tests, 2 ignored;
- `cargo check -p emissary-cli --no-default-features` — pass;
- `cargo test -p emissary-cli --no-default-features` — pass, 56 tests;
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol` — pass;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast` — pass, 119 tests;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --no-fail-fast` — pass, 58 tests;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast` — pass, 9 tests;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol` — pass, 1,347 tests;
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` — pass;
- `git diff --check` — pass.

The first full core run had one transient `ml_kem_768_only` integration
failure caused by `Connection reset by peer`; the isolated `ml_kem` suite then
passed 16 tests and the complete core suite was rerun successfully. The
repository-wide `cargo fmt --all -- --check` remains qualified-failing because
the installed stable toolchain cannot apply the repository's configured nightly-only
rustfmt options and unrelated pre-existing files require formatting. No unrelated
formatter changes are retained.

## 4. Invariant, failure, and contention review

The handle performs synchronous read-only collection. It enumerates current
canonical storage at call time, checks the caller bound before returning the
collection, copies public bytes, and releases storage guards before returning.
If a churned entry has no serialized RouterInfo, the source fails closed; it
never fabricates an empty or adjacent value. There is no worker, timer, cache,
poller, persistence, cancellation task, or lock held across I2PControl async or
wire work. Restart reconstructs the handle from the new router instance with no
migration.

No router selection, NetDB behavior, discovery, eviction, transport, tunnel,
congestion, retry, cryptographic, LeaseSet, I2NP, AddressBook, proxy, UI,
workflow, or release behavior changed.

## 5. Future-plan disposition

M045 is now corrected/closed through this M053 record. M046 is unblocked and is
updated to `ready` as the sole next dependency-ready implementation plan. M047,
M048, M049, M050, M051, and M052 remain `blocked` behind their named hard
dependencies; no other future plan became ready. The overall Proposal 170
status remains `partial Proposal 170 support`.

## 6. Internal-only attestation

External Proposal 170/reference material was accessed read-only. No upstream
repository or maintainer channel was mutated, and no upstream review, merge,
adoption, submission, or contribution artifact was requested or prepared.

**Disposition: M053 closed; M045 corrected/closed; M046 ready; RouterInfo source matrix 19/1/23.**
