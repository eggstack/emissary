# M124 — Corrected Yosemite Y005 Auth-Consistency Pin Adoption

Status: **closed**

Class: infrastructure / dependency adoption

Repository: `eggstack/emissary`

Planning baseline: `045d1e8b4eba1141d2488882f99c5ce994db91a8`

Closure: `plans/closure/i2pcontrol-proposal-170/124-closure.md`

Implementation commit: `8a302b0`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Dependencies:

- **hard:** M123 closure with no open high/medium server-transaction finding;
- **hard:** Yosemite Y005 closure with an exact reviewed implementation SHA suitable for consumer pinning;
- **interface:** ADR-0005 exact optional I2PControl-only Yosemite dependency boundary remains accepted.

Current dependency baseline:

- ordinary workspace Yosemite remains registry `0.7.0`;
- optional `yosemite-i2pcontrol` alias is exact-pinned to Y005 implementation `59140a2277bf296928d2e8ce39a148182eeff044`;
- no Proposal LeaseSet client-auth mapping is currently active.

Pinned Proposal 170 revision: `2026-05-20` (Open).

## 1. Objective

After Yosemite Y005 closes, advance **only** the optional I2PControl-owned Yosemite alias from the exact Y004 revision to the exact reviewed Y005 implementation revision, and prove that Y005's cross-field LeaseSet auth validation is reachable from the I2PControl dependency boundary without changing Proposal support state.

M124 is transport/dependency hygiene only. It does not implement `EncryptLeaseSet`, `LeaseSetClientAuths`, encrypted LeaseSet construction, client authorization, NetDb behavior, or any M113 residual.

The authoritative M095 matrix must remain `284 apply / 98 blocked_primitive / 458 not_applicable`.

## 2. Why a separate consumer plan is required

Y005 will tighten Yosemite's generic typed LeaseSet API so auth mode, numbered DH/PSK entries and applicable LeaseSet type cannot form combinations that the reference would ignore.

Even though current Emissary production does not map those Proposal options, ADR-0005 requires every fork revision change to be independently reviewed and exact-pinned. M124 prevents a future M113 successor from accidentally building against the known Y004 inconsistency or a floating fork head.

## 3. Readiness gates

Do not start M124 until all are true:

1. Y005 closure names an exact implementation commit and states whether it is suitable for Emissary pinning;
2. Y005 has no unresolved high/medium protocol/security finding in the option/serialization surface Emissary may consume;
3. Y005 production diff is Yosemite-generic and confined to its authorized files;
4. M123 is closed so this dependency-only milestone does not overlap an active server transaction corrective;
5. current Emissary still uses the ADR-0005 optional package alias and has not globally patched/replaced Yosemite.

If any gate fails, M124 remains blocked.

## 4. Invariants

M124 MUST preserve:

- ordinary/non-I2PControl Yosemite provenance unchanged;
- exact `git + rev` pin, never branch/tag/floating HEAD;
- fork dependency optional and activated only by `i2pcontrol`;
- all `yosemite_i2pcontrol` production imports remain below `emissary-cli/src/i2pcontrol/**`;
- no `[patch.crates-io]`, path dependency, vendoring or workspace-wide replacement;
- no Proposal support promotion based only on dependency capability;
- no raw SAM command construction in Emissary;
- no secret material in adapter tests/logs/errors;
- no core/router/frontend/startup production changes;
- Y003/Y004 historical closure records remain unchanged.

## 5. Authorized production/dependency scope

Allowed changes:

- `emissary-cli/Cargo.toml` — revision only for existing `yosemite-i2pcontrol` alias;
- `Cargo.lock` — source/revision hash update caused by that exact pin;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` — exact fork revision evidence;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` **tests/helpers only** if needed to prove Y005 rejection/reachability through the real dependency;
- `emissary-cli/tests/m062_dependency_containment.rs` — exact revision/allowed path evidence;
- M124 plan/closure, registry, roadmap, implementation README and dependency documentation.

No other production path is authorized.

Specifically forbidden:

- Proposal option mapping for LeaseSet auth;
- `emissary-core/**` or router crypto;
- server/client runtime changes;
- default workspace Yosemite revision/source changes;
- new dependency or feature;
- upstream activity.

## 6. Work packages

### WP1 — review exact Y005 diff

Compare the currently pinned Y004 implementation commit against the exact Y005 implementation commit.

