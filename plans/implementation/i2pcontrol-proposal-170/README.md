# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; RouterInfo source completion active; M052 ready

This directory contains bounded internal implementation/closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`;
- retained M044 closure: `plans/closure/i2pcontrol-proposal-170/044-closure.md`;
- blocked M045 closure: `plans/closure/i2pcontrol-proposal-170/045-closure.md`.
- accepted M049 closure: `plans/closure/i2pcontrol-proposal-170/049-closure.md`.

Pinned external authority: Proposal 170 `I2PControl Expansion`, Open, revision `2026-05-20`, plus existing I2PControl authentication/JSON-RPC contract.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications/reference implementations are read-only. No plan authorizes an upstream issue, pull request, review, discussion, submission, adoption request, merge request, maintainer outreach, contribution package, branch/tag/release push, or connector write against an upstream/third-party repository.

## Current handoff

M053 is closed through `plans/closure/i2pcontrol-proposal-170/053-closure.md`.
M045 is corrected/closed through that record. M050 is now the sole
dependency-ready handoff:

- `053-m045-live-profile-storage-corrective.md` — closed; corrected M045's rejected stale startup-snapshot source.

M045 is corrected/closed through M053's accepted independent closure, which records the three known-peer fields as live. M046 is closed through `046-closure.md`; M047 is closed through `047-closure.md`; M048 is closed through `048-closure.md`; M049 is closed through `049-closure.md`; M050 is closed through `050-closure.md`; M051 is blocked by its accepted semantic disposition and M052 is ready to perform the final integration/reclosure review.

## RouterInfo source-completion sequence

| Handoff | Status | Target | Hard dependency |
|---|---|---|---|
| M053 — M045 live ProfileStorage corrective | closed | correct M045's 3 fields | accepted closure `053-closure.md` |
| M045 — known-peer directory | closed | 3 fields | corrected through M053 closure |
| M046 — active-peer inventory + transport limits | closed | 4 fields | `046-closure.md` |
| M047 — active-peer statistics | closed | 1 field | M046 closure; `047-closure.md` |
| M048 — tunnel-pool counts/details | closed | 7 fields | `048-closure.md` |
| M049 — rolling transit/build metrics + queues | closed | 4 fields | `049-closure.md` |
| M050 — v4/v6 network state | closed | 5 fields | `050-closure.md` |
| M051 — router news + banned peers | blocked | 2 fields retained unavailable | `051-closure.md` |
| M052 — integration/containment reclosure | ready | validation | M045–M051 accepted or semantically blocked |

Plans:

- `045-routerinfo-known-peer-directory.md`;
- `046-routerinfo-active-peer-inventory-and-limits.md`;
- `047-routerinfo-active-peer-stats.md`;
- `048-routerinfo-tunnel-pool-sources.md`;
- `049-routerinfo-rolling-metrics-and-queues.md`;
- `050-routerinfo-network-state-sources.md`;
- `051-routerinfo-news-and-banned-peer-semantics.md`;
- `052-routerinfo-source-integration-and-reclosure.md`;
- `053-m045-live-profile-storage-corrective.md`.

## M053 corrective boundary

M045's zero-core-change assumption was disproven by closure: `ProfileStorage` contains the canonical live directory, but `emissary-cli` cannot enumerate it because the required `Bucket` type is private to `emissary-core`. The prior workaround retained a one-shot `CoreSnapshot` and was correctly rejected as startup-stale.

M053 authorizes exactly two core production paths:

- `emissary-core/src/inspection.rs`;
- `emissary-core/src/router/mod.rs`.

Its preferred design is a neutral cloneable request-time inspection handle that privately retains the canonical `ProfileStorage` inside core and returns only bounded owned public RouterIds and serialized public RouterInfo. A narrow `Router` accessor exposes that inspection handle to composition. The plan explicitly forbids modifying `profile.rs`, `router/context.rs`, NetDB, or `lib.rs` public re-exports unless another corrective disposition is created.

The required regression constructs the source first, mutates canonical ProfileStorage second, and proves a later snapshot from the same source instance observes the new/current peer data. This is the evidence the original M045 fixtures lacked.

## Scope and ownership rule

The remaining target is exactly the 2 RouterInfo rows currently classified unavailable. Proposal 170 policy remains under `emissary-cli/src/i2pcontrol/**`: field/source disposition, rolling windows, aggregation, joins, sorting, bounds, Base64/numeric wire mapping, JSON types/serialization, compatibility behavior, and sanitized errors.

Changes outside `i2pcontrol/**` are exceptional and may only expose neutral bounded read-only facts from canonical owners. They must not contain Proposal 170 terminology or mutable control authority. Machine-readable budgets, including the M053 corrective overlay, are in `045-052-routerinfo-source-boundary.toml`.

## Source groups

The RouterInfo source-completion subset is decomposed into:

- known public peer directory: 3 (closed);
- active peer list/info + NTCP/SSU limits: 4 (closed);
- active peer stats: 1;
- participating/exploratory/client tunnel counts/details: 7;
- transit 15s/recent tunnel success/queue/TBM queue: 4 (closed);
- v4/v6 status/error/testing: 5;
- router news/banned peers: 2.

The last two remain semantic-risk fields. Empty values are not implementation evidence unless the pinned contract/reference proves they are authoritative current state for a router without those subsystems.

## Cross-cutting prohibitions

Throughout the roadmap, do not:

- add a missing tunnel data plane;
- modify router/peer selection, NetDB protocol/discovery, tunnel selection/build/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP behavior;
- expose sockets, private/session keys, mutable router/storage/transport/tunnel/session handles, command channels, or message payloads to I2PControl;
- add new network probes, polling daemons, persistent time-series stores, news feed/downloader, or ban engine solely for observability;
- substitute fabricated zero/false/empty/null/adjacent metrics for unavailable state;
- modify AddressBook, proxy/UI, workflows, release/publishing, or unrelated code;
- broaden base I2PControl compatibility or method inventory;
- interact with upstream write channels.

## Handoff discipline

Every milestone must inspect its accepted dependency head, pin exact field semantics before source disposition changes, use focused source/wire tests, preserve no-feature behavior, remain inside its path budget, create an implementation disposition and independent closure record, and report any required scope expansion rather than silently performing it.

Only the registry advances the next handoff to `ready` after its hard dependency closes. M053 closure must explicitly dispose the M045 blocker before M046 can be registered.

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

M053 additionally requires the post-construction peer-directory churn regression described in its plan. Use targeted formatting because the repository's stable/nightly formatter qualification is already documented. Do not add CI/release/coverage/fuzz/soak infrastructure.

## Final status rule

M051 confirmed that both remaining fields require absent substantive owners and
remain unavailable. M052 is therefore ready to validate the incomplete but
truthful final matrix; it must not claim RouterInfo source completion.

This does not automatically close full Proposal 170: unrelated unsupported tunnel families and other accepted partial dimensions remain outside this roadmap. If M051 proves that news or banned-peer semantics require an absent substantive subsystem, retain the field unavailable rather than expanding scope.

## Historical evidence

M040–M044 remain closed retained evidence. M039 remains historical-invalidated. The M045 attempt at `5ae0477` and blocked closure at `bf9c2eeb` are retained as corrective evidence; M053 does not erase or rewrite them.
