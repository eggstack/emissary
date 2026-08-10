# M049 — RouterInfo Rolling Transit, Tunnel Success, and Queue Sources

Status: closed

Planning baseline: `b759038`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Hard dependency: M048 closed

Milestone class: capability + infrastructure

## 1. Objective

Implement four Proposal 170 additions:

- `i2p.router.net.bw.transit.15s`;
- `i2p.router.net.tunnels.successrate`;
- `i2p.router.net.tunnels.queue`;
- `i2p.router.net.tunnels.tbmqueue`.

Use current cumulative/event sources where mathematically sufficient, and add only minimal neutral gauges at canonical tunnel queue/build transition points where exact semantics cannot be reconstructed in I2PControl.

## 2. Reference semantics

The pinned Proposal 170 text states these fields are adopted from i2pd unchanged. Read-only i2pd reference evidence shows:

- transit 15s is a rolling transit-bandwidth value;
- recent tunnel success uses `GetTunnelCreationSuccessRate()`, an EWMA-like tunnel creation success metric distinct from the cumulative total success rate;
- queue and TBM queue are instantaneous queue depths.

Do not implement recent success as the already-available cumulative success ratio. Do not substitute transit total bytes directly for a rate.

## 3. Current evidence

`EventHandle` already exposes cumulative transit inbound/outbound byte counters and cumulative tunnel build success/failure counters. This is sufficient for an I2PControl-owned rolling transit sampler. It may not preserve the event-order semantics required to reconstruct i2pd-style recent tunnel success. M048 establishes the bounded tunnel-inspection seam needed for queue state.

## 4. Invariants

1. Rolling-window/sampling policy belongs in I2PControl whenever cumulative counters are sufficient.
2. No new global metrics service, Prometheus dependency, polling daemon, or persistent time-series store.
3. A core gauge, if needed, is neutral and updated at an existing transition; it must not alter build decisions or queue behavior.
4. Queue values are read-only instantaneous snapshots from the actual owners, not approximations from task counts.
5. Counter reset/wrap/restart is handled explicitly; no negative or overflow-derived rates.
6. API requests never block waiting for a 15-second window to fill. Before sufficient samples exist, use the exact protocol/reference disposition established by fixtures; do not fabricate a mature rate.

## 5. Production budget

Primary: `emissary-cli/src/i2pcontrol/**` and composition only.

Core exceptions only if required by exact semantics:

- `emissary-core/src/events.rs` for a neutral recent-build gauge/counter input;
- M048 inspection/tunnel paths for queue/TBM queue depth publication.

No transport/tunnel algorithm change is authorized.

## 6. Work packages

1. Pin units, rounding, initialization, and recent-success semantics against Proposal 170/reference fixtures.
2. Add an I2PControl-local bounded rolling sampler over cumulative transit bytes; define monotonic-time sampling, maximum retained samples, restart behavior, and deterministic 15-second calculation.
3. Prove whether ordered build outcomes can be reconstructed from existing counters. If not, add the smallest neutral recent-success accumulator/gauge at the existing build-result publication point, matching reference semantics without coupling it to JSON-RPC.
4. Extend the M048 neutral tunnel snapshot with exact queue and TBM queue depths from their canonical owners.
5. Wire all four fields into `ProductionRouterInfoControl`, reuse existing serializers, and update only these four contract rows.

## 7. Failure/cancellation/restart/contention

Sampling is in-memory, bounded, and non-blocking. Cancellation drops only I2PControl sampling state. Counter rollback/restart resets the local window rather than emitting a bogus delta. Queue snapshot acquisition holds no lock across await or serialization. No persistence/migration is introduced.

## 8. Tests and verification

Focused tests: deterministic fake clock for 15s window; zero traffic; burst traffic; counter reset/wrap; startup before full window; ordered success/failure vectors against reference EWMA expectations; queue/TBM depth 0/N and rapid drain; no-feature path has no sampler.

Run I2PControl RouterInfo tests, core event/tunnel tests for touched gauges, feature/no-feature CLI suites, clippy on changed packages, and `git diff --check`.

## 9. Acceptance criteria

Transit 15s has correct units/window semantics; recent success is demonstrably not the cumulative ratio and matches reference fixtures; queue fields come from real queue owners; no new background infrastructure or data-plane behavior exists; all four fields are bounded/truthful and independently closed.

## 10. Stop conditions

Stop rather than guess units/rounding, approximate recent success with cumulative totals, or expose mutable queue structures.

## 11. Closure evidence

Closure includes reference-semantic fixtures, sampler math tests, initialization/reset evidence, queue-owner evidence, changed-path containment review, no-feature evidence, and internal-only attestation.
