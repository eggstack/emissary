# I2PControl Tunnel Backends

Status: M080-M083 server security corrective sequence closed; M077-M079 corrective work remains

All twelve tunnel families have bounded production backends. The integrated
runtime/security phase remains open for the ordered M077-M079 corrective work.

This document describes the tunnel backend interface and registry in Emissary.

## Overview

The tunnel backend system provides a clean separation between the Proposal 170 control plane and the actual tunnel runtime. Each tunnel type resolves to exactly one backend implementation.

## Backend trait

The `TunnelBackend` trait defines the interface for tunnel runtime backends:

```rust
#[async_trait]
pub trait TunnelBackend: Send + Sync {
    fn tunnel_type(&self) -> TunnelType;
    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()>;
    async fn stop(&self, definition: &TunnelDefinition) -> BackendResult<()>;
    fn inspect(&self, definition: &TunnelDefinition) -> BackendStatus;
}
```

### Contract

- `start` must not allocate listeners, destinations, sessions, tasks, or traffic paths for unsupported backends
- `stop` of an inactive definition must be safe and resource-free
- `inspect` must return the current state without side effects
- All methods must honor caller deadlines without blocking
- Runtime inspection is authoritative for control-plane state; persisted
  `StartOnLoad` is intent, not proof that a runtime is active

### Error types

```rust
pub enum BackendError {
    NotImplemented { tunnel_type: TunnelType },
    InvalidState {
        tunnel_type: TunnelType,
        current_state: TunnelRuntimeState,
        attempted_action: &'static str,
    },
    Internal { message: String },
}
```

## Backend registry

The `TunnelBackendRegistry` provides exhaustive registration of backends for all 12 tunnel types.

### Construction

```rust
let registry = TunnelBackendRegistry::new(backends)?;
```

Construction fails if:
- A tunnel type is registered more than once (`DuplicateRegistration`)
- A tunnel type is missing from the registration (`MissingRegistration`)

### Lookup

```rust
let backend = registry.get(tunnel_type);
```

Lookup is total for valid tunnel types. The registry is constructed once at startup and not modified thereafter.

### Default and production registries

`create_default_registry()` maps all 12 tunnel types to
`UnsupportedTunnelBackend` for tests and dependency-light compositions. The
production constructor with a server store registers the closed real backends
for all twelve tunnel families. The composed server
backends use a fixed `server-destinations/` store below the I2PControl state
root.

```rust
pub fn create_default_registry() -> Result<TunnelBackendRegistry, RegistryError> {
    let backends: Vec<Arc<dyn TunnelBackend>> = ALL_TUNNEL_TYPES
        .iter()
        .map(|&tt| Arc::new(UnsupportedTunnelBackend::new(tt)) as Arc<dyn TunnelBackend>)
        .collect();
    TunnelBackendRegistry::new(backends)
}
```

## Unsupported backend

`UnsupportedTunnelBackend` is the baseline backend for all tunnel types at M002:

- Constructible for any declared tunnel type
- Returns typed `NotImplemented` from `start`
- Treats `stop` of an inactive definition as safe and resource-free
- Inspects as internal `Unsupported`
- Never spawns or binds anything

### Behavior

| Operation | Behavior |
|---|---|
| `start()` | Returns `Err(BackendError::NotImplemented { tunnel_type })` |
| `stop()` | Returns `Ok(())` unconditionally |
| `inspect()` | Returns `BackendStatus { runtime_state: Unsupported, message: "..." }` |

## Fake backend

`FakeTunnelBackend` supports deterministic success/failure/state scripting for handler tests:

```rust
let script = FakeBackendScript {
    start_action: FakeAction::Success,
    stop_action: FakeAction::Success,
    inspect_state: TunnelRuntimeState::Running,
    inspect_message: "running in test".to_string(),
};
let backend = FakeTunnelBackend::with_script(TunnelType::Socks, script);
```

### Scripted behavior

- `FakeAction::Success` - Operation succeeds
- `FakeAction::Error(BackendError)` - Operation fails with the given error

Scripts can be updated at runtime via `set_script()`.

### Fake registry

`FakeBackendRegistry` provides an in-memory registry for tests:

