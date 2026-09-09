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
| Post-M154 LeaseSet-security corrective | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md` | **M155 closed; M156 registered / dependency-ready** |
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

### M156 — Narrow Red25519 and Ed25519-blinding primitive

Plan:

- `plans/implementation/i2pcontrol-proposal-170/156-neutral-red25519-blinding-primitive.md`.

Status: **registered / dependency-ready** (hard dependency on M155 closure
satisfied by `plans/closure/i2pcontrol-proposal-170/155-closure.md`).

Class: neutral cryptographic infrastructure.

Budgets:

- Proposal promotions: **zero**;
- production Rust changes: exact M155 §8 files only with M061/M062
  authorization at registration (new `emissary-core/src/crypto/red25519.rs`,
  `emissary-core/src/crypto/mod.rs` declaration only, direct
  `curve25519-dalek` edge);
- dependency changes: direct `curve25519-dalek 5.0.0-pre.6` edge only if
  registration confirms the M155 review.

M155 closed complete with `336/29/475` unchanged, ten-value table and legacy
AES disposition C frozen, narrow blinding formulas/vectors frozen, and exact
M156 files proposed. M156 must not make type 11 a persistent Destination
`SigType` and must not reopen M147/M148.

## Deferred corrected LeaseSet-security chain

All successors below M156 are **deferred / unregistered** until their hard dependencies close and exact M061/M062 production paths are explicitly authorized at registration.

| Milestone | Purpose | Promotion budget |
|---|---|---:|
| M157 | modern type-5 Encrypted LeaseSet2 publication/storage verification/rollover | 0 |
| M158 | lookup-secret + extended blinded-address primitive | 0 by default; M155 authorized up to 5 OptionalLookup only with full-contract proof (default defer to M162) |
| M159 | PSK client-authorization primitive | 0 |
| M160 | DH/X25519 client-authorization primitive | 0 |
| M161 | legacy AES/LS1 feasibility and contract gate | 0; zero production |
| M162 | Proposal LeaseSet-field integration | conditional: up to 15 total |
| M152 | final whole-surface requalification | 0 |

M155 is closed complete (`plans/closure/i2pcontrol-proposal-170/155-closure.md`;
`336/29/475` unchanged; legacy AES disposition C; narrow blinding frozen).
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
M155 LeaseSet semantic/owner re-freeze         [CLOSED]
  |
  v
M156 narrow Red25519/blinding                  [REGISTERED]
  |
  v
M157 -> M158 -> M159 -> M160                   [DEFERRED]
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

1. M156 is the **only registered** Proposal-170 successor.
2. Do not begin M156 implementation until M061/M062 explicitly authorize its exact files/dependency in the registration commit.
3. No M157-M162 candidate core path in a draft is executable authority.
4. Material architecture/path/dependency deviation requires plan amendment before implementation.
5. Closure evidence, not parser/serializer reachability, determines support.

Historical closure files remain unchanged.