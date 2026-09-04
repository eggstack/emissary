# M130 — Post-M127–M129 Corrective Requalification

Status: **blocked / unregistered**

Class: corrective requalification / conformance / security / containment

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Hard dependencies:

- M127 authentication token-lifetime corrective — must be closed;
- M128 JSON-RPC batch conformance corrective — must be closed;
- M129 non-loopback managed-TLS fail-closed corrective — must be closed.

Predecessor historical authority:

- M126 closure: `plans/closure/i2pcontrol-proposal-170/126-closure.md`;
- M095 full-support matrix;
- M105 residual-option audit;
- M061/M062 containment;
- M093 tunnel security.

Planning baseline: `9948cfd0782a3defbd5f68cf2d4523603bdc7940` for plan creation. M130 implementation/review baseline MUST be reset to the actual merged head after M127–M129 closures.

Pinned authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- the existing I2PControl transport/authentication/version behavior required by the extension surface;
- JSON-RPC 2.0 envelope/request-ID/notification/batch semantics used by that surface.

Current Proposal matrix expected entering M130: `284 apply / 96 blocked_primitive / 460 not_applicable` unless an earlier corrective produces independently evidenced cell-level reclassification.

## 1. Objective

Requalify the implemented Proposal 170 subset after the concrete M126-missed shared-control-plane defects are corrected, and establish a new current-head closure authority without rewriting M126 history.

M130 is not a residual-capability implementation milestone. It must answer whether the corrected I2PControl service is operationally and security-qualified on the actual post-M129 head, with particular emphasis on the defects that M126 incorrectly closed:

- finite token lifetime and reachable `TOKEN_EXPIRED` behavior;
- bounded JSON-RPC batch handling rather than blanket rejection;
- fail-closed remote/non-loopback TLS configuration.

It must also rerun enough of the broader Proposal 170 production-owner, persistence, lifecycle, source-truthfulness, and containment evidence to ensure those corrective changes did not regress previously qualified behavior.

M130 is the only milestone in this sequence allowed to restore a clean current-head “implemented subset qualified” statement.

## 2. Canonical scope boundary

`plans/000-long-term-specification.md` remains controlling.

