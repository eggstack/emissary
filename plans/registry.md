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
| Post-M154 LeaseSet-security corrective | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md` | **M158 registered / dependency-ready** |
| Post-M146 corrective | **historical through M154 / superseded** | `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md` | M153 complete; M154 disposition C; M147 path blocked |
| Residual primitive completion | **historical through M146 / superseded** | `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md` | M140-M145 complete; M146 blocked |
| Session-lifecycle completion | **closed complete** | `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md` | M134 complete |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current production/support state

M095 remains:

- `336 apply`;
- `29 blocked_primitive`;
- `475 not_applicable`;
- `840` total TunnelManager option/family cells.

Remaining blockers:

- `SigType` — 10;
- `EncryptLeaseSet` — 5;
- `OptionalLookup` — 5;
- `LeaseSetClientAuths` — 5;
- `UseOutproxyPlugin` — 4.

Full Proposal-170 status remains **partial**.

## Current qualification authority

**M153** remains the current whole-surface runtime/security qualification authority at `336/29/475`.

M156, M157, and registered M158 all have zero Proposal-promotion budgets, so they do not by themselves supersede the whole-surface qualification authority.

## Accepted blocked dispositions

### M146 — UseOutproxyPlugin

Closed blocked. Four cells remain unsupported because no real bounded local provider exists inside the accepted I2P-only egress/security boundary. Dummy/registry-only providers and `ProxyList` aliases have zero truthful support value; direct-clearnet DNS/TCP remains prohibited.

### M154 / M147 / M148 — configurable Destination SigType

M154 disposition C closed the general configurable Destination signature-suite path as blocked. Ten `SigType` cells remain unsupported under current policy.

This does not block the narrower type-7 Ed25519 -> type-11 Red25519 blinded key used internally by modern Encrypted LeaseSet2. M156 implemented that neutral primitive without reopening configurable `SigType`.

## Closed immediate ancestry

- M155: LeaseSet-security semantic/owner re-freeze; zero production/promotions.
- M156: neutral Red25519/blinding primitive; zero promotions.
- M157: neutral modern type-5/no-auth Encrypted LeaseSet2 publication/storage-verification/UTC-rollover primitive; zero promotions; exact ten-file production budget realized and reconciled in M061/M062.

M157 closure: `plans/closure/i2pcontrol-proposal-170/157-closure.md`.

## Registered handoff — M158

Plan:

- `plans/implementation/i2pcontrol-proposal-170/158-leaseset-lookup-secret-and-blinded-address-primitive.md`.

Status: **registered / dependency-ready**.

Hard dependency:

- M157 closed complete at `a18fba7fa307d195f2a6e7c2cae57c554b07eba8`.

Class:

- neutral standard lookup-secret/blinding and encrypted-service extended-B32 infrastructure;
- Proposal promotion budget **zero**.

### Exact production budget

M158 may modify exactly four existing production files:

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

No new production file is authorized.

These four paths are already individually present in M061's realized exact allowlist from M060/M157. M158 therefore creates **no new M061 source-boundary waiver**. The milestone-specific four-file subset is frozen by the M158 plan and this registry; implementation outside the subset requires amendment before editing.

M062 records the current M158 dependency budget:

- no new direct dependency;
- no Cargo manifest change;
- no lockfile change;
- no Yosemite change;
- no `emissary-cli/src/i2pcontrol/**` production change;
- no new source file.

### Standard secret contract

Direct Java I2P/I2PTunnel source establishes the standard session representation:

```text
i2cp.leaseSetSecret = Base64(UTF8(secret))
```

M158 consumes that standard property only. Proposal JSON `OptionalLookup` plaintext mapping remains M162 work.

The secret must be Base64-decoded and UTF-8 validated before activation. It must then be removed from the generic session-options map and transferred through a dedicated zeroizing/non-`Debug` neutral secret type. This avoids leaking the Base64 secret through the derived `Debug` representation of `SamCommand::CreateSession` or later retained generic options.

Core does not persist the secret. A new SAM generation must receive the same standard property to reproduce the same same-day blinded identity. I2PControl secret persistence/edit/restart transactionality remains M162 authority.

### Extended encrypted-service B32

M158 implements only the supported current type-7 -> type-11 one-byte-sigtype form:

- 35 decoded bytes / 56 Base32 characters plus `.b32.i2p`;
- flag bit 1 = secret required;
- flag bit 2 = client-auth required;
- reserved/two-byte-sigtype forms reject;
- sigtypes exactly 7 and 11;
- 32-byte unblinded Ed25519 public key;
- Java-compatible IEEE CRC-32 over the public-key bytes, XORed into the first three header bytes before I2P Base32 encoding.

M158 runtime emission sets `auth_required=false`; the codec may round-trip the public auth-required flag so M159/M160 can reuse the format.

For a published type-5 server destination, `sam/session.rs` emits the canonical extended B32 through the existing opaque address-string event seam. `events.rs` is not changed.

### Explicitly out of scope

M158 does not authorize changes to:

- `crypto/red25519.rs`;
- `crypto/mod.rs`;
- `destination/mod.rs` or `destination/session/mod.rs`;
- `events.rs`;
- any `i2np/**`, `netdb/**`, `primitives/**`, router, tunnel or transport file;
- `emissary-cli/src/i2pcontrol/**` or `emissary-cli/src/tunnel/**`;
- Yosemite;
- Cargo manifests or lockfile.

It also does not add client-side blinded lookup/decryption, PSK/DH authorization, legacy AES/LS1, or configurable Destination SigType.

### Promotion disposition

M158 promotes **zero** Proposal cells. `OptionalLookup` remains blocked until M162 supplies the I2PControl mapping, persistent secret custody, redacted Get/rawConfig behavior, edit/restart transactionality, and all five server-family integrations.

M095 must remain exactly `336/29/475` through M158.

### Hard stops

Stop and amend before an edit if M158 requires:

- any production file outside the exact four-file set;
- any new dependency/manifest/lockfile/Yosemite change;
- I2PControl production changes;
- a new NetDB resolver/query subsystem;
- keeping secret material in generic debug-capable options;
- plaintext/unsecreted fallback after secret-required type-5 activation.

## Deferred corrected LeaseSet-security chain

Only the next dependency-ready plan is registered at a time.

| Milestone | Purpose | Status / budget |
|---|---|---|
| M157 | modern type-5 Encrypted LeaseSet2 publication/storage verification/UTC rollover | closed; 0 promotions |
| M158 | lookup-secret + extended blinded-address primitive | **registered**; 0 promotions |
| M159 | PSK client authorization | deferred; 0 promotions |
| M160 | DH/X25519 client authorization | deferred; 0 promotions |
| M161 | legacy AES/LS1 feasibility and contract gate | deferred; 0 production/promotions |
| M162 | Proposal LeaseSet-field integration | deferred; conditional up to 15 |
| M152 | final whole-surface requalification | deferred; 0 promotions |

M149-M151 remain historical superseded drafts and must not be executed.

## Field-promotion rules

- Infrastructure/serialization alone has zero support value.
- `OptionalLookup` promotes only when its complete valid secret/blinding contract is operational/interoperable at the Proposal/I2PControl layer.
- `LeaseSetClientAuths` promotes only when complete PSK/DH semantics are operational, bounded and restart-safe.
- `EncryptLeaseSet` promotes only when every valid Proposal enum value is operational for the family; partial enum-domain support remains blocked.

Planning ceilings from `336/29/475` remain:

- modern lookup/auth complete but legacy AES still a valid unsupported value: at most `346/19/475`;
- all ten `EncryptLeaseSet` values complete: at most `351/14/475`.

These are ceilings, not current claims.

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
M156 narrow Red25519/blinding                  [CLOSED]
  |
  v
M157 modern Encrypted LS2 publication          [CLOSED]
  |
  v
M158 lookup-secret + blinded address           [REGISTERED]
  |
  v
M159 -> M160                                   [DEFERRED]
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
2. Neutral core seams require exact-file M061/M062 authority and Proposal-free APIs.
3. A registered milestone may use a stricter subset of already-realized M061 paths without creating a redundant source-boundary waiver; its plan + registry must enumerate that subset exactly.
4. No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waiver.
5. No direct-clearnet fallback for M146.
6. No cryptographic suite fallback or plaintext/unsecreted/unauthenticated LeaseSet downgrade.
7. Yosemite remains the exact accepted optional pin unless separately superseded.
8. External/upstream interaction remains read-only.

## Registration rules

1. M158 is the **only registered** Proposal-170 successor.
2. Do not begin M159 until M158 closes and the registry explicitly advances it.
3. M158 implementation must remain inside its exact four-file subset even though those files are already present in the broader historical M061 allowlist.
4. No M159-M162/M152 candidate path is executable authority.
5. Material architecture/path/dependency deviation requires an amendment before implementation.
6. Closure evidence, not parser/serializer reachability, determines support.

Historical closure files remain immutable.