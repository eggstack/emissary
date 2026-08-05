# M038B — Feature-Test Import Repair Corrective Pass

Status: closed

Source milestones:

- `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`
- `plans/implementation/i2pcontrol-proposal-170/038-live-runtime-interoperability.md`

Corrective findings:

- M037 moved SAM observation DTOs and the observation handle under the
  I2PControl owner, but retained feature-enabled tests still imported those
  symbols from `emissary_core`. `cargo clippy --all-targets --features
  i2pcontrol -D warnings` therefore failed before M038 evidence could be
  accepted.
- The retained M037 static guard scanned its own test module and recognized
  only two equivalent phrasings for
  the unsupported-backend resource-free invariant, while the implementation
  used the existing `resource-free` wording. Its own test name therefore
  produced a false positive when the guard scanned the whole source file.
- The M037 `base32_for_destination` import is used only by unit tests, so a
  feature-only library check emitted an unused-import warning when the test
  module was not compiled.

## Objective

Restore the feature-enabled test target without changing production behavior,
the SAM observation seam, or any I2PControl wire contract.

## Required change

Update the stale test-only imports, align the retained static guard with the
existing invariant wording, and fix the M038 harness lifetime lint. Do not
re-export the moved types from core or change ownership, event contents, or
runtime behavior.

## Verification and closure

Run the feature-enabled all-target clippy command, the affected feature test
suites, `git diff --check`, and the repository's bounded formatting check. The
pass closes when no stale core imports or warning-strict failures remain.
