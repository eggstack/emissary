# M118 — Neutral SAM Tunnel-Pool Variance and Backup Capability

Status: **ready**

Class: capability / neutral lower-layer exception / tunnel-pool semantics

Baseline: `464213f0434badeb04dbf80a95a8703530c6a909` (post-M116 closure head)

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Architecture authority:

- `plans/adrs/ADR-0005-internal-yosemite-fork-dependency-boundary.md`;
- M061/M062 containment authority;
- M093 tunnel security authority.

Consumer dependency: M111 remains blocked until both this neutral router-side capability and M117's Yosemite client-side adoption are closed.

## 1. Objective

Implement the smallest neutral Emissary SAM/tunnel-pool capability required to honor canonical session options for:

- `inbound.lengthVariance`;
- `outbound.lengthVariance`;
- `inbound.backupQuantity`;
- `outbound.backupQuantity`.

These are generic SAM/I2P tunnel-pool properties, not Proposal 170 APIs. The implementation belongs to Emissary's existing SAM parser/server and tunnel-pool owners.

M118 MUST NOT change I2PControl production code or promote Proposal support-matrix cells by itself.

## 2. Current evidence

At the baseline:

- the SAM parser preserves arbitrary `SESSION CREATE` options but validates only base length/quantity among tunnel-pool settings;
- `SamServer` constructs `TunnelPoolConfig` using only inbound/outbound base quantity and length;
- `TunnelPoolConfig` contains only `num_inbound`, `num_inbound_hops`, `num_outbound`, `num_outbound_hops`, and name;
- no neutral variance/backup fields are currently consumed by the tunnel pool.

Therefore Yosemite-side serialization alone would be accept-inert unless this lower-layer capability exists.

## 3. Authorized production paths

M118 is an explicit lower-layer exception. Production changes are limited to:

- `emissary-core/src/sam/parser.rs` — validation/normalization of the four generic SAM options;
- `emissary-core/src/sam/mod.rs` — transfer into the existing tunnel-pool configuration owner;
- `emissary-core/src/tunnel/pool/mod.rs` — configuration and runtime tunnel-pool semantics.

Tests may be added in the existing module/test locations for those owners.

No other `emissary-core/**` path is pre-authorized. If correct standby/variance semantics require another exact existing tunnel-pool owner, stop and amend/register the plan before changing it.

No `emissary-cli/src/**`, `emissary-util/**`, Cargo/dependency, startup, frontend, transport, NetDb, I2CP, workflow, release, or Yosemite production change is authorized by M118.

## 4. Invariants

M118 MUST preserve:

- no Proposal/I2PControl types or names in `emissary-core`;
- existing tunnel build cryptography/protocol unchanged;
- existing zero-hop support boundary unchanged;
- base quantity/length defaults unchanged when new options are absent;
- variance never produces an unsupported/invalid hop count;
- backup tunnels have reference-correct standby/failover semantics rather than merely inflating active selection quantity if the reference distinguishes them;
- active tunnel selection remains bounded and does not favor/consume standby tunnels prematurely;
- backup promotion/replenishment is generation/pool-local;
- pool shutdown/cancellation drops normal and backup state with no orphan tasks;
- no unbounded build storm when tunnels fail repeatedly;
- no lock across tunnel-build network I/O/sleeps;
- M093 anonymity/resource boundaries remain intact;
- no upstream interaction.

## 5. Explicit non-goals

M118 MUST NOT:

- implement Proposal validation/application policy;
- change Yosemite or Cargo provenance;
- implement `Reduce*`, `Close*`, `NewDest`, `Profile`, `UseSSL`, `SigType`, `CustomOptions`, or LeaseSet options;
- add a general dynamic tunnel-policy framework;
- add new tunnel types or transports;
- broaden zero-hop support merely to make variance arithmetic convenient;
- redesign tunnel selection, profiling, NetDb or peer scoring;
- add dependencies, CI, fuzz, benchmark, release, or upstream work.

## 6. Semantic freeze before code

Before editing production code, record direct I2P/SAM/reference behavior for:

- signed variance interpretation and units;
- how random tunnel length is chosen from base + variance;
- clamping/invalid behavior at supported hop-count boundaries;
- whether negative variance has asymmetric or symmetric semantics;
- whether `backupQuantity` means active-but-not-selected standby tunnels, extra build attempts, or another precise pool state;
- promotion/replenishment behavior when a normal tunnel expires/fails.

Do not guess from Java field names.

If reference semantics require zero-hop behavior Emissary does not already support for the relevant direction, keep that parameter range blocked/rejected rather than adding zero-hop scope.

## 7. Work packages

### WP1 — Parser validation

