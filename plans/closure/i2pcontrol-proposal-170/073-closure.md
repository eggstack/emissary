# M073 Closure — Generic Tunnel Option Truthfulness Corrective

Status: closed against implementation commit `3d1d8f1`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/073-generic-tunnel-option-truthfulness-corrective.md`

## 1. Disposition

M073 is closed. The generic `client` and `server` backends now validate their
typed and raw runtime-relevant option surfaces before destination-store lookup,
listener/session reservation, or task allocation. Options that are not
consumed by the existing runtime are rejected without exposing their values.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Generic client applies its supported runtime fields | `backends/client.rs::config`; `ClientTunnelRuntimeConfig` | pass |
| Generic client rejects `AccessList`, `AllowPlaintext`, custom, and I2CP options | `CLIENT_OPTIONS`; client negative tests | pass |
| Generic server preserves loopback target/port behavior | `backends/server.rs::runtime_config`; server lifecycle tests | pass |
| Generic server preserves `leaseSetEncType` as the only I2CP runtime option | `validate_i2cp_options`; existing Yosemite config mapping | pass |
| Generic server rejects privacy/identity/consumer fields it cannot consume | `SERVER_OPTIONS`; server negative tests | pass |
| Raw option names have bounded backend-local allowlists | `validate_raw_options` in client and server backends | pass |
| Validation precedes allocation | client/server `start` ordering and no-store/no-session tests | pass |
| Secret values are absent from errors/debug output | option errors contain names only; redaction tests and `TunnelDefinition` debug guard | pass |
| No public schema, core API, or router behavior changed | scoped diff and M062/M061 containment | pass |

## 3. Verification

Passed on `3d1d8f1`:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

The focused library run passed 533 tests before the subsequent M074 closure
documentation-only changes. M061/M062 containment remains an explicit
verification gate for the combined head.

## 4. Compatibility and security review

Existing generic runtime fields remain available. Recognized-but-unimplemented
fields now fail closed instead of being persisted and silently ignored. The
correction stays within `emissary-cli/src/i2pcontrol/**` plus the existing
containment test allowlist and adds no upstream interaction.

## 5. Future-plan disposition

M073's hard dependency is satisfied. M072's generic option rows may therefore
be accepted in its corrective reclosure. M074 was implemented immediately
after this predecessor and has its own closure record.

