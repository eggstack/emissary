# M022 — AddressBook Runtime Bridge and Canonical Source Reconciliation

Status: implemented

Primary class: capability/ownership corrective pass

Hard dependencies:

- M020 closed for implementation
- M021 closed for implementation where shared persistence/security primitives are consumed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Prior defect record:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

## 1. Bounded objective

Make Proposal 170 AddressBook success correspond to the actual address-book source owned by the running Emissary router, without redesigning resolver precedence or broadening I2PControl into a generic filesystem/configuration manager.

The current administrative generation store may be retained as a durable representation only if it is reconciled with the real runtime owner through one explicit authority model. It must not remain a disconnected shadow that reports successful mutation while runtime lookup is unchanged.

This milestone also corrects the canonical RouterInfo subscription/config object shapes and provenance needed by the address-book domain. M025 will integrate those fields into the final 43-selector matrix.

## 2. Current defects

- Four administrative books are persisted separately from the runtime address book.
- Successful API mutation does not necessarily affect normal runtime destination resolution.
- Runtime-owned entries may not appear in API lists.
- Restart can preserve two contradictory sources with no defined synchronization rule.
- Subscription and configuration selectors are implemented as a bare array/object in one path but classified unavailable elsewhere; Proposal 170 requires objects containing source/path and entry information.
- `SetConfig` accepts path-like values as inert strings, but the current documentation can imply that the actual address-book configuration was modified.

## 3. Required ownership decision

Before production edits, inspect the current runtime `AddressBook` implementation, loader/downloader, handles, and startup composition. Record one of these bounded models in the implementation disposition:

### Preferred model A — Existing owner gains a narrow administrative handle

The runtime address-book owner exposes purpose-specific asynchronous operations:

- list/lookup one Proposal 170 book;
- add or replace one entry;
- delete one entry;
- set/read subscription metadata;
- set/read address-book configuration metadata;
- produce a coherent bounded snapshot.

Persistence remains owned by the runtime address-book subsystem. I2PControl holds only the narrow handle.

### Acceptable model B — Generation store is authoritative and runtime receives published snapshots

The current I2PControl generation store remains the durable owner, but every successful publication is synchronously applied to the runtime lookup source before success is returned, and startup loads the same generation into runtime before serving requests.

This model is acceptable only if it does not introduce a second precedence system, polling task, or eventual-consistency window.

### Prohibited model

Two separately durable authoritative stores with best-effort import/export, periodic synchronization, or successful API responses before runtime application.

If neither bounded model is feasible, the method must fail explicitly and the subsystem must remain partial; do not fabricate runtime management.

## 4. Required invariants

1. A successful canonical add/replace is observable through the managed runtime lookup path according to the documented book precedence.
2. A successful delete removes the managed entry from that same source.
3. API list/lookup and runtime lookup derive from one coherent authority or one synchronously published snapshot.
4. Four Proposal 170 book identities remain independent in administrative semantics.
5. This milestone does not change the router's established precedence order among private/local/router/published sources.
6. Startup loads one coherent state before I2PControl accepts requests.
7. Mutation failure leaves prior durable and runtime state unchanged, or returns a precise blocked disposition if the existing owner cannot provide atomicity.
8. Address-book paths are never derived directly from untrusted request values.
9. `SetConfig` modifies only explicitly supported address-book settings owned by this subsystem; arbitrary path values are rejected or stored as non-operative metadata with no false success claim.
10. Subscription/config RouterInfo objects use exact pinned keys/types and truthful path/provenance values.
11. Full destinations, subscription contents, config values, tokens, and filesystem roots are not logged.
12. No frontend state or single-owner event receiver is consumed.

## 5. Explicit non-goals

- no change to destination lookup protocol, base32/base64 semantics, naming resolution order, or downloader scheduling beyond what is required to apply the same authoritative state;
- no hosts-file parser rewrite;
- no network fetching of subscriptions from I2PControl handlers;
- no arbitrary file editor or path-selection API;
- no migration of unrelated router configuration;
- no missing tunnel work;
- no broad runtime address-book redesign, event bus, watcher, or polling loop;
- no new dependency, CI, release, frontend, or upstream work.

## 6. Expected file boundary

Primary I2PControl files:

- `emissary-cli/src/i2pcontrol/address_book.rs`
- `emissary-cli/src/i2pcontrol/control_plane.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/stores/address_book_store.rs` if retained
- `emissary-cli/src/i2pcontrol/server.rs`
- focused tests and docs.

Permitted external seam:

- the existing runtime address-book owner/handle implementation;
- `emissary-cli/src/main.rs` or the current composition root solely to pass the handle and initial source metadata.

No other router/core files are authorized without an explicit stop-and-record justification.

## 7. Required work packages

### WP1 — Runtime owner inspection and source map

Document:

- current source files/data structures for each runtime book;
- precedence and merge behavior;
- startup load order;
- mutation/persistence capabilities already available;
- downloader/subscription ownership;
- whether the existing handle can be safely extended without exposing unrelated authority.

Add this map to directly affected internal documentation; do not create a generic architecture document.

### WP2 — Purpose-specific control trait

Define the smallest trait/handle needed by Proposal 170. Prefer immutable snapshot DTOs and bounded mutation methods over exposing maps, file paths, or internal locks.

The trait must not expose:

- downloader control;
- arbitrary file writes;
- resolver replacement;
- task cancellation;
- global router state.

Production construction must require the real adapter. Test constructors may use a fake.

### WP3 — Coherent persistence/application transaction

For each mutation:

1. validate request and destination before acquiring mutation ownership;
2. construct the complete proposed book/config state;
3. validate collisions, bounds, and source rules;
4. persist using the established authoritative owner;
5. publish/apply the same committed state to runtime before returning success;
6. on any pre-commit failure, preserve previous durable/runtime state;
7. on post-durable runtime-apply failure, fail closed at startup or use the owner's atomic publication mechanism; do not acknowledge contradictory state.

