# M107 — I2PControl Conformance and Managed-TLS Corrective Pass

Status: **ready**

Class: corrective capability / security

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Corrective authority and predecessor evidence:

- M093 tunnel/security closure: `plans/closure/i2pcontrol-proposal-170/093-closure.md`
- M096 AddressBook closure: `plans/closure/i2pcontrol-proposal-170/096-closure.md`
- M104 blocked full-support closure: `plans/closure/i2pcontrol-proposal-170/104-closure.md`
- M106 current implementation closure: `plans/closure/i2pcontrol-proposal-170/106-closure.md`

Repository baseline:

- `06a697006b7b7733587aafed166f438561552193` — `docs(i2pcontrol): close M106 DelayOpen handoff`

Pinned Proposal 170 authority:

- I2P Proposal 170, `I2PControl Expansion`, status `Open`, revision `2026-05-20`;
- <https://i2p.net/en/proposals/170-i2pcontrol-expansion/>.

Read-only compatibility/reference evidence:

- current I2PControl API documentation, updated `2026-07-10`, accurate for Java I2P `2.12.0`: Authenticate requests use API version `1`; unsupported API versions use error `-32006`;
- <https://i2p.net/en/docs/api/i2pcontrol/>;
- I2P Proposal 118, API 2, status `Rejected`; API 2 was rejected because it breaks API 1 backward compatibility;
- <https://i2p.net/en/proposals/118-i2pcontrol-api-2/>;
- I2P naming/address-book documentation: local naming maps are searched in order, the first match is used, conflicts are not detected, and private aliases may intentionally shadow names in broader books;
- <https://i2p.net/en/docs/overview/naming/>.

All external sources are read-only evidence. This plan authorizes writes only to `eggstack/emissary` and does not authorize upstream interaction.

## 1. Objective

Correct three concrete defects found by the post-M106 Proposal 170 conformance/security review while preserving the accepted containment boundary:

1. stop advertising/accepting the rejected I2PControl API version `2` and accept only API version `1`;
2. remove the incorrect global cross-address-book hostname uniqueness rule while preserving deterministic runtime precedence and per-book state;
3. harden I2PControl-managed TLS material so generated private keys are not created with permissive filesystem modes or followed through symlinks, and so the managed loopback certificate is valid for the ordinary loopback identities used to reach the service.

This is a corrective pass over existing implemented behavior. It is **not** a TunnelManager residual-option milestone and MUST NOT alter the M095 support matrix counts: `224 apply / 158 blocked_primitive / 458 not_applicable`.

## 2. Why prior verification missed these defects

### 2.1 API version acceptance

`emissary-cli/src/i2pcontrol/auth.rs` currently treats both `1` and `2` as supported. Existing authentication tests assert that behavior, so the test suite encoded the defect instead of detecting it. Earlier Proposal 170 work focused on the added methods/fields and did not re-audit the base Authenticate version contract against the current API 1 documentation and the rejected API 2 proposal.

Regression evidence must therefore test the normative version boundary, not merely the current helper behavior.

### 2.2 Cross-book hostname collision

`RuntimeAddressBookOwner` already rebuilds its effective runtime index in deterministic order (`private`, then `local`, then `router`, then `published`). However, load/mutation validation rejects any hostname that appears in more than one administrative book, so the precedence code cannot represent ordinary I2P shadowing/alias behavior.

M096 verified path confinement, persistence, mutation transactionality, subscriptions, and SetConfig behavior, but did not include fixtures where the same hostname intentionally exists in two different books. That left a Java/I2P naming compatibility assumption untested.

Regression evidence must exercise both the individual typed books and the effective runtime lookup after create/delete/restart.

### 2.3 Managed TLS material

`emissary-cli/src/i2pcontrol/tls.rs` currently writes generated certificate/key bytes with ordinary `std::fs::write`. It does not explicitly constrain the private-key mode on Unix, and its generated certificate names only `localhost`. Earlier server tests proved that TLS exists and that invalid generated material is regenerated, but did not assert filesystem confidentiality or certificate identity for `127.0.0.1`/`::1`.

Regression evidence must verify restrictive key creation and loopback identity validation without adding a new TLS/X.509 dependency.

## 3. Readiness and current evidence

M107 is dependency-ready because every required owner already exists inside `emissary-cli/src/i2pcontrol/**`:

- API version validation and token issuance are owned by `auth.rs` and the existing Authenticate handler;
- AddressBook state, mutation serialization, persistence, and runtime index reconstruction are owned by `address_book_runtime.rs` and the existing production adapter;
- managed certificate generation/loading is owned by `tls.rs` and uses the already-present `rcgen`, `rustls-pemfile`, and `tokio-rustls` dependencies.

No Yosemite/SAM primitive, tunnel backend, router-core owner, new dependency, schema version, or frontend state is required.

