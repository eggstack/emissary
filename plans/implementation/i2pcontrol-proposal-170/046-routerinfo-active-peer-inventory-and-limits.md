# M046 — RouterInfo Active-Peer Inventory and Transport Limits

Status: ready

Planning baseline: `b759038`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Milestone class: capability + infrastructure

Hard dependency: M045 closed

## 1. Objective

Make four Proposal 170 fields operational:

- `i2p.router.netdb.activepeers.list`;
- `i2p.router.netdb.activepeers.info`;
- `i2p.router.netdb.ntcp.limit`;
- `i2p.router.netdb.ssu.limit`.

Introduce the smallest neutral, cloneable, read-only transport-inspection seam needed to observe current connected peers and authoritative connection limits. Join active peer IDs to the M045 public RouterInfo directory inside I2PControl.

## 2. Current evidence

`TransportManager` already owns the live `routers` map and exposes bounded `connected_peer_ids(limit)` to `Router::inspection_snapshot()`. `Ntcp2Config` and `Ssu2Config` contain `max_connections`. No cloneable live inspection handle is currently passed to I2PControl; the one-shot `CoreSnapshot` is not sufficient for request-time truth.

## 3. Invariants

- Core exposure is neutral: no Proposal 170 names, JSON types, error codes, or serializers outside I2PControl.
- The new seam is snapshot-only: no dial/disconnect/control methods and no mutable transport/session handles.
- Active peer RouterInfo is resolved through M045; transport code does not duplicate NetDB/ProfileStorage ownership.
- `None`/unlimited connection configuration must not be mapped to an invented integer sentinel. Determine the actual finite/effective limit semantics from the transport owner and pinned/reference contract before changing the source disposition.
- No transport algorithm, connection admission behavior, timeout, congestion logic, or peer routing changes.

## 4. Production budget

Primary:

- `emissary-cli/src/i2pcontrol/{router_info.rs,router_info_handler.rs,production.rs,rpc.rs,server.rs}`;
- `emissary-cli/src/main.rs` composition only.

Core exception budget, only as required:

- `emissary-core/src/inspection.rs` neutral DTO/handle types;
- `emissary-core/src/transport/mod.rs` copy current IDs/limits into that source;
- `emissary-core/src/router/mod.rs` expose/clone the read-only handle.

No other core path is authorized by M046.

## 5. Work packages

1. Define a bounded neutral transport snapshot containing connected peer IDs and authoritative NTCP2/SSU2 limit state.
2. Make a cloneable read-only source available from `Router` without retaining `Router` in I2PControl.
3. Compose the source in `main.rs` only when I2PControl is enabled.
4. Implement active peer list/info by deterministic bounded join with M045.
5. Resolve exact limit semantics, add golden fixtures, then change only the four source dispositions to available.
6. Add static guards proving the handle contains no socket/session/channel/key/control type and no transport mutation method.

## 6. Failure/restart/contention

Snapshots must be copied under short synchronization and returned owned. No lock across await or serialization. Active-peer churn during a request may produce a coherent source snapshot followed by a missing public RI; the I2PControl adapter must apply one documented deterministic policy and must not fabricate an RI. Restart has no persistence/migration effect.

## 7. Tests and verification

Focused tests: empty/one/many active peers; bounded deterministic ordering; peer disappears between sources; NTCP/SSU finite limits; disabled transport; unlimited configuration disposition; no live handles in DTO.

Run focused I2PControl tests, core transport tests affected by the seam, no-feature CLI tests, full feature-enabled CLI tests, core tests for changed modules, clippy on changed packages, and `git diff --check`.

## 8. Acceptance criteria

All four fields use live authoritative sources; no invented limit value exists; I2PControl owns joining/bounds/serialization; core changes are confined to the three exception paths; default router behavior and transport tests are unchanged; closure independently reviews the core diff against upstream-sensitive containment rules.

## 9. Stop conditions

Stop if truthful limits require changing connection admission policy, or if active peers require exporting transport session objects rather than sanitized owned facts.

## 10. Closure evidence

Record exact changed paths, before/after source matrix, request fixtures for all four fields, contention/churn evidence, no-feature evidence, and internal-only/no-upstream attestation.
