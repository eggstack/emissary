# M021 — TunnelManager Exact Wire, Atomic Persistence, and Secret Boundary

Status: ready

Primary class: capability/invariant corrective pass

Hard dependency:

- M020 closed for implementation

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Prior evidence and defects:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`

## 1. Bounded objective

Correct `TunnelManager` to the exact pinned Proposal 170 contract while preserving the explicit unsupported-backend boundary for missing tunnel data planes.

This milestone owns:

- all seven canonical lowercase actions;
- exact action-specific parameter validation;
- exact structured success and operation-failure envelopes;
- exact `get` information and nested `rawConfig` schema;
- deterministic unsupported-runtime statuses;
- atomic create/edit/rename/delete persistence;
- secret-safe option storage, logging, and response serialization;
- compatibility aliases already shipped by Emissary, kept separate from canonical behavior.

It does not implement a tunnel data plane or broaden lifecycle authority over startup-owned services. Startup inventory and any safe existing-backend adapters are owned by M023.

## 2. Current defects

The current canonical `get` response serializes `Name`, `Type`, `State`, and flattened legacy fields. Proposal 170 requires `result.status` and `result.info` with fields such as `client`, `status`, key flags, destination fields, and nested `rawConfig` containing canonical lower-case `name` and `type`.

Current edit/rename removes the old definition in one published generation and inserts the replacement in another. Failure between publications can permanently delete the original.

Canonical option validation covers only a subset of the defined fields and does not fully validate enum values such as `EncryptLeaseSet`. Unknown top-level keys may be retained as though they were protocol extensions.

Secret-bearing values may be duplicated in typed options and `raw_config`, transparently serialized to disk, and reflected by `get`. Unix restrictive-permission failure is ignored.

`All` handling and lifecycle status wording do not consistently follow exact action semantics.

## 3. Required invariants

1. Canonical actions are exactly `create`, `edit`, `get`, `start`, `stop`, `restart`, and `delete`.
2. `All` is accepted only for `start`, `stop`, and `restart`, according to exact parameter-presence and boolean rules established by the pinned contract.
3. Canonical operational failures remain in `result.status`; malformed requests use JSON-RPC errors.
4. `get` returns the exact pinned `result.info` keys and JSON types.
5. `rawConfig` uses exact canonical key spelling and contains no handler/internal metadata that is not part of the contract.
6. All twelve tunnel types parse, persist, retrieve, edit, and delete.
7. Missing data planes remain explicit unsupported backends; no stub binds, spawns, creates a destination, or reports running.
8. One logical mutation publishes at most one new durable generation.
9. Failed mutation leaves both the previous in-memory and previous durable state unchanged.
10. Name collision checks include control-plane definitions and, after M023, startup-managed names.
11. Secrets are not logged or included in unintended response fields.
12. Secret-bearing persistence is minimized to fields required for future runtime use and stored only under enforced restrictive conditions where supported.
13. Compatibility action aliases remain separate and do not weaken canonical parsing.
14. No handler writes `router.toml` or edits startup tunnel configuration.

## 4. Explicit non-goals

- no implementation of HTTP, IRC, SOCKS-IRC, CONNECT, Streamr, bidirectional, or other missing tunnel data planes;
- no generic tunnel supervisor, task registry, plugin API, or service framework;
- no lifecycle control of startup-managed tunnels in this milestone;
- no cryptographic key-generation redesign;
- no encrypted-at-rest framework or dependency addition unless an explicit maintainer decision is recorded separately;
- no broad configuration migration;
- no RouterInfo or ClientServicesInfo source work beyond shared serializers directly required by this method;
- no CI, release, frontend, or upstream work.

## 5. Expected file boundary

Primary production files:

- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
- `emissary-cli/src/i2pcontrol/domain/tunnel.rs`
- `emissary-cli/src/i2pcontrol/stores/tunnel_store.rs`
- `emissary-cli/src/i2pcontrol/stores/generation_store.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- existing backend trait/unsupported registry files only if exact status translation requires it.

Tests and documentation:

- focused TunnelManager/store/conformance tests;
- `docs/i2pcontrol/tunnel-manager.md`;
- Proposal 170 support/conformance documents.

