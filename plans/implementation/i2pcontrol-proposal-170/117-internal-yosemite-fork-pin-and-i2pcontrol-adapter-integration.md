# M117 — Internal Yosemite Fork Pin and I2PControl Adapter Integration

Status: **proposed / dependency-blocked**

Class: dependency integration / containment / capability plumbing

Baseline: `464213f0434badeb04dbf80a95a8703530c6a909` (post-M116 closure head)

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Architecture authority:

- `plans/adrs/ADR-0005-internal-yosemite-fork-dependency-boundary.md`
- M061/M062 containment authority;
- M092 historical removal of unauthorized vendor/fork/core expansion;
- M093 tunnel security authority.

Cross-repository dependency:

- `eggstack/yosemite` planning registry;
- Yosemite Y001 closure;
- Yosemite Y002 closure.

## 1. Objective

Adopt the maintainer-authorized `eggstack/yosemite` fork **only for I2PControl production code**, pin it to the exact closed Yosemite revision, and route the existing I2PControl Yosemite calls through that alias without changing the ordinary workspace Yosemite dependency or non-I2PControl startup/tunnel behavior.

Then expose the closed generic Yosemite session-wire/signature-generation capabilities to the existing I2PControl session builder so M111 can execute without raw SAM construction.

M117 itself does not promote Proposal option cells to `apply`; it establishes the accepted dependency/runtime surface and proves containment.

## 2. Hard dependencies/readiness

M117 remains blocked until:

1. Yosemite Y001 is closed with exact commit evidence for bounded `SESSION CREATE` option serialization;
2. Yosemite Y002 is closed with exact commit evidence for signature-aware `DEST GENERATE`;
3. the exact Yosemite commit to pin contains both closures or an explicitly reviewed descendant containing only accepted follow-up corrections;
4. no unresolved high/medium Yosemite serialization/security defect remains.

After those conditions, the Emissary registry must explicitly mark M117 ready before implementation.

## 3. Required production changes

Authorized Emissary production scope after readiness:

- `emissary-cli/Cargo.toml`;
- `Cargo.lock`;
- Yosemite imports/use sites under `emissary-cli/src/i2pcontrol/**` only;
- existing I2PControl session-option/destination-generation adapter code under `emissary-cli/src/i2pcontrol/backends/**` and `client_secret_store.rs` only where needed to consume the generic fork API;
- M062 dependency-containment evidence/manifest and focused tests.

The existing root workspace Yosemite declaration in `Cargo.toml` MUST remain unchanged.

No `emissary-core/**`, `emissary-util/**`, `emissary-cli/src/main.rs`, `emissary-cli/src/tunnel/**`, frontend, workflow, release, or startup production change is authorized.

## 4. Dependency shape

Add one optional direct dependency owned by the `i2pcontrol` feature, using an alias equivalent to:

```toml
yosemite-i2pcontrol = {
  package = "yosemite",
  git = "https://github.com/eggstack/yosemite",
  rev = "<exact closed SHA>",
  features = ["async-extra"],
  optional = true,
}
```

Preserve the current Yosemite runtime feature behavior used by I2PControl; do not accidentally disable Tokio/default features relative to the current call path.

Add `dep:yosemite-i2pcontrol` to the `i2pcontrol` feature.

Do not use `[patch.crates-io]`, a path dependency, a branch-only git reference, a tag-only reference, or a vendored source tree.

## 5. Invariants

M117 MUST preserve:

- ordinary `yosemite = { workspace = true }` dependency and non-I2PControl imports unchanged;
- fork dependency absent from feature-disabled/default dependency reachability where the `i2pcontrol` feature is not selected;
- every fork import contained under `emissary-cli/src/i2pcontrol/**`;
- one SAM implementation per call path; no raw/parallel SAM command construction;
- Proposal-specific validation remains in I2PControl, not Yosemite;
- no option is marked operational merely because Yosemite serializes it;
- shared-session compatibility keys include every newly forwarded session-affecting setting before M111 promotes those cells;
- unsupported signature types fail truthfully; no fallback to 7;
- no secret values enter debug/log/error paths;
- internal-only external interaction.

## 6. Explicit non-goals

M117 MUST NOT:

- replace the workspace Yosemite dependency globally;
- implement tunnel variance/backup behavior in Emissary core (M118 owner);
- implement M111 cell promotion/closure;
- implement M112 lifecycle/proxy options;
- implement M113 LeaseSet behavior;
- implement Proposal `UseSSL` by mapping it to Yosemite's SAM-router `ssl` field without separate semantic proof;
- add signature algorithms to Emissary;
- add dependencies beyond the exact aliased Yosemite package;
- change router algorithms, SAM server, startup tunnel managers, frontend, CI or release behavior;
- contact or submit work upstream.

