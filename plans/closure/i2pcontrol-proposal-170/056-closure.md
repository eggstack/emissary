# M056 Closure Record — M049/M050 Corrective Integration Reclosure

Status: closed

Reviewed plan: `plans/implementation/i2pcontrol-proposal-170/056-m049-m050-corrective-reclosure.md`

Integrated reviewed head: `70c1539` (`plans: record M055 implementation commit`)

Closure date: 2026-08-11

## 1. Disposition

M056 is formally closed as an independent, production-free corrective
integration reclosure. The accepted M054 and M055 dispositions are consistent
with the integrated head, the Proposal 170 contract manifest, focused fixtures,
live authenticated response paths, and the retained broad RouterInfo suite.

The final canonical RouterInfo source matrix is:

- 43 additions total;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable: `i2p.router.news`,
  `i2p.router.netdb.bannedpeers`,
  `i2p.router.net.bw.transit.15s`,
  `i2p.router.net.error`, and
  `i2p.router.net.error.v6`.

RouterInfo source completion remains partial. M056 supersedes only the
invalidated transit-15s and network-error claims in the historical M049/M050
records and the historical M052 `40 available / 1 neutral / 2 unavailable`
integrated count. It does not reopen the retained M049 recent-success, queue,
or TBM fields; M050 status/testing; M051 news/ban adjudication; or M045–M048
accepted source work.

## 2. Accepted corrective heads and containment

| Milestone | Implementation head | Accepted closure head | Disposition |
|---|---|---|---|
| M054 | `eed0368` — `i2pcontrol: correct M054 transit bandwidth semantics` | `311a39e` — M054 closure record | transit-15s explicitly unavailable; request-local sampler removed |
| M055 | `b35d231` — `i2pcontrol: close M055 network error corrective` | `70c1539` records the exact implementation head in the accepted closure | v4/v6 network-error explicitly unavailable; dead error-only core scaffolding removed |

M054 touched only its authorized I2PControl composition/test/docs paths plus
`emissary-core/src/events.rs`; it did not touch tunnel, transport, router,
NetDB, or data-plane paths. M055's core cleanup was limited to
`emissary-core/src/events.rs` and `emissary-core/src/inspection.rs`, with
I2PControl contract/handler/tests/docs changes; it did not alter retained
status/testing transport behavior or add an error detector. M056 itself made
no production changes. Its changed paths are planning, closure, and the
reconciled I2PControl documentation only.

The machine-readable boundary remains
`plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`.
No forbidden core prefix, workflow, release, remote-CI, frontend, tunnel data
plane, or upstream path was added by this reclosure.

## 3. Historical defect reproductions

The invalidated behavior remains inspectable in the pre-correction head
`970252c`:

1. `emissary-cli/src/i2pcontrol/production.rs` contained
   `TransitBandwidthSampler`, and `transit_bandwidth_15s()` sampled cumulative
   transit bytes only when the RouterInfo getter was called. The historical
   unit fixture `transit_sampler_is_zero_until_a_full_window_then_uses_bytes_per_second`
   demonstrates that the first read is zero until request-created history
   exists and that a later request is the event which advances the window.
   A long query gap therefore made the result depend on API request history,
   not router traffic history. M054 removed this request-local authority and
   makes the canonical row deterministically unavailable.
2. The same historical head contained `network_error_code(None) => 0` while
   the production-writer audit found no runtime writer for a canonical
   `NetworkErrorReason`. The old adapter therefore serialized the positive
   reference meaning `No error` from source absence. M055 removed the mapper,
   unowned error fields/setters, and both requestable error rows.

The current regressions preserve the lessons of both failures: a fake transit
value cannot make the unavailable row requestable, and direct or combined
network-error requests fail before a result can contain code `0`.

## 4. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| M054/M055 accepted before independent reclosure | `054-closure.md`, `055-closure.md`, exact heads above, and current integrated head | pass |
| Transit-15s has truthful final disposition | `PROPOSAL_170_CONTRACT` marks it `Unavailable` with owner `transit-bandwidth`; `router_info_truthfulness` covers direct and version-plus-transit requests; live runtime covers the authenticated selector path | pass |
| Transit request history cannot be authoritative | no `TransitBandwidthSampler`, mutex, or request-local rolling state remains in `ProductionRouterInfoControl`; M054 historical source and regression evidence are retained in Git history | pass |
| Both network-error selectors fail closed | contract marks both `Unavailable` with owner `network-error`; direct v4/v6 and combined requests are covered by the retained truthfulness/live suites and return `-32603` with null results | pass |
| Missing error authority cannot serialize `0` | `network_error_code`, `NetworkErrorReason`, error atomics, setters, and error-only DTO scaffolding are absent; live assertions check the sanitized unavailable reason | pass |
| M050 retained fields remain operational | `status.v6`, `testing`, and `testing.v6` remain in the available selector fixture and broad RouterInfo/live tests; no retained transport owner changed | pass |
| Every available row has source accounting | `PROPOSAL_170_CONTRACT` has 43 unique rows; every row has a non-empty owner, serializer, fixture, and disposition; conformance and source-map tests assert 43 / 37 / 1 / 5 and one source-map row per canonical key | pass |
| Direct presence and no-partial-result semantics remain exact | literal manifest, truthfulness suite, compatibility/inventory checks, and full feature suite | pass |
| M045–M053 and M046–M048 retained behavior remains green | bounded `router_info` suite, full feature suite, conformance/literal fixtures, production composition, and live child-process path | pass |
| No production implementation was added under M056 | final M056 changes are limited to `plans/**` and reconciled `docs/i2pcontrol/**`; no M056 production path exists | pass |

