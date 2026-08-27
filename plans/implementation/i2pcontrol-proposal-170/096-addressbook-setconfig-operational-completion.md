# M096 — AddressBook SetConfig Operational Completion

Status: ready; dependency M095 closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`;
- retained AddressBook authority from M022/M030/M034 and current support documentation.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` plus M095 closure/matrix when dependency-ready.

Pinned external contract: I2P Proposal 170 revision `2026-05-20`.

Classification: capability / security / persistence.

## 1. Objective

Replace the current truthful `SetConfig` rejection boundary with operational semantics for all thirteen pinned Proposal 170 AddressBook configuration keys while keeping the implementation under `emissary-cli/src/i2pcontrol/**` and preventing authenticated remote administration from becoming arbitrary filesystem authority.

Current code inventories all thirteen keys but rejects every non-empty configuration request as either path-oriented or unsupported. That is safe but prevents a strict full-support claim.

M096 implements only the AddressBook configuration surface. CRUD, subscription replacement, runtime resolver precedence, TunnelManager, RouterInfo, and frontend work remain separate.

## 2. Pinned key inventory

M096 must implement explicit semantics for exactly:

Path/file-oriented:

- `subscriptions`;
- `published_addressbook`;
- `router_addressbook`;
- `local_addressbook`;
- `private_addressbook`;
- `etags`;
- `last_modified`;
- `log`.

Behavior/configuration:

- `update_delay`;
- `proxy_port`;
- `proxy_host`;
- `should_publish`;
- `theme`.

No additional SetConfig key is introduced unless M095 proves it belongs to the pinned proposal.

## 3. Required semantics

### 3.1 One durable configuration authority

Extend the existing Proposal 170 administrative state rather than create an unrelated second configuration database.

The durable configuration must be versioned, deterministic, atomically replaced, validated before publication, and reconstructible on restart.

A successful SetConfig response means the accepted configuration generation is durable and the active AddressBook runtime has either adopted the generation or reached the documented publication point. Do not return success for accepted-but-inert behaviorally meaningful values.

### 3.2 Administrative path root

Path-valued keys must resolve relative to one explicit I2PControl/AddressBook administrative root owned by the existing feature state directory.

Requirements:

- normalize `.` and `..` segments;
- reject resolution outside the authorized root;
- reject NUL/control bytes and platform-invalid forms;
- reject symlink escape at open/replace boundaries where practical;
- reject device/special files when a regular file is required;
- use same-filesystem atomic replacement for files M096 owns;
- never accept an arbitrary absolute host path merely because the caller is authenticated;
- do not log full sensitive path values on failure.

Relative examples containing `..` may succeed only if normalized resolution remains inside the configured root.

### 3.3 Address-book file ownership

The four address-book path keys select the durable administrative files used by the Proposal 170 AddressBook authority. Switching a path must be transactional:

1. validate/resolve the new path;
2. read/parse any existing target content within bounds;
3. prepare a complete new generation;
4. atomically update configuration;
5. publish the new administrative generation;
6. retain the prior known-good generation on pre-commit failure.

Do not change unrelated Emissary runtime resolver precedence unless the accepted AddressBook architecture already maps the administrative generation into that path. M096 is configuration of Proposal 170's AddressBook subsystem, not authority to redesign NetDB/name resolution.

### 3.4 Subscription metadata files

`subscriptions`, `etags`, and `last_modified` must map to the existing bounded subscription downloader/runtime state. Path changes must preserve complete-generation semantics and cannot silently discard the current subscription set on read/parse failure.

### 3.5 Update delay

`update_delay` must control the active subscription refresh cadence according to the pinned/reference unit semantics established by M095.

Enforce sane finite lower/upper bounds. A value may not create sub-second busy loops or effectively unbounded timer arithmetic. Changing cadence must cancel/replace only the owning refresh timer generation without duplicating background workers.

### 3.6 Proxy host/port

`proxy_host` and `proxy_port` configure the AddressBook subscription downloader's outbound proxy path if enabled by the pinned semantics.

Requirements:

