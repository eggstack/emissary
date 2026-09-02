# M111 — SAM Session-Wire Option Completion

Status: **ready** — M117's accepted Yosemite dependency gate and M118's neutral SAM/tunnel-pool capability are closed; execution still requires the semantic re-freeze and evidence defined below

Class: capability / dependency integration / protocol-adjacent containment

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Source evidence:

- M097 closure: `plans/closure/i2pcontrol-proposal-170/097-closure.md`
- M105 audit: `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`
- M092/M093 authority for the prohibition on unauthorized Yosemite vendoring/core expansion.

Pinned authority: I2P Proposal 170 revision `2026-05-20`, status Open.

All external repositories and dependency sources are read-only evidence. Writes are internal to `eggstack/emissary` only.

## 1. Objective

Complete the Proposal 170 option cells whose required effect is serialization into the SAM/Yosemite session-creation contract rather than application-layer behavior:

- `UseSSL` — 4 applicable cells;
- `TunnelVariance` — applicable client/server session cells in the M105 `SessionWire` family;
- `TunnelBackupQuantity` — same family;
- `SigType` — same family;
- `CustomOptions` — same family.

M105 records 40 cells in the `SessionWire` group plus four `UseSSL` cells, for a maximum M111 target of 44 current blocked cells. The implementation baseline must re-freeze the exact list from M095/M105 before changing any disposition.

The sole valid completion mechanism is to pass validated Proposal 170 values through an accepted public session-option interface to the actual Yosemite `SESSION CREATE` path used by Emissary. Storing raw values or reproducing SAM manually does not count.

## 2. Resolved dependency gate and execution condition

At the M105/M108 baseline, the accepted Yosemite 0.7.0 API did not expose all required
session-wire semantics; that historical evidence explains why M111 was blocked.

That dependency blocker is now resolved by M117's accepted exact-revision adapter, and
M118 now supplies the real neutral variance/backup tunnel-pool effect. M111 is promoted
to ready, but it must re-freeze the exact Proposal 170 option/cell set at execution time
and may keep any unsupported semantic slice blocked.

M111 is ready because the first condition is explicitly satisfied, subject to its own
execution-time semantic freeze:

1. a released crates.io Yosemite version exposes the needed public typed/raw session-option fields and can be adopted without Proposal-170-shaped changes outside the dependency boundary; or
2. a separately accepted architecture decision authorizes a narrowly scoped dependency strategy that preserves M062/M093 containment and does not recreate the invalid M091 vendor/path-dependency pattern.

A maintainer must explicitly register M111 after that evidence exists. The existence of this plan is not authorization to fork, vendor, patch, or contact Yosemite upstream.

## 3. Invariants

When unblocked, M111 MUST preserve:

- Yosemite remains the one SAM client implementation; no parallel raw-SAM stack;
- no Proposal-170-specific API is added to `emissary-core`;
- no dependency merely to make matrix counts green;
- exact value types/ranges and exact applicable tunnel families from the pinned Proposal/reference behavior;
- unknown/unsupported custom tokens fail before session allocation;
- `CustomOptions` cannot override security-critical options already represented by typed fields in a contradictory way;
- no silent downgrade of signing/encryption/tunnel settings;
- session-wire options are applied to the same generation whose successful start is reported;
- edit/restart uses new values only after old-generation cancellation and validation;
- secrets are not introduced into raw logs/errors;
- default/feature-disabled Emissary behavior remains unchanged;
- M093 anonymity/resource boundaries remain intact;
- internal-only external interaction.

## 4. Explicit non-goals

M111 MUST NOT:

- implement `Shared`, destination/key lifecycle or `PrivKeyFile` from M110;
- implement proxy/client-lifecycle residuals from M112;
- implement server presentation/LeaseSet residuals from M113;
- vendor Yosemite under `vendor/**`;
- use a git/path dependency without a separately accepted architecture decision explicitly authorizing it;
- copy Yosemite's SAM protocol into I2PControl;
- add string concatenation around `SESSION CREATE` outside the accepted dependency API;
- modify router algorithms, transports, tunnel-build protocol, frontend, or startup configuration;
- add API 2 or unrelated I2PControl methods;
- prepare/request upstream changes or review.

## 5. Expected production paths once unblocked

Preferred Emissary-side changes remain under:

- `emissary-cli/src/i2pcontrol/domain/tunnel.rs`;
- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` and affected backends;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`;
- M095/M105 matrix/audit artifacts and focused tests.

A Yosemite version change may require `emissary-cli/Cargo.toml` and `Cargo.lock` only if explicitly authorized by the readiness evidence. Root workspace ownership must not be widened merely for I2PControl.

No `emissary-core` or `emissary-util` production change is authorized by this plan.

## 6. Work packages

### WP1 — Dependency capability freeze

At execution time:

