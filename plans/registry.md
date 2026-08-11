# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies and interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency or evidence requirement prevents progress.
- **closing** — implementation landed and independent closure evidence is being gathered.
- **closed** — closure record accepted.
- **closed internally against pinned revision** — internal closure accepted against an explicitly named revision of an open external specification; does not imply upstream review or acceptance.
- **partial Proposal 170 support** — exact supported dimensions are closed, but one or more pinned source/runtime capabilities remain truthfully unavailable.
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 source/truthfulness | partial Proposal 170 support; M057 closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | no source-completion handoff | M051 remains blocked by absent substantive news/ban owners; accepted RouterInfo matrix remains 37/1/5 |
| I2PControl Proposal 170 containment | active | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061 — independent containment reclosure | M060 closed at `6085eca`; M061 ready |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|
| I2PControl Proposal 170 containment | M061 — independent containment reclosure | ready | `plans/implementation/i2pcontrol-proposal-170/061-containment-reclosure.md` | M060 closure accepted at `6085eca` |

M061 is now registered as dependency-ready because M060 is closed, per `plans/003-planning-process.md`.

## Blocked roadmap successors

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M051 — router news and banned peers | blocked with accepted semantic limitation | `plans/implementation/i2pcontrol-proposal-170/051-routerinfo-news-and-banned-peer-semantics.md` | substantive news/ban owners absent; no current owner-specific plan authorized |

M061 is the remaining dependency-ordered containment milestone and is now the sole dependency-ready successor. M051 remains independently blocked by absent substantive news/ban owners.

## Current containment corrective scope

Planning baseline: `adb2f52543764b267b2bcb282d093111001ae4b2` — merged M057 closure head.

Pinned upstream compare baseline/merge base: `eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f`.

The current review established that most Proposal 170 policy is correctly concentrated under `emissary-cli/src/i2pcontrol/**`, but the physical upstream delta still spans original CLI/runtime and audited `emissary-core` paths. The objective of M058–M061 is to establish and then enforce the **minimum justified non-`i2pcontrol` delta** without changing supported behavior.

The containment sequence is:

1. M058 — audit-only complete non-`i2pcontrol` fork-delta ledger; zero production changes.
2. M059 — original CLI/runtime adapter containment; closed with accepted M059 closure.
3. M060 — audited-core observation seam consolidation; closed with accepted core disposition.
4. M061 — production-free independent reclosure and current static containment guard; ready after M060 closure.

Only M061 is currently executable in the containment sequence.

### Containment ownership rule

- Proposal 170/I2PControl wire semantics, selectors, support disposition, administrative persistence, aggregation/bounds, authentication/TLS policy, and sanitized error mapping remain under `emissary-cli/src/i2pcontrol/**`.
- Original CLI/runtime files may retain only minimum feature/configuration composition, ordinary legacy runtime behavior, and neutral owner-local adapters required by accepted behavior.
- `emissary-core` may retain only neutral bounded passive read-only observations that cannot be truthfully obtained at a higher canonical owner.
- A deep SAM/NTCP2/SSU2/tunnel hook is not justified merely because current code uses it. Retention requires evidence that moving the fact upward would lose truth, ordering, or bounds.
- The target is not zero outside changes; it is zero **unjustified** outside changes.

### M058 production authority

Production changes: none.

M058 must create a machine-readable `058-containment-ledger.toml` that classifies every changed non-`i2pcontrol` production path as required composition, required owner seam, candidate revert, candidate consolidation, unrelated/accidental, or uncertain. `uncertain` paths cannot be modified by later milestones until resolved.

M058 closure must prove the inventory is complete and freeze the exact M059 path budget plus the provisional M060 core budget.

## Retained and corrective milestone disposition

