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
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | M146 closed as blocked; no registered successor (M147 deferred); M139 remains current runtime/security qualification authority; M140 closed as residual streaming applicability authority |
| Proposal 170 residual primitive completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md` | **M146 closed as blocked**; M147-M152 deferred/unregistered |
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
- M095 matrix `336 apply / 29 blocked_primitive / 475 not_applicable` (M145 completion; M146 closed as blocked with zero promotions; M144 was `334/31/475`; M143 was `330/35/475`; M142 was `329/36/475`; M141 was `327/38/475`; M140 re-freeze was `325/40/475`; M139-qualified head was `325/47/468`);
- M139 remains the current runtime/security qualification authority; M140 is closed as complete as the current residual streaming applicability authority with zero promotions; M141 is closed as complete as the HTTP unique-local source-address completion with 2 promotions; M142 is closed as complete as the HTTP SSLProxies + JumpList completion with 2 promotions; M143 is closed as complete as the retained streaming Profile completion with 1 promotion; M144 is closed as complete as the presentation UseSSL completion with 4 promotions; M145 is closed as complete as the LeaseSet reply-bundling completion with 2 promotions; M146 is closed as blocked as the outproxy-provider feasibility gate with zero promotions;
- M130 remains historical runtime/security qualification evidence;
- M131 remains historical residual applicability/primitive authority and is superseded where M140 explicitly reclassifies seven cells to `not_applicable` and where M141/M142/M143/M144/M145 promote eleven cells to `apply`.

M139 had zero Proposal promotion budget and changed no production Rust/dependency behavior.

## Registered implementation/qualification handoff

### M141 — HTTP unique local source address completion

Plan:

- `plans/implementation/i2pcontrol-proposal-170/141-http-unique-local-source-address-completion.md`.

Status: **closed as complete**.

Closure: `plans/closure/i2pcontrol-proposal-170/141-closure.md` (2 promotions; `327/38/475`).

Class: capability / local-network confinement.

Scope was exactly two blocked cells, both promoted:

- `UniqueLocalAddressPerClient` × `httpserver`, `httpbidirserver`.

### M142 — HTTP SSLProxies + JumpList completion

Plan:

- `plans/implementation/i2pcontrol-proposal-170/142-httpclient-sslproxies-and-jumplist-completion.md`.

Status: **closed as complete**.

Closure: `plans/closure/i2pcontrol-proposal-170/142-closure.md` (2 promotions; `329/36/475`).

Class: capability / HTTP proxy routing and presentation.

Scope was exactly two blocked cells, both promoted:

- `SSLProxies` × `httpclient`;
- `JumpList` × `httpclient`.

### M143 — retained streaming Profile completion

Plan:

- `plans/implementation/i2pcontrol-proposal-170/143-streaming-profile-runtime-completion.md`.

Status: **closed as complete**.

Closure: `plans/closure/i2pcontrol-proposal-170/143-closure.md` (1 promotion; `330/35/475`).

Class: infrastructure + capability / streaming runtime semantics.

Scope was exactly one retained cell, promoted:

- `Profile` × `client` (plain client only; six other Profile families stay
  `not_applicable` by M140).

No registered successor exists. M145-M152 remain deferred/unregistered;
M143 closure unblocks no future plan for execution (M144 hard dependency
satisfied but still requires its own semantic freeze/registration decision;
file presence alone never authorizes production work).

### M144 — presentation UseSSL completion

Plan:

- `plans/implementation/i2pcontrol-proposal-170/144-presentation-usessl-runtime-completion.md`.

Status: **closed as complete**.

Closure: `plans/closure/i2pcontrol-proposal-170/144-closure.md` (4 promotions; `334/31/475`).

Class: capability / TLS identity and trust.

Scope was exactly four cells, all promoted:

- `UseSSL` × `httpclient`, `connectclient` (TLS listener termination);
- `UseSSL` × `httpserver`, `httpbidirserver` (TLS to loopback target;
  bidir reuses the server owner for its target half).

No registered successor exists. M147-M152 remain deferred/unregistered;
see the future-plan unblock determination in the M146 closure.

### M145 — LeaseSet reply bundling / MultiHoming completion

Plan:

- `plans/implementation/i2pcontrol-proposal-170/145-leaseset-reply-bundling-multihoming-completion.md`.

Status: **closed as complete**.

Closure: `plans/closure/i2pcontrol-proposal-170/145-closure.md` (2 promotions; `336/29/475`).

Class: infrastructure + capability / I2P message privacy and reachability.

Scope was exactly two cells, both promoted:

- `MultiHoming` × `httpserver`, `httpbidirserver` (neutral `shouldBundleReplyInfo`
  policy; omitted/`true` bundles, `false` suppresses `ExistingSession` updates).

No registered successor exists. M147-M152 remain deferred/unregistered;
see the future-plan unblock determination in the M146 closure.

### M146 — Outproxy provider / UseOutproxyPlugin feasibility gate

Plan:

- `plans/implementation/i2pcontrol-proposal-170/146-outproxy-provider-useoutproxyplugin-completion.md`.

Status: **closed as blocked**.

Closure: `plans/closure/i2pcontrol-proposal-170/146-closure.md` (zero promotions;
`336/29/475` unchanged).

Class: infrastructure + capability / proxy routing (feasibility gate).

Scope was the four `UseOutproxyPlugin` cells, all retained as blocked:

- `UseOutproxyPlugin` × `httpclient`, `socks`, `socksirc`, `connectclient`
  (no real bounded I2P-routed local provider in budget; direct-clearnet
  provider prohibited; registry-only provider has zero support value).

No registered successor exists. M147-M152 remain deferred/unregistered;
see the future-plan unblock determination in the M146 closure.

## Deferred residual handoff chain

These plans exist for implementation handoff but are **unregistered and non-executable** until their hard dependencies close and the registry explicitly promotes the next plan (M146 closed as blocked; M147-M152 remain deferred):

| Milestone | Target | Registration constraint |
|---|---|---|
| M142 | HTTP `SSLProxies` + `JumpList` × 2 | **closed as complete** (`329/36/475`); I2P-only HTTP proxy routing/presentation |
| M143 | M140-retained `Profile` × 1 (`Profile:client`) | **closed as complete** (`330/35/475`); neutral streaming-window primitive with pinned interactive→16 mapping |
| M144 | application `UseSSL` × 4 | **closed as complete** (`334/31/475`); application TLS only, distinct from management/SAM TLS |
| M145 | `MultiHoming` / `shouldBundleReplyInfo` × 2 | **closed as complete** (`336/29/475`); neutral reply-bundling policy with handshake retained |
| M146 | `UseOutproxyPlugin` × 4 | **closed as blocked** (`336/29/475` unchanged); no safe I2P-routed provider in budget, direct-clearnet prohibited |
| M147 | neutral destination signature-suite primitive | after M146 closure (satisfied); zero promotions; exact crypto files/algorithm domain required before registration |
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

Current M095 authority after M146 closure (unchanged from M145):

- `336 apply`;
- `29 blocked_primitive`;
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
- M134 six applicable non-Streamr TCP `NewDest` cells;
- M141 two applicable `UniqueLocalAddressPerClient` HTTP server cells;
- M142 two applicable HTTP-client `SSLProxies`/`JumpList` cells;
- M143 one applicable `Profile:client` streaming-profile cell;
- M144 four applicable `UseSSL` presentation-TLS cells;
- M145 two applicable `MultiHoming` reply-bundling cells;
- M146 zero `UseOutproxyPlugin` promotions (four cells remain blocked; no safe provider).

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
M141 HTTP unique local source address          [CLOSED AS COMPLETE — 327/38/475, 2 PROMOTIONS]
  |
  v
M142 HTTP SSLProxies + JumpList                [CLOSED AS COMPLETE — 329/36/475, 2 PROMOTIONS]
  |
  v
M143 retained streaming Profile                 [CLOSED AS COMPLETE — 330/35/475, 1 PROMOTION]
  |
  v
M144 presentation UseSSL                       [CLOSED AS COMPLETE — 334/31/475, 4 PROMOTIONS]
  |
  v
M145 reply LeaseSet bundling / MultiHoming      [CLOSED AS COMPLETE — 336/29/475, 2 PROMOTIONS]
  |
  v
M146 outproxy provider / UseOutproxyPlugin      [CLOSED AS BLOCKED — 336/29/475, ZERO PROMOTIONS]
  |
  v
M147 -> M148 -> M149 -> M150 -> M151 -> M152
[ALL DEFERRED / UNREGISTERED]
```

