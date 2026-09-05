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
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | **M139 ready / registered** |
| Proposal 170 session-lifecycle completion | **closed as complete** | `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md` | M134 closed as complete |
| Post-M114 shared-control-plane corrective line | **closed / historical qualification lineage** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | M130 remains historical current-head authority pending M139 |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Active Proposal-170 implementation handoff

### M139 — post-lifecycle integrated requalification and authority rebase

Plan:

- `plans/implementation/i2pcontrol-proposal-170/139-post-lifecycle-integrated-requalification-and-authority-rebase.md`.

Status: **ready / registered**.

Planning/runtime baseline:

- lifecycle implementation head `e4f217cb1459e26bf011da46b67fc2c83cd192b5`;
- M134/M135/M136/M137 closed as complete;
- current M095 matrix `325 apply / 47 blocked_primitive / 468 not_applicable`;
- M130 remains historical runtime/security qualification authority until M139 closes;
- M131 remains residual applicability/primitive authority.

Objective:

- requalify the entire currently implemented Proposal-170 subset on the post-lifecycle head;
- separate historical M126/M130 assertions from durable current-head guards;
- reconcile stale roadmap/test authority language;
- re-prove M127-M129 shared-control-plane security plus M135-M137/M134 lifecycle composition;
- establish M139 as current runtime/security qualification authority if and only if all current-head evidence is clean.

M139 has **zero Proposal promotion budget** and authorizes **no production Rust, dependency, Yosemite, router, transport, NetDb, crypto, or frontend change**. A production defect is a stop condition requiring a separate corrective plan.

M139 must not register a residual capability successor at closure.

Numbering note: historical M137/M134 planning used “M138” as a possible NewDest corrective and recorded that it was not needed. No M138 plan was registered; this unrelated requalification uses M139 to avoid ambiguity.

## Current production/support state

Current M095 authority:

- `325 apply`;
- `47 blocked_primitive`;
- `468 not_applicable`;
- `840` TunnelManager option/family cells total.

Current qualified/implemented surface includes:

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence;
- all 12 canonical TunnelManager data planes and seven actions for the claimed subset;
- all six ClientServicesInfo selectors;
- M127 finite token lifetime;
- M128 bounded JSON-RPC batch conformance;
- M129 fail-closed non-loopback management TLS;
- M135 neutral live tunnel-quantity / LeaseSet desired-count primitive;
- M136 all 21 applicable `Reduce*` client cells;
- M137 all 14 applicable `Close`/`CloseTime` client cells;
- M134 six applicable non-Streamr TCP `NewDest` cells.

Full Proposal 170 status remains **partial**.

## Current authority chain

```text
M130 integrated requalification                 [CLOSED — HISTORICAL CURRENT-HEAD AUTHORITY]
  |
  v
M131 residual primitive re-freeze               [CLOSED AS BLOCKED — 284/88/468]
  |
  +--> M132 combined reduction attempt          [CLOSED AS BLOCKED]
  +--> M133 combined close attempt              [CLOSED AS BLOCKED]
  |
  v
M135 neutral quantity + LeaseSet primitive      [CLOSED AS COMPLETE — 284/88/468]
  |
  v
M136 Reduce* lifecycle corrective               [CLOSED AS COMPLETE — 305/67/468]
  |
  v
M137 Close* + reasoned termination              [CLOSED AS COMPLETE — 319/53/468]
  |
  v
M134 NewDest proven idle resume                 [CLOSED AS COMPLETE — 325/47/468]
  |
  v
M139 post-lifecycle integrated requalification  [READY / REGISTERED — ZERO PROMOTION]
```

A successful M139 closure supersedes M130 only for current-head runtime/security qualification. Historical M130 closure evidence remains unchanged.

## Remaining residual clusters

All remain **unregistered** under M131 residual authority while M139 runs:

- `SigType` destination signing — 10 cells;
- encrypted/authenticated LeaseSet cluster — 15;
- streaming `Profile` — 7;
- presentation `UseSSL` — 4;
- `UseOutproxyPlugin` — 4;
- HTTP `SSLProxies` + `JumpList` — 2;
- `UniqueLocalAddressPerClient` — 2;
- `MultiHoming` / `shouldBundleReplyInfo` — 2;
- Streamr `ConnectDelay` — 1.

Expected residual total: 47. M139 must mechanically re-derive this from M095 and report discrepancies rather than trusting prose.

## Canonical containment rules

1. Proposal/admin policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
2. M135–M137 neutral lower-layer seams remain limited to their accepted exact owners and must stay Proposal-free.
3. M134 NewDest policy remains I2PControl-owned; its `main.rs` composition seam is limited to wiring one volatile idle-resume tracker to the neutral SAM observation source.
4. M139 authorizes no production-source/dependency change.
5. M061/M062 exact-path/dependency evidence must remain current and at least as strict as before; no broad glob/prefix exception may be added merely to make tests green.
6. Yosemite remains the sole accepted SAM implementation; exact Y005 remains optional behind `yosemite-i2pcontrol`.
7. No global patch/path/vendor/floating Yosemite dependency.
8. No direct-clearnet fallback, loopback-confinement weakening, TLS verification bypass, LeaseSet security downgrade, or secret leakage.
9. No unrelated base-I2PControl parity or frontend coupling.
10. External/upstream interaction remains read-only.

## Registration rules

1. M139 is the **only active Proposal-170 implementation handoff**.
2. No remaining residual capability plan may register while M139 is open.
3. Material path/architecture deviations require plan amendment before implementation.
4. Closure evidence, not implementation assertions, determines support and qualification.
5. M139 must leave no residual capability successor registered even when it closes cleanly; selection is a separate planning decision.
6. Active documentation retains partial-support wording until all applicable residuals are resolved and requalified.

## Recently closed / current lineage

| Milestone | Disposition |
|---|---|
| M127 | closed; finite token lifetime |
| M128 | closed; bounded JSON-RPC batch conformance |
| M129 | closed; non-loopback managed-TLS fail-closed |
| M130 | closed; historical integrated runtime/security qualification authority pending M139 supersession |
| M131 | closed as blocked; residual applicability/primitive re-freeze; matrix `284/88/468` |
| M132 | closed as blocked; zero reduction promotions |
| M133 | closed as blocked; zero close promotions |
| M135 | closed as complete; neutral live-quantity/LeaseSet primitive; zero promotions |
| M136 | closed as complete; 21 `Reduce*` promotions; matrix `305/67/468` |
| M137 | closed as complete; 14 `Close*` promotions; matrix `319/53/468` |
| M134 | closed as complete; six `NewDest` promotions; matrix `325/47/468` |

Historical closure files remain unchanged.