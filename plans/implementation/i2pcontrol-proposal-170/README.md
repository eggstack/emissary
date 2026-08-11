# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; M057 post-M056 planning-record consistency corrective ready

This directory contains bounded internal implementation/closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`.

Pinned external authority: Proposal 170 `I2PControl Expansion`, Open, revision `2026-05-20`, plus read-only reference implementation evidence where the proposal adopts or leaves semantics terse.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications/reference implementations are read-only. No plan authorizes an upstream issue, pull request, review, discussion, submission, adoption request, merge request, maintainer outreach, contribution package, branch/tag/release push, or connector write against an upstream/third-party repository.

## Current handoff

M054, M055, and M056 are closed. M057 is now the sole dependency-ready handoff:

- `057-post-m056-planning-record-consistency-corrective.md` — ready; documentation/control-surface consistency only, with zero production authority.

The accepted production disposition remains unchanged:

- `054-m049-transit-15s-corrective.md` — closed; transit-15s is explicitly unavailable because no request-independent owner fits the bounded budget.
- `055-m050-network-error-truthfulness-corrective.md` — closed; both unowned v4/v6 network-error rows are unavailable and dead error scaffolding was removed.
- `056-m049-m050-corrective-reclosure.md` — closed; final integrated matrix is 37 available / 1 neutral / 5 unavailable.

M057 exists only because a small number of active planning sentences remained stale after M056: the roadmap dependency graph still labels M055 ready, and historical/current source-count wording must consistently distinguish the pre-corrective `970252c` 40/1/2 claim from the accepted post-M056 37/1/5 state. M057 must not change code, source disposition, runtime support, or the accepted closure records.

## Why the corrective source sequence was reopened

The merged post-M052 head `970252c` was reviewed after the original source-completion sequence closed. That review found three overclaimed rows:

1. `i2p.router.net.bw.transit.15s` was backed by an I2PControl request-local sampler. Traffic history only advanced when the field was queried, so first reads and reads after a long API gap could return zero despite recent transit traffic.
2. `i2p.router.net.error` had no canonical Emissary error owner but mapped unset internal state to code `0`, whose adopted reference meaning is `No error`.
3. `i2p.router.net.error.v6` had the same defect.

The pre-corrective `40 available / 1 neutral / 2 unavailable` matrix is historical only. M054/M055 corrected production truthfulness, and M056 accepted the final 37 available / 1 neutral / 5 unavailable matrix.

## RouterInfo source-completion sequence

| Handoff | Status | Target/disposition | Dependency |
|---|---|---|---|
| M053 — M045 live ProfileStorage corrective | closed | corrected M045's 3 known-peer fields | accepted `053-closure.md` |
| M045 — known-peer directory | closed | 3 fields | corrected through M053 |
| M046 — active-peer inventory + finite limits | closed | 4 fields | accepted `046-closure.md` |
| M047 — active-peer statistics | closed | 1 field | accepted `047-closure.md` |
| M048 — tunnel-pool counts/details | closed | 7 fields | accepted `048-closure.md` |
| M049 — rolling transit/build metrics + queues | corrected/closed through M054 and M056 | recent success + queue/TBM retained; transit 15s unavailable | accepted corrective closures |
| M050 — v4/v6 network state | corrected/closed through M055 and M056 | status.v6 + testing v4/v6 retained; error v4/v6 unavailable | accepted corrective closures |
| M051 — router news + banned peers | blocked with accepted limitation | 2 fields remain unavailable | retained `051-closure.md` |
| M052 — integration/containment reclosure | corrected/closed through M056 | historical `40/1/2` count superseded by final `37/1/5` audit | `056-closure.md` |
| M054 — transit 15s corrective | closed | truthful transit-15s unavailability; request-local sampler removed | `054-closure.md` |
| M055 — network-error truthfulness | closed | both error rows unavailable; dead error scaffold removed | `055-closure.md` |
| M056 — corrective integration reclosure | closed | no production changes; final 43-row audit accepted | `056-closure.md` |
| M057 — post-M056 planning-record consistency | ready | reconcile active planning statuses/baselines only; no production changes | M054–M056 accepted closures |

Plans:

- `045-routerinfo-known-peer-directory.md`;
- `046-routerinfo-active-peer-inventory-and-limits.md`;
- `047-routerinfo-active-peer-stats.md`;
- `048-routerinfo-tunnel-pool-sources.md`;
- `049-routerinfo-rolling-metrics-and-queues.md`;
- `050-routerinfo-network-state-sources.md`;
- `051-routerinfo-news-and-banned-peer-semantics.md`;
- `052-routerinfo-source-integration-and-reclosure.md`;
- `053-m045-live-profile-storage-corrective.md`;
- `054-m049-transit-15s-corrective.md`;
- `055-m050-network-error-truthfulness-corrective.md`;
- `056-m049-m050-corrective-reclosure.md`;
- `057-post-m056-planning-record-consistency-corrective.md`.

## Corrective boundaries

Machine-readable authority: `045-052-routerinfo-source-boundary.toml`.

M054:

- core production allowance: `emissary-core/src/events.rs` only;
- no tunnel/transport/router/NetDB data-plane path changes;
- no new I2PControl-specific sampler task or polling daemon.

M055:

- core production allowance: `emissary-core/src/events.rs` and `emissary-core/src/inspection.rs` only for dead error-scaffold cleanup;
- no transport/SSU2 changes to retained status/testing behavior.

M056:

- no production changes;
- closure-only integrated source audit and retained regression matrix.

M057:

- no production changes and no core paths;
- planning/control-surface consistency only;
- broad Rust verification is not required unless the changed-file boundary is violated;
- accepted M054–M056 closure records remain immutable historical evidence.

## Scope and ownership rule

Proposal 170 policy remains under `emissary-cli/src/i2pcontrol/**`: field/source disposition, aggregation, joins, sorting, bounds, Base64/numeric wire mapping, JSON types/serialization, compatibility behavior, and sanitized errors.

Changes outside I2PControl are exceptional and may only expose neutral bounded read-only facts from canonical owners. They must not contain Proposal 170 terminology or mutable control authority. M057 is even narrower: it has no production authority at all.

## Cross-cutting prohibitions

Throughout the roadmap, do not:

- add a missing tunnel data plane;
- modify router/peer selection, NetDB protocol/discovery, tunnel selection/build/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP behavior;
- expose sockets, private/session keys, mutable router/storage/transport/tunnel/session handles, command channels, or message payloads to I2PControl;
- add new network probes, polling daemons, persistent time-series stores, or a new background sampler solely for I2PControl;
- add a news feed/downloader or ban engine solely for telemetry;
- substitute fabricated zero/false/empty/null/adjacent values for unavailable state;
- map missing source authority to the positive `No error` code;
- modify AddressBook, proxy/UI, workflows, release/publishing, or unrelated code;
- broaden base I2PControl compatibility or method inventory;
- interact with upstream write channels.

## Handoff discipline

Every milestone must inspect its accepted dependency head, preserve exact semantics and scope, remain inside its path budget, and create an independent closure record. Material scope expansion is a blocker, not permission to improvise.

Only the registry advances the next handoff to `ready`. M057 is the sole current ready plan. After accepted M057 closure, no successor should be registered unless separately authorized; M051 remains blocked behind absent substantive news/ban owners.

## Verification rule

M057 uses targeted planning-integrity checks, `git diff --check`, and a changed-path audit. It must not rerun or expand the broad Rust/CI apparatus solely for documentation edits. The previously accepted M056 product verification remains controlling for production behavior.

Do not add CI/release/coverage/fuzz/soak infrastructure.

## Final status rule

M056 independently derived and validated the final matrix as 37 available / 1 protocol-permitted neutral / 5 unavailable. RouterInfo source completion remains partial under current owners because news, banned peers, transit-15s, and both network-error rows are unavailable. Broader Proposal 170 support also retains unrelated previously accepted partial dimensions.

M057 may close only planning-record consistency. It cannot change this final support disposition.

## Historical evidence

M040–M044 remain closed retained evidence. M039 remains historical-invalidated. The M045 stale-snapshot attempt and blocked closure remain corrective history; M053/M045 and M046–M048 remain accepted. M049/M050 historical closures are partially superseded by M054/M055 and M056. M052's `40/1/2` matrix is historical-invalidated and superseded by the accepted M056 `37/1/5` audit.
