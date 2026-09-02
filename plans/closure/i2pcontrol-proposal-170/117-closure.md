# M117 Closure — Internal Yosemite Fork Pin and I2PControl Adapter Integration

Status: **closed**

Plan: `plans/implementation/i2pcontrol-proposal-170/117-internal-yosemite-fork-pin-and-i2pcontrol-adapter-integration.md`

Implementation commit: `22c893a` (`feat(i2pcontrol): adopt pinned Yosemite fork adapter`)

Closure date: 2026-09-02

## Disposition

M117 is closed. The maintainer-authorized internal Yosemite fork is pinned and
feature-contained, and the I2PControl code reaches the closed generic session-wire
and signature-aware destination APIs through one adapter boundary. M117 is
infrastructure/capability plumbing only: no Proposal 170 matrix cell is promoted.

The selected Yosemite revision is `8026f5b424fc178d683e63555335f8b33e0aba04`, the
Y002 implementation commit and a descendant of Y001 implementation commit
`beafafa33e563760a0484df1b5fcaec4e0f8c5e4`. Yosemite's Y002 closure explicitly
identifies this revision as the Emissary pin candidate. Read-only inspection of
the Yosemite history and closure records found no unrelated dependency or
production work in the selected path from baseline `d0fe71da214b212790773be12a93162ae71f3e03`.

## Requirement-to-evidence matrix

| Requirement | Evidence and outcome |
|---|---|
| Exact authorized fork pin | `emissary-cli/Cargo.toml` declares the optional `yosemite-i2pcontrol` alias for package `yosemite`, exact `git` URL, revision `8026f5b...`, and `async-extra`; `Cargo.lock` records the exact git source. |
| Ordinary dependency unchanged | Root `Cargo.toml` remains the existing registry `yosemite = { version = "0.7.0", features = ["async-extra"] }`; non-I2PControl imports remain ordinary `yosemite`. |
| Feature and path containment | `i2pcontrol` alone activates `dep:yosemite-i2pcontrol`; M062 metadata/tree tests prove disabled/enabled reachability; all alias imports are under `emissary-cli/src/i2pcontrol/**`. |
| Y001 session-wire reachability | `generic_session_wire_adapter_reaches_fork_session_create_serializer` reaches a local SAM listener and observes variance, backup, `SIGNATURE_TYPE=11`, and a validated custom option in the emitted `SESSION CREATE`. |
| Y002 signature reachability | `generated_identity_uses_selected_signature_type_without_fallback` observes `DEST GENERATE SIGNATURE_TYPE=11` through the I2PControl destination store and accepts the returned private key. |
| Invalid signature behavior | Adapter and destination-store tests reject invalid signature types before allocation/network use and preserve no fallback to type 7. Fork errors propagate without retrying through ordinary Yosemite. |
| Shared-session compatibility | `compatibility_key` already includes the complete redacted Yosemite `SessionOptions` value; the regression test now distinguishes signature, variance, backup, and custom-option changes while continuing to redact private key material. |
| No raw parallel SAM path | I2PControl use sites were migrated to the alias and the containment suite's production-path/static checks pass; no second command serializer was added. |
| M062 evidence | `m062_dependency_containment` passes all 23 tests, including exact manifest, feature-isolation, lock-source, import-containment, and changed-path checks. |
| Default behavior | Feature-disabled CLI check and workspace check pass; the fork is absent from the disabled dependency tree and ordinary Yosemite remains present. |
| Matrix truthfulness | M095 remains `248 apply / 134 blocked_primitive / 458 not_applicable`; M117 promotes no cell and implements no M118 router behavior. |

## Changed paths

Production and dependency evidence changes are limited to `emissary-cli/Cargo.toml`,
`Cargo.lock`, `emissary-cli/src/i2pcontrol/**`,
`emissary-cli/tests/m062_dependency_containment.rs`, and
`plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`.
The planning changes are this plan, the Proposal 170 README, the full-support
roadmap, and `plans/registry.md`. No `emissary-core/**`, `emissary-util/**`,
startup/tunnel, frontend, workflow, release, or hosted-CI path changed.

