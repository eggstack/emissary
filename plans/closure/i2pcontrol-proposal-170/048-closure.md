# M048 Closure — RouterInfo Tunnel-Pool Counts and Detail Sources

Status: closed

Plan: `plans/implementation/i2pcontrol-proposal-170/048-routerinfo-tunnel-pool-sources.md`

Implementation commit: `0c50a21` (`i2pcontrol: expose live tunnel pool sources`)

Closure date: 2026-08-10

Pinned contract: Proposal 170, `I2PControl Expansion`, Open, revision 2026-05-20.

## Disposition

M048 is closed internally against the pinned Proposal 170 revision. All seven
planned tunnel-pool selectors have a production owner, exact wire fixtures,
bounded output, and live lifecycle coverage. Proposal 170 remains only partial:
the resulting RouterInfo matrix is 31 available, 1 protocol-permitted neutral,
and 11 unavailable.

## Source-transition model

The observation source is a bounded, cloneable core handle. It contains only
owned primitive facts; it never contains a pool, tunnel, channel, key, payload,
or router authority.

```text
TunnelPool<TunnelKind>
  ├─ successful outbound establish ──> publish(pool, tunnel, outbound)
  ├─ successful inbound establish  ──> publish(pool, tunnel, inbound)
  ├─ inbound expiry                 ──> remove(pool, tunnel, inbound)
  ├─ outbound destroy/failure       ──> remove(pool, tunnel, outbound)
  └─ pool drop                      ──> remove_pool(pool)

TransitTunnelManager
  ├─ accepted transit tunnel         ──> publish(participating, tunnel)
  ├─ completed/failed transit task  ──> remove(participating, tunnel)
  └─ manager drop                   ──> remove_pool(participating)

                    shared bounded TunnelInspection
                                  │
                                  ▼
       LiveTunnelSource → neutral TunnelDetails → I2PControl grouping
                                  │
                                  ▼
             Proposal 170 counts and minimal detail rows
```

Each request takes one source snapshot. I2PControl groups rows by
participating/exploratory/client, derives inbound/outbound counts, emits
deterministic primitive rows, and applies item and serialized-byte limits.
Proposal 170 policy and JSON/wire names remain in `emissary-cli`.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Participating detail is live and current | `TransitTunnelManager` publishes accepted transit tunnels and removes both successful and failed join results; `proposal_tunnel_sources_return_exact_live_rows_and_counts` | satisfied |
| Exploratory inbound/outbound counts and rows are live | `TunnelPool` publishes only after successful establish and removes on expiry/drop; exact fixture covers both directions | satisfied |
| Client inbound/outbound counts and rows are live across multiple pools | `TunnelManager` assigns process-local pool IDs and passes narrow observations to every client pool; fixture covers pool ID and direction | satisfied |
| No stale entries, duplicates, or placeholder promotion | `TunnelInspection` replaces exact duplicate facts, removes on terminal transitions, drops pool entries on owner destruction, and no longer uses the old zero summary for canonical fields | satisfied |
| Failure and cancellation do not perturb the data plane | publication is best-effort and ignored by owners; failed tunnel builds are never published; source locks are held only during primitive state mutation/copy | satisfied |
| Bounded and fail-closed source | `MAX_TUNNEL_INSPECTION_ENTRIES`, request `MAX_TUNNEL_DETAIL_ENTRIES`, 4 MiB detail serialization bound, and 10 MiB aggregate response bound; overflow returns `Incomplete` until owner recovery or restart | satisfied |
| Recovery and restart semantics | `tunnel_inspection_fails_closed_after_overflow_until_recovery` exercises overflow, fail-closed reads, and owner recovery; construction starts empty and lifecycle publication repopulates after restart | satisfied |
| No sensitive or mutable types cross the boundary | `tunnel_inspection_contains_only_bounded_public_facts` plus core DTO definitions use numeric IDs/enums and no live handles, keys, or payloads | satisfied |
| Exact seven selector dispositions and fixtures | `router_info_keys::PROPOSAL_170_CONTRACT`, `docs/i2pcontrol/router-info-source-map.md`, `conformance_manifest`, and `router_info_truthfulness` | satisfied |
| Compatibility and no-feature behavior remain intact | CLI no-feature suite and I2PControl conformance suite; no new wire fields outside the seven canonical selectors | satisfied |