```rust
let mut registry = FakeBackendRegistry::new();
registry.register(Arc::new(FakeTunnelBackend::new(TunnelType::Socks)));
let backend = registry.get(TunnelType::Socks);
```

## Tunnel types

All 12 tunnel types are mapped to backends:

| Type | Category | Backend |
|---|---|---|
| `client` | Client | Yosemite streaming client with per-name supervisor |
| `httpclient` | Client | Bounded HTTP client proxy with direct-I2P routing and explicit I2P outproxy support |
| `ircclient` | Client | Bounded IRC anonymity filter over a Yosemite stream |
| `socks` | Client | Bounded SOCKS4a/SOCKS5 CONNECT proxy |
| `socksirc` | Client | SOCKS CONNECT composed with the IRC anonymity filter |
| `connectclient` | Client | Strict HTTP CONNECT proxy with direct-I2P routing and explicit I2P outproxy support |
| `streamrclient` | Client | Bounded Yosemite repliable datagram consumer |
| `server` | Server | Peer-admitted accepted-stream raw relay with per-name supervisor and persistent destination identity |
| `httpserver` | Server | Bounded filtered accepted-stream HTTP server |
| `httpbidirserver` | Server | Deprecated composed filtered HTTP server plus direct-I2P local proxy; no clearnet outproxy |
| `ircserver` | Server | Bounded filtered accepted-stream IRC server |
| `streamrserver` | Server | Bounded Yosemite repliable datagram producer |

The Streamr datagram types use a separate bounded producer/consumer runtime. The
deprecated `httpbidirserver` type is a composition of the accepted HTTP server
and HTTP client paths: its inbound side uses the HTTP server filter and its
local proxy side uses the HTTP client sanitizer with outproxy routing disabled.
It owns both halves under one lifecycle generation and keeps one persistent
server destination identity; its client SAM session is a non-published sibling
session and never creates a second persistent destination.

The generic server uses the I2PControl-owned accepted-stream runtime, so
authenticated peer admission and the shared global/per-peer rate and
concurrency limits run before the handler connects to a local target. After
admission it performs a raw byte relay to the fixed loopback target; it does
not parse HTTP, IRC, or another application protocol. It never uses SAM
`STREAM FORWARD`. The backend publishes the actual public destination after
session setup; private destination material is stored only in the backend-owned
secret store. Stopped control-plane servers retain identity across restart and
rename. Running server rename is rejected, and delete awaits the exact runtime
task before removing durable definition and identity state. Startup-managed
server forwarding remains owned by the startup server manager and is unchanged.

The generic server is the only accepted-server family that accepts an I2CP
session-shaping option. `i2cp.leaseSetEncType` is the sole supported key
(M081). Its validated value is threaded into the accepted-stream
`SESSION CREATE` command via `SessionOptions::lease_set_enc_type`; any other
I2CP key, or a non-`leaseSetEncType` raw option, is rejected before
destination-store/session/task allocation. The startup-managed server
forwards `leaseSetEncType` through its separate path and remains unchanged.
The `httpserver`, `httpbidirserver`, and `ircserver` families explicitly pass
`None` for the new shared field so they do not silently gain a capability
their own option contracts do not document.

The production manager serializes start, stop, restart, edit, rename, and
delete per exact tunnel name. Post-load reconciliation starts only eligible
control-plane client/server definitions with `StartOnLoad`; failures are
isolated and leave the definition stopped. Restart stops the prior generation
before reloading and starting the latest durable definition.

## Design rationale

### Why a registry?

The registry ensures:
- No tunnel type is accidentally left without a backend
- Duplicate registrations are caught at startup
- Backend selection is O(1) via HashMap lookup
- The full set of backends is visible in one place

### Why unsupported backends?

Unsupported backends allow the control plane to:
- Accept and persist tunnel definitions for all types
- Return typed errors when attempting to start unsupported tunnels
- Avoid resource allocation for types without runtime support
- Provide a clean migration path as real backends are implemented

### Why not a mutable registry?

The registry is immutable after construction because:
- Backend assignments are fixed at compile time
- Runtime modification would require complex synchronization
- The exhaustive check is simpler with a fixed set
