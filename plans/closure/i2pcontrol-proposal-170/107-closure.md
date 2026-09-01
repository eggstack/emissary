# M107 Closure — I2PControl Conformance and Managed-TLS Corrective Pass

Status: **closed**

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/107-i2pcontrol-conformance-and-managed-tls-corrective-pass.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Review date: 2026-09-01.

## 1. Disposition and implementation head

M107 completed its bounded corrective objective at:

- `27a0376` — `fix(i2pcontrol): close M107 conformance corrective pass`

The pass corrected API-version negotiation, AddressBook cross-book shadowing,
and managed I2PControl TLS publication without changing TunnelManager option
support or moving ownership outside `emissary-cli/src/i2pcontrol/**`.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| API 1 is the only accepted Authenticate version | `auth::tests::validate_api_version_*`, handler tests, full feature suite | pass |
| API 2 returns `-32006` before token issuance | `server::tests::unsupported_api_version_does_not_issue_a_token`; token count remains zero | pass |
| API 1 success and existing password/throttle behavior | `authenticate_uses_standard_params_and_numeric_api`, password/throttle tests, live runtime | pass |
| Valid cross-book shadowing is retained as independent typed state | `address_book_runtime::tests::cross_book_shadowing_is_typed_persistent_and_precedence_ordered` | pass |
| Configured-generation load, precedence, delete, and restart work | same runtime regression: private/local fixtures, private > local lookup, restart, delete winner, lower entry exposed | pass |
| Legacy migration and RouterInfo projections retain both entries | `production::tests::legacy_address_book_migration_preserves_cross_book_shadowing` and selector assertions | pass |
| Entry, aggregate, path, symlink, and transactional guards remain active | existing runtime/address-book tests plus M061/M062 containment suites | pass |
| Managed key is restrictive and managed directory is owner-only on Unix | `tls::tests::managed_private_key_is_owner_only` (`0600`/`0700`) | pass |
| Managed symlink/non-regular paths fail closed | `tls::tests::managed_symlinks_fail_closed_without_touching_targets` covers key and certificate links | pass |
| Managed certificate validates loopback identities | `tls::tests::managed_certificate_validates_all_loopback_server_names` rustls handshakes for `localhost`, `127.0.0.1`, `::1` | pass |
| Invalid regular material regenerates and valid material is reused | existing managed TLS round-trip/corrupt-material tests | pass |
| Explicit TLS ownership remains unchanged | `TlsConfig::is_explicit` and explicit-loading path unchanged; no explicit-path production changes | pass |

## 3. Changed paths and containment review

The exact implementation paths in commit `27a0376` are:

- `emissary-cli/src/i2pcontrol/address_book_runtime.rs`
- `emissary-cli/src/i2pcontrol/auth.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/rpc.rs`
- `emissary-cli/src/i2pcontrol/server.rs`
- `emissary-cli/src/i2pcontrol/tls.rs`
- `emissary-cli/tests/adversarial.rs`
- `emissary-cli/tests/fixtures/mod.rs`
- `emissary-cli/tests/golden_fixtures.rs`
- `emissary-cli/tests/i2pcontrol.rs`
- `emissary-cli/tests/i2pcontrol_live_runtime.rs`
- `emissary-cli/tests/m062_dependency_containment.rs`
- `docs/i2pcontrol/README.md`
- `docs/i2pcontrol/address-book.md`

No Cargo manifest, `Cargo.lock`, Yosemite/SAM source, `emissary-core`,
`emissary-util`, frontend, workflow, or release path changed. The M062 guard
now contains an exact M107 allowlist for these I2PControl-local and regression
paths; M061/M062 containment passes.

## 4. Verification outcomes

Passed:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
  652 passed
  (final focused production migration and TLS suites also passed)

cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
  1783 passed across 26 suites

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
  1 passed

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
  26 passed across 2 suites

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
  2 suites passed

cargo check -p emissary-cli --no-default-features --features i2pcontrol
  pass

cargo check
  pass

cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
  pass

git diff --check
  pass
```

The required formatter command was run exactly:

```text
cargo fmt --all -- --check
  fails with exit code 1
```

It reports the repository’s established stable/nightly rustfmt mismatch,
including nightly-only-option warnings and pre-existing workspace formatting
differences outside M107. It also reports stable formatting for some newly
added long assertions. No formatter-only churn was retained, and the failure
does not indicate a Rust parse, build, test, or lint defect.

## 5. Invariant, failure, recovery, and security review

- API rejection occurs before password comparison, token issuance, or token-store mutation. API 1 response spelling and `-32005`/`-32006` error codes are unchanged.
- AddressBook books remain four independent maps. Only global cross-book collision gates were removed. Per-entry validation, total bounds, configured path confinement, symlink checks, atomic publication, mutation serialization, and failed-generation preservation remain in force.
- Effective lookup remains private > local > router > published. Deleting a higher-precedence entry exposes the lower entry without changing its typed book. Durable restart and configured artifact loading preserve both entries.
- Managed TLS explicit paths remain operator-owned. Managed directory/file type checks use `symlink_metadata`; generated files publish via same-directory temporary files and restrictive Unix key permissions. Unsafe managed material fails initialization; no plaintext fallback exists.
- Certificate SAN/handshake evidence uses only existing rcgen/rustls dependencies. Invalid regular material keeps the existing bounded regeneration behavior; valid managed material is reused.
- No new lock, task, protocol field, dependency, schema migration, router algorithm, data-plane behavior, or TunnelManager matrix disposition was introduced.

## 6. Compatibility, migration, and scope disposition

API 1 clients are unchanged. Accidental API 2 clients now receive the
normative unsupported-version response and no token. No API 2 alias or
negotiation layer was added.

AddressBook persistence has no schema change. Previously valid single-book
state remains valid; cross-book duplicates are now loadable and are resolved by
existing precedence. No old-generation rewrite was needed. The legacy
migration regression confirms valid private/published shadowing loads and both
typed entries reach RouterInfo projections.

M107 does not implement unrelated base-method parity, define token expiration,
relax AddressBook path confinement, or address any TunnelManager residual
option. Proposal 170 remains partial, with the M095 inventory unchanged at:

- `224 apply`
- `158 blocked_primitive`
- `458 not_applicable`

## 7. Future-plan unblock audit

M107 unblocks no future implementation plan. M104 remains **closed as
blocked** because the 158 applicable TunnelManager residual cells still need
independent primitive, semantic, or architecture evidence and a future live
reclosure. No future plan has a newly satisfied hard dependency, so no plan
status is changed to `ready` and no successor is registered.

The planning controls now record M107 as closed in:

- `plans/implementation/i2pcontrol-proposal-170/107-i2pcontrol-conformance-and-managed-tls-corrective-pass.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- `plans/registry.md`

## 8. Internal-only attestation and final disposition

Proposal 170, current I2PControl documentation, Proposal 118, and I2P naming
documentation were accessed only as read-only external evidence. All writes
remained within the authorized internal `eggstack/emissary` repository. No
upstream issue, pull request, review, adoption request, submission, merge,
release, maintainer contact, or contribution artifact was created or mutated.

M107 is therefore **closed**. Full Proposal 170 support remains correctly
partial, and the residual TunnelManager blockers remain explicitly visible.
