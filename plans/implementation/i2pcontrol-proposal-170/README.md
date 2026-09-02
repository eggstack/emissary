# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 production support; M109, **M115, and M110 are closed**; M111-M114 remain roadmap-defined and blocked; current TunnelManager matrix is `255 apply / 127 blocked_primitive / 458 not_applicable`

This directory contains bounded internal implementation, audit, corrective, and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative references:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`
- ADR-0001/0002/0003/0004
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- `061-containment-boundary.toml`
- `062-dependency-containment.toml`
- `095-full-support-matrix.toml`
- `105-residual-option-audit.toml`
- `plans/registry.md`

Pinned Proposal 170 revision: `2026-05-20` (proposal remains Open).

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, I2P/Java I2P/i2pd/I2P+/Yosemite source, issues, commits, pull requests and reference routers are read-only evidence.

No plan authorizes upstream submission, review request, maintainer contact, contribution preparation, merge/adoption request, issue/PR mutation, branch/tag push, release, or repository write outside this fork.

## Scope and containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

M061/M062/M063 remain the containment authority. Non-I2PControl production changes require an exact neutral canonical owner, pre-authorized paths, and a registered plan. M091 remains the cautionary case: unauthorized vendored Yosemite/core/dependency expansion was removed by M092 and independently reclosed by M093.

M109/M115 are the bounded exception for existing CLI startup tunnel ownership: only `emissary-cli/src/main.rs` and `emissary-cli/src/tunnel/{client,server}.rs` may carry the neutral lifecycle seam described by the registered plan. This is not a general authorization to move Proposal policy outside I2PControl.

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys, cross-book shadowing and normal lookup coherence are operational.
- Exactly 12 Proposal 170 tunnel types have real backends.
- Exactly seven canonical TunnelManager action handlers exist.
- All six ClientServicesInfo selectors are operational.
- API 1-only authentication and M107/M108 managed TLS hardening are operational.
- M097/M098/M099/M106 applied bounded option subsets with real runtime effects.
- Unsupported residual options fail before allocation.
- M109 added startup named lifecycle and `All=true`; M115 corrected runtime-disable isolation, contention-truthful state observation, and controlled shared-client session recovery/lifetime.
- Full public/reseeded/reference-router certification remains open.

Current M095 matrix:

- 255 `apply`;
- 127 `blocked_primitive`;
- 458 `not_applicable`;
- 0 planned/unknown/unsupported/accept-inert cells.

Official status remains **partial Proposal 170 support** until the residual-option line closes and M114 succeeds.

## Current/future handoff sequence

| Handoff | Status | Scope |
|---|---|---|
| M095 | closed | exact full-support matrix/containment budget |
| M096 | closed | all 13 AddressBook SetConfig keys |
| M097 | closed as blocked | common session/key safe subset; residuals retained |
| M098 | closed | client proxy/outproxy/auth/privacy subset |
| M099 | closed internally — partial | server access/filter/admission/rate subset |
| M100 | closed | transit 15-second source |
| M101 | closed | signed router-news source |
| M102 | closed | v4/v6 network-error owner |
| M103 | closed | by-design-empty banned-peer source |
| M104 | closed as blocked | prior final reclosure stopped on residual option cells |
| M105 | closed | residual primitive/applicability audit |
| M106 | closed | six TCP-client DelayOpen cells |
| M107 | closed | API1/AddressBook/fresh managed-TLS corrective |
| M108 | closed | managed TLS upgrade-permission corrective |
| M109 | closed | startup-managed named lifecycle + `All=true` action semantics; edit/delete contract disposition |
| **M115** | **closed** | M109 runtime-disable isolation, lifecycle state truthfulness, shared startup-client session recovery/lifetime |
| **M110** | **closed** | shared client sessions + destination/key/PrivKeyFile ownership; 31 cells |
| M111 | proposed / dependency-blocked | real Yosemite SAM session-wire option transport; up to 44 cells |
| M112 | proposed / blocked | client proxy and session-lifecycle residuals; up to 62 cells |
| M113 | proposed / blocked | server presentation/address routing/LeaseSet residuals; up to 21 cells |
| M114 | proposed / blocked | zero-residual live/reference interoperability and final reclosure |

M110-M114 were already reserved before the M109 post-closure defects were discovered, so the corrective uses the next free identifier M115. Execution order is M109 → M115 → M110 → M111 → M112 → M113 → M114.

Plans in this current completion line:

- `109-startup-managed-tunnel-action-semantics-corrective.md`
- `110-shared-client-session-and-destination-key-ownership-completion.md`
- `111-sam-session-wire-option-completion.md`
- `112-client-proxy-and-session-lifecycle-residual-completion.md`
- `113-server-presentation-address-routing-and-leaseset-residual-completion.md`
- `114-full-proposal-170-live-interoperability-and-final-reclosure.md`
- `115-m109-runtime-disable-and-lifecycle-truthfulness-corrective-pass.md`

Per `plans/003-planning-process.md`, M110 is closed and no successor is currently registered: M111-M114 remain roadmap-only until their named capability gates are satisfied.

## M109 — historical closed handoff

M109 established neutral startup lifecycle control, named canonical start/stop/restart, mixed `All=true`, and the immutable startup edit/delete ownership disposition without changing the option matrix.

Closure: `plans/closure/i2pcontrol-proposal-170/109-closure.md`.

Post-closure review does not rewrite M109 history; M115 is the required new corrective pass under planning governance.

## M115 — closed corrective handoff

M115 is intentionally narrower than M109. It does not add actions or option support. Closure: `plans/closure/i2pcontrol-proposal-170/115-closure.md`.

Required outcome:

- runtime I2PControl disabled uses the historical startup managers even when the feature is compiled;
- runtime I2PControl enabled retains M109 lifecycle behavior;
- startup state observation never invents `Starting` merely because an internal mutex is contended;
- the controlled startup-client shared Yosemite session is retryable after creation failure;
- active controlled startup clients retain one shared session;
- stopping the final active controlled client releases that session;
- restart/session ownership remains generation-safe;
- no core/util/Yosemite/Cargo/frontend/workflow change;
- M095 is now `255 / 127 / 458`; M115's historical closure remains `224 / 158 / 458`.

Plan: `115-m109-runtime-disable-and-lifecycle-truthfulness-corrective-pass.md`.

## Residual option ownership

M105's historical 164-cell audit is reconciled by the M110 completion ledger. The current 127 cells remain partitioned without overlap:

- M111: 44 — `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions`;
- M112: 62 — proxy/plugin/jump plus post-M106 client lifecycle rows;
- M113: 21 — server presentation/address-routing plus LeaseSet rows.

M115 owns zero residual option cells.

A cell becomes `apply` only with request→real-runtime evidence. A cell becomes `not_applicable` only with affirmative pinned/reference evidence. Difficulty or Java-specific implementation details alone are insufficient.

## M110 — shared sessions and destination/key ownership

M110 is closed by `plans/closure/i2pcontrol-proposal-170/110-closure.md`. It provides the bounded I2PControl-local shared-session/client-secret owner, confined imported-key ownership, and accepted Yosemite 0.7.0 `DestinationKind::Persistent`/`SessionOptions` handoff for all 31 targeted cells.

M115's neutral startup-client session owner must not be generalized into M110's Proposal `Shared` implementation by implication. M110 now owns its separate capability and secret-ownership implementation.

## M111 — SAM session-wire options

M111 is dependency-blocked. The required semantics must reach actual Yosemite session creation through an accepted public dependency API.

No vendored/path/git Yosemite, parallel SAM stack, or Proposal-shaped core API is authorized. If the accepted dependency cannot express a cell, the cell remains blocked and full support remains partial.

## M112 — client proxy/lifecycle residuals

M112 separates portable Proposal contract effects from Java plugin/profile/timer mechanisms. It may add bounded I2PControl generation-local lifecycle policy where real owners exist, but must not build plugin, TLS MITM, router-profile, or global timer frameworks for parity.

Proxy anonymity/trust rules from M093/M098 remain mandatory.

## M113 — server presentation/LeaseSet residuals

M113 is security-sensitive. LeaseSet encryption/client authorization requires real accepted session/LeaseSet primitives and fails closed without downgrade. Presentation/address-routing options may not relax literal-loopback/no-SSRF boundaries merely to reproduce Java local networking mechanisms.

## M114 — final reclosure

M114 remains blocked until M115 and M110-M113 are closed as applicable and M095/M105 contain zero unresolved applicable residuals.

M114 performs no feature implementation. It independently re-verifies exact wire inventory, production owners, mixed startup/control-plane actions, all twelve data planes, applicable option semantics, local/reference runtime evidence, security/anonymity/failure/recovery, and containment.

Only M114 closure may state `full Proposal 170 support against pinned revision 2026-05-20`.

## Security invariants retained

- trusted peer identity is Yosemite-derived;
- server admission remains bounded/transactional;
- server local targets remain confined/literal-loopback under current authority;
- HTTP/IRC filters remain non-bypassable;
- direct I2P proxy traffic never falls through to clearnet DNS;
- clearnet proxy traffic requires explicit I2P outproxy;
- Streamr state remains bounded;
- secret/key/path values remain redacted/confined;
- managed I2PControl key material retains M107/M108 protections;
- LeaseSet security never silently downgrades;
- lifecycle workers/timers remain generation-local and cancellable;
- feature-compiled but runtime-disabled execution must remain on the historical startup path after M115.

## Verification policy

Use focused tests plus the existing feature-gated matrix/containment/live-runtime suite. Do not add a CI farm.

Baseline:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

The historical `m063_feature_reachability` target is absent in the current checkout. The repository also has a known stable/nightly rustfmt mismatch. Record those limitations; do not invent replacement scope or broad formatting churn.

## Closure discipline

Each implementation plan closes only through a closure record containing exact commits/paths, requirement-to-evidence mapping, verification outcomes, security/compatibility/containment review, failure/restart/contention evidence, unresolved findings, next-handoff decision, and internal-only attestation.

Blocked future files may not be executed until the registry names them ready.
