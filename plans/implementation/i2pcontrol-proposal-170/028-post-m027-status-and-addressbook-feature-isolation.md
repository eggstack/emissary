# M028 — Post-M027 Status and AddressBook Feature-Isolation Corrective Pass

Status: ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Source invalidation:

- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`

Applicable decisions and governance:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`

Repository baseline:

- `03a384aec495232e64468dcf61d60dd2bab5cfe0`

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`
- status: `Open`
- created and last updated: `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

## 1. Bounded objective

Restore the post-M027 planning/status authority and make the M022 runtime
AddressBook bridge strictly conditional on I2PControl compile-time and runtime
enablement.

M028 must preserve the enabled-mode Proposal 170 behavior already established by
M022 while proving that disabled I2PControl does not alter ordinary AddressBook
loading, lookup, download, persistence, dependency footprint, or restart
semantics.

This is one coherent corrective slice because both defects concern whether a
feature-gated administrative API remains confined to its declared ownership and
support boundary.

## 2. Readiness and current evidence

M028 is dependency-ready.

Retained implementation evidence:

- M020 restored base authentication, token, JSON-RPC, and direct RouterInfo
  interoperability.
- M021 corrected TunnelManager wire shape, validation, atomic publication, and
  secret boundaries.
- M022 connected Proposal 170 AddressBook operations to a runtime owner.
- M023 added truthful startup tunnel inventory and service lifecycle sources.
- M024 corrected bounded SAM recovery.
- M025/M026 froze the 16 available / 1 neutral / 26 unavailable RouterInfo
  source matrix.
- M027 added literal external-contract fixtures and selected the truthful
  `partial Proposal 170 support` disposition.

Current defects at the baseline:

1. `03a384a` revived M019 as the controlling closure and changed current status
   documents from M027's partial-support disposition to a broader closed claim.
2. `AddressBookManager::new` always constructs `RuntimeAddressBookOwner`.
3. construction always reads `control-state.json` and rebuilds ordinary runtime
   indexes from it.
4. normal downloader persistence always calls `owner.merge_downloaded(...)`,
   which can create/update Proposal 170 control-state files without an active
   I2PControl service.
5. `AddressBookHandle` always carries the control-state owner.
6. `serde_json` is unconditional in `emissary-cli/Cargo.toml` after M022.

These facts are sufficient to plan the correction. M028 must inspect current
code before editing and must not assume the exact internal type split described
below if a smaller equivalent arrangement is available.

## 3. Required invariants

1. A build without the `i2pcontrol` feature has no Proposal 170 control-state
   owner, control-state file access, migration, or downloader publication.
2. A build with `i2pcontrol` compiled but runtime configuration disabled behaves
   like the non-I2PControl build for AddressBook loading, lookup, and
   persistence.
3. Enabling I2PControl activates exactly one Proposal 170 AddressBook authority
   that shares the live lookup maps and preserves M022's four-book precedence.
4. Disabling I2PControl after prior enabled use does not delete or rewrite
   control-state files; it simply excludes them from runtime authority until the
   service is re-enabled.
5. Re-enabling I2PControl loads the retained control state and resumes the M022
   precedence/migration rules without duplicating entries.
6. The ordinary `addresses`, `destinations/`, subscription download, and
   `host_modified_times` behavior remains unchanged when the control plane is
   inactive.
7. Runtime enablement cannot create two independently authoritative AddressBook
   stores.
8. Proposal 170 mutations remain durable before success and failure-atomic.
9. No arbitrary input controls paths; existing path confinement remains.
10. `serde_json` is optional and owned by `i2pcontrol` unless a separately
    identified unconditional CLI consumer requires it.
11. The current 16/1/26 RouterInfo source matrix is unchanged by M028.
12. Missing tunnel data planes remain explicit unsupported stubs.
13. No upstream write, review request, submission, adoption request, or merge
    activity occurs.

## 4. Explicit non-goals

M028 must not:

- implement HTTP, IRC, SOCKS-IRC, CONNECT, Streamr, bidirectional, or any other
  missing tunnel data plane;
- add real lifecycle control to startup-managed generic tunnel tasks;
- add or reclassify RouterInfo sources;
- create historical telemetry, polling loops, event buses, task registries,
  schema frameworks, or generic control interfaces;
- change resolver precedence or address-book download policy;
- change SAM observation behavior without a newly demonstrated correctness
  defect;
- redesign router, transport, NetDB, peer selection, streaming, LeaseSet,
  cryptographic, frontend, or configuration architecture;
