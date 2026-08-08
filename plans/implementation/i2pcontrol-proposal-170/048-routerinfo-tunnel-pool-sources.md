# M048 — RouterInfo Tunnel-Pool Counts and Detail Sources

Status: blocked

Planning baseline: `b759038`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Hard dependency: M047 closed

Milestone class: capability + infrastructure

## 1. Objective

Make seven currently unavailable Proposal 170 tunnel sources operational:

- `i2p.router.net.tunnels.participating.info`;
- `i2p.router.net.tunnels.exploratory.inbound`;
- `i2p.router.net.tunnels.exploratory.outbound`;
- `i2p.router.net.tunnels.exploratory.info.list`;
- `i2p.router.net.tunnels.client.inbound`;
- `i2p.router.net.tunnels.client.outbound`;
- `i2p.router.net.tunnels.client.info.list`.

Expose sanitized, bounded, read-only tunnel lifecycle facts from the canonical tunnel owners while keeping Proposal 170 grouping, aggregation, redaction, bounds, and serialization under `i2pcontrol/**`.

## 2. Current evidence and architectural constraint

`TunnelManager` owns the exploratory pool and creates client pools, but client pools are spawned and not retained as a queryable manager collection. Transit tunnels have a separate owner. `Router::inspection_snapshot()` currently fills exploratory/client counts with literal zero placeholders; those placeholders must never be promoted to canonical availability.

A general mutable `TunnelManagerHandle` is not an acceptable inspection interface. Prefer one neutral bounded observation source shared with pool/transit owners, analogous to the existing passive SAM observation pattern but with no Proposal 170 policy in core.

## 3. Invariants

- No tunnel selection, build, expiration, recreation, routing, garlic/I2NP, LeaseSet, or transit admission behavior changes.
- No mutable pool/tunnel handles cross to I2PControl.
- No tunnel encryption/session keys, private destinations, build records, or message payloads enter observation DTOs.
- Core publishes only lifecycle facts required to reconstruct current state; I2PControl owns maps, grouping, deterministic ordering, response-size bounds, and wire DTOs.
- Observation failure must not fail or alter tunnel operation.
- The source is bounded and recoverable from authoritative create/establish/fail/expire/remove transitions; no unbounded event queue.

## 4. Production budget

Primary I2PControl paths plus composition in `main.rs`.

Core exception budget:

- `emissary-core/src/inspection.rs` neutral DTO/source types;
- `emissary-core/src/router/mod.rs` source construction/plumbing only;
- `emissary-core/src/tunnel/mod.rs` source plumbing only;
- `emissary-core/src/tunnel/pool.rs` minimal pool lifecycle publication;
- `emissary-core/src/tunnel/transit.rs` minimal participating-tunnel publication.

If another tunnel file appears necessary, stop and document the exact missing authoritative transition before expanding the budget.

## 5. Work packages

1. Re-read the pinned Proposal 170 object schemas and enumerate the exact non-secret fields required for participating, exploratory, and client detail rows.
2. Define neutral observation events/snapshots using internal tunnel IDs only where they are public/non-secret and necessary for stable lifecycle reconciliation.
3. Install a bounded source at tunnel-manager construction and pass narrow clones into exploratory, client, and transit owners.
4. Publish activation/update/removal facts synchronously or through a bounded non-blocking mechanism that cannot perturb the data plane.
5. Aggregate the live state in I2PControl and expose count/detail methods through `RouterInfoControl`.
6. Replace the seven unavailable contract rows only after focused lifecycle tests prove no stale entries, duplicates, or placeholder values.

## 6. Failure, cancellation, restart, contention

A dropped/failed observer leaves tunnel operation unchanged and marks the I2PControl snapshot incomplete until an authoritative recovery point; incomplete snapshots fail closed rather than returning partial state. No tunnel lock is held across observation serialization or await. Restart starts from empty observation state and repopulates from live owners; no persistence is introduced.

## 7. Tests and verification

Focused tests must cover exploratory and client inbound/outbound lifecycle, participating create/expire/remove, multiple pools, replacement/rebuild, observer overflow/failure recovery, response bounds, and absence of secret/live types. Re-run core tunnel tests, feature/no-feature CLI tests, I2PControl RouterInfo fixtures, clippy on changed packages, and `git diff --check`.

## 8. Acceptance criteria

All seven fields reflect actual current tunnel owners; zero/empty values are returned only after a complete authoritative snapshot; no core policy or wire terminology is introduced; core changes are passive observation only; default router behavior and tunnel tests remain unchanged; closure includes an upstream-sensitive changed-path review.

## 9. Stop conditions

Stop rather than retain tunnel objects for administrative convenience, introduce polling over mutable pools, change build/selection behavior, or copy sensitive tunnel material.

## 10. Closure evidence

Require source-transition diagrams, exact fixtures for all seven selectors, lifecycle/recovery tests, static secret/handle guards, changed-path accounting, no-feature evidence, and internal-only/no-upstream attestation.