## Verification record

All commands were run from the repository root on 2026-09-02.

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | PASS |
| `cargo check -p emissary-cli --no-default-features` | PASS |
| `cargo check` | PASS |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | PASS — 673 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | PASS — 33 tests |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | PASS — no issues |
| `cargo tree -p emissary-cli --no-default-features --edges normal` | PASS — no fork URL; registry Yosemite present |
| `cargo tree -p emissary-cli --no-default-features --features i2pcontrol --edges normal` | PASS — registry Yosemite and exact fork revision present |
| `git diff --check` | PASS before the implementation commit and again during closure preparation |
| `cargo fmt --all -- --check` | FAIL due the repository's existing stable/nightly rustfmt configuration mismatch; it reports broad pre-existing formatting churn, including untouched files. No formatter churn was retained. |

The stale test-target spelling initially attempted during verification (`m061_i2pcontrol_proposal_170`)
was corrected to this checkout's actual `m061_containment`; the corrected command is
the passing command recorded above.

## Invariant, failure, recovery, and security review

- The alias is optional, owned only by the `i2pcontrol` feature, and is not a
  workspace patch, path dependency, branch reference, tag reference, or vendored tree.
- I2PControl typed validation remains the policy boundary. The adapter only
  transfers already-modeled values into Yosemite's generic fields and maps
  serializer rejection to truthful unsupported-option errors.
- Signature parsing is bounded to `u16`; invalid values fail before destination
  generation. No fallback to the default signature is possible for an explicitly
  selected invalid or unsupported type.
- Existing M115/M116 cancellation, restart, rollback, bounded-session ownership,
  and no-lock-across-I/O rules remain unchanged. Yosemite API errors use existing
  failure propagation; no raw command fallback or retry path was introduced.
- Compatibility equality retains all session-affecting options while redacting
  persistent private keys. Added tests prove relevant wire-setting differences do
  not share a session and that secret material does not enter debug/display output.
- Custom options are sent only through Yosemite Y001's bounded validated API; raw
  key/value command construction was not added. Error messages contain option
  names, not secret values.
- No signature algorithms, router behavior, LeaseSet behavior, proxy lifecycle,
  or tunnel-pool behavior were added to Emissary. M118 remains the neutral owner
  for variance/backup runtime semantics.

## Unresolved findings

1. **Low — tooling:** stable and installed nightly rustfmt versions disagree with
   this repository's committed formatting baseline and unstable rustfmt settings.
   This is pre-existing repository/toolchain drift, not an M117 source defect;
   `cargo fmt --all -- --check` remains recorded as failed and no unrelated files
   were reformatted.
2. **Deferred by design — M111 semantics:** the generic API is now available, but
   M111 still must re-freeze Proposal semantics, especially `UseSSL`, and may only
   promote cells with request-to-runtime evidence.

No high- or medium-severity Yosemite serialization/security finding remains open
for the selected Y001/Y002 capabilities.

## Future-plan unblocking decision

M117's dependency gate is closed. M118 is therefore the next dependency-ready
Emissary handoff and remains registered as ready. M111 is not promoted: it remains
blocked on M118's neutral tunnel-pool/router effect for variance and backups and
its own semantic re-freeze. M112 remains blocked on its client proxy/lifecycle
scope, M113 remains blocked on server presentation/LeaseSet semantics, and M114
remains blocked on the zero-residual final reclosure. Yosemite Y003 remains
semantically blocked until M113 freezes its exact LeaseSet interface.

## Internal-only attestation

Yosemite source, history, and closure records were inspected read-only. No
Yosemite repository, upstream repository, issue, pull request, review, maintainer
channel, or release artifact was mutated or requested. The only repository write
authorized here is the internal `eggstack/emissary` implementation and planning
closure; no upstream contribution artifact was prepared.
