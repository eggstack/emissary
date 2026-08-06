# M040 Implementation Disposition — Startup Server Cancellation Owner

Status: implemented; closure accepted

Source plan:

- `plans/implementation/i2pcontrol-proposal-170/040-startup-server-cancellation-correction.md`

Implementation/test head: `c316487` — `fix: retain startup server cancellation owner`

## Finding and correction

`ServerTunnelManager::server_event_loop` now retains a local watch sender until
`run_single_server` returns. The sender is only a lifetime owner; it is not
exposed as an administrative handle and startup tasks remain externally owned.

The in-module startup-manager regression uses a bounded loopback fake SAM
endpoint that accepts the separate session and forwarding connections. It
asserts HELLO negotiation, SESSION CREATE, destination observation,
`STREAM FORWARD`, and continued runtime liveness. A companion low-level test
confirms that an intentionally closed sender still cancels the reusable
primitive.

## Verification

- startup-manager regression — pass;
- closed-sender primitive regression — pass;
- `cargo test -p emissary-cli --no-default-features` — pass, 56 tests;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` — pass, 8 tests;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m033_tunnel_lifecycle` — pass, 3 tests;
- feature-enabled all-target clippy with `-D warnings` — pass;
- `git diff --check` — pass.

Changed production path is limited to `emissary-cli/src/tunnel/server.rs`.
Startup ownership and control-plane supervisor ownership remain separate. No
core, protocol, tunnel-family, or upstream change occurred.

## Internal-only attestation

This correction and its evidence are internal to Emissary. No upstream or
third-party repository or maintainer channel was mutated, and no upstream
review, merge, adoption, submission, or contribution artifact was prepared.
