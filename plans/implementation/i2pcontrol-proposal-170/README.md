# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M139 is current runtime/security qualification authority; M140 closed as residual streaming applicability authority; M141 is the sole registered residual handoff**.

Pinned Proposal revision: `2026-05-20` (Open).

Current authorities:

- current runtime/security qualification: M139 closure `plans/closure/i2pcontrol-proposal-170/139-closure.md`;
- current residual streaming applicability: M140 closure `plans/closure/i2pcontrol-proposal-170/140-closure.md` (`325/40/475`, zero promotions, retained `Profile:client` frozen for M143);
- historical runtime/security qualification: M130 closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- historical residual applicability/primitive authority: M131 closure and `131-residual-primitive-map.toml`, superseded only by explicit later current-head closures (M140 for seven cells);
- lifecycle implementation authority: M135/M136/M137/M134 closures;
- current M095 matrix after M140: `325 apply / 40 blocked_primitive / 475 not_applicable`;
- active residual roadmap: `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- registered plan: `141-http-unique-local-source-address-completion.md`.

M140 has zero Proposal promotion budget and authorizes no production code/dependency change. The matrix is `325/40/475` after M140 closure with mechanically proven dispositions.

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

## Current registered handoff — M141

Plan:

- `141-http-unique-local-source-address-completion.md`.

Status: **registered / dependency-ready**.

Scope: exactly two blocked cells — `UniqueLocalAddressPerClient` × `httpserver`, `httpbidirserver`.

Purpose: implement reference-compatible per-client loopback source binding through the canonical peer-hash owner, entirely I2PControl-local.

M140 is closed as complete (`plans/closure/i2pcontrol-proposal-170/140-closure.md`): eight cells adjudicated, seven reclassified to `not_applicable` with affirmative constructor/UDP-ownership evidence, `Profile:client` retained as the exact M143 target set, matrix `325/40/475`, zero promotions.

## Deferred residual implementation chain

All files below are committed for handoff but are **unregistered / non-executable** until their hard dependencies close and the registry promotes them:

| Milestone | Plan | Purpose |
|---|---|---|
| M141 | `141-http-unique-local-source-address-completion.md` | `UniqueLocalAddressPerClient` ×2 through canonical peer-hash loopback source binding |
| M142 | `142-httpclient-sslproxies-and-jumplist-completion.md` | HTTP-only `SSLProxies` + `JumpList` with bounded I2P-only routing/presentation |
| M143 | `143-streaming-profile-runtime-completion.md` | only M140-retained Profile cells; real neutral streaming-window/profile consumer |
| M144 | `144-presentation-usessl-runtime-completion.md` | application/presentation `UseSSL` ×4, separate from management/SAM TLS |
| M145 | `145-leaseset-reply-bundling-multihoming-completion.md` | `MultiHoming`/`shouldBundleReplyInfo` ×2 via real outbound reply-LeaseSet bundling |
| M146 | `146-outproxy-provider-useoutproxyplugin-completion.md` | `UseOutproxyPlugin` ×4 with a real bounded I2P-routed provider |
| M147 | `147-neutral-destination-signature-suite-primitive.md` | zero-promotion neutral signing/key-generation infrastructure |
| M148 | `148-proposal-sigtype-completion.md` | `SigType` ×10 using actual selected destination/signature suites |
| M149 | `149-encrypted-leaseset-runtime-and-encryptleaseset-completion.md` | encrypted LeaseSet runtime + `EncryptLeaseSet` ×5 |
| M150 | `150-leaseset-optionallookup-completion.md` | blinded/secret lookup + `OptionalLookup` ×5 |
| M151 | `151-leaseset-client-auths-completion.md` | PSK/DH client auth as required + `LeaseSetClientAuths` ×5 |
| M152 | `152-final-residual-proposal-170-requalification.md` | zero-promotion final whole-surface qualification and completion decision |

M143, M145 and M147/M149-M151 explicitly require pre-registration amendments with exact retained cells/files/spec/security decisions. Their current candidate path descriptions do not authorize production work.

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

Current machine authority after M140 is M095 `325/40/475` across 840 TunnelManager option/family cells.

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
- M134 six `NewDest` promotions.

Full Proposal 170 support is **not** claimed.

## Current execution graph

```text
M130 historical qualification
  -> M131 residual re-freeze
  -> M135 -> M136 -> M137 -> M134
  -> M139 current integrated qualification
  -> M140 [CLOSED AS COMPLETE — 325/40/475, ZERO PROMOTION]
  -> M141 [REGISTERED / DEPENDENCY-READY]
  -> M142 -> M143 -> M144 -> M145 -> M146
  -> M147 -> M148 -> M149 -> M150 -> M151 -> M152
     [ALL AFTER M141 DEFERRED / UNREGISTERED]
```

## Residual inventory after M140

Machine-derived blockers total 40:

- `SigType` — 10;
- encrypted/authenticated LeaseSets — 15;
- `Profile` (retained `client` only) — 1;
- `UseSSL` — 4;
- `UseOutproxyPlugin` — 4;
- HTTP `SSLProxies` + `JumpList` — 2;
- `UniqueLocalAddressPerClient` — 2 (M141 registered target);
- `MultiHoming` / `shouldBundleReplyInfo` — 2.

## Containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

M140 changes no production source. M141/M142/M144/M146 are designed to remain I2PControl-local. M143/M145 and cryptographic M147/M149-M151 may require neutral lower-layer seams, but their deferred plans require exact-file M061/M062 authorization before registration; no broad `crypto/`, `netdb/`, `i2np/`, `sam/` or other directory waiver is granted by planning.

Yosemite remains exact-pinned through optional `yosemite-i2pcontrol`; no global patch, vendoring, path dependency, floating ref or parallel SAM stack is permitted.

## Internal-only rule

All writes remain internal to `eggstack/emissary` unless a separate explicit maintainer directive authorizes another internal target. External I2P/upstream Emissary/upstream Yosemite resources remain read-only evidence. No plan authorizes upstream submission/review/contact/merge activity.