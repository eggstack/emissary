# M019A Closure Invalidation — Proposal 170 Corrective Reopen

Status: corrective pass required

Date: 2026-07-31

Invalidated closure:

- `plans/closure/i2pcontrol-proposal-170/019a-closure.md`
- frozen implementation head recorded there: `a3c4f469f4877e5ff4a0bb4230da298f0b367ed2`

Successor roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Successor implementation sequence:

- M020 through M027 under `plans/implementation/i2pcontrol-proposal-170/`

## 1. Disposition

M019A is preserved as historical evidence but is no longer authoritative closure.

The Proposal 170 workstream returns to `corrective pass required` because a later source/specification audit identified material defects that were not represented in the M019A requirement-to-evidence matrix. The prior closure's internal-only/no-upstream attestation remains valid; its implementation-completeness conclusion does not.

This invalidation does not discard all prior work. Existing durable stores, bounded request handling, explicit unsupported tunnel backends, direct Proposal 170 parameter forms, passive service observation, and bounded SAM observation remain candidate retained implementation. Each successor plan must revalidate only the part it consumes.

## 2. Material findings

| ID | Finding | Severity | Corrective owner |
|---|---|---|---|
| C20-01 | Standard I2PControl authentication is incompatible: mandatory nonstandard username, string-valued API response, header-only token transport, and generic application errors | high | M020 |
| C20-02 | JSON-RPC notifications are discarded without executing the requested operation; malformed request IDs may be coerced rather than rejected | high | M020 |
| C20-03 | Direct RouterInfo requests do not preserve the existing I2PControl selector surface and cannot accept the standard `Token` parameter | high | M020 |
| C21-01 | Canonical `TunnelManager.get` returns a non-Proposal-170 `Name`/`Type`/`State` shape rather than the required structured `info` and `rawConfig` contract | high | M021 |
| C21-02 | Tunnel edit/rename publishes remove and insert as separate durable generations and can lose the original definition if the second publication fails | high | M021 |
| C21-03 | Tunnel option validation is incomplete and `All` is validated by truth value rather than exact action/parameter semantics | medium | M021 |
| C21-04 | Secret-bearing options are duplicated into raw configuration, transparently serialized, and may be returned by `get`; restrictive-permission failure is ignored | high | M021 |
| C22-01 | AddressBook methods mutate an administrative shadow store that does not affect or truthfully reflect the running router's address-book owner | high | M022 |
| C22-02 | Canonical address-book subscription/config RouterInfo shapes and availability classifications contradict the implementation and pinned proposal | medium | M022/M025 |
| C23-01 | Startup-configured Emissary tunnels are not imported into the I2PControl inventory; `StartupManaged` ownership is not production-backed | high | M023 |
| C23-02 | Proxy exit does not publish `Stopped`, allowing stale `ClientServicesInfo` enabled/listening state | medium | M023 |
| C23-03 | I2PTunnel service addresses may be synthesized from unrelated local target fields or empty strings | high | M023 |
| C24-01 | SAM observation overflow is process-lifetime sticky and the per-session socket bound can permanently disable truthful snapshots after transient pressure | medium | M024 |
| C25-01 | The 43-selector contract labels many selectors unavailable while support documentation claims internal completion | high claim defect | M025 |
| C26-01 | NetDB, peer, tunnel-pool, queue, and recent-rate selectors lack bounded read-only sources; no plan distinguishes feasible adjacent snapshots from invasive new telemetry | medium | M026 |
| C27-01 | Existing conformance fixtures prove internal implementation shapes rather than the exact pinned external wire contract in several paths | high evidence defect | M027 |

## 3. Why prior verification missed the defects

M018A and M019A concentrated on the findings then active: transit-byte semantics, structured TunnelManager operation failures, manifest counting, and no-upstream governance. They treated earlier method-level fixtures and support matrices as retained evidence without re-deriving the base I2PControl contract, the complete TunnelManager `get` schema, runtime ownership, and persistence transaction semantics from source.

The verification suite was broad in test count but insufficiently adversarial in contract selection. It verified that current handlers matched repository fixtures; it did not consistently verify that the fixtures matched the pinned specification and existing I2PControl authentication flow. Persistence tests exercised successful round trips but not failure between rename publications. Service tests exercised startup/listening paths but not post-bind task exit, startup-managed inventory, or transient observation overflow recovery.

The corrective sequence therefore requires literal wire fixtures, failure injection at transaction boundaries, production-composition provenance tests, and claim/source reconciliation. It does not require a larger CI system.

## 4. Scope and authority

All successor work is internal to `eggstack/emissary`.

No successor plan authorizes:

- upstream issues, pull requests, merge requests, reviews, discussions, submissions, patches, or maintainer outreach;
- any write to an upstream or third-party repository;
- missing HTTP, IRC, SOCKS-IRC, Streamr, bidirectional, or other tunnel data-plane implementation;
- broad router, transport, NetDB, peer-selection, cryptographic, streaming, resolver, frontend, release, or CI redesign;
- fabricated values for unavailable Proposal 170 selectors.

Read-only external specification and reference-source inspection is permitted solely for internal implementation and verification.

## 5. Required corrective sequence

```text
M020 base I2PControl and JSON-RPC interoperability
    |
    v
M021 TunnelManager exact wire, atomic persistence, and secret boundary
    |
    +--------------------+
    |                    |
    v                    v
M022 AddressBook runtime bridge   M023 startup tunnel inventory and client-service truthfulness
    |                    |
    +---------+----------+
              v
M024 recoverable bounded SAM observation
              |
              v
M025 RouterInfo contract/source reconciliation
              |
              v
M026 bounded core inspection for feasible remaining selectors
              |
              v
M027 conformance, documentation, and independent reclosure
```

M020 is the only dependency-ready implementation handoff at registration time. Later plans are written and registered as dependency-blocked successors.

## 6. Closure effect

Until M027 records a new disposition:

- subsystem status is `corrective pass required`;
- documentation must not state unqualified or revision-bound completion;
- unavailable sources and unsupported runtimes remain truthful limitations, not completed capability;
- M019A may be cited only as historical evidence with this invalidation attached;
- no implementation or verification result may imply upstream review, acceptance, adoption, certification, or merge.

A future closure may conclude either:

1. `closed internally against pinned revision`, only if exact wire behavior and every claimed source/runtime dimension are supported by evidence; or
2. `partial Proposal 170 support`, if one or more pinned selectors remain truthfully unavailable after bounded-source work.

The second disposition is acceptable and preferable to scope expansion or fabricated data.