Record:

- production files changed;
- public API differences;
- default behavior differences;
- validation/serialization differences;
- dependency/features unchanged;
- absence of Emissary/Proposal concepts;
- unresolved Y005 closure findings.

Stop if Y005 contains unrelated expansion or cannot be reviewed as a bounded generic dependency update.

### WP2 — advance exact alias revision

Change only the `rev` on the existing `yosemite-i2pcontrol` alias and the corresponding lockfile source.

Verify the ordinary registry Yosemite instance remains unchanged.

### WP3 — dependency-boundary regression

Using the actual `yosemite_i2pcontrol` package alias, add or update focused tests proving representative Y005 behavior:

- a mode-coherent DH configuration can be constructed/serialized by the dependency;
- a mode-coherent PSK configuration can be constructed/serialized;
- at least one representative mismatched/mixed auth configuration fails before fake-SAM connection/command bytes;
- no Y003/Y004 historical non-canonical namespace reappears;
- secret-bearing fixtures remain redacted.

These tests are dependency evidence only. Do not add a Proposal `TunnelOptions` mapping to reach them.

### WP4 — containment/provenance

Prove:

```text
cargo tree -p emissary-cli --no-default-features --edges normal
```

contains only registry Yosemite, while:

```text
cargo tree -p emissary-cli --no-default-features --features i2pcontrol --edges normal
```

contains the registry instance plus exactly the Y005 fork revision.

Enumerate all `yosemite_i2pcontrol` import paths and prove they remain I2PControl-owned.

### WP5 — planning/docs reconciliation

Update current dependency evidence to Y005 while retaining:

- matrix `284 / 98 / 458`;
- M113's 21 cells blocked;
- no claim that Emissary can construct encrypted/authenticated LeaseSets;
- future LeaseSet capability/crypto-ownership audit is authorized after M124 closes, but no implementation plan is registered until the audit freezes a safe owner.

## 7. Failure, cancellation, restart and migration semantics

M124 introduces no persistent-data migration, task, runtime owner or lifecycle change.

If the new dependency fails to compile or an API incompatibility appears, stop rather than modifying unrelated Emissary production code.

Rollback is the exact Y004 revision pin plus corresponding lockfile source; there is no data-format migration.

## 8. Focused tests

Required evidence:

1. exact manifest revision equals Y005 implementation SHA;
2. lockfile fork source equals the same SHA;
3. feature-disabled dependency tree has no eggstack Yosemite fork;
4. I2PControl-enabled tree has exactly one optional fork instance at Y005;
5. all fork import sites remain under I2PControl production/tests;
6. representative Y005 valid DH/PSK path reaches fake SAM or controller bytes through the dependency;
7. representative mismatched auth mode fails before wire;
8. secret/debug redaction remains intact;
9. M095/M105 tests remain unchanged/green.

## 9. Broad verification

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo tree -p emissary-cli --no-default-features --edges normal
cargo tree -p emissary-cli --no-default-features --features i2pcontrol --edges normal
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 10. Acceptance criteria

M124 may close only when:

1. exact reviewed Y005 revision is pinned only through the optional I2PControl alias;
2. ordinary Yosemite dependency/provenance is unchanged;
3. Y005 cross-field validation is demonstrably reachable from the dependency boundary;
4. no Proposal LeaseSet mapping/runtime behavior is added;
5. matrix remains `284 / 98 / 458`;
6. containment and dependency-tree guards pass;
7. no high/medium M124 finding remains open;
8. closure explicitly decides whether the focused M113/LeaseSet capability audit may proceed.

## 11. Stop conditions

Stop if:

- Y005 closure is not suitable for pinning;
- Y005 introduces unresolved high/medium issues;
- adoption requires changing non-I2PControl Yosemite consumers;
- adoption requires a global patch/path/vendor strategy;
- production adaptation requires core/router changes;
- an implementation agent begins M113/LeaseSet feature work here;
- upstream interaction is proposed.

## 12. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/124-closure.md` with:

- exact Y004→Y005 diff review;
- exact pin/lock evidence;
- dependency-tree provenance;
- adapter/rejection/redaction tests;
- changed-path/containment audit;
- broad verification outcomes;
- matrix unchanged evidence;
- unresolved findings/severity;
- decision on next LeaseSet audit readiness;
- internal-only external-interaction attestation.