Add bounded numeric parsing for the four canonical keys. Invalid/overflow/malformed values must reject `SESSION CREATE` rather than fall back to defaults.

Validation should reject configurations whose required resultant hop counts cannot be represented safely within existing Emissary tunnel limits unless the reference defines an exact clamp that can be implemented without new zero-hop behavior.

### WP2 — Extend `TunnelPoolConfig`

Add neutral fields for inbound/outbound variance and backup quantity with defaults preserving current behavior.

Avoid Proposal terminology.

### WP3 — SAM server mapping

Populate the new config fields from the validated session options. Do not duplicate parser policy in the server.

### WP4 — Variance at build selection

For each new tunnel build, compute the reference-correct per-build hop count using the runtime RNG/accepted random source and existing tunnel-pool owner.

The randomized result must be bounded, deterministic under the test runtime where required, and passed to the existing tunnel build path without changing the build protocol.

Do not mutate the base configuration after one random choice.

### WP5 — Backup state

Represent reference-correct standby capacity separately from normal active quantity when required by the reference semantics.

A backup tunnel must not be selected for ordinary traffic while designated standby if the reference treats it as standby. On normal tunnel loss, promote a suitable backup atomically/pool-locally, then replenish standby capacity subject to existing build bounds/rate behavior.

If correct backup semantics cannot be represented without a broad pool redesign or new subsystem, stop and close the backup slice as blocked rather than treating `quantity + backupQuantity` as equivalent without proof.

### WP6 — Lifecycle/failure bounds

Ensure pool shutdown, build failure, expiration and repeated replacement cannot create an unbounded queue or runaway build loop due to backup maintenance.

## 8. Failure, cancellation, restart and contention

All new state is owned by the existing tunnel pool instance. Destroying the pool cancels/forgets normal and backup capacity together.

Build failure does not create persistent configuration mutation. Replacement/replenishment follows existing bounded maintenance cadence unless a smaller exact existing owner is required.

No new global state or cross-pool lock is permitted.

## 9. Compatibility

Absent new options, `TunnelPoolConfig::default()` and SAM-created pool behavior remain unchanged.

Existing `inbound.length`, `outbound.length`, and quantity semantics remain controlling base values.

No storage migration, API version change, or startup configuration change.

## 10. Focused tests

At minimum:

- absent options preserve current config/default behavior;
- valid signed variance parses exactly; malformed/out-of-range values reject;
- deterministic test RNG demonstrates per-build variation within exact allowed bounds;
- base length remains stable while individual build lengths vary;
- unsupported zero-hop/resulting invalid lengths never reach tunnel build;
- backup count is represented separately from normal quantity when reference requires standby;
- standby is not ordinarily selected before promotion;
- normal tunnel expiration/failure promotes backup and replenishes correctly;
- repeated failures stay bounded and do not create build storms;
- pool shutdown clears backup maintenance/state;
- existing quantity/length tests remain green.

## 11. Broad verification

Run focused core SAM/tunnel tests plus:

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

M095/M105 counts should remain unchanged during M118 because no I2PControl cell is promoted by this neutral primitive alone.

## 12. Static containment evidence

M062 exact-path authority must be amended at implementation time to record only the M118 production paths actually changed. No broad `emissary-core/**` authorization.

Add/retain a regression that Proposal/I2PControl identifiers do not appear in the changed core owner.

## 13. Acceptance criteria

M118 closes only when:

1. exact reference semantics are recorded;
2. all implemented variance/backup settings alter real SAM-created tunnel-pool runtime behavior;
3. defaults remain unchanged when settings are absent;
4. invalid/unsupported configurations fail before pool allocation;
5. no zero-hop/protocol/selection scope expansion occurs;
6. backup failure/promotion/replenishment is bounded and reference-correct;
7. changed paths remain within the exact neutral owner;
8. M061/M062/M093 regression evidence passes;
9. closure records whether M111 remains blocked only on M117/Yosemite adoption.

## 14. Stop conditions

Stop a slice if:

- reference semantics are ambiguous enough that behavior would be guessed;
- correct behavior requires a new tunnel type/transport or broader zero-hop implementation;
- backup behavior needs a broad tunnel-manager/selector redesign outside the named owner;
- a dependency or Proposal-shaped core API is proposed;
- changes outside the named production paths become necessary without a registered amendment.

Partial truthfulness is acceptable: a slice may remain `blocked_primitive` rather than implementing an approximation.

## 15. Closure evidence

Require semantic-reference freeze, exact changed paths, focused variance/standby/failover tests, cancellation/resource review, broad verification outcomes, containment/security review, unresolved findings, M111 readiness decision, and internal-only attestation.
