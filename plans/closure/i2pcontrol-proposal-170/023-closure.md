# I2PControl Proposal 170 Milestone M023 — Closure Status

Status: closed internally against pinned revision

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/023-startup-tunnel-inventory-and-client-services.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline reviewed: `b6ba56f`

Implementation commits:

- `00a17b5` — Implement M023 startup tunnel inventory

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/023-implementation-disposition.md`

## 1. Executive finding

M023 is complete for its bounded capability boundary. Production
`TunnelManagerControl` now exposes one deterministic inventory composed from
read-only startup generic client/server definitions and the durable
control-plane store. Startup ownership is immutable and collision-safe.
ClientServicesInfo uses actual remote/client and session-published/server I2P
destinations, with explicit failure for unknown required address state. HTTP
and SOCKS task exit now clears active state through the existing generation-
fenced observer path.

The generic startup managers were reviewed and no safe named lifecycle adapter
exists without introducing prohibited task supervision authority. Startup
lifecycle therefore remains explicitly externally managed and unsupported by
Proposal 170 operations.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Startup generic client/server definitions appear once | `m023_startup_inventory` mapping and combined-list tests | pass | `BTreeMap` ordering and cross-type duplicate rejection are covered. |
| Startup entries are `StartupManaged` and read-only | ownership, create, edit/rename, delete, and lifecycle tests | pass | No startup mutation reaches `TunnelStore`. |
| Persisted startup-name collision fails closed | `persisted_startup_name_collision_fails_closed` | pass | Load returns a sanitized collision error. |
| Restart reconstructs combined inventory | `production_inventory_combines_restartably_and_rejects_startup_mutations` | pass | Startup source is reconstructed from composition DTOs; control-plane data reloads from its generation store. |
| Client address provenance is truthful | `client_services_uses_actual_destinations_and_errors_when_missing` | pass | Only client target destination is serialized. |
| Server address provenance is truthful | same test plus `ServerTunnelManager` `Session::destination()` callback | pass | Local TCP target/listen metadata is not a server I2P address. |
| Missing address is explicit | same test | pass | No empty fabricated address is emitted. |
| HTTP normal/runtime exit becomes inactive | composition calls `observe_proxy_stopped` after `run()` and observer stopped tests | pass | Runtime failures are classified before terminal stopped. |
| SOCKS normal/runtime exit becomes inactive | equivalent SOCKS composition path and observer tests | pass | Same generation fencing and terminal state. |
| Stale observer cannot stop replacement | existing generation-fence tests in observer/service registry suites | pass | Stale updates are rejected. |
| Inventory query errors propagate | `resolve_i2ptunnel_live` awaits `list()` directly | pass | Handler returns JSON-RPC internal error rather than empty success. |
| Unsupported definitions remain inactive | existing TunnelManager unsupported backend and lifecycle suites | pass | M023 adds no missing data plane. |
| Combined inventory is bounded | startup oversize test and production/list/serializer bounds | pass | Responses fail explicitly; no truncation. |
| Production uses one shared source and no fake | `production_composition_uses_shared_startup_inventory` static guard | pass | Main, server, and production seams are connected. |
| Lifecycle feasibility is recorded | implementation disposition | pass | Explicit externally-managed decision; no adapter or supervisor added. |

## 3. Production implementation evidence

Implemented:

- composition-time startup mapping using small parsed DTOs;
- shared `StartupTunnelInventory` with bounded deterministic reads;
- fail-closed startup/persisted name collision handling;
- read-only startup ownership checks for create, rename, delete, and lifecycle;
- actual Yosemite server destination publication;
- truthful ClientServicesInfo I2PTunnel serialization;
- proxy runtime-error classification and terminal stopped observation;
- focused integration and static composition tests;
- ownership, provenance, bounds, lifecycle, and operations documentation.

Not implemented by design:

- HTTP/IRC/SOCKS-IRC/CONNECT/Streamr/bidirectional tunnel data planes;
- generic named-task cancellation or supervision;
- startup configuration mutation or migration;
- polling, auto-restart, frontend state, or new dependencies.

## 4. Verification executed

### Commands run

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m023_startup_inventory
cargo test -p emissary-cli --no-default-features --features i2pcontrol client_services
cargo test -p emissary-cli --no-default-features --features i2pcontrol startup_managed
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol proxy
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo check -p emissary-cli --no-default-features
cargo +nightly fmt --manifest-path emissary-cli/Cargo.toml
cargo +nightly fmt --all -- --check
```

