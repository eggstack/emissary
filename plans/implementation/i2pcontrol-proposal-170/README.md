# Proposal 170 Implementation Handoffs

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/017-closure-invalidation.md`
- `plans/closure/i2pcontrol-proposal-170/018-implementation-disposition.md`

## Internal-only rule

These handoffs are for internal `eggstack/emissary` work only.

No plan authorizes:

- an upstream issue, pull request, merge request, discussion, review request, or patch submission;
- upstream review, feedback, approval, adoption, or merge solicitation;
- pushing branches, commits, tags, patches, or artifacts to an upstream remote;
- upstream maintainer outreach;
- preparation of an upstream contribution package or merge plan.

External Proposal 170 and reference-implementation sources may be inspected read-only for internal verification. All writes must remain in `eggstack/emissary` unless a future explicit maintainer directive supersedes the normative planning policy.

## Current handoffs

| Handoff | Status | Plan | Activation dependency |
|---|---|---|---|
| M018 — Initial exact wire-contract reconciliation | corrective pass required | `018-exact-wire-contract-reconciliation.md` | implementation retained at `ea35de9`; post-disposition defects recorded |
| M018A — Wire semantics and internal-only corrective pass | ready | `018a-wire-semantics-and-internal-only-corrective-pass.md` | sole executable implementation handoff |
| M019 — Original pinned-revision reclosure | superseded | `019-pinned-revision-independent-reclosure.md` | non-executable; replaced by M019A |
| M019A — Internal pinned-revision reclosure | blocked | `019a-internal-pinned-revision-reclosure.md` | complete frozen M018A head, disposition, and distinct internal reviewer |

Earlier milestones remain historical as recorded in the subsystem roadmap and registry.

## Current execution order

```text
M018A wire semantics and internal-only corrective pass
    |
    v
M019A internal pinned-revision independent reclosure
```

Execute only M018A. Do not begin M019A until the registry marks M018A `closing` and M019A `ready`.

Never execute the superseded M019 plan.

## Why M018 was reopened

The initial M018 implementation correctly added the principal canonical request forms and manifests, but later internal review found:

- `i2p.router.net.total.transit.bytes` sums received and sent transit counters instead of returning the forwarded/transmitted total;
- some valid canonical TunnelManager operation failures leave the structured `result.status` envelope;
- the conformance manifest still counts base and compatibility surfaces as canonical Proposal 170 inventory;
- minor TunnelManager documentation errors;
- missing normative prohibition against upstream submission and review solicitation.

The retained implementation is not discarded. M018A is a narrow semantic, evidence, and governance correction.

## M018A scope

M018A owns only:

- transit-byte semantic correction and distinct-counter regression;
- canonical TunnelManager operational-failure result envelopes;
- canonical/base/compatibility manifest separation;
- directly affected documentation;
- internal-only policy verification and disposition attestation.

M018A must not add:

- upstream submission, review, adoption, merge, or maintainer-contact activity;
- missing tunnel data planes;
- broad router, transport, NetDB, peer, tunnel, cryptographic, resolver, frontend, SAM, or I2CP architecture;
- generic protocol/schema/fixture frameworks;
- dependencies;
- repository-wide formatting;
- CI, release, publishing, platform, coverage, or generated-evidence machinery;
- fabricated values.

Required output:

- corrected production behavior;
- focused regressions;
- updated directly affected documentation;
- `plans/closure/i2pcontrol-proposal-170/018a-implementation-disposition.md`;
- frozen implementation/test head.

## M019A closure rule

M019A is an independent internal review only.

It must verify:

- the pinned proposal revision using read-only source access;
- exact 43-key RouterInfo inventory and transit semantics;
- canonical AddressBook modes and adjudicated result envelope;
- all seven lowercase TunnelManager actions and structured success/failure operation outcomes;
- direct ClientServicesInfo behavior;
- canonical/base/compatibility inventory separation;
- truthful unavailable and unsupported behavior;
- qualified SAM evidence;
- final changed-file scope and targeted commands;
- no-upstream compliance.

Any unresolved high or medium finding rejects closure and returns to M018A.

Final status may only be `closed internally against pinned revision`. It must not imply upstream review, acceptance, certification, adoption, or merge.

## Verification rule

Use local package-scoped checks:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Focused runs:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol transit
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest
```

Use touched-file configured formatting checks when unrelated workspace baseline differences prevent full stable formatting. Do not reformat unrelated files.

Remote CI, upstream CI, release verification, platform matrices, coverage gates, fuzz campaigns, network farms, submission checks, and generated evidence bundles are not required.