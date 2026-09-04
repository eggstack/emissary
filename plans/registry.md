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
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | **M135 ready / registered** |
| Proposal 170 session-lifecycle completion | **active / corrective** | `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md` | **M135 ready / registered**; M136/M137 deferred |
| Post-M114 shared-control-plane corrective line | closed | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | M130 current runtime/security authority |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Active Proposal-170 implementation handoff

### M135 — neutral live tunnel quantity and LeaseSet reconfiguration primitive

Plan:

- `plans/implementation/i2pcontrol-proposal-170/135-neutral-live-tunnel-quantity-and-leaseset-reconfiguration-primitive.md`.

Status: **ready / registered**.

Baseline:

- pre-M135 planning head `517decf733352dfc2bf24ad349c5ab4cf9315742`;
- current matrix `284 apply / 88 blocked_primitive / 468 not_applicable`;
- M132/M133 are closed as blocked and remain immutable historical evidence.

Objective:

- establish only the generic lower-layer current desired inbound/outbound tunnel quantity plus dynamic LeaseSet desired inbound count;
- preserve immutable base quantities, normal excess-tunnel expiry/failure convergence and backup semantics;
- coordinate pool/LeaseSet target updates at the destination owner;
- create no SAM idle policy and no Proposal support promotion.

M135 **must close with M095 unchanged at `284/88/468`**. Any Proposal cell promotion under M135 is a plan violation.

Direct read-only Java reference authority for M135-M137:

- `i2p/i2p.i2p@2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`.

The corrective reference freeze establishes that Java quantity reduction is a live client-pool settings reconfiguration, existing excess tunnels remain usable until normal lifecycle removal, future build demand follows the new quantity, and LeaseSet wanted count follows current inbound quantity.

## Deferred corrective successors

### M136 — M132 corrective: SAM idle reduction and Proposal Reduce completion

Plan:

- `plans/implementation/i2pcontrol-proposal-170/136-m132-corrective-sam-idle-reduction-and-proposal-reduce-completion.md`.

Status: **deferred / unregistered**; hard-depends on successful M135 closure.

M136 may register only if M135 proves:

- stable destination-scoped target update/restore;
- reference-compatible excess-tunnel convergence;
- correct dynamic LeaseSet desired count;
- bounded generation-local control;
- unchanged matrix and clean containment;
- no unresolved high/medium primitive defect.

If registered later, M136 adds one canonical SAM session activity/timer owner and standard `i2cp.reduce*` consumption, then maps Proposal `Reduce`/`ReduceTime`/`ReduceCount` through Yosemite's validated generic session-option path. Direct reference supports Streamr session-level applicability, but matrix promotion remains evidence-driven.

### M137 — M133 corrective: SAM idle close and reasoned termination

Plan:

- `plans/implementation/i2pcontrol-proposal-170/137-m133-corrective-sam-idle-close-and-reasoned-termination.md`.

Status: **deferred / unregistered**; hard-depends on successful M136 closure.

M137 extends the exact M136 activity/timer owner with standard close-on-idle behavior, canonical session teardown and a neutral authoritative in-process termination cause. It may then map Proposal `Close`/`CloseTime`.

M137 does not implement `NewDest`.

### NewDest successor

Historical plan:

- `plans/implementation/i2pcontrol-proposal-170/134-newdest-on-proven-idle-resume.md`.

Status: **deferred / unregistered / requires explicit rebase after M137**.

After successful M137 closure, M134 may be amended/rebased only if its assumptions match the proven termination/reopen interface. Otherwise create a corrective M138. No NewDest execution is authorized now.

## Closed failed predecessor line

### M132

Plan/closure:

- `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`;
- `plans/closure/i2pcontrol-proposal-170/132-closure.md`.

Status: **closed as blocked**, zero promotions, no production implementation.

M132 correctly stopped because its combined vertical slice could not freeze excess-tunnel, LeaseSet and Streamr reference behavior in that execution and its path budget did not provide an independently proven lower-layer primitive.

### M133

Plan/closure:

- `133-neutral-sam-idle-close-and-reasoned-termination.md`;
- `plans/closure/i2pcontrol-proposal-170/133-closure.md`.

Status: **closed as blocked**, zero promotions, no production implementation.

M133 was blocked because M132 never produced the canonical activity/timer owner it hard-depended on.

Historical closures remain unchanged; M135-M137 are explicit correctives, not revisions of closure evidence.

## Current production/support state

Current implemented-subset runtime/security authority remains M130:

- closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- implementation head `fe1a981`;
- closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`.

M131 remains residual applicability/primitive authority:

- closure `plans/closure/i2pcontrol-proposal-170/131-closure.md`;
- primitive map `plans/implementation/i2pcontrol-proposal-170/131-residual-primitive-map.toml`;
- closure head `3a829d7d3d6314ecf09e42dbf0339506f0917c96`.

Current supported surface according to closure evidence:

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence operational;
- all 12 canonical TunnelManager data planes and seven actions exist for the claimed subset;
- all six ClientServicesInfo selectors operational;
- finite API token lifetime, bounded JSON-RPC batches and fail-closed non-loopback management TLS qualified by M127-M130.

Current M095 matrix:

- `284 apply`;
- `88 blocked_primitive`;
- `468 not_applicable`.

Full Proposal 170 status remains **partial**.

## Dependency graph

```text
M130 integrated requalification                    [CLOSED — CURRENT RUNTIME AUTHORITY]
  |
  v
M131 residual primitive re-freeze                  [CLOSED AS BLOCKED — 284/88/468]
  |
  +--> M132 combined reduction attempt             [CLOSED AS BLOCKED]
  |      x
  +--> M133 combined close attempt                 [CLOSED AS BLOCKED]
  |
  v
M135 neutral live quantity + LeaseSet target       [READY / REGISTERED]
  |
  v
M136 M132 corrective Reduce*                       [DEFERRED / UNREGISTERED]
  |
  v
M137 M133 corrective Close*                        [DEFERRED / UNREGISTERED]
  |
  v
M134 rebased OR M138 NewDest corrective            [FUTURE / UNREGISTERED]
```

## Residual clusters outside the active line

Remain unregistered under M131 authority:

- presentation `UseSSL`;
- `SigType` destination signing;
- `UseOutproxyPlugin`;
- HTTP `SSLProxies` / `JumpList`;
- streaming `Profile`;
- Streamr `ConnectDelay` if still residual;
- `UniqueLocalAddressPerClient`;
- `MultiHoming` / `shouldBundleReplyInfo`;
- encrypted/authenticated LeaseSets.

No successor from these clusters may be smuggled into M135-M137.

## Canonical containment rules

1. Proposal policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
2. M135 is an explicit neutral lower-layer exception limited to its named tunnel-pool/destination/LeaseSet owners.
3. M135 authorizes no SAM or I2PControl production change and no matrix promotion.
4. M136/M137 remain unregistered and authorize no production work until their dependency gates close.
5. No Proposal-shaped `emissary-core` API is accepted merely to improve matrix counts.
6. Yosemite remains the sole accepted SAM implementation; no parallel raw SAM stack.
7. Exact Y005 remains isolated behind optional `yosemite-i2pcontrol`.
8. No global patch/path/vendor/floating Yosemite dependency.
9. No direct-clearnet fallback, loopback-confinement weakening, TLS verification bypass or LeaseSet security downgrade.
10. No secrets in RPC/log/Debug/planning evidence.
11. No frontend/startup/config rewrite unless separately authorized.
12. External/upstream activity remains read-only.

## Registration rules

1. **M135 is the sole active/registered Proposal-170 implementation plan.**
2. M136 registers only after successful M135 closure explicitly proves its readiness contract.
3. M137 registers only after successful M136 closure explicitly proves its readiness contract.
4. NewDest registers only after successful M137 closure proves an authoritative idle-close/reopen contract.
5. Material path/architecture deviations require plan amendment before code.
6. Closure evidence, not implementation assertions, determines support/matrix promotion.
7. Active documentation retains partial-support wording until all applicable residuals are resolved and requalified.

## Recently closed / current authority

| Milestone | Disposition |
|---|---|
| M121 | closed; semantic truthfulness demotion of `SigType` and `Close`/`CloseTime`/`NewDest` |
| M123 | closed; commit-phase cancellation/lifecycle atomicity |
| M125 | closed; two `AllowInternalSSL` cells corrected to not applicable |
| M127 | closed; finite token lifetime |
| M128 | closed; bounded JSON-RPC batch conformance |
| M129 | closed; non-loopback managed-TLS fail-closed |
| M130 | closed; current implemented-subset runtime/security qualification |
| M131 | closed as blocked; residual applicability/primitive re-freeze; matrix 284/88/468 |
| M132 | closed as blocked; zero reduction promotions |
| M133 | closed as blocked; zero close promotions |
| M135 | **ready / registered**; neutral live-quantity/LeaseSet primitive, zero-promotion requirement |

Historical closure files remain unchanged.