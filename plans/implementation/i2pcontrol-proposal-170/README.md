# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M153 is current whole-surface qualification authority; M160 registered**.

Pinned Proposal revision: `2026-05-20` (Open).

Current M095 state: `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 cells.

Residual blockers: 10 `SigType`, 5 `EncryptLeaseSet`, 5 `OptionalLookup`, 5 `LeaseSetClientAuths`, 4 `UseOutproxyPlugin`.

## Current execution authority

Roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Sole registered handoff:

- **M160** `160-leaseset-dh-client-authorization-primitive.md`.

M160 is neutral infrastructure with **zero Proposal promotion budget**.

## Immediate lineage

- M153 — current whole-surface qualification authority at `336/29/475`.
- M154 — configurable Destination SigType path closed blocked.
- M155 — LeaseSet-security semantic/owner re-freeze closed.
- M156 — Red25519/blinding closed, zero promotions.
- M157 — modern type-5/no-auth Encrypted LeaseSet2 publication closed, zero promotions.
- M158 — standard lookup-secret contribution + encrypted-service extended B32 closed, zero promotions.
- M159 — neutral standard PSK client authorization closed, zero promotions.
- **M160 — registered**; DH (X25519) client authorization, zero promotions.

M146 UseOutproxyPlugin ×4 and M154/M147/M148 SigType ×10 remain blocked and are not reopened.

## M159 closed record

M159 modified exactly the four files below and closed with zero promotions (closure: `plans/closure/i2pcontrol-proposal-170/159-closure.md`). M095 remains `336/29/475`.

## M160 exact production budget

Only these four existing files may change:

1. `emissary-core/src/crypto/els2.rs`
2. `emissary-core/src/destination/lease_set.rs`
3. `emissary-core/src/sam/parser.rs`
4. `emissary-core/src/sam/session.rs`

All four already exist in M061's realized exact allowlist. M160 creates no new M061 path waiver.

M062 freezes:

- no new source file;
- no new dependency;
- no Cargo/lockfile change;
- no Yosemite change;
- no I2PControl production-source change.

Any fifth production file or dependency change requires amendment before editing.

## M160 standard DH contract

Standard properties:

```text
i2cp.leaseSetType=5
i2cp.leaseSetAuthType=1
i2cp.leaseSetPrivKey=Base64(32B X25519 private)
i2cp.leaseSetClient.dh.N=[Base64(UTF8(name)) ":"] Base64(32B X25519 public)
```

The public key derived from the base private key is always in the authorized set; indexed entries are additional. Duplicate entries are preserved exactly as configured. Core does not retain user names or persist DH material.

Authenticated layer-1 uses:

- flags `0x01`;
- a fresh ephemeral X25519 keypair and fresh auth cookie per regenerated object;
- `ELS2_XCA` 52-byte HKDF-SHA256 output: key32 + IV12 + clientID8;
- 8-byte client ID + 32-byte encrypted auth-cookie records;
- explicit all-zero shared-secret rejection;
- auth-cookie-bound layer-2 derivation (L1 unchanged);
- randomized order for multiple clients.

M160 adopts pinned Java `EncryptedLeaseSet.MAX_ENCRYPTED_SIZE=4096` as the authenticated encrypted-data allocation/O(N) work ceiling. Checked complete-size calculation occurs before any per-client X25519 work. Clients are never silently dropped/truncated to fit.

Parser extraction must remove `leaseSetPrivKey` and indexed DH values from generic debug-capable option state before activation and carry them in zeroizing/non-`Debug` neutral types. DH and PSK modes are mutually exclusive.

M158 lookup secret may coexist independently; extended B32 must set `auth_required=true` and preserve the correct `secret_required` flag.

## Proposal disposition

M160 promotes **zero cells**. M095 must remain `336/29/475`.

`LeaseSetClientAuths` stays blocked because Proposal mapping/names/persistent custody/edit/restart/Get-redaction/five-family integration remain M162.

## Remaining chain — already reviewed

```text
M159 PSK authorization             [CLOSED]
  -> M160 DH/X25519 authorization  [REGISTERED]
  -> M161 legacy AES/LS1 gate      [DEFERRED; HARD-DEPENDS M160; ZERO PRODUCTION]
  -> M162 Proposal integration     [DEFERRED; CONTRACT PRE-CORRECTED]
  -> M152 final requalification    [DEFERRED]
```

M161 runs only after M160. It cannot implement LS1; outcome A creates a separate exact successor, B/C leave EncryptLeaseSet blocked.

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

Execute **M160 only**.