# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

Accepted Proposal 170 architecture decisions:

- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`
- `plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`
- `plans/adrs/ADR-0005-internal-yosemite-fork-dependency-boundary.md`

Pinned Proposal 170 revision: `2026-05-20` (proposal remains Open).

Authorized internal repositories for the dependency-completion/corrective line:

- `eggstack/emissary`;
- `eggstack/yosemite`, only under ADR-0005 and its own registered plans.

All upstream/third-party repositories and maintainer channels remain read-only.

## Status vocabulary

- **proposed** — document exists but is not approved for execution;
- **ready** — dependencies/interfaces are satisfied and the plan may be handed off;
- **active** — implementation or closure work is in progress;
- **blocked** — a named dependency/evidence requirement prevents execution;
- **closing** — implementation landed and closure evidence is being gathered;
- **closed** — closure accepted for the pinned implementation head;
- **closed as blocked** — authorized review/safe subset completed but named capability blockers remain;
- **corrective pass required** — later evidence invalidated a material prior claim.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Blocker/next transition |
|---|---|---|---|---|
| I2PControl Proposal 170 full-support completion | active / partial | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | historical M114 closed as blocked | corrective line below must close before another final reclosure |
| Proposal 170 post-M114 corrective line | **active** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | **M120 ready** | M121 → M122; Y004 proceeds separately in Yosemite |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority | every corrective must retain exact path/dependency ownership |
| I2PControl tunnel security | closed at M093 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | regression authority | M119/M120/M121 must preserve tunnel/secret/proxy boundaries |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational.
- All 12 TunnelManager data planes and seven canonical action handlers exist.
- All six ClientServicesInfo selectors are operational.
- API 1-only negotiation and M107/M108 managed TLS hardening are operational.
- M109/M115 startup lifecycle and M110/M116 shared-session/destination ownership corrective line are closed.
- M117 exact I2PControl-only Yosemite fork integration is closed at Y002 revision `8026f5b424fc178d683e63555335f8b33e0aba04`.
- M118 neutral tunnel variance/backup capability is closed historically and corrected by M119 standby-expiry/variance semantics.
- M119 neutral tunnel-pool correctness is closed; M120 server transaction work is ready.
- M111 is historically closed with 40 SessionWire cells applied and four `UseSSL` cells blocked; M121 will re-audit `SigType` truthfulness.
- M112 is historically closed as blocked after applying 24 TCP client lifecycle cells; M121 will re-audit the 18 `Close`/`CloseTime`/`NewDest` cells.
- M113 is historically closed as blocked with 21 server presentation/routing/LeaseSet cells.
- M114 is historically closed as blocked and remains valid for its reviewed head; it is not the final completion record after this corrective line.
- Current M095 artifact records `312 apply / 70 blocked_primitive / 458 not_applicable`. Treat these as the current persisted counts, not a KPI: M121 may truthfully demote previously applied cells.
- Full Proposal 170 status remains **partial**.

## Internal Yosemite dependency state

ADR-0005 remains authoritative.

The ordinary workspace Yosemite dependency remains unchanged for non-I2PControl code. I2PControl alone uses optional alias `yosemite-i2pcontrol`, exact-pinned to Y002 implementation `8026f5b424fc178d683e63555335f8b33e0aba04`.

Yosemite Y003 implementation `9ac7d9a0ac2a8d526e363f150466b579b017e116` is **not consumed** by Emissary. Post-closure review found material LeaseSet wire-semantic defects. `eggstack/yosemite` now registers Y004 as the sole ready corrective handoff.

No `[patch.crates-io]`, workspace replacement, path dependency, vendoring, floating branch/tag, or upstream activity is authorized.

## Corrective dependency graph

```text
                         eggstack/yosemite
Y003 LeaseSet attempt                         [HISTORICAL CLOSED; CORRECTIVE REQUIRED]
  |
  v
Y004 canonical LeaseSet wire corrective      [READY IN YOSEMITE]
  |
  +------------------------------------------------------+
                                                         |
                         eggstack/emissary               |
M114 historical final reclosure                          |
  |                                                      |
  v                                                      |
M119 M118 standby-expiry + variance semantics            |
[CLOSED]                                                 |
  |                                                      |
  v                                                      |
M120 server validation + secret transactionality         |
[READY]                                                  |
  |                                                      |
  v                                                      |
M121 M111/M112 semantic truthfulness                     |
[PROPOSED / BLOCKED ON M120]                             |
  |                                                      |
  +------------------------------+-----------------------+
                                 |
                                 v
M122 corrected Yosemite exact-pin adoption
[PROPOSED / BLOCKED ON M121 + Y004]
  |
  v
fresh M113/LeaseSet neutral-capability audit/plan        [FUTURE / NOT YET AUTHORIZED]
  |
  v