Do not touch `emissary-core`, proxy startup, runtime tunnel managers, or `.github/**`.

## 6. Required production changes

### WP1 — Literal action contract inventory

Create one typed canonical inventory derived from the pinned proposal:

- required and optional parameters per action;
- accepted tunnel types;
- common and type-specific option names;
- JSON type/range/enum constraints;
- response shape per action;
- operation status wording pattern.

Use this inventory for validation/tests/documentation where practical, without building a generic schema framework.

Reject unknown top-level canonical keys except explicitly defined extensibility containers such as `CustomOptions` and `LeaseSetClientAuths`. Compatibility-only legacy keys must be parsed only on the compatibility path.

### WP2 — Exact `get` serializer

Implement a dedicated canonical `TunnelManager.get` serializer that emits:

- `client` boolean derived from exact tunnel type;
- `status` mapped from truthful runtime state;
- `persistentClientKey` boolean;
- `offlineKeys` boolean;
- `targetDestination` only from the correct canonical source;
- `localDestination`, `destination`, and `destinationB32` only when a truthful source exists;
- nested `rawConfig` with exact canonical `name`, `type`, and permitted configuration fields.

Do not substitute `Name`, `Type`, `State`, legacy `i2p.tunnel.*` keys, local target hosts, empty strings, or fabricated destinations for unavailable canonical values.

Where the pinned proposal permits omission or neutral values, apply that exact rule. Where it does not, return an explicit operation error rather than inventing data.

The compatibility `Get` response may retain its existing shape only under the compatibility action path.

### WP3 — Complete parameter validation

Validate:

- name/new-name non-empty, length, and control-character bounds;
- port and count ranges;
- all booleans as booleans;
- `EncryptLeaseSet` exact allowed values;
- signing/encryption types against the accepted textual or numeric domain defined by the pinned contract/reference implementation;
- list/map containers and element types;
- action-specific prohibited parameters;
- `All` semantics and conflict with `Name`;
- create-required `Type` and edit preservation rules.

Do not silently ignore malformed known fields.

### WP4 — Atomic store mutation

Add one store operation for update/rename:

1. acquire the existing store mutation lock;
2. clone the current complete map;
3. validate existence, ownership, and target-name collision;
4. apply all field changes to the clone;
5. publish one complete generation;
6. update in-memory state only after successful publication.

Create and delete should follow the same one-publication rule.

Add a test-only failpoint at the generation publication boundary or a deterministic failing store abstraction. Do not add production fault-injection machinery.

### WP5 — Secret boundary

Classify every option as:

- public configuration safe to round-trip;
- sensitive configuration required for a future backend;
- runtime-generated secret/key material that must never be accepted or returned through generic raw configuration;
- unsupported/unknown and rejected.

Requirements:

- sensitive fields appear once in the internal model, not duplicated in `rawConfig`;
- debug/display/tracing are redacted;
- canonical `get` follows the exact pinned contract but does not expose fields the protocol does not explicitly require;
- compatibility responses do not reveal more than canonical responses;
- persisted files containing sensitive values require restrictive permissions on Unix; inability to establish those permissions is a mutation failure, not best effort;
- temporary files are removed on failed publication where feasible;
- errors contain field names at most, never values.

Do not introduce encryption-at-rest as an incidental subproject.

### WP6 — Status translation and unsupported backends

Map backend outcomes at the handler boundary to exact status text tied to the requested tunnel name/action.

Unsupported behavior remains:

- create/edit/get/delete: durable administrative definition operations;
- start/restart: deterministic `error - ... not implemented` operation status;
- stop: safe idempotent inactive outcome without targeting unrelated tasks;
- get/inspection: never reports running.

Do not allow backend-specific internal strings such as `ok - client started` to leak as canonical status wording.

### WP7 — Compatibility isolation

- Retain capitalized actions and `List` only as documented Emissary extensions.
- Keep compatibility parameter aliases out of canonical validation.
- Reject requests that mix canonical action casing/options with compatibility-only top-level forms when ambiguous.
- Ensure compatibility fixtures do not count toward canonical Proposal 170 coverage.

## 7. Failure, cancellation, restart, and contention semantics

