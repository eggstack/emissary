# M038B Implementation Disposition — Feature-Test Import and Guard Repair

Status: implemented; closure accepted

Source plan:

- `plans/implementation/i2pcontrol-proposal-170/038b-feature-test-import-repair.md`

Frozen implementation head: `a5864d2`

## Finding and correction

M037's feature-enabled test targets retained imports for moved SAM observation
types from `emissary_core`; they now import the canonical I2PControl owner.
The test-only `base32_for_destination` import is now gated to test builds so a
library-only feature check is warning-free. The retained unsupported-backend
static guard now scans only the production section and accepts the existing
`resource-free` invariant wording. The M038 harness lifetime lint was also
removed without behavioral change.

These are test/import hygiene corrections only. No production runtime,
observation seam, wire contract, or ownership policy changed.

## Verification

- feature-enabled all-target clippy with `-D warnings` — pass
- feature-enabled CLI suite — 1,325 passed
- no-feature CLI suite — 54 passed
- targeted live-runtime suite — 1 passed
- `git diff --check` — pass

## Attestation

The corrective pass is internal to the Emissary repository. No upstream or
third-party interaction occurred.
