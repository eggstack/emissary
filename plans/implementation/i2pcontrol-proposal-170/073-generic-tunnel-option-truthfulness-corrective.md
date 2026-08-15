# M073 — Generic Tunnel Option Truthfulness Corrective

Status: closed — implementation and closure recorded

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Source reclosure: `plans/closure/i2pcontrol-proposal-170/072-closure.md`.

Planning production baseline: the M072 closing head.

## 1. Objective

Repair the generic Proposal 170 `client` and `server` backend option boundary
identified during M072. Every typed or raw runtime-relevant option must be
applied by the existing generic Yosemite runtime or rejected before listener,
destination, session, or task allocation. Preserve the public schema and the
existing generic data-plane ownership model.

## 2. Confirmed finding

The generic client currently declares `AccessList`, `AllowPlaintext`, custom
options, and I2CP options accepted, but its runtime configuration consumes only
destination, ports, and listen interface. It also has no raw-option allowlist.

The generic server currently declares `IsPrivate`, `HashCash`, `SignatureType`,
and `Consumer` accepted, accepts custom options, and has no raw-option allowlist;
only `i2cp.leaseSetEncType` reaches the existing server runtime configuration.
Unknown and recognized-but-unimplemented raw options can therefore be persisted
and silently ignored at start.

## 3. Required disposition

- Keep exact Proposal 170 wire names and persistence round-trip behavior.
- Reject unsupported typed options through the existing operation-status error
  channel before runtime reservation.
- Add bounded backend-local raw-option allowlists for metadata, backend-owned
  identity fields, and options actually consumed by the generic runtime.
- Keep `leaseSetEncType` as the only accepted generic-server I2CP option unless
  another existing runtime consumer is demonstrated.
- Do not expose secret values in option errors, logs, or debug output.
- Add focused tests proving every rejected option fails before allocation and
  that accepted options are observable in the runtime configuration.

## 4. Non-goals

- No new public fields, statuses, actions, tunnel types, or compatibility aliases.
- No new router-core API or change to the existing Yosemite data plane.
- No implementation of generic access-list, plaintext, signature, hashcash,
  consumer, custom-option, or arbitrary I2CP semantics merely to avoid rejection.
- No changes to the already closed specialized M066–M071 families.
- No hosted CI, release, fuzz, soak, or public-network certification machinery.

## 5. Verification

Run the generic backend tests, the full feature-enabled package tests, M061 and
M062/M063 containment checks, feature-disabled and feature-enabled checks,
feature-enabled all-targets Clippy, scoped nightly rustfmt, and `git diff --check`.

## 6. Acceptance and closure

M073 may close only when the M072 blocked rows are removed from the integrated
option matrix, no generic runtime-relevant option is silently ignored, and the
M072 reclosure can be accepted without changing the twelve-type production
registry or the unrelated RouterInfo/AddressBook limitations.

Any option requiring a new runtime or protocol capability becomes a separate
bounded plan rather than being implemented opportunistically here.
