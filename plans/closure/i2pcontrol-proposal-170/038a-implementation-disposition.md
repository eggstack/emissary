# M038A Implementation Disposition — SAM Observation Hook Call-Shape

Status: implemented; closure accepted

Source plan:

- `plans/implementation/i2pcontrol-proposal-170/038a-sam-observation-hook-call-shape.md`

Frozen implementation head: `a5864d2`

## Finding and correction

M037 passed `SocketAddr` through `Option::map` to a sanitizer that accepts the
complete `Option<SocketAddr>`. The feature-enabled production composition
therefore failed to compile. The one call site now passes
`socket.peer_addr()` directly to the existing sanitizer.

No SAM lifecycle, event, ownership, bounds, or I2PControl behavior changed.

## Verification

- `cargo check -p emissary-cli --no-default-features --features i2pcontrol` — pass
- `cargo test -p emissary-core sam --no-fail-fast` — 149 passed
- feature-enabled CLI suite — pass
- `git diff --check` — pass

The repository-wide stable/nightly rustfmt mismatch is a pre-existing tooling
finding; no unrelated formatter changes were retained.

## Attestation

The correction is internal to the Emissary repository. No upstream or
third-party interaction occurred.
