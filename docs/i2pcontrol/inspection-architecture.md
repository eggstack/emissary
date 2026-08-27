# I2PControl Inspection Architecture

Status: M014 implementation accepted by M017 (truthful live sources and local resource bounds)

This document describes the read-only inspection architecture for I2PControl Proposal 170 in Emissary.

The cross-domain completion inventory is maintained separately in
[`plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`](../../plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml).
It records current source dispositions and future owner budgets; this
read-only architecture remains partial until later capability milestones
provide production evidence.

## Design principles

1. **Read-only boundary**: Inspection requests never mutate router state
2. **Truthful state**: No fabricated values; unavailable data returns protocol-permitted null/error
3. **Bounded responses**: All collections and byte sizes have explicit limits
4. **No event consumption**: `EventSubscriber` is never consumed by I2PControl
5. **No core dependencies**: Core remains free of HTTP/JSON-RPC/Serde-JSON server dependencies
6. **Fail-closed startup**: Production store failures abort I2PControl initialization
7. **Shared identity**: All tunnel consumers share one loaded service object via `Arc`

## Architecture layers

```text
┌─────────────────────────────────────────────┐
│  I2PControl HTTPS Server (axum)             │
│  ├── Authentication (token service)         │
│  ├── JSON-RPC dispatch                      │
│  ├── Handler limiter (Semaphore)            │
│  └── Pre-spawn connection bound             │
├─────────────────────────────────────────────┤
│  RouterInfo Handler                         │
│  ├── Selector parsing (presence-only)       │
│  ├── Budget estimation (pre-query)          │
│  ├── Response assembly (only requested keys)│
│  └── Per-selector dispatch                  │
├─────────────────────────────────────────────┤
│  Data Sources                               │
│  ├── I2pControlState (startup values)       │
│  ├── EventMetrics (canonical live counters) │
│  ├── Recent traffic (unavailable unless fed)│
│  ├── LogRing (bounded, redacted, clearable) │
│  ├── RouterInfoControl trait (fakes/adapters)│
│  └── AddressBookControl trait (M003)        │
├─────────────────────────────────────────────┤
│  Core (emissary-core)                       │
│  ├── EventHandle (atomic counters)          │
│  ├── Narrow neutral inspection handles      │
│  └── Subsystem managers (tunnels, peers...) │
└─────────────────────────────────────────────┘
```

## Tunnel runtime ownership boundary

Inspection and tunnel administration share the I2PControl feature boundary,
but runtime protocol ownership remains separate from RouterInfo inspection.
M065 adds two small internal seams under `backends/runtime`:

```text
control-plane backend
  ├── client listener
  │     ├── validated local bind
  │     ├── one outbound Yosemite session
  │     └── bounded connection handlers
  └── accepted-stream server
        ├── persistent destination/session
        ├── SAM-derived TrustedPeerIdentity
        └── handler decides before local-target connect
```

The accepted-server seam uses Yosemite's application-visible `accept()` path,
not `STREAM FORWARD`, so future HTTP/IRC filters can inspect bounded initial
bytes and trusted peer identity before forwarding. Runtime tasks stop on the
exact instance cancellation path and are drained within a bounded timeout;
handler panics are isolated to their connection. A narrow option-capability
validator runs before listener/session allocation and reports only option names,
never stored values.

These helpers remain lifecycle infrastructure only. M065 itself did not
register specialized backends; the subsequent M066-M071 adapters keep their
protocol policy in I2PControl. Streamr intentionally does not use these
streaming helpers: its producer/consumer loops own Yosemite datagrams and Tokio
UDP directly, while startup-managed tunnel ownership remains unchanged.

Core exposes purpose-specific, bounded owned snapshots and passive owner-local
lifecycle facts. Aggregation, recovery, public bounds, and wire serialization
remain in the application adapter; there is no aggregate core snapshot or
control-plane policy in `emissary-core`.

## Key components

### I2pControlState

