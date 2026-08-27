# M096 Closure — AddressBook SetConfig Operational Completion

Status: closed

Review date: 2026-08-27

Source plan:

- plans/implementation/i2pcontrol-proposal-170/096-addressbook-setconfig-operational-completion.md

Dependency evidence:

- M095 is closed by plans/closure/i2pcontrol-proposal-170/095-closure.md.
- The authoritative M095 matrix is
  plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml.
- M095 pinned thirteen AddressBook SetConfig keys, their units, and the M096
  containment budget.

## Implementation

The implementation is recorded in internal commit
c438367 (feat(i2pcontrol): operationalize AddressBook SetConfig).
No pull request was created and no upstream interaction occurred.

The implementation changes are confined to the M096 boundary:

- emissary-cli/src/i2pcontrol/address_book.rs — exact key inventory,
  request validation, and truthful error mapping;
- emissary-cli/src/i2pcontrol/address_book_runtime.rs — versioned typed
  configuration, durable authority, path confinement, generation switching,
  artifact publication, restart recovery, and focused tests;
- emissary-cli/src/address_book.rs — the recorded containment amendment:
  the existing neutral downloader seam consumes validated settings and retains
  one bounded refresh worker;
- emissary-cli/src/i2pcontrol/control_plane.rs and
  emissary-cli/src/i2pcontrol/production.rs — composition and contract
  documentation/test updates;
- emissary-cli/tests/m062_dependency_containment.rs — the M096-specific
  allowlist entry for the authorized implementation and documentation paths;
- the M095 matrix, M096 plan, support documentation, and planning indexes.

The neutral downloader amendment was necessary to make cadence, proxy, and
metadata paths operational without creating a second downloader authority.
Only typed settings supplied by the I2PControl owner cross that seam; no core,
router, dependency, startup proxy, frontend, or global logger path was added.

## Requirement-to-evidence matrix

| Requirement | Evidence |
|---|---|
| Exactly thirteen keys | emissary-cli/src/i2pcontrol/address_book.rs::CONFIG_KEYS contains the eight path keys, four behavior keys, and theme; unit coverage asserts the count and validates every key. |
| One durable, versioned authority | RuntimeAddressBookConfiguration is schema version 2 and is stored inside RuntimeAddressBookSnapshot; current/backup persistence is retained. |
| Typed validation before publication | RuntimeAddressBookConfiguration::from_external validates all values and validate_stored rejects invalid persisted generations before activation. |
| Confined path resolution | resolve_confined_path normalizes . and .., rejects absolute/traversal/control/backslash/reserved paths, and rejects symlink, special-file, and non-directory components. |
| Atomic owned-file replacement | atomic_write writes a bounded same-directory temporary file, syncs it, applies restrictive Unix permissions, and renames it into place. |
| Four address-book path keys | artifact_paths, load_configured_generation, and publish_configured_artifacts load and publish private, local, router, and conditionally published books. |
| Subscription metadata paths | RuntimeRefreshSettings, load_runtime_host_modified_times, and write_refresh_file consume and atomically update configured subscriptions, etags, and last_modified files within bounds. |
| Update cadence | update_delay_hours is bounded to 1–720 hours and drives the existing worker’s timer; configuration changes replace the timer deadline without spawning another worker. |
| Proxy host/port | Validated host/port settings are used by build_refresh_client for future downloader refreshes, including correct IPv6 URL bracketing. |
| Publication control | should_publish controls publication of the configured published artifact; disabling it does not remove the prior published file. |
| AddressBook-owned log path | A configured log path is confined and consumed by the owner for a bounded administrative artifact; the global logger is unchanged. |
| Theme semantics | theme is validated, durably round-tripped, and explicitly metadata-only with no frontend coupling. |

## Verification

The following checks were run against the implementation:

- cargo check -p emissary-cli --no-default-features --features i2pcontrol —
  passed.
- cargo test -p emissary-cli --no-default-features --features i2pcontrol —
  passed; all workspace test targets completed successfully, including 615
  library tests.
- cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment —
  passed; 7 tests.
- cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment —
  passed; 19 tests.
- cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings —
  passed with no issues.
- cargo check —
  passed for the default build.
- git diff --check —
  passed.

The plan’s requested m063_feature_reachability target does not exist in this
checkout and no matching test file is present. The available feature
reachability/static-guard targets were inspected; this is recorded as a
repository test-inventory mismatch, not as an M096 implementation failure.

The exact cargo fmt --all -- --check command does not pass in the available
toolchain environment: stable rustfmt reports pre-existing formatting
differences across unrelated repository files, and the installed nightly
rustfmt reports a separate baseline/configuration mismatch. The implementation
files were formatted with the available formatter, and git diff --check
passes. No formatter-only changes outside the M096 file set were retained.

## Invariant and safety review

- Authentication remains enforced by the existing control-plane mutation path.
- All thirteen pinned keys now have explicit semantics; empty SetConfig remains
  a compatibility no-op and unknown keys remain invalid.
- Every configured path is relative to the existing AddressBook administrative
  root and is checked before read or replacement.
- No configuration value controls the global logger or grants arbitrary
  filesystem traversal.
- The control-state current/backup pair is the durable authority; derived
  artifacts are bounded and atomically replaced.
- Existing resolver precedence remains private, local, router, then published.
- The downloader has one existing refresh worker, one bounded command channel,
  and one coalesced pending refresh.
- Errors and warnings do not include configured sensitive path values,
  credentials, or full destinations.
- Feature-disabled/default builds do not construct the I2PControl runtime.
- No tunnel data plane, router algorithm, protocol surface, frontend coupling,
  or upstream workflow was added.

## Failure, recovery, and contention

invalid_path_and_target_failure_preserve_prior_generation proves that
traversal and invalid target content fail before the active configuration is
changed. Configuration mutation is serialized by the owner’s async mutation
mutex. Existing current/backup state recovery remains in use, while persisted
configuration versions and configured artifact contents are validated on
restart. A refresh-worker or proxy construction failure is bounded diagnostic
work after the durable configuration point and cannot create a duplicate
worker. Refresh commands and timer refreshes coalesce to one pending
generation.

## Compatibility, migration, and security

The no-feature and runtime-disabled paths retain the legacy AddressBook
behavior. Historical flat M034 configuration values were inert; the loader
maps them to the version-2 defaults rather than reviving behavior that was
never active. No protocol fields, dependencies, lockfile entries, or wire
responses were added. The RouterInfo configuration view continues using the
existing runtime getter and now reflects the durable accepted values.

Path, file-size, subscription-count, subscription-length, update-delay,
theme, and destination bounds are enforced. Symlinks and special files are
rejected at the administrative boundary. Proxy hosts are constrained to
literal IPs or restricted host syntax, and the downloader remains an
outbound AddressBook consumer rather than an exposed proxy.

## Documentation and operations

The following operational references were updated:

- docs/i2pcontrol/address-book.md;
- docs/i2pcontrol/proposal-170-support.md;
- docs/i2pcontrol/proposal-170-conformance.md;
- docs/i2pcontrol/administrative-state.md;
- docs/i2pcontrol/security.md;
- docs/i2pcontrol/README.md;
- the M095 matrix and M096 planning/roadmap/registry records.

They document the thirteen keys, typed persistence, path confinement,
refresh-worker ownership, failure/restart behavior, publication control, and
metadata-only theme semantics.

## Findings and disposition

No unresolved M096 functional or security finding remains at high or medium
severity. The missing M063 target and rustfmt baseline mismatch are low-severity
repository-process/toolchain findings and do not alter the implementation
disposition. They remain visible here for follow-up rather than being treated
as silent verification passes.

Disposition: closed.

Future-plan review: closing M096 does not newly unblock a later plan. M097 and
M100–M103 remain ready; M098 and M099 remain blocked on M097; M104 remains
blocked on M097–M103. The roadmap, registry, implementation README, and M096
plan record those statuses.
