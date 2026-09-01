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
| I2PControl Proposal 170 full-support completion | active | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | **M107 ready** | post-M106 API-version/AddressBook/TLS corrective pass; 158 TunnelManager residual cells remain independently blocked |
| I2PControl Proposal 170 source/truthfulness | RouterInfo source line closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | none | current RouterInfo matrix: 42 available / 1 protocol-permitted neutral / 0 unavailable |
| I2PControl Proposal 170 containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M107 regression scope | M061/M062/M063 rules remain controlling; M107 authorizes no lower-layer production path |
| I2PControl tunnel runtime | all 12 data planes real | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | option semantics only | do not redesign data planes for M107 or residual option parity |
| I2PControl tunnel security | closed at M093 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | no tunnel-security corrective handoff | M107 managed TLS is the I2PControl administrative listener owner, not TunnelManager TLS/LeaseSet scope |

## Current full-support sequence

M104 reached the final residual gate and closed as blocked. M105 audited the 164 blocked cells, and M106 implemented the only dependency-ready residual subset: six TCP-client `DelayOpen` cells. Production is now 224 `apply`, 158 `blocked_primitive`, and 458 `not_applicable` cells.

A post-M106 review found three independent correctness/security defects in already-implemented I2PControl behavior. Those defects have existing I2PControl-local owners and do not change the TunnelManager matrix. M107 is therefore the sole current dependency-ready corrective handoff.

```text
M095 full-support matrix/containment                 [CLOSED]
  |
  +--> M096 AddressBook SetConfig                    [CLOSED]
  +--> M100 transit 15s                              [CLOSED]
  +--> M101 router news                              [CLOSED]
  +--> M102 network-error owner                      [CLOSED]
  +--> M103 banned-peer semantics                    [CLOSED]
  |
  +--> M097 common session/key options               [CLOSED AS BLOCKED]
         | supported subset landed
         | residual shared-session / SAM-wire /
         | client-key / PrivKeyFile primitives remain
         |
         +------------------------------+
                                        |
M098 client/proxy/HTTP independent slice [CLOSED]   |
  |                                                  |
  v                                                  |
M099 server/access/throttle independent slice        |
[CLOSED INTERNALLY — PARTIAL]                        |
  |                                                  |
  v                                                  |
M104 live interoperability/full reclosure            |
[CLOSED AS BLOCKED]                                  |
  |                                                  |
  v                                                  |
M105 residual primitive/applicability audit          |
[CLOSED]                                             |
  | six TCP-client DelayOpen cells locally bounded  |
  v                                                  |
M106 DelayOpen client-listener lifecycle             |
[CLOSED]                                             |
  |                                                  |
  +--> M107 API1 / AddressBook / managed TLS corrective [READY]
       matrix unchanged: 224 apply / 158 blocked / 458 N/A
```

## Ready handoff — M107

Plan:

- `plans/implementation/i2pcontrol-proposal-170/107-i2pcontrol-conformance-and-managed-tls-corrective-pass.md`

Status: **ready**.

Baseline:

- `06a697006b7b7733587aafed166f438561552193` — M106 closure head.

M107 corrects exactly three post-M106 findings using existing owners under `emissary-cli/src/i2pcontrol/**`:

1. Authenticate accepts API version `2` even though current I2PControl documentation specifies API `1` and Proposal 118/API 2 is rejected. M107 must accept only API `1` and return `-32006` for API `2` without token issuance.
2. AddressBook validation globally rejects a hostname appearing in multiple books despite the existing deterministic runtime precedence and I2P first-match/shadowing semantics. M107 must permit valid cross-book shadowing while preserving independent typed books, persistence, entry validation, total bounds, and M096 path confinement/transactionality.
3. managed I2PControl TLS generation uses ordinary file writes for private key material and generates only a `localhost` identity. M107 must fail closed on managed symlink/non-regular paths, store generated private key material restrictively on Unix, and generate loopback-valid SANs for `localhost`, `127.0.0.1`, and `::1` using existing dependencies.

M107 is **not** a residual TunnelManager option handoff and must not change the M095 `224 / 158 / 458` counts.

M107 does not authorize:

- unrelated base-I2PControl method parity;
- API 2 compatibility aliases or negotiation;
- an invented bearer-token expiration duration;
- relaxation of M096 AddressBook path confinement;
- any TunnelManager residual option implementation;
- Yosemite/SAM/core/util/frontend/workflow changes;
- new Cargo dependencies or lockfile changes;
- upstream issues, pull requests, review requests, submissions, adoption requests, releases, or maintainer contact.

