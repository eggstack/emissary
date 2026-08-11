# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; containment corrective sequence active with M060 ready

This directory contains bounded internal implementation/closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` — accepted source/truthfulness work through M057;
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` — current containment corrective workstream;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml` — historical/current RouterInfo source-budget authority through M057.

Pinned external authority: Proposal 170 `I2PControl Expansion`, revision `2026-05-20` as accepted by the existing workstream, plus read-only reference implementation evidence where the proposal adopts or leaves semantics terse.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications/reference implementations/upstream source are read-only. No plan authorizes an upstream issue, pull request, review, discussion, submission, adoption request, merge request, maintainer outreach, contribution package, branch/tag/release push, or connector write against an upstream/third-party repository.

## Current handoff

M060 is the sole dependency-ready handoff:

- `058-non-i2pcontrol-delta-inventory.md` — **closed**; audit-only complete upstream/fork non-`i2pcontrol` production delta and create a machine-readable containment ledger. Production changes were forbidden.
- `059-cli-runtime-containment.md` — **closed**; original CLI/runtime containment accepted with no core changes.
- `060-core-observation-containment.md` — **ready**; reduce the audited core observation delta using the accepted M058 core budget.

The remaining containment handoff is planned but is not registered ready until its predecessor closes:

- `061-containment-reclosure.md` — planned; hard-blocked on M060 closure.

Per planning governance, only M060 is executable now.

## Accepted Proposal 170 production disposition

The prior source/truthfulness sequence remains accepted:

- M054 — closed; transit-15s explicitly unavailable because no request-independent owner fits the bounded budget;
- M055 — closed; both unowned v4/v6 network-error rows unavailable and dead error scaffolding removed;
- M056 — closed; final integrated RouterInfo matrix accepted as 37 available / 1 protocol-permitted neutral / 5 unavailable;
- M057 — closed; planning-record consistency only, no production change.

The five unavailable RouterInfo additions remain:

- `i2p.router.net.bw.transit.15s`;
- `i2p.router.news`;
- `i2p.router.netdb.bannedpeers`;
- `i2p.router.net.error`;
- `i2p.router.net.error.v6`.

The containment sequence does not attempt to make these available. The historical pre-corrective M052 `40/1/2` state remains superseded by the accepted M056 `37/1/5` state.

## Why containment work is reopened after M057

M037 previously reduced Proposal 170 policy leakage and established an early static boundary. Later M045–M055 RouterInfo source work added live neutral observations after M037, including peer, transport, tunnel, queue, and testing facts. As a result, most business logic remains correctly under `emissary-cli/src/i2pcontrol/**`, but the physical fork delta still spans a broad set of original CLI/runtime and audited `emissary-core` files.

The new corrective objective is not source completeness and not a zero-diff target. It is:

> every production change outside `emissary-cli/src/i2pcontrol/**` must be individually justified as minimum feature/composition or neutral canonical-owner observation, otherwise it must be removed, reverted, or consolidated.

This is primarily a security-review surface reduction.

## Containment sequence

| Handoff | Status | Objective | Hard dependency |
|---|---|---|---|
| M058 — non-I2PControl delta inventory | closed | 47-path ledger accepted; exact M059/M060 budgets frozen; no production changes | `058-closure.md` |
| M059 — original CLI/runtime containment | closed | original CLI/runtime policy contained; exact M059 closure accepted with no core changes | `059-closure.md` |
| M060 — core observation containment | ready | reduce audited-core inspection/SAM/I2CP/transport/tunnel delta to minimum neutral owner seams | M059 closure |
| M061 — independent containment reclosure | planned | production-free recompute/review; create current exact-path manifest and static guard | M060 closure |

Plans:

- `058-non-i2pcontrol-delta-inventory.md`;
- `059-cli-runtime-containment.md`;
- `060-core-observation-containment.md`;
- `061-containment-reclosure.md`.

## M058 boundary

M058 changes no production code. It creates `058-containment-ledger.toml` during implementation and classifies each changed non-`i2pcontrol` production path as one of:

- required composition;
- required canonical-owner seam;
- candidate revert;
- candidate consolidation;
- unrelated/accidental;
- uncertain.

An `uncertain` path cannot be modified by M059/M060 until resolved. The ledger must name owner, consumer, policy-leak status, rationale, next milestone, and required regressions.

The audit is pinned to fork baseline `adb2f52543764b267b2bcb282d093111001ae4b2` and upstream merge-base comparison `eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f` unless implementation records a newer fork planning-only head. Upstream advancement is recorded separately rather than silently moving the compare baseline.

## M059 ownership rule

M059 may touch only original CLI/runtime paths accepted into its M058 path budget plus `emissary-cli/src/i2pcontrol/**` and focused tests/docs.

Target state:

- `address_book.rs` owns ordinary runtime behavior plus the smallest neutral administrative overlay seam; administrative persistence/validation/subscription policy lives under `i2pcontrol`;
- `config.rs` owns only feature/config enable/bind values, not method/source policy;
- `main.rs` owns composition/start-stop wiring, not control-plane semantics;
- logger/proxy/tunnel original modules expose only passive sanitized owner-local lifecycle facts/capabilities;
- no `emissary-core/**` change is permitted under M059.

## M060 ownership rule

M060 applies the accepted M058 core budget after M059 closes.

Core changes are evaluated in this order:

1. revert unnecessary fork change to upstream;
2. derive the same authoritative fact at an already-retained higher owner;
3. consolidate several low-level hooks into one neutral owner-level seam;
4. retain a deep hook only when moving it upward would lose truth, ordering, or required bounds.

Deep SAM/NTCP2/SSU2/tunnel path changes are high-sensitivity. Current use is not sufficient justification. Core may not contain Proposal 170 selectors, wire semantics, administrative persistence, support classification, or JSON-RPC types.

No new core path, event framework, sampler, probe, persistent metrics service, or unsupported source/data plane is authorized.

## M061 final authority

M061 changes no production code. It independently recomputes the final production diff, verifies every retained path, and creates:

- `061-containment-boundary.toml` with exact allowed paths;
- a focused current static guard such as `emissary-cli/tests/m061_containment.rs`;
- `061-closure.md`.

The M061 manifest becomes the current containment authority while M037 remains historical evidence of the earlier boundary. High-sensitivity core allowances must be exact paths, not broad transport/tunnel directory prefixes.

## Historical RouterInfo source-completion sequence

| Handoff | Status | Target/disposition |
|---|---|---|
| M053 / corrected M045 | closed | live known-peer directory |
| M046 | closed | active-peer inventory + finite limits |
| M047 | closed | active-peer statistics |
| M048 | closed | tunnel-pool counts/details |
| M049 | corrected/closed through M054/M056 | recent success + queue/TBM retained; transit 15s unavailable |
| M050 | corrected/closed through M055/M056 | status.v6 + testing v4/v6 retained; error v4/v6 unavailable |
| M051 | blocked with accepted limitation | news and banned peers remain unavailable |
| M052 | corrected/closed through M056 | historical `40/1/2` count superseded by `37/1/5` |
| M054 | closed | transit-15s truthfulness corrective |
| M055 | closed | network-error truthfulness corrective |
| M056 | closed | final 43-row audit accepted at `37/1/5` |
| M057 | closed | planning-record consistency; no production change |

## Cross-cutting prohibitions

Throughout M058–M061, do not:

- add a missing tunnel data plane;
- implement the five currently unavailable RouterInfo sources;
- modify router/peer selection, NetDB protocol/discovery, tunnel selection/build/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP behavior;
- expose sockets, private/session keys, mutable router/storage/transport/tunnel/session handles, command channels, or message payloads to I2PControl;
- add network probes, polling daemons, persistent time-series stores, background samplers, or a generalized event bus solely for I2PControl;
- substitute fabricated zero/false/empty/null/adjacent values for unavailable state;
- broaden base I2PControl compatibility or method inventory;
- perform a repository-wide crate extraction/refactor merely to reduce file count;
- add/expand CI, release, coverage, fuzz, soak, or platform verification infrastructure;
- interact with upstream write/review/submission channels.

## Verification discipline

M058 is audit-only and uses compare/path/search evidence; broad Rust verification is unnecessary.

M059 uses focused `emissary-cli` no-feature/feature checks plus AddressBook, ClientServicesInfo, tunnel lifecycle, production composition, and containment regressions. Core must not change.

M060 adds focused core SAM/I2CP/transport/tunnel/inspection regressions and the control-plane tests consuming those observations.

M061 runs a bounded integrated reclosure plus current static guard. No hosted CI expansion is required.

## Final status rule

Containment completion does not mean full Proposal 170 support. The accepted API remains partial under current Emissary capabilities unless separately authorized source/runtime work closes unavailable dimensions.

The final target of this workstream is minimum justified non-`i2pcontrol` delta, not a predetermined changed-file count and not upstream submission.
