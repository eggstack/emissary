# M051 — RouterInfo Router-News and Banned-Peer Semantic Sources

Status: blocked

Planning baseline: `b759038`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Hard dependency: M050 closed

Milestone class: capability + contract adjudication

## 1. Objective

Resolve and implement the final two currently unavailable Proposal 170 RouterInfo additions:

- `i2p.router.news`;
- `i2p.router.netdb.bannedpeers`.

These are semantic-risk fields because current Emissary has no router-news subsystem and no identified canonical ban-list owner. The milestone must establish whether an authoritative capability-empty value is contract-correct, or whether the field is genuinely blocked by an absent subsystem. It must not add unrelated router functionality merely to make the matrix green.

## 2. Required reference adjudication

Before production edits, inspect the pinned Proposal 170 text and read-only reference implementation/history for the exact meaning of:

- no news subsystem / no news entries;
- no banned peers / router implementation without a ban-list capability;
- required object keys and value types for banned-peer entries.

Document whether empty string and empty map are valid authoritative states or whether the contract assumes an implemented source owner. A hard-coded empty value is not acceptable evidence by itself.

## 3. Invariants

1. Do not create a news downloader/feed, release checker, UI news service, ban engine, peer-blocking policy, firewall rule, or routing penalty solely for I2PControl.
2. Do not reinterpret profile buckets, dial failures, unreachable peers, or temporary transport errors as bans.
3. Capability-empty values may be reported only when reference evidence establishes that they exactly represent the current router state.
4. If a canonical existing owner is discovered, expose it through the smallest read-only source and keep all mapping/bounds/serialization in I2PControl.
5. No change to peer selection, routing, transport acceptance, NetDB behavior, or frontend state.

## 4. Production budget

Preferred and expected: `emissary-cli/src/i2pcontrol/**` only.

`emissary-cli/src/main.rs` may wire an already-existing owner if one is discovered. No `emissary-core/**` production change is authorized by M051 without a new explicit corrective/architecture plan.

## 5. Work packages

1. Produce a short contract-adjudication table for news and banned peers with Proposal 170/reference citations and exact empty/non-empty semantics.
2. Audit current Emissary for a real news or ban owner; distinguish absence of capability from an empty current set.
3. If contract-correct, model implementation capability explicitly in I2PControl (e.g. a source that authoritatively reports no news/no bans because Emissary implements no such subsystem), rather than scattering hard-coded serializer defaults.
4. If an existing owner exists, add a bounded read-only adapter and fixtures.
5. Update only the two `PROPOSAL_170_CONTRACT` rows whose semantic source has been proven.

## 6. Failure/restart/contention

Capability-empty sources are immutable and require no locks/persistence. A real discovered owner must be read-only, bounded, and restart-native. Source failure uses existing sanitized inspection errors; no fabricated fallback.

## 7. Tests and verification

Tests must distinguish capability-empty from source failure, prove exact empty wire values when authorized, and verify non-empty banned-peer serialization if an owner exists. Re-run RouterInfo contract/golden suites, full feature/no-feature CLI tests, clippy, and `git diff --check`.

## 8. Acceptance criteria

Both selectors have documented authoritative semantics tied to the pinned contract/reference; no unrelated news/ban subsystem is added; no peer-control behavior changes; empty values, if used, are explicitly proven as current-state values rather than placeholders; source-map rows and docs are updated consistently.

## 9. Stop conditions

If Proposal 170 requires a substantive news feed or ban-list subsystem that Emissary does not implement, stop and close M051 as blocked for that field. Do not expand scope. M052 must then report RouterInfo source completion as incomplete rather than fabricating support.

## 10. Closure evidence

Closure requires the two-field semantic adjudication record, exact fixtures, owner audit, changed-path review, no-feature evidence, and internal-only/no-upstream attestation.
