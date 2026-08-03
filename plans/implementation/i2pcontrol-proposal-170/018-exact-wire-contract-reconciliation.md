# I2PControl Proposal 170 Milestone 018 — Initial Exact Wire-Contract Reconciliation

Status: closed

Planning baseline: `2816857633a927b629c051e07e7efa5baa8d6e07`

Frozen initial implementation head:

- `ea35de9be339fa2c963f9c553cbbcf01540e3ee3`

Current disposition:

- `plans/closure/i2pcontrol-proposal-170/018-implementation-disposition.md`

Current corrective successor:

- `plans/implementation/i2pcontrol-proposal-170/018a-wire-semantics-and-internal-only-corrective-pass.md`

Ready internal closure successor after the corrective implementation:

- `plans/implementation/i2pcontrol-proposal-170/019a-internal-pinned-revision-reclosure.md`

This file is a retained record of the initial M018 handoff. It is not executable. The detailed original plan remains available in repository history before this corrective status update.

## Retained objective and implementation

M018 initially reconciled the principal Emissary I2PControl surface with the pinned 2026-05-20 Open Proposal 170 revision by adding:

- the exact 43-key RouterInfo addition manifest;
- canonical direct RouterInfo parameter-presence behavior;
- canonical AddressBook entry, `SetSubscriptions`, and `SetConfig` modes;
- seven lowercase TunnelManager action paths and structured success responses;
- canonical direct ClientServicesInfo service parameters;
- separation of wire, source, and runtime support documentation;
- literal focused fixtures and closest-production SAM composition evidence;
- compatibility preservation for existing Emissary request forms.

Those implementation components remain retained unless M018A finds a direct regression.

## Why corrective work is required

Post-implementation internal review found:

1. `i2p.router.net.total.transit.bytes` returns received plus sent transit bytes instead of the forwarded/transmitted total;
2. some valid canonical TunnelManager operational failures return JSON-RPC application errors instead of structured `result.status` outcomes;
3. the static conformance manifest still counts base and compatibility surfaces as canonical Proposal 170 inventory;
4. directly affected TunnelManager documentation contains minor canonical casing and `Name`/`All` defects;
5. the planning system required an absolute internal-only/no-upstream-submission rule.

These findings are owned by M018A. The original M019 handoff is superseded and must not be executed.

## Internal-only boundary

This workstream is internal to `eggstack/emissary`.

No current plan authorizes:

- upstream issues, pull requests, merge requests, discussions, reviews, or patch submissions;
- upstream review, approval, feedback, adoption, or merge requests;
- pushes to upstream remotes;
- upstream maintainer outreach;
- contribution-package, patch-series, or merge-plan preparation;
- connector or API writes against an upstream repository.

External Proposal 170 and reference-implementation material may be inspected read-only for internal verification only.

All writes must remain in `eggstack/emissary` unless a future explicit maintainer directive supersedes `plans/003-planning-process.md`.

## Current execution rule

M018A is complete and its disposition is recorded. M019A is now ready; its independent review
must remain within the successor handoff.

M018A completed the required successor conditions:

- corrects all high and medium findings;
- adds the required regressions;
- runs targeted local verification;
- freezes a complete implementation/test head;
- creates `plans/closure/i2pcontrol-proposal-170/018a-implementation-disposition.md`;
- records compliance with the internal-only boundary.

Final closure, when supportable, may only be described as `closed internally against pinned revision`. It must not imply upstream review, acceptance, adoption, certification, or merge.