- add dependencies, CI, release workflows, coverage gates, fuzzing, soak tests,
  or generated evidence bundles;
- perform repository-wide formatting;
- rewrite M020–M027 implementation history;
- claim unqualified full Proposal 170 operational support.

## 5. Authorized production file boundary

Primary files:

- `emissary-cli/src/address_book.rs`
- `emissary-cli/src/main.rs`
- `emissary-cli/src/lib.rs`
- `emissary-cli/Cargo.toml`

Directly affected I2PControl adapter files only if required by the final type
split:

- `emissary-cli/src/i2pcontrol/address_book.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/control_plane.rs`
- focused tests under `emissary-cli/tests/**`

Documentation/planning files:

- `docs/i2pcontrol/README.md`
- `docs/i2pcontrol/address-book.md`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/proposal-170-conformance.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- M028 disposition and closure records

Any edit outside this boundary is a stop condition unless the implementation
disposition identifies a concrete compilation dependency and demonstrates that
no narrower change works.

## 6. Target composition

The implementation must preserve a simple two-mode model.

### 6.1 Legacy/default mode

Active when either:

- the binary is compiled without `i2pcontrol`; or
- the feature is compiled but `[i2pcontrol].enabled` is false.

Properties:

- `AddressBookManager` loads the existing `addressbook/addresses` source;
- downloads update the existing addresses/destinations files;
- no `control-state.json` or backup is read, written, migrated, or consulted;
- normal lookup cannot be influenced by stale Proposal 170 control state;
- no Proposal 170 mutation handle exists;
- behavior matches the pre-M022 path except for unrelated retained fixes.

### 6.2 I2PControl-enabled mode

Active only when the feature is compiled and runtime configuration enables the
service.

Properties:

- one purpose-specific control-state owner is constructed;
- it wraps or publishes to the same live lookup maps used by the router;
- it may load retained control-state files and perform the already-documented
  legacy administrative migration;
- downloaded published entries merge through that owner;
- I2PControl receives a dedicated mutation/inspection handle;
- the general runtime AddressBook handle exposes only ordinary lookup behavior
  unless the existing trait already requires more;
- no second owner is constructed by I2PControl itself.

A small enum/optional handle at the composition root is acceptable. A generic
plugin system, dynamic owner registry, global singleton, or second event loop is
not.

## 7. Ordered work packages

### WP1 — Freeze and restore planning authority

- Record `027-closure-invalidation.md` as the current invalidation.
- Restore M027 as the latest retained closure evidence, not the active final
  disposition.
- Mark M019 and `019-closure.md` superseded historical evidence.
- Set current subsystem status to `corrective pass required`.
- Register M028 as ready and M029 as blocked on M028.
- Preserve the internal-only/no-upstream rule.

WP1 must not claim that documentation repair alone closes the feature-isolation
defect.

### WP2 — Separate ordinary AddressBook ownership from Proposal 170 control state

- Identify the smallest type boundary that lets ordinary lookup/download code
  operate without `RuntimeAddressBookOwner`.
- Compile Proposal 170 snapshot, four-book, control-state persistence, migration,
  and mutation code only with `cfg(feature = "i2pcontrol")` where practical.
- Remove the unconditional owner field from ordinary runtime objects, or make it
  absent and unreachable in disabled mode.
- Keep existing ordinary `AddressBook` trait behavior stable.
- Avoid copying full state between two owners.

The preferred shape is a dedicated I2PControl control handle, not adding more
conditional methods to the ordinary lookup handle.

### WP3 — Gate activation on runtime configuration

- Resolve runtime enablement in the composition root before constructing the
  Proposal 170 owner.
- Pass explicit activation state or an optional dedicated handle into
  `AddressBookManager`/I2PControl composition.
- Ensure feature-compiled but runtime-disabled startup never touches control
  state.
- Ensure I2PControl startup fails closed if enabled but the control owner cannot
  initialize safely.
- Do not silently fall back to a shadow administrative store.

### WP4 — Preserve enabled-mode M022 semantics

- Retain four independent private/local/router/published identities.
- Retain deterministic collision handling and documented precedence.
- Retain atomic current/backup publication and sanitized failures.
- Retain one-time migration only when no runtime control authority exists.
- Retain subscription/config metadata behavior.
- Retain immediate visibility of successful mutations in normal lookup.
- Retain download merge behavior only while the control owner is active.

### WP5 — Restore optional dependency ownership

