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

Authorized internal repositories for the current dependency-completion line:

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
- **closed as blocked** — authorized safe subset/review completed but a named capability blocker remains;
- **corrective pass required** — material defect invalidated the prior disposition.

## Active subsystem roadmap

| Subsystem | Status | Roadmap | Current handoff | Blocker/next transition |
|---|---|---|---|---|
| I2PControl Proposal 170 full-support completion | active | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | **M111 ready** | M117 and M118 are closed; M111 semantic re-freeze is the next transition |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M062 regression authority | ADR-0005 permits only optional I2PControl-owned exact-revision fork alias; no global patch/vendor/path dependency |
| I2PControl tunnel security | closed at M093 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | regression authority | M118 closure preserves tunnel anonymity/resource boundaries |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational.
- All 12 TunnelManager data planes and seven canonical action handlers exist.
- All six ClientServicesInfo selectors are operational.
- API 1-only negotiation and M107/M108 managed TLS hardening are operational.
- M109/M115 startup lifecycle and M110/M116 shared-session/destination ownership corrective line are closed.
- M095 records `248 apply / 134 blocked_primitive / 458 not_applicable` after M116.
- Seven client `NewDest` cells remain transferred to M112.
- Full Proposal 170 status remains **partial**.

## Dependency architecture change

ADR-0005 records the maintainer-authorized internal Yosemite fork strategy.

The ordinary workspace Yosemite dependency remains unchanged for non-I2PControl code. M117 added a second optional package alias pinned by exact `git + rev` to `eggstack/yosemite`, activated only by `i2pcontrol` and imported only below `emissary-cli/src/i2pcontrol/**`.

No `[patch.crates-io]`, workspace replacement, path dependency, vendoring, floating branch dependency, or upstream activity is authorized.

The Yosemite fork now has its own planning registry. Its immediate handoff is Y001 (`SESSION CREATE` option surface), followed by Y002 (signature-aware `DEST GENERATE`). Y003 (LeaseSet session-option transport) remains semantically blocked until M113 freezes the exact interface.

## Current dependency graph

```text
M116 post-M110 corrective                            [CLOSED]
  |
  +-----------------------------+
  |                             |
  v                             v
M118 neutral SAM/tunnel-pool    Yosemite Y001
variance + backups              SESSION CREATE surface
[CLOSED]                        [READY IN YOSEMITE]
  |                             |
  |                             v
  |                         Yosemite Y002
  |                         signature DEST GENERATE
  |                         [CLOSED]
  |                             |
  |                             v
  |                         M117 exact fork alias/adoption
  |                         [CLOSED; PIN 8026f5b]
  |                             |
  +--------------+--------------+
                 |
                 v
M111 SAM session-wire Proposal mapping               [READY / SEMANTIC RE-FREEZE]
  |
  v
M112 client proxy/session lifecycle                  [ROADMAP / BLOCKED]
  |
  v
M113 server presentation + LeaseSet                  [ROADMAP / BLOCKED]
  |   \
  |    `-- interface freeze may unlock Yosemite Y003
  v
