# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

Accepted Proposal-170 architecture/security authority:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security.

Pinned Proposal 170 revision: `2026-05-20` (Open).

Authorized internal repositories:

- `eggstack/emissary`;
- `eggstack/yosemite` only under ADR-0005 and Yosemite's own registered plans.

All upstream/third-party repositories and maintainer channels remain read-only.

## Active roadmaps

| Subsystem | Status | Roadmap | Current handoff |
|---|---|---|---|
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | subordinate to current corrective roadmap |
| Post-M154 LeaseSet-security corrective | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md` | **M155 registered / dependency-ready** |
| Post-M146 corrective | **historical through M154 / superseded for next execution** | `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md` | M153 complete; M154 disposition C; M147 path blocked |
| Residual primitive completion | **historical through M146 / superseded** | `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md` | M140-M145 complete; M146 blocked |
| Session-lifecycle completion | **closed complete** | `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md` | M134 complete |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current production/support state

Current M095 authority:

- `336 apply`;
- `29 blocked_primitive`;
- `475 not_applicable`;
- `840` TunnelManager option/family cells total.

Remaining blockers:

- `SigType` — 10;
- `EncryptLeaseSet` — 5;
- `OptionalLookup` — 5;
- `LeaseSetClientAuths` — 5;
- `UseOutproxyPlugin` — 4.

Full Proposal-170 status remains **partial**.

## Current qualification authority

**M153** is closed complete and is the current whole-surface runtime/security qualification at `336/29/475`.

- plan: `plans/implementation/i2pcontrol-proposal-170/153-post-m146-current-head-requalification-and-authority-rebase.md`;
- closure: `plans/closure/i2pcontrol-proposal-170/153-closure.md`.

M139 is historical qualification ancestry only.

## Accepted blocked dispositions

### M146 — UseOutproxyPlugin

Closed blocked. Four cells remain unsupported because no real bounded local provider exists inside the accepted I2P-only egress/security boundary. Dummy/registry-only providers and `ProxyList` aliases have zero truthful support value; direct-clearnet DNS/TCP is prohibited.

### M154 / M147 / M148 — configurable Destination SigType

M154 closed complete with disposition C and closed the M147 general signature-suite path as blocked. The ten `SigType` cells remain blocked under current policy because the full destination-capable configurable domain cannot be generated/persisted safely without a broad legacy/crypto/storage migration.

This blocked disposition is **not** reopened by the LeaseSet-security line. Modern Encrypted LeaseSet2 may use a narrower internal type-7 Ed25519 -> type-11 Red25519 blinding primitive without making type 11 a persistent selectable Destination SigType.

## Registered handoff

### M155 — Post-M154 LeaseSet-security semantic and exact-owner re-freeze

Plan:

- `plans/implementation/i2pcontrol-proposal-170/155-post-m154-leaseset-security-semantic-and-owner-refreeze.md`.

Status: **registered / dependency-ready**.

Class: invariant / architecture-security qualification.

Budgets:

- Proposal promotions: **zero**;
- production Rust/dependency/Yosemite changes: **zero**.

M155 must freeze:

1. all ten valid `EncryptLeaseSet` values and exact Java property/runtime mapping;
2. whether legacy `encrypted (aes)` is coherent current behavior, legacy-LS1-only, or unresolved;
3. the narrow Ed25519 -> Red25519 blinded signing requirement for modern Encrypted LS2;
4. exact coupling of `OptionalLookup` and PSK/DH `LeaseSetClientAuths` to EncryptLeaseSet modes;
5. current Emissary exact owners for signing, LeaseSet2, DatabaseStore, publication/storage verification, secret stores, X25519/KDF/ChaCha primitives and B32;
6. maintained Rust dependency/security/no-std posture for the narrow Red25519 primitive;
7. exact production files for M156 and candidate exact files for later milestones.

M155 must preserve `336/29/475` and cannot authorize implementation by itself.

## Deferred corrected LeaseSet-security chain

All successors below are **deferred / unregistered** until their hard dependencies close and exact M061/M062 production paths are explicitly authorized at registration.

| Milestone | Purpose | Promotion budget |
|---|---|---:|
| M156 | narrow Red25519 + Ed25519 blinding primitive | 0 |
| M157 | modern type-5 Encrypted LeaseSet2 publication/storage verification/rollover | 0 |
| M158 | lookup-secret + extended blinded-address primitive | 0 by default; M155 may authorize up to 5 OptionalLookup only with full-contract proof |
| M159 | PSK client-authorization primitive | 0 |
| M160 | DH/X25519 client-authorization primitive | 0 |
| M161 | legacy AES/LS1 feasibility and contract gate | 0; zero production |
| M162 | Proposal LeaseSet-field integration | conditional: up to 15 total |
| M152 | final whole-surface requalification | 0 |

M149-M151 remain in-tree as historical drafts but are **superseded for execution** by M155-M162. Their old dependency ordering through M148 is no longer current authority.

M152 is re-gated on M162 closure plus any implementation successor created by M161 outcome A.

## Corrected field-promotion rules

- Infrastructure or option serialization alone has zero promotion value.
- `OptionalLookup` may promote only when its entire valid secret/blinding contract is operational/interoperable.
- `LeaseSetClientAuths` may promote only when all valid PSK/DH uses are bounded, restart-safe and interoperable.
- `EncryptLeaseSet` may promote only when **every valid Proposal enum value** is operational for the family. Partial enum-domain support remains blocked.

Planning ceilings from the current `336/29/475` baseline:

- modern lookup/auth complete but legacy AES still a valid unsupported value: at most `346/19/475`;
- all ten EncryptLeaseSet values complete: at most `351/14/475`.

These are not current claims. `SigType` and `UseOutproxyPlugin` remain independent blockers unless separately resolved.

## Current execution chain

```text
M146 UseOutproxyPlugin                         [CLOSED BLOCKED]
  |
  v
M153 current-head requalification              [CLOSED]
  |
  v
M154 SigType domain/security re-freeze         [CLOSED; DISPOSITION C]
  |
  +--> M147/M148 configurable SigType          [BLOCKED]
  |
  v
M155 LeaseSet semantic/owner re-freeze         [REGISTERED]
  |
  v
M156 -> M157 -> M158 -> M159 -> M160           [DEFERRED]
  |
  +--> M161 legacy AES/LS1 feasibility         [DEFERRED]
  |
  v
M162 Proposal LeaseSet integration             [DEFERRED]
  |
  v
M152 final requalification                     [DEFERRED]
```

## Canonical containment rules

1. Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
2. Neutral core seams require exact-file M061/M062 authorization and Proposal-free APIs.
3. Planning-only M062 entries never authorize production paths/dependencies.
4. No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waiver.
5. No direct-clearnet fallback for M146.
6. No cryptographic suite fallback or plaintext/unsecreted/unauthenticated LeaseSet downgrade.
7. Yosemite remains the exact accepted optional pin unless separately superseded.
8. External/upstream interaction remains read-only.

## Registration rules

1. M155 is the **only registered** Proposal-170 successor.
2. Do not begin M156 until M155 closes complete and registry/M061/M062 explicitly advance it.
3. No M156-M162 candidate core path in a draft is executable authority.
4. Material architecture/path/dependency deviation requires plan amendment before implementation.
5. Closure evidence, not parser/serializer reachability, determines support.

Historical closure files remain unchanged.