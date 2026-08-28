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
- **blocked** — a named dependency or evidence requirement prevents execution.
- **closing** — implementation landed and closure evidence is being gathered.
- **closed** — closure record accepted for the pinned implementation head.
- **closed as blocked** — the milestone executed its authorized safe subset and reached a named stop condition; unresolved capability remains blocked.
- **closed internally against pinned revision** — internal closure against an explicitly pinned open external specification; not upstream acceptance.
- **corrective pass required** — a material implementation/planning/evidence defect invalidated the prior disposition.
- **superseded** — replaced and not executable.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Blocker/next transition |
|---|---|---|---|---|
| I2PControl Proposal 170 full-support completion | active | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | **residual option blocker** | M104 closed as blocked; no bounded residual primitive plan is dependency-ready |
| I2PControl Proposal 170 source/truthfulness | RouterInfo source line closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | none | current RouterInfo matrix: 42 available / 1 protocol-permitted neutral / 0 unavailable |
| I2PControl Proposal 170 containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | none | M061/M062/M063 rules remain controlling |
| I2PControl tunnel runtime | all 12 data planes real | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | option semantics only | do not redesign data planes for option parity |
| I2PControl tunnel security | closed at M093 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | no corrective security handoff | M088 lower-layer residual remains accepted absent new evidence |

## Current full-support sequence

The original M098/M099 dependency graph was corrected after M097 closure proved that milestone-wide blocking was too coarse. Dependency authority is now per option cell.

```text
M095 full-support matrix/containment                [CLOSED]
  |
  +--> M096 AddressBook SetConfig                   [CLOSED]
  +--> M100 transit 15s                             [CLOSED]
  +--> M101 router news                             [CLOSED]
  +--> M102 network-error owner                     [CLOSED]
  +--> M103 banned-peer semantics                   [CLOSED]
  |
  +--> M097 common session/key options              [CLOSED AS BLOCKED]
         | supported subset landed
         | residual shared-session / SAM-wire /
         | client-key / PrivKeyFile primitives remain
         |
         +------------------------------+
                                        |
M098 client/proxy/HTTP independent slice [CLOSED]  |
  |   transfer genuine primitive-dependent cells --+
  v
M099 server/access/throttle independent slice       [CLOSED INTERNALLY — PARTIAL]
  |   transfer genuine LeaseSet/session/unsafe cells ----+
  v                                                       |
residual TunnelManager blocker line                 [BLOCKED]
  | no plan is executable until a bounded primitive       |
  | path exists under current containment rules            |
  +--------------------------------------------------------+
                                                           |
                                                           v
M104 live interoperability/full reclosure          [CLOSED AS BLOCKED]
```

## Closed handoff — M098

Plan:

- `plans/implementation/i2pcontrol-proposal-170/098-client-proxy-management-and-http-option-completion.md`

Status: **closed**.

M098 is a corrective dependency revision, not a broader client redesign. Before production code it must reconcile every M098-owned matrix cell against the M097 closure. It may implement only cells with exact runtime semantics inside existing I2PControl client/proxy/filter ownership. Genuine M097-dependent cells are transferred to explicit `blocked_primitive` residual ownership before coding.

Expected independent work includes bounded proxy/outproxy/authentication and HTTP privacy/filter semantics. Client-management fields remain in M098 only when exact generation-local behavior exists; no router tunnel-pool operation may be approximated with an application timeout.

M098 does not authorize:

- Yosemite vendoring/forking/patching;
- new `emissary-core/**` API;
- new dependency/lockfile changes;
- SOCKS protocol expansion;
- an outproxy plugin subsystem;
- weakening LAN/DNS/anonymity boundaries.

M098 closure updated the M095 matrix and advances M099 as the next handoff. Its
applicable proxy/auth/privacy cells are operational; residual plugin/TLS-proxy,
jump-list, and client-management cells name their missing primitives explicitly.

## Closed handoff — M099

Plan:

- `plans/implementation/i2pcontrol-proposal-170/099-server-access-throttle-and-leaseset-option-completion.md`

Status: **closed internally against the pinned 2026-05-20 revision; partial**.

M099 reconciled server-role matrix cells and implemented the exact subset owned by existing accepted I2PControl server admission/filter/runtime paths. Its closure is recorded at `plans/closure/i2pcontrol-proposal-170/099-closure.md`.

Expected independent work includes HTTP presentation/filter policy, access lists, confined filter-file loading, connection ceilings, peer/global rates, POST limits, periods, and tunnel-local temporary denial. LeaseSet/session-security cells remain residual blockers when the current supported Yosemite/SAM path cannot implement them without downgrade.