1. record the exact Yosemite version/source;
2. inspect its public API and actual session serialization path read-only;
3. map each target Proposal option to one exact public field/typed setting or safe documented raw-option channel;
4. prove the dependency path reaches real `SESSION CREATE` serialization;
5. record unsupported fields before any Emissary code change.

If any target requires a dependency fork/private patch, stop that cell and keep it blocked.

### WP2 — Typed validation and normalization

Implement strict I2PControl-side validation before constructing Yosemite options.

- tunnel variance/backups: numeric bounds and units must match pinned semantics;
- `SigType`: accept only supported canonical values and fail explicitly when Yosemite/router support is absent;
- `UseSSL`: map only if it has real SAM session meaning in the accepted dependency; do not confuse it with I2PControl HTTPS or proxy TLS;
- `CustomOptions`: parse into a bounded token map/list, reject malformed/control/oversized content, duplicates/conflicts, and keys that would bypass typed policy.

Do not pass arbitrary unreviewed strings through to the dependency.

### WP3 — Session creation integration

Extend the existing I2PControl session builder/config path so validated values are present before any session/listener allocation. Every affected backend must consume the same typed session configuration rather than duplicating wire policy.

### WP4 — Edit/restart semantics

Session-affecting options require a new session generation. An edit must not mutate a live session in place if Yosemite cannot do so safely.

- validate and persist the candidate definition transactionally;
- stop/cancel the old generation through existing lifecycle ownership;
- allocate the new session with the new wire options;
- on allocation failure, preserve/restore the prior durable/runtime generation according to existing TunnelManager rollback semantics;
- never report a new value as active while the old session is still running.

### WP5 — Matrix evidence

For each cell changed to `apply`, add a focused fake-SAM or dependency-level regression that captures the actual `SESSION CREATE` semantics where possible. A parser/unit test that never reaches the dependency is insufficient.

## 7. Failure, cancellation, restart, contention

All validation occurs before session allocation. Session creation is bounded by existing startup/runtime deadlines and generation cancellation. No lock crosses dependency network I/O.

Restart must reconstruct the same validated session options from durable definitions. Unsupported values in legacy/injected state fail activation rather than being dropped.

A failed M111 option edit must not terminate unrelated shared sessions from M110; compatibility keys must include M111 session-affecting fields when sharing is enabled.

## 8. Compatibility and migration

Previously blocked values have no operational migration promise. Once supported, old definitions containing raw blocked values may be activated only after they pass the new typed validation and are represented by the canonical owner.

If a Yosemite version update is required, closure must record the exact dependency diff and prove default feature isolation. Do not change dependency provenance silently.

## 9. Focused tests

At minimum:

- exact serialized session option evidence for each implemented family;
- invalid/out-of-range values reject before allocation;
- unsupported `SigType` does not downgrade;
- `CustomOptions` cannot override typed security/session settings;
- edit/restart creates a new generation with changed wire settings;
- failure leaves prior definition/runtime truthful;
- M110 shared-session compatibility includes all M111 fields;
- feature-disabled/default dependency reachability remains unchanged except any explicitly approved optional dependency version update.

## 10. Broad verification

Run the active roadmap's standard feature, containment, matrix/audit, live-runtime, check, clippy, fmt-attempt and diff checks. If a dependency update is authorized, additionally run the existing M062 dependency ownership guards and inspect `Cargo.lock`/dependency provenance explicitly.

No new CI/fuzz/release infrastructure is required.

## 11. Acceptance criteria

M111 closes only when:

1. every target cell moved to `apply` reaches real Yosemite session creation with the requested semantics;
2. remaining cells are explicitly blocked with exact dependency evidence rather than accepted inertly;
3. no parallel SAM implementation, vendored/path Yosemite copy, or Proposal-shaped core API exists;
4. edit/restart and shared-session compatibility are generation-safe;
5. no security option silently downgrades;
6. matrix/audit counts and exact cell deltas are recorded;
7. dependency/default-feature containment passes;
8. closure names the next registry-ready plan.

## 12. Stop conditions

Stop if:

- the released accepted Yosemite API still cannot express a target option;
- completion requires raw SAM construction outside Yosemite;
- a dependency fork/vendor/path override would be required without an explicit superseding architecture decision;
- a target option's reference semantics are ambiguous enough that runtime behavior would be guessed;
- `CustomOptions` cannot be bounded without bypassing typed security policy;
- implementation would require `emissary-core` protocol changes.

## 13. Closure evidence

Require dependency/source/version evidence, exact public API-to-wire mapping, cell-by-cell matrix changes, serialized/fake-SAM evidence, failure/restart/contention review, M062 dependency containment, M093 security review, full verification outcomes, unresolved findings, next-handoff decision, and internal-only attestation.

## 14. Internal-only boundary

No external repository write, dependency PR/issue, review request, patch submission, maintainer contact, release, or contribution preparation is authorized. External dependency/specification inspection is read-only only.
