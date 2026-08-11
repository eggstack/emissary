# M054 — M049 Corrective Request-Independent Transit 15s Source

Status: ready

Planning baseline: `970252c` — merged M053–M052 implementation/reclosure head

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrects:

- `plans/implementation/i2pcontrol-proposal-170/049-routerinfo-rolling-metrics-and-queues.md`;
- `plans/closure/i2pcontrol-proposal-170/049-closure.md`;
- the affected `i2p.router.net.bw.transit.15s` finding in `plans/closure/i2pcontrol-proposal-170/052-closure.md`.

Milestone class: corrective capability + containment invariant

Hard dependencies:

- M048 remains closed;
- post-M052 review at `970252c` accepted the transit-15s semantic defect described below.

Pinned authority: I2P Proposal 170 `I2PControl Expansion`, Open, revision `2026-05-20`, plus the read-only i2pd implementation adopted by the proposal for this field.

## 1. Objective

Correct exactly one canonical Proposal 170 field:

- `i2p.router.net.bw.transit.15s`.

The field must report a truthful router-owned recent 15-second transit-bandwidth value in bytes per second that does not depend on when or how often I2PControl clients query RouterInfo.

The current request-driven `TransitBandwidthSampler` in `emissary-cli/src/i2pcontrol/production.rs` is not an acceptable source. Replace it with the smallest request-independent neutral observation available from an existing canonical owner, or restore the field to explicit `Unavailable` if exact semantics cannot be provided within this corrective budget without adding a new polling subsystem or materially altering the data plane.

Do not reopen M049's recent tunnel-success, tunnel queue, or TBM queue fields; those remain closed unless a separate defect is demonstrated.

## 2. Defect and why prior verification missed it

M049 currently records cumulative transit-byte samples only inside `ProductionRouterInfoControl::transit_bandwidth_15s()`, meaning sampling occurs only when a caller asks for the field. The sampler also removes samples older than 15 seconds before selecting the oldest remaining point and returns zero until the retained span reaches a full 15 seconds.

Consequences:

- the first RouterInfo request can return zero after real transit traffic because no prior request-created sample exists;
- a request after a gap longer than the retained window can again return zero despite recent transit traffic;
- the returned value is therefore a function of I2PControl request history rather than solely of router traffic history.

M049 tests drove the sampler directly at synthetic timestamps, so they proved arithmetic on a pre-populated request-local history but did not prove that traffic occurring without RouterInfo requests remains observable. M052's live child-process request proved the field was callable, not that the rolling source was independent of prior API reads.

The required regression for this corrective pass must generate/record transit traffic without invoking the RouterInfo field, then query the same production source later and observe the expected current rolling value.

## 3. Current owner evidence and preferred boundary

Current core already owns the relevant cumulative fact:

- `EventHandle` receives cumulative outbound transit bytes from the transit-tunnel data path;
- `EventManager` already runs as part of `Router`, owns an existing timer, and periodically reads the cumulative transit counters for status/metrics publication;
- `refresh_interval` is configurable, so an implementation must not assume that the existing status timer always fires once per second;
- `Runtime::now()`/`Runtime::Instant::elapsed()` are available for monotonic elapsed-time calculations.

The preferred correction is therefore a neutral request-independent rolling observation associated with the existing event/metric owner, exposed to I2PControl as a scalar read-only metric. If the existing event cadence can support exact reference semantics using actual elapsed time, reuse that owner rather than adding another task. If it cannot, perform the bounded readiness audit in WP1 and stop/demote rather than hiding approximation behind the canonical selector.

The I2PControl layer continues to own the Proposal 170 field name, JSON type, availability policy, and sanitized failure mapping. Core must not contain Proposal 170 terminology.

## 4. Authorized production path budget

Primary corrective paths:

- `emissary-cli/src/i2pcontrol/production.rs`;
- other `emissary-cli/src/i2pcontrol/**` files only for trait/contract/handler/tests/source disposition;
- `emissary-core/src/events.rs` only if a neutral request-independent rolling source is required and can be implemented at the existing event owner.

Composition-only `emissary-cli/src/main.rs` is authorized only if the replacement source requires passing an already-existing read-only handle; no new service/task composition is authorized.

Not authorized by M054:

