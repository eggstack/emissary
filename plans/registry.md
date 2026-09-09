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
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | execution after M146 is superseded by the post-M146 corrective roadmap |
| Proposal 170 residual primitive completion | **historical through M146 / superseded for next execution** | `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md` | M140-M145 complete; M146 closed blocked |
| Proposal 170 post-M146 corrective | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md` | **M153 registered / dependency-ready**; M154 and M147-M152 deferred |
| Proposal 170 session-lifecycle completion | **closed as complete** | `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md` | M134 closed as complete |
| Post-M114 shared-control-plane corrective line | **closed / historical qualification lineage** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | M130 historical; M139 later requalified lifecycle head |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current production/support state

Current M095 authority after M145 and unchanged by blocked M146:

- `336 apply`;
- `29 blocked_primitive`;
- `475 not_applicable`;
- `840` TunnelManager option/family cells total.

Remaining blockers:

- `SigType` — 10;
- `EncryptLeaseSet` — 5;
- `OptionalLookup` — 5;
- `LeaseSetClientAuths` — 5;
- `UseOutproxyPlugin` — 4.

M140 reclassified six `Profile` cells plus `ConnectDelay:streamrclient` to affirmative `not_applicable`. M141-M145 promoted eleven runtime-backed cells. M146 promoted zero cells and left all four `UseOutproxyPlugin` cells blocked.

Full Proposal 170 status remains **partial**.

## Current qualification authority

### Historical current-head authority — M139

M139 is closed as complete and remains the last whole-surface integrated runtime/security qualification, but it predates M141-M145 production work. It must not be described as a current-head qualification after those changes.

Plan/closure:

- `plans/implementation/i2pcontrol-proposal-170/139-post-lifecycle-integrated-requalification-and-authority-rebase.md`;
- `plans/closure/i2pcontrol-proposal-170/139-closure.md`.

M153 is registered specifically to supersede M139 for current-head runtime/security qualification if its zero-production requalification closes cleanly.

## Registered implementation/qualification handoff

### M153 — post-M146 current-head requalification and authority rebase

Plan:

- `plans/implementation/i2pcontrol-proposal-170/153-post-m146-current-head-requalification-and-authority-rebase.md`.

Status: **registered / dependency-ready**.

Class: invariant / qualification / corrective.

Baseline:

- repository registration baseline `a0c4a791a6a7a974d34eaf93c45330aafd11116f`;
- expected last production-bearing head `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`;
- M095 `336/29/475`;
- M146 closed blocked with no production delta.

M153 has **zero Proposal promotion budget** and **zero production Rust/dependency/Yosemite budget**.

It must:

- mechanically preserve `336/29/475` and exact 29-cell residual identity;
- correct M095 `current_production_head` metadata;
- separate historical milestone assertions from durable current-head test assertions so broad I2PControl testing no longer fails merely from later legitimate matrix deltas;
- requalify M127-M129, lifecycle behavior, and M140-M146 composition on the actual post-M145 production source;
- explicitly absorb the M145 `7cbd80a...` no-std/format follow-up into accepted current production evidence;
- re-prove M061/M062 containment/dependency isolation without widening it;
- become the new current runtime/security qualification authority only if no behavioral or medium/high security defect remains.

Any required production change is a stop condition and requires a separate corrective implementation plan.

## Deferred corrective/implementation chain

### M154 — M147 signature-domain/security/exact-owner re-freeze

Plan:

- `plans/implementation/i2pcontrol-proposal-170/154-m147-signature-domain-security-and-owner-refreeze.md`.

Status: **deferred / unregistered; hard-depends on M153 closure**.

M154 has zero production and zero promotion budget. It exists because M147 itself forbids registration until a dedicated audit freezes:

- exact Proposal/reference SigType domain;
- algorithm security dispositions;
- generate/sign/verify/serialize/persist support per suite;
- exact file owners;
- persistence/migration implications;
- exact dependencies and no-std posture;
- exact-file M061/M062 budget.

M154 closure must amend/register M147, split it, or leave SigType blocked.

### M147-M152

All remain **deferred / unregistered**:

| Milestone | Target | Hard gate |
|---|---|---|
| M147 | neutral destination signature-suite primitive; zero promotions | M154 closure/readiness disposition |
| M148 | `SigType` × 10 | M147 or all required split primitives close |
| M149 | `EncryptLeaseSet` × 5 | M148 closes and exact encrypted-LS owner/security freeze is satisfied |
| M150 | `OptionalLookup` × 5 | M149 closes and exact blinded/secret lookup owner is frozen |
| M151 | `LeaseSetClientAuths` × 5 | M150 closes and auth mode/crypto/interoperability is frozen |
| M152 | final whole-surface requalification; zero promotions | M151 closes; all residuals mechanically recomputed |

File presence and M062 planning bookkeeping do not authorize production work.

## M146 terminal blocked disposition

M146 closure:

- `plans/closure/i2pcontrol-proposal-170/146-closure.md`.

Status: **closed as blocked**.

The four applicable `UseOutproxyPlugin` cells remain blocked because:

- no real bounded local outproxy provider exists in the current architecture;
- a registry/dummy provider has zero observable support value;
- aliasing ordinary `ProxyList` would collapse distinct semantics and be accept-inert;
- a direct OS DNS/TCP clearnet provider violates the accepted egress/security boundary.

M146 is not reopened by M153-M154 or the cryptographic tail. A future provider successor requires a separate architecture/security decision and plan.

If M146 remains blocked through M152, the residual workstream may close as **safe partial / terminal under current policy**, but full Proposal-170 support may not be claimed.

## Current authority / execution chain

```text
M139 post-lifecycle integrated requalification  [CLOSED — HISTORICAL CURRENT-HEAD AUTHORITY]
  |
  v
