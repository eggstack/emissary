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
| Proposal 170 post-M114 corrective line | **active** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | **M122 closed; no implementation handoff ready** | focused LeaseSet capability/crypto-ownership audit authorized but not yet registered |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority | every corrective must retain exact path/dependency ownership |
| I2PControl tunnel security | closed at M093 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | regression authority | M119/M120/M121 must preserve tunnel/secret/proxy boundaries |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational.
- All 12 TunnelManager data planes and seven canonical action handlers exist.
- All six ClientServicesInfo selectors are operational.
- API 1-only negotiation and M107/M108 managed TLS hardening are operational.
- M109/M115 startup lifecycle and M110/M116 shared-session/destination ownership corrective line are closed.
- M117 exact I2PControl-only Yosemite fork integration is closed at Y002 revision `8026f5b424fc178d683e63555335f8b33e0aba04`; M122 advances the alias to Y004 revision `c2db73dba35dd9392947af5c74df29b0b556775f` with corrected LeaseSet wire reachability and no matrix change.
- M118 neutral tunnel variance/backup capability is closed historically and corrected by M119 standby-expiry/variance semantics.
- M119 neutral tunnel-pool correctness is closed; M120 server transaction work is closed.
- M120 server preallocation/secret transactionality is closed; M121 semantic truthfulness corrective is closed (284/98/458).
- M111 is historically closed with 40 SessionWire cells applied and four `UseSSL` cells blocked; M121 demotes its 10 `SigType` cells to blocked (Outcome C).
- M112 is historically closed as blocked after applying 24 TCP client lifecycle cells; M121 demotes 18 `Close`/`CloseTime`/`NewDest` cells to blocked, leaving `ConnectDelay` applied.
- M113 is historically closed as blocked with 21 server presentation/routing/LeaseSet cells.
- M114 is historically closed as blocked and remains valid for its reviewed head; it is not the final completion record after this corrective line.
- Current M095 artifact records `284 apply / 98 blocked_primitive / 458 not_applicable` after M121 truthful demotion.
- Full Proposal 170 status remains **partial**.

## Internal Yosemite dependency state

ADR-0005 remains authoritative.

The ordinary workspace Yosemite dependency remains unchanged for non-I2PControl code. I2PControl alone uses optional alias `yosemite-i2pcontrol`, exact-pinned to Y004 implementation `c2db73dba35dd9392947af5c74df29b0b556775f` (closed Yosemite corrective; supersedes the Y002 pin).

Yosemite Y003 implementation `9ac7d9a0ac2a8d526e363f150466b579b017e116` was **never consumed** by Emissary. Yosemite Y004 implementation `c2db73dba35dd9392947af5c74df29b0b556775f` is **closed** with canonical I2CP LeaseSet property names, mode-aware DH/PSK client authorization, and reference-backed numeric domains; Emissary M122 adopts exactly this revision for corrected SAM-client transport only.

No `[patch.crates-io]`, workspace replacement, path dependency, vendoring, floating branch/tag, or upstream activity is authorized.

## Corrective dependency graph

```text
                         eggstack/yosemite
Y003 LeaseSet attempt                         [HISTORICAL CLOSED; CORRECTIVE REQUIRED]
  |
  v
Y004 canonical LeaseSet wire corrective      [CLOSED IN YOSEMITE]
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
[CLOSED]                                                 |
  |                                                      |
  v                                                      |
M121 M111/M112 semantic truthfulness                     |
[CLOSED 284/98/458]                                      |
  |                                                      |
  +------------------------------+-----------------------+
                                   |
                                   v
M122 corrected Yosemite exact-pin adoption
[CLOSED at Y004 c2db73d — transport only, no matrix change]
  |
  v
fresh M113/LeaseSet neutral-capability audit/plan        [AUTHORIZED / NOT YET REGISTERED]
  |
  v
remaining residual implementation + new final reclosure [FUTURE]
```

## Current Emissary handoff — M122 closed

Plan:

- `plans/implementation/i2pcontrol-proposal-170/122-corrected-yosemite-leaseset-pin-adoption.md`

Status: **closed** (implementation `548c174`; closure `plans/closure/i2pcontrol-proposal-170/122-closure.md`).

Baseline:

- `feafc6a1d9650887015a01f87bf21b57a4e92085`.

Outcome:

- Optional I2PControl alias pins exact reviewed Y004 `c2db73dba35dd9392947af5c74df29b0b556775f`; ordinary Yosemite provenance unchanged; corrected LeaseSet API/wire reachable via fake-SAM adapter tests; matrix unchanged at `284 / 98 / 458`. No core/Yosemite/crypto change; no Proposal mapping added.

