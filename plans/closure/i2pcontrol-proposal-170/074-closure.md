# M074 Closure — Shared Server Admission and Rate-Limit Hardening

Status: closed against implementation commit `3d1d8f1`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md`

## 1. Disposition

M074 is closed. `httpserver`, `ircserver`, and the inbound half of
`httpbidirserver` now consume one shared, I2PControl-owned admission state
implementation. Admission occurs after trusted Yosemite peer identity
validation and before handler task or local-target allocation.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Shared admission component | `backends/runtime/admission.rs` and `runtime/mod.rs` | pass |
| Default global ceiling is 30; hard maximum remains 128 | `ServerAdmissionPolicy`; paused-time default/global tests | pass |
| Explicit `MaxConcurrentConns` applies and invalid values fail closed | raw policy parsing; HTTP configuration tests | pass |
| Per-peer active ceiling is finite and defaults to 8 | `DEFAULT_MAX_CONCURRENT_PER_PEER`; fairness test | pass |
| Peer minute/hour/day controls | fixed monotonic windows and peer expiry tests | pass |
| Aggregate minute/hour/day controls | aggregate counter in shared state and aggregate test | pass |
| Bounded peer accounting and no churn eviction | fixed `MAX_PEER_ENTRIES`, fixed-size `PeerKey`, expiry queue, capacity test | pass |
| Expired inactive state is reclaimed | paused-time table-reclamation test | pass |
| RAII release covers return, panic isolation, cancellation, and abort | `AdmissionLease::Drop`; accepted runtime stores lease in the handler task | pass |
| Denial precedes handler/local target work | `run_accepted_server` ordering; protocol backends only receive admitted connections | pass |
| `httpbidirserver` reuses the HTTP accepted handler and one admission policy | `run_composite`, `make_accepted_handler`, shared-policy test | pass |
| Underspecified fields are rejected | backend raw allowlists reject `PerClientPeriod`, `TotalPeriod`, and `TotalBanTime` | pass |
| No timing theater or raw peer/private values in overload diagnostics | no sleeps/jitter; redacted `TrustedPeerIdentity` debug; rejection enum has no peer value | pass |
| Production path containment remains within authorized I2PControl seams | M061/M062 guards and scoped diff | pass |

The bounded peer table stores at most 4096 fixed eight-byte keys plus compact
counter records and expiry metadata per tunnel generation. It retains active
and rate-state peers; unseen identities are rejected while capacity is full.

## 3. Implementation details

`ServerAdmissionPolicy` uses reference-scale defaults: peer rates 30/80/200
per minute/hour/day, aggregate rates 50 per minute and unlimited per hour/day.
Explicit zero disables an individual rate. Windows use Tokio's monotonic
clock, are O(1) per admission, and are deterministic under paused time.

The task-group semaphore receives the same configured global capacity as the
admission policy; there is no second independently configurable server limit.
Each accepted task owns its `AdmissionLease`, so normal completion, parser
failure, handler panic isolation, cancellation, and task abort all release the
global and peer active counts by drop.

## 4. Verification

Passed on the combined implementation head:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
rustfmt +nightly --check --edition 2021 --config-path rustfmt.toml <all touched Rust files>
git diff --check
```

The initial combined verification found a stale M062 allowlist for security
plan/closure paths already introduced after its pinned baseline. The allowlist
was updated to cover the current M073-M079 planning sequence and the new
admission module; the rerun passed. This was containment metadata repair, not
a runtime exception.

The repository's inherited `cargo check -p emissary-core --no-default-features`
limitation remains outside M074: unrelated feature-disabled core modules still
import `RwLock` without the feature-gated provider. No M074 core production
change was made.

## 5. Security, compatibility, and internal-only review

No Proposal 170 wire field, tunnel type, router algorithm, SAM/core API, local
destination selection rule, or application parser was added. HTTP and IRC
sanitizers remain their protocol owners. The implementation uses only the
trusted peer identity from Yosemite, does not log private destination material,
and performs no upstream writes, review requests, or submission preparation.

## 6. Future-plan disposition

M073 and M074 are closed. M075, M076, and M077 now have their named hard
dependencies satisfied and are marked ready in their handoff records; M075 is
the next registered handoff. M078 remains blocked behind the ordered M075-M077
sequence, and M079 remains blocked until M074-M078 close.