M130 MUST NOT turn this corrective sequence into a general base-I2PControl parity project. Unrelated base methods such as `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, and `AdvancedSettings` remain explicit non-goals merely for Proposal 170 completion.

Shared base behavior is in scope only where needed by the implemented Proposal 170 extension surface: authentication/version/token semantics, HTTPS serving, JSON-RPC envelopes/IDs/notifications/batches, and protected dispatch.

This resolves the audit concern without contradicting the repository's canonical scope.

## 3. Why a new requalification is required

M126 closure stated that no authentication/TLS/JSON-RPC defect was present. Subsequent independent review found concrete defects in exactly that evidence class. Historical M126 remains a useful record of the reviewed head and test suite, but its clean shared-control-plane qualification is superseded for current authority.

A sequence of individual corrective closures is necessary but insufficient by itself: M130 must prove their composition and re-exercise the surrounding production service.

## 4. Hard invariants

M130 MUST verify, not merely assume:

- finite token lifetime with atomic expiry removal and exact expired/unknown errors;
- no protected request succeeds after expiry;
- valid single JSON-RPC requests remain compatible;
- valid bounded batches execute with per-element authentication and correct notification suppression;
- oversized batches execute zero elements;
- no batch request creates unbounded task/resource fan-out;
- managed TLS is loopback-only and non-loopback binds require explicit material;
- no TLS failure falls back to plaintext;
- passwords/tokens/private keys/destinations/filesystem internals remain secret-safe;
- AddressBook uses the authoritative resolver/runtime owner and remains persistence/failure atomic;
- TunnelManager claimed `apply` behavior reaches real backends and retains M123 cancellation guarantees;
- blocked options remain fail-before-effect;
- RouterInfo and ClientServicesInfo remain source-truthful;
- Proposal-specific business policy remains primarily under `emissary-cli/src/i2pcontrol/**`;
- default/feature-disabled Emissary remains unaffected;
- optional Yosemite alias containment remains exact;
- no active documentation claims full Proposal 170 support while applicable residuals remain.

## 5. Ordered work packages

### WP1 — freeze the post-corrective head and evidence inventory

1. Record exact M127, M128, and M129 implementation/closure commits.
2. Freeze the actual M130 reviewed head.
3. Reconcile active registry/roadmap/README status.
4. Recompute M095 matrix counts mechanically.
5. Verify none of M127–M129 silently changed Proposal option applicability/support.

### WP2 — shared control-plane black-box qualification

Against the real child-process TLS server, prove:

- successful API-1 Authenticate and protected request;
- missing/invalid/conflicting credential rejection;
- finite token expiry semantics through deterministic lower-level evidence plus live pre-expiry success;
- single-request IDs and notification suppression;
- bounded valid/mixed/all-notification batch behavior;
- malformed/empty/over-cap batch behavior;
- remote/non-loopback managed-TLS configuration rejects before listener/filesystem side effects;
- loopback managed TLS remains operational;
- complete explicit TLS remains the only accepted non-loopback path;
- plaintext and failed TLS never reach JSON-RPC dispatch;
- request body, connection, concurrency, batch cardinality, auth throttle, handshake, and request deadline limits remain effective.

### WP3 — AddressBook regression qualification

Rerun focused production/live evidence for:

- Add/Lookup/Delete;
- subscriptions and SetConfig;
- restart/persistence round trip;
- cross-book precedence;
- confinement/symlink/type/malformed input handling;
- concurrent mutation and failed publication behavior.

No broad new AddressBook implementation is authorized.

### WP4 — TunnelManager regression qualification

Rerun representative and matrix-driven evidence for:

- canonical create/edit/get/start/stop/restart/delete;
- all twelve backends' registered real runtime ownership;
- failed start/edit/restart rollback;
- M123 cancellation terminalization;
- duplicate/collision/startup ownership behavior;
- local-target and application-filter boundaries;
- resource/admission bounds;
- blocked option fail-before-allocation/secret-generation/persistence.

M130 does not attempt to reduce the 96 blocked residuals.

### WP5 — RouterInfo and ClientServicesInfo regression qualification

Verify exact current selectors against authoritative request-time/live sources and the six service selectors against actual configured/runtime state. Retain the one protocol-permitted neutral RouterInfo disposition only if current evidence still supports it.

### WP6 — containment and dependency audit

Compare the post-M129 diff to M061/M062 authority.

M127–M129 should require no production changes outside `emissary-cli/src/i2pcontrol/**`. Any unexplained external production change is a closure blocker.

Recheck optional exact `yosemite-i2pcontrol` pin isolation and ordinary registry Yosemite use.

### WP7 — active authority reconciliation

Update active docs/planning to state:

- M126 historical qualification was superseded for the affected shared-control-plane findings;
- M127–M129 are closed with their exact scopes;
- M130 is the new current-head requalification authority if it closes cleanly;
- Proposal support remains partial with the mechanically reproduced matrix;
- unrelated base I2PControl method parity remains outside this workstream per canonical specification.

Historical closure records are not rewritten.

## 6. Failure, cancellation, restart, and contention semantics

M130 must explicitly cover:

- token expiry races;
- batch deadline/cancellation after earlier independent mutations have committed;
- no batch-level transactional rollback claim;
- server shutdown clearing all tokens and active request state;
- TLS configuration failure before service side effects;
- AddressBook concurrent mutation/persistence failure behavior;
- TunnelManager per-name lifecycle exclusion and cancellation terminalization;
- restart recovery for persisted administrative state and destination secrets;
- no lock held across unrelated network I/O merely for requalification instrumentation.

## 7. Compatibility and migration review

Closure must distinguish intentional behavior corrections from regressions:

- expired credentials now require re-authentication;
- valid JSON-RPC batch clients become supported;
- non-loopback users relying on managed loopback-only identity must configure explicit TLS material;
- single-request and loopback managed-TLS clients should otherwise remain compatible;
- no Proposal persistence schema migration should result from M127–M129;
- no unrelated base I2PControl methods are added merely for the closure claim.

## 8. Focused test requirements

M130 should add only integration/requalification tests missing from M127–M129. Do not duplicate all lower-level unit coverage.

At minimum maintain a current-head requalification test that fails if:

- token storage has no finite expiry behavior;
- `TOKEN_EXPIRED` becomes unreachable;
- valid batch arrays regress to blanket invalid-request responses;
- all-notification batches emit a JSON-RPC body;
- over-cap batches execute side effects;
- non-loopback managed TLS is accepted;
- plain HTTP reaches dispatch;
- fake adapters appear in production composition;
- active matrix/docs disagree with current authority.

## 9. Broad verification

Run and record exact outcomes:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial --test i2pcontrol_live_runtime --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

If new milestone-specific requalification tests are added, include them explicitly in the closure command list.

Known stable/nightly rustfmt limitations must be recorded rather than normalized through unrelated churn.

## 10. Acceptance criteria

M130 closes only when:

1. M127–M129 are individually closed and their commits are present in the reviewed head;
2. shared auth/JSON-RPC/TLS behavior is black-box qualified at current head;
3. token expiry, batch handling, and remote TLS fail-closed behavior have durable regression evidence;
4. no high/medium shared-control-plane security/conformance defect remains open;
5. AddressBook/TunnelManager/RouterInfo/ClientServicesInfo representative production evidence remains green;
6. M123 cancellation and existing application/filter security boundaries remain intact;
7. all blocked Proposal options remain fail-before-effect unless separately reclassified from independent evidence;
8. matrix counts are mechanically reproduced or truthfully corrected;
9. no unexplained production change exists outside accepted containment;
10. active docs/registry/roadmap identify M130 as current authority and retain partial-support wording;
11. unrelated base I2PControl parity is not smuggled into the workstream;
12. broad verification has no unexplained regression.

## 11. Stop conditions and successor rule

Stop clean closure and register M131+ only for a concrete independently evidenced defect such as:

- auth/token expiry bypass;
- batch auth/resource bypass;
- TLS/plaintext/remote identity regression;
- production fake/shadow state;
- mutation success-before-commit;
- lifecycle cancellation/resource defect;
- source-truthfulness regression;
- containment regression;
- newly available residual primitive with exact canonical owner and semantics.

Do not opportunistically implement residual Proposal capability inside M130.

## 12. Closure evidence required

M130 closure must include:

- exact post-M129 reviewed head;
- M127–M129 closure/commit table;
- current matrix recomputation;
- shared-control-plane black-box results;
- representative production-owner traces;
- persistence/restart/cancellation/contention evidence;
- containment/dependency audit;
- compatibility/migration/security review;
- exact verification commands/outcomes;
- unresolved findings with severity;
- current-authority and successor-readiness disposition;
- internal-only external-interaction attestation.

## 13. External-interaction boundary

External I2P/JSON-RPC/reference sources remain read-only. Repository writes are authorized only to `eggstack/emissary`.

No upstream issue, PR, review, discussion, release, submission, merge/adoption request, maintainer contact, contribution package, or third-party repository mutation is authorized.