- validate host/port before publication;
- do not perform unsafe request-driven local DNS where the accepted downloader design can use a literal/configured endpoint;
- do not turn the downloader into a general-purpose exposed proxy;
- changing proxy configuration affects future fetches without duplicating workers;
- failures preserve the prior durable configuration until commit semantics are satisfied.

### 3.7 Publication

`should_publish` controls whether the active administrative router/local state updates the configured published address-book artifact according to the adopted Proposal 170/SusiDNS semantics.

Publication must remain bounded and atomic. Disabled publication must not delete or corrupt the last published file unless the pinned semantics explicitly require that behavior.

### 3.8 Log path

`log` configures the AddressBook subsystem's own bounded administrative log artifact only if required by the pinned semantics. It must not reconfigure or redirect the global Emissary logger.

If the adopted semantics merely persist the configured path for AddressBook-owned diagnostics, M096 must still prove the value is consumed by the AddressBook owner rather than accepted inertly.

### 3.9 Theme

`theme` is administrative/frontend metadata in the reference configuration. Emissary has no Proposal 170 frontend in scope.

M096 may persist and round-trip the validated theme string without any router/frontend side effect, because the value is non-behavioral metadata. It must be explicitly classified as such in the M095 matrix and support docs; it must not create frontend coupling.

## 4. Readiness and current evidence

M096 is not executable until M095 closes and supplies:

- exact key types/units/reference semantics;
- existing runtime/persistence owner mapping;
- path budget;
- whether any current administrative getter already exposes configuration paths and therefore needs synchronized response changes.

No production work should begin from this prewritten plan if M095 materially changes those assumptions. Update the plan/roadmap first.

## 5. Preferred authorized path boundary

Target production changes:

- `emissary-cli/src/i2pcontrol/address_book.rs`;
- `emissary-cli/src/i2pcontrol/address_book_runtime.rs`;
- `emissary-cli/src/i2pcontrol/domain/address_book.rs`;
- `emissary-cli/src/i2pcontrol/production.rs` only if composition/state construction must pass the durable configuration authority;
- existing I2PControl persistence/state modules under `emissary-cli/src/i2pcontrol/**` identified by M095;
- focused I2PControl tests and support documentation.

No `emissary-core/**`, startup proxy/tunnel module, root manifest, lockfile, frontend, workflow, or unrelated CLI path is authorized by default.

If implementation proves a non-I2PControl production path is required, stop and amend containment before editing it.

## 6. Invariants

1. Authentication remains required before configuration mutation.
2. All 13 keys have explicit semantics; no unknown/inert behaviorally meaningful key.
3. Paths are confined to the administrative root.
4. No configuration value grants arbitrary filesystem traversal or global logger control.
5. Subscription/address-book generations are atomic and restart-safe.
6. Runtime resolver precedence is not redesigned incidentally.
7. Configuration changes do not create duplicate refresh/publish workers.
8. Secrets/tokens/full destination values do not leak through logs/errors.
9. Default/feature-disabled Emissary does not construct the configuration runtime.
10. No upstream interaction occurs.

## 7. Explicit non-goals

M096 MUST NOT:

- alter RouterInfo source counts except synchronized AddressBook config/subscription selector output required by SetConfig;
- implement router news/transit/network error/banned peers;
- alter TunnelManager or tunnel backends;
- add frontend theme rendering;
- change global Emissary logging configuration;
- make administrative books a new router-resolution authority beyond accepted architecture;
- add a general filesystem browser/API;
- add hosted CI/fuzz/release machinery;
- contact or submit to upstream.

## 8. Ordered work packages

### A. Model/version the configuration

Define a typed configuration object with exact thirteen-key coverage and versioned durable representation. Keep raw external strings only where the pinned contract requires strings; convert to validated internal types before publication.

### B. Implement confined path resolver

Create one reusable I2PControl-local resolver for AddressBook file keys. Test traversal, absolute paths, symlink escape where supported, special files, platform separators, and valid nested relative paths.

### C. Integrate path-backed administrative files

Wire the configured AddressBook/subscription/metadata files to the existing durable/runtime authority with transactional generation replacement.

