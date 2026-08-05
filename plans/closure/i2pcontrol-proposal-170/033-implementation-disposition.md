# M033 Implementation Disposition — Tunnel Lifecycle Reconciliation

Status: closed for implementation; M033 final-head closure accepted

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/033-tunnel-lifecycle-reconciliation.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Implementation commit:

- `5a2e216` — `feat: reconcile I2PControl tunnel lifecycle`

Frozen implementation/test head: `5a2e216`

## Disposition

M033 is implemented within its bounded lifecycle scope. Production tunnel
manager composition now performs one post-load reconciliation pass for
control-plane-owned generic `client` and `server` definitions whose durable
intent is `StartOnLoad`. Each start uses the existing backend path and is
isolated from later definitions when it fails.

Lifecycle operations are serialized by exact tunnel name without holding the
TunnelStore or server-secret-store locks across runtime awaits. Edit and rename
are stopped-only for eligible control-plane runtimes. Delete stops and awaits
the exact active task before removing durable state. Restart completes stop and
reloads the latest durable definition before starting the next generation.
Runtime inspection continues to come from the backend supervisor rather than
persisted intent.

Unsupported tunnel types remain exhaustive, resource-free, and non-auto-start;
startup-managed definitions remain externally owned. No public Proposal 170
field, action, status, or tunnel type was added, and no `emissary-core/**`
production file changed.

## Changed-file classification

Production and tests:

- `emissary-cli/src/i2pcontrol/production.rs` — post-load reconciliation,
  per-name lifecycle locks, authoritative active-state checks, stopped-only
  edit policy, stop-before-delete, and stop/reload/start restart ordering.
- `emissary-cli/tests/m033_tunnel_lifecycle.rs` — composed StartOnLoad,
  failure-isolation, ownership-boundary, edit/delete, and restart regressions.

Documentation:

- `docs/i2pcontrol/tunnel-manager.md` — lifecycle and StartOnLoad behavior.
- `docs/i2pcontrol/tunnel-backends.md` — authoritative inspection and manager
  serialization guarantees.
- `docs/i2pcontrol/proposal-170-support.md` — M033 support disposition.
- `docs/i2pcontrol/security.md` — bounded automatic-start and ownership guards.

No production file outside `emissary-cli/src/i2pcontrol/**` changed. No core,
router, SAM protocol, transport, frontend, AddressBook, RouterInfo, CI, or
release code changed.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Post-load StartOnLoad reconciliation | `start_on_load_starts_eligible_client_and_isolates_failure` | pass |
| One failed StartOnLoad does not block another | same test; invalid and valid client definitions | pass |
| Only control-plane client/server definitions auto-start | reconciliation filter plus ownership-boundary test | pass |
| Unsupported and startup-managed definitions are skipped | `start_on_load_skips_unsupported_and_startup_managed_definitions` | pass |
| Runtime inspection is backend-authoritative | manager `with_runtime_state`; restart/get regression | pass |
| Per-name start/stop/restart serialization | lifecycle lock map and existing backend generation fences | pass |
| Restart uses latest durable definition | `restart_reloads_latest_stopped_definition_and_delete_stops_first` | pass |
| Delete stops before durable removal | same regression; exact client task is cancellable | pass |
| Running edit/rename is rejected without mutation | M033 running-edit regression and atomic store tests | pass |
| Failed task recovery remains restartable | M031/M032 supervisor failure and cancellation suites plus M033 composition | pass |
| Task completion releases runtime state | retained client/server supervisor completion tests; authoritative inspection | pass |
| Server identity remains lifecycle-coherent | retained M032 identity/restart/rename/delete tests; M033 ordering review | pass |
| `All` remains bounded, deterministic, and truthful | existing TunnelManager `All` suite; sorted `BTreeMap` inventory and unsupported backend guards | pass |
| Exactly two real backends and ten unsupported backends | retained production registry tests and unchanged registry composition | pass |
| Public wire shape and status inventory unchanged | conformance manifest, literal fixtures, and unchanged handler contract | pass |
| No core production changes | frozen changed-path inventory | pass |
| No upstream interaction | internal-only attestation below | pass |

## Verification outcomes

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager` | pass, 139 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol start_on_load` | pass, 2 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m033_tunnel_lifecycle` | pass, 3 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter` | pass, 20 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` | pass, 1 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest` | pass, 58 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures` | pass, 7 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass, 1,254 |
| `cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings` | pass |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| direct rustfmt check for changed Rust files | pass, with known stable-toolchain warnings |
| `cargo fmt --all -- --check` | fails on pre-existing repository-wide stable/nightly formatting differences |
| `git diff --check` | pass |

The repository-wide formatter result is a low tooling finding inherited from
the baseline. Running stable `cargo fmt --all` rewrites unrelated files and
nightly-only formatting choices; those generated changes were removed. The
changed Rust files pass direct formatting checks and the implementation commit
contains no formatter spillover.

## Findings and attestation

No unresolved M033 high or medium correctness, security, compatibility,
ownership, scope, or lifecycle finding remains. The ten unsupported tunnel
families remain an intentional high runtime capability gap owned by future
roadmap work, not an M033 defect. M034, AddressBook setter truthfulness, is the
only future plan newly unblocked. M035 through M039 remain blocked on their
named predecessors.

All implementation and closure evidence is internal to `eggstack/emissary`.
External specifications were accessed read-only. No upstream or third-party
issue, pull request, review, submission, adoption request, maintainer contact,
or connector write was created. The explicit maintainer directive to commit and
push authorizes publication of this repository branch only.