- `emissary-core/src/tunnel/**`;
- `emissary-core/src/transport/**`;
- `emissary-core/src/router/**`;
- `emissary-core/src/netdb/**`;
- cryptographic, I2NP, LeaseSet, AddressBook, proxy/UI, workflow/release, or unrelated paths.

If exact request-independent semantics require touching a canonical data-plane path outside `events.rs`, adding a dedicated timer/task/poller solely for I2PControl, or changing transit accounting semantics, stop and record the blocker rather than broadening this plan.

## 5. Invariants

1. The returned value is independent of RouterInfo request cadence.
2. No RouterInfo request sleeps or waits for a 15-second window to mature.
3. The source uses actual elapsed time; it must not silently assume a configured event refresh interval is exactly one second.
4. The source is bounded in memory and process-local; restart/reset behavior is explicit.
5. Counter rollback/reset cannot emit wrapped, negative, or fabricated rates.
6. Zero is emitted only when it is a truthful authoritative rolling result or the pinned reference explicitly defines the immature-startup state as zero. Missing source capability is `Unavailable`, not zero.
7. No new network probe, traffic-generation mechanism, polling daemon, persistent time series, metrics service, or background I2PControl worker is added.
8. Transit admission, forwarding, scheduling, tunnel lifecycle, bandwidth accounting, and router behavior are unchanged.
9. Core additions, if any, are protocol-neutral read-only observations.
10. M049's other three fields and all M045–M048 fields are not reopened.
11. No upstream interaction or contribution preparation is authorized.

## 6. Work packages

### WP1 — Pin exact rolling semantics and owner feasibility

Before editing production code:

1. Re-read the pinned Proposal 170 field definition and the read-only i2pd `GetTransitBandwidth15s()` implementation path, including startup/window behavior and units.
2. Record whether the reference uses an exact trailing window, sample buckets, smoothing, or another deterministic rule.
3. Audit `EventManager` timer cadence, configurable `refresh_interval`, cumulative transit counter ownership, and runtime monotonic-clock primitives.
4. Decide whether exact semantics can be maintained at the existing event owner without a new task and without changing tunnel/transport data-plane paths.

If not, choose truthful demotion for this milestone. Do not approximate the reference value merely to preserve the current `available` count.

### WP2 — Remove request-driven sampling

Delete or retire `TransitBandwidthSampler` and its `Mutex` from `ProductionRouterInfoControl` as the production source for this canonical field.

`transit_bandwidth_15s()` must become a pure read of the neutral current metric/source. Calling it must not mutate the sample history that determines future results.

Add a static/behavioral regression that would fail if sampling is reintroduced exclusively inside the RouterInfo getter.

### WP3 — Add the smallest neutral owner-side source, if feasible

If WP1 proves exact source feasibility within `events.rs`:

- maintain only the minimum bounded rolling state needed by the pinned semantics;
- use real monotonic elapsed time rather than assuming timer punctuality;
- update it from the already-running event owner/cadence or another already-existing event transition;
- expose only a read-only numeric accessor through `EventHandle`/`EventMetrics`;
- do not expose history, mutable sampler state, timers, or channels to I2PControl;
- ensure source maintenance continues when there are zero I2PControl requests.

If the configurable existing cadence makes exact semantics impossible and the only fix would be a new I2PControl-specific periodic task or data-plane instrumentation outside budget, do not add it. Demote the field to `Unavailable` and document the missing owner.

### WP4 — Wire truthful source disposition

If exact source implementation succeeds:

- retain `SourceDisposition::Available` for `P170_NET_BW_TRANSIT_15S`;
- update the production adapter to read the neutral current value;
- keep exact bytes-per-second JSON integer behavior and direct-presence semantics.

If it does not succeed within budget:

- change only this contract row to `Unavailable` with a precise owner/reason;
- ensure direct canonical requests fail before partial assembly;
- update source maps/docs and source counts truthfully.

No other M049 source row may change disposition in this plan.

### WP5 — Regression and containment evidence

Add tests that explicitly distinguish traffic history from API request history:

