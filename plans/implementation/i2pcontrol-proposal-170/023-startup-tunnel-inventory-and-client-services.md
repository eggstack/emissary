# M023 — Startup Tunnel Inventory and ClientServicesInfo Truthfulness

Status: implemented

Primary class: capability/ownership corrective pass

Hard dependency:

- M021 closed for implementation

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Prior defect record:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

## 1. Bounded objective

Make `TunnelManager` inventory and `ClientServicesInfo` truthfully reflect Emissary services that already exist, while keeping missing tunnel data planes and broad lifecycle redesign out of scope.

This milestone owns:

- read-only import of startup-configured generic client/server tunnels as `StartupManaged` definitions;
- collision and ownership rules between startup-managed and control-plane definitions;
- narrowly justified lifecycle adapters for already-existing generic client/server managers only if the current manager architecture can target one named tunnel safely;
- actual proxy/listener exit observation;
- truthful I2PTunnel address/port sourcing;
- production composition proving one shared inventory source.

It does not implement any missing tunnel type and does not make startup configuration writable through I2PControl.

## 2. Current defects

The running CLI starts `ClientTunnelManager` and `ServerTunnelManager` from startup configuration, but the I2PControl production tunnel manager loads only its private generation store. Startup tunnels therefore do not appear in `TunnelManager.get`, RouterInfo I2PTunnel summaries, or ClientServicesInfo.

`StartupManaged` ownership exists in the domain model but is not populated by production composition.

HTTP and SOCKS observers publish `Listening` or constructor failure, but task exit does not publish `Stopped`, allowing stale enabled state after a bound proxy terminates.

ClientServicesInfo currently derives a tunnel `address` from target-destination or local hosting-target fields. Server local target hosts are not I2P destinations and must not be presented as such.

## 3. Required invariants

1. Every startup-configured generic client/server tunnel appears once in the shared inventory with `StartupManaged` ownership.
2. Startup-managed entries are read-only through Proposal 170 unless a safe existing lifecycle adapter is explicitly proven and limited to start/stop/restart.
3. Control-plane create/rename cannot collide with a startup-managed name.
4. I2PControl never rewrites startup tunnel configuration.
5. Missing tunnel types remain unsupported administrative definitions only.
6. ClientServicesInfo enabled state is true only while the corresponding listener/service is actually active.
7. Proxy task exit publishes inactive state even after successful bind.
8. I2PTunnel client/server address fields come only from actual I2P destination state; local target host/port is not substituted.
9. Unknown destination/address state is explicit and does not become an empty or fabricated string.
10. Inventory is bounded and deterministic.
11. No generic task registry, service supervisor, or new runtime manager is introduced.
12. No frontend state is consulted.

## 4. Explicit non-goals

- no HTTP/IRC/SOCKS-IRC/CONNECT/Streamr/bidirectional tunnel implementation;
- no startup tunnel config mutation or migration into the control-plane store;
- no generic named-task cancellation framework;
- no changes to tunnel construction, streaming, LeaseSets, routing, or cryptography;
- no auto-restart/supervision policy;
- no new polling task for service status;
- no new dependency, CI, release, frontend, or upstream work.

## 5. Expected file boundary

Primary I2PControl files:

- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/control_plane.rs`
- `emissary-cli/src/i2pcontrol/client_services.rs`
- `emissary-cli/src/i2pcontrol/observers.rs`
- `emissary-cli/src/i2pcontrol/server.rs`
- tunnel domain/store code only for ownership-aware collision behavior.

Permitted external composition changes:

- `emissary-cli/src/main.rs` to map already-parsed startup tunnel definitions into read-only DTOs and publish proxy exit transitions;
- existing generic client/server tunnel manager modules only if one narrow named lifecycle handle already fits their ownership model or can be added without redesign.

No `emissary-core` changes are expected in this milestone.

## 6. Required work packages

### WP1 — Startup definition mapping

Inspect the current startup client/server tunnel configuration types and produce a bounded, deterministic mapping to Proposal 170 inventory fields:

- name;
- generic type (`client` or `server`) only when semantically correct;
- ownership `StartupManaged`;
- truthful runtime state or `ExternallyManaged`;
- actual I2P destination/address if available;
- local target/listen metadata retained only in internal/raw configuration fields where the exact contract permits it.

Do not infer unsupported specific types from configuration heuristics.

The mapping should occur at composition/startup, not by reparsing `router.toml` inside I2PControl.

### WP2 — Shared inventory model

Make the production tunnel control expose one logical inventory combining:

- startup-managed read-only definitions;
- control-plane persisted definitions.

Requirements:

- deterministic ordering;
- name uniqueness across both sets;
- startup entries cannot be edited/deleted;
- a persisted control-plane entry colliding with startup ownership causes fail-closed startup or explicit migration error, never shadowing;
- listing/get and ClientServicesInfo consume the same combined view;
- control-plane store remains the durable owner only for control-plane definitions.

Do not copy startup entries into the generation store as mutable definitions.

### WP3 — Existing generic lifecycle feasibility gate

Inspect whether the current generic client/server tunnel managers can safely expose a purpose-specific named handle with:

- `start(name)`;
- `stop(name)`;
- `restart(name)`;
- `inspect(name)`.

Only add the adapter if:

- names map one-to-one to manager-owned tasks;
- operations cannot affect unrelated tunnels;
- lifecycle authority already belongs to the manager;
- no new supervisor/task registry is required;
- shutdown/cancellation semantics are already defined.

If any condition fails, retain startup-managed lifecycle as explicit externally-managed/unsupported operation status and record it. Do not redesign the manager in this workstream.

### WP4 — Proxy exit observation

At the existing HTTP and SOCKS task spawn sites:

- publish `Listening` after successful bind as today;
- publish sanitized `Failed` on constructor/bind/runtime error as appropriate;
- publish `Stopped` whenever `run()` returns, regardless of success/error after the final error classification is recorded;
- ensure stale generations cannot overwrite a newer task's state;
- preserve current proxy lifecycle and shutdown ownership.

Do not add a sibling polling task or auto-restart.

### WP5 — Truthful ClientServicesInfo I2PTunnel serialization

For each inventory entry:

- client address is an actual remote/target I2P destination only when the Proposal 170 field calls for it;
- server address is the actual local I2P destination published by the server tunnel, not its local TCP target;
- server `port` uses the exact field semantics from the pinned proposal;
- absence of a destination is explicit according to the contract or causes a selector error when no neutral form exists;
- control-plane unsupported definitions never masquerade as active services;
- startup-managed entries reflect actual/external runtime state, not the control-plane store default.

### WP6 — Service lifecycle source review

Confirm source ownership for:

- HTTPProxy;
- SOCKS;
- I2CP;
- SAM listener state;
- BOB false;
- I2PTunnel inventory.

This milestone may correct direct listener/proxy state mapping. Active SAM session internals remain M024.

### WP7 — Bounds and failure propagation

- Bound startup and combined inventory counts using existing configuration limits or a conservative explicit API bound.
- Reject oversized combined responses explicitly; never truncate without a protocol rule.
- A failed inventory query propagates as a method error rather than empty objects.
- Missing actual address data is not converted to `""`.

## 7. Failure, cancellation, restart, and contention semantics

- Startup inventory is built before I2PControl begins serving.
- Collision between startup and persisted names aborts I2PControl initialization or the full router startup according to the established fail-closed composition policy; it must not silently drop either entry.
- Proxy task exit updates the current generation only; stale tasks cannot mark a replacement stopped.
- If a named lifecycle adapter is accepted, concurrent start/stop/restart is serialized by the existing manager and returns coherent status.
- If no safe adapter exists, operation fails before touching runtime state.
- Restart reconstructs the same startup/control-plane ownership split.
- Control-plane deletion never targets startup configuration or manager-owned tasks.

## 8. Compatibility and migration

- Existing startup configurations remain authoritative and unchanged.
- Existing persisted control-plane definitions remain readable unless they collide with startup names; collisions require explicit operator resolution.
- Capitalized compatibility actions observe the same ownership rules.
- ClientServicesInfo may change from empty/fabricated address strings to explicit unavailable/error behavior; this is a correctness correction and must be documented.
- No data-plane migration occurs.

## 9. Focused tests

Required tests:

1. Startup client/server definitions appear in TunnelManager list/get and ClientServicesInfo.
2. Startup entries are marked read-only and cannot be edit/delete targets.
3. Control-plane create and rename reject startup name collisions.
4. Restart reconstructs the same combined inventory without duplication.
5. Persisted collision at startup fails closed with a sanitized message.
6. HTTP proxy bind then normal exit yields `enabled: false` after exit.
7. HTTP proxy bind then runtime error yields failed/stopped inactive state.
8. SOCKS equivalent lifecycle tests.
9. Stale observer generation cannot overwrite a replacement proxy state.
10. Server local target host is never serialized as I2P address.
11. Missing destination data is explicit and never an empty fabricated address.
12. Inventory query failure does not return empty success.
13. Unsupported control-plane definitions remain configured/inactive.
14. If a generic lifecycle adapter is accepted: named operation affects only the target and preserves other tunnels.
15. If rejected: exact externally-managed operation status fixtures prove no runtime effect.
16. Combined inventory bound failure is explicit.

## 10. Verification commands

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol client_services
cargo test -p emissary-cli --no-default-features --features i2pcontrol startup_managed
cargo test -p emissary-cli --no-default-features --features i2pcontrol production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol proxy
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

No live I2P network, missing tunnel implementation, remote CI, platform matrix, or soak test is required.

## 11. Documentation and static guards

- Document startup-managed versus control-plane ownership.
- Remove claims that all twelve tunnel types have equivalent runtime support.
- Record whether existing generic client/server lifecycle authority was accepted or explicitly deferred.
- Add a composition test/static guard proving production inventory includes the startup source and does not install a fake.
- Document proxy exit semantics and actual address provenance.

## 12. Acceptance criteria

M023 is implementation-complete only when:

- startup generic client/server inventory is visible and read-only;
- collisions cannot create contradictory ownership;
- ClientServicesInfo derives addresses and enabled state from actual sources;
- proxy exit clears active state;
- any accepted lifecycle adapter is narrow and target-specific, or the lack of safe authority is explicitly retained;
- external production changes are limited to composition and unavoidable existing-manager handles;
- tests cover restart, collision, task exit, stale generation, and address truthfulness;
- package verification passes or exact unrelated blockers are recorded;
- no missing data plane, broad manager redesign, dependency, CI, or upstream action occurs;
- an implementation disposition records the lifecycle feasibility decision.

## 13. Stop conditions

Stop if:

- startup names are not stable/unique enough for safe mapping;
- a named lifecycle adapter requires a new global task registry or manager rewrite;
- actual server I2P destination is unavailable and would have to be guessed;
- collision resolution would rewrite operator configuration;
- service truthfulness requires polling or frontend state;
- work expands into tunnel protocol/security implementation, CI, or upstream activity.

M023 closure unblocks M024 and contributes source evidence to M025.
