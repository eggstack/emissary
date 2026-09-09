# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M153 is current whole-surface qualification authority; M155 closed, M156 is the sole registered successor**.

Pinned Proposal revision: `2026-05-20` (Open).

Current M095 state:

- `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells;
- 10 `SigType` blockers;
- 5 `EncryptLeaseSet` blockers;
- 5 `OptionalLookup` blockers;
- 5 `LeaseSetClientAuths` blockers;
- 4 `UseOutproxyPlugin` blockers.

## Current execution authority

Roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Registered handoff:

- **M156** `156-neutral-red25519-blinding-primitive.md` — registered/dependency-ready on the M155 closure (`plans/closure/i2pcontrol-proposal-170/155-closure.md`); neutral primitive, zero promotion budget; exact M061/M062 authorization required at registration before coding.

M155 closed complete (`plans/closure/i2pcontrol-proposal-170/155-closure.md`): ten-value `EncryptLeaseSet` table frozen, legacy AES disposition C delegated to M161, narrow type-7 Ed25519 -> type-11 Red25519 blinding path frozen without reopening M147/M148, field coupling and exact M156 files frozen.

## Historical/blocked authority

- M146 `UseOutproxyPlugin`: closed blocked; four cells remain unsupported under current egress/security architecture.
- M153: closed complete; current whole-surface runtime/security qualification authority at `336/29/475`.
- M154: closed complete, disposition C; general signature-domain audit.
- M155: closed complete; LeaseSet-security semantic/owner re-freeze (legacy AES disposition C, narrow blinding frozen, M156 unblocked).
- M147/M148 configurable Destination `SigType`: closed/deferred behind the M154 blocked disposition; ten cells remain unsupported. This line is **not reopened** by encrypted-LS2 work.

Historical closure files remain immutable.

## Corrected LeaseSet-security chain

```text
M155 LeaseSet semantic/owner refreeze               [CLOSED; ZERO PRODUCTION]
  |
  v
M156 narrow Red25519 + Ed25519 blinding              [REGISTERED; ZERO PROMOTION]
  |
  v
M157 modern Encrypted LeaseSet2 publication          [DEFERRED; ZERO PROMOTION]
  |
  v
M158 lookup-secret + blinded-address primitive       [DEFERRED]
  |
  v
M159 PSK client-authorization primitive              [DEFERRED; ZERO PROMOTION]
  |
  v
M160 DH client-authorization primitive               [DEFERRED; ZERO PROMOTION]
  |
  +--> M161 legacy AES/LS1 feasibility               [DEFERRED; ZERO PRODUCTION]
  |
  v
M162 Proposal LeaseSet-field integration             [DEFERRED; CONDITIONAL PROMOTIONS]
  |
  v
M152 final whole-surface requalification             [DEFERRED; ZERO PROMOTION]
```

Only one next dependency-ready plan is registered at a time.

## Superseded drafts

M149, M150 and M151 remain in-tree as historical planning drafts but are **superseded for execution** by M155-M162. Their old dependency ordering assumed M148 completion and treated `EncryptLeaseSet`, `OptionalLookup` and `LeaseSetClientAuths` too independently.

M152 remains the final requalification milestone, but is now re-gated on M162 plus any legacy-AES implementation successor created by M161 outcome A.

## Promotion rules for the corrected line

Infrastructure alone has zero Proposal support value.

- `OptionalLookup` may promote up to five server-family cells only when its entire valid secret/blinding contract is operational and interoperable.
- `LeaseSetClientAuths` may promote up to five cells only when all valid PSK/DH uses are bounded, restart-safe and interoperable.
- `EncryptLeaseSet` may promote up to five cells only when **every valid Proposal enum value** is operational for that family. Partial enum-domain support remains blocked.

Possible planning ceilings from `336/29/475`:

- modern lookup/auth complete but legacy AES still a valid unsupported value: at most `346/19/475`;
- all ten EncryptLeaseSet values also complete: at most `351/14/475`.

These are ceilings, not current claims. `SigType` and `UseOutproxyPlugin` remain separate blockers unless independently resolved.

## Containment rules

Accepted authority remains ADR-0001 through ADR-0005, M061/M062 and M093.

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral lower-layer primitives require exact-file authorization and Proposal-free APIs.
- Planning-only M062 entries do not authorize production paths.
- No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waiver.
- No direct-clearnet fallback for M146.
- No plaintext/unsecreted/unauthenticated LeaseSet downgrade.
- No crypto suite fallback to manufacture support.
- Yosemite remains the exact optional accepted revision unless separately superseded.
- External/upstream access remains read-only.

Machine evidence remains centered on M061/M062, M095, M105 and the current qualification closure.