## 7. Work packages

### WP1 — Freeze Yosemite provenance

Record:

- exact Y001/Y002 closure commits;
- exact selected descendant commit if different;
- diff from Yosemite baseline `d0fe71da...` to selected commit;
- changed paths and dependency changes in Yosemite;
- verification/closure findings.

Reject a pin containing unrelated/unreviewed fork work.

### WP2 — Add optional package alias

Add the exact git/rev dependency under the I2PControl dependency owner and feature activation. Update the lockfile only through Cargo resolution.

### WP3 — Contained import migration

Change only I2PControl use sites from ordinary `yosemite` to the alias. Do not modify non-I2PControl imports to make compilation convenient.

If a shared helper outside I2PControl requires one Yosemite concrete type from both sources, stop rather than widening the alias; introduce an I2PControl-local abstraction if it can remain contained and trivial.

### WP4 — Generic capability adapter

Map existing I2PControl typed configuration into the closed generic Yosemite fields/APIs needed by M111:

- signature type where semantically accepted;
- tunnel variance/backup wire fields;
- bounded additional options for `CustomOptions` only after current I2PControl validation produces safe key/value pairs;
- signature-aware destination generation used by I2PControl-owned generated destination/key paths.

This WP does not assert router-side effect.

### WP5 — Shared-session compatibility

Ensure all forwarded session/security settings that affect session identity/behavior participate in M110/M116 collision-safe compatibility equality. Different effective session-wire settings must not silently share one Yosemite session.

### WP6 — Dependency containment evidence

Update M062 evidence so the new direct dependency is explicitly optional/feature-owned and the lockfile source/revision is expected for I2PControl-enabled builds.

## 8. Failure, cancellation, restart and contention

M117 changes dependency/API selection, not lifecycle ownership. Existing M115/M116 generation, cancellation and shared-session rules remain controlling.

Validation occurs before Yosemite session creation. A fork API error propagates through existing I2PControl start/edit rollback behavior; do not retry through ordinary Yosemite or construct raw SAM as fallback.

No lock may cross Yosemite network I/O.

## 9. Compatibility and migration

No persistent-state migration is required.

Feature-disabled/default builds continue to use ordinary Yosemite only. I2PControl-enabled builds may contain both package instances in the lock/dependency graph; that is an accepted ADR-0005 consequence.

Stored definitions remain subject to existing validation and support disposition.

## 10. Focused tests/evidence

At minimum:

- dependency metadata/tree without `i2pcontrol` does not activate the git-fork alias;
- dependency metadata/tree with `i2pcontrol` activates the exact fork revision;
- ordinary workspace Yosemite remains present from its existing source/version;
- no non-I2PControl production path imports the alias;
- I2PControl session builder reaches fork Y001 serialization in a fake/local SAM command test;
- I2PControl generated destination path reaches Y002 selected signature type;
- unsupported signature type has no fallback;
- differing M111-relevant settings do not share one M110/M116 session;
- no raw SAM command builder appears under I2PControl.

## 11. Broad verification

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Also capture `cargo tree`/`cargo metadata` evidence for both feature-disabled and I2PControl-enabled dependency reachability.

Do not add hosted CI or release verification.

## 12. Acceptance criteria

M117 closes only when:

1. exact authorized Yosemite fork revision is pinned;
2. ordinary workspace Yosemite remains unchanged for non-I2PControl code;
3. the fork is optional and I2PControl-feature-owned;
4. all alias imports are under I2PControl production paths;
5. Y001/Y002 functionality is reachable through real I2PControl code without raw SAM;
6. shared-session compatibility includes forwarded settings;
7. M062 dependency/source containment explicitly accepts the new alias and lockfile delta;
8. default/feature-disabled behavior remains unchanged;
9. closure records whether M111 is still blocked on M118 or other semantic/runtime gates.

## 13. Stop conditions

Stop if:

- the selected Yosemite commit includes unrelated/unreviewed fork work;
- Cargo requires replacing/patching the workspace Yosemite dependency globally;
- a non-I2PControl production path must migrate to the alias;
- raw SAM construction is required;
- fork/default feature differences alter ordinary Emissary behavior;
- a requested option's semantics are unresolved;
- implementation would modify core/router behavior (M118 or later plan required).

## 14. Closure evidence

Require exact Yosemite SHA/diff, Cargo source/provenance evidence, dependency-tree feature isolation, changed-path list, fork API reachability tests, shared-session compatibility evidence, M061/M062 results, broad verification outcomes, unresolved findings, M111 readiness decision, and internal-only attestation.
