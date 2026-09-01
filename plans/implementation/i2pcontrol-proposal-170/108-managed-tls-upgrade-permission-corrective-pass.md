# M108 — Managed TLS Upgrade-Permission Corrective Pass

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/108-closure.md`

Class: corrective capability / security

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Corrective authority and predecessor evidence:

- M107 plan: `plans/implementation/i2pcontrol-proposal-170/107-i2pcontrol-conformance-and-managed-tls-corrective-pass.md`
- M107 closure: `plans/closure/i2pcontrol-proposal-170/107-closure.md`

Repository baseline:

- `a108b1b62f3ad9d79fe455ccf3910f96d7a5e06f` — `plans(i2pcontrol): close M107 corrective pass`
- Implementation head: `0a5e8c9` — `fix(i2pcontrol): repair managed TLS upgrade permissions`

Pinned Proposal 170 authority:

- I2P Proposal 170, `I2PControl Expansion`, status `Open`, revision `2026-05-20`;
- <https://i2p.net/en/proposals/170-i2pcontrol-expansion/>.

All external sources are read-only evidence. This plan authorizes writes only to `eggstack/emissary` and does not authorize upstream interaction.

## 1. Objective

Close the remaining upgrade-path confidentiality gap in M107 managed I2PControl TLS handling without reopening unrelated Proposal 170 work.

M107 correctly hardened freshly generated managed material, but a router upgrading from the pre-M107 implementation may already have:

- an `i2pcontrol-certs/` directory created with permissions broader than `0700`; and/or
- a regular managed `key.pem` created with permissions broader than `0600`.

The current M107 loader accepts those existing regular paths and reuses them without tightening their mode. In addition, regeneration creates the temporary key file before the later `set_permissions(0600)` call, so the temporary file initially receives ordinary create/umask semantics.

M108 must make managed-key confidentiality true for both fresh installs and existing managed material on Unix:

1. tighten/revalidate an existing Emissary-managed TLS directory before child material is read;
2. tighten/revalidate an existing Emissary-managed private key before it is read;
3. create a temporary managed private-key file with owner-only permissions from the instant it is created, rather than relying on a post-write chmod;
4. fail I2PControl startup if the managed owner cannot establish and verify those permissions safely;
5. reconcile stale planning-control text left after M107 closure.

This is not a TunnelManager milestone. The M095 support inventory must remain exactly `224 apply / 158 blocked_primitive / 458 not_applicable`.

## 2. Why M107 verification missed this defect

M107 added tests proving that a newly created managed directory is `0700` and a newly generated managed private key ends at `0600`. Those tests start from an empty temporary directory.

They do not construct a realistic upgrade fixture containing an already-existing regular managed directory/key with permissive Unix modes. The loader therefore remained able to reuse legacy material without repairing it.

The M107 publication test also inspects the final mode only. It cannot detect that the temporary private-key inode is created first and restricted only after key bytes have already been written.

M108 regression evidence must therefore start from pre-existing permissive managed objects and must make the create-time mode part of the implementation contract rather than only the final state.

## 3. Readiness and current evidence

M108 is dependency-ready.

The required owner already exists entirely in:

- `emissary-cli/src/i2pcontrol/tls.rs`.

Current evidence at the M108 baseline:

- managed directory/file type validation already uses `symlink_metadata`;
- managed symlink and non-regular-file cases already fail closed;
- explicit operator certificate/key paths are a separate code path and must remain untouched;
- managed material is generated and loaded synchronously during I2PControl initialization, before the HTTPS listener begins serving requests;
- Unix-specific permission handling already uses `std::os::unix::fs::PermissionsExt`;
- no new crate is required to set create-time Unix modes because `std::os::unix::fs::OpenOptionsExt` is available under `cfg(unix)`;
- M107 already proves fresh SAN coverage for `localhost`, `127.0.0.1`, and `::1`; M108 does not need certificate parsing or a new X.509 dependency.

No Yosemite/SAM primitive, router-core owner, manifest change, lockfile change, schema migration, or frontend state is required.

## 4. Invariants

The implementation MUST preserve all of the following:

- Proposal 170 remains pinned to `2026-05-20`;
- the TunnelManager matrix remains `224 apply / 158 blocked_primitive / 458 not_applicable`;
- production changes remain under `emissary-cli/src/i2pcontrol/**`;
- explicit operator-supplied TLS certificate/key paths remain operator-owned and are never chmodded, rewritten, relocated, regenerated, or otherwise managed by M108;
- only automatically managed material under `<base_path>/i2pcontrol-certs/` is eligible for permission repair;
- symlink and non-regular managed paths continue to fail closed before material is read or replaced;
- managed private key bytes must never be logged, included in errors, or exposed through debug output;
- I2PControl initialization must fail if required managed permission repair, revalidation, publication, sync, or load fails;
- no plaintext fallback is introduced;
- valid managed material remains stable across restart; permission repair must not rotate a valid key solely because its prior mode was permissive;
- current managed certificate SAN behavior from M107 remains unchanged;
- non-Unix behavior remains portable and does not acquire a new ACL abstraction;
- existing request/auth/concurrency limits and all non-TLS I2PControl behavior remain unchanged;
- feature-disabled/default Emissary behavior remains unchanged.

## 5. Explicit non-goals

M108 MUST NOT:

- implement or reclassify any of the 158 TunnelManager residual cells;
- modify the M095 support matrix or M105 residual audit;
- reopen API-version, AddressBook, RouterInfo, ClientServicesInfo, tunnel runtime, or LeaseSet behavior;
- implement unrelated base I2PControl methods;
- invent token-expiration policy;
- relax AddressBook path confinement;
- add certificate-version metadata or a managed-certificate migration subsystem;
- add certificate parsing solely to detect whether pre-M107 certificates contain loopback IP SANs;
- rotate otherwise valid managed certificate/key material merely to obtain new permissions;
- change explicit remote-management certificate policy;
- add `libc`, `nix`, an X.509 crate, or any other Cargo dependency;
- modify `Cargo.toml`, `Cargo.lock`, Yosemite, SAM, `emissary-core`, `emissary-util`, frontend/UI code, workflows, or release automation;
- fix unrelated workspace rustfmt churn or GitHub Pages configuration;
- prepare or request upstream review, merge, adoption, submission, release, or maintainer contact.

## 6. Required production changes

Expected production path:

- `emissary-cli/src/i2pcontrol/tls.rs`.

Tests may remain colocated in `tls.rs` and/or use existing `emissary-cli/tests/**` integration/containment suites.

Planning/documentation changes may touch:

- `plans/registry.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- the eventual M108 closure record;
- `docs/i2pcontrol/**` only if user-facing managed-permission semantics need clarification.

If implementation requires any production change outside `emissary-cli/src/i2pcontrol/**`, stop and return to planning rather than widening containment.

## 7. Work packages

### WP1 — Repair existing managed directory permissions before child access

On Unix:

1. Preserve the existing `symlink_metadata` type check for `i2pcontrol-certs`.
2. If the managed directory already exists and is a real directory, establish owner-only mode `0700` before any managed certificate/key child is read.
3. Treat this as repair of Emissary-owned state, not operator-explicit state.
4. Immediately re-read metadata after the permission change and verify that the path is still a non-symlink directory and that group/other permission bits are absent.
5. If mode repair or revalidation fails, return a TLS initialization error and do not continue to child material.
6. A newly created managed directory must continue to be `0700`.

Do not add ownership-changing behavior (`chown`) or try to infer arbitrary operator ACL policy.

### WP2 — Repair existing managed private-key permissions before reading

On Unix:

1. After the existing managed key path has been established as a regular non-symlink file, set it to mode `0600` before `load_key()` can read its bytes.
2. Re-read metadata and verify it remains a regular non-symlink file with no group/other permission bits.
3. If repair or revalidation fails, fail I2PControl initialization.
4. Do not rotate or rewrite a valid key merely because its prior mode was permissive.
5. Do not apply this chmod behavior to explicit operator-provided key paths.

The certificate file is not secret and does not need owner-only mode solely for M108.

### WP3 — Make temporary private-key creation owner-only at inode creation

On Unix:

1. Split managed publication behavior sufficiently to distinguish secret private-key creation from certificate creation.
2. For the private-key temporary file, use the standard-library Unix create-mode facility (`std::os::unix::fs::OpenOptionsExt`) so the requested mode is `0600` when the inode is created.
3. Preserve `create_new(true)` and the existing same-directory temporary-file strategy.
4. Keep a final permission verification before rename; a later chmod may remain as defense in depth but must not be the first confidentiality boundary.
5. Preserve cleanup on write/sync/permission/rename failure.
6. On non-Unix, retain the current portable creation path without inventing an ACL implementation.

### WP4 — Planning-state reconciliation

Update the planning control surface so it describes actual status rather than pre-M107 state:

1. `plans/registry.md` must register M108 as the sole dependency-ready handoff and record M107 as closed.
2. Remove stale statements that M107 is `[READY]`, is the current handoff, or has API/AddressBook/TLS corrections still pending.
3. `plans/implementation/i2pcontrol-proposal-170/README.md` and the full-support roadmap must be reconciled during M108 implementation/closure if they still contain pre-M107 language.
4. The roadmap must continue to show M104 blocked on 158 residual TunnelManager cells independently of M108.
5. Closing M108 must not manufacture a residual-option successor. A future residual plan may be registered only if separate evidence actually resolves one of the existing blockers.

## 8. Failure, restart, cancellation, and contention semantics

### Startup/repair

Managed TLS repair happens synchronously during I2PControl initialization before listener service begins. Any directory/key permission-repair or revalidation failure aborts I2PControl initialization.

M108 must not start with a permissive managed key after a failed repair and must not fall back to plaintext.

### Restart

A restart with valid managed material must reuse the same certificate/key bytes. If permissions are already restrictive, startup is idempotent. If a legacy managed key/directory is permissive, the first successful M108 startup repairs permissions and then reuses the same material; subsequent restarts observe the restrictive state.

### Publication failure

Temporary-file cleanup semantics remain bounded. A failed key write, sync, permission check, or rename must remove the temporary file where possible and must not replace the last known valid managed key with partial bytes.

### Contention/local races

M108 must preserve the existing assumption that the router base path is an operator-controlled local state directory. Within that boundary it must narrow the pre-read exposure window by restricting the managed directory before child access and by creating secret temporary files with `0600` from inception.

Do not claim a new general-purpose defense against an attacker who can arbitrarily replace objects inside the router base directory between path-based standard-library filesystem calls. Adding `openat`/`O_NOFOLLOW`-style descriptor ownership through a new dependency is outside this corrective pass.

## 9. Compatibility and migration

This is an in-place security repair for automatically managed TLS material.

- Fresh installations behave as under M107, except private-key temporary files become restrictive at creation rather than after write.
- Existing managed certificate/key bytes are reused when valid.
- Existing managed directories broader than `0700` and managed private keys broader than `0600` are tightened on Unix before key bytes are read.
- No schema or file-name migration occurs.
- Explicit operator TLS paths are unchanged.
- Non-Unix behavior is unchanged except for shared refactoring required to keep publication logic coherent.

If a managed directory/key cannot be repaired to the required Unix permissions, I2PControl startup fails with a bounded TLS error rather than silently accepting insecure managed state.

## 10. Focused regression tests

At minimum add or adjust tests proving:

### Existing managed material

- a pre-existing regular managed directory with mode `0755` is repaired to `0700` on Unix;
- a pre-existing valid managed key with mode `0644` is repaired to `0600` before normal reuse;
- key/certificate bytes remain unchanged when only permission repair is needed;
- a second startup is idempotent and reuses the same material;
- permission-repair failure, where reproducible without privileged/environment-specific assumptions, is treated as initialization failure rather than ignored.

### Managed type safety

- existing symlink key/certificate cases continue to fail closed and external symlink targets remain untouched;
- non-regular managed objects continue to fail closed;
- permission repair never follows an explicit operator TLS path.

### Create-time confidentiality

The implementation must contain a direct/static regression proving that the Unix private-key temporary-file open path requests mode `0600` at creation. Prefer a focused unit-level helper contract over timing-sensitive attempts to observe the file between write and chmod.

Also keep the final-mode test proving the published key is `0600` and the managed directory is `0700`.

### Existing M107 behavior

Retain passing evidence for:

- managed certificate validation for `localhost`, `127.0.0.1`, and `::1`;
- valid material restart reuse;
- invalid ordinary regular material regeneration;
- no plaintext fallback;
- API 1-only and AddressBook regressions through the broader suite.

## 11. Broad verification

Run focused tests plus the existing feature/containment suite:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit
cargo check
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

The known workspace rustfmt gate may still report unrelated existing formatting differences. Record the exact outcome; do not retain broad formatter churn outside M108-authorized paths merely to make that unrelated gate green.

No new hosted CI, fuzzing, coverage, or release loop is required.

## 12. Documentation and static guards

Implementation/closure must ensure the planning control surface is internally consistent:

- M107 is closed at `27a0376`;
- M108 is the current corrective handoff until closure;
- the three M107 protocol/AddressBook/fresh-TLS corrections are described as landed, not pending;
- M108 is described specifically as upgrade-path managed-permission hardening;
- full Proposal 170 support remains partial for the independent 158 TunnelManager residual cells.

Reuse existing M061/M062/M095/M105 guards. Do not create a redundant CI framework.

If the existing containment guard has an exact milestone allowlist that must mention M108 test/planning paths, update only that guard and keep production authority limited to `emissary-cli/src/i2pcontrol/**`.

## 13. Acceptance criteria

M108 may enter closure only when all are true:

1. On Unix, an existing Emissary-managed TLS directory is restrictive (`0700`) before managed child material is read, or startup fails.
2. On Unix, an existing regular Emissary-managed private key is restrictive (`0600`) before key bytes are read, or startup fails.
3. A valid legacy managed key/certificate is not rotated solely for permission repair.
4. On Unix, a newly created private-key temporary file requests `0600` at inode creation rather than relying on a post-write chmod as the first confidentiality control.
5. Symlink/non-regular managed-path failures remain fail-closed and do not mutate external targets.
6. Explicit operator TLS material behavior remains unchanged.
7. M107 loopback SAN and fresh-material behavior remains passing.
8. Feature-gated tests, containment tests, live local runtime test, check, and clippy pass, or any unrelated pre-existing failure is precisely recorded.
9. No production file outside `emissary-cli/src/i2pcontrol/**` changes.
10. No Cargo manifest, lockfile, dependency, Yosemite/SAM, core/util, frontend, workflow, or release change occurs.
11. M095 remains `224 apply / 158 blocked_primitive / 458 not_applicable`.
12. `plans/registry.md`, the implementation README, and the full-support roadmap no longer describe closed M107 work as ready/pending/current.
13. Closure records exact implementation commits, changed paths, permission migration evidence, verification results, residual risks, and internal-only attestation.

## 14. Stop conditions

Stop and return to planning if:

- secure legacy-key repair requires changing explicit operator TLS ownership;
- correctness requires a new filesystem/ACL dependency, privileged helper, or router-wide filesystem abstraction;
- implementation requires production changes outside `emissary-cli/src/i2pcontrol/**`;
- tests require weakening symlink/type checks to make permission repair pass;
- the corrective pass begins rotating managed material for unrelated certificate-policy reasons;
- any proposed change touches TunnelManager option semantics or changes the M095 matrix;
- work expands into the known workspace rustfmt or GitHub Pages issues.

## 15. Closure evidence required

The M108 closure record must contain:

- implementation commit(s) and exact implementation head;
- requirement-to-evidence matrix;
- exact changed paths;
- before/after Unix mode evidence for legacy managed directory/key fixtures;
- evidence that valid legacy key/certificate bytes remain stable across repair and restart;
- create-time `0600` private-key publication evidence;
- symlink/non-regular fail-closed regression evidence;
- explicit-path non-interference review;
- focused and broad verification command outcomes;
- confirmation that M095 remains `224 / 158 / 458`;
- confirmation that planning state no longer marks M107 ready/pending;
- residual-risk statement for the operator-controlled-base-directory assumption;
- internal-only attestation that external sources were read-only and no upstream interaction or contribution artifact was created.
