# M049 Closure — RouterInfo Rolling Metrics and Queue Sources

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/049-routerinfo-rolling-metrics-and-queues.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline reviewed: `bc4512f`

Implementation commit:

- `cd9ee99` — `i2pcontrol: add rolling metrics and queue sources`

Closure date: 2026-08-10

Pinned contract: Proposal 170, `I2PControl Expansion`, Open, revision 2026-05-20.

## 1. Executive finding

M049 is closed internally against the pinned Proposal 170 revision. All four
planned RouterInfo additions have truthful production owners, exact wire
fixtures, bounded behavior, and request-time/live-source evidence:

- transit bandwidth uses a bounded I2PControl-local 15-second sampler over
  outbound transit bytes;
- recent tunnel success uses an ordered reference EWMA, separate from the
  cumulative success ratio;
- tunnel build queue depth comes from live pending-build listeners;
- TBM queue depth comes from the live transit build-message receiver queue.

The broader subsystem remains `partial Proposal 170 support`. The RouterInfo
matrix is now 35 available, 1 protocol-permitted neutral, and 7 unavailable.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Transit 15s has exact rolling units and window behavior | `TransitBandwidthSampler` in `emissary-cli/src/i2pcontrol/production.rs`; deterministic 0/7.5s/15s/30s fixture | pass | Computes bytes/sec over the last 15 seconds and returns zero before a full window |
| Recent success is not cumulative | Neutral ordered accumulator in `emissary-core/src/events.rs`; pool build-result calls in `tunnel/pool/mod.rs`; reference-vector core test and wire fixture | pass | Uses the reference smoothing constant/start value and rounds the recent percentage |
| Queue depth is from the canonical tunnel-build owner | `TunnelPool::publish_queue_depth()` observes pending inbound/outbound listener lengths; `TunnelInspection` aggregates live pool gauges | pass | Publication is best-effort and does not affect build decisions |
| TBM depth is from the canonical transit owner | `TransitTunnelManager` publishes the existing transit build-message receiver depth into `TunnelInspection` | pass | Reports the owner’s bounded queued work-item depth, including zero after drain |
| Reset, wrap, startup, and zero traffic are explicit | Sampler rollback/reset and zero-traffic tests; checked subtraction, `u128` rate math, and bounded sample retention | pass | No negative, wrapped, or overflow-derived values are emitted |
| No request waits for the rolling window | Sampler is invoked while servicing RouterInfo reads and returns zero when immature | pass | No background sampler, timer, or polling daemon was added |
| Exact presence semantics and serializers | `m049_wire_fixture_returns_rolling_metric_and_live_queues`; four contract rows in `rpc.rs` and source map | pass | Direct parameter presence remains independent of parameter value |
| No-feature behavior remains unchanged | Full CLI no-feature test suite and no-feature clippy/check | pass | The sampler and production sources are feature-gated with I2PControl |

## 3. Production implementation evidence

The implementation is split at the existing ownership boundary:

- `emissary-cli/src/i2pcontrol/production.rs` owns the bounded rolling
  sampler, monotonic startup-relative sampling clock, metric read, and recent
  success adapter.
- `emissary-core/src/events.rs` owns only the neutral ordered recent-build
  accumulator input and sanitized percentage observation.
- `emissary-core/src/inspection.rs` stores only bounded primitive queue gauges.
- `emissary-core/src/tunnel/pool/mod.rs` publishes pending-build depth and
  ordered build results at existing transitions.
- `emissary-core/src/tunnel/transit/mod.rs` publishes the existing transit
  build-message receiver depth.
- `emissary-cli/src/i2pcontrol/router_info_handler.rs` snapshots each source
  once per request and reuses existing serializers.

No router, transport, tunnel-selection, tunnel-build, routing, cryptographic,
I2NP, NetDB, or data-plane algorithm was changed. No persistent metric store,
new task, network probe, or service was introduced.

## 4. Verification executed

### Commands run

```bash
rtk cargo check -p emissary-core
rtk cargo check -p emissary-cli --no-default-features
rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo test -p emissary-core inspection --no-fail-fast
rtk cargo test -p emissary-core --no-fail-fast
rtk cargo test -p emissary-cli --no-default-features --no-fail-fast
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --test router_info_truthfulness --no-fail-fast
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
rtk cargo clippy -p emissary-core --all-targets -- -D warnings
rtk cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings
rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
rtk git diff --check
rtk cargo fmt --all -- --check
```

