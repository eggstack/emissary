# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M153 is current whole-surface qualification authority; M159 registered**.

Pinned Proposal revision: `2026-05-20` (Open).

Current M095 state: `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 cells.

Residual blockers: 10 `SigType`, 5 `EncryptLeaseSet`, 5 `OptionalLookup`, 5 `LeaseSetClientAuths`, 4 `UseOutproxyPlugin`.

## Current execution authority

Roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Sole registered handoff:

- **M159** `159-leaseset-psk-client-authorization-primitive.md`.

M159 is neutral infrastructure with **zero Proposal promotion budget**.

## Immediate lineage

- M153 — current whole-surface qualification authority at `336/29/475`.
- M154 — configurable Destination SigType path closed blocked.
- M155 — LeaseSet-security semantic/owner re-freeze closed.
- M156 — Red25519/blinding closed, zero promotions.
- M157 — modern type-5/no-auth Encrypted LeaseSet2 publication closed, zero promotions.
- M158 — standard lookup-secret contribution + encrypted-service extended B32 closed, zero promotions.
- **M159 — registered**; PSK client authorization, zero promotions.

M146 UseOutproxyPlugin ×4 and M154/M147/M148 SigType ×10 remain blocked and are not reopened.

## M159 exact production budget

Only these four existing files may change:

1. `emissary-core/src/crypto/els2.rs`
2. `emissary-core/src/destination/lease_set.rs`
3. `emissary-core/src/sam/parser.rs`
4. `emissary-core/src/sam/session.rs`

All four already exist in M061's realized exact allowlist. M159 creates no new M061 path waiver.

M062 freezes:

- no new source file;
- no new dependency;
- no Cargo/lockfile change;
- no Yosemite change;
- no I2PControl production-source change.

Any fifth production file or dependency change requires amendment before editing.

## M159 standard PSK contract

Standard properties:

```text
i2cp.leaseSetType=5
i2cp.leaseSetAuthType=2
i2cp.leaseSetPrivKey=Base64(32B)
i2cp.leaseSetClient.psk.N=[Base64(UTF8(name)) ":"] Base64(32B)
```

The base `leaseSetPrivKey` is itself an authorized PSK; indexed entries are additional. Core does not retain Proposal user names or persist PSK material.

Authenticated layer-1 uses:

- flags `0x03`;
- fresh auth cookie and auth salt;
- `ELS2PSKA` 52-byte HKDF-SHA256 output: key32 + IV12 + clientID8;
- 8-byte client ID + 32-byte encrypted auth-cookie records;
- auth-cookie-bound layer-2 derivation;
- randomized order for multiple clients.

M159 adopts pinned Java `EncryptedLeaseSet.MAX_ENCRYPTED_SIZE=4096` as the authenticated encrypted-data allocation/O(N) work ceiling. Keys are never silently dropped/truncated to fit.

Parser extraction must remove `leaseSetPrivKey` and indexed PSK values from generic debug-capable option state before activation and carry them in zeroizing/non-`Debug` neutral types.

M158 lookup secret may coexist independently; extended B32 must set `auth_required=true` and preserve the correct `secret_required` flag.

## Proposal disposition

M159 promotes **zero cells**. M095 must remain `336/29/475`.

`LeaseSetClientAuths` stays blocked because Proposal mapping/names/persistent custody/edit/restart/Get-redaction/five-family integration remain M162, and DH remains M160.

## Remaining chain — already reviewed

```text
M159 PSK authorization             [REGISTERED]
  -> M160 DH/X25519 authorization  [DEFERRED; SAME 4-FILE/ZERO-DEP ENVELOPE PRE-FROZEN]
  -> M161 legacy AES/LS1 gate      [DEFERRED; HARD-DEPENDS M160; ZERO PRODUCTION]
  -> M162 Proposal integration     [DEFERRED; CONTRACT PRE-CORRECTED]
  -> M152 final requalification    [DEFERRED]
```

M160 may be registered directly after a clean M159 closure if the four-file owner graph and existing X25519 dependency remain sufficient. No additional generic exact-path research pass is required in that case.

M161's dependency has been corrected: it runs only after M160. It cannot implement LS1; outcome A creates a separate exact successor, B/C leave EncryptLeaseSet blocked.

M162 has been corrected to require the exact direct-source mappings:

```text
legacy AES -> i2cp.encryptLeaseSet=true
modern -> i2cp.leaseSetType=5
OptionalLookup -> i2cp.leaseSetSecret=Base64(UTF8(value))
PSK -> authType=2 + persistent base key + optional indexed psk clients
DH  -> authType=1 + persistent X25519 private base key + optional indexed dh clients
```

M162 is the first milestone allowed to promote the 15 LeaseSet-security cells and must implement typed/redacted I2PControl state plus transactional secret custody across all five server families.

M149-M151 remain superseded historical drafts and must not be executed.

## Containment rules

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral core seams require exact-file authority and Proposal-free APIs.
- No broad crypto/netdb/i2np/destination/primitives/transport waiver.
- No plaintext/unsecreted/unauthenticated downgrade.
- No secret/private/auth material in generic diagnostics or response-facing storage.
- External/upstream access remains read-only.

Execute **M159 only**.