remaining residual implementation + new final reclosure [FUTURE]
```

## Current Emissary handoff — M120

Plan:

- `plans/implementation/i2pcontrol-proposal-170/120-server-start-preallocation-validation-and-secret-transactionality-corrective.md`

Status: **ready**.

Baseline:

- `feafc6a1d9650887015a01f87bf21b57a4e92085`.

Objective:

- fail every deterministically invalid/unsupported server start before private destination allocation/import/persistence;
- make remaining server-secret mutation transactional across runtime start failure;
- remain I2PControl-only with no core/router/Yosemite change and no matrix promotion.

M120 is the sole dependency-ready Emissary implementation handoff. M119 is closed by `plans/closure/i2pcontrol-proposal-170/119-closure.md`.

## Registered future corrective plans — not ready

### M121

`plans/implementation/i2pcontrol-proposal-170/121-m111-m112-semantic-truthfulness-corrective.md`

Status: proposed/blocked on M120 closure.

Objective: independently settle `SigType` support classification and reference `Close`/`CloseTime`/`NewDest` idle-session semantics. Preserve truthfulness over counts: affected cells may be returned to `blocked_primitive` if exact semantics require unavailable lower-layer primitives.

No core/Yosemite/crypto implementation is authorized by M121.

### M122

`plans/implementation/i2pcontrol-proposal-170/122-corrected-yosemite-leaseset-pin-adoption.md`

Status: proposed/blocked on M121 and Yosemite Y004 closure.

Objective: advance only the optional I2PControl Yosemite alias to the exact reviewed Y004 implementation SHA and prove corrected generic LeaseSet wire reachability. M122 is infrastructure only and cannot promote M113 cells.

## Yosemite current handoff — external/internal dependency

Repository: `eggstack/yosemite`

Plan:

- `plans/implementation/004-y003-leaseset-wire-semantics-corrective.md`

Status: **ready in Yosemite**.

Y004 corrects canonical LeaseSet private/signing-key property names, DH/PSK per-client authorization representation, and guessed type domains while retaining bounded validation/redaction/default compatibility. It implements no router cryptography.

Emissary agents must not implement Y004 in this repository and must not pin Y003.

## Historical M111-M118/M114 state

| Milestone | Current disposition |
|---|---|
| M111 | historical closed; M121 re-audits SigType classification |
| M112 | historical closed as blocked; M121 re-audits 18 applied Close/CloseTime/NewDest cells |
| M113 | historical closed as blocked; 21 server residuals remain |
| M114 | historical closed as blocked at 312/70/458 and missing external interoperability evidence |
| M115 | closed |
| M116 | closed |
| M117 | closed at exact Y002 pin |
| M118 | historical closed; corrected by M119 |
| M119 | closed; M120 corrective required |

Historical closure records remain unchanged. Later corrective closures supersede only the affected claims.

## Residual ownership and future LeaseSet rule

At the current persisted artifact, blocked count 70 consists of:

- 4 M111 `UseSSL` cells;
- 45 M112 proxy/plugin/profile/reduction/Streamr lifecycle cells;
- 21 M113 presentation/routing/LeaseSet cells.

M121 may increase blocked count if applied semantic claims do not survive independent re-freeze.

Y004/M122 provide only corrected SAM-client transport. Current Emissary `SamSession` locally constructs a normal LeaseSet2; there is no accepted encrypted/authenticated LeaseSet construction owner yet. A new neutral-core LeaseSet plan must not be registered until M122 closes and the exact crypto/secret/NetDb owner is frozen. Do not add router crypto merely for matrix parity.

A cell becomes `apply` only with real request → validated mapping → actual runtime effect evidence. A serializer/config field alone is not support. No `accept_inert` state is permitted.

## Registry maintenance rules

1. M120 is the sole dependency-ready Emissary handoff.
2. Yosemite Y004 is separately ready only in `eggstack/yosemite`.
3. M121 and M122 must not execute until their named gates close and this registry promotes the specific next plan.
4. Do not consume Yosemite Y003. Current fork pin remains Y002 until M122.
5. Treat `312 / 70 / 458` as current artifact state only; M121 may legitimately alter it.
6. Keep Proposal 170 policy under `emissary-cli/src/i2pcontrol/**` wherever possible. M119 was a specifically bounded neutral-core corrective to M118.
7. No new neutral LeaseSet/core plan is authorized until corrected Y004 adoption and a focused capability/crypto ownership audit.
8. No global Yosemite patch/replacement/vendor/path dependency is permitted.
9. Proposal 170 remains pinned to `2026-05-20`; later revisions require a delta audit.
10. All external/upstream sources are read-only. No upstream issue/PR/review/submission/merge/adoption/contact/release activity is authorized.