Numbering note: historical M137/M134 planning used “M138” as a possible NewDest corrective and recorded that it was not needed. No M138 plan was registered.

## Remaining residual clusters after M146 (unchanged from M145)

Machine-derived current residual total is 29:

- `SigType` destination signing — 10;
- encrypted/authenticated LeaseSet cluster — 15;
- `UseOutproxyPlugin` — 4.

M140 reclassified six `Profile` cells and Streamr `ConnectDelay` to `not_applicable` with affirmative evidence. M141 promotes the two `UniqueLocalAddressPerClient` cells to `apply`. M142 promotes the two HTTP-client `SSLProxies`/`JumpList` cells to `apply`. M143 promotes the retained `Profile:client` cell to `apply`. M144 promotes the four `UseSSL` presentation-TLS cells to `apply`. M145 promotes the two `MultiHoming` reply-bundling cells to `apply`. M146 promotes zero `UseOutproxyPlugin` cells (four remain blocked; no safe provider). Full support remains partial.

## Canonical containment rules

1. Proposal/admin policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
2. Existing neutral lower-layer seams remain limited to accepted exact owners and Proposal-free.
3. M140 authorizes no production-source/dependency change.
4. M141 authorizes only `emissary-cli/src/i2pcontrol/backends/http_server.rs` and `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` production changes (shared accepted-handler source bind); no core/Cargo/Yosemite/NetDB/transport/crypto/frontend/startup change.
5. Deferred M147-M152 candidate path budgets are not executable authority.
5. Any future non-I2PControl change requires the relevant plan to be amended/registered with exact-file M061/M062 authority before implementation; no broad crypto/NetDB/I2NP glob/prefix waiver.
6. Yosemite remains the sole accepted SAM implementation; exact Y005 remains optional behind `yosemite-i2pcontrol` unless separately superseded under ADR-0005.
7. No global patch/path/vendor/floating Yosemite dependency.
8. No direct-clearnet fallback, loopback-confinement weakening, TLS verification bypass, LeaseSet security downgrade, or secret leakage.
9. No unrelated base-I2PControl parity or frontend coupling.
10. External/upstream interaction remains read-only.