Shared application state holding:
- Token service for authentication
- Router info control adapter (`Arc<dyn RouterInfoControl>`)
- Address book control adapter (`Arc<dyn AddressBookControl>`)
- Tunnel manager control adapter (`Arc<dyn TunnelManagerControl>`)
- Control plane adapter (`Arc<dyn ControlPlane>`)
- Startup-retained values (router ID, RI bytes, RI Base64)
- MetricsSnapshot for cumulative counters
- RollingWindow for recent traffic
- Concurrency semaphore

Production state is constructed via `I2pControlState::new_production()` with all required
dependencies supplied explicitly. Test state is constructed via `I2pControlState::new_test()`
which installs fake adapters. The production constructor cannot omit a dependency or default
to a fake.

All trait-object fields use `Arc` (not `Box`) to enable shared identity across consumers.
The tunnel manager, router info, and address book all reference the same underlying service
objects through their `Arc` clones.

### RouterInfoControl trait

Defines the read-only interface for router inspection:
- Identity/version/uptime
- Network status (IPv4/IPv6)
- UDP/TCP transport snapshots
- NetDB summary
- Bandwidth metrics
- Tunnel summaries
- Peer lists and statistics
- Log snapshot/clear
- Address book state

All methods return `Result<T, InspectionError>` to distinguish unavailable,
failed, and successfully-empty states. The `InspectionGroup` enum identifies
snapshot groups for grouped request dispatch.

### EventMetrics

Read-only adapter over the application-owned `EventHandle`:
- Cumulative transport/transit bytes
- Atomic gauges for connected routers and participating tunnels
- Tunnel build successes/failures
- Current firewall status
- Non-destructive reads of canonical runtime state

### Recent traffic

Recent-window selectors are returned only when an existing production source is wired.
The current production adapter has no canonical recent-window source and returns the
existing unavailable error instead of fabricating zeroes.

### LogRing

Bounded, redacted, independently clearable log buffer:
- Fixed maximum entries and total bytes
- Redaction of private keys, passwords, tokens
- Clear affects only this ring
- Concurrent readers receive coherent snapshot
- Wired as `tracing_subscriber::Layer`

## Data flow

### Startup values

```
Router::new() → (Router, EventSubscriber, serialized_RI)
    ↓
setup_router() → I2pControlState::set_startup_values(router_id, RI_bytes, RI_b64)
    ↓
I2pControlState retains values for handler reads
```

### Cumulative metrics

```
Transport sessions → EventHandle::transport_inbound_bandwidth(bytes)
                   → EventHandle::transport_outbound_bandwidth(bytes)
    ↓
EventMetrics adapter over the canonical event handle
    ↓
Handler reads MetricsSnapshot::snapshot()
```

### Rolling traffic

```
No canonical production recent-window source is currently wired; interval selectors
therefore return unavailable rather than a fabricated zero.
```

### Log events

```
tracing::event!() → LogRingLayer::on_event()
    ↓
LogRing::push(entry) with redaction
    ↓
Handler reads LogRing::snapshot() for log queries
Handler calls LogRing::clear() for log clear
```

## Security properties

- Authentication before any inspection dispatch
- No private keys, session keys, or tokens in responses
- Log redaction before ring insertion
- Bounded response sizes prevent DoS
- No mutation of router state
- No consumption of frontend events
- No direct filesystem reads from handlers
- Error messages sanitized (no internal paths)

## Testing approach

- All tests use `FakeRouterInfoControl` which defaults to `Unavailable`
- Available-zero vs unavailable tests prove truthful state
- Handler tests verify no partial results on failure
- Static guards prevent fabricated defaults in production
- Unit tests per selector group
- `router_info_selectors_complete` test verifies selector count
- `unrelated_keys_absent` test verifies only-requested-key behavior
- LogRing tests: push/eviction/clear/redaction/concurrency
- EventMetrics adapter tests: live nonzero counters
- Budget enforcement tests: pre-query estimation

## Completion

All I2PControl milestones (M001–M004, M008–M009) are closed. M010–M012 have residual corrective findings resolved by M014. M014 implements spec-constrained truthfulness and local hardening. M015 is the independent closure gate.
