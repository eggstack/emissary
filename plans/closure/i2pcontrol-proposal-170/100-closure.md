# M100 Closure — RouterInfo Transit 15-Second Source Completion

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/100-routerinfo-transit-15s-source-completion.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Implementation commit: `5421cad7d86b9d0975048e62b3b5d63473b69c1a` (`i2pcontrol: complete M100 transit bandwidth source`).
Closure commit: the follow-on internal commit containing this record.
Review date: 2026-08-28.

## 1. Disposition

M100 closes as implemented. `i2p.router.net.bw.transit.15s` is now backed by a
request-independent, I2PControl-owned sampler over the existing authoritative cumulative
transit-byte source. No router-core, dependency, lockfile, workflow, or frontend path was
changed.

The authoritative matrix is
`plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`; its current
RouterInfo disposition is 43 rows: 38 available, 1 protocol-permitted neutral, and 4
unavailable.

## 2. Requirement/evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| M095/reference semantics | `TransitWindow` samples every 1 second, retains at most 17 points, requires a complete 15-second monotonic window, and divides by actual elapsed milliseconds with integer floor rounding. | pass |
| Request independence | `TransitBandwidthSampler` owns one background Tokio task; `production_adapter` and sampler tests advance the clock and mutate metrics without making RouterInfo reads. | pass |
| Authoritative source | `EventHandleMetrics::transit_bytes_snapshot` reads the existing cumulative outbound transit counter. No second counter or traffic-path accounting was added. | pass |
| Warmup and real zero | A valid source with less than 15 seconds returns the explicit warmup reason; equal cumulative endpoints return `0` after a complete window. | pass |
| Reset/wrap and failure | Counter decreases start a new generation; `None` clears history and reports source unavailable. | pass |
| Staleness | A sampled value older than the bounded three-second freshness limit reports an explicit stale reason. | pass |
| Cancellation/resource bounds | One sampler handle owns one cancellable task; history is bounded to 17 samples, reads/writes hold the lock only for the snapshot, and `Drop` aborts the task. | pass |
| Feature boundary | Sampler composition is in the optional I2PControl production adapter; the default/no-feature test suite passes. | pass |
| Regression containment | Static guards reject handler-local sampling and require the sampler cadence/missed-tick policy; M061/M062 containment suites pass. | pass |

The RouterInfo wire key, return type, and serializer remain unchanged. Only its source
disposition changed from unavailable to available after the request-independent owner was
added.

## 3. Exact changed paths

The implementation commit changed exactly these paths:

- `AGENTS.md`
- `docs/i2pcontrol/README.md`
- `docs/i2pcontrol/proposal-170-conformance.md`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/router-info-source-map.md`
- `docs/i2pcontrol/router-info.md`
- `emissary-cli/src/i2pcontrol/mod.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/rpc.rs`
- `emissary-cli/src/i2pcontrol/server.rs`
- `emissary-cli/src/i2pcontrol/transit_sampler.rs`
- `emissary-cli/tests/conformance_manifest.rs`
- `emissary-cli/tests/m027_literal_fixtures.rs`
- `emissary-cli/tests/m062_dependency_containment.rs`
- `emissary-cli/tests/m095_full_support_matrix.rs`
- `emissary-cli/tests/production_adapter.rs`
- `emissary-cli/tests/router_info_truthfulness.rs`
- `emissary-cli/tests/static_guards.rs`
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
- `plans/implementation/i2pcontrol-proposal-170/100-routerinfo-transit-15s-source-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

The closure commit adds only this closure record and updates the matrix production-head
field to the implementation commit.

## 4. Verification outcomes

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass; 1,724 tests across 25 suites |
| `cargo test -p emissary-cli --no-default-features` | pass; 56 tests across 25 suites |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| focused sampler, production adapter, RouterInfo truthfulness, matrix, manifest, static-guard, composition, and containment tests | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment` | pass; 7 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | pass; 19 tests |
| `git diff --check` | pass |

The requested `m063_feature_reachability` command was attempted but this checkout has no
such test target; Cargo listed the available targets and exited before compiling tests.
Feature-disabled coverage still passes, and the M062 dependency allowlist covers the
feature reachability boundary.

`cargo fmt --all -- --check` remains qualified: it reports the repository's pre-existing
nightly-only formatting drift in untouched files. Changed Rust files were formatted with
the available stable formatter, and no unrelated formatting changes were retained.

## 5. Compatibility, security, and operational notes

- No persisted schema or migration is needed; the recent-rate history is intentionally
  in-memory and resets on service/router restart.
- The source is read-only from the router traffic path and cannot turn a missing, reset, or
  stale observation into a fabricated current zero.
- The sampler is created once by `ProductionRouterInfoControl`; no handler request can
  create a competing sampler or advance its history.
- The existing partial Proposal 170 boundary remains explicit: M101-M103 are now
  dependency-ready, while M098/M099 remain blocked on M097 and M104 remains blocked on
  M097-M103. M100 does not unblock those M097-dependent plans.
- No high- or medium-severity findings were introduced. The absent M063 test target and
  repository-wide formatter drift are low-severity verification/tooling limitations, not
  M100 implementation blockers.

## 6. Future-plan disposition

The registry and roadmap now mark M100 closed. M101 (router news), M102 (network error),
and M103 (banned peers) remain ready because M100 does not change their dependencies.
M098 and M099 remain blocked on M097; M104 remains blocked on M097-M103. No future plan
was incorrectly advanced by this milestone.

## 7. Internal-only attestation

All repository writes, commits, and the push are limited to the internal
`eggstack/emissary` repository. The official Proposal 170 and i2pd reference material
were read-only evidence only. No upstream issue, pull request, review, submission, merge,
contribution artifact, maintainer contact, or external repository write was created.

**Disposition: closed.**
