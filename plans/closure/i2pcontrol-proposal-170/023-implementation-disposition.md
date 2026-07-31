# M023 Implementation Disposition — Startup Tunnel Inventory and ClientServicesInfo

Status: implemented

Implementation commit: `00a17b5` — Implement M023 startup tunnel inventory

Lifecycle feasibility decision: deferred as explicitly externally managed.
`ClientTunnelManager` owns a retrying `JoinSet` and `ServerTunnelManager` owns
manager-spawned tasks without independently cancellable named handles. A safe
`start(name)`, `stop(name)`, `restart(name)`, or `inspect(name)` adapter would
require new task authority or a supervisor/registry, so no adapter was added.
Proposal 170 lifecycle operations reject startup-owned names before touching
those managers.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| Startup client/server mapping | `StartupTunnelInventory::from_configs` maps already-parsed composition DTOs to bounded `StartupManaged` definitions; `m023_startup_inventory::startup_mapping_is_bounded_deterministic_and_read_only`. |
| One shared production inventory | `main.rs` constructs one inventory, passes a clone to `ServerTunnelManager` for destination publication and a clone through `ServerInitContext` to `ProductionTunnelManagerControl`; `production_composition_uses_shared_startup_inventory` static guard. |
| Deterministic union | Production list merges startup and persisted definitions through `BTreeMap`, bounds the combined count, and returns one sorted view. |
| Collision safety | Startup cross-type duplicate rejection, persisted collision fail-closed load, control-plane create/rename rejection, and read-only delete/lifecycle tests are in `m023_startup_inventory.rs`. |
| No startup persistence mutation | Startup definitions remain in `StartupTunnelInventory`; only control-plane definitions call `TunnelStore` mutations. |
| Actual server address | Existing Yosemite `Session::destination()` is published through a narrow callback; the server local TCP port/host is never used as the I2P address. |
| Truthful ClientServicesInfo | Client entries use `target_destination`; server entries use published `hosting_destination`; absent address data returns a method error instead of `""`. |
| Proxy exit lifecycle | HTTP and SOCKS composition tasks classify runtime errors, then publish generation-fenced `Stopped` after `run()` returns; observer tests cover stopped and stale-generation behavior. |
| Bounds and failure propagation | Startup and combined inventory limits are explicit; list/store/query failures propagate; no truncation or fabricated address is used. |

## Exact changed files

- `docs/i2pcontrol/client-services.md`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/tunnel-manager.md`
- `emissary-cli/src/i2pcontrol/client_services.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/server.rs`
- `emissary-cli/src/main.rs`
- `emissary-cli/src/tunnel/server.rs`
- `emissary-cli/tests/m023_startup_inventory.rs`
- `emissary-cli/tests/static_guards.rs`
- planning registry, roadmap, and M023 implementation status files

## Compatibility and migration

Startup configuration remains authoritative and is not rewritten or migrated.
Persisted control-plane generations remain readable unless their names collide
with startup ownership, in which case I2PControl initialization fails closed.
Control-plane definitions retain their existing durable owner and restart
behavior. ClientServicesInfo now returns an explicit method error when a
required actual destination is unavailable instead of an empty address.

## Security and scope attestation

The implementation adds no private-key or destination-path exposure to startup
DTOs, does not add a lifecycle supervisor, does not touch tunnel protocol/data
planes, and does not consult frontend state. Error messages for persisted
collisions and bounds are sanitized; server destination publication carries
only the actual public destination returned by the existing session. No
upstream repository, issue, pull request, remote service, or external write
was used.

## Frozen implementation/test head

The implementation and focused tests are frozen at commit `00a17b5`. Closure
evidence is recorded separately in `023-closure.md`.
