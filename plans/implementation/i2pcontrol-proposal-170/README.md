# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M153 is current whole-surface qualification authority; M158 registered**.

Pinned Proposal revision: `2026-05-20` (Open).

Current M095 state:

- `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells;
- blockers: 10 `SigType`, 5 `EncryptLeaseSet`, 5 `OptionalLookup`, 5 `LeaseSetClientAuths`, 4 `UseOutproxyPlugin`.

## Current execution authority

Roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Sole registered handoff:

- **M158** `158-leaseset-lookup-secret-and-blinded-address-primitive.md`.

M158 has zero Proposal promotion budget and uses four already-realized M061 exact core owners. It authorizes no new source file, dependency, Cargo/lockfile change, Yosemite change, or I2PControl production source change.

## Immediate lineage

- M153 — closed complete; current whole-surface runtime/security qualification authority at `336/29/475`.
- M154 — closed disposition C; general configurable Destination `SigType` path blocked.
- M155 — closed complete; LeaseSet-security semantic/owner re-freeze.
- M156 — closed complete; neutral Red25519/Ed25519 blinding primitive; zero promotions.
- M157 — closed complete; modern type-5/no-auth Encrypted LeaseSet2 publication/storage verification/UTC rollover; zero promotions.
- **M158 — registered**; standard lookup-secret contribution + encrypted-service extended B32; zero promotions.

M146 `UseOutproxyPlugin` remains closed blocked ×4. M147/M148 configurable Destination `SigType` remains blocked ×10. Neither line is reopened.

## M158 exact path budget

Authorized production paths:

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

No other production file is authorized.

All four are already individually present in M061's realized exact allowlist. M158 is therefore a stricter milestone-specific subset, not a new broad containment waiver. M062 records the current zero-dependency/zero-manifest/zero-lock/zero-Yosemite/zero-I2PControl-source budget.

## M158 key contracts

### Standard lookup secret

Direct Java I2P/I2PTunnel source establishes:

```text
i2cp.leaseSetSecret = Base64(UTF8(secret))
```

M158 consumes this standard property only. Proposal `OptionalLookup` plaintext mapping remains M162 work.

The parser must Base64-decode and UTF-8 validate the property before activation, remove it from generic session options, and transfer it through a dedicated zeroizing/non-`Debug` secret type. The source Base64 value and decoded secret must not survive in generic `SamCommand`/`SamSession` option state.

Core does not persist the secret. Same destination/day/secret must reproduce the same blinded/store key; a changed secret must change the blinded/store key. The same secret must remain in force across M157 UTC rollover.

### Extended encrypted-service B32

M158 implements the current type-7 -> type-11 one-byte-sigtype format:

- 35 decoded bytes;
- 56 I2P Base32 characters plus `.b32.i2p`;
- public `secret_required` and `auth_required` flags;
- sigtypes exactly 7 and 11;
- unblinded 32-byte Ed25519 public key;
- Java-compatible IEEE CRC-32 over the public-key bytes, XORed into the first three header bytes before Base32 encoding;
- reserved/two-byte-sigtype/invalid-key/checksum/length forms fail closed.

M158 runtime emission sets `auth_required=false`. Secret-required type-5 servers publish the same extended public address form with the secret-required flag; the secret itself is never encoded in the hostname.

The existing event API already carries an opaque address string, so `events.rs` is intentionally untouched.

## Explicit M158 exclusions

Do not modify:

- `emissary-core/src/crypto/red25519.rs` or `crypto/mod.rs`;
- `destination/mod.rs` or `destination/session/mod.rs`;
- `events.rs`;
- `i2np/**`, `netdb/**`, `primitives/**`;
- router/tunnel/transport/frontend code;
- `emissary-cli/src/i2pcontrol/**` or `emissary-cli/src/tunnel/**`;
- Yosemite;
- Cargo manifests or `Cargo.lock`.

M158 also does not add client-side blinded lookup/decryption, PSK/DH client auth, legacy AES/LS1, or configurable Destination `SigType`.

## Promotion rules

Infrastructure alone has zero Proposal support value.

M158 promotes **zero** cells. `OptionalLookup` stays blocked until M162 proves I2PControl validation/mapping, persistent secret custody, edit/restart transactionality, Get/rawConfig redaction, and five-family integration.

Planning ceilings remain:

- modern lookup/auth complete but legacy AES still valid/unsupported: at most `346/19/475`;
- all ten `EncryptLeaseSet` values operational: at most `351/14/475`.

These are ceilings, not claims.

## Corrected LeaseSet-security chain

```text
M155 LeaseSet semantic/owner refreeze               [CLOSED]
  |
  v
M156 narrow Red25519 + Ed25519 blinding              [CLOSED]
  |
  v
M157 modern Encrypted LeaseSet2 publication          [CLOSED]
  |
  v
M158 lookup-secret + blinded-address primitive       [REGISTERED]
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

## Containment rules

Accepted authority remains ADR-0001 through ADR-0005, M061/M062 and M093.

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral lower-layer primitives require exact-file authority and Proposal-free APIs.
- A registered milestone may freeze a stricter subset of already-realized M061 paths without duplicating the historical allowlist.
- No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waiver.
- No direct-clearnet fallback and no plaintext/unsecreted/unauthenticated LeaseSet downgrade.
- External/upstream access remains read-only.

Do not begin M159 until M158 closes and the registry explicitly advances it.