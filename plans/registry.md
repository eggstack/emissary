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
| I2PControl Proposal 170 full-support completion | active | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | M108 | M107 closed at `27a0376`; M108 repairs legacy managed-TLS permissions; M104 independently remains closed as blocked with 158 TunnelManager residual cells |
| I2PControl Proposal 170 source/truthfulness | RouterInfo source line closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | none | current RouterInfo matrix: 42 available / 1 protocol-permitted neutral / 0 unavailable |
| I2PControl Proposal 170 containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M108 regression scope | M061/M062/M063 rules remain controlling; M108 authorizes no lower-layer production path |
| I2PControl tunnel runtime | all 12 data planes real | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | option semantics only | do not redesign data planes for M108 or residual option parity |
| I2PControl tunnel security | closed at M093 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | no tunnel-security corrective handoff | M108 concerns the I2PControl administrative listener's managed key storage, not TunnelManager TLS/LeaseSet scope |

## Current full-support sequence

M104 reached the final residual gate and closed as blocked. M105 audited the 164 blocked cells, and M106 implemented the only dependency-ready residual subset: six TCP-client `DelayOpen` cells. M107 then corrected API-version negotiation, AddressBook shadowing, and fresh managed-TLS key/SAN handling without touching the TunnelManager inventory.

A post-M107 review found one bounded security gap in the upgrade path: existing regular managed TLS directories/private keys created before M107 may retain permissive Unix modes and be reused unchanged, while temporary private-key files are restricted only after creation. That defect already has an I2PControl-local owner and requires no new dependency or lower-layer primitive. M108 is therefore the sole current dependency-ready corrective handoff.

Production remains `224 apply / 158 blocked_primitive / 458 not_applicable` TunnelManager cells.

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
  v                                                  |
M107 API1 / AddressBook / fresh managed TLS corrective
[CLOSED]                                             |
  | upgrade-path managed-key permission gap         |
  v                                                  |
M108 managed TLS upgrade-permission corrective       |
[READY]                                              |
  |                                                  |
  +--> no residual-option successor implied          |
      matrix unchanged: 224 apply / 158 blocked / 458 N/A
```

## Ready handoff — M108

Plan:

- `plans/implementation/i2pcontrol-proposal-170/108-managed-tls-upgrade-permission-corrective-pass.md`

Status: **ready**.

Baseline:

- `a108b1b62f3ad9d79fe455ccf3910f96d7a5e06f` — M107 planning closure head.

M108 corrects one narrowly bounded post-M107 security issue and planning-state drift:

1. on Unix, existing Emissary-managed `i2pcontrol-certs/` state must be made restrictive before child/key material is read: managed directory `0700`, managed private key `0600`, with type/mode revalidation and fail-closed startup on repair failure;
2. newly created private-key temporary files must request `0600` at inode creation through the standard-library Unix `OpenOptionsExt` path rather than relying on a post-write chmod as the first confidentiality boundary;
3. valid managed key/certificate bytes must remain stable when only permissions need repair;
4. explicit operator TLS certificate/key paths remain untouched;
5. stale planning text describing M107 as ready/current/pending must be reconciled during M108 implementation/closure.

M108 is **not** a residual TunnelManager option handoff and MUST NOT change the M095 `224 / 158 / 458` counts.

M108 does not authorize:

- unrelated base-I2PControl method parity;
- API 2 compatibility or token-expiration policy;
- AddressBook path-policy changes;
- certificate parsing or rotation solely to migrate pre-M107 SANs;
- any TunnelManager residual option implementation;
- Yosemite/SAM/core/util/frontend/workflow changes;
- new Cargo dependencies or lockfile changes;
- workspace rustfmt or GitHub Pages corrective work;
- upstream issues, pull requests, review requests, submissions, adoption requests, releases, or maintainer contact.

## Closed handoff — M107

Plan:

- `plans/implementation/i2pcontrol-proposal-170/107-i2pcontrol-conformance-and-managed-tls-corrective-pass.md`

Closure:

- `plans/closure/i2pcontrol-proposal-170/107-closure.md`

Status: **closed** at implementation head:

- `27a0376` — `fix(i2pcontrol): close M107 conformance corrective pass`.

M107 successfully landed:

- API version `1` as the sole accepted Authenticate version, with API `2` returning `-32006` before token issuance;
- independent AddressBook cross-book shadowing with deterministic private > local > router > published effective lookup;
- fresh managed TLS private-key/directory final modes and fail-closed symlink/non-regular handling;
- fresh managed certificate SAN coverage for `localhost`, `127.0.0.1`, and `::1`.

M108 does not reopen those closed contract changes. It repairs only the legacy/upgraded managed-permission path and create-time private-key mode.

## Residual TunnelManager blocker authority

M104 remains **closed as blocked**. M105 audited the residuals and M106 moved six exact TCP-client `DelayOpen` cells from blocked to apply.

Current production matrix:

- 224 `apply`;
- 158 applicable `blocked_primitive`;
- 458 `not_applicable`;
- 0 `planned_apply`, unknown, unsupported, or accept-inert cells.

Named residual primitive families remain:

- bounded shared-session ownership/handoff for `Shared`;
- actual SAM `SESSION CREATE` serialization for `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, and `CustomOptions`;
- client destination/key lifecycle for `NewDest` and `PersistentClientKey`;
- confined validated private-key import/store/handoff for `PrivKeyFile`;
- residual proxy/plugin/TLS/jump-list primitives;
- remaining client lifecycle semantics;
- server TLS/address-routing ownership;
- LeaseSet security/serializer/key handoff.

