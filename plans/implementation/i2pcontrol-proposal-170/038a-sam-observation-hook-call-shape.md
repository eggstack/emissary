# M038A — SAM Observation Hook Call-Shape Corrective Pass

Status: closed

Source milestones:

- `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`
- `plans/implementation/i2pcontrol-proposal-170/038-live-runtime-interoperability.md`

Corrective finding:

- M037 committed `socket.peer_addr().map(super::sanitized_peer)` even though
  `sanitized_peer` accepts `Option<SocketAddr>`. The feature-enabled workspace
  does not compile, so M038 cannot build its production-composition harness.

## Objective

Restore compilation of the existing passive SAM observation seam without
changing SAM behavior, event contents, bounds, ownership, or the M038 runtime
scope.

## Required change

Change the one call site to pass the complete optional peer address to the
existing sanitizer. Do not alter the sanitizer, event type, core lifecycle,
observation policy, or any I2PControl wire behavior.

## Verification

Run:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core sam --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

The known stable/nightly rustfmt option mismatch is a baseline tooling finding;
do not retain unrelated formatter changes.

## Acceptance and closure

The pass closes only when the feature-enabled check and focused SAM tests pass,
the diff contains only the single call-shape correction, and no production
behavior or external interaction changes.