There is no dependency-ready Emissary implementation handoff. A focused read-only M113/LeaseSet capability/crypto-ownership audit is authorized by the M122 closure but not yet registered; no M113-successor implementation plan exists.

## Registered future corrective plans — none ready

### M122

`plans/implementation/i2pcontrol-proposal-170/122-corrected-yosemite-leaseset-pin-adoption.md`

Status: **closed** (implementation `548c174`; closure `plans/closure/i2pcontrol-proposal-170/122-closure.md`).

Outcome: optional I2PControl Yosemite alias pins exact reviewed Y004 `c2db73dba35dd9392947af5c74df29b0b556775f` with corrected generic LeaseSet wire reachability. Infrastructure only; no M113 cell promoted.

### Next: M113/LeaseSet capability audit (not yet registered)

A focused read-only audit freezing the exact LeaseSet type(s), crypto
primitives, core owner, secret lifecycle, SAM mapping, no-downgrade rule,
client-auth bounds, NetDb semantics, and minimal paths is authorized by the
M122 closure. No implementation plan may be registered until that audit lands.

## Yosemite current handoff — external/internal dependency

Repository: `eggstack/yosemite`

Plan:

- `plans/implementation/004-y003-leaseset-wire-semantics-corrective.md`

Status: **closed** at implementation `c2db73dba35dd9392947af5c74df29b0b556775f` (closure `plans/closure/004-y003-leaseset-wire-semantics-corrective.md`).

Y004 corrects canonical LeaseSet private/signing-key property names, DH/PSK per-client authorization representation, and reference-backed type domains while retaining bounded validation/redaction/default compatibility. It implements no router cryptography. Emissary M122 adopts exactly this revision; no Y003 revision was ever consumed.

Emissary agents must not implement Y004 in this repository and must not pin Y003.

## Historical M111-M118/M114 state

| Milestone | Current disposition |
|---|---|
| M111 | historical closed; corrected by M121 (10 SigType cells demoted, Outcome C) |
| M112 | historical closed as blocked; corrected by M121 (18 Close/CloseTime/NewDest cells demoted, ConnectDelay retained) |
| M113 | historical closed as blocked; 21 server residuals remain |
| M114 | historical closed as blocked at 312/70/458 and missing external interoperability evidence |
| M115 | closed |
| M116 | closed |
| M117 | closed at exact Y002 pin |
| M118 | historical closed; corrected by M119 |
| M119 | closed; M120 corrective required |
| M120 | closed; M121 corrective closed |
| M121 | closed (284/98/458); M122 adoption closed |
| M122 | closed at Y004 `c2db73d` (transport only, matrix unchanged); LeaseSet capability audit authorized, not yet registered |

Historical closure records remain unchanged. Later corrective closures supersede only the affected claims.

## Residual ownership and future LeaseSet rule

At the current persisted artifact, blocked count 98 consists of:

- 4 M111 `UseSSL` cells;
- 10 M121-demoted `SigType` cells (Outcome C);
- 63 M112 cells (45 proxy/plugin/profile/reduction/Streamr lifecycle + 18 M121-demoted `Close`/`CloseTime`/`NewDest`);
- 21 M113 presentation/routing/LeaseSet cells.

Y004/M122 provide only corrected SAM-client transport (M122 proves it reachable
via adapter tests). Current Emissary `SamSession` locally constructs a normal
LeaseSet2; there is no accepted encrypted/authenticated LeaseSet construction
owner yet. The focused capability/crypto-ownership audit authorized by the M122
closure must land before any neutral-core LeaseSet implementation plan is
registered. Do not add router crypto merely for matrix parity.

A cell becomes `apply` only with real request → validated mapping → actual runtime effect evidence. A serializer/config field alone is not support. No `accept_inert` state is permitted.

## Registry maintenance rules

1. No Emissary implementation handoff is dependency-ready; the M113/LeaseSet capability audit is authorized but not yet registered.
2. Yosemite Y004 is closed at `c2db73d` and adopted by Emissary M122; Y003 was never consumed.
3. M122 is closed; its transport-only disposition changes no matrix cell.
4. Current artifact state is `284 / 98 / 458` after M121 truthful demotion.
5. Keep Proposal 170 policy under `emissary-cli/src/i2pcontrol/**` wherever possible. M119 was a specifically bounded neutral-core corrective to M118.
6. No new neutral LeaseSet/core implementation plan is registered until the authorized capability/crypto-ownership audit lands.
7. No global Yosemite patch/replacement/vendor/path dependency is permitted.
8. Proposal 170 remains pinned to `2026-05-20`; later revisions require a delta audit.
9. All external/upstream sources are read-only. No upstream issue/PR/review/submission/merge/adoption/contact/release activity is authorized.