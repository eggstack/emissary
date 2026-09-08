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
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | M141 registered; M139 remains current runtime/security qualification authority; M140 closed as residual streaming applicability authority |
| Proposal 170 residual primitive completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md` | **M141 registered / dependency-ready**; M140 closed as complete; M142-M152 deferred/unregistered |
| Proposal 170 session-lifecycle completion | **closed as complete** | `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md` | M134 closed as complete |
| Post-M114 shared-control-plane corrective line | **closed / historical qualification lineage** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | M130 historical; superseded by M139 for current-head qualification |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current Proposal-170 qualification authority

### M139 — post-lifecycle integrated requalification and authority rebase

Plan:

- `plans/implementation/i2pcontrol-proposal-170/139-post-lifecycle-integrated-requalification-and-authority-rebase.md`.

Status: **closed as complete**.

Current qualified baseline:

- lifecycle implementation head `e4f217cb1459e26bf011da46b67fc2c83cd192b5`;
- M134/M135/M136/M137 closed as complete;
- M095 matrix `325 apply / 40 blocked_primitive / 475 not_applicable` (M140 re-freeze; M139-qualified head was `325/47/468`);
- M139 remains the current runtime/security qualification authority; M140 is closed as complete as the current residual streaming applicability authority with zero promotions;
- M130 remains historical runtime/security qualification evidence;
- M131 remains historical residual applicability/primitive authority and is superseded only where M140 explicitly reclassifies seven cells to current-head `not_applicable`.

M139 had zero Proposal promotion budget and changed no production Rust/dependency behavior.

## Registered implementation/qualification handoff

### M141 — HTTP unique local source address completion

Plan:

- `plans/implementation/i2pcontrol-proposal-170/141-http-unique-local-source-address-completion.md`.

Status: **registered / dependency-ready**.

Class: capability / local-network confinement.

Scope is exactly two currently blocked cells:

- `UniqueLocalAddressPerClient` × `httpserver`, `httpbidirserver`.

M141 hard-depends on M140 closure with a reconciled current M095 baseline (`325/40/475`).
Its readiness seams (`TrustedPeerIdentity` before local-target connect,
canonical 32-byte `canonical_id()`, shared HTTP accepted-handler path, and
literal-loopback-only target normalization) were verified present at M140 closure
with no amendment required. M141 has a maximum 2-cell promotion budget and
authorizes only I2PControl-local accepted-stream HTTP data-plane paths; no core,
Cargo/dependency, Yosemite, NetDB, transport, crypto, frontend, or startup-tunnel
change is pre-authorized beyond its exact plan budget.

M140 closure: `plans/closure/i2pcontrol-proposal-170/140-closure.md` (closed as
complete; zero promotions; `325/40/475`; retained `Profile:client` frozen for M143 amendment).

## Deferred residual handoff chain

These plans exist for implementation handoff but are **unregistered and non-executable** until their hard dependencies close and the registry explicitly promotes the next plan:

| Milestone | Target | Registration constraint |
|---|---|---|
| M142 | HTTP `SSLProxies` + `JumpList` × 2 | after M141; I2P-only HTTP proxy routing/presentation |
| M143 | M140-retained `Profile` cells (`Profile:client` × 1, frozen by M140) | after M142; must be amended with exact neutral streaming files (retained set already frozen; amendment still required for exact files before registration) |
| M144 | application `UseSSL` × 4 | after M143; application TLS only, distinct from management/SAM TLS |
| M145 | `MultiHoming` / `shouldBundleReplyInfo` × 2 | after M144; must be amended with exact outbound-message/LeaseSet owner files before registration |
| M146 | `UseOutproxyPlugin` × 4 | after M145; requires a real bounded I2P-routed provider, not an empty registry |
| M147 | neutral destination signature-suite primitive | after M146; zero promotions; exact crypto files/algorithm domain required before registration |
| M148 | `SigType` × 10 | after M147; actual generated identity/signature suite, no fallback |
| M149 | `EncryptLeaseSet` × 5 | after M148; exact encrypted LeaseSet modes/files/spec required before registration |
| M150 | `OptionalLookup` × 5 | after M149; exact blinded/secret lookup + NetDB owner required |
| M151 | `LeaseSetClientAuths` × 5 | after M150; exact auth modes/crypto/client interoperability required |
| M152 | final whole-surface Proposal-170 requalification | after M151; zero promotions / no production changes |

Planning paths:

- `plans/implementation/i2pcontrol-proposal-170/141-http-unique-local-source-address-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/142-httpclient-sslproxies-and-jumplist-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/143-streaming-profile-runtime-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/144-presentation-usessl-runtime-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/145-leaseset-reply-bundling-multihoming-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/146-outproxy-provider-useoutproxyplugin-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/147-neutral-destination-signature-suite-primitive.md`
- `plans/implementation/i2pcontrol-proposal-170/148-proposal-sigtype-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/149-encrypted-leaseset-runtime-and-encryptleaseset-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/150-leaseset-optionallookup-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/151-leaseset-client-auths-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/152-final-residual-proposal-170-requalification.md`

The presence of these files in M062 is planning-only bookkeeping and does not pre-authorize their production path budgets.

## Current production/support state

Current M095 authority after M140 closure:

- `325 apply`;
- `40 blocked_primitive`;
- `475 not_applicable`;
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

## Current authority / execution chain

```text
M130 integrated requalification                 [CLOSED — HISTORICAL]
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
M139 post-lifecycle integrated requalification  [CLOSED AS COMPLETE — ZERO PROMOTION]
  |
  v
