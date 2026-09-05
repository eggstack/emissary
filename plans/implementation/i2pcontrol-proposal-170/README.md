# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M139 ready / registered for post-lifecycle integrated requalification**.

Pinned Proposal revision: `2026-05-20` (Open).

Current authorities and handoff:

- historical runtime/security qualification: M130 closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- residual applicability/primitive authority: M131 closure `plans/closure/i2pcontrol-proposal-170/131-closure.md` and `131-residual-primitive-map.toml`;
- lifecycle implementation authority: M135/M136/M137/M134 closures;
- current M095 matrix: `325 apply / 47 blocked_primitive / 468 not_applicable`;
- **current registered handoff: M139** `139-post-lifecycle-integrated-requalification-and-authority-rebase.md`.

M139 is expected to replace M130 as current-head runtime/security qualification authority only if its current-head verification closes cleanly. It has zero Proposal promotion budget and authorizes no production Rust/dependency/Yosemite change.

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

## Active handoff — M139

Plan:

- `139-post-lifecycle-integrated-requalification-and-authority-rebase.md`.

Status: **ready / registered**.

Purpose:

- mechanically requalify the current `325/47/468` head;
- correct stale current-head assertions in historical M126/M130-era tests without erasing historical milestone facts;
- re-prove M127 finite token lifetime, M128 bounded JSON-RPC batches and M129 fail-closed TLS on the lifecycle head;
- re-prove M135 quantity/LeaseSet, M136 Reduce*, M137 Close*/reason and M134 NewDest as one integrated lifecycle composition;
- reconcile the registry, roadmaps, support docs and durable current-head guards;
- make M139 the current runtime/security qualification authority only on clean closure.

M139 is test/documentation/qualification work. Production-source changes are a stop condition requiring a separate corrective plan.

M139 must not register the next residual capability cluster.

Numbering note: M137/M134 historical documents contemplated an optional NewDest-corrective “M138” and recorded it as unnecessary. No M138 implementation plan was registered. The unrelated requalification therefore uses M139.

## Closed lifecycle chain

### M135 — neutral live quantity + LeaseSet desired count

- plan `135-neutral-live-tunnel-quantity-and-leaseset-reconfiguration-primitive.md`;
- closure `plans/closure/i2pcontrol-proposal-170/135-closure.md`;
- closed as complete with zero Proposal promotions; matrix remained `284/88/468` at that closure.

M135 established immutable base quantities plus bounded destination-scoped live desired quantities, reference-compatible excess-tunnel convergence and dynamic LeaseSet desired inbound count.

### M136 — M132 corrective: Reduce*

- plan `136-m132-corrective-sam-idle-reduction-and-proposal-reduce-completion.md`;
- closure `plans/closure/i2pcontrol-proposal-170/136-closure.md`;
- closed as complete; 21 client `Reduce`/`ReduceCount`/`ReduceTime` cells promoted; matrix `305/67/468`.

### M137 — M133 corrective: Close* + reasoned termination

- plan `137-m133-corrective-sam-idle-close-and-reasoned-termination.md`;
- closure `plans/closure/i2pcontrol-proposal-170/137-closure.md`;
- closed as complete; 14 client `Close`/`CloseTime` cells promoted; matrix `319/53/468`.

### M134 — NewDest on proven idle resume

- plan `134-newdest-on-proven-idle-resume.md`;
- closure `plans/closure/i2pcontrol-proposal-170/134-closure.md`;
- closed as complete; six non-Streamr TCP `NewDest` cells promoted; matrix `325/47/468`.

M134 kept identity rotation I2PControl-owned and consumes only the authoritative M137 `IdlePolicy` lifecycle fact. Streamr NewDest remains not applicable.

## Historical blocked predecessor attempts

- M132 `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md` — closed as blocked, zero promotions;
- M133 `133-neutral-sam-idle-close-and-reasoned-termination.md` — closed as blocked, zero promotions.

Historical closure files are immutable. M135-M137 are corrective successors, not edits to those closures.

## Current support state

Current machine authority is M095 `325/47/468` across 840 TunnelManager option/family cells.

Qualified/implemented surface includes:

- RouterInfo: 43 additions / 42 available / 1 neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence;
- all 12 canonical TunnelManager data planes and seven actions for the claimed subset;
- all six ClientServicesInfo selectors;
- M127 finite token lifetime;
- M128 bounded batch conformance;
- M129 fail-closed non-loopback TLS;
- M135 neutral live quantity/LeaseSet primitive;
- M136 21 `Reduce*` promotions;
- M137 14 `Close*` promotions;
- M134 six `NewDest` promotions.

Full Proposal 170 support is **not** claimed.

## Dependency graph

```text
M130 integrated requalification                [CLOSED — HISTORICAL CURRENT-HEAD AUTHORITY]
  |
  v
M131 residual primitive re-freeze              [CLOSED AS BLOCKED — 284/88/468]
  |
  +--> M132 combined reduction attempt         [CLOSED AS BLOCKED]
  +--> M133 combined close attempt             [CLOSED AS BLOCKED]
  |
  v
M135 neutral quantity + LeaseSet primitive     [CLOSED AS COMPLETE]
  |
  v
M136 Reduce* corrective                        [CLOSED AS COMPLETE — 305/67/468]
  |
  v
M137 Close* corrective                         [CLOSED AS COMPLETE — 319/53/468]
  |
  v
M134 NewDest proven idle resume                [CLOSED AS COMPLETE — 325/47/468]
  |
  v
M139 post-lifecycle integrated requalification [READY / REGISTERED — ZERO PROMOTION]
```

## Remaining residual clusters

Remain unregistered under M131 while M139 runs:

- `SigType` destination signing — 10;
- encrypted/authenticated LeaseSets — 15;
- streaming `Profile` — 7;
- presentation `UseSSL` — 4;
- `UseOutproxyPlugin` — 4;
- HTTP `SSLProxies` + `JumpList` — 2;
- `UniqueLocalAddressPerClient` — 2;
- `MultiHoming` / `shouldBundleReplyInfo` — 2;
- Streamr `ConnectDelay` — 1.

Expected total: 47, to be mechanically re-derived by M139.

## Containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

M135–M137 are accepted neutral lower-layer exceptions only in their exact named owners. M134 is I2PControl-owned except the minimal application-composition seam required to share its volatile tracker with the neutral SAM observation source. M139 authorizes no new production-source seam.

Yosemite remains exact-pinned through optional `yosemite-i2pcontrol`; no global patch, vendoring, path dependency, floating ref or parallel SAM stack is permitted.

## Internal-only rule

All writes remain internal to `eggstack/emissary` unless a separate explicit maintainer directive authorizes another internal target. External I2P/upstream Emissary/upstream Yosemite resources remain read-only evidence. No plan authorizes upstream submission/review/contact/merge activity.