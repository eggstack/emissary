# M116 Closure — M110 Shared-Session, Streamr Isolation, and NewDest Corrective Pass

Status: **closed**

Plan: `plans/implementation/i2pcontrol-proposal-170/116-m110-shared-session-and-newdest-corrective-pass.md`

Pinned contract: I2PControl Proposal 170, revision `2026-05-20`, still Open.

Implementation commit: `626d76311a6dc142ecc07827845081b9a9f4c860`.
Closure documentation commit: the commit containing this closure record.
Pre-corrective matrix SHA-256: `7fa3a6923abab146e060f8ae431e1691905174f1518ac009c94c50528414d94d`.
Closed matrix SHA-256: `76bcd0d0d8ba12d9865eb264f820be5a474134c3d3226c086f7228dc546e3af8`.

## Disposition

M116 is complete against the post-M110 baseline. The bounded I2PControl-only corrective
implementation is landed and the authoritative matrix is reconciled to:

| Matrix state | Before M116 | After M116 | Delta |
|---|---:|---:|---:|
| `apply` | 255 | 248 | -7 |
| `blocked_primitive` | 127 | 134 | +7 |
| `not_applicable` | 458 | 458 | 0 |
| total | 840 | 840 | 0 |

The seven client `NewDest` cells are now `blocked_primitive`, owned by M112. No cell is
`planned_apply`, `unknown`, `unsupported`, or `accept_inert`. `Shared × streamrclient`
remains `apply` because authenticated canonical producer matching is enforced inside the
existing Streamr owner.

## Findings and corrective evidence

| Finding | Resolution | Evidence |
|---|---|---|
| F1 shared-session lost wakeup | waiter registration is enabled while holding the registry mutex, then awaited after release | `waiter_registration_precedes_notification`; existing concurrent first-acquisition tests |
| F2 creator cancellation poisons a key | `CreationReservation` removes an unpublished creating entry on drop and wakes waiters | `dropped_creator_reservation_reopens_its_key`; failure path removes and wakes |
| F3 64-bit identity fingerprint collision | compatibility equality/order retains the exact persistent key privately; custom `Debug`/`Display` redact it | compatibility equality/redaction tests in `runtime/session.rs` |
| F4 Streamr cross-producer delivery | each client stores a canonical producer identity and forwards only matching Yosemite peer identities; aliases are rejected | `streamr_peer_matching_requires_canonical_destination_identity` |
| F5 `NewDest` trigger semantics were unproven | outcome B: all seven client cells remain blocked under M112; common validation rejects before client secret staging/session allocation | M095 `NewDest` row, M112 handoff update, production start ordering |
| F6 raw secret `Debug` surfaces | raw `Debug` derives were removed from `StoredEntry`, `Envelope`, `PendingEntry`, and `State`; public secret wrappers remain redacted | source audit and secret-store redaction tests |

## NewDest semantic freeze

The pinned Proposal lists `NewDest` alongside `Close`, `CloseTime`, and
`PersistentClientKey`; the reference Java client resumes a closed session with a new
destination only under the coupled close-on-idle/new-destination configuration. That is a
session lifecycle trigger, not a generic “generate on every start” instruction. M116 therefore
does not implement an approximation or introduce M112 timers. It rejects any supplied
`NewDest` before `ClientDestinationStore::stage`, which is before destination generation and
Yosemite session construction. M112 now owns the seven residual cells and their exact
close-on-idle/resume generation semantics.

Reference evidence was read-only: Proposal 170’s option inventory and a pinned I2P Java source
snapshot at commit `094624f0990d545526674c3267ce0e6d9985d8b2` were inspected. The runtime
implementation remains limited to the existing Yosemite 0.7.0 public API and I2PControl seams.

## Changed paths and containment

Production paths changed, exactly:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs`;
- `emissary-cli/src/i2pcontrol/backends/streamr.rs`;
- `emissary-cli/src/i2pcontrol/client_secret_store.rs`;
- `emissary-cli/src/i2pcontrol/production.rs`.

The exact M062 authorization guard was extended for these paths and the focused M116 planning
artifacts. No core, util, dependency, startup, frontend, workflow, or release path changed.

## Planning and future handoff audit

M112 remains `proposed / blocked` with 69 cells: its original 62 plus the seven transferred
`NewDest` cells. M111 remains dependency-blocked on an accepted Yosemite session-wire API.
M113 remains blocked on accepted server presentation/routing/LeaseSet primitives. M114 remains
blocked until those residuals and all live/reference/containment gates are closed. No future
plan can be promoted to `ready` from M116 alone; the registry current handoff is therefore
`none`.

M110’s historical closure is not rewritten. Its completion ledger and M105 audit receive only
explicit post-M116 reconciliation fields. The next executable handoff must be selected by a
later planning review after its independent gate is satisfied.

## Verification

The following evidence was run for this closure; command outcomes are recorded here at final
closure time:

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | passed |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | passed; 668 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | passed; 3 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | passed; 29 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | passed; full feature-gated suite, including live runtime |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | passed |
| `cargo fmt --all -- --check` | tooling mismatch: stable rustfmt requests unrelated repository-wide formatting; no formatter churn retained |
| `git diff --check` | passed |

## Attestation

This is an internal repository closure for the pinned Proposal 170 planning line. External
references were read-only. No upstream issue, pull request, review, contribution, release, or
maintainer action was requested or performed.