## 4. Invariants

The implementation MUST preserve all of the following:

- Proposal 170 remains pinned to `2026-05-20` and the 158 TunnelManager residual cells remain unchanged;
- authentication must never issue a token for an unsupported API version;
- API 1 request/response spelling and existing I2PControl error codes remain exact;
- AddressBook `Type` remains an independent selector for `private`, `local`, `router`, and `published` state;
- each book remains a deterministic map with no duplicate key inside the same book;
- the effective runtime naming index remains deterministic and uses the existing precedence order unless separate architecture evidence changes it;
- removing global collision rejection must not weaken destination validation, total entry bounds, path confinement, persistence transactionality, or subscription validation;
- a failed AddressBook mutation must leave the prior durable and live generation intact;
- I2PControl remains TLS-only in the current Emissary implementation; TLS hardening must never fall back to plaintext;
- explicit operator-supplied certificate/key paths retain their existing ownership and are not rewritten, chmodded, relocated, or silently replaced by the managed-material path;
- managed private key material must not be exposed through logs, errors, debug output, AddressBook/TunnelManager state, or permissive Unix file modes;
- existing request/body/concurrency/auth-throttle bounds remain unchanged;
- feature-disabled/default Emissary behavior remains unchanged;
- production changes remain under `emissary-cli/src/i2pcontrol/**`.

## 5. Explicit non-goals

M107 MUST NOT:

- implement `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings`, Echo, or other unrelated base-I2PControl parity work;
- add API 2 aliases, lowercase API 2 parameter names, API 2 negotiation, or any version beyond API 1;
- invent a token expiration duration merely because error `-32004` exists; no current normative lifetime was established by this audit;
- relax the existing confined AddressBook SetConfig path policy to reproduce Java example paths such as `../userhosts.txt`; those examples do not justify arbitrary authenticated filesystem selection in this fork;
- change I2P hostname normalization/case rules beyond the exact cross-book collision defect;
- change subscription conflict/merge policy unless a focused regression proves it is the same cross-book validation defect;
- make `published` or any other administrative book newly authoritative through a different router/core path;
- change TunnelManager option support, the M095 matrix, Yosemite, SAM wire serialization, destination/key ownership, or LeaseSet behavior;
- require client certificates/mTLS, invent a certificate-authority subsystem, or redesign remote-management trust policy;
- require an explicit certificate solely because the listener is non-loopback; existing non-loopback warning/configuration policy remains unchanged in this pass;
- add Cargo dependencies or modify `Cargo.lock`;
- change `emissary-core`, `emissary-util`, tunnel data planes, frontend/UI code, workflows, or release automation;
- fix unrelated repository-wide rustfmt/nightly churn;
- prepare or request upstream review, merge, adoption, or submission.

## 6. Required production changes

Expected production paths are limited to:

- `emissary-cli/src/i2pcontrol/auth.rs`;
- the existing Authenticate dispatch/handler path under `emissary-cli/src/i2pcontrol/**` only if required for exact error/token assertions;
- `emissary-cli/src/i2pcontrol/address_book_runtime.rs`;
- the existing I2PControl AddressBook production adapter only if it contains an additional cross-book collision gate;
- `emissary-cli/src/i2pcontrol/tls.rs`;
- an existing I2PControl-local publication helper only if reuse is materially smaller and safer than duplicating TLS-specific file publication logic.

Tests and documentation may change under the existing `emissary-cli/tests/**`, `docs/i2pcontrol/**`, and planning/closure paths.

If implementation requires production changes outside `emissary-cli/src/i2pcontrol/**`, stop and return to planning rather than widening M061 containment implicitly.

## 7. Work packages

### WP1 — Restore exact API 1 authentication negotiation

1. Change the supported API-version predicate to accept exactly version `1`.
2. Preserve the existing `-32005` missing-version and `-32006` unsupported-version error vocabulary.
3. Ensure a request with API `2` and a correct password returns the unsupported-version error and does not allocate/store a token.
4. Preserve successful API 1 authentication and an API `1` response.
5. Replace tests that currently encode API 2 acceptance with regression tests for API 1-only behavior.
6. Add at least one handler-level test rather than relying only on the helper predicate.

Do not alter password comparison, token entropy/capacity, authentication throttling, JSON-RPC ID semantics, or protected-method token validation.

### WP2 — Correct AddressBook cross-book shadowing semantics