## Field-owner matrix

| Selector group | Production owner | Neutral fact | Wire mapping |
|---|---|---|---|
| participating.info | `TransitTunnelManager` | tunnel ID, participating kind | `[{"tunnelId": n}]` |
| exploratory.inbound/outbound/info.list | exploratory `TunnelPool` | tunnel ID, direction | counts by direction; rows include `tunnelId`, `direction` |
| client.inbound/outbound/info.list | each client `TunnelPool` | pool ID, tunnel ID, direction | counts by direction; rows include `poolId`, `tunnelId`, `direction` |

The pinned Proposal 170 text specifies these detail values as lists of maps but
does not prescribe member names. This implementation therefore uses the
smallest stable internal row shape required to identify a live tunnel and its
pool/direction, without exposing tunnel internals.

## Verification

Successful checks:

- `rtk cargo check -p emissary-core` — 0 errors.
- `rtk cargo test -p emissary-core --no-fail-fast` — 1,059 passed, 2 ignored.
- `rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol` — 0 errors.
- `rtk cargo test -p emissary-core inspection --no-fail-fast` — 6 passed.
- `rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards --test conformance_manifest --test router_info_truthfulness --no-fail-fast` — 130 passed.
- `rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures --no-fail-fast` — 7 passed.
- `rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` — 1,358 passed.
- `rtk cargo test -p emissary-cli --no-default-features --no-fail-fast` — 56 passed.
- `rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` — no issues.
- `rtk git diff --check` — clean.

The repository's checked-in baseline does not pass the available stable or nightly
`cargo fmt --all -- --check`: both report pre-existing formatting differences
outside this change, caused by the repository's nightly-only rustfmt settings
being unavailable or differing from the checked-in baseline. No unrelated
formatter churn was retained.

`rtk cargo check -p emissary-core --no-default-features` also fails in the
pre-existing no-default configuration: `RwLock` is unresolved throughout
profile, destination/session, subsystem, router context, tunnel, and the
inspection module. The supported CLI no-feature suite passes; default-feature
core compilation and the full core suite pass. This is recorded as a baseline
configuration limitation, not an M048 regression.

## Invariant, failure, compatibility, and security review

- Tunnel selection, construction, encryption, routing, garlic/I2NP handling,
  LeaseSets, transit admission, expiration behavior, and runtime ownership are
  unchanged; observation calls are side effects only.
- No mutable pool/tunnel handle, private/session key, destination, build record,
  message, or channel crosses into I2PControl.
- Source contention is limited to a short `RwLock` mutation or copy. No source
  lock is held across serialization or an await, and observation failure is
  ignored by the data plane.
- Overflow is fail-closed rather than partially serialized. The owner recovery
  hook replaces the complete bounded set; process restart begins with an empty
  source and repopulates from subsequent authoritative lifecycle transitions.
- No persistence, polling task, network probe, migration, or new data plane was
  introduced. Existing compatibility aliases and no-feature compilation remain
  unchanged.
- The only external authority consulted was the pinned Proposal 170 page in
  read-only mode. No upstream repository, maintainer channel, review, merge,
  adoption, or contribution artifact was mutated or prepared. The implementation
  was committed only to the authorized internal repository/fork.

## Changed-path accounting

Core exception paths were limited to `inspection.rs`, `router/mod.rs`,
`tunnel/mod.rs`, `tunnel/pool/mod.rs`, and `tunnel/transit/mod.rs` for neutral
source plumbing and authoritative lifecycle publication. Non-core behavior was
limited to I2PControl adapter, handler, contract, server composition, tests,
and the source map. No files outside the M048 implementation budget were
retained.

## Unresolved findings

None. The formatter-baseline mismatch is recorded as a repository verification
limitation, not an M048 implementation defect.

## Future-plan unblock audit

M049 is unblocked and is now the sole `ready` handoff because its hard
dependency, M048, is closed. M050 and M051 remain blocked on M049 and M050
respectively; M052 remains blocked until M045–M051 are accepted. The registry,
roadmap, implementation README, and M049 plan were updated accordingly.

Disposition: **closed**.
