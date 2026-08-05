# M031 Implementation Disposition — Client Tunnel Runtime Backend

Status: closed for implementation; M031 final-head closure accepted

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/031-client-tunnel-runtime-backend.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Implementation commit:

- `8f635616c174e8681ba86de79f80ca3fff2cccee` — `feat: add I2PControl client tunnel runtime`

Frozen implementation/test head: `8f635616c174e8681ba86de79f80ca3fff2cccee`

## Disposition

M031 is implemented within its bounded runtime scope. Production composition
now registers exactly one real backend for control-plane-owned generic `client`
definitions. That backend reuses the existing Yosemite streaming data plane
through a plain cancellation-aware single-client primitive and an
I2PControl-owned, per-name supervisor.

Startup-managed definitions remain read-only and lifecycle-rejected. The
generic `server` type and the other ten declared tunnel types remain explicit
unsupported backends. No Proposal 170 wire field, method, status, or tunnel
type was added.

## Changed-file classification

I2PControl backend and registry:

- `emissary-cli/src/i2pcontrol/backends/client.rs` — bounded per-name runtime
  supervisor, generation fencing, cancellation, readiness, timeout cleanup,
  client-definition validation, inspection, and the real `client` backend.
- `emissary-cli/src/i2pcontrol/backends/mod.rs` — backend module registration.
- `emissary-cli/src/i2pcontrol/backends/registry.rs` — production registry
  constructor with exactly one real client backend; default/fake registry
  behavior remains all-unsupported.

Production composition:

- `emissary-cli/src/i2pcontrol/production.rs` — optional existing SAM-port
  composition, runtime-state inspection overlay, and stop-before-start
  restart error handling.
- `emissary-cli/src/i2pcontrol/server.rs` — narrow SAM-port composition input.
- `emissary-cli/src/main.rs` — passes the already-bound SAM TCP port only.

Reusable original CLI seam and library test composition:

- `emissary-cli/src/tunnel/client.rs` — plain runtime configuration, readiness
  and cancellation-aware single-client runner, and an internal runtime error
  type; the startup manager's existing behavior remains intact.
- `emissary-cli/src/lib.rs` — library-side test exposure of the original client
  module with minimal configuration stubs required by integration tests.

Documentation and governance:

- `docs/i2pcontrol/{proposal-170-conformance.md,proposal-170-support.md,
  tunnel-backends.md,tunnel-manager.md}` — current client/server/unsupported
  support boundary and lifecycle ownership.
- `plans/implementation/i2pcontrol-proposal-170/{031-client-tunnel-runtime-backend.md,README.md}`
  — implemented status and handoff advancement.
- `plans/registry.md` and `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
  — M031 closed and M032 moved to ready.

No `emissary-core/**` file changed. No startup task was adopted, no private key
or router handle was passed to the backend, and no upstream or third-party
repository was written.

## Before/after disposition

Before M031, production construction used `UnsupportedTunnelBackend` for all
twelve tunnel types, and client lifecycle returned deterministic
not-implemented results. The startup client manager owned an unaddressable
retrying `JoinSet` and exposed no safe per-name cancellation seam.

After M031, production construction receives the existing SAM TCP port and
maps only `TunnelType::Client` to `ClientTunnelBackend`. A control-plane client
start validates its target destination, listen port, and interface before
allocating resources; it reports success only after listener bind and Yosemite
session creation. Stop targets the exact name/generation, waits for graceful
cancellation, and uses a bounded abort fallback. Failed, panicked, completed,
or cancelled tasks release their slot and stale generations cannot overwrite a
replacement.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Real production client backend | `create_production_registry`; registry test; client lifecycle tests | pass |
| Startup-managed lifecycle remains external | existing ownership guards retained; startup inventory remains separate | pass |
| Duplicate start rejected | `client_lifecycle_is_named_cancellable_and_restartable` | pass |
| Stop is idempotent and exact-name scoped | same lifecycle test plus `client_failure_isolated_by_exact_name` | pass |
| Restart has no overlap | stop is awaited before start; lifecycle restart regression | pass |
| Bind failure releases slot | `client_bind_failure_releases_runtime_slot` | pass |
| Failure isolation | `client_failure_isolated_by_exact_name` | pass |
| Unsupported types remain resource-free | existing unsupported backend suite; production registry test | pass |
| Startup manager still builds without feature | no-feature check, no-feature clippy, `client_tunnel` tests | pass |
| No public wire expansion or handler side effects | changed-path review; handler unchanged | pass |
| No core production change | frozen commit path inventory | pass |
| Lock-free network awaits | supervisor/store locks are released before backend awaits; code review | pass |
| Sanitized diagnostics and bounded runtime | fixed backend failure messages, generation map, task cap, timeout tests/review | pass |

## Verification outcomes

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features client_tunnel` | pass, 3 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol client` | pass, 104 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager` | pass, 139 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter` | pass, 20 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` | pass, 8 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest` | pass, 58 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures` | pass, 7 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass, 1,241 |
| `cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings` | pass |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `cargo fmt --all` | pass with known stable-toolchain warnings for nightly-only options; unrelated generated edits reverted |
| `git diff --check` | pass |

`cargo fmt --all -- --check` cannot pass on the repository baseline with the
available stable formatter because the checked-in configuration requests
nightly-only formatting options and existing files consequently differ. This
is a repository/toolchain limitation, not an M031 code or scope finding; the
implementation files were formatted by `cargo fmt --all` and the unrelated
generated changes were removed before the frozen commit.

## Findings and attestation

No unresolved M031 high or medium correctness, security, compatibility, scope,
or lifecycle finding remains. The stable-rustfmt baseline mismatch is a low
tooling finding inherited from the repository and does not alter source
behavior or the frozen changed-file inventory.

M031 is formally closed. M032, generic server backend and destination identity,
is newly dependency-ready and is marked `ready` in the implementation README,
subsystem roadmap, and active registry. M033 and later plans remain blocked on
their named hard dependencies; no other future plan is newly unblocked.

All work remained within the Emissary repository. No upstream issue, pull
request, review, submission, or third-party connector write was created.