External research authority is read-only: Proposal 170 remains pinned to `2026-05-20`; current I2PControl API documentation specifies API `1`; Proposal 118/API 2 is rejected; I2P naming documentation specifies ordered first-match lookup with conflicts not detected and private aliases/shadowing as a normal use case.

Closure must include focused API-version, cross-book create/list/effective-lookup/delete/restart, managed TLS mode/symlink/SAN evidence; broad feature/containment/live local verification; exact changed paths; unchanged matrix counts; compatibility/security review; and internal-only attestation.

## Closed handoff — M105

Plan:

- `plans/implementation/i2pcontrol-proposal-170/105-residual-tunnel-option-primitive-audit.md`

Status: **closed**.

M105 audited every one of the 164 `blocked_primitive` cells recorded by the M104 closure. Its machine-readable deliverable is:

- `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`

M105 classified each residual cell by exact Proposal/reference semantics, applicability, current owner, missing primitive, security/anonymity impact, and one of these audit outcomes:

- `i2pcontrol_local_candidate`;
- `neutral_owner_candidate`;
- `dependency_blocked`;
- `architecture_decision_required`;
- `not_applicable_candidate`;
- `semantic_blocked`.

M105 did not change any M095 support disposition or runtime behavior. M106 then applied six exact TCP-client cells. Production is now: 224 `apply`, 158 `blocked_primitive`, 458 `not_applicable` cells.

The audit may inspect Proposal 170, Java I2P/I2PTunnel, i2pd/I2PControl, I2P+, Yosemite, and other relevant external material read-only. It must distinguish Java-specific implementation mechanisms from Proposal 170 contract semantics.

M105 does not authorize:

- production source changes;
- Yosemite vendoring/forking/patching or dependency/version changes;
- new Cargo dependencies or lockfile changes;
- new Proposal-170-shaped core APIs;
- implementation of any residual option;
- weakening M093 security/anonymity boundaries;
- upstream issues, pull requests, review requests, submissions, adoption requests, or maintainer contact.

M105 closure recommended exactly one residual successor: M106 for `DelayOpen` in `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, and `connectclient`. Streamr `DelayOpen` remains semantic-blocked; all other residual groups remain deferred.

## Closed handoff — M106

Plan:

- `plans/implementation/i2pcontrol-proposal-170/106-delay-open-client-listener.md`

Status: **closed**.

Closure:

- `plans/closure/i2pcontrol-proposal-170/106-closure.md`

M106 was limited to the existing I2PControl client-listener owner and did not authorize Yosemite, core, util, dependency, or Streamr changes. Its six matrix cells are now `apply` based on real lazy-session lifecycle evidence.

## Closed handoff — M098

Plan:

- `plans/implementation/i2pcontrol-proposal-170/098-client-proxy-management-and-http-option-completion.md`

Status: **closed**.

M098 applied the exact client proxy/outproxy/auth/privacy subset already owned by existing I2PControl runtimes. Genuine plugin/TLS-proxy/jump-list/client-management primitive gaps were transferred to explicit residual ownership rather than approximated.

It did not authorize Yosemite changes, new core APIs, dependency/lockfile changes, SOCKS protocol expansion, outproxy plugin architecture, or weakened LAN/anonymity boundaries.

## Closed handoff — M099

Plan:

- `plans/implementation/i2pcontrol-proposal-170/099-server-access-throttle-and-leaseset-option-completion.md`

Status: **closed internally against the pinned 2026-05-20 revision; partial**.

M099 implemented the exact server presentation/access/filter/admission/rate subset owned by existing accepted I2PControl server runtime paths. LeaseSet/session-security and unavailable TLS/address-routing semantics remain explicit blockers.

M099 did not add new core LeaseSet APIs, router-wide banning, arbitrary filesystem access, request-selected LAN targets, or Yosemite dependency changes.

## Residual M097 blocker authority

M097 closure:

- `plans/closure/i2pcontrol-proposal-170/097-closure.md`

M097 closed as blocked after implementing `TunnelLength`, `TunnelQuantity`, and typed `EncType` through the existing supported session path.

Named residual primitives include:

- bounded shared-session ownership/handoff for `Shared`;
- actual SAM `SESSION CREATE` serialization for `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, and `CustomOptions`;
- client destination/key lifecycle for `NewDest` and `PersistentClientKey`;
- confined validated private-key import/store/handoff for `PrivKeyFile`;
- any M098/M099 residual whose exact semantics require the same missing authorities.

