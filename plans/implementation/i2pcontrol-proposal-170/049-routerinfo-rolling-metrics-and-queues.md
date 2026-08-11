# M049 — RouterInfo Rolling Transit, Tunnel Success, and Queue Sources

Status: corrected/closed through M054 and M056; transit-15s is unavailable and the other three fields retain accepted closure

Planning baseline: `b759038`

Post-closure corrective authority: `plans/implementation/i2pcontrol-proposal-170/054-m049-transit-15s-corrective.md`

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

Post-closure review at `970252c` invalidated only the transit-15s source: its sample history is created by RouterInfo reads and therefore is not request-independent router traffic state. M054 supersedes that requirement/closure finding. Recent tunnel success, queue, and TBM queue remain accepted unless another direct defect is demonstrated.

## 2. Reference semantics

The pinned Proposal 170 text states these fields are adopted from i2pd unchanged. Read-only i2pd reference evidence shows:

- transit 15s is a rolling transit-bandwidth value;
- recent tunnel success uses `GetTunnelCreationSuccessRate()`, an EWMA-like tunnel creation success metric distinct from the cumulative total success rate;
- queue and TBM queue are instantaneous queue depths.

Do not implement recent success as the already-available cumulative success ratio. Do not substitute transit total bytes directly for a rate.

## 3. Current evidence

`EventHandle` already exposes cumulative transit inbound/outbound byte counters and cumulative tunnel build success/failure counters. This is sufficient for arithmetic but the original assumption that an I2PControl-owned request-local rolling sampler was sufficient was disproven by post-closure review: the canonical field requires request-independent traffic history. M054 owns that correction.

M048 establishes the bounded tunnel-inspection seam needed for queue state.

## 4. Invariants

1. Rolling-window/sampling policy belongs in I2PControl only when the source history is request-independent; the RouterInfo getter must not create the only history used to answer future reads.
2. No new global metrics service, Prometheus dependency, polling daemon, or persistent time-series store.
3. A core gauge, if needed, is neutral and updated at an existing transition; it must not alter build decisions or queue behavior.
4. Queue values are read-only instantaneous snapshots from the actual owners, not approximations from task counts.
5. Counter reset/wrap/restart is handled explicitly; no negative or overflow-derived rates.
6. API requests never block waiting for a 15-second window to fill. Before sufficient authoritative history exists, use the exact protocol/reference disposition; do not fabricate a mature rate.

## 5. Production budget

Primary: `emissary-cli/src/i2pcontrol/**` and composition only.

Core exceptions only if required by exact semantics:

- `emissary-core/src/events.rs` for neutral recent-build/rolling observation;
- M048 inspection/tunnel paths for queue/TBM queue depth publication.

The post-closure corrective path is narrowed by M054 and the machine-readable boundary manifest. No transport/tunnel algorithm change is authorized.

## 6. Work packages

1. Pin units, rounding, initialization, and recent-success semantics against Proposal 170/reference fixtures.
2. Establish a request-independent bounded rolling transit source; the original request-local sampler approach is superseded by M054.
3. Prove whether ordered build outcomes can be reconstructed from existing counters. If not, add the smallest neutral recent-success accumulator/gauge at the existing build-result publication point, matching reference semantics without coupling it to JSON-RPC.
4. Extend the M048 neutral tunnel snapshot with exact queue and TBM queue depths from their canonical owners.
5. Wire all four fields into `ProductionRouterInfoControl`, reuse existing serializers, and update only these four contract rows when their sources are truthful.

## 7. Failure/cancellation/restart/contention

Sampling is in-memory, bounded, and non-blocking. Request cancellation must not reset or advance router-owned rolling history. Counter rollback/restart resets the window rather than emitting a bogus delta. Queue snapshot acquisition holds no lock across await or serialization. No persistence/migration is introduced.

## 8. Tests and verification

Focused tests: deterministic 15s semantics; zero traffic; burst traffic; counter reset/wrap; startup before full window; no-prior-query and long-query-gap request-independence regressions; ordered success/failure vectors against reference EWMA expectations; queue/TBM depth 0/N and rapid drain; no-feature path has no I2PControl-specific sampler.

Run I2PControl RouterInfo tests, core event/tunnel tests for touched gauges, feature/no-feature CLI suites, clippy on changed packages, and `git diff --check`.

## 9. Acceptance criteria

Transit 15s has correct units/window semantics and is independent of RouterInfo request cadence; recent success is demonstrably not the cumulative ratio and matches reference fixtures; queue fields come from real queue owners; no new prohibited background infrastructure or data-plane behavior exists; all available fields are bounded/truthful and independently closed.

M054 is the authoritative corrective plan for transit-15s. If exact request-independent semantics cannot be achieved within its budget, that field must be unavailable rather than approximated.

## 10. Stop conditions

Stop rather than guess units/rounding, approximate recent success with cumulative totals, use request history as router traffic history, expose mutable queue structures, or broaden beyond the corrective path budget.

## 11. Closure evidence

The original closure remains historical evidence but is partially invalidated for transit-15s. M054 closure must supersede that finding with reference-semantic, request-independence, initialization/reset, changed-path, no-feature, and internal-only evidence. The other three M049 fields retain their accepted closure unless independently invalidated.