## 5. Final 43-row source audit

The audit source is `rpc.rs::router_info_keys::PROPOSAL_170_CONTRACT`, not a
hand-maintained closure count. The exact canonical partition at the reviewed
head is:

| Partition | Count | Evidence |
|---|---:|---|
| Available | 37 | contract source dispositions and named owners; conformance manifest and literal fixture |
| Protocol-permitted neutral | 1 | clock-skew `null` disposition and exact neutral fixture |
| Unavailable | 5 | news, banned peers, transit-15s, v4 network-error, v6 network-error; deterministic preflight/error fixtures |
| Total | 43 | exact addition and contract arrays, uniqueness assertions |

No row is counted available merely because a serializer exists. The five
unavailable rows have explicit owners/reasons in the contract and fail before
source acquisition or partial assembly. The docs/source map, conformance
manifest, literal fixture, RouterInfo behavior document, support document, and
implementation planning records all report the same 37/1/5 partition.

## 6. Retained verification

Successful commands at the integrated head:

- `cargo check -p emissary-core`.
- `cargo test -p emissary-core --no-fail-fast` — 1062 passed, 2 ignored.
- `cargo clippy -p emissary-core --all-targets -- -D warnings`.
- `cargo check -p emissary-cli --no-default-features`.
- `cargo test -p emissary-cli --no-default-features` — 56 passed.
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol` — 1372 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast` — 135 passed, 1237 filtered out.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --no-fail-fast` — 58 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures --no-fail-fast` — 7 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness --no-fail-fast` — 36 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast` — 9 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` — 1 passed.
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings`.
- `git diff --check`.

`cargo fmt --all -- --check` and `cargo +nightly fmt --all -- --check` remain
qualified failures caused by extensive pre-existing formatting differences in
untouched workspace files and the active stable/nightly rustfmt configuration.
Neither command modified the worktree, and no unrelated formatting churn was
absorbed. This is a repository-baseline qualification, not a M056 source or
closure defect.

## 7. Failure, restart, cancellation, and contention review

- Transit-15s has no production rolling owner after M054. Requests, request
  cancellation, request inactivity, and concurrent callers cannot reset,
  advance, or contend on transit history. Restart semantics are intentionally
  bounded to explicit unavailability until a separately authorized
  request-independent owner exists.
- Network-error requests are rejected during canonical source-disposition
  preflight, before network snapshot acquisition, serialization, waiting, or
  partial result assembly. They perform no source mutation and add no lock,
  task, timer, or await path.
- M050's status/testing observations remain passive and operational. The live
  child process verifies the final selector response path without adding a
  traffic generator or network harness.
- No lock spans await, serialization, network I/O, or external observation;
  observation failure cannot perturb router data-plane behavior.

## 8. Compatibility and security review

The JSON-RPC method, selector spelling, declared integer types, direct-presence
behavior, compatibility boundary, authentication, TLS composition, bounds,
and sanitized error contract remain unchanged. No schema, configuration,
persistence, migration, AddressBook, tunnel-manager, or public API expansion
was introduced by M056.

The core observation boundary exposes no keys, sockets, channels, mutable
sessions/tunnels/router owners, or Proposal 170 wire terminology. The corrective
heads reduce or preserve the observation surface and do not add probes,
credentials, private material, network behavior, or router lifecycle control.

## 9. Residual limitations and severity

| Severity | Limitation | Disposition |
|---|---|---|
| Medium, accepted | Five canonical RouterInfo rows remain unavailable because their authoritative owners do not exist: news, bans, transit-15s, and v4/v6 network-error | Explicit in contract/docs and fail-closed; M051 remains blocked for its separate news/ban owner question; no unaccepted truthfulness finding remains |
| Low, accepted | Repository-wide rustfmt check reports pre-existing unrelated churn under the configured toolchains | Recorded qualification; no unrelated formatting changes retained |

No high- or critical-severity truthfulness, security, containment, or
compatibility finding remains within M056. Overall Proposal 170 remains partial
for unrelated previously accepted unsupported dimensions, including deferred
tunnel data planes.

## 10. Planning disposition and supersession

M049 is corrected/closed through M054 and M056: its recent-success, queue, and
TBM fields remain accepted, while only transit-15s is superseded to explicit
unavailability. M050 is corrected/closed through M055 and M056: status.v6 and
testing v4/v6 remain accepted, while only the two network-error claims are
superseded to explicit unavailability. M052's historical `40/1/2` final count
is superseded by this independent `37/1/5` audit. The original closure records
remain historical evidence and were not rewritten.

M056 itself is closed. No future plan became dependency-ready: M051 remains
blocked with its accepted news/ban semantic limitation because M056 created no
substantive owners, and no owner-specific successor plan is currently
authorized or registered. RouterInfo source completion and overall Proposal 170
support therefore remain partial.

## 11. Internal-only attestation

Proposal 170 and read-only reference material were used only as internal
correctness evidence. No upstream repository or maintainer channel was
mutated; no upstream issue, review, merge, adoption request, submission, or
contribution artifact was created or prepared. All repository writes remain
within the authorized internal `eggstack/emissary` repository.

**Disposition: M056 closed; M049/M050/M052 corrected only for the named
invalidated claims; final RouterInfo source matrix 37/1/5; no future plan
newly unblocked.**
