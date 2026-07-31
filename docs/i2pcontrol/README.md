# I2PControl for Emissary

Status: corrective pass required

Proposal 170 is still **Open**. This documentation is pinned to the
2026-05-20 revision (created and last updated 2026-05-20).

The prior M019A `closed internally against pinned revision` disposition is
invalidated by:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

Current roadmap and support classification:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- [proposal-170-support.md](proposal-170-support.md)

This document describes the current I2PControl HTTPS JSON-RPC service foundation in Emissary. It must not be read as a claim that Proposal 170 is complete.

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
- Authentication, token placement, secret persistence, and response-redaction behavior are under corrective review in M020/M021. Do not rely on the prior completion claim.

## HTTPS certificate behavior

I2PControl is served over HTTPS. Certificate behavior:

1. **Operator-provided**: If `certificate` and `private_key` paths are configured, those files are loaded.
2. **Managed self-signed**: If no paths are configured, a self-signed certificate is generated under `<base_path>/i2pcontrol-certs/`.
   - Generated only when I2PControl is enabled.
   - Written atomically; not regenerated on every start.
   - Certificate identity is stable across restarts.
   - Invalid existing material triggers regeneration.

**No plaintext HTTP fallback is supported.**

## Authentication corrective notice

The existing I2PControl contract authenticates with `API` and `Password`, returns a numeric `API` and a `Token`, and expects that token in the `params` object of subsequent protected requests.

The current Emissary implementation does not yet conform fully to that flow: it requires a nonstandard username, serializes `API` as a string, and primarily accepts the token through `X-I2PControl-Token`.

M020 owns the correction:

- `plans/implementation/i2pcontrol-proposal-170/020-base-i2pcontrol-and-jsonrpc-interoperability.md`

Until M020 lands, the following current implementation example describes the existing compatibility behavior, not the intended canonical contract:

```json
{
  "jsonrpc": "2.0",
  "method": "Authenticate",
  "params": {
    "API": 2,
    "Username": "i2pcontrol",
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
    "API": "2"
  }
}
```

Current protected requests use `X-I2PControl-Token`. M020 will restore standard `params.Token` and retain the header only as a separately documented compatibility extension if it remains unambiguous.

### Token behavior retained for review

- Tokens are cryptographically random and opaque.
- Tokens are stored in memory only.
- Tokens are invalidated on process restart.
- Token count is bounded.

The exact error-code and conflict behavior is part of M020 and must not be treated as closed before its implementation disposition.

## Foundation status

The repository contains a substantial I2PControl foundation:

- feature-gated HTTPS serving;
- bounded request bodies, connection tasks, and concurrent requests;
- typed Proposal 170 tunnel/action/domain models;
- exhaustive explicit unsupported tunnel backend registry;
- versioned generation-store persistence;
- passive service registry;
- bounded SAM observation handle;
- RouterInfo contract/source adapters;
- production composition and focused tests.

These components are retained candidate implementation, not proof of complete Proposal 170 conformance.

## Corrective sequence

| Milestone | Scope |
|---|---|
| M020 | existing I2PControl authentication/token/error and JSON-RPC correctness |
| M021 | exact TunnelManager wire, atomic persistence, and secret boundary |
| M022 | actual runtime AddressBook authority and source objects |
| M023 | startup tunnel inventory and ClientServicesInfo lifecycle/address truthfulness |
| M024 | recoverable bounded SAM observation |
| M025 | exact RouterInfo contract/source matrix |
| M026 | feasible bounded read-only router inspection sources |
| M027 | literal conformance, documentation reconciliation, and independent closure |

See `plans/implementation/i2pcontrol-proposal-170/README.md` for dependencies and handoff rules.

## Support dimensions

Every claim is separated into:

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