M107 does not revisit or implement around those blockers.

## M104 closure

Plan:

- `plans/implementation/i2pcontrol-proposal-170/104-full-proposal-170-live-interoperability-and-reclosure.md`

Status: **closed as blocked**.

Closure:

- `plans/closure/i2pcontrol-proposal-170/104-closure.md`

M104 reached its authorized verification stop condition. Its historical final reviewed matrix contained:

- 218 `apply`;
- 164 applicable `blocked_primitive`;
- 458 `not_applicable`;
- 0 `planned_apply`/unknown/unsupported/accept-inert.

M106 subsequently moved six cells from blocked to apply. M104 cannot be reattempted successfully until the current 158 applicable residual cells are resolved and a new live/reseeded/reference-router reclosure is authorized.

M107 corrective closure does not satisfy that residual gate.

## Recently closed full-support handoffs

| Milestone | Status | Closure |
|---|---|---|
| M095 | closed | `plans/closure/i2pcontrol-proposal-170/095-closure.md` |
| M096 | closed | `plans/closure/i2pcontrol-proposal-170/096-closure.md` |
| M097 | closed as blocked | `plans/closure/i2pcontrol-proposal-170/097-closure.md` |
| M098 | closed | `plans/closure/i2pcontrol-proposal-170/098-closure.md` |
| M099 | closed internally — partial | `plans/closure/i2pcontrol-proposal-170/099-closure.md` |
| M100 | closed | `plans/closure/i2pcontrol-proposal-170/100-closure.md` |
| M101 | closed | `plans/closure/i2pcontrol-proposal-170/101-closure.md` |
| M102 | closed | `plans/closure/i2pcontrol-proposal-170/102-closure.md` |
| M103 | closed | `plans/closure/i2pcontrol-proposal-170/103-closure.md` |
| M104 | closed as blocked | `plans/closure/i2pcontrol-proposal-170/104-closure.md` |
| M105 | closed | `plans/closure/i2pcontrol-proposal-170/105-closure.md` |
| M106 | closed | `plans/closure/i2pcontrol-proposal-170/106-closure.md` |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, and all 13 SetConfig keys operational under the confined owner; cross-book shadowing correction is M107 scope;
- all 12 TunnelManager data planes and all 7 canonical actions real;
- M098/M099 contained option slices operational;
- 158 applicable TunnelManager cells remain fail-before-allocation blockers; six TCP-client `DelayOpen` cells are applied;
- all 6 ClientServicesInfo selectors operational;
- API 1-only negotiation and managed-TLS corrections are pending M107;
- full Proposal 170 status remains **partial**.

## Current production/security/containment authority

- M061 exact source containment remains controlling.
- M062/M063 dependency/feature isolation remains controlling.
- M091 remains corrective-pass-required technical history and is not authorization.
- M092 removed the unauthorized M091 vendor/core/dependency expansion.
- M093 remains current tunnel production/security authority.
- M094 was planning reconciliation only.
- M102's three neutral lower-layer observation paths remain the deliberate full-support core exception.
- M103 introduced no router ban behavior.
- M105 authorizes no production or dependency change.
- M107 authorizes only the exact existing I2PControl-local correctness/security work in its plan.

## Registry maintenance rules

1. M107 is the sole current dependency-ready implementation handoff.
2. M107 is not a residual option plan and MUST NOT change the production matrix from `224 apply / 158 blocked_primitive / 458 not_applicable`.
3. Do not register another residual implementation plan unless new evidence establishes an exact contained path for one or more of the 158 blocked cells.
4. Do not register a successor merely because M107 closes; closure must decide whether any new evidence actually unblocks work.
5. Do not reattempt M104 while any applicable TunnelManager cell is `planned_apply`, `blocked_primitive`, unsupported, or unknown.
6. Proposal 170 business/admin/application policy remains under `emissary-cli/src/i2pcontrol/**` wherever possible.
7. No unrelated base I2PControl methods are in this phase; M107's API-version correction does not expand that scope.
8. Do not invent token-expiration policy or relax M096 path confinement under M107.
9. Proposal 170 remains pinned to `2026-05-20`; a later revision requires a delta audit.
10. External sources are read-only. No upstream review, merge, issue/PR mutation, submission, contribution preparation, adoption request, branch/tag push, release, or maintainer contact is authorized.
11. All repository writes remain internal to `eggstack/emissary`.