- All mutations serialize under the existing store owner.
- Duplicate creates and rename collisions fail before publication.
- Publication failure preserves the previous generation and previous in-memory map.
- Cancellation before completed publication returns no success and leaves prior state.
- Cancellation after successful publication may return an error only if the request deadline fires after durability; the closure record must document this unavoidable response/durability edge rather than attempt rollback.
- Restart loads the newest valid complete generation and falls back only to a prior valid generation on corruption.
- Unsupported start/restart perform no persistence or runtime resource acquisition.
- Concurrent get sees either the complete before-state or complete after-state.

## 8. Compatibility and migration

- Existing generation schema should be migrated in place only if the current serialized model can be safely interpreted.
- If removing duplicate secret fields changes schema, add a versioned loader migration that preserves needed values and publishes the new shape only after complete validation.
- Never silently discard a persisted secret needed by a future backend; reject startup with a precise recovery instruction if safe migration is impossible.
- Compatibility action responses remain stable unless they currently disclose secrets; security correction takes precedence and must be documented.
- Startup-managed entries remain outside this store until M023 imports read-only inventory.

## 9. Focused tests

Required canonical fixtures:

1. All twelve types create with exact success envelope.
2. Duplicate create returns structured operation error.
3. Edit preserves omitted fields and emits exact status.
4. Rename succeeds with one generation increment.
5. Injected publication failure during rename preserves the original name and definition after in-memory check and restart.
6. Get emits exact `info` keys/types and nested lower-case `rawConfig.name/type`.
7. Get does not emit `Name`, `Type`, `State`, legacy flattened aliases, or invented destination values on the canonical path.
8. Missing get/edit/start/stop/restart uses structured operation failure.
9. Delete absent/present behavior follows exact contract.
10. `All` valid/invalid action and conflict matrix.
11. Every defined range and enum boundary has valid-min, valid-max, below/above, and wrong-type tests.
12. Unknown canonical top-level key is rejected; allowed extensibility containers round-trip.
13. Unsupported start/restart bind no sockets and spawn no tasks; stop is idempotent.
14. Secret values do not occur in JSON responses, debug output, logs, errors, or duplicate serialized fields except where the pinned contract explicitly requires a protected authenticated return.
15. Permission-setting failure causes mutation failure on Unix test support.
16. Corrupt newest generation falls back to prior valid complete generation.
17. Compatibility action fixtures remain isolated.

## 10. Verification commands

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_store
cargo test -p emissary-cli --no-default-features --features i2pcontrol generation_store
cargo test -p emissary-cli --no-default-features --features i2pcontrol secret
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

No remote CI, network tunnel tests, platform matrix, fuzz campaign, or missing data-plane test is required.

## 11. Documentation and static guards

- Replace canonical examples with literal pinned request/response fixtures.
- Document every tunnel type separately across wire/CRUD/runtime dimensions.
- Add a static assertion that the canonical action inventory is exactly seven and type inventory exactly twelve.
- Add a static fixture that canonical `get` keys equal the pinned set.
- Document secret handling and the fact that authenticated control still does not imply arbitrary key disclosure.
- Keep subsystem status `corrective pass required`.

## 12. Acceptance criteria

M021 is implementation-complete only when:

- canonical action and option inventories match the pinned proposal;
- canonical `get` matches the exact `info/rawConfig` schema;
- every logical mutation is one failure-atomic publication;
- rename failure injection proves no data loss before and after restart;
- secret boundary tests pass;
- unsupported backends remain resource-inert;
- compatibility behavior is separated and documented;
- package-scoped check/test/clippy pass or exact unrelated blockers are recorded;
- no runtime tunnel implementation, broad refactor, new dependency, CI, or upstream action occurs;
- an implementation disposition records the frozen head and residual findings.

## 13. Stop conditions

Stop if:

- exact canonical output requires a destination/key source that does not yet exist; record the field as blocked for M023 rather than fabricate it;
- safe persistence migration cannot be performed without operator action;
- proposed lifecycle work reaches startup-managed tasks or missing data planes;
- secret requirements appear to demand an encryption-at-rest project;
- implementation expands beyond the listed file boundary without a named unavoidable seam;
- any upstream write/review/submission action is proposed.

Successful M021 closure unblocks M022 and M023. It does not authorize real missing tunnel backends.
