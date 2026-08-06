# M041 Implementation Disposition — Authentication Throttle Source Accounting

Status: implemented; closure accepted

Source plan:

- `plans/implementation/i2pcontrol-proposal-170/041-auth-throttle-source-accounting.md`

Implementation/test head: `f7a9b37` — `fix: account authentication failures by source IP`

Disposition tightening commit: see git history (`041-implementation-disposition.md` tightened after M043/M044 acceptance)

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

## Production paths changed

- `emissary-cli/src/i2pcontrol/auth.rs` — `HashMap<IpAddr, FailureState>`;
  `reserve_failure` replaces `delay_for_failure` + `record_failure`; `clear`
  normalizes via `.ip()`; new unit tests cover IPv4/IPv6 normalization,
  IP-independence, the documented delay schedule, and post-cancellation
  preservation.
- `emissary-cli/src/i2pcontrol/server.rs` — `invalid_password_response` calls
  the new atomic `reserve_failure`; `handle_authenticate_with_source` covers
  port-churn reconnect with a single throttle entry.
- `docs/i2pcontrol/security.md` — documents per-IP, in-memory only
  failed-authentication state; no forwarded-header, persistent, or
  distributed rate limiting.

No `emissary-core/**`, tunnel, AddressBook, RouterInfo, ClientServicesInfo,
token, password-hashing, or upstream change was introduced.

## Verification

- `throttle_normalizes_source_ports_to_one_ip_identity` — pass;
- `throttle_normalizes_ipv6_source_ports_to_one_ip_identity` — pass;
- `throttle_keeps_distinct_ips_independent` — pass;
- `throttle_reserves_concurrent_failures_atomically` — pass, distinct delay
  schedule as a multiset across 8 concurrent threads;
- `throttle_matches_documented_delay_schedule` — pass: ZERO, BASE, 2×, 4×,
  8×, 16×, 32×, capped at `THROTTLE_MAX_DELAY`;
- `throttle_reservation_preserved_through_dropped_sleep` — pass: the next
  reservation continues from the recorded count after the sleep is dropped;
- `authentication_throttle_is_shared_across_reconnect_ports` — pass: two
  wrong attempts from different ports share one entry, correct attempt
  clears it;
- `failed_authentication_is_bounded_and_throttled` — pass;
- `successful_authentication_resets_failure_state` — pass;
- `throttle_capacity_is_bounded_under_source_churn` — pass;
- `throttle_delay_is_bounded` — pass;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol i2pcontrol::auth` — 40 tests pass (lib + bin);
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol authentication` — 11 tests pass;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial` — 64 tests pass;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test golden_fixtures` — 44 tests pass;
- `cargo check -p emissary-cli --no-default-features` — pass (feature isolation);
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol` — pass;
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` — pass;
- `git diff --check` — pass.

## Failing-before/passing-after evidence

The ephemeral-port regression is verified against baseline `563e093` (the
defect baseline named in the plan). The same test fixture placed on the
baseline returns `left: 2, right: 1` (one entry per `SocketAddr`) and fails;
on `f7a9b37` and the disposition-tightening commit the same scenario yields a
single entry and passes. The temporary reproduction is not retained in the repository.

## Wire and contract evidence

Authentication errors, API versions, token issuance, protected-request
authentication, constant-time password comparison, oversized-rejection
behavior, and JSON-RPC fixture surfaces are unchanged. The package suite,
golden fixtures, and adversarial fixtures all pass against the corrective
head.

## Internal-only attestation

No upstream repository, issue, review, submission, adoption, merge, maintainer
channel, or contribution artifact was created or mutated. External Proposal
170 material was accessed read-only.