- Determine every direct `serde_json` use in `emissary-cli`.
- If all unconditional uses were introduced for Proposal 170, make
  `serde_json` optional again and include it in the `i2pcontrol` feature.
- If an unrelated unconditional consumer now exists, record it and do not force
  an artificial dependency move; the acceptance criterion then becomes that
  Proposal 170 code itself is fully feature-gated.
- Do not change versions or add replacement serialization dependencies.

### WP6 — Add focused negative and transition regressions

Required tests must cover the cases in Section 9. Prefer focused unit and
composition tests using temporary directories. Do not add a new harness.

### WP7 — Reconcile directly affected documentation

- Restore `partial Proposal 170 support` as the expected bounded end state.
- While M028 is open, use `corrective pass required`.
- Explain that disabled mode ignores retained control state without deleting it.
- Preserve 16/1/26 RouterInfo counts and missing-data-plane exclusions.
- Remove M019-as-current language.
- Create the M028 implementation disposition and freeze the implementation/test
  head before M029 becomes ready.

## 8. Failure, cancellation, restart, and contention semantics

### Initialization failure

- Legacy/default mode must not fail because Proposal 170 control state is
  corrupt; it does not read that state.
- Enabled mode must use current/backup recovery as retained from M022.
- If both control-state generations are unusable, I2PControl activation fails
  explicitly and must not replace normal lookup with an empty or partial state.

### Mutation failure

- Enabled-mode add/update/delete/subscription/config failure leaves the previous
  durable state and live lookup state unchanged.
- A failed runtime publication must not report success.
- Disabled mode exposes no Proposal 170 mutation path.

### Downloader failure

- Legacy/default mode preserves existing warning/retry and legacy persistence
  behavior.
- Enabled mode preserves existing download behavior and only commits a complete
  merged control generation when publication succeeds.
- Download failure must not erase prior control or legacy state.

### Disable/re-enable transition

- Disabling I2PControl does not delete, migrate, rewrite, or merge retained
  control-state files.
- While disabled, runtime lookup is derived only from legacy/default sources.
- Re-enabling loads retained control state and re-establishes the M022 owner.
- Transition behavior is restart-based; M028 does not add live hot-toggle
  machinery unless such machinery already exists and requires no new owner
  design.

### Cancellation and contention

- No lock is held across network download awaits.
- Control mutations remain serialized by the existing owner lock.
- Ordinary lookup remains lock-bounded and synchronous as before.
- Cancellation before durable publication leaves prior state.
- Cancellation after publication may cause a lost response; retry observes the
  committed generation, as already documented.

## 9. Required tests

### 9.1 Compile/feature boundary

- `emissary-cli` builds with `--no-default-features` and without `i2pcontrol`.
- No Proposal 170-only type or dependency leaks into that build.
- The `i2pcontrol` feature build still compiles all control adapters.
- A static guard or manifest assertion verifies intended optional dependency
  ownership without introducing a generated dependency audit.

### 9.2 No-feature AddressBook behavior

Using a temporary base path:

- initialize the manager without `i2pcontrol`;
- place a valid stale `control-state.json` containing an entry absent from
  legacy addresses;
- prove lookup does not expose that entry;
- run/save a normal address-book update;
- prove no control-state current, backup, or temporary file is created or
  modified by that execution;
- prove legacy addresses/destinations behavior remains functional.

### 9.3 Feature compiled, runtime disabled

- compile with `i2pcontrol`;
- compose with `[i2pcontrol].enabled = false`;
- prove the same no-read/no-write/no-influence behavior as Section 9.2;
- prove no I2PControl AddressBook mutation handle is supplied to a running
  server because the server is not started.

### 9.4 Feature and runtime enabled

- load valid current control state;
- prove private/local/router/published precedence is retained;
- prove successful mutation is immediately visible to normal lookup;
- prove download merge updates the active owner;
- prove restart reloads the same accepted generation;
- prove current corruption falls back to backup;
- prove current+backup corruption fails enabled activation explicitly.

### 9.5 Transition behavior

- enabled run writes control state;
- subsequent disabled run ignores but does not delete or modify it;
- subsequent re-enabled run restores it;
- legacy state modified while disabled is merged according to the documented
  enabled-mode policy without duplicate or silent precedence inversion.

### 9.6 Retained Proposal 170 regressions

Run focused existing tests for:

- canonical AddressBook operations and source objects;
- RouterInfo address-book selectors;
- production composition;
- literal M027 fixtures touching AddressBook;
- secret/path sanitization;
- no missing tunnel data-plane resource allocation.

