# I2PControl Proposal 170 Corrective Roadmap

Status: partial Proposal 170 support

Current corrective baseline:

- `9c35e7f3a09613bd63b51ad12b7832fe75724ab4`

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`
- status: `Open`
- created and last updated: `2026-05-20`
- `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`
- existing I2PControl authentication and error contract at `https://i2p.net/en/docs/api/i2pcontrol`

Canonical internal references:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/closure/i2pcontrol-proposal-170/029-closure-invalidation.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`

## 1. Purpose

This roadmap closes the remaining demonstrated in-scope Proposal 170 defect
without reopening missing tunnel data planes, unavailable RouterInfo sources,
core router architecture, or general AddressBook redesign.

M020–M028 remain substantial retained evidence. M029's final disposition is
invalidated because enabled-mode AddressBook lookup and published destination
storage are not owner-coherent:

- normal Base64 lookup can return a stale legacy destination file before the
  active control owner;
- first activation can seed Proposal 170 published entries from Base32 cache
  values rather than full destinations;
- an active download can retain the incomplete seed because of `or_insert`;
- retained tests do not exercise control update/delete against a stale legacy
  destination file.

M030 owns this narrow correction.

## 2. Retained implementation and source disposition

The following remain retained unless M030 exposes a direct regression:

- standard base I2PControl authentication, token, error, JSON-RPC, notification,
  and request-ID behavior;
- exact Proposal 170 method names, parameters, casing, and response shapes;
- TunnelManager validation, atomicity, secret handling, startup ownership, and
  explicit unsupported runtimes;
- ClientServicesInfo startup/proxy/I2CP/SAM source behavior;
- M028 compile-time/runtime AddressBook feature isolation;
- exact 43-selector RouterInfo contract;
- 16 available, 1 protocol-permitted neutral, and 26 unavailable RouterInfo
  source dispositions;
- literal contract fixtures;
- internal-only/no-upstream boundary.

Missing tunnel data planes and unavailable RouterInfo sources remain outside
this workstream.

## 3. Current defect and invalidation

Authoritative invalidation:

- `plans/closure/i2pcontrol-proposal-170/029-closure-invalidation.md`

At the M029 closure head:

1. `AddressBookHandle::resolve_base64` reads a legacy destination file before
   consulting the active owner;
2. control update/delete mutates only the active owner;
3. stale legacy files can therefore survive a successful API update/delete in
   normal Base64 resolution;
4. first activation seeds the published control book from the legacy Base32
   lookup cache;
5. Proposal 170 list/lookup/RouterInfo may therefore expose a Base32 value where
   a full destination is required;
6. active download merge does not repair an existing incomplete seed;
7. focused evidence checks Base32 deletion but not Base64 deletion with a stale
   file.

The controlling subsystem status is now `partial Proposal 170 support`; M030
closed the demonstrated in-scope AddressBook defect.

## 4. Scope boundary

### 4.1 In scope

- owner-coherent enabled Base32 and Base64 lookup;
- full-destination import on first enabled activation;
- bounded validation and schema-preserving repair of historical Base32-seeded
  published entries;
- full-destination active download merge behavior;
- update/delete regressions spanning API, RouterInfo, Base32, and Base64 views;
- preservation of M028 disabled/runtime-disabled isolation;
- directly affected documentation, status, and closure evidence;
- independent final-head reclosure after M030.

### 4.2 Out of scope

- arbitrary merge of edits made while disabled into an established control
  authority;
- bidirectional legacy/control synchronization;
- tombstones, provenance metadata, a new persistence schema, or a generic
  migration engine;
- new AddressBook precedence policy or resolver redesign;
- changes to `emissary-core/**`;
- missing tunnel data planes;
- new RouterInfo sources, telemetry, samplers, polling, NetDB inspection, or
  fabricated values;
- router, transport, streaming, LeaseSet, cryptographic, SAM, frontend, CI,
  release, packaging, fuzzing, soak, or generated-evidence work;
- upstream contribution, review, submission, adoption, or merge activity.

## 5. Production file budget

Primary production work should remain in:

- `emissary-cli/src/i2pcontrol/production.rs`;
- directly affected `emissary-cli/src/i2pcontrol/**` adapters/tests.

Changes outside the I2PControl crate are permitted only where the shared runtime
lookup owner must change:

- `emissary-cli/src/address_book.rs` for owner-aware Base64 lookup, bounded full
  destination loading/validation, one purpose-specific import/repair method, and
  focused tests;
- `emissary-cli/src/main.rs` only for one narrow activation input or call if
  required.

No core file is authorized. If correctness requires broader persistence or
resolver changes, M030 stops rather than expanding.

## 6. Target ownership model

### 6.1 Disabled/default mode

Retain M028 exactly:

- legacy `addresses` and `destinations/` files drive lookup;
- downloads update legacy files;
- control state is ignored and untouched;
- no mutation handle exists.

### 6.2 First enabled activation

When no control authority exists:

- load a bounded, path-confined snapshot from full legacy destination files;
- validate every hostname and destination;
- publish one complete control generation before starting the service;
- derive Base32 indexes from full destinations;
- never copy Base32 cache values into Proposal 170 destination fields.

### 6.3 Existing control authority

Current/backup control state remains authoritative.

Before service startup:

- valid full destinations are retained;
- a historical invalid/Base32-seeded published value may be repaired only from
  a matching validated legacy destination file;
- repair publishes one complete generation;
- unrepairable invalid state fails I2PControl activation without mutating the
  prior files.

Legacy entries absent from an established authority are not silently imported
on every re-enable. This preserves delete semantics and avoids a provenance or
tombstone architecture.

### 6.4 Active lookup and download

When the owner is active:

- Base32 lookup uses the owner's derived effective map;
- Base64 lookup uses the owner and does not fall through to stale legacy files;
- update/delete is immediately coherent across all views;
- active downloads merge validated full destinations and cannot preserve an
  incomplete Base32 seed.

## 7. Dependency sequence

```text
M020–M028 retained implementation/evidence
                    |
                    v
M029 closure invalidated
                    |
                    v
M030 AddressBook destination and owner coherence
                    |
                    v
future distinct final-head reclosure
```

M030 is the only implementation handoff for this corrective slice. Its
implementation and distinct final-head closure are recorded below; no successor
is dependency-ready within the current scope.

## 8. Milestones

### M020–M028 — Retained corrective evidence

Status: retained

Do not reopen unrelated method families without a newly demonstrated defect.

### M029 — In-scope conformance reclosure

Status: invalidated final disposition; evidence retained

Invalidation:

- `plans/closure/i2pcontrol-proposal-170/029-closure-invalidation.md`

Non-AddressBook evidence remains reusable. M029 is not controlling closure.

### M030 — AddressBook destination and owner coherence

Plan:

- `plans/implementation/i2pcontrol-proposal-170/030-addressbook-destination-owner-coherence.md`

Status: closed

Objective:

- establish full-destination control-state storage;
- make active owner lookup authoritative for Base32 and Base64;
- repair bounded historical Base32-seeded published entries;
- prevent stale legacy fallback after update/delete;
- retain M028 feature isolation and transition model;
- add regressions that would have rejected M029;
- freeze a corrected implementation/test head.

Exit conditions:

- administrative, RouterInfo, Base32, and Base64 views agree after add/update/
  delete;
- first activation imports full destinations;
- malformed historical entries repair or fail closed;
- active download state contains full destinations;
- disabled/default behavior remains unchanged;
- production diff stays within the explicit file budget;
- no high/medium M030 defect remains;
- implementation disposition and closure are committed.

M030 is closed by:

- `plans/closure/i2pcontrol-proposal-170/030-implementation-disposition.md`;
- `plans/closure/i2pcontrol-proposal-170/030-closure.md`.

The final disposition remains `partial Proposal 170 support`: the corrected
AddressBook dimension is closed, while unavailable RouterInfo sources and
missing tunnel data planes remain explicit and out of scope.

## 9. Cross-cutting invariants

1. Existing base I2PControl clients continue to work without modification.
2. Canonical Proposal 170 wire spelling and types remain exact.
3. Active AddressBook mutation affects the actual runtime lookup source.
4. Active lookup never bypasses the control owner through a stale legacy file.
5. Published Proposal 170 values are structurally valid full destinations.
6. Disabled/default execution remains independent of control state.
7. Existing control authority remains authoritative across re-enable.
8. Persistent mutation and repair are durable-before-success and failure-atomic.
9. Unsupported tunnel types remain resource-free and never report running.
10. RouterInfo unavailable data remains explicit and unfabricated.
11. No second authority, new schema, generic reconciler, core change, new
    dependency, CI expansion, or upstream interaction is introduced.

## 10. Failure, restart, cancellation, and contention policy

- Validation completes before import or repair mutation.
- Failed validation writes nothing.
- Failed persistence leaves prior current/backup state and live indexes.
- Unrepairable active state blocks I2PControl startup with a sanitized error.
- Disabled router startup remains unaffected because it does not read control
  state.
- Mutation retains the existing serialized owner lock and complete-generation
  publication.
- Readers observe old or new complete generations, never a partial repair.
- No network await or new long-running task is added under the owner lock.
- First activation imports once; enabled restart uses current/backup state;
  disabled restart ignores it; re-enable restores it without stale-file merge.

## 11. Compatibility and migration

- Existing router configuration and disabled legacy files remain valid.
- Existing valid control-state snapshots remain readable without schema change.
- Historical Base32-seeded published values receive only bounded,
  schema-preserving data repair from matching full destination files.
- Unrepairable values fail enabled activation but do not affect disabled mode.
- No automatic merge of arbitrary disabled-period edits is introduced.
- TunnelManager, ClientServicesInfo, RouterInfo, authentication, and JSON-RPC
  public compatibility remain unchanged.

## 12. Verification policy

M030 uses focused and CLI package-scoped verification only:

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

Use targeted formatting and `git diff --check`.

M030 does not authorize repair of unrelated `emissary-core` test debt. Remote
CI, platform matrices, release checks, coverage, fuzzing, soak testing, and
generated evidence are not required.

## 13. Milestone status

| Milestone | Status | Disposition |
|---|---|---|
| 001–019A | historical/superseded/invalidated as recorded | retained history only |
| 020–021 | retained closed evidence | base and TunnelManager corrections |
| 022 | retained except reopened coherence slice | enabled AddressBook authority |
| 023–028 | retained evidence | service, SAM, RouterInfo, conformance, feature isolation |
| 029 | invalidated final disposition | non-AddressBook evidence retained |
| 030 | ready | destination and owner-coherence corrective pass |

## 14. Completion definition

This workstream is not complete until:

- M030 corrects active destination/lookup coherence within the file budget;
- focused regressions cover stale-file update/delete and full-destination
  import/repair;
- M028 disabled/runtime-disabled isolation still passes;
- no unrelated method family or core subsystem changes;
- one independent final-head review accepts the corrected implementation;
- documentation and registry chronology are consistent;
- no high/medium correctness, security, compatibility, scope, or claim defect
  remains;
- no upstream interaction occurred.

After correction and independent review, the expected honest final label remains
`partial Proposal 170 support`, not unqualified full operational support.
