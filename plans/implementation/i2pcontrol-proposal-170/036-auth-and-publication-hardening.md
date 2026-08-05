# M036 — Authentication and Persistent Publication Hardening

Status: closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Applicable governance:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`

Repository baseline:

- `5620cb8` — accepted M035 implementation/closure head

Hard dependency:

- M035 closed

## 1. Bounded objective

Harden two remaining security/reliability boundaries without changing Proposal
170 wire behavior:

1. replace the hand-written password comparison and add bounded failed-login
   throttling; and
2. make I2PControl persistent publication claims match actual file and directory
   durability/recovery guarantees.

M036 covers I2PControl token/auth state, tunnel-definition persistence, server
secret persistence, and AddressBook control-state publication only where they
share the same bounded publication primitive. It does not redesign router-wide
storage or authentication.

## 2. Current findings

Authentication currently uses a custom byte-loop comparison. Its documentation
claims additional time padding that is not actually performed, and the
unequal-length accumulator is not consumed in a way that guarantees optimizer
retention. The service also has request/connection bounds but no explicit
failed-authentication throttle.

Persistent stores generally use temp/write/file-sync/rename and current/backup
recovery. Containing-directory sync and backup durability are not consistently
established, while some documents describe success as fully durable.

M036 must either strengthen publication to the claimed level or narrow the
claim precisely on unsupported platforms.

## 3. Required invariants

1. Authentication request/response fields and standard I2PControl error codes do
   not change.
2. Password comparison uses a reviewed constant-time primitive or an existing
   audited repository utility, not a new hand-written loop.
3. Failed-login throttling is bounded in memory and work.
4. Throttling cannot lock out successful loopback administration indefinitely.
5. Authentication occurs before protected expensive parsing/work.
6. Tokens remain cryptographically random, opaque, bounded, restart-invalidated,
   and excluded from logs.
7. Publication success is reported only after the documented persistence point.
8. Failure leaves a recoverable prior generation and no active partial state.
9. Directory synchronization is performed where supported if strict rename
   durability is claimed.
10. Platform-specific limitations are explicit and tested/documented rather than
    hidden.
11. No general database/storage framework or router-wide persistence refactor.
12. No public protocol extension, core/router algorithm, frontend, CI/release, or
    upstream work.

## 4. Scope and file budget

Primary production scope:

- `emissary-cli/src/i2pcontrol/auth.rs`;
- `emissary-cli/src/i2pcontrol/server.rs` authentication gate;
- I2PControl store/publication helpers;
- AddressBook control-owner publication helper only if it can consume the same
  bounded primitive without changing ownership;
- server destination secret store from M032;
- focused tests and directly affected documentation.

A single small dependency may be added only if:

- no existing audited constant-time primitive is available;
- it is narrowly used for password equality;
- features/default features are minimized;
- dependency and license review are recorded.

Prefer an existing direct dependency or repository crypto utility. Do not add a
general authentication framework.

Hard exclusions:

- OAuth/session-account systems;
- remote identity providers;
- TLS redesign;
- global IP reputation or firewall integration;
- router-wide storage abstraction;
- core, data-plane, frontend, CI/release, or unrelated changes.

## 5. Target authentication model

### 5.1 Constant-time comparison

Use a reviewed primitive with explicit handling of different lengths. Remove
incorrect comments about artificial delay unless real bounded delay is
implemented.

Do not log provided/expected passwords, hashes, lengths, or timing.

### 5.2 Failed-login throttle

Use a bounded local in-memory structure keyed by the narrowest trustworthy
connection/source identity available at the TLS server boundary. Requirements:

- fixed maximum entries with deterministic eviction;
- monotonic-time windows;
- bounded exponential or token-bucket delay no greater than the request
  deadline;
- successful authentication clears or decays the relevant failure state;
- no unbounded task per failure;
- no sleep while holding the throttle lock;
- global emergency bound to prevent source-key churn from exhausting memory;
- restart clears throttle state.

Loopback remains the default bind. Non-loopback deployment warnings remain.

### 5.3 Token capacity

Review current clear-all-at-capacity behavior. Retain it if documented and safe,
or replace it with deterministic bounded eviction without changing token wire.
Do not make token state persistent.

## 6. Target publication model

### 6.1 Shared bounded helper

Where practical, factor an I2PControl-owned helper for:

- serialize/validate size;
- create parent directory;
- write fixed-name temp file;
- sync temp file;
- preserve recoverable prior generation;
- atomic rename;
- sync containing directory on supported Unix-like platforms;
- cleanup temp on failure;
- sanitized error classification.

Do not accept request-selected paths. Each store supplies a fixed path root and
bounded payload.

### 6.2 Recovery

Tests must cover:

- valid current;
- corrupt current/valid backup;
- missing current/valid backup;
- failed temp write/sync/rename/directory sync;
- stale temp file;
- cancellation before and after rename;
- unsupported-platform durability qualification.

Live state changes only after the chosen durable publication point succeeds.

### 6.3 Documentation claim

Use exact language:

- process-crash atomicity;
- prior-generation recovery;
- power-loss durability where directory sync is available and succeeds;
- qualified behavior where platform APIs do not expose equivalent guarantees.

Do not use unqualified `durable-before-success` if evidence is weaker.

## 7. Ordered work packages

### WP1 — Freeze authentication defects

Add focused tests/guards that reject the current hand-written comparator and
exercise bounded throttle behavior, successful reset, capacity, and concurrent
failures.

### WP2 — Select and integrate the comparison primitive

Record dependency/utility decision and remove dead timing comments/code.

### WP3 — Add bounded throttle at the authentication boundary

Throttle only authentication failures. Protected requests with invalid tokens
retain existing token error behavior unless a direct abuse bound is needed and
separately justified.

### WP4 — Inventory publication paths and claims

List every affected store, exact current algorithm, size bound, backup behavior,
and platform guarantee. Avoid silently changing unrelated legacy AddressBook
files.

### WP5 — Implement/consume the bounded publication primitive

Apply it only to I2PControl-owned stores and the M030 owner file where ownership
requires consistency. Preserve file formats and recovery compatibility.

### WP6 — Failure injection and recovery evidence

Use focused local filesystem fault hooks/fakes already present or small test-only
injection points. Do not add a large filesystem abstraction.

### WP7 — Documentation and disposition

Update security/persistence docs and create:

- `plans/closure/i2pcontrol-proposal-170/036-implementation-disposition.md`.

## 8. Failure, cancellation, restart, and contention semantics

- Throttle state lock is released before delay.
- Cancellation during delay changes no token/password state.
- Successful authentication after the allowed delay issues one token and clears
  relevant failure state.
- Publication cancellation before rename leaves prior state; after completed
  rename/directory sync leaves committed state.
- Failed backup or directory sync returns failure according to the documented
  durability contract and does not update live state.
- Concurrent writers retain existing store serialization and publish complete
  generations only.

## 9. Compatibility and migration

- No authentication wire change.
- Existing passwords/config remain valid.
- Tokens remain in-memory and restart-invalidated.
- Existing store formats/current/backup files remain readable.
- New directory sync does not require data migration.
- Documentation may narrow prior durability claims where platform evidence is
  unavailable.

## 10. Security review requirements

Review and test:

- constant-time primitive use and no optimizer-sensitive custom fallback;
- no password/token/source-key disclosure;
- throttle memory/time bounds and source-churn behavior;
- no lock-held sleep;
- loopback and non-loopback behavior;
- fixed path/temp names and symlink/irregular-file handling;
- restrictive server-secret permissions;
- failure logs are sanitized;
- no upstream interaction.

## 11. Focused tests

Required semantics include:

- `password_comparison_uses_reviewed_primitive`;
- `failed_authentication_is_bounded_and_throttled`;
- `successful_authentication_resets_failure_state`;
- `throttle_capacity_is_bounded_under_source_churn`;
- `throttle_sleep_holds_no_lock`;
- `publication_syncs_file_and_directory_where_supported`;
- `publication_failure_preserves_prior_live_generation`;
- `corrupt_current_recovers_valid_backup`;
- `stale_temp_does_not_override_current`;
- `server_secret_publication_retains_permissions_and_redaction`.

## 12. Verification commands

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol auth
cargo test -p emissary-cli --no-default-features --features i2pcontrol persistence
cargo test -p emissary-cli --no-default-features --features i2pcontrol recovery
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Also run no-feature check if a shared AddressBook publication helper changes:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features address_book
```

Use targeted formatting and `git diff --check`. No CI/release/fuzz/soak
expansion.

## 13. Documentation and static guards

Add guards proving:

- no custom constant-time byte loop remains in authentication;
- throttle capacity/window/delay constants are bounded;
- no password/token logging;
- publication paths are fixed-owner paths;
- live state updates after publication success only;
- public wire remains unchanged;
- no core changes.

## 14. Acceptance criteria

M036 may close only when:

- reviewed password comparison and bounded throttle are evidenced;
- publication/recovery guarantees match documentation;
- existing store formats and wire remain compatible;
- no high/medium M036 security or durability defect remains;
- all shared-file changes are justified;
- implementation disposition and frozen head are committed;
- no upstream interaction occurred.

## 15. Stop conditions

Stop and record `blocked` if:

- hardening requires a general auth/account system;
- source identity cannot be obtained safely for bounded throttling;
- strict durability requires a router-wide storage rewrite;
- existing formats cannot be preserved;
- a broad dependency expansion is required;
- unrelated core/frontend/CI work becomes necessary;
- external authority changes materially;
- upstream action is requested without explicit authorization.