1. Remove only the validation that treats a hostname present in two different books as corrupt/invalid state.
2. Preserve all entry-local validation: key/hostname consistency, hostname bounds/control-character restrictions, structurally valid destinations, and aggregate entry limits.
3. Preserve separate typed books. A lookup/list request selecting `private`, `local`, `router`, or `published` must return that book's own entry even when the same hostname exists elsewhere.
4. Preserve the current effective runtime precedence implemented by index rebuilding. The expected order for the current owner is `private` > `local` > `router` > `published`.
5. Prove shadowing lifecycle: when a higher-precedence entry is removed, rebuilding the runtime index exposes the next lower-precedence entry without mutating that lower book.
6. Prove restart persistence: a durable state containing the same hostname in multiple books loads successfully and reconstructs the same effective winner.
7. Audit `validate_runtime_snapshot`, `validate_loadable_snapshot`, configured-generation loading, import/migration, and mutation paths for duplicate global-collision checks. Remove only gates representing the same defect; do not weaken per-book or malformed-state checks.
8. Preserve M096 path confinement and atomic publication behavior unchanged.

The implementation must not use a merged map as the authoritative persisted state. Shadowing is a lookup property over independent books, not destructive deduplication.

### WP3 — Harden managed TLS key publication and loopback identity

1. Keep explicit operator-provided certificate/key handling unchanged.
2. For automatically managed material under `i2pcontrol-certs`, reject existing certificate/key paths that are symlinks or non-regular files before reading or replacement. Do not follow a managed key symlink and do not clobber its target.
3. On Unix, create/publish the managed private key with mode `0600`. The dedicated managed directory SHOULD be owner-only (`0700`) when Emissary creates it. A pre-existing unsafe object must fail closed rather than be followed.
4. Preserve portable behavior on non-Unix platforms without inventing an ACL abstraction in this pass.
5. Generate a managed certificate that is valid for the ordinary loopback identities used to reach the service: DNS `localhost` and IP SANs `127.0.0.1` and `::1`.
6. Use only the already-present rcgen/rustls stack. Do not add an X.509 parser dependency solely for tests.
7. Preserve restart reuse of valid managed material. Invalid ordinary regular-file material may continue to trigger regeneration, but symlink/non-regular-file cases must remain fail-closed.
8. Ensure any publication failure leaves the service initialization failed; never continue with plaintext or partially trusted material.

M107 does not define hostname validation for arbitrary non-loopback remote-management names. Operators using a named remote endpoint may continue to provide explicit TLS material appropriate for that endpoint.

## 8. Failure, cancellation, restart, and contention semantics

### Authentication

Unsupported API versions fail synchronously before token issuance. Concurrent authentication requests retain the existing bounded token store and throttle behavior. No new lock or await is introduced by the version check.

### AddressBook

The existing mutation mutex remains the serialization authority. A mutation that fails validation/publication must not update the live snapshot or effective index. Cross-book shadowing does not add a new lock or background task. Restart must reconstruct the same independent books and effective precedence from durable state.

If configured AddressBook artifacts contain valid duplicate hostnames across distinct books, load must succeed; malformed entries or duplicate-path configuration continue to fail according to M096 rules.

### TLS

TLS material is startup state. A permission/type/write/load error must fail I2PControl initialization rather than start a weaker server. Restart should reuse valid regular managed files. Managed-material validation/publication must not hold an async runtime lock or introduce a long-lived task.

## 9. Compatibility and migration

### API version

This is a compatibility correction. Clients using normative API 1 are unchanged. A client that relied on Emissary's accidental API 2 acceptance will now receive the standard unsupported-version error. No migration alias is provided because API 2 is rejected upstream and was never an accepted compatibility target.

### AddressBook

No schema version change is expected. Previously valid single-book state remains valid. States with the same hostname in multiple books become representable/loadable. Existing precedence code determines the effective lookup result; no entry is discarded during migration.

If legacy persisted state cannot currently contain cross-book duplicates because prior validation rejected publication, no migration rewrite is necessary. The closure record must state whether any old generation repair was needed based on actual repository evidence.

### TLS

Existing valid managed certificate/key material may be reused. If an older managed certificate lacks the new loopback SANs, the implementation must make an explicit decision supported by tests: either regenerate once under a narrowly detectable managed-material condition or retain it until normal regeneration while guaranteeing newly generated material is correct. Do not silently overwrite operator-explicit material.

The preferred behavior is deterministic and bounded; avoid introducing a certificate schema/version subsystem solely for this correction.

## 10. Focused regression tests

At minimum add/adjust evidence for:

### Authentication

- `validate_api_version(1)` succeeds;
- `0`, `2`, `3`, and negative values fail;
- Authenticate API 1 + correct password returns API `1` and a usable token;
- Authenticate API 2 + correct password returns `-32006` and no token is subsequently usable/issued;
- missing API still returns `-32005`;
- existing wrong-password behavior remains unchanged.

### AddressBook

- same hostname with different valid destinations may exist in two books;
- typed lookup/list returns the correct per-book destination for both entries;
- effective runtime lookup chooses the current precedence winner;
- deleting the winner exposes the next entry;
- restart/load preserves both entries and the same winner;
- configured-generation load accepts cross-book shadowing;
- malformed hostname/destination, aggregate entry overflow, same configured path collision, path escape, and symlink guards remain rejected;
- RouterInfo address-book projections retain both entries in their respective lists.

