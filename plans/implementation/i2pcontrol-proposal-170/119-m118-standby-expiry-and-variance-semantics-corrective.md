# M119 — M118 Standby Expiry and Variance Semantics Corrective

Status: **closed**

Class: corrective / neutral router capability / tunnel-pool correctness

Baseline: `feafc6a1d9650887015a01f87bf21b57a4e92085`

Corrects:

- `plans/implementation/i2pcontrol-proposal-170/118-neutral-sam-tunnel-pool-variance-backup-capability.md`;
- `plans/closure/i2pcontrol-proposal-170/118-closure.md`.

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`.

Applicable architecture/security authority:

- ADR-0004 pinned Proposal 170 completion boundary;
- ADR-0005 internal Yosemite fork dependency boundary;
- M061/M062 containment authority;
- M093 tunnel security regression authority.

## 1. Objective

Correct two neutral tunnel-pool semantics introduced by M118 without adding Proposal-shaped APIs or broadening Emissary router scope:

1. a promoted inbound standby tunnel must retain and advertise its **actual remaining tunnel lifetime**, rather than being republished with a fresh full lifetime while its existing destruction timer continues counting down;
2. negative tunnel-length variance must match the independently frozen I2P reference selection semantics, or the closure must explicitly prove that a different distribution is contract-equivalent before retaining the current implementation.

M119 is infrastructure correctness only. It does not promote or demote Proposal 170 matrix cells by itself.

## 2. Defects and why M118 verification missed them

### F1 — promoted standby inbound lease expiration can exceed real tunnel lifetime

M118 builds standby inbound tunnels immediately and registers them with the existing tunnel timer. When an active inbound tunnel later expires, a standby may be promoted into the active pool.

Current promotion constructs a new owner-visible `Lease` with expiration `now + TUNNEL_EXPIRATION`, even though the standby tunnel's original timer is not reset. A standby that has already aged can therefore be advertised as usable beyond the time at which its actual tunnel will be destroyed.

M118 tests established separation/promotion/accounting but did not assert that owner-visible lease expiration remains coupled to the promoted tunnel's original absolute expiration.

### F2 — negative length-variance probability differs from Java reference

M118's helper samples negative variance uniformly over the complete integer offset interval. The Java tunnel selector instead samples a magnitude and a sign, which gives the base value different probability mass for negative variance.

Proposal 170 exposes the variance value rather than a probability distribution, so this may be a compatibility-quality rather than externally visible safety failure. However M118 closure claims exact reference semantics. Yanking that statement without re-audit is not acceptable; either match the reference or record affirmative evidence that the distribution is outside the pinned contract.

M118 tests checked bounds and that more than one value is reachable, but not deterministic distribution semantics against a seeded/reference vector.

## 3. Scope and production ownership

Authorized production path:

- `emissary-core/src/tunnel/pool/mod.rs`.

The implementation MAY update existing tunnel-pool tests in that module and the exact M061/M062 containment metadata/tests necessary to authorize this already-neutral corrective.

No other `emissary-core/**` production path is authorized without a recorded stop/deviation.

No `emissary-cli/src/i2pcontrol/**`, `emissary-util/**`, Cargo/dependency, SAM parser, NetDb, transport, frontend, workflow, release, or Yosemite production change is authorized.

## 4. Required behavior

### 4.1 Standby lifetime ownership

Every built inbound standby that may later be promoted must retain enough owner-local metadata to recover the tunnel's real absolute expiration when promoted.

Acceptable implementation shapes include carrying an absolute expiration beside the standby entry or deriving it from one existing canonical timer owner if that owner exposes an exact non-racy value. Do not create a second independent lifetime clock that can drift from the destruction timer.

On promotion:

- the tunnel's destruction/rebuild timer remains based on its original construction lifetime;
- the newly registered active `Lease` uses the same absolute expiration or a conservatively earlier one;
- promotion MUST NOT extend the real or advertised lifetime;
- a standby that is already too close to/at expiration to be useful must not be promoted as a fresh lease;
- owner registration failure must preserve internally consistent active/standby/routing state and must not fabricate a usable lease.

Outbound standbys do not publish Lease expiration, but their timer/promotion behavior must remain unchanged and covered by regression tests so the inbound fix does not accidentally reset or duplicate outbound timers.

### 4.2 Negative variance semantics

Re-read the pinned I2P tunnel pool/reference selector and freeze the exact length-selection rule for positive, zero, and negative variance.

If Java-reference parity is the accepted contract, implement the same random-choice semantics with the existing runtime RNG and no new dependency. Tests must use a deterministic/mock RNG or an exact helper-level reference vector; statistical/flaky assertions are insufficient.

If the implementation concludes that Proposal 170 only requires the representable length set and not the distribution, closure must cite affirmative pinned/reference evidence and downgrade the M118 'exact reference semantics' claim rather than silently retaining it. The default preferred outcome is to match the reference because the owner is already being touched and the change is small.

Boundary behavior must remain fail-closed before tunnel builds when the configured base/variance cannot produce only supported non-zero-hop lengths.

## 5. Invariants and non-goals

M119 MUST preserve:

- M093 tunnel cryptography, peer-selection safety, and no-zero-hop boundaries;
- M118 active vs standby separation;
- standby tunnels are not published/selected/owner-registered before promotion;
- bounded active + standby build accounting and pending markers;
- no lock held across router/network I/O;
- generation/pool-local ownership and shutdown cleanup;
- existing absent-option/default behavior;
- no Proposal/I2PControl types or names in core;
- no new dependency;
- no upstream activity.

M119 does not:

- change `TunnelBackupQuantity` or `TunnelVariance` Proposal applicability/matrix status;
- redesign the tunnel timer system;
- change tunnel lifetime constants;
- add zero-hop support;
- alter SAM option spelling/validation;
- implement Reduce/Close lifecycle behavior;
- perform M114 interoperability reclosure.

## 6. Work packages

### WP1 — freeze actual lifetime data flow

Trace inbound tunnel construction → standby insertion → timer registration → promotion → owner Lease registration → destruction. Record the one canonical absolute expiration or derive a safe exact representation.

### WP2 — couple standby metadata to timer lifetime

Add only the minimal standby metadata required to publish the true remaining lifetime. Ensure cleanup removes it on build failure, standby expiry, promotion, active expiry, and pool shutdown.

### WP3 — promotion transaction tests

Test promotion of a deliberately aged standby and assert the promoted Lease expiration does not exceed the original tunnel expiration. Cover registration failure and standby-expired-before-promotion behavior.

### WP4 — variance reference parity

Freeze and implement/test the accepted negative-variance choice rule without changing supported base-length bounds.

### WP5 — containment/closure

Update exact containment metadata/tests and write a new M119 closure. Do not rewrite M118 history.

## 7. Failure, cancellation, restart, contention

Tunnel-pool polling remains the sole mutable owner. No new shared lock or background worker is permitted.

Build failures must clear pending standby markers exactly as before. Promotion must be atomic with respect to pool-local maps/selectors: no route may be visible as active with an owner Lease that claims a later expiration than the underlying tunnel.

Pool shutdown removes both active and standby routes/timers/metadata. No lifetime metadata may survive pool destruction.

## 8. Compatibility

Absent variance/backups preserve existing defaults. Correcting promoted lease expiration changes only defective M118 standby behavior.

If negative-variance random distribution changes to match the reference, the reachable set remains bounded by the same configured values; only selection probability changes.

No public JSON-RPC, SAM, configuration, dependency, or storage migration is in scope.

## 9. Focused tests

At minimum:

- standby inbound stores/derives its original absolute expiry;
- promoting an aged standby publishes an expiry equal to or earlier than the original expiry, never `promotion_time + full_lifetime`;
- promoted standby still expires/destroys on its original timer;
- standby that expires before promotion is removed and not advertised;
- failed owner registration cannot leave a fabricated lease or double-active accounting;
- outbound standby promotion retains existing timer behavior;
- deterministic positive/zero/negative variance vectors match the frozen reference rule;
- invalid base/variance range still fails before build selection;
- active/standby build-count saturation and shutdown regressions remain green.

## 10. Broad verification

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

The known stable/nightly rustfmt drift may be dispositioned, but no new M119-specific formatting or lint failure is acceptable.

## 11. Matrix and documentation rules

M119 is a neutral prerequisite corrective. M095 must remain `312 apply / 70 blocked_primitive / 458 not_applicable` unless an independent semantic finding proves an M118-applied Proposal cell itself must be demoted. If that occurs, stop and record the required matrix-corrective successor rather than silently changing counts inside this core-only pass.

Update M119 closure, registry, corrective roadmap, and containment evidence. M118 closure remains historical and is referenced as corrected by M119.

## 12. Acceptance criteria

M119 closes only when:

1. promoted inbound standby Lease expiration can never outlive the underlying tunnel;
2. promotion/shutdown/failure paths do not leak or duplicate timer/route/owner state;
3. negative variance behavior has an explicit independently tested reference disposition;
4. active/standby bounds and M093 invariants remain intact;
5. no Proposal-specific API enters core;
6. focused and broad verification pass or baseline-only tooling failures are explicitly dispositioned;
7. closure states whether M120 may be promoted.

## 13. Stop conditions

Stop rather than broaden scope if:

- the fix requires a router-global timer redesign;
- exact expiry cannot be preserved without changing unrelated tunnel owners;
- a new dependency is proposed;
- zero-hop behavior must be altered;
- a Proposal matrix change becomes necessary for reasons outside the two defects above.

## 14. External-interaction boundary

External I2P/reference sources are read-only evidence. Writes are internal to `eggstack/emissary` only. No upstream issue, PR, review, release, submission, merge/adoption request, contribution package, or maintainer contact is authorized.

## 15. Closure evidence required

Record changed paths, exact lifetime data flow, deterministic variance reference evidence, focused/broad test outcomes, M061/M062/M093 invariant review, matrix non-change (or stop disposition), unresolved findings, implementation SHA, and M120 readiness.

## 16. Closure disposition

M119 is closed by implementation commit `282c059dc15c727aeceba1acb50aa7d82c5fc087` and
closure record `plans/closure/i2pcontrol-proposal-170/119-closure.md`.

The lifetime data flow retains the canonical absolute expiration beside each
inbound standby and reuses it verbatim on promotion; the destruction timer is
never reset and registration failure rolls back to pre-promotion accounting.
Negative variance implements the frozen Java `TunnelPeerSelector.getLength`
magnitude/sign rule with deterministic reference vectors. M095 remains
`312 apply / 70 blocked_primitive / 458 not_applicable` and M120 is ready.