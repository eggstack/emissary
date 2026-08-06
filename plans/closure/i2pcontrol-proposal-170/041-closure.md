# I2PControl Proposal 170 Milestone M041 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/041-auth-throttle-source-accounting.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/041-implementation-disposition.md`

Implementation/test head: `f7a9b37`

Disposition tightening head: see git history (`041-closure.md` tightened after M043/M044 acceptance)

## 1. Finding

M041 closes both authentication accounting defects. Source ports no longer
create independent throttle identities, and concurrent invalid attempts reserve
successive counts before any delay is awaited.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| IP identity (IPv4) | `throttle_normalizes_source_ports_to_one_ip_identity` | pass |
| IP identity (IPv6) | `throttle_normalizes_ipv6_source_ports_to_one_ip_identity` | pass |
| Independent addresses | `throttle_keeps_distinct_ips_independent` | pass |
| Port churn (handler) | `authentication_throttle_is_shared_across_reconnect_ports` | pass |
| Atomic reservation | `throttle_reserves_concurrent_failures_atomically` (8-way barrier) | pass: counts/delays are unique and ordered as a multiset |
| Documented delay schedule | `throttle_matches_documented_delay_schedule` (ZERO, BASE, 2×, 4×, 8×, 16×, 32×, capped) | pass |
| Cancellation-safe reservation | `throttle_reservation_preserved_through_dropped_sleep` | pass: third reservation continues from incremented count |
| Delay bounds | `throttle_delay_is_bounded`; existing constants | pass |
| Capacity bounds | `throttle_capacity_is_bounded_under_source_churn` | pass |
| Success reset | `successful_authentication_resets_failure_state`; new-port clear in reconnect test | pass |
| Handler-level compatibility | `failed_authentication_is_bounded_and_throttled`, `authentication_throttle_is_shared_across_reconnect_ports`, full authentication suite | pass |
| Lock/sleep semantics | `reserve_failure` returns before handler `tokio::time::sleep`; no mutex across await | pass |
| Wire compatibility | authentication error, API, token, golden/adversarial fixtures | pass |
| Security boundary | `docs/i2pcontrol/security.md` documents per-IP, in-memory only; no forwarded-header logic | pass |
| Ephemeral-port regression on baseline | `m041_ephemeral_port_regression_baseline_check` against `563e093` (manual reproduction, not retained in tree) | fails on baseline (left=2, right=1) |

## 3. Residual findings

No high or medium security, compatibility, correctness, or evidence finding
remains. Throttling is intentionally process-local, in-memory, non-persistent,
and not proxy-aware or distributed.

## 4. Future-plan disposition

M042 is dependency-ready and was completed at `ef30155`. M043 is the combined
validation gate for M040–M042 and was completed at `342420e`. M044 is the
corrective final-head reclosure and was completed at `342420e`. The full
corrective sequence (M040–M044) is closed; no successor implementation plan
is dependency-ready, deferred RouterInfo sources and the ten unsupported
tunnel families remain outside this roadmap with no accepted owner.

## 5. Internal-only attestation

All work and evidence are internal-only. No upstream channel was mutated or
contacted. No upstream issue, pull request, review, submission, adoption,
merge, or contribution artifact was created or prepared under this plan.

**Disposition: closed.**