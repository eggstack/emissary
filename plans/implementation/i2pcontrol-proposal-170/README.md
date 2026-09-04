# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M135/M136 closed as complete, M137 dependency-ready**.

Pinned Proposal revision: `2026-05-20` (Open).

Current authorities:

- runtime/security: M130 closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- residual applicability/primitive map: M131 closure `plans/closure/i2pcontrol-proposal-170/131-closure.md` and `131-residual-primitive-map.toml`;
- neutral live-quantity primitive: M135 closure `plans/closure/i2pcontrol-proposal-170/135-closure.md`;
- idle reduction: M136 closure `plans/closure/i2pcontrol-proposal-170/136-closure.md`;
- current matrix: `305 apply / 67 blocked_primitive / 468 not_applicable`;
- next handoff: M137 (dependency-ready, registers on its own step).

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

## Closed primitive — M135

Plan:

- `135-neutral-live-tunnel-quantity-and-leaseset-reconfiguration-primitive.md`.

Status: **closed as complete** (`plans/closure/i2pcontrol-proposal-170/135-closure.md`).

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

M135 delivered the neutral lower-layer primitive (desired inbound/outbound
quantities, reference-compatible convergence, dynamic LeaseSet desired count,
bounded destination-scoped coordination) with **zero Proposal matrix
promotions**. Closing matrix at its closure `284/88/468` (now superseded by
M136 `305/67/468`).

Pinned read-only Java lifecycle reference snapshot for M135-M137:

- `i2p/i2p.i2p@2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`.

## Closed corrective — M136

Plan:

- `136-m132-corrective-sam-idle-reduction-and-proposal-reduce-completion.md`.

Status: **closed as complete** (`plans/closure/i2pcontrol-proposal-170/136-closure.md`).

M136 corrects the failed combined M132 vertical slice. After M135 proves the lower primitive, M136 adds one generation-local SAM session activity/timer owner, consumes standard `i2cp.reduceOnIdle`, `i2cp.reduceIdleTime`, `i2cp.reduceQuantity`, then maps Proposal `Reduce`, `ReduceTime`, `ReduceCount` through Yosemite's validated generic additional-session-option path. All 21 client cells (six TCP families plus Streamr) promote to `apply` with end-to-end evidence; servers remain `not_applicable`. Closing matrix `305/67/468`.

## Dependency-ready successor — M137

Plan:

- `137-m133-corrective-sam-idle-close-and-reasoned-termination.md`.

Status: **dependency-ready / unregistered**; M136 closure satisfies its
hard-dependency gate. It registers on its own registration step.

M137 extends the same M136 activity/timer owner with standard `i2cp.closeOnIdle` / `i2cp.closeIdleTime`, exact close-before-reduce ordering, canonical real session teardown, and one neutral authoritative generation-local termination cause. It then maps Proposal `Close`/`CloseTime`.

M137 does not implement `NewDest`.

## NewDest successor

Historical plan:

- `134-newdest-on-proven-idle-resume.md`.

Status: **deferred / unregistered / stale dependency until M137**.

After successful M137 closure, either:

- explicitly amend/rebase M134 against the proven termination/reopen contract; or
- create a corrective M138 if M134 assumptions no longer match.

No NewDest implementation is authorized before that gate.

## Closed predecessor attempts

### M132

- plan `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`;
- closure `plans/closure/i2pcontrol-proposal-170/132-closure.md`;
- status **closed as blocked**, zero promotions and no production implementation.

M132 combined lower-layer reconfiguration, LeaseSet convergence, session activity/timer policy and Proposal translation into one milestone. Its execution lacked direct evidence for several lower-layer behaviors and stopped rather than approximate.

### M133

- plan `133-neutral-sam-idle-close-and-reasoned-termination.md`;
- closure `plans/closure/i2pcontrol-proposal-170/133-closure.md`;
- status **closed as blocked**, zero promotions and no production implementation.

M133 hard-depended on the M132 activity/timer owner, so it remained blocked when M132 produced no such owner.

Historical closures are not rewritten; M135-M137 are corrective successors.

## Current support state

According to current closure authority:

- RouterInfo: 43 additions / 42 available / 1 neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence operational;
- all 12 canonical TunnelManager data planes and seven actions exist for the claimed subset;
- all six ClientServicesInfo selectors operational;
- M127 finite token lifetime, M128 bounded batch conformance and M129 fail-closed non-loopback TLS are requalified by M130;
- M131 corrected eight false applicability blockers and retained 88 genuine primitive blockers at its closure;
- M135 proved the neutral live-quantity primitive with zero promotions;
- M136 promotes 21 `Reduce*` client cells (six TCP families plus Streamr) to `apply`; 67 blocked remain.

Full Proposal 170 support is not claimed.

## Active dependency graph

```text
M130 current runtime/security authority              [CLOSED]
  |
  v
M131 residual primitive re-freeze                    [CLOSED AS BLOCKED — 284/88/468]
  |
  +--> M132 combined reduction attempt               [CLOSED AS BLOCKED]
  |      x
  +--> M133 combined close attempt                   [CLOSED AS BLOCKED]
  |
  v
 M135 neutral live quantity + LeaseSet target       [CLOSED AS COMPLETE]
   |
   v
 M136 M132 corrective Reduce*                       [CLOSED AS COMPLETE — 305/67/468]
  |
  v
M137 M133 corrective Close*                          [DEPENDENCY-READY / UNREGISTERED]
  |
  v
M134 rebased OR M138 corrective NewDest              [FUTURE / UNREGISTERED]
```

## Other residual clusters

Remain unregistered under M131 authority:

- presentation `UseSSL`;
- destination `SigType`;
- outproxy provider/plugin integration;
- HTTP `SSLProxies` and `JumpList`;
- streaming `Profile`;
- retained Streamr residuals such as `ConnectDelay`;
- `UniqueLocalAddressPerClient`;
- `MultiHoming`/`shouldBundleReplyInfo`;
- encrypted/authenticated LeaseSets.

No active lifecycle plan authorizes those areas.

## Containment

Preferred Proposal production ownership remains `emissary-cli/src/i2pcontrol/**`.

M135 is a narrowly registered lower-layer exception limited to the existing tunnel-pool/destination/LeaseSet owners named in that plan. It authorizes no SAM or I2PControl production source and no Cargo/Yosemite changes.

M136/M137 are planning documents only until registered by predecessor closure. Their presence does not authorize production changes.

Core APIs must remain neutral and contain no Proposal/I2PControl business concepts.

Yosemite remains exact-pinned through the optional `yosemite-i2pcontrol` alias; no global patch, vendoring, path dependency, floating ref or parallel raw SAM stack is permitted.

## Verification baseline

Individual plans refine focused tests. Broad implementation verification includes relevant core/CLI checks and tests, M061/M062 containment, M095/M105 matrix guards, live runtime where applicable, clippy, `cargo fmt --all -- --check`, and `git diff --check`.

Known repository-wide stable/nightly rustfmt drift is recorded rather than normalized through unrelated churn.

## Internal-only rule

All writes remain internal to `eggstack/emissary` unless a separate explicit maintainer directive authorizes another internal target. External I2P/upstream Emissary/upstream Yosemite sources are read-only evidence.

No plan authorizes upstream issue/PR/review/contact/submission/release/merge/adoption activity.