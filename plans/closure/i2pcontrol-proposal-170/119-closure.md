# M119 Closure — M118 Standby Expiry and Variance Semantics Corrective

Status: **closed**

Plan: `plans/implementation/i2pcontrol-proposal-170/119-m118-standby-expiry-and-variance-semantics-corrective.md`

Implementation commit: `282c059dc15c727aeceba1acb50aa7d82c5fc087`

Closure date: 2026-09-03

## Disposition

M119 is closed. The two neutral tunnel-pool defects introduced by M118 are
corrected within the already-authorized `emissary-core/src/tunnel/pool/mod.rs`
owner plus its exact containment test seam. Promoted inbound standbys reuse
their canonical absolute expiration verbatim and never mint a fresh full
lifetime; negative length variance matches the frozen Java reference
magnitude/sign semantics. No Proposal 170 matrix cell is promoted or demoted
and M095 remains `312 apply / 70 blocked_primitive / 458 not_applicable`.

M118 closure remains historical and is referenced as corrected by M119. No
M118 history was rewritten.

## Semantic reference freeze

The following sources were inspected read-only before implementation:

- `TunnelPoolSettings.java` (`getLengthVariance` doc: negative variance skews
  `(length - variance)..=(length + variance)`, positive skews
  `length..=(length + variance)`, inclusive);
- `TunnelPeerSelector.java` `getLength(TunnelPoolSettings)` (pinned logic):

```java
int length = settings.getLength();
int override = settings.getLengthOverride();
if (override >= 0) {
    length = override;
} else if (settings.getLengthVariance() != 0) {
    int skew = settings.getLengthVariance();
    if (skew > 0)
        length += ctx.random().nextInt(skew+1);
    else {
        skew = 1 - skew;
        int off = ctx.random().nextInt(skew);
        if (ctx.random().nextBoolean())
            length += off;
        else
            length -= off;
    }
}
if (length < 0)
    length = 0;
else if (length > 7)
    length = 7;
```

Frozen behavior:

- `variance == 0` preserves the base length without consuming RNG;
- positive variance samples an inclusive additive offset `0..=variance`;
- negative variance samples a magnitude uniformly from `0..=|variance|` and
  then a sign uniformly (`nextBoolean`); magnitude zero yields the base
  regardless of sign, so the base carries `1/(M+1)` mass and each non-zero
  offset carries `1/(2*(M+1))` for `M = |variance|`;
- Emissary retains its existing fail-closed boundary instead of Java clamping:
  inbound `1..7` and outbound `1..8` remain enforced; SAM parser rejects
  configurations whose complete possible range crosses those boundaries and
  per-build selection returns `None` (build skipped) if a sampled result
  cannot be represented; zero-hop behavior was not broadened.

The default preferred outcome from the plan (match the reference) was taken;
no downgrade of the M118 exact-semantics claim was needed.

## Lifetime data flow freeze (WP1)

- Construction: `PendingTunnel::try_build_tunnel` → `TunnelBuilder::build` →
  `InboundTunnel::new` starts its `TUNNEL_EXPIRATION` (10-minute) delay at
  build completion, on the same poll tick that the pool processes the build
  success.
- Standby insertion: pool `poll` inserts the gateway into
  `backup_inbound_tunnels` with `expires = time_since_epoch() +
  TUNNEL_EXPIRATION` on that same tick and registers
  `tunnel_timers.add_inbound_tunnel` (Rebuild at 9 minutes, skipped for
  standbys). The stored `Duration` is not a second ticking clock.
- Timer: standby Rebuild events are skipped by backup-membership check;
  destruction arrives via the tunnel's own `inbound` JoinSet event after
  10 minutes.
- Promotion: on active expiry, `promote_standby_inbound` selects the latest
  future-expiry standby, moves it to `inbound_tunnels`/selector, and registers
  `Lease { expires: backup_expires }` verbatim. No new timer is registered.
- Destruction: the promoted tunnel still expires via its original JoinSet
  event; shutdown removes both active and standby routing state.

## Requirement-to-evidence matrix

| Requirement | Evidence and outcome |
|---|---|
| Standby lifetime ownership (§4.1) | `backup_inbound_tunnels` is `HashMap<TunnelId, (TunnelId, RouterId, HashSet<RouterId>, Duration)>`; insertion captures `now + TUNNEL_EXPIRATION` on the build-success tick; promotion reuses it verbatim. |
| No lifetime extension | `promote_standby_inbound` never computes `now + TUNNEL_EXPIRATION` for standbys; aged-standby test asserts `lease.expires == original (now+60s) < now+600s`. |
| Expired standby not promoted | `select_promotable_inbound_standby` filters `expires > now`; expired entries are left for timer cleanup to avoid misclassifying their later JoinSet event; test asserts no promotion and preserved counts. |
| Registration-failure atomicity | Promotion inserts active/selector tentatively, then on `register_inbound_tunnel_built` error removes both and restores the standby entry with its expiry; no publish/gauge occurs on failure; test drops the owner channel and asserts restored accounting. |
| Outbound timer behavior unchanged | No outbound production path touched; outbound backup map type and Destroy/Rebuild handling are byte-identical except the M062 helper; outbound accounting test retained. |
| Negative variance parity (§4.2) | `apply_length_variance` implements magnitude/sign mapping; `varied_tunnel_length` samples magnitude plus one sign draw for negative variance and magnitude only for positive; deterministic vectors cover positive/zero/negative/boundary/invalid cases. |
| Fail-closed bounds | Parser validation unchanged; helper returns `None` for unrepresentable base/result; `maintain_pool` skips the build on `None`; tests assert `None` for base 0/8 and out-of-range offsets. |
| Containment | M061 manifest unchanged (production path already authorized); M062 helper `is_authorized_m119_path` records only the neutral owner, its test seam, the new closure, the 119/120 plan status transitions, registry, and corrective roadmap. No Proposal/I2PControl API in core; no new dependency. |
| Security | M093 cryptography/peer-selection/no-zero-hop boundaries untouched; no lock across I/O added; pool-local ownership and shutdown cleanup preserved. |

