# M108 Closure — Managed TLS Upgrade-Permission Corrective Pass

Status: **closed**

Review date: 2026-09-01.

## 1. Disposition and implementation head

M108 completed its bounded corrective objective at:

- `0a5e8c9` — `fix(i2pcontrol): repair managed TLS upgrade permissions`

The implementation repairs legacy managed TLS permissions on Unix before
managed material is read, requests owner-only permissions when creating a
managed private-key temporary inode, and preserves the existing explicit TLS
ownership and managed-material fail-closed boundaries.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Existing managed directory is restrictive before child access | `ensure_managed_directory` applies `0700`, then re-reads metadata and verifies a non-symlink directory with no group/other mode bits | pass |
| Existing managed private key is restrictive before key reads | `load_or_generate_managed_tls` repairs and revalidates an existing regular key before the `load_key` call | pass |
| Permission repair fails initialization when it cannot be established | Permission-setting and metadata revalidation errors return `I2pControlError::Tls`; no fallback path is introduced | pass; no portable unprivileged failure fixture |
| Valid legacy material remains byte-stable across repair and restart | `tls::tests::permissive_managed_material_is_repaired_and_reused` starts with directory `0755` and key `0644`, then compares certificate/key bytes after repair and a second startup | pass |
| New private-key inode requests `0600` at creation | `tls::tests::managed_private_key_requests_owner_only_mode_at_creation` inspects the file before any write; `OpenOptionsExt::mode(0o600)` is on the private-key create path | pass |
| Final managed key and directory modes remain restrictive | `tls::tests::managed_private_key_is_owner_only` and the legacy repair test verify key `0600` and directory `0700` | pass |
| Symlink/non-regular managed paths remain fail-closed | `tls::tests::managed_symlinks_fail_closed_without_touching_targets`; existing `symlink_metadata` type checks remain before reads/replacement | pass |
| Explicit operator TLS paths are not repaired | `load_explicit_tls` remains separate and calls only `load_certs`/`load_key`; permission helpers are reachable only from managed TLS functions | pass |
| M107 managed TLS behavior remains intact | `managed_certificate_validates_all_loopback_server_names`, `managed_tls_generates_and_loads`, and `managed_tls_recovers_from_invalid_cert` pass | pass |
| No plaintext fallback or unrelated Proposal 170 behavior changed | full feature suite, live runtime, containment, and matrix/audit suites pass; no TunnelManager production path changed | pass |

## 3. Exact changed paths and containment

Implementation commit `0a5e8c9` changed exactly:

- `emissary-cli/src/i2pcontrol/tls.rs`
- `emissary-cli/tests/m062_dependency_containment.rs`

The containment test allowlist change is limited to the M108 planning path.
The subsequent closure/planning commit changed only:

- `plans/closure/i2pcontrol-proposal-170/108-closure.md`
- `plans/implementation/i2pcontrol-proposal-170/108-managed-tls-upgrade-permission-corrective-pass.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

No Cargo manifest, `Cargo.lock`, Yosemite/SAM source, `emissary-core`,
`emissary-util`, frontend, workflow, release, or unrelated production path
changed. The M095 inventory remains exactly:

- `224 apply`;
- `158 blocked_primitive`;
- `458 not_applicable`.

## 4. Verification outcomes

Passed:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib tls::tests --no-fail-fast
  8 passed

cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
  1787 passed across 26 suites

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
  26 passed across 2 suites

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
  1 passed

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
  2 suites passed

cargo check -p emissary-cli --no-default-features --features i2pcontrol
  pass

cargo check
  pass

cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
  pass

git diff --check
  pass
```

The required formatter command was run exactly:

```text
cargo fmt --all -- --check
  fails with exit code 1
```

It reports the established stable/nightly rustfmt mismatch, including
nightly-only-option warnings and pre-existing workspace formatting differences
outside M108. Formatter-only churn was not retained.

## 5. Invariant, failure, recovery, and security review

- Directory repair occurs synchronously during managed TLS initialization,
  before certificate/key child reads. Key repair occurs before `load_key()`.
- `symlink_metadata` checks remain in place for the managed directory, managed
  files, and temporary publication path. External symlink targets are not
  followed or modified by the tested failure cases.
- Temporary private-key creation retains `create_new(true)` and the existing
  same-directory publication strategy. Key write, permission, sync, and
  rename failures retain bounded temporary-file cleanup.
- Valid managed material is reused after permission repair; repair does not
  rotate certificates or keys. Invalid regular material retains the existing
  regeneration behavior.
- Explicit operator certificate/key paths remain read-only inputs to the
  explicit loader and are not chmodded, rewritten, relocated, or regenerated.
- No plaintext fallback, protocol change, dependency, lock, task, schema,
  router algorithm, tunnel data-plane behavior, or TunnelManager disposition
  was introduced.
- The implementation does not claim protection against an attacker who can
  replace objects inside the operator-controlled router base directory between
  path-based standard-library filesystem calls. Descriptor-based ownership or
  ACL abstraction remains outside M108.

## 6. Planning state and future-plan unblock audit

The active planning control surface now records M108 as closed in:

- `plans/implementation/i2pcontrol-proposal-170/108-managed-tls-upgrade-permission-corrective-pass.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

M107 is recorded as closed rather than ready/current/pending. M108 is not a
residual-option milestone and does not unblock a future M104 reattempt. No
future plan has acquired a satisfied hard dependency: M104 remains **closed as
blocked** on the 158 applicable TunnelManager residual cells. No successor is
registered merely because M108 closed.

## 7. Unresolved findings and final disposition

No M108-scoped unresolved implementation finding remains. The residual risk is
the explicit operator-controlled-base-directory race assumption documented in
the plan and above; addressing it would require a separate filesystem/ACL
architecture decision and is not a M108 defect.

M108 is therefore **closed** against implementation head `0a5e8c9`. Full
Proposal 170 support remains correctly partial.

## 8. Internal-only attestation

External sources were read-only evidence. All repository writes remained within
the authorized internal `eggstack/emissary` repository. No upstream issue,
pull request, review, adoption request, submission, merge, release, maintainer
contact, or contribution artifact was created or mutated.
