# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M139 is current runtime/security qualification authority; M140 closed as residual streaming applicability authority; M141 closed as HTTP unique-local source-address completion; M142 closed as HTTP SSLProxies + JumpList completion; M143 closed as retained streaming Profile completion; M144 closed as presentation UseSSL completion; M145 closed as LeaseSet reply-bundling completion; no registered successor**.

Pinned Proposal revision: `2026-05-20` (Open).

Current authorities:

- current runtime/security qualification: M139 closure `plans/closure/i2pcontrol-proposal-170/139-closure.md`;
- current residual streaming applicability: M140 closure `plans/closure/i2pcontrol-proposal-170/140-closure.md` (`325/40/475`, zero promotions, retained `Profile:client` frozen for M143);
- HTTP unique-local source-address completion: M141 closure `plans/closure/i2pcontrol-proposal-170/141-closure.md` (`327/38/475`, 2 promotions);
- HTTP SSLProxies + JumpList completion: M142 closure `plans/closure/i2pcontrol-proposal-170/142-closure.md` (`329/36/475`, 2 promotions);
- retained streaming Profile completion: M143 closure `plans/closure/i2pcontrol-proposal-170/143-closure.md` (`330/35/475`, 1 promotion);
- presentation UseSSL completion: M144 closure `plans/closure/i2pcontrol-proposal-170/144-closure.md` (`334/31/475`, 4 promotions);
- LeaseSet reply-bundling completion: M145 closure `plans/closure/i2pcontrol-proposal-170/145-closure.md` (`336/29/475`, 2 promotions);
- historical runtime/security qualification: M130 closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- historical residual applicability/primitive authority: M131 closure and `131-residual-primitive-map.toml`, superseded where M140 reclassifies seven cells and where M141/M142/M143/M144/M145 promote eleven cells;
- lifecycle implementation authority: M135/M136/M137/M134 closures;
- current M095 matrix after M145: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- active residual roadmap: `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- registered plan: none (M145 closed; M146-M152 deferred).

M140 has zero Proposal promotion budget and authorizes no production code/dependency change. The matrix was `325/40/475` after M140 closure. M141 promotes 2 cells to `327/38/475` with I2PControl-local accepted-handler changes only. M142 promotes 2 cells to `329/36/475` with I2PControl-local HTTP-client changes only. M143 promotes 1 cell to `330/35/475` with the neutral streaming-window primitive (core streaming owners + I2PControl Profile mapping). M144 promotes 4 cells to `334/31/475` with I2PControl-local presentation TLS (listener/target owners + shared helper). M145 promotes 2 cells to `336/29/475` with the neutral reply-bundling primitive (core SessionManager owner + I2PControl MultiHoming mapping).

## Authority order

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. subsystem roadmaps;
7. `plans/registry.md`;
8. the specific registered plan.

Containment/support evidence remains centered on:

- `061-containment-boundary.toml`;
- `062-dependency-containment.toml`;
- `095-full-support-matrix.toml`;
- `105-residual-option-audit.toml`;
- `110-completion-ledger.toml`.

## Current registered handoff — none (M145 closed)

Last closed plan:

- `145-leaseset-reply-bundling-multihoming-completion.md`.

Status: **closed as complete**.

Scope was exactly two cells, both promoted — `MultiHoming` × `httpserver`, `httpbidirserver`.

Purpose: neutral reply LeaseSet bundling via `shouldBundleReplyInfo` with handshake retained.

M141 is closed as complete (`plans/closure/i2pcontrol-proposal-170/141-closure.md`): two `UniqueLocalAddressPerClient` cells promoted to `apply` with reference-compatible source-bind evidence, matrix `327/38/475`. M142 is closed as complete (`plans/closure/i2pcontrol-proposal-170/142-closure.md`): two HTTP-client cells promoted to `apply` with bounded I2P-only routing/presentation evidence, matrix `329/36/475`. M143 is closed as complete (`plans/closure/i2pcontrol-proposal-170/143-closure.md`): one retained `Profile:client` cell promoted to `apply` with the neutral streaming-window primitive, matrix `330/35/475`. M144 is closed as complete (`plans/closure/i2pcontrol-proposal-170/144-closure.md`): four `UseSSL` cells promoted to `apply` with I2PControl-local presentation TLS, matrix `334/31/475`. M145 is closed as complete (`plans/closure/i2pcontrol-proposal-170/145-closure.md`): two `MultiHoming` cells promoted to `apply` with the neutral reply-bundling primitive, matrix `336/29/475`. No successor is registered; M146-M152 remain deferred.

## Deferred residual implementation chain

All files below are committed for handoff but are **unregistered / non-executable** until their hard dependencies close and the registry promotes them:

| Milestone | Plan | Purpose |
|---|---|---|
| M141 | `141-http-unique-local-source-address-completion.md` | `UniqueLocalAddressPerClient` ×2 through canonical peer-hash loopback source binding — **closed as complete** |
| M142 | `142-httpclient-sslproxies-and-jumplist-completion.md` | HTTP-only `SSLProxies` + `JumpList` with bounded I2P-only routing/presentation — **closed as complete** |
| M143 | `143-streaming-profile-runtime-completion.md` | only M140-retained `Profile:client`; real neutral streaming-window consumer — **closed as complete** |
| M144 | `144-presentation-usessl-runtime-completion.md` | application/presentation `UseSSL` ×4, separate from management/SAM TLS — **closed as complete** |
| M145 | `145-leaseset-reply-bundling-multihoming-completion.md` | `MultiHoming`/`shouldBundleReplyInfo` ×2 via real outbound reply-LeaseSet bundling — **closed as complete** |
| M146 | `146-outproxy-provider-useoutproxyplugin-completion.md` | `UseOutproxyPlugin` ×4 with a real bounded I2P-routed provider |
| M147 | `147-neutral-destination-signature-suite-primitive.md` | zero-promotion neutral signing/key-generation infrastructure |
| M148 | `148-proposal-sigtype-completion.md` | `SigType` ×10 using actual selected destination/signature suites |
| M149 | `149-encrypted-leaseset-runtime-and-encryptleaseset-completion.md` | encrypted LeaseSet runtime + `EncryptLeaseSet` ×5 |
| M150 | `150-leaseset-optionallookup-completion.md` | blinded/secret lookup + `OptionalLookup` ×5 |
| M151 | `151-leaseset-client-auths-completion.md` | PSK/DH client auth as required + `LeaseSetClientAuths` ×5 |
| M152 | `152-final-residual-proposal-170-requalification.md` | zero-promotion final whole-surface qualification and completion decision |

M143, M147/M149-M151 explicitly require pre-registration amendments with exact retained cells/files/spec/security decisions. Their current candidate path descriptions do not authorize production work. M145 was amended via M145-AMEND-01 before registration.

## Closed qualification / lifecycle chain

### M139 — post-lifecycle integrated requalification

- plan `139-post-lifecycle-integrated-requalification-and-authority-rebase.md`;
- closure `plans/closure/i2pcontrol-proposal-170/139-closure.md`;
- closed as complete; zero promotions; current runtime/security qualification authority at `325/47/468`.

### M135 — neutral live quantity + LeaseSet desired count

- closed as complete with zero Proposal promotions; established reference-compatible live desired quantities and dynamic LeaseSet desired inbound count.

### M136 — Reduce*

- closed as complete; 21 promotions; matrix `305/67/468`.

### M137 — Close* + reasoned termination

- closed as complete; 14 promotions; matrix `319/53/468`.

### M134 — NewDest on proven idle resume

- closed as complete; six promotions; matrix `325/47/468`.

Historical M132/M133 remain closed as blocked and their closure files are unchanged.

Numbering note: historical planning contemplated an optional NewDest-corrective “M138” and recorded it as unnecessary. No M138 implementation plan was registered.

## Current support state

Current machine authority after M145 is M095 `336/29/475` across 840 TunnelManager option/family cells.

Qualified/implemented surface includes:

- RouterInfo: 43 additions / 42 available / 1 neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence;
- all 12 canonical TunnelManager data planes and seven actions for the claimed subset;
- all six ClientServicesInfo selectors;
- M127 finite token lifetime;
- M128 bounded batch conformance;
- M129 fail-closed non-loopback management TLS;
- M135 neutral live quantity/LeaseSet primitive;
- M136 21 `Reduce*` promotions;
- M137 14 `Close*` promotions;
- M134 six `NewDest` promotions;
- M141 two `UniqueLocalAddressPerClient` promotions;
- M142 two `SSLProxies`/`JumpList` promotions;
- M143 one `Profile:client` promotion;
- M144 four `UseSSL` promotions;
- M145 two `MultiHoming` promotions.

Full Proposal 170 support is **not** claimed.

## Current execution graph

```text
M130 historical qualification
  -> M131 residual re-freeze
  -> M135 -> M136 -> M137 -> M134
  -> M139 current integrated qualification
  -> M140 [CLOSED AS COMPLETE — 325/40/475, ZERO PROMOTION]
  -> M141 [CLOSED AS COMPLETE — 327/38/475, 2 PROMOTIONS]
  -> M142 [CLOSED AS COMPLETE — 329/36/475, 2 PROMOTIONS]
  -> M143 [CLOSED AS COMPLETE — 330/35/475, 1 PROMOTION]
  -> M144 [CLOSED AS COMPLETE — 334/31/475, 4 PROMOTIONS]
  -> M145 [CLOSED AS COMPLETE — 336/29/475, 2 PROMOTIONS]
  -> M146
  -> M147 -> M148 -> M149 -> M150 -> M151 -> M152
     [ALL AFTER M145 DEFERRED / UNREGISTERED]
```

## Residual inventory after M145

Machine-derived blockers total 29:

- `SigType` — 10;
- encrypted/authenticated LeaseSets — 15;
- `UseOutproxyPlugin` — 4.

## Containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

M140 changes no production source. M141/M142/M144/M146 are designed to remain I2PControl-local. M143 and cryptographic M147/M149-M151 may require neutral lower-layer seams, but their deferred plans require exact-file M061/M062 authorization before registration; no broad `crypto/`, `netdb/`, `i2np/`, `sam/` or other directory waiver is granted by planning. M145 required one exact neutral file (`destination/session/mod.rs`) plus two already-allowed core owners, authorized via M145-AMEND-01.

Yosemite remains exact-pinned through optional `yosemite-i2pcontrol`; no global patch, vendoring, path dependency, floating ref or parallel SAM stack is permitted.

## Internal-only rule

All writes remain internal to `eggstack/emissary` unless a separate explicit maintainer directive authorizes another internal target. External I2P/upstream Emissary/upstream Yosemite resources remain read-only evidence. No plan authorizes upstream submission/review/contact/merge activity.