## Changed paths

Implementation commit changes:

- `emissary-core/src/tunnel/pool/mod.rs` — standby expiry storage, promotion transaction with rollback, magnitude/sign variance plus deterministic helper, extracted promotable-selection helper, six focused tests;
- `emissary-cli/tests/m062_dependency_containment.rs` — `is_authorized_m119_path` and wiring into both budget asserts.

Closure commit changes only planning evidence:

- this record;
- `plans/implementation/i2pcontrol-proposal-170/119-m118-standby-expiry-and-variance-semantics-corrective.md` status/disposition;
- `plans/implementation/i2pcontrol-proposal-170/120-server-start-preallocation-validation-and-secret-transactionality-corrective.md` promotion to ready;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`.

No `emissary-cli/src/i2pcontrol/**`, `emissary-util/**`, Cargo/dependency, SAM parser, NetDb, transport, frontend, workflow, release, or Yosemite production change.

## Focused and broad verification

All commands were run from the repository root:

| Command | Outcome |
|---|---|
| `cargo check -p emissary-core` | PASS |
| `cargo test -p emissary-core --no-fail-fast` | PASS — 1,075 tests, 2 ignored (1,069 baseline + 6 new M119 tests) |
| `cargo check` | PASS |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | PASS — 33 tests |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | PASS — no issues |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | FAIL due the repository's pre-existing stable/nightly rustfmt configuration/toolchain mismatch; it reports broad formatting churn in untouched files, and no formatter churn was retained |
| M095 matrix counts | Unchanged: `312 apply / 70 blocked_primitive / 458 not_applicable` |

Focused owner tests:

- `varied_tunnel_length_uses_bounded_inclusive_variance` (retained statistical bounds check);
- `apply_length_variance_matches_java_reference_vectors` (deterministic positive/zero/negative/boundary/invalid vectors);
- `standby_selection_prefers_latest_and_rejects_expired`;
- `promoted_aged_standby_reuses_original_expiry` (aged 60s standby promotes with 60s, never fresh 600s);
- `expired_standby_is_not_promoted`;
- `failed_owner_registration_restores_standby_accounting`;
- `outbound_standby_accounting_retains_existing_timer_shape`;
- existing quantity/length/default/mapping tests remain green.

## Failure, cancellation, contention review

- Build failures clear pending standby markers exactly as before; no backup entry is created on failure.
- Promotion is pool-local and atomic with respect to active/standby/selector maps: no route is visible as active with an owner Lease the owner never received.
- Expired standbys are not pre-removed on the promotion path, preserving single-owner JoinSet cleanup and avoiding misclassification as active expiry.
- Pool shutdown removes both active and standby routes/timers/metadata; no lifetime metadata survives destruction.
- No new shared lock or background worker; tunnel-pool polling remains the sole mutable owner.

## Compatibility, migration, security review

- Absent variance/backups preserve existing defaults; only defective M118 standby behavior changes.
- Reachable length set is unchanged; only negative-variance selection probability changes to reference parity.
- No public JSON-RPC, SAM, configuration, dependency, or storage migration.
- M061/M062 guards pass; no Proposal term leaks into core; no new dependency; M093 invariants intact.

## Future-plan readiness audit

- **M120 — ready.** M119 closure satisfies its hard gate. Its I2PControl-only server preallocation/secret transaction work may now be handed off.
- **M121 — remains blocked.** Still gated on M120 closure.
- **M122 — remains blocked.** Still gated on M121 and Yosemite Y004 closure.
- **Y004 — separately ready in Yosemite.** No Emissary action; Y003 remains unconsumed.
- No future plan other than M120 became unblocked. M095 counts remain a capability baseline and no neutral prerequisite was treated as Proposal support.

## Unresolved findings

1. **Low — tooling:** the repository's committed rustfmt configuration is not accepted consistently by the installed stable/nightly formatters. This is pre-existing toolchain drift, not an M119 source defect.
2. **None — high/medium.** No open M119 security, containment, correctness, or lifecycle finding remains.

## Internal-only attestation

External specifications and reference-router sources were inspected read-only. No upstream repository, issue, pull request, review, maintainer channel, release artifact, or external branch/tag was mutated or requested. All implementation and planning writes are internal to `eggstack/emissary`.
