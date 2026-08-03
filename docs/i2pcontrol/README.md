# I2PControl for Emissary

Status: closed against the pinned 2026-05-20 Proposal 170 revision

Proposal 170 is still **Open**. This documentation is pinned to the
2026-05-20 revision (created and last updated 2026-05-20).

The prior M019A `closed internally against pinned revision` disposition is
historical and invalidated by:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

Final internal roadmap and support classification:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- [proposal-170-support.md](proposal-170-support.md)
- `plans/closure/i2pcontrol-proposal-170/027-closure.md`

This document describes Emissary's exact supported Proposal 170 wire surfaces
and their separate source/runtime/persistence dimensions. It does not claim
that every pinned source exists or that every tunnel data plane is available.

## Compile feature

I2PControl is an independent Cargo feature in `emissary-cli`. It is **not** enabled by default.

```bash
# Build without I2PControl (default)
cargo build -p emissary-cli

# Build with I2PControl enabled
cargo build -p emissary-cli --no-default-features --features i2pcontrol

# Build with both UI and I2PControl
cargo build -p emissary-cli --all-features
```

## Runtime enablement

Even when compiled with the `i2pcontrol` feature, the service is **disabled by default**.
It only starts when explicitly enabled in the configuration.

Add an `[i2pcontrol]` section to `router.toml`:

```toml
[i2pcontrol]
enabled = true
bind = "127.0.0.1:7650"
password = "your-secure-password"
```

## Configuration

| Field | Type | Default | Description |
|---|---|---|---|
| `enabled` | boolean | `false` | Enable I2PControl listener |
| `bind` | string | `"127.0.0.1:7650"` | Bind address |
| `password` | string | `""` | Authentication password |
| `certificate` | string | (managed) | Optional TLS certificate path |
| `private_key` | string | (managed) | Optional TLS private key path |

### Security notes

- **Default binding is loopback only** (`127.0.0.1:7650`). Non-loopback binding requires explicit configuration and produces a security warning.
- **Empty password is rejected** when I2PControl is enabled.
- **Existing configurations without `[i2pcontrol]`** parse unchanged and preserve prior behavior.
- Authentication, token placement, secret persistence, and response-redaction
  behavior are closed in M020/M021 and independently rechecked by M027.

## HTTPS certificate behavior

I2PControl is served over HTTPS. Certificate behavior:

1. **Operator-provided**: If `certificate` and `private_key` paths are configured, those files are loaded.
2. **Managed self-signed**: If no paths are configured, a self-signed certificate is generated under `<base_path>/i2pcontrol-certs/`.
   - Generated only when I2PControl is enabled.
   - Written atomically; not regenerated on every start.
   - Certificate identity is stable across restarts.
   - Invalid existing material triggers regeneration.

**No plaintext HTTP fallback is supported.**

## Authentication

Emissary follows the I2PControl authentication flow: `Authenticate` accepts
`API` and `Password`, returns a string `Token` and numeric `API`, and protected
requests place that token in `params.Token`.

```json
{
  "jsonrpc": "2.0",
  "method": "Authenticate",
  "params": {
    "API": 2,
    "Password": "your-password"
  },
  "id": 1
}
```

Current success response:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "Token": "hex-encoded-token",
    "API": 2
  }
}
```

Subsequent protected requests include `Token` with their method parameters:

```json
{
  "jsonrpc": "2.0",
  "method": "RouterInfo",
  "params": {
    "Token": "hex-encoded-token",
    "i2p.router.version": true
  },
  "id": 2
}
```

`X-I2PControl-Token` remains a compatibility-only transport. If both forms
are present, they must match; the header never overrides `params.Token`.

Authentication failures use the standard I2PControl-specific error inventory:
missing password (`-32001`), missing token (`-32002`), unknown token
(`-32003`), missing API version (`-32005`), and unsupported API version
(`-32006`). Token state is in-memory only, so restart invalidates all tokens.

Notifications execute normal authentication, validation, and handler side
effects but return HTTP `204 No Content`. An explicit `id: null` is retained as
a response ID and is not treated as a notification.

### Token behavior retained for review

- Tokens are cryptographically random and opaque.
- Tokens are stored in memory only.
- Tokens are invalidated on process restart.
- Token count is bounded.

The exact error-code and conflict behavior is part of M020 and must not be treated as closed before its implementation disposition.

## Foundation status

The repository contains a substantial I2PControl implementation with M027
literal conformance evidence:

- feature-gated HTTPS serving;
- bounded request bodies, connection tasks, and concurrent requests;
- typed Proposal 170 tunnel/action/domain models;
- exhaustive explicit unsupported tunnel backend registry;
- versioned generation-store persistence;
- passive service registry;
- bounded SAM observation handle;
- RouterInfo contract/source adapters;
- production composition and focused tests.

These components are not all operational Proposal 170 capability: unavailable
RouterInfo sources and unsupported tunnel data planes remain explicit.

## Corrective sequence

| Milestone | Scope |
|---|---|
| M020 | existing I2PControl authentication/token/error and JSON-RPC correctness |
| M021 | exact TunnelManager wire, atomic persistence, and secret boundary |
| M022 | actual runtime AddressBook authority and source objects |
| M023 | startup tunnel inventory and ClientServicesInfo lifecycle/address truthfulness |
| M024 | recoverable bounded SAM observation |
| M025 | exact RouterInfo contract/source matrix |
| M026 | closed bounded-source audit; no additional authoritative sources identified |
| M027 | closed: partial Proposal 170 support; literal conformance and independent reclosure |

See `plans/implementation/i2pcontrol-proposal-170/README.md` for dependencies and handoff rules.

## Support dimensions

M018 reconciled the wire contract and M019 independently accepted the final
head against the pinned revision. See the planning records for the bounded
closure statement and source metadata.

- **Wire** — exact names, casing, parameter-presence rules, response fields, and JSON types.
- **Source** — a truthful current Emissary data source exists.
- **Runtime** — a real backend performs the operation.
- **Persistence** — mutation is durable and failure-atomic.
- **Evidence** — literal external-contract, failure, restart, and production-composition proof exists.

Compatibility aliases, unavailable fields, administrative shadow stores, and unsupported backend stubs are not counted as full operational implementation.

## Missing tunnel data planes

Proposal 170 corrective work does **not** implement missing tunnel data planes. HTTP, IRC, SOCKS-IRC, CONNECT, Streamr, bidirectional, and other missing listener/destination/LeaseSet/traffic implementations remain separate security-focused work.

The current API may retain their definitions and explicit unsupported runtime behavior. It must not report them running or simulate success.

## No frontend controls

I2PControl does not add frontend controls, screens, views, or frontend-owned state. It runs independently of the UI. The corrective roadmap preserves this separation.

## Internal-only boundary

All work is internal to `eggstack/emissary`.

No plan authorizes upstream issues, pull requests, reviews, discussions, submissions, patches, maintainer outreach, merge preparation, or writes to any upstream/third-party repository. External specifications and source trees may be inspected read-only for internal correctness only.
