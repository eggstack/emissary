# M041 Implementation Disposition — Authentication Throttle Source Accounting

Status: implemented; closure accepted

Source plan:

- `plans/implementation/i2pcontrol-proposal-170/041-auth-throttle-source-accounting.md`

Implementation/test head: `f7a9b37` — `fix: account authentication failures by source IP`

## Finding and correction

`AuthThrottle` now normalizes the accepted `SocketAddr` to `IpAddr` before
accessing its bounded in-memory map. `reserve_failure` expires stale entries,
increments the failure count, applies deterministic capacity eviction, and
returns the delay in one short locked operation. The lock is released before
the handler sleeps. Invalid-password reservations therefore remain recorded if
the request is cancelled during its delay.

The authentication handler retains the existing errors, token behavior, API
versions, and constant-time password comparison. Successful authentication
clears the normalized source IP, including when it arrives from a new port.

## Verification

- `throttle_normalizes_source_ports_to_one_ip_identity` — pass;
- `throttle_reserves_concurrent_failures_atomically` — pass, distinct delay schedule;
- `authentication_throttle_is_shared_across_reconnect_ports` — pass;
- capacity and bounded-delay tests — pass;
- `failed_authentication_is_bounded_and_throttled` — pass;
- `successful_authentication_resets_failure_state` — pass;
- feature-enabled `auth` unit tests — 16 tests pass in both lib and binary targets;
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol` — pass;
- feature-enabled all-target clippy with `-D warnings` — pass;
- `git diff --check` — pass.

Changed production paths are limited to
`emissary-cli/src/i2pcontrol/auth.rs` and
`emissary-cli/src/i2pcontrol/server.rs`; the security documentation states the
per-IP, in-memory boundary. No authentication wire behavior changed.

## Internal-only attestation

No upstream repository, issue, review, submission, adoption, merge, maintainer
channel, or contribution artifact was created or mutated.