### Results

- Core check passed.
- Core inspection tests: 7 passed.
- Full core suite: 1,061 passed, 2 ignored.
- CLI no-feature suite: 56 passed.
- Focused feature conformance/truthfulness tests: 92 passed.
- Full CLI feature suite: 1,364 passed.
- Core and both CLI clippy invocations passed with `-D warnings`.
- `git diff --check` passed before commit.
- Formatting check remains blocked only by the repository’s documented
  stable/nightly rustfmt mismatch in pre-existing files:
  `emissary-util/src/reseeder.rs`, `runtime/smol.rs`, `runtime/tokio.rs`, and
  `storage.rs`. No unrelated formatter churn was retained.

The repository’s no-default core configuration remains a pre-existing
`RwLock` resolution limitation recorded by earlier closures; the supported CLI
no-feature path passed.

## 5. Invariant review

1. Rolling policy remains in I2PControl; core exposes no metric service.
2. The only core additions are neutral primitive observations at existing
   event, pool, inspection, and transit-owner seams.
3. Queue gauges are read-only and sourced from live owner transitions rather
   than task-count estimates.
4. Counter rollback clears the local window; checked arithmetic and `u128`
   intermediate math prevent negative or wrapped rates.
5. Immature windows return zero immediately; requests never sleep or wait for
   samples.
6. Recent success is event-ordered and distinct from cumulative totals.

## 6. Failure and recovery review

Sampler state is process-local, bounded to at most 16 samples, and naturally
restarts empty. A counter rollback clears prior samples. Duplicate timestamps
replace the prior sample. Queue source locks are held only for short primitive
updates/copies; no lock crosses an await or serialization. Pool destruction
removes its queue gauge, transit drain publishes zero, and observation failure
is ignored by the data plane. JSON assembly remains bounded by the existing
RouterInfo response limits.

## 7. Migration and compatibility review

No schema, configuration, persistence, or migration changes were introduced.
The four canonical selectors retain their existing exact names and direct
presence semantics. Existing aliases, authentication, TLS, and no-feature
behavior are unchanged. There is no rollback data migration required.

## 8. Security review

The additions remain behind the existing authenticated I2PControl path and
reuse sanitized read-only serializers. No keys, credentials, payloads,
mutable handles, channels, sockets, or router authority cross the inspection
boundary. Sample and queue state are bounded to avoid unbounded observability
memory growth. No new network or privilege surface was added.

## 9. Documentation and operations

Updated `docs/i2pcontrol/router-info-source-map.md` with the four owners,
fixtures, and bounds. Contract manifests, truthfulness fixtures, and static
source-count expectations now record 35 available / 1 neutral / 7 unavailable.
The full feature/no-feature suites exercise the canonical response path and
containment guards. There is no background process or recovery operation to
operate; restart resets only process-local rolling/observation state.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | Repository-wide formatting check is sensitive to the unavailable/different rustfmt qualification and fails on pre-existing `emissary-util` files | Formatting evidence cannot be globally green in this environment | Retain the documented baseline qualification; do not reformat unrelated files |

No medium, high, or critical implementation findings remain.

## 11. Roadmap disposition

M049 is closed and its hard dependency is satisfied for M050. M050 is now the
sole dependency-ready handoff. M051 remains blocked on M050, and M052 remains
blocked until M045–M051 have accepted dispositions. The broader Proposal 170
roadmap remains partial and internal-only.

## 12. Registry updates

The following records were updated together with this closure:

- M049 implementation plan status changed to `closed`;
- M050 implementation plan status changed to `ready`;
- `plans/registry.md` now lists M049 closed and M050 as the sole ready plan;
- `plans/implementation/i2pcontrol-proposal-170/README.md` now reflects the
  35/1/7 matrix and M050 handoff;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` now records M049
  closed, M050 ready, and the seven remaining unavailable rows.

The only external authority consulted was read in read-only mode for contract
and reference semantics. No upstream repository, issue, pull request, review,
discussion, submission, adoption request, merge, maintainer channel, or
contribution artifact was mutated or prepared. Commits were made only to the
authorized internal repository/fork.

Disposition: **closed**.
