# M073 Closure — Generic Tunnel Option Truthfulness Corrective

Status: closed; corrective history — M081 re-establishes the M073 invariant at the current head (`leaseSetEncType` is carried into the accepted-stream `SESSION CREATE`)

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/073-generic-tunnel-option-truthfulness-corrective.md`

## 1. Historical disposition

M073 closed against `3d1d8f1`. At that pinned implementation head, generic `client` and `server` backends validated typed/raw runtime-relevant option surfaces before allocation, rejected fields not consumed by the then-current runtime, and preserved the generic server's one supported I2CP session-shaping option, `leaseSetEncType`.

This historical evidence remains valid for that pinned commit. It does not assert that later runtime migrations preserved every invariant.

## 2. Requirement-to-evidence matrix at `3d1d8f1`

| Requirement | Evidence | Historical result |
|---|---|---|
| Generic client applies supported runtime fields | `backends/client.rs::config`; `ClientTunnelRuntimeConfig` | pass |
| Generic client rejects unsupported access/plaintext/custom/I2CP fields | `CLIENT_OPTIONS`; negative tests | pass |
| Generic server preserves loopback target/port behavior | then-current `backends/server.rs::runtime_config`; lifecycle tests | pass |
| Generic server preserves `leaseSetEncType` as its only I2CP runtime option | then-current server/Yosemite session mapping | pass at pinned head |
| Generic server rejects privacy/identity/consumer fields it cannot consume | `SERVER_OPTIONS`; negative tests | pass |
| Raw option names have bounded backend-local allowlists | client/server `validate_raw_options` | pass |
| Validation precedes allocation | start ordering and no-store/no-session tests | pass |
| Secret values are absent from errors/debug output | option-name-only errors; redaction tests | pass |
| No public schema/core/router behavior changed | scoped diff; M061/M062 containment | pass |

## 3. Historical verification

Passed on `3d1d8f1`:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

The focused library run passed 533 tests before subsequent closure-documentation changes. M061/M062 containment remained an explicit combined-head gate.

## 4. Historical compatibility/security review

At the pinned M073 head, recognized-but-unimplemented generic fields failed closed rather than being persisted and silently ignored. The correction remained inside the I2PControl production boundary plus authorized containment metadata and added no upstream interaction.

## 5. Post-closure regression discovered after M075

Independent review of current head `1618de172e7a78a193fc1bb117af269f31174030` found that M075's accepted-stream migration changed the generic server runtime shape:

- the backend still accepts `leaseSetEncType` as the sole supported generic-server I2CP option;
- the new `GenericServerRuntimeConfig`/`AcceptedServerRuntimeConfig` path does not carry that option;
- `run_accepted_server` therefore constructs Yosemite `SessionOptions` without applying it.

The current head again contains an accepted-but-ignored runtime-relevant option. That is a regression of the M073 invariant, not evidence that M073's pinned historical verification never passed.

Corrective owner:

- `plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md`.

M081 must apply `leaseSetEncType` in accepted-stream session setup or reject it before allocation. Restoring control-plane `STREAM FORWARD` is not authorized.

## 6. Current disposition

Do not use this closure record alone to claim current generic-server option truthfulness. The historical M073 evidence is accepted for `3d1d8f1`, but the workstream's present invariant remains corrective until M081 closes and M079 independently re-audits the final head.

External source access under this workstream remains read-only. No upstream review, merge, submission, or contribution preparation is authorized.
