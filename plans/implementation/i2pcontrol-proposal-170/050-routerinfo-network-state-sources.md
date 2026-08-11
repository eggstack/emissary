# M050 — RouterInfo IPv4/IPv6 Network State Sources

Status: closed

Planning baseline: `b759038`

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

## 2. Reference semantics

These fields are adopted from i2pd. Read-only reference evidence distinguishes separate v4/v6 status, error, and testing state. i2pd status codes include OK, Firewalled, Unknown and additional modes; error codes distinguish none, clock skew, offline, symmetric NAT, full-cone NAT, and no descriptors; testing is independent boolean state.

Emissary currently caches only `FirewallStatus` for v4/v6. Do not infer error or testing state from firewall status.

## 3. Invariants

- Core enums/booleans are protocol-neutral and do not embed Proposal 170 integer codes.
- Existing reachability behavior, peer tests, transport publication, external-address discovery, and firewall decisions are unchanged.
- State changes are published only at the authoritative existing transition that already knows the result.
- Unknown really means unknown; it is not rewritten as testing, offline, or success.
- I2PControl owns numeric mapping, exact JSON integer types, and compatibility semantics.
- No new network probe or periodic reachability test is introduced for observability.

## 4. Production budget

Primary I2PControl paths plus `main.rs` composition.

Core exception budget:

- `emissary-core/src/events.rs` or `inspection.rs` for neutral cached state;
- `emissary-core/src/transport/mod.rs` for authoritative aggregate transitions;
- the specific SSU2/NTCP2 reachability file only if the transition cannot be observed at manager level.

Any broader core change requires a new plan.

## 5. Work packages

1. Build an exact state/code mapping table from the pinned proposal/reference and identify which states Emissary can actually produce.
2. Audit current v4/v6 reachability transitions and error/testing knowledge points.
3. Extend the neutral event/inspection source with separate v4/v6 status, optional error reason, and testing booleans. Use explicit `Unknown`/`None` internal states.
4. Wire the read-only source through `ProductionRouterInfoControl` and implement the numeric mapping exclusively in I2PControl.
5. Add transition fixtures for success, firewalled/symmetric NAT where supported, testing enter/exit, unknown startup, and independent v4/v6 changes.
6. Change the five contract rows only after every emitted numeric value is proven against the mapping table.

## 6. Failure/restart/contention

State is process-local atomic/short-lock inspection data. Restart initializes to truthful unknown/not-testing unless the canonical owner has a stronger current fact. Observation publication cannot fail the transport operation. No lock may span network I/O or await.

## 7. Tests and verification

Run focused event/transport reachability tests, exact numeric RouterInfo fixtures, independent v4/v6 transition tests, feature/no-feature CLI suites, core package tests, clippy, static no-wire-term guards for core DTOs, and `git diff --check`.

## 8. Acceptance criteria

All five selectors derive from actual independently tracked v4/v6 state; error/testing are not inferred from firewall status; numeric mapping is isolated to I2PControl; no probe/algorithm behavior changes; startup unknown behavior is explicit and tested; changed core paths remain within budget.

## 9. Stop conditions

Stop rather than introduce new reachability traffic, alter transport state machines, or invent error causes not known to Emissary.

## 10. Closure evidence

Closure must contain the internal-state-to-wire-code table, source transition locations, exact fixtures for all five fields, startup/recovery evidence, containment diff review, no-feature evidence, and internal-only/no-upstream attestation.
