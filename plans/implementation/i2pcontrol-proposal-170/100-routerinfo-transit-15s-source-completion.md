# M100 — RouterInfo Transit 15-Second Source Completion

Status: blocked on M095

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- ADR-0004 full-support completion boundary;
- M049 historical rolling-metric implementation;
- M054 transit-15s truthfulness corrective;
- M056 current 37/1/5 RouterInfo reclosure authority.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` plus M095 closure when dependency-ready.

Pinned external contract: `i2p.router.net.bw.transit.15s`, Proposal 170 revision `2026-05-20`, return type `long`, adopted from i2pd.

Classification: capability / infrastructure.

## 1. Objective

Make `i2p.router.net.bw.transit.15s` truthfully available using a request-independent, feature-gated I2PControl-owned rolling sampler over Emissary's already-authoritative cumulative transit-byte counter.

M054 correctly removed the prior request-local sampler because API request history determined the result. ADR-0004 now explicitly authorizes a bounded background sampler within the optional I2PControl service so the metric exists independently of RouterInfo request cadence without changing router-core traffic instrumentation.

## 2. Required semantic

The returned value is the recent 15-second average transit bandwidth in bytes per second, using the exact rounding/integer behavior established by M095/reference evidence.

The metric must represent router transit traffic over elapsed monotonic time, not:

- total transport traffic;
- client/application traffic;
- request count;
- time since the previous RouterInfo call;
- a fixed zero before any request history exists.

## 3. Source architecture

```text
existing authoritative cumulative transit bytes
               |
               v
      bounded read-only snapshot
               |
               v
I2PControl-owned periodic sampler task
               |
          bounded ring/history
               |
               v
15-second delta / actual elapsed time
               |
               v
