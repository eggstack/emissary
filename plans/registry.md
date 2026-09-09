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
| Proposal 170 post-M146 corrective | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md` | **M154 closed complete (disposition C, M147 path blocked)**; M148-M152 deferred |
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

### Current-head authority — M153

M153 is closed as complete and is the current whole-surface integrated runtime/security qualification at `336/29/475` with zero promotions and zero production changes.

Plan/closure:

- `plans/implementation/i2pcontrol-proposal-170/153-post-m146-current-head-requalification-and-authority-rebase.md`;
- `plans/closure/i2pcontrol-proposal-170/153-closure.md`.

### Historical current-head authority — M139

M139 is closed as complete and was the last whole-surface integrated runtime/security qualification before M141-M145 production work. It is superseded by M153 for current-head purposes and must not be described as a current-head qualification after those changes.

Plan/closure:

- `plans/implementation/i2pcontrol-proposal-170/139-post-lifecycle-integrated-requalification-and-authority-rebase.md`;
- `plans/closure/i2pcontrol-proposal-170/139-closure.md`.

## Registered implementation/qualification handoff

### M153 — post-M146 current-head requalification and authority rebase (closed)

Plan/closure:

- `plans/implementation/i2pcontrol-proposal-170/153-post-m146-current-head-requalification-and-authority-rebase.md`;
- `plans/closure/i2pcontrol-proposal-170/153-closure.md`.

Status: **closed as complete**.

Class: invariant / qualification / corrective.

Baseline:

- repository registration baseline `a0c4a791a6a7a974d34eaf93c45330aafd11116f`;
- accepted last production-bearing head `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`;
- M095 `336/29/475`;
- M146 closed blocked with no production delta.

M153 had **zero Proposal promotion budget** and **zero production Rust/dependency/Yosemite budget**.

It:

- mechanically preserved `336/29/475` and the exact 29-cell residual identity;
- corrected M095 `current_production_head` metadata to `7cbd80a...`;
- separated historical milestone assertions from durable current-head test assertions;
- requalified M127-M129, lifecycle behavior, and M140-M146 composition on the actual post-M145 production source;
- explicitly absorbed the M145 `7cbd80a...` no-std/format follow-up into accepted current production evidence;
- re-proved M061/M062 containment/dependency isolation without widening it;
- superseded M139 as the current runtime/security qualification authority.

## Deferred corrective/implementation chain

### M154 — M147 signature-domain/security/exact-owner re-freeze (closed)

Plan/closure:

- `plans/implementation/i2pcontrol-proposal-170/154-m147-signature-domain-security-and-owner-refreeze.md`;
- `plans/closure/i2pcontrol-proposal-170/154-closure.md`.

Status: **closed as complete; disposition C — M147 path closed as blocked**.

M154 had zero production and zero promotion budget. It froze:

- exact Proposal/reference SigType domain (destination-capable `{0, 1, 2, 3, 7, 11}`);
- algorithm security dispositions (type-0 generation rejected by policy; types 1–3 legacy-only; type 11 missing a maintained primitive);
- generate/sign/verify/serialize/persist support per suite (only type 7 complete);
- exact file owners (planning metadata only — no executable M061/M062 authorisation);
- persistence/migration implications (versioned storage would be required; none designed or authorised);
- exact dependencies and no-std posture (none added).

M154 closure left SigType blocked: no bounded single primitive (A) and no honest split (B) can satisfy the configurable field.

### M147 — neutral destination signature-suite primitive (closed as blocked)

Plan/closure:

- `plans/implementation/i2pcontrol-proposal-170/147-neutral-destination-signature-suite-primitive.md`;
- `plans/closure/i2pcontrol-proposal-170/154-closure.md` (disposition C authority).

Status: **closed as blocked (path)**. No production, dependency, or M061/M062 budget is authorised. Re-opening the path requires a separate explicit architecture/security decision and plan.

### M148-M152

All remain **deferred / unregistered**:

| Milestone | Target | Hard gate |
|---|---|---|
| M148 | `SigType` × 10 | M147 closure-complete — unsatisfiable on this path (blocked-behind-M147; not closed, audit never executes) |
| M149 | `EncryptLeaseSet` × 5 | M148 closes and exact encrypted-LS owner/security freeze is satisfied (re-gating debt recorded in M154 closure §11 — M154 creates no gate) |
| M150 | `OptionalLookup` × 5 | M149 closes and exact blinded/secret lookup owner is frozen |
| M151 | `LeaseSetClientAuths` × 5 | M150 closes and auth mode/crypto/interoperability is frozen |
| M152 | final whole-surface requalification; zero promotions | M151 closes; all residuals mechanically recomputed; terminal state is at best safe-partial (10 SigType + 4 UseOutproxyPlugin blockers) |

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
M153 current-head requalification               [CLOSED — ZERO PROMOTION]
  |
  v
M154 signature-domain/security owner re-freeze  [CLOSED — DISPOSITION C, ZERO PROMOTION]
  |
  v
M147 neutral destination signature primitive    [CLOSED AS BLOCKED (PATH)]
  |
  v
M148 Proposal SigType                            [DEFERRED — BLOCKED BEHIND M147]
  |
  v
M149 -> M150 -> M151 -> M152                     [DEFERRED / UNREGISTERED]
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

1. No plan is currently registered; M154 is closed complete and M147 is closed as blocked (path).
2. M147 cannot be re-registered without a separate explicit architecture/security decision and plan superseding M154 disposition C.
3. M148 remains deferred behind the unsatisfiable M147 gate; M149-M152 remain unregistered behind hard dependencies (M149-M151 need explicit re-gating before any registration).
4. M146 remains closed blocked and has no implicit successor.
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
| M153 | closed complete; zero promotions; current runtime/security qualification authority at `336/29/475` |
| M154 | closed complete; zero promotions; disposition C — M147 path blocked, ten SigType cells terminal under current policy |
| M147 | closed as blocked (path, via M154 disposition C); no production/dependency budget authorised |

Historical closure files remain unchanged.
