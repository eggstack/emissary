# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; RouterInfo corrective truthfulness sequence closed

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

M056 is closed. The corrective RouterInfo truthfulness sequence has no current
dependency-ready successor:

- `054-m049-transit-15s-corrective.md` — closed; transit-15s is explicitly unavailable because no request-independent owner fits the bounded budget.
- `055-m050-network-error-truthfulness-corrective.md` — closed; both unowned v4/v6 network-error rows are unavailable and dead error scaffolding was removed.

Completed corrective successor:

- `056-m049-m050-corrective-reclosure.md` — closed; final integrated matrix is 37 available / 1 neutral / 5 unavailable.

## Why the corrective sequence was reopened

The merged post-M052 head `970252c` was reviewed after the original source-completion sequence closed. That review found three overclaimed rows:

1. `i2p.router.net.bw.transit.15s` is backed by an I2PControl request-local sampler. Traffic history only advances when the field is queried, so first reads and reads after a long API gap can return zero despite recent transit traffic.
2. `i2p.router.net.error` has no canonical Emissary error owner but maps unset internal state to code `0`, whose adopted reference meaning is `No error`.
3. `i2p.router.net.error.v6` has the same defect.

The pre-review `40 available / 1 neutral / 2 unavailable` matrix is therefore no longer accepted. The review-corrected disposition before implementation is 37 available / 1 neutral / 5 unavailable.

M054 may restore transit-15s to available only with a request-independent source. M055 is expected to retain both error rows unavailable unless a real existing owner is discovered. M056 owns the final integrated count/reclosure.

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
| M054 — transit 15s corrective | closed | truthful transit-15s unavailability; request-local sampler removed | `054` closure |
| M055 — network-error truthfulness | closed | both error rows unavailable; dead error scaffold removed | `055` closure |
| M056 — corrective integration reclosure | closed | no production changes; final 43-row audit accepted | `056-closure.md` |

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
- `056-m049-m050-corrective-reclosure.md`.

## Corrective boundaries

Machine-readable authority: `045-052-routerinfo-source-boundary.toml`.

M054:

- core production allowance: `emissary-core/src/events.rs` only;
- no tunnel/transport/router/NetDB data-plane path changes;
- no new I2PControl-specific sampler task or polling daemon;
- required regression: traffic/source history advances with zero RouterInfo reads, and a later read plus a >15-second API-gap case still reflects the current reference window.

M055:

- core production allowance: `emissary-core/src/events.rs` and `emissary-core/src/inspection.rs` only for dead error-scaffold cleanup;
- no transport/SSU2 changes to retained status/testing behavior;
- required regression: v4/v6 error requests fail unavailable and never serialize `0` solely because internal state is unset.

M056:

- no production changes;
- closure-only integrated source audit and retained regression matrix.

## Scope and ownership rule

Proposal 170 policy remains under `emissary-cli/src/i2pcontrol/**`: field/source disposition, aggregation, joins, sorting, bounds, Base64/numeric wire mapping, JSON types/serialization, compatibility behavior, and sanitized errors.

Changes outside I2PControl are exceptional and may only expose neutral bounded read-only facts from canonical owners. They must not contain Proposal 170 terminology or mutable control authority.

A request-local cache/history is not sufficient evidence for a router-owned rolling metric. A numeric mapping is not sufficient evidence for a network-error source owner.

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

Every milestone must inspect its accepted dependency head, pin exact semantics before source-disposition changes, use focused source/wire regressions, preserve no-feature behavior, remain inside its path budget, and create an independent closure record. Material scope expansion is a blocker, not permission to improvise.

Only the registry advances the next handoff to `ready`. No current plan is
dependency-ready after M056; M051 remains blocked behind its absent substantive
owners.

## Verification rule

Use focused semantic tests first, then the bounded package matrix. Typical broad commands remain:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
git diff --check
```

Use targeted formatting because the repository's stable/nightly formatter qualification is already documented. Do not add CI/release/coverage/fuzz/soak infrastructure.

## Final status rule

M056 independently derived and validated the final matrix as 37 available / 1
protocol-permitted neutral / 5 unavailable. RouterInfo source completion remains
partial under current owners because news, banned peers, transit-15s, and both
network-error rows are unavailable. Broader Proposal 170 support also retains
unrelated previously accepted partial dimensions.

## Historical evidence

M040–M044 remain closed retained evidence. M039 remains historical-invalidated.
The M045 stale-snapshot attempt and blocked closure remain corrective history;
M053/M045 and M046–M048 remain accepted. M049/M050 historical closures are
partially superseded by M054/M055 and this M056 reclosure. M052's `40/1/2`
matrix is historical-invalidated and superseded by the accepted M056 `37/1/5`
audit.
