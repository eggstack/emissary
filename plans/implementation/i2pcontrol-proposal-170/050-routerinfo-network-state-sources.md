# M050 — RouterInfo IPv4/IPv6 Network State Sources

Status: corrected/closed through M055 and M056; error rows are unavailable and status/testing fields retain accepted closure

Planning baseline: `b759038`

Post-closure corrective authority: `plans/implementation/i2pcontrol-proposal-170/055-m050-network-error-truthfulness-corrective.md`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Hard dependency: M049 closed

Milestone class: capability + containment invariant

## 1. Objective

Implement five canonical network-state additions:

- `i2p.router.net.status.v6`;
- `i2p.router.net.error`;
- `i2p.router.net.error.v6`;
- `i2p.router.net.testing`;
- `i2p.router.net.testing.v6`.

Preserve neutral internal reachability/error/testing state outside I2PControl and map it to the exact Proposal 170/i2pd numeric vocabulary only inside I2PControl.

Post-closure review at `970252c` retained status.v6 and v4/v6 testing but invalidated both error rows: M050 closure itself records that Emissary has no canonical error owner, while the production mapper turns unset `None` state into wire code `0` (`No error`). M055 is the authoritative corrective plan for those two rows.

## 2. Reference semantics

These fields are adopted from i2pd. Read-only reference evidence distinguishes separate v4/v6 status, error, and testing state. i2pd status codes include OK, Firewalled, Unknown and additional modes; error codes distinguish none, clock skew, offline, symmetric NAT, full-cone NAT, and no descriptors; testing is independent boolean state.

Emissary has real production owners for v4/v6 reachability status and SSU2 peer-test activity. The current repository does not have a canonical production owner for i2pd-style v4/v6 error reason state. Do not infer error or testing state from firewall status.

## 3. Invariants

- Core enums/booleans are protocol-neutral and do not embed Proposal 170 integer codes.
- Existing reachability behavior, peer tests, transport publication, external-address discovery, and firewall decisions are unchanged.
- State changes are published only at an authoritative existing transition that already knows the result.
- Unknown really means unknown; it is not rewritten as testing, offline, success, or `No error`.
- Missing source authority is unavailable, not error code `0`.
- I2PControl owns numeric mapping, exact JSON integer types, source disposition, and compatibility semantics.
- No new network probe or periodic reachability test is introduced for observability.

## 4. Production budget

Primary I2PControl paths plus `main.rs` composition.

Original core exception budget:

- `emissary-core/src/events.rs` or `inspection.rs` for neutral cached state;
- `emissary-core/src/transport/mod.rs` for authoritative aggregate transitions;
- the specific SSU2/NTCP2 reachability file only if the transition cannot be observed at manager level.

The post-closure M055 correction is narrower: only `events.rs` and `inspection.rs` may be touched in core, and only to remove unowned error-only scaffolding where safe. Retained status/testing transport paths are not reopened.

Any broader core change requires a new plan.

## 5. Work packages

1. Build an exact state/code mapping table from the pinned proposal/reference and identify which states Emissary can actually produce.
2. Audit current v4/v6 reachability transitions and error/testing knowledge points, distinguishing production writers from test-only setters.
3. Extend or retain neutral event/inspection state only for facts with canonical owners. Do not treat an unowned optional error field as evidence of an error source.
4. Wire the read-only source through `ProductionRouterInfoControl` and implement numeric mapping exclusively in I2PControl for source-backed values.
5. Add transition fixtures for success, firewalled/symmetric NAT where supported, testing enter/exit, unknown startup, and independent v4/v6 changes.
6. Change contract rows to available only after every emitted numeric value is proven against a real production owner. Under current evidence, M055 must demote both error rows.

## 6. Failure/restart/contention

State is process-local atomic/short-lock inspection data. Restart initializes reachability status to truthful unknown and testing false until canonical owners publish stronger facts. Missing network-error authority remains unavailable; it must not be synthesized as code `0`. Observation publication cannot fail the transport operation. No lock may span network I/O or await.

## 7. Tests and verification

Run focused event/transport reachability tests, exact numeric RouterInfo fixtures for retained status/testing, unavailable/no-partial-result fixtures for unowned error rows, independent v4/v6 transition tests, feature/no-feature CLI suites, core package tests, clippy, static no-wire-term guards for core DTOs, and `git diff --check`.

## 8. Acceptance criteria

Retained status.v6 and testing selectors derive from actual independently tracked v4/v6 state; numeric mapping is isolated to I2PControl; no probe/algorithm behavior changes occur; startup behavior is explicit and tested.

The error selectors may be available only when an actual canonical production owner distinguishes the adopted error states. Under the current owner audit they must be unavailable, with no `None -> 0` fabrication. M055 owns this correction and any safe removal of dead error-only core state.

## 9. Stop conditions

Stop rather than introduce new reachability traffic, alter transport state machines, invent error causes not known to Emissary, infer error state from adjacent signals, or retain availability merely because tests can manually populate an error setter.

## 10. Closure evidence

The original closure remains historical evidence but is partially invalidated for the two network-error rows. M055 closure must provide the production-writer audit, zero-without-owner regression, direct/combined unavailable behavior, retained status/testing evidence, changed-path containment review, no-feature evidence, and internal-only/no-upstream attestation.
