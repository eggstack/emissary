# M030 — AddressBook Destination and Owner-Coherence Corrective Pass

Status: ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Original implementation and corrective records:

- M022, `plans/implementation/i2pcontrol-proposal-170/022-addressbook-runtime-bridge.md`
- M028, `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`
- M029, `plans/implementation/i2pcontrol-proposal-170/029-in-scope-conformance-reclosure.md`
- `plans/closure/i2pcontrol-proposal-170/029-closure-invalidation.md`

Applicable governance and decision:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`

Repository baseline:

- `9c35e7f3a09613bd63b51ad12b7832fe75724ab4`

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`
- status: `Open`
- created and last updated: `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

## 1. Bounded objective

Restore one coherent enabled-mode AddressBook destination authority.

A successful Proposal 170 AddressBook add, update, or delete must be reflected
immediately and consistently by:

- administrative list/lookup;
- RouterInfo address-book selectors;
- normal Base32 resolution; and
- normal Base64 destination resolution.

Every published entry exposed through Proposal 170 must contain a structurally
valid full destination, never a Base32 lookup value copied from the legacy
`addresses` cache.

This pass does not redesign disabled-mode persistence, implement bidirectional
store synchronization, or reopen any other Proposal 170 method family.

## 2. Why prior verification missed the defect

M022 established a runtime control owner and M028 isolated it behind compile-time
and runtime enablement. The retained tests proved immediate Base32 visibility,
restart persistence, feature isolation, and disable/re-enable restoration.

They did not cover two conflicting implementation details:

1. `AddressBookHandle::resolve_base64` consults a legacy destination file before
   the active control owner; and
2. first activation seeds the published control book from the Base32 `addresses`
   map instead of loading full destination files.

The retained delete test asserts only Base32 absence. No regression begins with
an existing legacy destination file, performs a control update/delete, and then
checks both resolution paths. No fixture validates every imported published
entry as a full `Destination`.

M030 must add the tests that would have rejected M029 closure.

## 3. Readiness

M030 is dependency-ready.

Retained evidence that must not be reimplemented:

- M020 base I2PControl and JSON-RPC interoperability;
- M021 TunnelManager wire, validation, atomicity, and secret boundaries;
- M023 startup tunnel inventory and ClientServicesInfo lifecycle truthfulness;
- M024 bounded recoverable SAM observation;
- M025/M026 exact RouterInfo contract and 16/1/26 source classification;
- M027 literal wire fixtures;
- M028 compile-time/runtime AddressBook feature isolation and optional dependency
  ownership.

Current defects are independently visible in the baseline code and are recorded
in `029-closure-invalidation.md`.

## 4. Required invariants

1. When no control owner is active, legacy AddressBook behavior is unchanged.
2. When a control owner is active, it is authoritative for both Base32 and
   Base64 lookup.
3. Active lookup never falls through to a stale legacy destination file after a
   control update or delete.
4. Every active published entry stores a validated full destination.
5. Base32 values are derived from full destinations and are never stored or
   emitted as Proposal 170 destinations.
6. A failed import, repair, or mutation leaves the previous durable and live
   control generation unchanged.
7. Existing current/backup control-state recovery remains intact.
8. Existing compile-time and runtime feature isolation remains intact.
9. Disabled mode does not read, write, migrate, or consult control state.
10. Re-enable restores the retained control authority; it does not silently
    merge arbitrary disabled-period edits into an established authority.
11. No second AddressBook authority, background reconciler, polling task, or
    generic migration framework is introduced.
12. No missing tunnel data plane, RouterInfo source, router algorithm, protocol,
    frontend, SAM, transport, NetDB, or cryptographic behavior is changed.
13. No upstream interaction is authorized.

## 5. Scope and file budget

### 5.1 Primary production scope

Prefer implementation inside:

- `emissary-cli/src/i2pcontrol/production.rs`;
- directly affected `emissary-cli/src/i2pcontrol/**` tests or adapter fixtures.

The production adapter should own activation-time validation, import/repair
policy, and sanitized errors where practical.

### 5.2 Permitted shared AddressBook seam

Changes to `emissary-cli/src/address_book.rs` are allowed only for the minimum
mechanisms that cannot live inside the I2PControl crate:

- owner-aware Base64 lookup precedence;
- a bounded loader/snapshot of legacy full destinations;
- a purpose-specific control-handle method that atomically imports or repairs a
  validated published snapshot;
- focused unit tests for these mechanisms.

Do not move general AddressBook logic into I2PControl and do not expose generic
filesystem or mutation authority.

### 5.3 Conditional composition scope

`emissary-cli/src/main.rs` may change only if activation must pass one bounded
legacy full-destination snapshot or invoke one explicit validation/repair step.
No lifecycle, task, router, proxy, tunnel, or configuration restructuring is
allowed.

### 5.4 Hard file exclusions

M030 must not modify:

- `emissary-core/**`;
- router, transport, NetDB, streaming, LeaseSet, tunnel data-plane, SAM, crypto,
  or frontend modules;
- `.github/workflows/**`;
- release, packaging, publishing, version, coverage, fuzz, soak, platform-matrix,
  or generated-evidence machinery.

If correctness requires a broader shared-module or persistence redesign, stop
and record the blocker instead of expanding M030.

## 6. Target ownership model

### 6.1 Disabled/default execution

Retain M028 behavior exactly:

- legacy `addresses` and `destinations/` files drive lookup;
- downloads update legacy files;
- control-state files are ignored and untouched;
- no control mutation handle exists.

### 6.2 First enabled activation without control authority

The production activation path may import the existing legacy published source,
but it must use full destination files.

Required behavior:

1. enumerate only bounded, filename-confined legacy destination entries;
2. pair each hostname with the file's full destination contents;
3. validate hostname and destination using the same canonical structural rules
   used by the Proposal 170 handler;
4. reject unsafe paths, malformed destinations, excessive entry counts, or
   excessive serialized size;
5. create one complete published control snapshot;
6. persist it atomically before exposing the control service;
7. derive the Base32 runtime map from those full destinations.

The legacy `addresses` cache may be used as a consistency signal, but never as
the source of a Proposal 170 destination value.

### 6.3 Existing control authority

An existing valid current/backup control generation remains authoritative.

Activation must inspect published entries before service startup:

- a structurally valid full destination is retained;
- an invalid value that exactly matches a legacy Base32-style seed may be
  repaired only from a matching validated legacy destination file;
- repair must publish one complete corrected generation before service startup;
- an unrepairable invalid published value fails I2PControl activation with a
  sanitized error and leaves the previous files intact.

This is a schema-preserving data repair, not a new persistence version or a
migration framework.

Legacy entries absent from an established control authority are not
silently imported on every re-enable. This preserves M028's one-authority
transition semantics and prevents deleted entries from being resurrected by
stale files.

### 6.4 Active lookup

When the owner is active:

- Base32 lookup uses the owner's derived effective index;
- Base64 lookup consults the owner first and exclusively for owner-managed
  lookup;
- absence from the owner returns absence rather than falling through to a stale
  legacy destination file.

When the owner is inactive, the existing legacy Base64 file path remains
unchanged.

### 6.5 Active downloads

Downloads merged into an active owner must carry the validated full destination.
They must not preserve or reinsert an incomplete Base32 seed.

Use deterministic replacement rules only where the downloaded source is already
the owner-approved published source. Do not create a new downloader policy.

## 7. Ordered work packages

### WP1 — Freeze the defect with focused regressions

Before production edits, add focused tests demonstrating:

- a legacy destination file followed by control update returns the new
  destination through Base64 lookup;
- a legacy destination file followed by control delete returns no Base64 or
  Base32 result;
- first activation imports full destinations rather than Base32 cache values;
- Proposal 170 list/lookup and RouterInfo serialization expose the full
  destination;
- an active download repairs/replaces an incomplete seed where authorized;
- an unrepairable invalid persisted published value fails activation without
  changing the prior generation;
- disabled mode retains legacy fallback and never consults the owner;
- re-enable restores established control state and does not resurrect a deleted
  hostname from stale legacy files.

The first test run should fail for the expected reason and be recorded in the
implementation disposition.

### WP2 — Add a bounded full-destination snapshot seam

Implement the smallest shared AddressBook helper needed to obtain legacy full
published destinations.

Requirements:

- path traversal is impossible;
- only regular files directly under the existing destination directory are
  considered;
- hostname, entry count, per-entry size, and total size are bounded;
- destination contents are structurally validated;
- errors contain no destination contents or private filesystem details;
- no new dependency is added.

Prefer returning a purpose-specific typed snapshot to the production adapter.
Do not expose a generic directory reader.

### WP3 — Correct activation import and repair

In the production I2PControl adapter:

- replace Base32-cache seeding as the source of published destination values;
- import full destinations on first authority creation;
- validate existing published control entries;
- perform only the bounded schema-preserving repair described above;
- fail closed before starting I2PControl when repair is impossible;
- retain current/backup fallback and atomic publication.

Do not rewrite private/local/router entries unless directly required to reject a
cross-book collision.

### WP4 — Correct active lookup precedence

Make the active owner authoritative for normal Base64 lookup.

Required result:

- update is immediately visible through both resolution forms;
- delete removes both resolution forms even if a stale legacy destination file
  remains;
- inactive mode remains byte-for-byte behaviorally equivalent to the legacy
  path;
- no ordinary lookup consumer gains control mutation authority.

### WP5 — Correct active download merge semantics

Ensure the active published owner receives validated full destinations and does
not retain an incomplete Base32 seed because of `or_insert` behavior.

Keep the existing download retry, source ordering, modified-time, and network
behavior unchanged.

### WP6 — Reconcile evidence and status

Create:

- `plans/closure/i2pcontrol-proposal-170/030-implementation-disposition.md`.

Update directly affected AddressBook/support documentation to state the exact
transition semantics and full-destination authority.

Move M030 from `ready` to `closing` only after the implementation/test head is
frozen and no unresolved high/medium M030 finding remains.

A distinct final closure review is required after M030. Do not mark the
subsystem back to `partial Proposal 170 support` in the implementation commit
or disposition alone.

## 8. Failure, restart, cancellation, and contention semantics

### 8.1 Import and repair failure

- Validation completes before mutation.
- Failed validation writes nothing.
- Failed persistence leaves the prior current/backup generation and live maps.
- Service startup fails explicitly rather than exposing incomplete published
  entries.
- Errors are sanitized and bounded.

### 8.2 Runtime mutation

- Existing serialized owner locking remains the only mutation lock.
- Update/delete publishes durable state before live-index replacement.
- Readers observe either the old complete generation or the new complete
  generation.
- No filesystem read or network await occurs while holding the owner mutation
  lock unless already required by the existing bounded publication path.

### 8.3 Restart

- First activation imports one full-destination published snapshot.
- Subsequent enabled restart uses current/backup control state.
- A repaired generation survives restart.
- Disabled restart ignores control state.
- Re-enable restores established control state and does not merge stale legacy
  files over it.

### 8.4 Cancellation

- Cancellation before publication leaves the prior generation.
- Cancellation or response loss after publication leaves the committed
  generation; retry observes current state.
- No new long-running task requires cancellation handling.

## 9. Compatibility and migration

- Existing router configuration remains valid.
- Existing legacy files remain valid in disabled mode.
- Existing valid M022 control-state snapshots remain readable without schema
  change.
- Historical Base32-seeded published values receive bounded data repair only
  when a matching validated full destination exists.
- Unrepairable values block I2PControl activation but do not block the router
  when I2PControl is disabled.
- No automatic import of arbitrary disabled-period edits into an established
  control authority is added.
- No tunnel, RouterInfo, ClientServicesInfo, authentication, or JSON-RPC public
  compatibility behavior changes.

## 10. Security review requirements

Review and test:

- destination directory confinement and symlink/irregular-file handling;
- maximum file count, per-file bytes, and aggregate bytes;
- destination structural validation before state publication;
- no destination contents, token, password, private path, or raw snapshot in
  logs/errors;
- no stale legacy file can override active control state;
- no invalid published value can be serialized through Proposal 170;
- no mutation authority leaks through `AddressBookHandle`;
- unsupported tunnel backends remain resource-free;
- no upstream interaction occurs.

## 11. Focused tests

Required focused coverage includes exact equivalents of:

- `active_owner_update_overrides_legacy_destination_file`;
- `active_owner_delete_blocks_legacy_destination_fallback`;
- `first_activation_imports_full_legacy_destinations`;
- `persisted_base32_seed_repairs_from_matching_destination_file`;
- `unrepairable_published_seed_fails_activation_without_mutation`;
- `active_download_stores_full_destination`;
- `published_api_and_routerinfo_emit_full_destination`;
- `disabled_mode_retains_legacy_base64_lookup`;
- `reenable_does_not_resurrect_deleted_published_entry`.

Names may follow repository conventions, but each semantic case must exist.

## 12. Verification commands

Run focused tests first, then the bounded package matrix:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features address_book
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Run targeted formatting checks on touched Rust files and `git diff --check`.

Do not repair unrelated `emissary-core` test debt inside M030. Because M030 is
forbidden from changing core/SAM code, package-level CLI evidence plus retained
M024 evidence is the required boundary. Record any pre-existing workspace
failure precisely without weakening changed-path evidence.

Remote CI, coverage, fuzzing, soak testing, network farms, platform matrices,
release checks, and generated evidence bundles are not required.

## 13. Documentation and static guards

Update only directly affected documents:

- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/address-book.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/proposal-170-conformance.md`;
- registry, subsystem roadmap, implementation README, and closure records.

Add static or focused guards proving:

- the control owner remains feature-gated;
- the legacy full-destination loader is bounded and path-confined;
- active Base64 lookup cannot fall through to a legacy file;
- published control entries cannot be emitted without destination validation;
- no new dependency or prohibited file category entered the diff.

## 14. Acceptance criteria

M030 implementation is ready for independent closure only when:

1. every failing regression in WP1 passes;
2. active update/delete is coherent across administrative, RouterInfo, Base32,
   and Base64 views;
3. first activation imports full destinations, not Base32 cache values;
4. existing malformed Base32-seeded values are repaired or fail closed;
5. active downloads retain full destination values;
6. disabled/default behavior and M028 feature isolation remain unchanged;
7. established control authority remains controlling across re-enable;
8. no second authority or new persistence schema exists;
9. all required package-scoped commands pass or an unrelated baseline failure
   is precisely isolated;
10. changed production files remain within the explicit file budget;
11. no unresolved high/medium correctness, security, compatibility, or scope
    finding remains;
12. implementation disposition and frozen head are committed;
13. no upstream write, review request, submission, adoption request, or merge
    solicitation occurred.

## 15. Stop conditions

Stop and record `blocked` or `corrective pass required` if:

- a full fix requires modifying `emissary-core`;
- correctness requires a new persistence schema, tombstone/provenance model, or
  cross-store transactional framework;
- active and legacy stores cannot be kept distinct without a second authority;
- a tunnel data plane, RouterInfo source, router algorithm, SAM behavior, or
  frontend change becomes necessary;
- a new dependency is required without maintainer authorization;
- any secret, destination, or private path is logged;
- an unsupported backend opens resources or reports running;
- the Proposal 170 revision changed;
- upstream interaction occurred.

Do not weaken owner-coherence requirements to avoid a stop condition.

## 16. Required closure evidence

The implementation disposition must contain:

- exact implementation commits and frozen head;
- exact changed-file classification, including justification for every file
  outside `emissary-cli/src/i2pcontrol/**`;
- before/after failing regression evidence;
- requirement-to-evidence matrix;
- command outcomes;
- full-destination import and repair evidence;
- update/delete lookup-coherence evidence;
- failure/restart/cancellation/contention review;
- compatibility and migration review;
- security review;
- unresolved findings with severity;
- scope and internal-only/no-upstream attestation.

A separate review run must inspect the frozen M030 head before restoring the
controlling `partial Proposal 170 support` disposition.