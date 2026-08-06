# I2PControl Proposal 170 Milestone M041 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/041-auth-throttle-source-accounting.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/041-implementation-disposition.md`

Implementation/test head: `f7a9b37`

## 1. Finding

M041 closes both authentication accounting defects. Source ports no longer
create independent throttle identities, and concurrent invalid attempts reserve
successive counts before any delay is awaited.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| IP identity | `AuthThrottle` map keyed by `IpAddr`; IPv4/IPv6 normalization tests | pass |
| Port churn | pure reservation and handler-level reconnect tests | pass |
| Atomic reservation | barrier-based concurrent reservation test | pass: counts/delays are unique and ordered as a multiset |
| Delay bounds | capacity and bounded-delay tests; existing constants | pass |
| Success reset | new-port handler success and existing reset test | pass |
| Lock/cancellation semantics | reservation returns before handler sleep | pass |
| Wire compatibility | authentication error, API, token, and fixture suites | pass |
| Security boundary | security documentation and no forwarded-header logic | pass |

## 3. Residual findings

No high or medium security, compatibility, correctness, or evidence finding
remains. Throttling is intentionally process-local, in-memory, non-persistent,
and not proxy-aware or distributed.

## 4. Future-plan disposition

M042 is dependency-ready and was completed at `ef30155`. M043 remains the
combined validation gate for M040–M042.

## 5. Internal-only attestation

All work and evidence are internal-only. No upstream channel was mutated or
contacted.