RouterInfo serializer
```

No new timer/counter is added to the router traffic path. The sampler runs only when I2PControl production composition is enabled.

## 4. Sampling design

M095 must freeze the exact cadence and window behavior. Preferred design:

- sample cumulative transit bytes at a fixed cadence substantially smaller than 15 seconds (for example 1 second) using `tokio::time::Instant`/monotonic time;
- retain only the samples needed to cover slightly more than 15 seconds plus one prior point;
- compute the oldest/newest usable points spanning the target window;
- divide byte delta by actual elapsed duration, not assumed tick count;
- handle counter wrap/reset/restart explicitly;
- bound history to a small compile-time/configured maximum;
- no allocation proportional to traffic volume.

Do not sample from the handler itself.

## 5. Startup/warmup semantics

The metric needs a truthful defined behavior before a complete 15-second history exists.

M095/reference evidence must choose the exact behavior. Acceptable forms include calculating over the actual available elapsed interval after a minimum sample pair if that matches i2pd semantics, or returning the protocol's defined neutral/unavailable behavior until the full window exists.

Do not return `0` merely because the sampler is warming up unless actual measured transit bandwidth is zero or the pinned semantics explicitly define zero during warmup.

## 6. Failure/restart semantics

- sampler observation failure must not mutate router state;
- a stale last value must not be presented indefinitely as current without an explicit staleness bound;
- router/service restart resets the in-memory 15-second history; no persistence is required for a recent-rate metric;
- cumulative counter reset/wrap starts a new history generation rather than producing a huge negative/positive rate;
- stopping I2PControl cancels the sampler task cleanly;
- repeated service construction must not create duplicate samplers for one I2PControl state.

## 7. Preferred authorized path boundary

Target changes under `emissary-cli/src/i2pcontrol/**`, likely:

- `observability.rs` or a dedicated I2PControl-local recent-metric module;
- `observers.rs` only if that is the accepted typed observation composition point;
- `production.rs`/`server.rs` only to own/start/cancel one sampler handle;
- `router_info.rs` / `router_info_handler.rs` only for source disposition/serialization;
- focused tests/docs/M095 matrix updates.

Existing `emissary-core/src/events.rs` cumulative transit counter is read-only input and is not authorized for modification under M100.

No new `emissary-core/**`, dependency, manifest, lockfile, workflow, or frontend path is authorized.

## 8. Invariants

1. Request frequency cannot affect the metric.
2. Only authoritative transit bytes feed the metric.
3. Monotonic elapsed time is used.
4. History/task count is bounded.
5. No router-core task/timer/traffic instrumentation is added.
6. Feature-disabled/default builds create no sampler.
7. No fabricated zero/stale value on missing source or reset.
8. No new public field/type/alias.
9. M049's other accepted metrics remain unchanged.
10. No upstream interaction occurs.

## 9. Explicit non-goals

M100 MUST NOT:

- modify transit forwarding accounting;
- add persistent metrics storage;
- add general metrics/exporter infrastructure;
- change tunnel success/queue metrics;
- implement news/network-error/banned peers;
- change router algorithms/timing;
- add hosted monitoring/CI;
- interact upstream.

## 10. Ordered work packages

A. Freeze exact M095/reference metric semantics, cadence, warmup, rounding, and reset behavior.

B. Add the bounded I2PControl sampler state/handle and one owner task.

C. Connect it to the existing cumulative transit observation without adding a second traffic counter.

D. Replace RouterInfo unavailability for this row only after a valid sampled value is available according to the pinned semantics.

E. Add deterministic clock/counter tests; avoid wall-clock sleeps where a paused Tokio clock or injectable sample function can prove behavior.

F. Update M095 matrix/support docs after evidence.

## 11. Contention/resource semantics

- sampler writes use a small lock/atomic state and never hold it across await/sleep;
- RouterInfo reads are bounded snapshots;
- a slow/blocked RouterInfo request cannot delay sampling materially;
- one sampler task per composed I2PControl service;
- history size independent of number of API clients/requests.

## 12. Compatibility/migration

No persisted schema migration. Public return type remains Proposal 170 `long`.

The current explicit-unavailable behavior remains until the sampler is valid; older clients are unaffected except that the requested field becomes available.

## 13. Tests

At minimum:

- no RouterInfo calls for >15s, then correct measured result;
- many RouterInfo calls do not alter result;
- zero real transit produces zero after valid history;
- counter increase yields correct bytes/sec;
- irregular scheduling uses actual elapsed time;
- counter reset/wrap starts new generation safely;
- warmup exact behavior;
- staleness/source failure behavior;
- sampler cancellation/drop;
- duplicate construction guard;
- feature-off no task/source access;
- M054 regression fixture proving handler-local sampling cannot reappear.

## 14. Verification

Run focused RouterInfo/sampler tests plus:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
git diff --check
```

No core test suite is required because M100 modifies no core path.

## 15. Documentation/static guards

Update M095 RouterInfo row from unavailable/planned to available only after request-independent tests pass. Update support/conformance docs while overall support remains partial.

Retain a regression guard/comment/test that rejects any future implementation whose sample state advances only in the getter path.

## 16. Acceptance and stop conditions

M100 closes only if:

- the metric is request-independent and matches pinned semantics;
- no core path changed;
- history/task state is bounded/cancellable;
- warmup/reset/stale behavior is truthful;
- M054's defect cannot regress;
- no upstream interaction occurred.

Stop if the existing cumulative transit counter cannot be read at the required cadence without changing router behavior or if the pinned semantics require a data-plane sampling mechanism unavailable at the I2PControl boundary.

## 17. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/100-closure.md` containing:

- M095 semantic dependency;
- exact changed paths;
- sampler ownership/cadence/window evidence;
- request-independence regression evidence;
- warmup/reset/failure/cancellation evidence;
- feature-off/containment results;
- updated RouterInfo matrix totals;
- unresolved findings;
- internal-only/no-upstream attestation.

## 18. Internal-only rule

All writes remain internal to `eggstack/emissary`; external i2pd/I2P sources are read-only evidence. No upstream write/review/submission/merge/contribution activity is authorized.