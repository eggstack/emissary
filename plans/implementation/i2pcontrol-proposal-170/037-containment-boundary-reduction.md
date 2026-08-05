# M037 — I2PControl Containment Boundary Reduction

Status: closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Applicable governance and decisions:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`

Repository baseline:

- accepted M036 implementation/closure head: `5afe953`

Hard dependency:

- M036 closed

Closure records:

- `plans/closure/i2pcontrol-proposal-170/037-closure.md`
- `plans/closure/i2pcontrol-proposal-170/037-implementation-disposition.md`

## 1. Bounded objective

Reduce Proposal 170-specific policy and machinery outside
`emissary-cli/src/i2pcontrol/**` while preserving the now-stable operational
behavior from M031–M036.

M037 targets two demonstrated containment leaks:

1. AddressBook administrative persistence, migration/repair, and configuration
   policy embedded in the original CLI AddressBook module; and
2. bounded SAM observation aggregation/recovery state embedded across several
   `emissary-core` SAM lifecycle files.

The target is not a repository-wide crate extraction. The target is a smaller,
explicit adapter surface: original modules publish or consume narrow runtime
facts, while I2PControl owns Proposal 170 policy, bounds, aggregation,
persistence, and serialization.

M037 may modify core only to remove Proposal 170-specific machinery and leave a
minimal optional passive observation hook. It must not add router behavior.

## 2. Readiness and current evidence

Execute only after functional and security behavior is stable through M036.
Containment refactoring before that point would mix semantics and movement.

Current AddressBook leak includes runtime administrative DTOs, control-state
persistence, import/repair, subscriptions/config metadata, and mutation methods
in `emissary-cli/src/address_book.rs`.

Current SAM leak includes observation maps, recovery limits, stable socket IDs,
incomplete-state recovery, and exported observation handles across core SAM
files. Some lifecycle hook points are necessarily core-owned because only the
SAM owner sees authoritative activation/removal events; the aggregation policy
need not be.

## 3. Required invariants

1. No public Proposal 170 wire, runtime behavior, persistence format, or support
   classification changes.
2. M028 disabled/runtime-disabled AddressBook isolation remains exact.
3. M030 Base32/Base64/full-destination owner coherence remains exact.
4. M034 active subscription semantics remain exact.
5. SAM ClientServicesInfo returns the same complete bounded snapshots and fails
   on incomplete state as before.
6. Core SAM lifecycle, task ownership, sockets, sessions, protocol behavior, and
   performance are unchanged except for a passive optional notification seam.
7. Original runtime modules do not import JSON-RPC handlers or Proposal 170
   request/response types.
8. I2PControl does not gain socket/session/private-key/control handles.
9. No new global event bus, broadcast framework, polling loop, second registry,
   or unbounded queue.
10. No broad crate/workspace reorganization.
11. No missing tunnel data-plane, RouterInfo source, frontend, CI/release, or
    upstream work.

## 4. Scope and changed-path budget

### 4.1 AddressBook target

Prefer moving into `emissary-cli/src/i2pcontrol/**`:

- administrative book DTO/policy;
- control-state generation persistence;
- legacy import/repair policy;
- subscription/config field disposition;
- Proposal 170 validation and bounds.

Leave in `emissary-cli/src/address_book.rs` only:

- legacy downloader/runtime behavior;
- normal AddressBook trait implementation;
- one optional overlay/owner trait object or typed handle for Base32/Base64
  resolve and downloaded-entry publication;
- one bounded runtime subscription command seam from M034;
- no Proposal 170 wire or administrative store policy.

A full move is not mandatory if it increases duplication or risk. Every retained
block must have a documented reason tied to runtime ownership.

### 4.2 SAM target

Prefer moving into `emissary-cli/src/i2pcontrol/**`:

- session/socket observation maps;
- public response bounds;
- complete/incomplete/recovery policy;
- serialization DTOs;
- generation bookkeeping and overflow behavior.

Leave in `emissary-core` only the minimum authoritative hook, such as an optional
trait/callback receiving sanitized lifecycle facts:

- primary session activated/removed;
- socket activated/removed;
- stable identifiers and sanitized peer metadata already available at the hook
  point.

The hook must be no-op/absent by default, feature-neutral where possible, and
must not expose live sockets, destinations, keys, command channels, or mutable
session state.

If moving aggregation outside core would require an unbounded event queue or
would lose authoritative ordering/recovery, retain the minimum current core
state and record the blocker rather than weakening correctness.

### 4.3 Static boundary manifest

Add a machine-readable or test-owned changed-path/dependency boundary that
classifies:

- I2PControl-owned production files;
- permitted narrow adapter files;
- prohibited core/router/protocol areas;
- exact approved core passive hook files, if any.

Do not implement a general repository policy engine.

## 5. Target adapter contracts

### 5.1 AddressBook overlay

The runtime-facing interface should express capabilities rather than
administrative schema, for example:

- resolve full destination;
- resolve derived Base32 address;
- publish validated downloaded entries;
- obtain/replace active subscription sources;
- expose a read-only effective snapshot only where RouterInfo requires it.

The I2PControl owner implements the interface. Disabled mode uses no overlay and
retains legacy behavior.

### 5.2 SAM passive observer

The core-facing observer must:

- be optional and cheap when absent;
- avoid await/blocking in core poll paths;
- use bounded synchronous publication or direct callback into a lock with
  measured critical sections;
- sanitize metadata before crossing the seam;
- preserve event order required for reconstruction;
- report publication failure without crashing or mutating SAM lifecycle;
- never become a second lifecycle authority.

I2PControl owns recovery/overflow semantics and refuses partial snapshots.

## 6. Ordered work packages

### WP1 — Freeze behavior and changed-path baseline

Record current production files outside `i2pcontrol/**`, lines/modules of
Proposal 170-specific policy, and focused behavior tests. Add black-box tests that
must remain unchanged through movement.

### WP2 — Extract AddressBook policy behind the narrow overlay

Move policy/persistence incrementally while keeping file format and owner
semantics. Avoid a simultaneous schema migration.

After each step, run no-feature, runtime-disabled, enabled, restart, and stale
legacy regressions.

### WP3 — Define the SAM passive event contract

Write the internal event DTO and ordering/failure contract inside I2PControl or a
small neutral module only if necessary. Review every field for sensitivity and
runtime need.

### WP4 — Move SAM aggregation/recovery policy

Replace core-owned Proposal 170 aggregation with the passive hook and
I2PControl-owned bounded state. Preserve complete/incomplete recovery tests.

If a hook event cannot represent current authoritative state safely, stop the
SAM extraction portion and retain the minimum existing state with a documented
exception.

### WP5 — Add static containment guards

Tests must fail if:

- I2PControl handlers import core private/session/socket types;
- original client/server/AddressBook modules import JSON-RPC handlers;
- new core files begin depending on I2PControl;
- changed paths exceed the approved manifest without explicit plan update;
- unsupported tunnel backends allocate resources.

### WP6 — Reconcile documentation and disposition

Update architecture/security/support docs and create:

- `plans/closure/i2pcontrol-proposal-170/037-implementation-disposition.md`.

The disposition must include before/after production file counts and a reason
for every remaining non-I2PControl block.

## 7. Failure, cancellation, restart, and contention semantics

### AddressBook

- Movement preserves complete-generation publication and prior recovery.
- Overlay absence means legacy behavior; no fallback to stale control state.
- Failed overlay publication leaves prior live/durable generation.
- No lock is held across network download or filesystem enumeration beyond
  existing bounded publication rules.

### SAM

- Observer failure never crashes or alters SAM session/socket lifecycle.
- An event the I2PControl aggregator cannot represent marks the snapshot
  incomplete; it never returns partial or stale-as-current data.
- Removal events permit bounded recovery as before.
- Core poll paths never await I2PControl.
- Restart begins with an empty observer generation and repopulates from new
  authoritative lifecycle events; no stale cross-restart snapshot.

## 8. Compatibility and migration

- No public wire change.
- Existing AddressBook control-state files remain readable.
- Existing server secret/tunnel-definition files remain unchanged.
- No runtime adoption/migration of startup tunnels.
- Core public exports used only by I2PControl may be removed or narrowed after
  internal callers/tests migrate, but no unrelated public API cleanup is
  authorized.

## 9. Security and performance review requirements

Review and test:

- no new sensitive data crosses the SAM hook;
- no live socket/session handle escapes core;
- no unbounded queue/map;
- hook absent-path overhead is negligible and measured with focused tests or
  code inspection;
- lock critical sections are bounded and non-awaiting;
- AddressBook path/owner isolation and destination validation remain;
- no new dependencies unless explicitly justified;
- no upstream interaction.

## 10. Focused tests

Required semantics include:

- `addressbook_disabled_mode_has_no_control_overlay`;
- `addressbook_enabled_overlay_preserves_base32_base64_coherence`;
- `addressbook_control_state_format_remains_compatible`;
- `sam_observer_absent_does_not_change_core_behavior`;
- `sam_observer_complete_snapshot_matches_retained_fixture`;
- `sam_observer_incomplete_state_recovers_after_removal`;
- `sam_observer_failure_does_not_fail_session_lifecycle`;
- `core_hook_exposes_no_live_or_secret_types`;
- `approved_changed_path_manifest_is_enforced`;
- `original_runtime_modules_do_not_depend_on_jsonrpc_handlers`.

## 11. Verification commands

AddressBook/default boundary:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features address_book
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
```

SAM/core boundary:

```bash
cargo check -p emissary-core
cargo test -p emissary-core sam
cargo test -p emissary-cli --no-default-features --features i2pcontrol client_services
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
```

Broad changed-path matrix:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Use targeted formatting and `git diff --check`. Do not expand CI, release,
coverage, fuzz, or soak infrastructure.

## 12. Documentation and static guards

Document:

- final ownership map;
- exact remaining original-module seams;
- approved core hook and data fields;
- no-op/default cost;
- failure/incomplete semantics;
- changed-path comparison to the pre-M037 baseline.

Static guards must enforce the boundary without relying solely on comments.

## 13. Acceptance criteria

M037 may close only when:

- Proposal 170 policy outside `i2pcontrol/**` is materially reduced or every
  retained block has a direct runtime-ownership justification;
- behavior/persistence/wire remain unchanged;
- SAM core hook is passive, bounded, non-sensitive, and non-authoritative;
- default/no-feature behavior passes;
- no high/medium containment, security, or regression defect remains;
- implementation disposition and frozen head are committed;
- no upstream interaction occurred.

## 14. Stop conditions

Stop the affected extraction and record a precise blocker if:

- moving SAM aggregation loses authoritative ordering or bounded recovery;
- extraction requires an unbounded event channel or polling;
- AddressBook movement creates a second owner or schema migration;
- core behavior/performance changes beyond a passive hook;
- a repository-wide crate/service refactor becomes necessary;
- unrelated production files must change;
- external authority changes materially;
- upstream action is requested without explicit authorization.