1. create the production/core metric owner;
2. record or simulate cumulative transit traffic over the reference window while making zero RouterInfo transit-15s requests;
3. query once after the source has enough authoritative history and assert the reference-correct nonzero rate;
4. leave a RouterInfo-query gap longer than 15 seconds while continuing source-side traffic observation, then query again and assert the value reflects the current trailing window rather than resetting due to API inactivity;
5. cover zero traffic, startup/immature window, counter rollback/reset, restart, and maximum arithmetic values;
6. cover a non-default `refresh_interval` or otherwise prove the source does not assume one-second ticks;
7. prove repeated reads without new traffic do not mutate the underlying observation history.

The failing-before condition must demonstrate that the current request-local sampler cannot pass at least the no-prior-query and long-query-gap cases.

## 7. Failure, cancellation, restart, and contention

The rolling source is local read-only telemetry. Request cancellation must not cancel, reset, advance, or otherwise mutate the traffic history. A RouterInfo read returns immediately from current owned state.

Restart clears process-local history. Until the pinned reference-defined startup condition is satisfied, return only the exact reference-defined immature value; if that behavior cannot be established, return `Unavailable` rather than inventing zero.

Any synchronization must be bounded and short-lived. No lock may be held across `.await`, JSON serialization, network I/O, sleep, or tunnel processing. Observation failure must never alter transit forwarding.

## 8. Compatibility, migration, and security

No schema, configuration, persistence, authentication, TLS, AddressBook, tunnel-management, or compatibility-selector migration is authorized. The field spelling/type remain unchanged.

No private material, tunnel object, packet/message payload, mutable router handle, or command channel may enter the observation source. The source exposes at most owned primitive rolling-state data or a scalar current value.

External Proposal 170/i2pd material is read-only evidence only. Do not open, prepare, or update upstream issues, PRs, reviews, submissions, merge requests, or maintainer communications.

## 9. Verification

Run focused source tests first, including the no-prior-query and long-query-gap regressions. Then run at minimum:

```bash
cargo test -p emissary-core events --no-fail-fast
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter --no-fail-fast
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Use targeted formatting only and retain the repository's existing stable/nightly rustfmt qualification. Do not add CI, coverage, fuzz, soak, release, or platform-matrix infrastructure.

## 10. Documentation and static guards

Update the RouterInfo source map and Proposal 170 support documentation to describe the corrected owner or truthful unavailability. Add a static or structural guard that prevents `ProductionRouterInfoControl::transit_bandwidth_15s()` from owning a request-mutated rolling sample history again.

Do not rewrite M049's historical closure record as if the defect never occurred; M054 closure must explicitly supersede only its transit-15s finding.

## 11. Acceptance criteria

M054 may close only if one of these two dispositions is independently evidenced:

### Preferred capability closure

- the canonical field is backed by a request-independent source maintained by a canonical existing owner;
- no RouterInfo call is required to create the history used by a later RouterInfo call;
- no-prior-query and >15-second query-gap regressions pass;
- units/window/startup semantics match the pinned reference;
- memory/state is bounded and restart/reset behavior is correct;
- the only core production change is `emissary-core/src/events.rs`;
- no new task, poller, timer solely for I2PControl, or data-plane path change exists;
- M049's other three fields remain unchanged.

### Truthful limitation closure

- if exact semantics cannot be achieved within the authorized owner/path budget, the field is restored to explicit `Unavailable`;
- the request-driven sampler is removed as a claimed production source;
- no fabricated zero/approximation remains;
- the blocker identifies the exact missing owner/cadence primitive.

In either case, closure must update the review-corrected source matrix and must not claim the pre-review `40/1/2` matrix as final.

## 12. Stop conditions

Stop and record a new blocker rather than:

- adding a new background sampler task or polling daemon solely for I2PControl;
- touching tunnel/transport data-plane code to maintain this metric without a separately authorized plan;
- assuming configurable refresh cadence is one second;
- approximating 15-second semantics with request frequency, cumulative totals, or a different window;
- changing transit accounting or forwarding behavior;
- broadening into M050/M051 or unrelated RouterInfo work.

## 13. Closure evidence required

The closure record must include the defect reproduction, pinned reference-semantic notes, source-owner decision, failing-before/passing-after request-independence regression, exact changed-path audit, startup/reset/window evidence, source-count reconciliation, no-feature verification, security/containment review, and internal-only attestation.

M055 must remain blocked until this corrective disposition is accepted.