## Registration rules

1. M139 remains current runtime/security qualification authority; M140 is closed as the current residual streaming applicability authority; M141 is closed as the HTTP unique-local source-address completion; M142 is closed as the HTTP SSLProxies + JumpList completion; M143 is closed as the retained streaming Profile completion; M144 is closed as the presentation UseSSL completion; M145 is closed as the LeaseSet reply-bundling completion; M146 is closed as blocked as the outproxy-provider feasibility gate.
2. No Proposal residual handoff is currently registered; M147-M152 are deferred/unregistered.
3. M147-M152 are deferred/unregistered and may not be executed from their file presence alone.
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
| M141 | **closed as complete**; HTTP unique-local source-address completion; 2 promotions; matrix `327/38/475` |
| M142 | **closed as complete**; HTTP `SSLProxies` + `JumpList` completion; 2 promotions; matrix `329/36/475` |
| M143 | **closed as complete**; retained streaming Profile completion; 1 promotion; matrix `330/35/475` |
| M144 | **closed as complete**; presentation UseSSL completion; 4 promotions; matrix `334/31/475` |
| M145 | **closed as complete**; LeaseSet reply-bundling completion; 2 promotions; matrix `336/29/475` |
| M146 | **closed as blocked**; outproxy-provider feasibility gate; zero promotions; matrix `336/29/475` unchanged |

Historical closure files remain unchanged.