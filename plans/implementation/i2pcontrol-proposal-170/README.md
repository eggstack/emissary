# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

Pinned external authority:

- Proposal 170 `I2PControl Expansion`, Open, created/updated `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

## Internal-only rule

These handoffs are internal to `eggstack/emissary`.

No plan authorizes:

- an upstream issue, pull request, merge request, discussion, review request, or patch submission;
- upstream review, feedback, approval, adoption, or merge solicitation;
- pushing branches, commits, tags, patches, artifacts, or releases to an upstream remote;
- upstream maintainer outreach;
- preparation of an upstream contribution package, patch series, submission checklist, or merge plan;
- connector/API writes against any upstream or third-party repository.

External specifications and reference implementations may be inspected read-only for internal correctness. All writes must remain in `eggstack/emissary` unless a future explicit maintainer directive supersedes the normative planning policy.

Violation is a stop condition and invalidates affected evidence.

## Scope rule

The Proposal 170 corrective sequence owns API correctness and the smallest truthful source/ownership adapters.

It must not implement missing tunnel data planes. The following remain separate security-focused work:

- HTTP client/server and bidirectional server tunnels;
- IRC client/server tunnels;
- SOCKS-IRC and CONNECT variants;
- Streamr client/server tunnels;
- any other listener, destination, LeaseSet, or traffic path not already implemented by Emissary.

Unsupported tunnel types remain explicit administrative definitions with deterministic inactive/not-implemented lifecycle behavior under ADR-0001.

Changes outside `emissary-cli/src/i2pcontrol/**` are permitted only for:

- one purpose-specific runtime AddressBook handle;
- composition-time startup tunnel inventory and existing-handle wiring;
- passive proxy exit observations;
- correction to the already-introduced bounded SAM observation seam;
- bounded read-only RouterInfo snapshots adjacent to existing authoritative owners.

No broad router, protocol, transport, NetDB, peer-selection, tunnel, cryptographic, streaming, resolver, frontend, CI, release, dependency, or formatting project is authorized.

## Closure invalidation

M019A is historical evidence only. Its internal-only/no-upstream attestation remains valid, but its implementation-completeness disposition is invalidated by:

- base I2PControl authentication/token incompatibility;
- JSON-RPC notification and request-ID defects;
- incorrect canonical TunnelManager `get` schema;
- non-atomic rename and secret-boundary defects;
- disconnected AddressBook shadow state;
- missing startup tunnel inventory and stale proxy lifecycle state;
- sticky SAM observation overflow;
- unresolved RouterInfo source/claim contradictions;
- fixtures that validated repository output rather than the full pinned contract.

See `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`.

## Corrective handoffs

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| 014 — Spec-constrained truthfulness and local hardening | implementation retained | `014-spec-constrained-truthfulness-and-local-hardening.md` | broad final acceptance reopened by M018/M019 |
| 015 — Focused independent reclosure | invalid historical closure | `015-focused-independent-reclosure.md` | superseded |
| 016 — Bounded SAM session observation corrective pass | implementation retained | `016-sam-fencing-and-connection-proof-corrective-pass.md` | bounded SAM component accepted |
| 017 — Final-head independent reclosure | corrective pass required | `017-final-head-independent-reclosure.md` | closure invalidated by exact Proposal 170 contract findings |
| 018 — Exact wire-contract reconciliation | closed | `018-exact-wire-contract-reconciliation.md` | implementation accepted at `ea35de9`; M019 closure recorded |
| 019 — Pinned-revision independent reclosure | closed against pinned revision | `019-pinned-revision-independent-reclosure.md` | final source/head/SAM review accepted |

Earlier plans remain historical as recorded in the subsystem roadmap. M019 remains superseded and non-executable.

## Execution order

```text
M018 exact wire-contract reconciliation (closed)
    |
    v
M019 pinned-revision independent reclosure (closed against pinned revision)
```

M018 implementation was frozen at `ea35de9`; M019 independently reviewed the
actual final head and recorded acceptance in
`plans/closure/i2pcontrol-proposal-170/019-closure.md`.

## Handoff discipline

Each implementation plan must produce an implementation disposition containing:

- implementation commits;
- exact changed files;
- requirement-to-evidence matrix for its bounded objective;
- focused and broad command outcomes;
- failure/restart/contention evidence;
- compatibility and migration effects;
- security review;
- unresolved findings with severity;
- scope/no-upstream attestation;
- frozen implementation/test head.

A successful implementation commit or broad test count is not closure by itself.

See:

- `plans/closure/i2pcontrol-proposal-170/017-closure-invalidation.md`.

## M018 handoff rule

M018 owns only exact wire reconciliation and directly affected evidence/documentation.

Required areas:

- exact 43-key RouterInfo manifest and direct parameter presence;
- exact AddressBook canonical modes;
- exact lowercase TunnelManager actions and structured results;
- exact direct-parameter ClientServicesInfo requests;
- compatibility aliases clearly separated from canonical behavior;
- literal official-example fixtures;
- strongest feasible production-composition SAM lifecycle evidence;
- separate wire/source/runtime support claims.

Compatibility forms may remain, but cannot substitute for canonical Proposal 170 forms or count toward canonical coverage.

M018 must not add:

- missing tunnel data planes;
- broad router, transport, NetDB, peer, tunnel, cryptographic, resolver, frontend, SAM, or I2CP architecture;
- generic protocol/schema/fixture/inspection frameworks;
- repository-wide formatting;
- CI, release, publishing, platform, coverage, or generated-evidence machinery;
- fabricated values for unavailable sources.

## M019 closure rule

M019 must independently refetch the still-open Proposal 170 source and verify that the implementation matches the pinned revision.

The reviewer must be distinct from the final M018 implementation executor and identify the separate agent/run.

M019 independently checks:

- exact 43 RouterInfo strings and types;
- AddressBook primary-source response adjudication;
- all seven lowercase TunnelManager actions and structured results;
- direct ClientServicesInfo selection with any value;
- compatibility extension isolation;
- truthful unavailable and unsupported behavior;
- SAM current-session/removal evidence;
- final changed-file scope and targeted command outcomes.

Any unresolved high/medium finding rejects closure and returns work to M018.

Final status is `closed against pinned revision`, because Proposal 170 remains Open.

## Verification rule

Default CLI package scope:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Run core package scope only for plans that touch the permitted runtime/SAM/inspection seams:

```bash
cargo check -p emissary-core
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
```

Each plan lists focused filters to run first. Use touched-file formatting when unrelated workspace formatting differences remain.

Remote CI, upstream CI, release checks, platform matrices, coverage gates, fuzz campaigns, network farms, long soak tests, submission checks, and generated evidence bundles are not required.

## Final-status rule

M027 restored the final subsystem disposition.

Possible outcomes:

- `closed internally against pinned revision` when exact wire behavior and every claimed source/runtime dimension have evidence;
- `partial Proposal 170 support` when one or more pinned sources remain truthfully unavailable after bounded-source work;
- `corrective pass required` for unresolved high/medium defects;
- `blocked` when necessary evidence cannot be obtained.

Missing tunnel data planes may remain explicit runtime-unsupported stubs without violating the API scope, but documentation must never count those stubs as real runtime implementation.

No final status implies upstream review, acceptance, certification, adoption, or merge.
