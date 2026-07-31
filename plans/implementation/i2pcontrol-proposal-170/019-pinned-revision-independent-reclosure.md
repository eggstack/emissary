# I2PControl Proposal 170 Milestone 019 — Superseded Reclosure Handoff

Status: superseded

This handoff is not executable.

It was activated after the initial M018 implementation disposition, but internal review identified unresolved high/medium findings:

- `i2p.router.net.total.transit.bytes` used received-plus-sent rather than forwarded/transmitted semantics;
- valid canonical TunnelManager operational failures did not consistently use structured `result.status` responses;
- the conformance manifest still counted base and compatibility surfaces as canonical Proposal 170 inventory;
- active planning did not yet contain an absolute no-upstream-submission rule.

The current sequence is:

- `plans/implementation/i2pcontrol-proposal-170/018a-wire-semantics-and-internal-only-corrective-pass.md`
- `plans/implementation/i2pcontrol-proposal-170/019a-internal-pinned-revision-reclosure.md`

M019A is the only future closure handoff and remains blocked until M018A completes on a frozen head.

## Internal-only rule

This repository is reviewing and implementing Proposal 170 internally. No plan authorizes an upstream issue, pull request, merge request, review request, patch submission, discussion, maintainer outreach, branch push, or merge attempt.

External proposal and reference-implementation sources may be read for internal verification only. All writes must remain in `eggstack/emissary`.

No agent may execute this superseded file as a basis for closure or upstream activity.