M099 does not authorize new core LeaseSet APIs, router-wide banning, arbitrary filesystem access, request-selected LAN targets, or Yosemite dependency changes.

## Residual M097 blocker authority

M097 closure:

- `plans/closure/i2pcontrol-proposal-170/097-closure.md`

M097 closed as blocked after implementing `TunnelLength`, `TunnelQuantity`, and typed `EncType` through the existing supported session path.

Named residual primitives include:

- bounded shared-session ownership/handoff for `Shared`;
- actual SAM `SESSION CREATE` serialization for `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, and `CustomOptions`;
- client destination/key lifecycle for `NewDest` and `PersistentClientKey`;
- confined validated private-key import/store/handoff for `PrivKeyFile`;
- any M098/M099 cell whose exact semantics are proven to require those same missing authorities.

No successor implementation plan for these residuals is registered now. A future plan may be written only when current repository/dependency evidence shows a bounded path that obeys containment. Current planning does **not** authorize vendoring/forking Yosemite or adding Proposal-170-shaped core APIs merely to remove blockers.

## M104 closure

Plan:

- `plans/implementation/i2pcontrol-proposal-170/104-full-proposal-170-live-interoperability-and-reclosure.md`

Status: **closed as blocked**.

Closure: `plans/closure/i2pcontrol-proposal-170/104-closure.md`.

M104 reached its authorized verification stop condition. It requires:

- M095, M096, M100-M103 closures;
- revised M098 closure;
- revised M099 closure;
- zero applicable `planned_apply` or `blocked_primitive` TunnelManager cells;
- closure of any future bounded residual-option plan;
- then focused live/reseeded/reference-router interoperability and integrated security/containment reclosure.

The final matrix still contains 164 applicable `blocked_primitive` cells, so M104
cannot claim full support or weaken the completion definition to accommodate an
external/library blocker. No residual implementation plan is dependency-ready.

## Recently closed full-support handoffs

| Milestone | Status | Closure |
|---|---|---|
| M095 | closed | `plans/closure/i2pcontrol-proposal-170/095-closure.md` |
| M096 | closed | `plans/closure/i2pcontrol-proposal-170/096-closure.md` |
| M097 | closed as blocked | `plans/closure/i2pcontrol-proposal-170/097-closure.md` |
| M098 | closed | `plans/closure/i2pcontrol-proposal-170/098-closure.md` |
| M100 | closed | `plans/closure/i2pcontrol-proposal-170/100-closure.md` |
| M101 | closed | `plans/closure/i2pcontrol-proposal-170/101-closure.md` |
| M102 | closed | `plans/closure/i2pcontrol-proposal-170/102-closure.md` |
| M103 | closed | `plans/closure/i2pcontrol-proposal-170/103-closure.md` |
| M104 | closed as blocked | `plans/closure/i2pcontrol-proposal-170/104-closure.md` |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, and all 13 SetConfig keys operational under the confined owner;
- all 12 TunnelManager data planes and all 7 canonical actions real;
- supported M097 common options applied; remaining primitive-dependent options fail before allocation;
- all 6 ClientServicesInfo selectors operational;
- full Proposal 170 status remains **partial** until all applicable TunnelManager option cells and M104 live closure complete.

## Current production/security/containment authority

- M061 exact source containment remains controlling.
- M062/M063 dependency/feature isolation remains controlling.
- M091 remains corrective-pass-required technical history and is not authorization.
- M092 removed the unauthorized M091 vendor/core/dependency expansion.
- M093 remains current tunnel production/security authority.
- M094 was planning reconciliation only.
- M102's three neutral lower-layer observation paths remain the deliberate full-support core exception.
- M103 introduced no router ban behavior.

## Registry maintenance rules

1. M104 is closed as blocked; the residual option blocker remains the current handoff.
2. M098 residual cells remain explicitly blocked and must not be treated as implemented.
3. M097 residual cells remain blocked and fail before allocation.
4. Do not register a residual-primitive implementation plan until a bounded contained primitive path exists.
5. Do not advance M104 while any applicable TunnelManager cell is `planned_apply`, `blocked_primitive`, unsupported, or unknown.
6. Proposal 170 business/admin/application policy remains under `emissary-cli/src/i2pcontrol/**` wherever possible.
7. No unrelated base I2PControl methods are in this phase.
8. Proposal 170 remains pinned to `2026-05-20`; a later revision requires a delta audit.
9. External sources are read-only. No upstream review, merge, issue/PR mutation, submission, contribution preparation, adoption request, branch/tag push, or maintainer contact is authorized.
10. All repository writes remain internal to `eggstack/emissary`.