| Handoff | Current status | Evidence / correction |
|---|---|---|
| M037 — earlier containment boundary reduction | closed historical containment evidence | `037-closure.md`; later RouterInfo source work expanded observation paths, so its manifest is not the final current authority |
| M045 / M053 — known-peer directory corrective | closed | live `ProfileStorage` source accepted by `053-closure.md` |
| M046 — active-peer inventory and limits | closed | `046-closure.md` |
| M047 — active-peer statistics | closed | `047-closure.md` |
| M048 — tunnel-pool counts/details | closed | `048-closure.md` |
| M049 — rolling metrics/queues | corrected/closed through M054 and M056 | recent success + queue/TBM retained; transit 15s explicitly unavailable |
| M050 — v4/v6 network state | corrected/closed through M055 and M056 | status.v6 + testing v4/v6 retained; error rows unavailable with no canonical owner |
| M051 — news/banned peers | blocked with accepted limitation | `051-closure.md`; both rows remain unavailable |
| M052 — integration reclosure | corrected/closed through M056 | historical `40/1/2` matrix superseded by accepted `37/1/5` audit |
| M054–M056 | closed | transit/error truthfulness and integrated source reclosure accepted |
| M057 | closed | planning-record consistency accepted; no production changes |
| M058 | closed | `058-closure.md` accepted; 47-path ledger and M059/M060 budgets frozen |
| M059 | closed | `059-closure.md`; exact original-CLI budget implemented with no core changes |
| M060 | closed | `060-closure.md`; 23 retained core paths, 9 formatting-only paths reverted, no new core path |
| M061 | ready | M060 closure accepted at `6085eca`; final current-boundary reclosure is next |

## Accepted Proposal 170 support state

The accepted RouterInfo source matrix remains exactly:

- 43 canonical Proposal 170 RouterInfo additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable: transit 15s, news, banned peers, and both v4/v6 network-error rows.

The historical `970252c` M052-era 40/1/2 claim remains historical only. M054/M055 corrected three overclaims and M056 accepted 37/1/5. The containment roadmap is not authorized to change this matrix.

Unsupported Emissary tunnel data planes remain out of scope. Do not implement them to improve Proposal 170 completeness.

## Prohibited scope throughout the containment sequence

- new HTTP, HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, or other unsupported tunnel data planes;
- startup task adoption/control;
- router/peer selection, NetDB protocol/discovery, tunnel selection/build/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP behavior changes;
- fabricated RouterInfo values or placeholder promotion;
- new owner solely for transit 15s, network errors, news, or banned peers;
- public export of mutable ProfileStorage/NetDB/router authority for inspection convenience;
- sockets, keys, mutable session/tunnel/transport handles, channels, or message payloads crossing the inspection boundary;
- new network probes, polling daemons, persistent metric stores, background sampler tasks, generalized event buses, news feeds, or ban engines solely for I2PControl;
- broad crate/workspace extraction or service refactor merely to reduce file count;
- `.github/workflows/**`, remote CI expansion, release/publishing, coverage, fuzz, soak, platform matrices, or generated evidence bundles;
- upstream issues, pull requests, reviews, submissions, adoption, merge, maintainer contact, contribution preparation, branches/tags/releases, or connector writes against upstream/third-party repositories.

## Pinned authority

Proposal 170 remains pinned internally to revision `2026-05-20` for this workstream. A materially changed external revision blocks affected implementation/closure and requires an explicit contract-rebase plan.

Upstream `eepnet/emissary` is accessed read-only for containment comparison. The M058 pinned compare is against the fork merge base `9b43484a`; upstream advancement does not silently change the audit baseline.

## Registry maintenance rules

1. M061 is the sole dependency-ready containment handoff; M060 and M059 are closed.
2. Register M061 only after M060 closure; this condition is now satisfied.
3. Preserve M037 as historical evidence rather than rewriting its boundary to include later RouterInfo work.
4. Preserve M053/M045 and M046–M048 closure history unless a direct new defect is demonstrated.
5. Preserve M049/M050/M052 historical records while retaining only their named superseded findings.
6. M054–M057 remain closed and are not reopened by containment refactoring absent a direct behavior defect.
7. Keep Proposal 170 policy under I2PControl; outside changes must be neutral owner/composition seams only.
8. Do not mark a retained outside path required until its canonical owner, consumer, necessity, bounds, and regression evidence are recorded.
9. Keep verification local/package-scoped and proportional; do not expand CI/release apparatus.
10. Overall Proposal 170 remains partial; containment completion does not imply source completeness.
11. M051 remains blocked by absent substantive news/ban owners.
12. No upstream interaction is authorized.
