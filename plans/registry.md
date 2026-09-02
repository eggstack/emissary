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
| I2PControl Proposal 170 full-support completion | active | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | **M110 ready** | implement bounded shared client-session and destination/key ownership; M111 remains dependency-blocked |
| I2PControl Proposal 170 source/truthfulness | RouterInfo source line closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | none | 42 available / 1 neutral / 0 unavailable |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M115 exact M109 CLI-tunnel seam only | M061/M062/M063 remain controlling |
| I2PControl tunnel runtime | all 12 data planes real | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | M115 corrective only | do not redesign data planes or option semantics |
| I2PControl tunnel security | closed at M093 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | regression authority | later work must preserve M093 invariants |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational.
- All 12 TunnelManager data planes and seven canonical action handlers exist.
- All six ClientServicesInfo selectors are operational.
- API 1-only negotiation and M107/M108 managed TLS hardening are operational.
- Current TunnelManager option matrix: `224 apply / 158 blocked_primitive / 458 not_applicable`.
- M109 added named startup lifecycle and mixed `All=true`; M115 corrected runtime-disable isolation, shared-client session recovery/lifetime, and lock-contention state truthfulness.
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
M110 shared session + destination/key ownership     [READY — REGISTERED HANDOFF]
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

M110-M114 were numbered before this post-M109 corrective was discovered. Their numeric identifiers remain stable; execution order is M109 → M115 → M110 → M111 → M112 → M113 → M114.

Per `plans/003-planning-process.md`, M110 is the sole dependency-ready implementation handoff. M111-M114 MUST NOT be executed until their predecessor/capability gates are satisfied and this registry marks the specific plan ready.

## Closed corrective — M115

Plan:

- `plans/implementation/i2pcontrol-proposal-170/115-m109-runtime-disable-and-lifecycle-truthfulness-corrective-pass.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/115-closure.md`.

Baseline:

- `ee3b444` — M115 implementation head.

Bounded objective:

1. select the M109 lifecycle-controlled startup path only when runtime I2PControl is enabled;
2. preserve the historical startup client/server path when I2PControl is disabled, even in a feature-capable binary;
3. remove fabricated `Starting` state under lifecycle lock contention;
4. replace one-shot controlled client session seeding with a retryable bounded shared-session owner tied to active startup clients;
5. preserve one-session sharing while clients are active and release it after the final member stops;
6. reconcile stale planning state.

Authorized non-I2PControl production paths are limited to:

- `emissary-cli/src/main.rs`;
- `emissary-cli/src/tunnel/client.rs`;
- `emissary-cli/src/tunnel/server.rs`.

Proposal adaptation may change only the existing M109 paths under `emissary-cli/src/i2pcontrol/**` where required for truthful neutral state/action mapping.

M115 MUST NOT:

- change M095/M105 dispositions or implement any of the 158 residual option cells;
- implement M110 `Shared`/destination/key behavior;
- rewrite startup configuration;
- change Yosemite/Cargo/core/util/frontend/workflows;
- build a router-global session/lifecycle owner;
- weaken M093 security/anonymity bounds;
- interact with upstream repositories/maintainers.

M115 closure leaves M095 exactly `224 / 158 / 458` and records that M110's independent readiness gates are satisfied.

## Ready handoff — M110

### M110

`plans/implementation/i2pcontrol-proposal-170/110-shared-client-session-and-destination-key-ownership-completion.md`

Ready/registered. Owns up to 31 current cells: `Shared`, `NewDest`, `PersistentClientKey`, `PrivKeyFile`. M115 closure accepts the bounded I2PControl-local ownership model and records sufficient accepted Yosemite 0.7.0 public primitives for the required destination-material handoff.

## Roadmap-defined future plans — NOT registered for execution

### M111

`plans/implementation/i2pcontrol-proposal-170/111-sam-session-wire-option-completion.md`

Proposed/dependency-blocked. Owns up to 44 cells: `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions`. Requires an accepted public Yosemite session-option path. No vendor/fork/path dependency/parallel SAM stack is authorized.

### M112

`plans/implementation/i2pcontrol-proposal-170/112-client-proxy-and-session-lifecycle-residual-completion.md`

Proposed/blocked. Owns up to 62 post-M106 client proxy/lifecycle cells. Must separate portable Proposal behavior from Java plugin/profile/timer mechanisms and preserve no-DNS/no-clearnet-fallback security.

### M113

`plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`

Proposed/blocked. Owns up to 21 server presentation/address-routing/LeaseSet cells. No local-target expansion or LeaseSet security downgrade; exact accepted primitives required.

### M114

`plans/implementation/i2pcontrol-proposal-170/114-full-proposal-170-live-interoperability-and-final-reclosure.md`

Proposed/blocked. Final reclosure only after M115 is closed, zero applicable M095/M105 residuals remain, and there is no open high/medium Proposal-scoped security/correctness corrective. It implements no missing feature.

## Residual option ownership

Current 158 cells remain partitioned without overlap:

- M110: 31;
- M111: 44;
- M112: 62;
- M113: 21.

M115 owns none of these cells.

A cell may move to `apply` only with real request→runtime evidence. A cell may move to `not_applicable` only with affirmative pinned/reference evidence. Difficulty alone is not evidence. No accept-inert state is permitted.

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

M109 remains historical closure evidence; M115 is the new corrective pass required by post-closure findings, following the planning-governance rule that corrective passes are new implementation plans rather than rewrites of prior closure history.

M093 remains the current tunnel production/security authority. M092 remains authority for removal of the unauthorized M091 Yosemite/core/vendor delta.

## Registry maintenance rules

1. M110 is the sole dependency-ready implementation handoff.
2. M111-M114 are roadmap/indexed only and MUST NOT be executed until this registry marks the specific plan ready.
3. M115 does not change the `224 / 158 / 458` option matrix.
4. Do not reattempt final reclosure while M115 is open or any applicable option cell is blocked/planned/unsupported/unknown/inert.
5. Proposal 170 policy remains under `emissary-cli/src/i2pcontrol/**` wherever possible.
6. Non-I2PControl production paths require a neutral canonical owner and exact pre-authorization.
7. No Yosemite vendoring/forking/path override/parallel SAM or Proposal-shaped core API is authorized by these plans.
8. Proposal 170 remains pinned to `2026-05-20`; a later revision requires a delta audit.
9. External sources are read-only. No upstream issue/PR/review/submission/merge/adoption/contact, branch/tag push, release, or contribution preparation is authorized.
10. All repository writes remain internal to `eggstack/emissary`.
