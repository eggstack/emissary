# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M153 is current whole-surface qualification authority; M157 registered**.

Pinned Proposal revision: `2026-05-20` (Open).

Current M095 state:

- `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells;
- blockers: 10 `SigType`, 5 `EncryptLeaseSet`, 5 `OptionalLookup`, 5 `LeaseSetClientAuths`, 4 `UseOutproxyPlugin`.

## Current execution authority

Roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Sole registered handoff:

- **M157** `157-modern-encrypted-leaseset2-publication-primitive.md`.

M157 has zero Proposal promotion budget and an exact ten-file neutral core path budget. M061/M062 contain registration-time pending ledgers for those paths; no Cargo, lockfile, Yosemite, I2PControl production source, or new dependency change is authorized.

## Immediate lineage

- M153 — closed complete; current whole-surface runtime/security qualification authority at `336/29/475`.
- M154 — closed disposition C; general configurable Destination `SigType` path blocked.
- M155 — closed complete; LeaseSet-security semantic/owner re-freeze.
- M156 — closed complete; neutral Red25519/Ed25519 blinding primitive; zero promotions.
- **M157 — registered**; modern type-5 Encrypted LeaseSet2 publication/storage verification/UTC rollover; zero promotions.

M146 `UseOutproxyPlugin` remains closed blocked ×4. M147/M148 configurable Destination `SigType` remains blocked ×10. Neither line is reopened by M157.

## M157 exact path budget

Authorized production paths:

1. `emissary-core/src/crypto/els2.rs` — new neutral helper;
2. `emissary-core/src/crypto/mod.rs` — declaration/re-export only;
3. `emissary-core/src/primitives/lease_set.rs`;
4. `emissary-core/src/primitives/mod.rs` — exact type re-export only;
5. `emissary-core/src/i2np/database/store.rs`;
6. `emissary-core/src/netdb/mod.rs`;
7. `emissary-core/src/destination/lease_set.rs`;
8. `emissary-core/src/destination/mod.rs`;
9. `emissary-core/src/sam/parser.rs`;
10. `emissary-core/src/sam/session.rs`.

No other production file is authorized.

### Why the list is larger than the M155 candidates

Current-source research established:

- `SamSession` owns the Ed25519 signing seed and ordinary signed inner LS2;
- `Destination` owns composition and direct storage-verification DatabaseStore handling;
- the NetDB floodfill cache currently loses LeaseSet store type and always re-emits type 3, so type preservation must be corrected when type 5 becomes parseable;
- the type-5 outer common structure needs a primitive export;
- M145's `destination/session/mod.rs` is intentionally not touched because authenticated end-to-end clients may receive ordinary inner LS2 inside wrapped garlic while floodfill publication remains encrypted.

## Direct Proposal-PR mapping correction

For future M157/M162 execution, direct Proposal-170 PR source is authoritative:

- legacy `encrypted (aes)` sets `i2cp.encryptLeaseSet=true`;
- modern blinded/PSK/DH modes use `i2cp.leaseSetType=5` and do **not** use the legacy flag as the modern selector.

Historical M155 closure evidence remains immutable; this direct-source correction supersedes any older planning table that states otherwise.

## Corrected LeaseSet-security chain

```text
M155 LeaseSet semantic/owner refreeze               [CLOSED]
  |
  v
M156 narrow Red25519 + Ed25519 blinding              [CLOSED]
  |
  v
M157 modern Encrypted LeaseSet2 publication          [REGISTERED]
  |
  v
M158 lookup-secret + blinded-address primitive       [DEFERRED]
  |
  v
M159 PSK client-authorization primitive              [DEFERRED]
  |
  v
M160 DH client-authorization primitive               [DEFERRED]
  |
  +--> M161 legacy AES/LS1 feasibility               [DEFERRED]
  |
  v
M162 Proposal LeaseSet-field integration             [DEFERRED]
  |
  v
M152 final whole-surface requalification             [DEFERRED]
```

M149-M151 remain historical superseded drafts and must not be executed directly.

## Promotion rules

Infrastructure alone has zero Proposal support value.

- `OptionalLookup` may promote only when its complete secret/blinding contract is operational/interoperable.
- `LeaseSetClientAuths` may promote only when complete PSK/DH semantics are bounded, restart-safe and interoperable.
- `EncryptLeaseSet` may promote only when **every valid Proposal enum value** is operational for that family.

Planning ceilings remain:

- modern lookup/auth complete but legacy AES still valid/unsupported: at most `346/19/475`;
- all ten `EncryptLeaseSet` values operational: at most `351/14/475`.

These are ceilings, not claims.

## Containment rules

Accepted authority remains ADR-0001 through ADR-0005, M061/M062 and M093.

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral lower-layer primitives require exact-file authorization and Proposal-free APIs.
- M061's M157 `registered_pending` ledger is exact registration authority, not a broad prefix; the first source commit must reconcile newly realized paths into the ordinary upstream-diff allowlist.
- M062 records zero dependency/Cargo/lockfile/Yosemite/I2PControl-source budget for M157.
- No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waiver.
- No direct-clearnet fallback and no plaintext/unsecreted/unauthenticated LeaseSet downgrade.
- External/upstream access remains read-only.

Do not begin M158 until M157 closes and the registry explicitly advances it.