### TLS

- newly generated managed key is mode `0600` on Unix;
- managed directory creation is owner-only on Unix if the implementation changes its mode;
- an existing symlink key/certificate path is rejected without modifying the symlink target;
- ordinary invalid regular managed material retains deterministic regenerate-or-fail behavior;
- generated certificate validates for `localhost`, `127.0.0.1`, and `::1` using existing rustls/rcgen facilities and no new dependency;
- valid managed material is reused across restart;
- explicit certificate/key configuration behavior remains unchanged.

Use local/loopback fixtures only; this milestone does not require a public I2P network.

## 11. Broad verification

Run the focused tests plus the existing feature/containment suite:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo check
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

The repository has a documented stable/nightly rustfmt mismatch. Run the formatter check and record its exact outcome, but do not retain formatter-only churn outside the authorized I2PControl/test/documentation paths merely to make that unrelated gate green.

M095/M105 matrix tests should also be run if any shared test/docs guard is touched. Their production counts MUST remain unchanged.

## 12. Documentation and static guards

Implementation must update the existing I2PControl documentation where necessary to state:

- Emissary implements I2PControl API version 1, not rejected API 2;
- independent AddressBook types may shadow the same hostname and the current effective runtime precedence is deterministic;
- managed self-signed material includes ordinary loopback identities and generated private key material is stored restrictively on Unix;
- full Proposal 170 support remains partial because the 158 applicable TunnelManager cells are unaffected by M107.

Do not change M095 option dispositions or use this correction to claim full Proposal 170 support.

A static/containment guard should fail if M107 adds a new dependency, modifies Yosemite/core/util production paths, or changes the M095 matrix counts. Reuse M061/M062/M095 guards rather than creating a redundant CI framework.

## 13. Acceptance criteria

M107 may enter closure only when all are true:

1. API version `1` is the sole accepted Authenticate API version and API `2` receives the standard unsupported-version response without token issuance.
2. Valid cross-book hostname shadowing is accepted, persists across restart, and resolves according to the existing deterministic runtime precedence while typed books remain independent.
3. No AddressBook destination/path/size/transactionality guard is weakened beyond removal of the global collision defect.
4. Fresh managed TLS private keys are restrictive on Unix and managed symlink/non-regular-file attacks fail closed.
5. Fresh managed certificates validate for `localhost`, `127.0.0.1`, and `::1` using existing dependencies.
6. Explicit operator TLS material behavior remains unchanged.
7. Feature-gated I2PControl tests, containment tests, live local runtime test, check, and clippy pass or any unrelated pre-existing failure is precisely recorded and reproduced outside the changed scope.
8. No production file outside `emissary-cli/src/i2pcontrol/**` changed.
9. No Cargo manifest/lockfile/dependency change occurred.
10. M095 remains `224 apply / 158 blocked_primitive / 458 not_applicable`.
11. Documentation continues to state partial Proposal 170 support.
12. Closure records the exact implementation commits, regression evidence, compatibility/security review, and internal-only attestation.

## 14. Stop conditions

Stop and return to planning if:

- exact API 1 behavior appears to require implementing unrelated base methods or an API 2 compatibility layer;
- AddressBook shadowing cannot be corrected without changing router-core naming ownership or weakening M096 confinement/transactionality;
- certificate SAN or permission testing requires a new dependency rather than the existing rcgen/rustls stack;
- TLS hardening would require changing explicit operator certificate ownership or global router TLS policy;
- any fix requires Yosemite, SAM, core/util, frontend, workflow, or dependency changes;
- any M095 TunnelManager cell would change disposition;
- evidence contradicts the pinned/current external specifications named above.

A stop condition must be recorded as a planning/closure finding rather than worked around with broader architecture.

## 15. Closure evidence required

The M107 closure record must include:

- exact implementation commit(s);
- requirement-to-evidence table for WP1-WP3;
- exact external read-only sources and the contract conclusions drawn from them;
- authentication error/token evidence for API 1 vs API 2;
- cross-book create/list/effective-lookup/delete/restart evidence;
- TLS mode, symlink, SAN/handshake, regeneration/reuse evidence;
- focused and broad verification commands with exact outcomes;
- M061/M062 containment review and changed-path inventory;
- confirmation that M095 remains `224 / 158 / 458`;
- compatibility/migration review;
- unresolved findings with severity;
- explicit statement that base-method parity, token-expiration policy, AddressBook path-confinement relaxation, and TunnelManager residuals remain outside M107;
- attestation that external sources were read-only, all writes stayed in `eggstack/emissary`, and no upstream issue/PR/review/submission/adoption/merge/contact or contribution artifact was created.