### D. Integrate cadence/proxy/publication behavior

Apply `update_delay`, proxy host/port, and `should_publish` to the active worker generation. Ensure cancellation/replacement is exact and bounded.

### E. Integrate log/theme semantics

Keep AddressBook logging isolated from global logging. Persist/round-trip theme as explicit non-runtime metadata.

### F. Update getters/support evidence

Ensure RouterInfo AddressBook config/subscription selectors and direct AddressBook responses reflect the active generation/path values without leaking host absolute paths beyond the contract's intended representation. If the contract expects a path, return the configured administrative path form, not an incidental internal canonical host path unless M095 establishes otherwise.

## 9. Failure, cancellation, restart, and contention semantics

- Parse/validation failure: no state change.
- Pre-commit file/read/runtime validation failure: retain prior generation and return deterministic error.
- Post-durable-publication worker refresh failure: durable configuration remains accepted; expose operational failure through bounded diagnostics rather than pretending the mutation did not occur.
- Worker cadence/proxy replacement: cancel the exact prior worker generation and start at most one successor.
- Concurrent SetConfig: serialize through the existing administrative mutation authority; later committed generation wins deterministically.
- Restart: validate durable config before activation; invalid/corrupt generation falls back through the repository's accepted recovery strategy without deleting prior good data.

## 10. Compatibility and migration

There should be no migration from accepted-but-inert SetConfig because current code rejects non-empty requests. The durable schema may still need a version increment when configuration is first persisted operationally.

Existing CRUD/subscriptions state must remain readable. Existing default file locations remain the default when no SetConfig generation exists.

Public key spelling and response shapes remain pinned Proposal 170 behavior.

## 11. Security tests

At minimum cover:

- traversal/absolute/symlink escape rejection;
- special-file rejection;
- oversized path/value/count bounds;
- transactional failure preserving prior generation;
- proxy host/port validation;
- update-delay lower/upper bounds;
- concurrent mutation determinism;
- no duplicate worker after repeated config changes;
- restart persistence;
- should_publish enable/disable behavior;
- log path cannot redirect global logging;
- theme has no router/frontend side effect;
- sanitized errors/logging.

## 12. Verification

Run focused AddressBook/I2PControl tests plus:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
git diff --check
```

Do not add a new remote CI workflow solely for M096.

## 13. Documentation/static guards

Update the M095 matrix from `planned_apply`/current rejection to evidenced SetConfig availability only after production tests pass.

Update `docs/i2pcontrol/proposal-170-support.md` to describe operational SetConfig and confinement semantics while overall Proposal 170 status remains partial until M104.

Add static guards for exact thirteen-key coverage and path confinement ownership where useful; avoid brittle line-number or transient-test-count assertions.

## 14. Acceptance and stop conditions

M096 closes only if:

- all thirteen keys have the M095-approved operational disposition;
- behaviorally meaningful values are actually consumed;
- valid configuration survives restart;
- unsafe paths fail before mutation;
- no global/unrelated filesystem or logger authority is introduced;
- no non-I2PControl production path changed outside the M095/M096 budget;
- focused/broad tests pass;
- no high/medium security finding remains;
- no upstream interaction occurred.

Stop rather than widen scope if:

- a key requires changing runtime resolver precedence beyond accepted AddressBook architecture;
- the contract appears to require arbitrary host filesystem paths;
- an implementation would need a new core/router API;
- global logging/frontend changes would be required for parity.

## 15. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/096-closure.md` with:

- M095 dependency evidence and implementation heads;
- exact changed paths;
- 13-key requirement-to-evidence matrix;
- path-confinement tests;
- runtime worker/cadence/proxy/publication evidence;
- persistence/restart/failure/concurrency evidence;
- RouterInfo config/subscription selector reconciliation;
- containment tests;
- unresolved findings/severity;
- M095 matrix updates;
- internal-only/no-upstream attestation.

## 16. Internal-only rule

All writes remain internal to `eggstack/emissary`. External sources may be read only. No upstream issue/PR/review/submission/merge/contribution preparation or maintainer contact is authorized.