### Results

- New M023 integration suite: 5 passed.
- Focused ClientServicesInfo suite: 86 passed.
- Startup-managed suite: 10 passed.
- Production composition integration suite: 8 passed.
- Proxy-filtered suite: 48 passed.
- Full feature-gated package suite: 388 library tests, 431 binary tests,
  and all integration suites passed.
- Feature-gated check and no-feature check passed.
- Clippy passed with `-D warnings`.
- Touched-file nightly formatting passed. Workspace formatting reports only
  the pre-existing `examples/rust-tutorial/src/main.rs` match-arm difference;
  that unrelated file was not changed.

## 5. Invariant review

1. Startup entries are built before I2PControl serving and appear in the shared
   production list/get source.
2. Startup ownership is never persisted and cannot be edited/deleted/started
   through the control plane.
3. Persisted collisions fail closed; no shadowing or silent drop occurs.
4. ClientServicesInfo uses actual destination fields only and propagates
   unavailable state.
5. Proxy `Listening` is followed by `Failed` classification when appropriate
   and `Stopped` after task exit; stale generations cannot overwrite newer
   producers.
6. Inventory and response bounds are explicit and deterministic.
7. No unsupported data plane, lifecycle supervisor, poller, or frontend state
   was introduced.

## 6. Failure and recovery review

Cross-source duplicate names, persisted collisions, oversized startup/combined
inventories, absent server destinations, store/list failures, stale observer
generations, proxy constructor failures, and proxy runtime failures are all
explicitly handled. Restart reloads the control-plane generation and rebuilds
the startup source from the same parsed configuration. No auto-restart policy
was added; existing manager retry behavior remains unchanged.

## 7. Migration and compatibility review

Existing startup configuration remains authoritative and unchanged. Existing
control-plane generations remain readable except for explicit ownership-name
collisions requiring operator resolution. Capitalized compatibility actions use
the same combined inventory and ownership checks. The only public correction is
that missing I2P address data now returns an explicit method error rather than
an empty string.

## 8. Security review

Authentication and authorization remain in the existing protected dispatcher.
Startup DTOs omit private destination paths and key material. Collision,
bound, and unavailable-source errors do not include secrets or full stored
definitions. No request selects startup files, no startup config is rewritten,
and no new task authority or network capability is exposed.

## 9. Documentation and operations

Updated ClientServicesInfo, TunnelManager, Proposal 170 support, and planning
records document ownership, address provenance, proxy exit semantics, bounds,
and the deferred lifecycle-adapter decision. The static composition guard and
the M023 integration suite provide repository-local drift detection.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| medium | SAM observation overflow remains sticky/incomplete under the existing bounded source | ClientServicesInfo SAM recovery is outside M023 | M024, now dependency-ready |
| high claim defect | RouterInfo source/claim matrix remains unresolved | Whole Proposal 170 disposition remains corrective-pass required | M025–M027; M023 supplies its tunnel/source evidence |

Neither finding is an M023 implementation defect or a reason to reopen this
bounded milestone.

## 11. Roadmap disposition

M023 is closed internally against the pinned Proposal 170 revision. M024 is
unblocked and moved to `ready`. M025 remains blocked on M024 as well as its
other declared dependencies; M026 remains blocked on M025; M027 remains
blocked on M020–M026. The subsystem remains `corrective pass required` until
M027 performs its final independent reclosure.

## 12. Registry updates

Updated:

- `plans/registry.md`: M023 resolved, M024 ready, M025–M027 unchanged blocked;
- `plans/implementation/i2pcontrol-proposal-170/README.md`: M023 closed and
  M024 ready;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`: matching status table;
- `docs/i2pcontrol/proposal-170-support.md`: current executable handoff M024.

No remote CI, upstream contribution, external write, or final subsystem
completion claim was made.
