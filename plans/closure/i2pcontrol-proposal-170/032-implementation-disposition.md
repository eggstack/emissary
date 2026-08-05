# M032 Implementation Disposition — Generic Server Backend and Destination Identity

Status: closed for implementation; M032 final-head closure accepted

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/032-server-tunnel-runtime-backend.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Implementation commit:

- `3a03aea` — `feat: add I2PControl server tunnel runtime`

Frozen implementation/test head: `3a03aea`

## Disposition

M032 is implemented within its bounded runtime scope. Production composition
now registers real backends only for generic `client` and `server`; the other
ten Proposal 170 tunnel types remain explicit unsupported backends. The server
backend reuses the existing Yosemite streaming data plane through a narrow
cancellation-aware runner and an I2PControl-owned per-name supervisor.

Control-plane server definitions receive a stable internal identity on first
start. The private destination is generated through the already-bound SAM
endpoint, published into the fixed `server-destinations/` store with current/
backup recovery, and supplied to the runtime without any request-selected
path. The public destination is retained only as validated metadata and is
available to live inspection after session setup. Rename preserves the
identity while stopped; running rename is rejected. Delete awaits the exact
runtime task before removing the definition and secret state, with rollback on
secret publication failure and bounded orphan cleanup on restart.

The existing startup server manager remains externally owned. Its destination
path and task behavior are unchanged; the reusable runner is shared without
adopting its tasks. `PrivKeyFile` remains rejected, and private material does
not enter `rawConfig`, response serializers, Debug/Display output, or backend
errors.

## Changed-file classification

Runtime and backend:

- `emissary-cli/src/tunnel/server.rs` — purpose-specific cancellable server
  runner and persistent-destination generation helper; startup manager behavior
  remains separate and unchanged in ownership.
- `emissary-cli/src/i2pcontrol/backends/server.rs` — bounded server supervisor,
  identity lookup, host/port translation, readiness, cancellation, generation
  fencing, inspection, and real backend.
- `emissary-cli/src/i2pcontrol/backends/{mod,registry,client,fake,unsupported}.rs`
  — server module registration, public-destination status slot, exact real /
  unsupported disposition, and test updates.

Persistence and composition:

- `emissary-cli/src/i2pcontrol/server_secret_store.rs` — fixed-path bounded
  identity-to-private-destination store, validation, permissions, atomic
  current/backup publication, recovery, redacted wrapper, and orphan cleanup.
- `emissary-cli/src/i2pcontrol/production.rs` — store composition, first-start
  identity transaction, public-destination persistence, server rename/delete
  ordering, runtime inspection overlay, and startup inventory isolation.
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs` — filters internal server
  metadata from compatibility responses.
- `emissary-cli/src/{lib.rs,main.rs}` — narrow library/binary composition seam
  for the original server runtime module.

Documentation:

- `docs/i2pcontrol/{proposal-170-support.md,security.md,tunnel-backends.md,tunnel-manager.md}`
  — server runtime support, identity, inspection, lifecycle, and security
  boundary.

No `emissary-core/**` or unrelated runtime subsystem file changed. No startup
task was adopted, no public Proposal 170 field/status/type was added, and no
upstream or third-party repository was written.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Real generic server backend | `create_production_registry` test plus server lifecycle test | pass |
| Exactly two real production backends | production registry disposition test | pass |
| Startup server lifecycle remains external | startup inventory guards and ownership rejection test | pass |
| Cancellable single-server runtime | `run_single_server`; server lifecycle cancellation test | pass |
| SAM/session/forward failure is sanitized and slot is released | bounded supervisor setup/readiness handling; failure tests and clippy review | pass |
| Destination survives stop/restart and router restart | durable server identity/public metadata store and current/backup reload tests | pass |
| Stopped rename preserves identity | definition identity stored separately from name; atomic TunnelStore rename path; documented policy | pass |
| Running rename is rejected | production update state guard | pass |
| Delete stops before secret removal | production delete ordering and server supervisor exact-task stop | pass |
| Secret recovery and safe publication | `current_corruption_recovers_valid_backup`, bounded validation, temp/sync/rename, permission handling | pass |
| No arbitrary path or filename | fixed `server-destinations/` derivation and symlink tests | pass |
| No private material in generic state or output | redacted `StoredDestination`, non-Debug runtime config, internal-key response filtering, `PrivKeyFile` guard | pass |
| Actual public destination only after real session | supervisor destination publication/readiness and ClientServices live overlay | pass |
| Unsupported families resource-free | exhaustive registry and unsupported backend suite | pass |
| No core production changes | frozen changed-path inventory at `3a03aea` | pass |
| No public Proposal 170 extension | response/manifest/conformance tests | pass |
| Lock-free network/cancellation boundary | store snapshots are released before SAM/backend awaits; supervisor locks are released before task await | pass |

## Compatibility and migration

Existing persisted definitions remain readable. Existing startup destination
files are untouched. Control-plane server definitions created before M032 have
no internal identity until first start; first start allocates and durably
publishes it before reporting runtime success. Legacy user `TargetHost`/
`Host` and `TargetPort` values remain parsed; the existing Yosemite forward
data plane accepts loopback target-host semantics and uses the target port (or
`Port` fallback). `StartOnLoad` remains deferred to M033.

## Verification outcomes

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features server_tunnel` | pass, no matching tests in the no-feature binary surface |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol server` | pass, 74 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager` | pass, 139 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter` | pass, 20 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` | pass, 8 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest` | pass, 58 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures` | pass, 7 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass, 1,251 |
| `cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings` | pass |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `cargo fmt --all` | pass with stable-toolchain warnings for nightly-only repository options; unrelated formatter-only edits were reverted |
| `cargo fmt --all -- --check` | fails on the pre-existing repository-wide stable/nightly rustfmt mismatch; no behavioral or M032 scope finding |
| `git diff --check` | pass |

## Findings and attestation

No unresolved M032 high or medium correctness, security, compatibility, scope,
or lifecycle finding remains. The repository rustfmt check mismatch is a low
tooling finding inherited from the baseline. M033 remains responsible for
StartOnLoad, post-load reconciliation, and the broader lifecycle transaction
model.

All work remained internal to `eggstack/emissary`. External Proposal 170
documentation was inspected read-only for option semantics. No upstream issue,
pull request, review, submission, adoption request, maintainer contact, or
third-party connector write was created.