M140 residual applicability re-freeze           [CLOSED — 325/40/475]
  |
  v
M141 UniqueLocalAddressPerClient                [CLOSED — 327/38/475]
  |
  v
M142 SSLProxies + JumpList                      [CLOSED — 329/36/475]
  |
  v
M143 Profile:client                             [CLOSED — 330/35/475]
  |
  v
M144 presentation UseSSL                        [CLOSED — 334/31/475]
  |
  v
M145 MultiHoming / reply LeaseSet bundling      [CLOSED — 336/29/475]
  |
  +--> 7cbd80a M145 no-std/format follow-up     [ACCEPTED PRODUCTION FOLLOW-UP]
  |
  v
M146 UseOutproxyPlugin feasibility              [CLOSED AS BLOCKED — 336/29/475]
  |
  v
M153 current-head requalification               [REGISTERED — ZERO PROMOTION]
  |
  v
M154 signature-domain/security owner re-freeze  [DEFERRED — ZERO PROMOTION]
  |
  v
M147 -> M148 -> M149 -> M150 -> M151 -> M152    [DEFERRED / UNREGISTERED]
```

## Canonical containment rules

1. Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
2. M143 and M145 neutral lower-layer seams remain limited to exact M061 owners and Proposal-free APIs.
3. Planning-only M062 entries never authorize production paths.
4. No broad `crypto`, `netdb`, `i2np`, `destination`, `primitives` or transport prefix exemption may be introduced for residual security work.
5. Yosemite remains the sole accepted SAM implementation and exact optional Y005 pin unless a separately accepted Yosemite plan supersedes it.
6. No direct-clearnet DNS/TCP fallback may be introduced for proxy/plugin work.
7. No cryptographic fallback/downgrade may be used to claim SigType or LeaseSet-security support.
8. No unrelated base-I2PControl parity, frontend coupling or deferred tunnel-type work.
9. External/upstream interaction remains read-only/internal-only.

## Registration rules

1. M153 is the only registered plan.
2. M154 is not executable until M153 closes complete and the registry explicitly registers M154.
3. M147 cannot be registered directly after M153; M154 must first satisfy its pre-registration audit gate.
4. M148-M152 remain unregistered behind hard dependencies.
5. M146 remains closed blocked and has no implicit successor.
6. Material architecture/path/dependency deviations require a plan amendment before implementation.
7. Closure evidence, not implementation assertions or parser reachability, determines support.

## Recently closed / current lineage

| Milestone | Disposition |
|---|---|
| M139 | closed complete; historical post-lifecycle integrated qualification |
| M140 | closed complete; seven blocked -> N/A; zero promotions |
| M141 | closed complete; 2 promotions |
| M142 | closed complete; 2 promotions |
| M143 | closed complete; 1 promotion |
| M144 | closed complete; 4 promotions |
| M145 | closed complete; 2 promotions; later no-std/format follow-up `7cbd80a...` |
| M146 | closed blocked; zero promotions; four UseOutproxyPlugin cells remain blocked |

Historical closure files remain unchanged.