M108 does not revisit, implement around, or reclassify those blockers.

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
| M107 | closed | `plans/closure/i2pcontrol-proposal-170/107-closure.md` |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys, and cross-book shadowing semantics operational under the confined owner;
- all 12 TunnelManager data planes and all 7 canonical actions real;
- M098/M099 contained option slices operational;
- 158 applicable TunnelManager cells remain fail-before-allocation blockers; six TCP-client `DelayOpen` cells are applied;
- all 6 ClientServicesInfo selectors operational;
- API 1-only negotiation operational;
- M107 fresh managed-TLS key final modes, symlink/type guards, and loopback SANs operational;
- legacy/upgraded managed TLS mode repair and create-time private-key mode are pending M108;
- full Proposal 170 status remains **partial**.

## Current production/security/containment authority

- M061 exact source containment remains controlling.
- M062/M063 dependency/feature isolation remains controlling.
- M091 remains corrective-pass-required technical history and is not authorization.
- M092 removed the unauthorized M091 vendor/core/dependency expansion.
- M093 remains current tunnel production/security authority.
- M102's three neutral lower-layer observation paths remain the deliberate full-support core exception.
- M103 introduced no router ban behavior.
- M105 authorizes no production or dependency change.
- M107 is closed and remains authority for its landed protocol/AddressBook/fresh-TLS behavior.
- M108 authorizes only the exact managed-TLS upgrade-permission correction in its implementation plan.

## Registry maintenance rules

1. M108 is the sole current dependency-ready implementation handoff.
2. M108 is not a residual option plan and MUST NOT change the production matrix from `224 apply / 158 blocked_primitive / 458 not_applicable`.
3. Do not register another residual implementation plan unless new evidence establishes an exact contained path for one or more of the 158 blocked cells.
4. Do not register a successor merely because M108 closes; closure must decide whether new evidence actually unblocks work.
5. Do not reattempt M104 while any applicable TunnelManager cell is `planned_apply`, `blocked_primitive`, unsupported, or unknown.
6. Proposal 170 business/admin/application policy remains under `emissary-cli/src/i2pcontrol/**` wherever possible.
7. M108 production scope is managed TLS only under `emissary-cli/src/i2pcontrol/**`; explicit operator TLS material must remain untouched.
8. No unrelated base I2PControl methods, token-expiration policy, AddressBook path relaxation, new dependency, or lower-layer architecture work is authorized by M108.
9. Proposal 170 remains pinned to `2026-05-20`; a later revision requires a delta audit.
10. External sources are read-only. No upstream review, merge, issue/PR mutation, submission, contribution preparation, adoption request, branch/tag push, release, or maintainer contact is authorized.
11. All repository writes remain internal to `eggstack/emissary`.