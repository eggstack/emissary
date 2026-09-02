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

Pinned Proposal 170 revision: `2026-05-20` (proposal remains Open).

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies/interfaces are satisfied and the plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency/evidence requirement prevents execution.
- **closing** — implementation landed and closure evidence is being gathered.
- **closed** — closure accepted for the pinned implementation head.
- **closed as blocked** — authorized safe subset/review completed but a named capability blocker remains.
- **corrective pass required** — material implementation/planning/evidence defect invalidated the prior disposition.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Blocker/next transition |
|---|---|---|---|---|
| I2PControl Proposal 170 full-support completion | active | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | none | M116 closed; M111-M114 remain blocked on independent capability/residual gates |
| I2PControl Proposal 170 source/truthfulness | RouterInfo source line closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | none | 42 available / 1 neutral / 0 unavailable |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | none | M061/M062/M063 remain controlling; no new lower-layer seam |
| I2PControl tunnel runtime | all 12 data planes real | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | none | M116 closed; do not redesign data planes or option program |
| I2PControl tunnel security | closed at M093 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | regression authority | M116 must restore Streamr/shared-session isolation without weakening M093 |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational.
- All 12 TunnelManager data planes and seven canonical action handlers exist.
- All six ClientServicesInfo selectors are operational.
- API 1-only negotiation and M107/M108 managed TLS hardening are operational.
- M109 startup lifecycle and M115 runtime-disable/lifecycle corrections are closed.
- M110 implemented shared client sessions and destination/key ownership, but post-closure review found M116-scoped concurrency, cancellation, Streamr isolation, compatibility-key, `NewDest`, and internal secret-Debug defects.
- M095 currently records `248 apply / 134 blocked_primitive / 458 not_applicable`; M116 reclassified seven client `NewDest` cells to M112 and retained `Shared × streamrclient` after canonical producer matching.
- Full Proposal 170 status remains **partial**.

## Current dependency graph

```text
M108 managed TLS upgrade corrective                 [CLOSED]
  |
  v
M109 startup-managed action semantics               [CLOSED]
  |
  v
M115 M109 runtime/lifecycle corrective              [CLOSED]
  |
  v
M110 shared session + destination/key ownership     [CLOSED — HISTORICAL]
  |
  v
M116 M110 shared/session/NewDest corrective         [CLOSED]
  |
  v
M111 SAM session-wire options                       [ROADMAP ONLY / DEPENDENCY-BLOCKED]
  |
  v
M112 client proxy/session-lifecycle residuals       [ROADMAP ONLY / BLOCKED]
  |
  v
M113 server presentation + LeaseSet residuals       [ROADMAP ONLY / BLOCKED]
  |
  | zero applicable residual cells + no open corrective
  v
M114 live/reference final reclosure                 [ROADMAP ONLY / BLOCKED]
```

M110-M114 were numbered before later correctives M115/M116 were discovered. Numeric identifiers remain stable. Current execution order is M109 → M115 → M110 → M116 → M111 → M112 → M113 → M114.

Per `plans/003-planning-process.md`, M116 was the sole dependency-ready implementation handoff and is now closed. M111-M114 MUST NOT be executed until their independent gates are satisfied and this registry explicitly marks the specific plan ready.

## Closed handoff — M116

Plan:

- `plans/implementation/i2pcontrol-proposal-170/116-m110-shared-session-and-newdest-corrective-pass.md`

Status: **closed**. Closure: `plans/closure/i2pcontrol-proposal-170/116-closure.md`.

Baseline:

- `09247ccf8367a7b3a7050e0584614c4e59cafe8e` — post-M110 closure/containment head.

Bounded objective:

1. eliminate the `Notify::notify_waiters()` lost-wakeup race in shared stream/datagram acquisition;
2. make in-flight shared-session creator reservations cancellation/drop/failure safe;
3. replace the 64-bit private-identity fingerprint with collision-safe, redacted compatibility equality;
4. prevent shared Streamr clients from cross-delivering another producer's authenticated datagrams and drop unrelated-peer traffic;
5. directly resolve `NewDest` against pinned/reference lifecycle semantics rather than treating every manual start as a new-destination trigger;
6. return `NewDest` cells to `blocked_primitive` under M112 if correct semantics require M112's `Close*` lifecycle owner;
7. remove raw private-destination `Debug` surfaces;
8. reconcile M095/M105/M110 ledger to exact post-corrective counts.

Authorized production paths are I2PControl-only and limited to:

- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs`;
- `emissary-cli/src/i2pcontrol/backends/streamr.rs`;
- `emissary-cli/src/i2pcontrol/client_secret_store.rs`;
- `emissary-cli/src/i2pcontrol/backends/options.rs` when required by final option relationships;
- `emissary-cli/src/i2pcontrol/production.rs` when required by final identity transaction semantics.

M116 MUST NOT change Yosemite/Cargo/core/util/startup/frontend/workflow/release paths or implement M111-M114 residual features.

The pre-corrective `255 / 127 / 458` matrix was not an acceptance target. The closed M116 matrix is `248 / 134 / 458`; any future cell without exact safe runtime semantics must remain `blocked_primitive`.

## Historical closed handoffs

### M115

`plans/implementation/i2pcontrol-proposal-170/115-m109-runtime-disable-and-lifecycle-truthfulness-corrective-pass.md`

Closed; closure: `plans/closure/i2pcontrol-proposal-170/115-closure.md`. It corrected M109 runtime-disable selection, state truthfulness, and neutral startup-client session recovery/lifetime. It owns no Proposal option cell.

### M110

`plans/implementation/i2pcontrol-proposal-170/110-shared-client-session-and-destination-key-ownership-completion.md`

Historically closed; closure: `plans/closure/i2pcontrol-proposal-170/110-closure.md`. M116 does not rewrite this closure; it is the separately numbered corrective authority for later-discovered defects. M110's completion ledger/current matrix are provisional evidence until M116 reclosure.

## Roadmap-defined future plans — NOT registered for execution

### M111

`plans/implementation/i2pcontrol-proposal-170/111-sam-session-wire-option-completion.md`

Proposed/dependency-blocked. Owns up to 44 current cells: `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions`. Requires an accepted public Yosemite session-option path. No vendor/fork/path dependency/parallel SAM stack is authorized.

### M112

`plans/implementation/i2pcontrol-proposal-170/112-client-proxy-and-session-lifecycle-residual-completion.md`

Proposed/blocked. Owns 69 client proxy/lifecycle cells, including seven `NewDest` cells transferred by M116. It must not be executed merely to preserve M110 matrix counts.

### M113

`plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`

Proposed/blocked. Owns up to 21 server presentation/address-routing/LeaseSet cells. No local-target expansion or LeaseSet security downgrade; exact accepted primitives required.

### M114

`plans/implementation/i2pcontrol-proposal-170/114-full-proposal-170-live-interoperability-and-final-reclosure.md`

Proposed/blocked. Final reclosure requires M116 closed, zero applicable residual cells, and no open high/medium Proposal-scoped security/correctness corrective. It implements no missing feature.

## Residual option ownership

The post-M116 residual count is 134:

- M111: 44;
- M112: 69 (including seven `NewDest` cells transferred by M116);
- M113: 21.

M116 returned seven `NewDest` cells to `blocked_primitive` under M112 and retained `Shared × streamrclient` as `apply` after canonical producer matching. The registry uses the exact closed counts rather than preserving `127` by convention.

A cell may remain/become `apply` only with real request→runtime evidence. A cell may become `not_applicable` only with affirmative pinned/reference evidence. No accept-inert state is permitted.

## Recently closed handoffs

| Milestone | Status | Closure |
|---|---|---|
| M104 | closed as blocked | `plans/closure/i2pcontrol-proposal-170/104-closure.md` |
| M105 | closed | `plans/closure/i2pcontrol-proposal-170/105-closure.md` |
| M106 | closed | `plans/closure/i2pcontrol-proposal-170/106-closure.md` |
| M107 | closed | `plans/closure/i2pcontrol-proposal-170/107-closure.md` |
| M108 | closed | `plans/closure/i2pcontrol-proposal-170/108-closure.md` |
| M109 | closed | `plans/closure/i2pcontrol-proposal-170/109-closure.md` |
| M115 | closed | `plans/closure/i2pcontrol-proposal-170/115-closure.md` |
| M110 | closed historically; M116 corrective required | `plans/closure/i2pcontrol-proposal-170/110-closure.md` |
| M116 | closed | `plans/closure/i2pcontrol-proposal-170/116-closure.md` |

M093 remains the current tunnel production/security regression authority. M092 remains authority for removal of the unauthorized M091 Yosemite/core/vendor delta.

## Registry maintenance rules

1. No future handoff is currently ready; M116 is closed.
2. M111-M114 are roadmap/indexed only and MUST NOT execute until explicitly promoted.
3. Treat `248 / 134 / 458` as the current closed matrix; preserve evidence if future plans alter it.
4. Do not reattempt final reclosure while any applicable cell is blocked/planned/unsupported/unknown/inert or while M116 remains open.
5. Proposal 170 policy remains under `emissary-cli/src/i2pcontrol/**` wherever possible.
6. M116 authorizes no non-I2PControl production path.
7. No Yosemite vendoring/forking/path override/parallel SAM or Proposal-shaped core API is authorized.
8. Proposal 170 remains pinned to `2026-05-20`; a later revision requires a delta audit.
9. External sources are read-only. No upstream issue/PR/review/submission/merge/adoption/contact, release, or contribution preparation is authorized.
10. All repository writes remain internal to `eggstack/emissary`.