M140 residual streaming applicability re-freeze [CLOSED AS COMPLETE — 325/40/475, ZERO PROMOTION]
  |
  v
M141 HTTP unique local source address          [REGISTERED / DEPENDENCY-READY — max 2 promotions]
  |
  v
M142 -> M143 -> M144 -> M145 -> M146 -> M147 -> M148 -> M149 -> M150 -> M151 -> M152
[ALL DEFERRED / UNREGISTERED]
```

Numbering note: historical M137/M134 planning used “M138” as a possible NewDest corrective and recorded that it was not needed. No M138 plan was registered.

## Remaining residual clusters after M140

Machine-derived current residual total is 40:

- `SigType` destination signing — 10;
- encrypted/authenticated LeaseSet cluster — 15;
- streaming `Profile` (retained `client` only) — 1;
- presentation `UseSSL` — 4;
- `UseOutproxyPlugin` — 4;
- HTTP `SSLProxies` + `JumpList` — 2;
- `UniqueLocalAddressPerClient` — 2 (M141 registered target);
- `MultiHoming` / `shouldBundleReplyInfo` — 2.

M140 reclassified six `Profile` cells and Streamr `ConnectDelay` to `not_applicable` with affirmative evidence. Full support remains partial.

## Canonical containment rules

1. Proposal/admin policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
2. Existing neutral lower-layer seams remain limited to accepted exact owners and Proposal-free.
3. M140 authorizes no production-source/dependency change.
4. Deferred M141-M152 candidate path budgets are not executable authority.
5. Any future non-I2PControl change requires the relevant plan to be amended/registered with exact-file M061/M062 authority before implementation; no broad crypto/NetDB/I2NP glob/prefix waiver.
6. Yosemite remains the sole accepted SAM implementation; exact Y005 remains optional behind `yosemite-i2pcontrol` unless separately superseded under ADR-0005.
7. No global patch/path/vendor/floating Yosemite dependency.
8. No direct-clearnet fallback, loopback-confinement weakening, TLS verification bypass, LeaseSet security downgrade, or secret leakage.
9. No unrelated base-I2PControl parity or frontend coupling.
10. External/upstream interaction remains read-only.

## Registration rules

1. M139 remains current runtime/security qualification authority; M140 is closed as the current residual streaming applicability authority.
2. M141 is the sole registered Proposal residual handoff.
3. M142-M152 are deferred/unregistered and may not be executed from their file presence alone.
4. After each closure, register at most the next dependency-ready plan; amend deferred exact-path/security assumptions first where the plan requires it.
5. Material path/architecture deviations require plan amendment before implementation.
6. Closure evidence, not implementation assertions, determines support and qualification.
7. Active documentation retains partial-support wording until M152 closes complete.

## Recently closed / current lineage

| Milestone | Disposition |
|---|---|
| M127 | closed; finite token lifetime |
| M128 | closed; bounded JSON-RPC batch conformance |
| M129 | closed; non-loopback managed-TLS fail-closed |
| M130 | closed; historical integrated runtime/security qualification; superseded by M139 for current head |
| M131 | closed as blocked; residual applicability/primitive re-freeze; matrix `284/88/468` |
| M132 | closed as blocked; zero reduction promotions |
| M133 | closed as blocked; zero close promotions |
| M135 | closed as complete; neutral live-quantity/LeaseSet primitive; zero promotions |
| M136 | closed as complete; 21 `Reduce*` promotions; matrix `305/67/468` |
| M137 | closed as complete; 14 `Close*` promotions; matrix `319/53/468` |
| M134 | closed as complete; six `NewDest` promotions; matrix `325/47/468` |
| M139 | closed as complete; current post-lifecycle runtime/security qualification; zero promotions; matrix `325/47/468` at M139 head (superseded for residual counts by M140) |
| M140 | **closed as complete**; residual streaming applicability re-freeze; zero promotions; matrix `325/40/475`; retained `Profile:client` frozen for M143 |
| M141 | **registered / dependency-ready**; `UniqueLocalAddressPerClient` × 2; max 2 promotions |

Historical closure files remain unchanged.