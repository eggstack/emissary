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
| Post-M154 LeaseSet-security corrective | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md` | **M157 registered / dependency-ready** |
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

M157 has zero Proposal promotion budget, so a successful M157 closure does not supersede M153 as whole-surface qualification authority unless a later dedicated requalification says so.

## Accepted blocked dispositions

### M146 — UseOutproxyPlugin

Closed blocked. Four cells remain unsupported because no real bounded local provider exists inside the accepted I2P-only egress/security boundary. Dummy/registry-only providers and `ProxyList` aliases have zero truthful support value; direct-clearnet DNS/TCP remains prohibited.

### M154 / M147 / M148 — configurable Destination SigType

M154 disposition C closed the general configurable Destination signature-suite path as blocked. Ten `SigType` cells remain unsupported under current policy.

This does **not** block the narrower type-7 Ed25519 -> type-11 Red25519 blinded key used internally by modern Encrypted LeaseSet2. M156 implemented that neutral primitive without reopening configurable `SigType`.

## Registered handoff — M157

Plan:

- `plans/implementation/i2pcontrol-proposal-170/157-modern-encrypted-leaseset2-publication-primitive.md`.

Status: **registered / dependency-ready**.

Hard dependency:

- M156 closed complete at `1678790cc74d4075e800425c57312da89e1169fe` with zero Proposal promotions.

M157 class:

- neutral modern Encrypted LeaseSet2 / DatabaseStore / publication infrastructure;
- Proposal promotion budget **zero**.

### Frozen exact production paths

M157 may modify exactly:

1. `emissary-core/src/crypto/els2.rs` — new;
2. `emissary-core/src/crypto/mod.rs` — module declaration/re-export only;
3. `emissary-core/src/primitives/lease_set.rs`;
4. `emissary-core/src/primitives/mod.rs` — exact type re-export only;
5. `emissary-core/src/i2np/database/store.rs`;
6. `emissary-core/src/netdb/mod.rs`;
7. `emissary-core/src/destination/lease_set.rs`;
8. `emissary-core/src/destination/mod.rs`;
9. `emissary-core/src/sam/parser.rs`;
10. `emissary-core/src/sam/session.rs`.

No production path outside this list is authorized.

### Exact containment registration

M061 now contains a guarded `[registered_pending]` M157 ledger with the exact ten paths and exact owner evidence. It intentionally does **not** pre-add newly untouched files to the realized `[allowed]` upstream-diff ledger; the first production commit must atomically move every newly changed path into `[allowed]`/`[[evidence]]` so the current-diff invariant remains truthful.

M062 independently records the same M157 exact path budget and:

- no new direct dependency;
- no Cargo manifest change;
- no lockfile change;
- no Yosemite change;
- no `emissary-cli/src/i2pcontrol/**` production change.

M157 reuses closed M156 Red25519/curve25519-dalek plus existing HMAC-SHA256, ChaCha20, RNG, zeroize, bytes and nom.

### Key exact-owner findings behind the amendment

The M155 candidate path list was insufficient on the current source tree:

- `SamSession` owns the type-7 signing seed and constructs every ordinary signed inner LS2, so `sam/session.rs` and the neutral pre-allocation `sam/parser.rs` seam are required;
- `Destination` joins `SessionManager` and `LeaseSetManager`, and direct DatabaseStore storage-verification replies arrive there, so `destination/mod.rs` is required;
- `NetDb` currently stores raw LeaseSet payload bytes without their type and always re-emits them as ordinary type 3; once type 5 parses, leaving `netdb/mod.rs` unchanged would corrupt a cached encrypted store when flooding or replying;
- the encrypted outer common structure requires a new primitive exported from `primitives/mod.rs`;
- M145's `destination/session/mod.rs` is intentionally **not** in scope: the I2P ELS2 specification permits an authenticated end-to-end session to carry the ordinary unencrypted inner LeaseSet inside wrapped garlic while floodfill publication remains encrypted.

### Reference mapping correction

Direct Proposal-170 PR source is authoritative for future execution:

- `i2cp.encryptLeaseSet=true` is set only for legacy `encrypted (aes)`;
- modern blinded/PSK/DH modes use `i2cp.leaseSetType=5` and do not use the legacy flag as their selector.

The immutable M155 closure is not rewritten, but any older table implying the legacy flag is true for modern type-5 modes is superseded for M157/M162 execution by this direct-source correction.

### M157 hard stops

Stop and amend before editing if implementation requires:

- any production file outside the exact ten-path set;
- any Cargo/dependency/lockfile change;
- any Yosemite or I2PControl production source change;
- another NetDB/query subsystem;
- modifying M145's end-to-end session bundling;
- plaintext type-3 fallback for a type-5 destination.

## Deferred corrected LeaseSet-security chain

Only the next dependency-ready plan is registered at a time.

| Milestone | Purpose | Status / budget |
|---|---|---|
| M157 | modern type-5 Encrypted LeaseSet2 publication/storage verification/UTC rollover | **registered**, 0 promotions |
| M158 | lookup-secret + extended blinded-address primitive | deferred; 0 by default |
| M159 | PSK client authorization | deferred; 0 promotions |
| M160 | DH/X25519 client authorization | deferred; 0 promotions |
| M161 | legacy AES/LS1 feasibility and contract gate | deferred; 0 production/promotions |
| M162 | Proposal LeaseSet-field integration | deferred; conditional up to 15 |
| M152 | final whole-surface requalification | deferred; 0 promotions |

M149-M151 remain historical superseded drafts and must not be executed.

## Field-promotion rules

- Infrastructure/serialization alone has zero support value.
- `OptionalLookup` promotes only when its complete valid secret/blinding contract is operational/interoperable.
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
M157 modern Encrypted LS2 publication          [REGISTERED]
  |
  v
M158 -> M159 -> M160                           [DEFERRED]
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
3. Registration-time pending paths are exact authorization, not a broad prefix; realized source diffs must be reconciled into the ordinary M061 ledger atomically with implementation.
4. No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waiver.
5. No direct-clearnet fallback for M146.
6. No cryptographic suite fallback or plaintext/unsecreted/unauthenticated LeaseSet downgrade.
7. Yosemite remains the exact accepted optional pin unless separately superseded.
8. External/upstream interaction remains read-only.

## Registration rules

1. M157 is the **only registered** Proposal-170 successor.
2. Do not begin M158 until M157 closes and the registry explicitly advances M158.
3. The first M157 source commit must reconcile newly realized paths from M061 `[registered_pending]` into its current `[allowed]`/`[[evidence]]` ledger in the same commit.
4. No M158-M162 candidate path is executable authority.
5. Material architecture/path/dependency deviation requires an amendment before implementation.
6. Closure evidence, not parser/serializer reachability, determines support.

Historical closure files remain immutable.