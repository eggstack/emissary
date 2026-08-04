# I2PControl Proposal 170 Milestone M030 — Final Closure

Status: partial Proposal 170 support

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/030-addressbook-destination-owner-coherence.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/030-implementation-disposition.md`

Frozen implementation/test head reviewed:

- `29b42f29fdd98914ef95d44f80f9353175019ee0` — `fix: make I2PControl address book owner coherent`

Review date: 2026-08-04

## Final finding

This distinct final-head review closes M030. The enabled AddressBook corrective
scope is exact and evidenced: administrative list/lookup, RouterInfo selectors,
normal Base32 resolution, and normal Base64 resolution all use one coherent
owner; published entries contain full structurally valid destinations; stale
legacy files cannot override update/delete; bounded first activation and
historical repair either publish complete state or fail closed.

The truthful subsystem status remains `partial Proposal 170 support` because
26 RouterInfo sources and missing tunnel data planes remain explicitly
unavailable/unsupported under the retained M020–M028 evidence and ADR-0001.
This is an internal repository disposition, not upstream review, acceptance,
certification, adoption, or merge approval.

## Requirement-to-evidence matrix

| Dimension | Evidence | Result |
|---|---|---|
| Owner coherence | update/delete with stale legacy file; Base32/Base64 assertions | pass |
| Full-destination authority | first activation, API, RouterInfo, and download regressions | pass |
| Repair/fail-closed behavior | matching seed repair and unchanged-file unrepairable test | pass |
| Feature isolation | no-feature and runtime-disabled tests; disable/re-enable tests | pass |
| Persistence/restart | current/backup owner tests and re-enable non-resurrection test | pass |
| Security bounds | loader confinement, regular-file/symlink handling, size limits, sanitized errors | pass |
| Scope | three implementation/test files; no core or prohibited subsystem changes | pass |
| Governance | implementation disposition, roadmap, registry, and support/conformance docs reconciled | pass |

## Verification

The required bounded matrix passed at the frozen implementation/test head:

- no-feature check, AddressBook tests, and clippy;
- enabled-feature check;
- enabled AddressBook tests (237);
- `production_adapter` (20), `production_composition` (8),
  `conformance_manifest` (58), and `m027_literal_fixtures` (7);
- full enabled CLI suite (1,231 tests);
- enabled-feature clippy with `-D warnings`;
- `git diff --check`.

No unrelated `emissary-core` test debt was repaired or used to expand M030.

## Failure, compatibility, and security review

Import/repair validates before mutation and preserves the previous durable and
live generation on failure. Existing current/backup recovery, serialized owner
locking, durable-before-success mutation, cancellation semantics, disabled-mode
legacy behavior, and re-enable authority semantics remain intact. No new
persistence schema, reconciler, background task, dependency, or network
behavior was introduced.

The loader is direct-child and regular-file confined, bounded by count,
per-entry, aggregate, and state limits, and rejects unsafe filenames. Full
destinations are structurally validated before active publication. Errors and
logs do not expose destination contents, tokens, passwords, or private paths.

## Future-plan disposition

M030 was the only dependency-ready implementation handoff. Its final-head
review is now complete, so no future plan is newly unblocked within this
corrective scope. The registry records no successor handoff. Missing RouterInfo
sources and missing tunnel data planes remain separate, explicitly unavailable
work and require new bounded plans and authorization before execution.

## Internal-only attestation

All repository writes remained within `eggstack/emissary`. External authority
was read-only. No upstream issue, pull request, review request, discussion,
submission, adoption request, merge solicitation, maintainer contact, or
contribution artifact was created or prepared.