If model A provides one owner, this should be naturally atomic. If model B is selected, use one serialized transaction and prove no request can observe the intermediate state.

### WP4 — Canonical AddressBook wire adjudication

Re-derive literal fixtures for:

- entry add/replace;
- entry delete selected by exact `Delete` presence semantics;
- `SetSubscriptions`;
- `SetConfig`;
- result envelope placement where proposal examples are inconsistent.

Record the adjudication and reference implementation evidence internally. Do not create an additional public alias to satisfy both examples.

Compatibility action-style CRUD and separate Set methods may remain isolated extensions.

### WP5 — Destination and hostname validation

Use existing Emissary destination primitives to decode and structurally validate submitted destinations before persistence/runtime publication.

Do not accept arbitrary non-empty strings as destinations.

Hostname validation should enforce:

- length and control-character bounds;
- canonical normalization policy already used by runtime lookup;
- no path separators or filesystem interpretation;
- deterministic collision behavior after normalization.

Do not introduce IDNA/domain policy unrelated to the existing router.

### WP6 — Subscription/config source objects

Implement exact Proposal 170 RouterInfo object shapes for:

- `i2p.router.addressbook.subscriptions`;
- `i2p.router.addressbook.config`.

The object must expose only the path/source metadata and entries required by the pinned contract. Paths must come from trusted startup configuration or the authoritative owner, not request-supplied arbitrary strings.

If Emissary has no actual path-backed source for a field, represent that limitation exactly as decided by M025; do not return a fake path.

### WP7 — Startup and migration

- Load legacy administrative generations only after validating whether they can be merged into the selected authority without overwriting runtime-owned entries.
- Define deterministic collision policy; default to fail closed with operator-visible recovery rather than silent precedence changes.
- Publish a new generation only after successful migration.
- Preserve rollback to the last known-good source.
- Do not auto-delete legacy files until closure evidence confirms migration.

## 8. Failure, cancellation, restart, and contention semantics

- All mutations serialize through one owner.
- Concurrent lookups observe before or after state, never a partial map.
- Failed destination parsing performs no persistence.
- Failed persistence performs no runtime mutation.
- Restart after success reconstructs the same runtime/API state.
- Restart after corrupt newest generation/source falls back according to existing owner semantics and reports the fallback.
- Cancellation before commit leaves prior state; after commit, the state remains committed even if the response is lost.
- Subscription fetching remains asynchronous owner behavior and is never triggered within a request deadline.

## 9. Compatibility and migration effects

- Existing runtime lookups should gain visibility of successful control-plane entries without changing precedence.
- Existing runtime/local hosts sources remain intact unless an exact collision policy says otherwise.
- Existing administrative-only generations require explicit one-time migration or archival; they cannot silently remain a second authority.
- Compatibility CRUD may continue but must route through the same real owner.
- Existing configurations without I2PControl remain unchanged.

## 10. Focused tests

Required tests:

1. Canonical add valid destination succeeds and runtime lookup resolves it.
2. Canonical replace changes both API lookup and runtime lookup coherently.
3. Canonical delete removes it from both views.
4. Invalid destination never reaches persistence or runtime.
5. Four books remain isolated and precedence remains unchanged.
6. Concurrent read during mutation observes complete before/after state.
7. Injected persistence failure preserves runtime and disk state.
8. Injected runtime publication failure cannot produce acknowledged contradictory state.
9. Restart reproduces state after successful mutation.
10. Legacy administrative generation migration/collision behavior is deterministic.
11. SetSubscriptions and SetConfig exact fixtures and bounds.
12. Subscription/config RouterInfo object keys/types match the pinned contract.
13. Untrusted path-like values cannot choose files or escape the owner directory.
14. No destination, subscription value, config value, or path root appears in logs/errors.
15. Compatibility actions use the same owner and remain separately counted.

## 11. Verification commands

Focused:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol addressbook
cargo test -p emissary-cli --no-default-features --features i2pcontrol production_composition
```

If the runtime address-book owner is in `emissary-core`, also run focused owner tests and:

```bash
cargo check -p emissary-core
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
```

Then:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

No live subscription network test, remote CI, filesystem matrix, or broad router integration farm is required.

## 12. Documentation and static guards

- Remove administrative-shadow success claims.
- Document the selected authority model, precedence preservation, and legacy migration.
- Add a production-composition guard preventing fake/separate AddressBook adapters in production.
- Add literal canonical AddressBook and RouterInfo subscription/config fixtures.
- Keep runtime/source/wire status separate.

## 13. Acceptance criteria

M022 is implementation-complete only when:

- one authoritative/coherently published address-book source is proven;
- successful API mutation is visible through normal runtime lookup;
- all failure/restart tests preserve coherent state;
- exact canonical entry/subscription/config fixtures pass;
- no arbitrary path authority or subscription fetching is introduced;
- external changes are limited to one narrow owner handle and composition wiring;
- package/core verification passes or exact unrelated blockers are recorded;
- no resolver precedence redesign, dependency, CI, missing tunnel, or upstream activity occurs;
- an implementation disposition records the selected model and residual source limitations.

## 14. Stop conditions

Stop if:

- the existing runtime owner cannot expose bounded mutation without broad redesign;
- two-source eventual consistency appears necessary;
- applying one book requires changing global resolver precedence;
- exact subscription/config source paths do not exist and would have to be fabricated;
- migration risks overwriting operator-managed entries without an explicit policy;
- work expands into downloader/network scheduling, frontend, CI, or upstream activity.

A blocked disposition is preferable to retaining a false successful shadow API.