M114 live/reference final reclosure                  [ROADMAP / BLOCKED]
```

M110-M114 identifiers remain stable; later corrective/dependency milestones are inserted without renumbering historical plans.

## Closed Emissary handoff — M117

Plan:

- `plans/implementation/i2pcontrol-proposal-170/117-internal-yosemite-fork-pin-and-i2pcontrol-adapter-integration.md`

Status: **closed**; implementation commit `22c893a`; closure:
`plans/closure/i2pcontrol-proposal-170/117-closure.md`.

Yosemite Y001 and Y002 are closed in `eggstack/yosemite`. M117 pins the exact Y002
implementation revision `8026f5b424fc178d683e63555335f8b33e0aba04`, which contains Y001,
and is limited to the I2PControl feature-owned adapter boundary. It does not promote
Proposal cells or implement M118 router behavior.

## Closed Emissary handoff — M118

Plan:

- `plans/implementation/i2pcontrol-proposal-170/118-neutral-sam-tunnel-pool-variance-backup-capability.md`

Status: **closed**; implementation commit `e7f3e04beccbf9f894ca23ec6d7e3ee21a180001`; closure:
`plans/closure/i2pcontrol-proposal-170/118-closure.md`.

Baseline:

- `464213f0434badeb04dbf80a95a8703530c6a909`.

Objective: add only the neutral Emissary SAM/tunnel-pool primitives necessary to honor canonical length-variance and backup-quantity session settings.

Authorized production paths were limited to:

- `emissary-core/src/sam/parser.rs`;
- `emissary-core/src/sam/mod.rs`;
- `emissary-core/src/tunnel/pool/mod.rs`.

M118 changed no I2PControl production code and did not alter M095 counts. Exact reference semantics,
standby/failover behavior, zero-hop limits, cancellation, and build bounds are recorded in the
closure evidence.

## M117 dependency gate — satisfied

Plan:

- `plans/implementation/i2pcontrol-proposal-170/117-internal-yosemite-fork-pin-and-i2pcontrol-adapter-integration.md`

Status: **satisfied** by Yosemite Y001/Y002 closure and the exact reviewed fork commit above.

M117 did not replace the workspace Yosemite dependency. It added only the optional
`yosemite-i2pcontrol` package alias in `emissary-cli`, exact-revision pinned and
feature-owned, and migrated I2PControl-only imports/use sites.

M117 implements no router behavior and no Proposal cell promotion.

## Roadmap-defined future plans — not ready

### M111

`plans/implementation/i2pcontrol-proposal-170/111-sam-session-wire-option-completion.md`

Ready after M117 and M118 closure. M117 provides the accepted internal fork API and M118 provides the real neutral variance/backup effect. M111 must re-freeze `UseSSL` semantics and MUST NOT map Proposal `UseSSL` to Yosemite's SAM-router transport `ssl` merely because the field exists.

### M112

`plans/implementation/i2pcontrol-proposal-170/112-client-proxy-and-session-lifecycle-residual-completion.md`

Blocked. Owns 69 client proxy/lifecycle cells, including seven `NewDest` cells transferred by M116. It may consume closed Yosemite generic capabilities if exact semantics require them, but remains I2PControl-owned wherever possible.

### M113

`plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`

Blocked. Owns up to 21 server presentation/routing/LeaseSet cells. Its semantic freeze is an interface dependency for Yosemite Y003. Any real encrypted/authenticated LeaseSet primitive missing from Emissary core still requires a separately registered neutral-owner plan before core changes.

### M114

`plans/implementation/i2pcontrol-proposal-170/114-full-proposal-170-live-interoperability-and-final-reclosure.md`

Blocked. Implements no missing feature. Readiness requires zero applicable residual cells and no open high/medium Proposal-scoped correctness/security corrective.

## Residual ownership

Current blocked count remains 134 until a Proposal-owner plan changes M095:

- M111: 44 current cells pending semantic re-freeze;
- M112: 69;
- M113: 21.

M117/M118/Y001/Y002 are infrastructure/capability prerequisites and do not change matrix counts by themselves.

A cell becomes `apply` only with real request→runtime evidence. `not_applicable` requires affirmative pinned/reference evidence. No accept-inert state is permitted.

## Recently closed handoffs

| Milestone | Status | Closure |
|---|---|---|
| M107 | closed | `plans/closure/i2pcontrol-proposal-170/107-closure.md` |
| M108 | closed | `plans/closure/i2pcontrol-proposal-170/108-closure.md` |
| M109 | closed | `plans/closure/i2pcontrol-proposal-170/109-closure.md` |
| M115 | closed | `plans/closure/i2pcontrol-proposal-170/115-closure.md` |
| M110 | closed historically; corrected by M116 | `plans/closure/i2pcontrol-proposal-170/110-closure.md` |
| M116 | closed | `plans/closure/i2pcontrol-proposal-170/116-closure.md` |
| M117 | closed | `plans/closure/i2pcontrol-proposal-170/117-closure.md` |
| M118 | closed | `plans/closure/i2pcontrol-proposal-170/118-closure.md` |

M093 remains tunnel production/security regression authority. M092 remains historical authority against unauthorized Yosemite/core/vendor changes; ADR-0005 supersedes only its blanket internal-fork prohibition with the exact alias/revision strategy above.

## Registry maintenance rules

1. M111 is the current dependency-ready Emissary implementation handoff.
2. M117 is closed at the exact pinned Yosemite revision above.
3. M118 is closed at implementation commit `e7f3e04`; its neutral capability does not change matrix counts.
4. Yosemite Y001 is separately ready only in `eggstack/yosemite`; Emissary agents must not implement it in this repository.
5. M111 is ready but must re-freeze its exact semantic/cell scope before implementation; M112-M114 remain blocked.
6. Treat `248 / 134 / 458` as the current closed matrix; prerequisite infrastructure does not alter it.
7. Keep Proposal 170 policy under `emissary-cli/src/i2pcontrol/**` wherever possible; M118 is a specifically authorized neutral lower-layer exception.
8. No global Yosemite patch/replacement/vendor/path dependency is permitted.
9. Proposal 170 remains pinned to `2026-05-20`; later proposal revisions require a delta audit.
10. Writes are internal to `eggstack/emissary` and plan-authorized `eggstack/yosemite` only. No upstream issue/PR/review/submission/merge/adoption/contact/release activity is authorized.