M028 must not duplicate the entire M027 literal suite with a new fixture layer.

## 10. Verification commands

Run focused tests first, using actual test names introduced by the implementation.

Required broad local commands:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features address_book
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol m027_literal_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Use touched-file configured formatting checks. Run `emissary-core` commands only
if implementation unexpectedly touches core; such a touch requires explicit
justification in the disposition.

Remote CI, release checks, coverage, fuzzing, platform matrices, network farms,
and generated evidence are not required.

## 11. Compatibility and migration

- Existing router configurations remain valid.
- Existing legacy address files remain authoritative in disabled mode.
- Existing M022 control-state files remain readable in enabled mode.
- M028 must not silently rewrite the control-state schema.
- If the type split requires a schema change, stop and create a separate
  migration plan; do not combine it into M028.
- Disabling I2PControl intentionally removes control-only entries from active
  lookup for that run while preserving files for re-enable. Document this
  clearly.
- No compatibility alias or base I2PControl behavior changes.

## 12. Security review requirements

The implementation disposition must verify:

- disabled mode cannot be influenced by stale or attacker-planted control-state
  files;
- enabled mode retains path confinement, size bounds, atomic publication, and
  sanitized errors;
- no private destination, credential, key, or full raw control state is logged;
- no symlink/path behavior is weakened;
- no new mutation authority is exposed through the ordinary AddressBook trait;
- no unsupported tunnel backend gains resources or lifecycle authority;
- no upstream interaction occurred.

## 13. Documentation and static guards

Required documentation updates:

- top-level I2PControl status;
- Proposal 170 support status;
- AddressBook enabled/disabled ownership behavior;
- roadmap/registry/implementation README chronology;
- M028 implementation disposition.

Required guards:

- one negative disabled-mode behavior test;
- one runtime-disabled-with-feature composition test;
- one enable/disable/re-enable persistence test;
- one manifest/dependency guard if `serde_json` becomes optional again;
- existing 16/1/26 source-count guard remains unchanged.

Do not add a repository-wide policy scanner.

## 14. Acceptance criteria

M028 is implementation-complete only when all of the following are true:

1. current status documents no longer identify M019 as controlling closure;
2. M027 is retained historical evidence and its final disposition is explicitly
   invalidated pending M029;
3. M028 is frozen in an implementation disposition with exact commits/files;
4. non-I2PControl builds do not compile, construct, read, write, migrate, or
   consult Proposal 170 AddressBook control state;
5. feature-compiled but runtime-disabled composition has the same property;
6. disabled mode preserves ordinary legacy AddressBook behavior;
7. enabled mode preserves M022's single-authority behavior;
8. disabling preserves but ignores control-state files, and re-enabling restores
   them without duplicate authority;
9. dependency ownership is restored or explicitly justified;
10. focused and broad commands pass;
11. no M020–M027 method-level regression is introduced;
12. 16/1/26 RouterInfo classification is unchanged;
13. missing tunnel data planes remain unsupported and resource-free;
14. no prohibited external or upstream action occurred;
15. M029 is moved from blocked to ready only after the implementation/test head
    is frozen.

## 15. Stop conditions

Stop and record a blocked/corrective disposition if:

- isolating the owner requires a control-state schema migration;
- disabled behavior cannot be restored without redesigning the resolver or
  downloader;
- enabled behavior requires two authoritative stores;
- a core/router/SAM change appears necessary;
- implementation would add a new dependency or generic framework;
- any canonical Proposal 170 wire behavior must change;
- any unavailable RouterInfo selector would need fabricated data;
- a missing tunnel data plane or new lifecycle supervisor becomes necessary;
- an upstream write, review request, or submission is proposed;
- required local commands fail for changes in M028 scope.

## 16. Closure evidence required

Create:

- `plans/closure/i2pcontrol-proposal-170/028-implementation-disposition.md`
- `plans/closure/i2pcontrol-proposal-170/028-closure.md`

The records must include:

- implementation commit(s) and frozen head;
- exact changed files;
- requirement-to-evidence matrix;
- no-feature, runtime-disabled, enabled, and transition test outcomes;
- failure/restart/contention review;
- compatibility/migration review;
- security and dependency review;
- retained 16/1/26 and unsupported-runtime attestations;
- unresolved findings with severity;
- internal-only/no-upstream attestation;
- disposition: closed, corrective pass required, or blocked.

M028 closure does not itself restore subsystem final status. M029 performs the
independent final-head reclosure.
