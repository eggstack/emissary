# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; RouterInfo source completion active; M045 conditionally closed

This directory contains bounded internal implementation/closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`;
- retained M044 closure: `plans/closure/i2pcontrol-proposal-170/044-closure.md`.

Pinned external authority: Proposal 170 `I2PControl Expansion`, Open, revision `2026-05-20`, plus existing I2PControl authentication/JSON-RPC contract.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications/reference implementations are read-only. No plan authorizes an upstream issue, pull request, review, discussion, submission, adoption request, merge request, maintainer outreach, contribution package, branch/tag/release push, or connector write against an upstream/third-party repository.

## Current handoff

M045 is in closure review; no later plan is dependency-ready:

- `045-routerinfo-known-peer-directory.md` — conditionally closed; live-source evidence remains outstanding.

Per `plans/003-planning-process.md`, later plans exist for handoff clarity but are not registered as executable until their hard dependency closes.

## RouterInfo source-completion sequence

| Handoff | Status | Target fields | Hard dependency |
|---|---|---:|---|
| M045 — known-peer directory | conditionally closed | 3 | M044 closed |
| M046 — active-peer inventory + transport limits | blocked | 4 | M045 live-source closure condition |
| M047 — active-peer statistics | blocked | 1 | M046 closure |
| M048 — tunnel-pool counts/details | blocked | 7 | M047 closure |
| M049 — rolling transit/build metrics + queues | blocked | 4 | M048 closure |
| M050 — v4/v6 network state | blocked | 5 | M049 closure |
| M051 — router news + banned peers | blocked | 2 | M050 closure |
| M052 — integration/containment reclosure | blocked | validation | M045–M051 accepted |

Plans:

- `045-routerinfo-known-peer-directory.md`;
- `046-routerinfo-active-peer-inventory-and-limits.md`;
- `047-routerinfo-active-peer-stats.md`;
- `048-routerinfo-tunnel-pool-sources.md`;
- `049-routerinfo-rolling-metrics-and-queues.md`;
- `050-routerinfo-network-state-sources.md`;
- `051-routerinfo-news-and-banned-peer-semantics.md`;
- `052-routerinfo-source-integration-and-reclosure.md`.

## Scope and ownership rule

The target is exactly the 26 RouterInfo rows currently classified unavailable. All Proposal 170 policy remains under `emissary-cli/src/i2pcontrol/**`: field/source disposition, rolling windows, aggregation, joins, sorting, bounds, wire numeric mappings, JSON types/serialization, compatibility behavior, and sanitized errors.

Changes outside `i2pcontrol/**` are exceptional and may only expose neutral bounded read-only facts from canonical owners. They must not contain Proposal 170 terminology or mutable control authority. The machine-readable per-milestone production budgets are in `045-052-routerinfo-source-boundary.toml`.

M045 is intentionally expected to require no core production change. M051/M052 authorize no core production change. M046–M050 enumerate the only core paths they may touch.

## Source groups

The 26 rows are decomposed into:

- known public peer directory: 3;
- active peer list/info + NTCP/SSU limits: 4;
- active peer stats: 1;
- participating/exploratory/client tunnel counts/details: 7;
- transit 15s/recent tunnel success/queue/TBM queue: 4;
- v4/v6 status/error/testing: 5;
- router news/banned peers: 2.

The last two are semantic-risk fields. Empty values are not accepted as implementation evidence unless the pinned contract/reference proves they are the authoritative current state for a router without those subsystems.

## Cross-cutting prohibitions

Throughout M045–M052, do not:

- add a missing tunnel data plane;
- modify router/peer selection, NetDB protocol/discovery, tunnel selection/build/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP behavior;
- expose sockets, private/session keys, mutable transport/tunnel/session handles, command channels, or message payloads to I2PControl;
- add new network probes, polling daemons, persistent time-series stores, news feed/downloader, or ban engine solely for observability;
- substitute fabricated zero/false/empty/null/adjacent metrics for unavailable state;
- modify AddressBook, proxy/UI, workflows, release/publishing, or unrelated code;
- broaden base I2PControl compatibility or method inventory;
- interact with upstream write channels.

## Handoff discipline

Every milestone must inspect the accepted dependency head, pin exact field semantics before source disposition changes, use focused source/wire tests, preserve no-feature behavior, remain inside its path budget, create an implementation disposition and independent closure record, and report any required scope expansion rather than silently performing it.

Only the registry advances the next handoff to `ready` after its hard dependency closes.

## Verification rule

Use focused tests first, then the bounded package matrix. Typical broad commands are:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
git diff --check
```

Each plan narrows/adds focused commands. Use targeted formatting because the repository's formatter baseline mismatch is already documented. Do not add CI/release/coverage/fuzz/soak infrastructure.

## Final status rule

If all 26 become truthful operational sources and M052 accepts the final head, the RouterInfo dimension may move to 42 available + 1 protocol-permitted neutral + 0 unavailable and be closed internally against the pinned revision.

This does not automatically close full Proposal 170: unrelated unsupported tunnel families and other accepted partial dimensions remain outside this roadmap. If M051 proves that news or banned-peer semantics require an absent substantive subsystem, retain the field unavailable and report RouterInfo source completion incomplete rather than expanding scope.

## Historical evidence

M040–M044 remain closed retained evidence. M039 remains historical-invalidated. The new roadmap supersedes only the old statement that the 26 RouterInfo sources were outside authorized scope; it does not rewrite prior closure history.
