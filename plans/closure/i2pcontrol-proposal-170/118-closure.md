# M118 Closure — Neutral SAM Tunnel-Pool Variance and Backup Capability

Status: **closed**

Plan: `plans/implementation/i2pcontrol-proposal-170/118-neutral-sam-tunnel-pool-variance-backup-capability.md`

Implementation commit: `e7f3e04beccbf9f894ca23ec6d7e3ee21a180001`

Closure date: 2026-09-02

## Disposition

M118 is closed. The four generic SAM session options are now validated before pool
allocation, transferred through the existing SAM server configuration seam, and
consumed by the existing tunnel-pool owner. Length variance is applied per build;
backup quantity is represented as separate standby capacity with pool-local promotion
and replenishment. No I2PControl cell is promoted and M095 remains unchanged.

## Semantic reference freeze

The following sources were inspected read-only before implementation:

- [I2CP specification](https://i2p.net/en/docs/specs/i2cp/);
- [I2CP overview](https://i2p.net/en/docs/specs/i2cp-overview/);
- [I2P router `TunnelPool.java`](https://raw.githubusercontent.com/i2p/i2p.i2p/master/router/java/src/net/i2p/router/tunnel/pool/TunnelPool.java);
- [I2P router `TunnelPoolSettings.java`](https://raw.githubusercontent.com/i2p/i2p.i2p/master/router/java/src/net/i2p/router/TunnelPoolSettings.java).

The frozen behavior is:

- `lengthVariance=0` preserves the base length;
- positive variance samples an inclusive additive range `base..=base+variance`;
- negative variance samples an inclusive symmetric range `base-abs(variance)..=base+abs(variance)`;
- configured results must remain valid tunnel lengths;
- `backupQuantity` is separately built standby capacity, not active selector quantity
  and not extra unbounded attempts; a standby tunnel is promoted after active loss and
  the next maintenance pass replenishes standby capacity.

The I2CP documentation permits signed variance in `-7..7`. Emissary retains its
existing representable boundaries: inbound tunnel length `1..7` and outbound tunnel
length `1..8`. A session option whose complete possible range crosses either relevant
boundary is rejected. Zero-hop behavior was not broadened.

## Requirement-to-evidence matrix

| Requirement | Evidence and outcome |
|---|---|
| Parser validation | `emissary-core/src/sam/parser.rs` validates quantities, base lengths, backup quantities, signed variance, overflow/malformed values, and all resulting length bounds before returning `SESSION CREATE`. |
| Neutral configuration | `TunnelPoolConfig` in `emissary-core/src/tunnel/pool/mod.rs` has four default-zero fields; existing defaults and absent-option behavior remain unchanged. |
| SAM transfer | `emissary-core/src/sam/mod.rs` maps validated canonical options into the existing pool constructor; it does not duplicate policy. |
| Per-build variance | The pool samples each build independently through the runtime RNG and passes the selected hop count to the existing selector/build path. Zero variance returns the base without consuming RNG. |
| Standby separation | Inbound and outbound standby maps and pending markers are separate from active maps and selectors. Standby tunnels are not published, owner-registered, active-counted, or ordinarily selected. |
| Promotion/replenishment | Active expiration removes the active tunnel, promotes one pool-local standby only when active capacity is below the configured target, registers/publishes it as active, and leaves maintenance to replenish standby capacity. Standby expiration/failure is isolated from active accounting. |
| Failure/cancellation bounds | Pending markers are removed on completion/failure, build counts use saturating bounded arithmetic, existing maintenance cadence is retained, and shutdown removes both active and standby routing state. |
| Containment | M061/M062 authority records only the changed neutral owner paths plus the necessary existing `config.rs` construction seam, the containment test, and planning evidence. No Proposal/I2PControl API was added to core. |
| Security | M093 tunnel-build cryptography, selector boundaries, zero-hop limits, no-lock-across-I/O behavior, and pool-local ownership remain unchanged. |

## Changed paths

Implementation commit changes:

- `emissary-core/src/sam/parser.rs`;
- `emissary-core/src/sam/mod.rs`;
- `emissary-core/src/tunnel/pool/mod.rs`;
- `emissary-core/src/config.rs` — mechanical struct-update defaulting required by
  the new neutral fields;
- `emissary-cli/tests/m062_dependency_containment.rs`;
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`;
- this plan's implementation record.

Closure commit changes only this record, the M118 plan status/evidence, the Proposal
170 implementation README, the subsystem roadmap, the registry, and M111's readiness
status. No `emissary-cli/src/**`, `emissary-util/**`, Cargo/dependency, startup,
frontend, transport, NetDb, I2CP, workflow, release, or upstream path changed.

## Focused and broad verification

All commands were run from the repository root on 2026-09-02:

| Command | Outcome |
|---|---|
| `cargo check -p emissary-core` | PASS |
| `cargo test -p emissary-core --no-fail-fast` | PASS — 1,069 tests, 2 ignored |
| `cargo check` | PASS |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | PASS — 33 tests |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | PASS — no issues |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | FAIL due the repository's pre-existing stable/nightly rustfmt configuration/toolchain mismatch; it reports broad formatting churn in untouched files, and no formatter churn was retained |
| M095 matrix counts | Unchanged: `248 apply / 134 blocked_primitive / 458 not_applicable` |

The focused owner tests include exact valid signed-option preservation, malformed and
out-of-range rejection, deterministic bounded inclusive variance, zero-variance default
behavior, and generic mapping of variance/standby fields. The existing tunnel-pool
runtime paths exercise the normal pending, expiry, timer, selector, owner-registration,
and shutdown machinery retained by the implementation.

## Future-plan readiness audit

- **M111 — ready.** M117's accepted Yosemite generic session API and M118's real neutral
  variance/backup runtime effect are both closed. M111 must perform its own execution-time
  semantic re-freeze, especially for `UseSSL`, and may leave unsupported cells blocked.
- **M112 — remains blocked.** Its 69 client proxy/session-lifecycle cells, including the
  seven `NewDest` cells transferred by M116, have an independent lifecycle blocker.
- **M113 — remains blocked.** Its up-to-21 server presentation/address-routing/LeaseSet
  cells require their own security and primitive freeze; this also controls Yosemite Y003.
- **M114 — remains blocked.** It cannot execute until applicable residuals and high/medium
  Proposal-scoped correctives are cleared.

No future plan other than M111 became unblocked. M095 counts remain a capability
baseline and no neutral prerequisite was treated as Proposal support.

## Unresolved findings

1. **Low — tooling:** the repository's committed rustfmt configuration is not accepted
   consistently by the installed stable/nightly formatters. This is pre-existing toolchain
   drift, not an M118 source defect.
2. **Deferred by design — Proposal mapping:** M111 must provide request-to-real-session
   evidence before changing any matrix cells; M118 only supplies the neutral lower-layer
   capability.

No high- or medium-severity M118 security, containment, correctness, or lifecycle finding
remains open.

## Internal-only attestation

External specifications and reference-router sources were inspected read-only. No upstream
repository, issue, pull request, review, maintainer channel, release artifact, or external
branch/tag was mutated or requested. All implementation and planning writes are internal to
`eggstack/emissary`.
