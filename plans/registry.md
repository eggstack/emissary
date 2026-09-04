# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

Accepted Proposal-170 architecture/security authority:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security.

Pinned Proposal 170 revision: `2026-05-20` (Open).

Authorized internal repositories:

- `eggstack/emissary`;
- `eggstack/yosemite` only under ADR-0005 and Yosemite's own registered plans.

All upstream/third-party repositories and maintainer channels remain read-only.

## Active roadmaps

| Subsystem | Status | Roadmap | Current handoff |
|---|---|---|---|
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | **M131 ready / registered** |
| Post-M114 shared-control-plane corrective line | closed | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | M130 closed as current runtime/security authority |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current registered handoff — M131

Plan:

- `plans/implementation/i2pcontrol-proposal-170/131-residual-applicability-and-primitive-architecture-refreeze.md`

Status: **ready / registered**.

Production-behavior baseline:

- M130 closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`.

M131 is a planning/evidence re-freeze of the 96 blocked TunnelManager cells. It does **not** implement a missing runtime capability.

Required outputs:

- exact mechanical inventory of all 96 starting blocked cells;
- cell-by-cell Proposal/reference applicability review;
- evidence-backed `blocked_primitive` corrections and, where positively proven, `not_applicable` corrections;
- zero `apply` promotions;
- a machine-readable residual primitive/owner/dependency map;
- exact future lower-layer path budgets and security/failure models;
- one next dependency-ready M132+ handoff recommendation/registration at closure, or explicit no-handoff disposition.

M131 must specifically re-freeze:

- `UseOutproxyPlugin`, `SSLProxies`, `JumpList` family applicability;
- all Streamr residual applicability;
- `Profile` streaming semantics;
- I2P-session activity/reduction/close/resume semantics for `Reduce*`, `Close*`, `NewDest`;
- `UniqueLocalAddressPerClient` source-bind semantics and confinement;
- `MultiHoming` versus `shouldBundleReplyInfo` semantics;
- exact `UseSSL` family/direction/identity/trust semantics;
- `SigType` lower-layer crypto/destination ownership;
- encrypted/authenticated LeaseSet ownership below Yosemite Y005.

No production Rust, Cargo/dependency or Yosemite change is authorized by M131.

## Current production/support state

Current implemented-subset runtime/security qualification authority is M130:

- `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- implementation head `fe1a981`;
- closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`.

Current supported surface according to closure evidence:

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence operational;
- all 12 canonical TunnelManager data planes and seven canonical actions exist for the claimed subset;
- all six ClientServicesInfo selectors operational;
- finite one-day API token lifetime with exact expired/unknown distinction;
- bounded JSON-RPC batch semantics with per-element auth and notification suppression;
- managed I2PControl TLS loopback-only; non-loopback requires complete explicit cert/key.

Current M095 matrix at M131 start:

- `284 apply`;
- `96 blocked_primitive`;
- `460 not_applicable`.

Full Proposal 170 status remains **partial**.

## Residual Proposal state

The starting blocked count for M131 is 96, currently summarized as:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells;
- 19 server presentation/routing/LeaseSet cells.

M131 must derive and audit the exact cells mechanically from M095. The partition above is only a cross-check.

No residual cell implementation is registered while M131 is active.

## Dependency graph

```text
M127 token lifetime                    [CLOSED]
  |
  v
M128 JSON-RPC batch                    [CLOSED]
  |
  v
M129 non-loopback TLS fail-closed      [CLOSED]
  |
  v
M130 integrated requalification        [CLOSED — CURRENT RUNTIME AUTHORITY]
  |
  v
M131 residual primitive re-freeze      [READY / REGISTERED]
  |
  +--> at most one evidence-selected M132+ handoff registered at M131 closure
```

M114 remains historically closed as blocked; M131 reopens the full-support residual line, not the closed M127-M130 corrective sequence.

## Canonical scope

This workstream does **not** implement unrelated base I2PControl methods merely for Proposal 170 completion. `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings` and similar unrelated parity remain out of scope.

Shared base behavior is in scope only where required by the implemented extension surface: API-1 authentication/version/token behavior, HTTPS serving, JSON-RPC envelopes/IDs/notifications/batches and protected dispatch.

## Containment rules for M131 and successors

1. Proposal policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
2. Any future production path outside I2PControl requires a neutral canonical owner, exact path budget and separately registered authorization.
3. No Proposal-shaped `emissary-core` API is accepted merely to improve matrix counts.
4. Yosemite remains the sole accepted SAM implementation; no parallel raw SAM stack.
5. Exact Y005 remains isolated behind the optional `yosemite-i2pcontrol` alias.
6. No global patch/path/vendor/floating Yosemite dependency.
7. No direct-clearnet fallback, loopback-confinement weakening, TLS verification bypass or LeaseSet security downgrade.
8. No secrets in RPC/log/Debug/RawConfig planning evidence.
9. No frontend/startup/config rewrite unless separately and explicitly authorized.
10. External/upstream activity remains read-only.

## Registration rules

1. **Only M131 is active/registered.**
2. M131 may describe future M132+ clusters but may not register more than one successor at closure.
3. M131 may correct blocked/not-applicable evidence but may not promote a cell to `apply`.
4. A cell may become `not_applicable` only with affirmative pinned/reference evidence.
5. Matrix-count reduction is not an acceptance criterion.
6. Material scope deviations require a plan/ADR correction before implementation.
7. Closure evidence, not implementation assertions, determines completion.
8. Active documentation must retain partial-support wording while applicable blocked cells remain.

## Recently closed / superseded claims

| Milestone | Disposition |
|---|---|
| M119 | closed |
| M120 | historical; cancellation claim superseded by M123 |
| M121 | closed; semantic truthfulness demotion of `SigType` and `Close`/`CloseTime`/`NewDest` |
| M122 | closed; exact Y004 dependency adoption |
| M123 | closed; commit-phase cancellation/lifecycle atomicity |
| M124 | closed; exact Y005 dependency adoption |
| M125 | closed; two `AllowInternalSSL` cells corrected to not applicable |
| M126 | historical; shared-control-plane clean claim superseded by M130 |
| M127 | closed; finite token lifetime |
| M128 | closed; bounded JSON-RPC batch conformance |
| M129 | closed; non-loopback managed-TLS fail-closed |
| M130 | closed; current implemented-subset requalification authority |
| M131 | **ready / registered**; residual applicability and primitive-architecture re-freeze |

Historical closure files remain unchanged.
