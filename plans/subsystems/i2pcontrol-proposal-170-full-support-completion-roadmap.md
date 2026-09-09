# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M155-M161 closed, no registered successor**

Pinned Proposal authority: revision `2026-05-20`, status Open.

Current machine authority: M095 `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells.

Current whole-surface qualification authority: M153.

Current execution authority:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`;
- no registered successor; M161 closed outcome B; M162 deferred with satisfied hard deps, registration pending exact-path amendment.

## 1. Current residual inventory

| Cluster | Cells | Current disposition |
|---|---:|---|
| `SigType` | 10 | blocked by M154/M147 configurable-destination path |
| `EncryptLeaseSet` | 5 | blocked; corrected LeaseSet-security line active |
| `OptionalLookup` | 5 | blocked; neutral M158 primitive closed, Proposal integration deferred |
| `LeaseSetClientAuths` | 5 | blocked; PSK/DH neutral line active |
| `UseOutproxyPlugin` | 4 | blocked by M146 |
| **Total** | **29** | |

## 2. Accepted lineage

- M140-M145 completed the safe residual streaming/proxy/presentation/reply-bundling work, reaching `336/29/475`.
- M146 closed UseOutproxyPlugin blocked.
- M153 requalified the current production head.
- M154 closed configurable Destination SigType blocked.
- M155 re-froze LeaseSet-security semantics/owners.
- M156 implemented narrow Red25519/blinding.
- M157 implemented modern type-5/no-auth encrypted-LS2 publication/storage verification/UTC rollover.
- M158 implemented standard lookup-secret contribution and encrypted-service extended B32.
- M159 implemented neutral standard PSK client authorization.
- M160 implemented neutral standard DH (X25519) client authorization.
- M156-M160 all closed with zero Proposal promotions.

Historical closures remain immutable.

## 3. Corrected LeaseSet-security chain

```text
M155 semantic/owner refreeze                   [CLOSED]
  -> M156 Red25519/blinding                    [CLOSED]
  -> M157 modern Encrypted LS2                 [CLOSED]
  -> M158 lookup-secret/blinded address        [CLOSED]
  -> M159 PSK client authorization             [CLOSED]
  -> M160 DH/X25519 authorization              [CLOSED]
  -> M161 legacy AES/LS1 feasibility           [CLOSED; OUTCOME B]
  -> M162 Proposal field integration           [DEFERRED]
  -> M152 final requalification                [DEFERRED]
```

M149-M151 are superseded historical drafts and must not be executed.

## 4. Current M160 boundary

M159 closed record: neutral, zero-promotion PSK infrastructure over the four exact files below (closure: `plans/closure/i2pcontrol-proposal-170/159-closure.md`). M095 remains `336/29/475`.

M160 closed record: neutral, zero-promotion DH infrastructure over the four exact files below (closure: `plans/closure/i2pcontrol-proposal-170/160-closure.md`). M095 remains `336/29/475`.

M160 realized production paths:

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

All are already exact realized M061 owners. M160 created no new source-boundary waiver and made no dependency, Cargo, lockfile, Yosemite, I2PControl production-source, or new-file change.

M160 implemented the standard DH type-5 authorization layer only. It adopted the pinned Java 4096-byte encrypted-data ceiling as the allocation/O(N) work bound, used `i2cp.leaseSetAuthType=1`, required the base `i2cp.leaseSetPrivKey` X25519 private key, accepted bounded contiguous indexed `i2cp.leaseSetClient.dh.N` public keys with duplicates preserved, rejected all-zero shared secrets, and kept all key material out of generic debug-capable state.

M095 remains `336/29/475` after M160.

## 5. Remaining plans reviewed in advance

M160 closed with the same four-file/zero-dependency owners realized against the exact envelope. It used existing X25519 support, the same 4096-byte work bound, explicit all-zero shared-secret rejection, duplicate preservation, and no persistent core secrets.

M161 closed outcome B (valid legacy-LS1 contract but blocked; zero production). It resolves legacy `encrypted (aes)` after the modern line is stable. No implementation successor is created; EncryptLeaseSet remains blocked.

M162 has been corrected to require the exact Proposal PR mappings, including PSK/DH base-key semantics, per-user `Base64(UTF8(name)):Key` properties, typed/redacted I2PControl fields, persistent LeaseSet-security secret custody, transactional definition+secret generations, and complete-field promotion only when the full valid domain is operational.

M152 has been rebased to requalify the closed M156-M159 lineage plus authenticated M159/M160 bounds, M161 disposition and M162 persistence/no-downgrade behavior.

## 6. Field-completeness rules

- Neutral infrastructure has zero Proposal support value by itself.
- `OptionalLookup` promotes only when its complete Proposal administrative/runtime/persistence contract is operational and interoperable.
- `LeaseSetClientAuths` promotes only when the complete PSK/DH list contract is bounded, persistent, restart-safe and interoperable across all five server families.
- `EncryptLeaseSet` promotes only when every valid enum value is operational for the family. If legacy AES remains valid and unsupported, all five cells remain blocked.

## 7. Planning ceilings

From `336/29/475`:

- complete OptionalLookup + LeaseSetClientAuths with EncryptLeaseSet still blocked: at most `346/19/475`;
- complete EncryptLeaseSet enum too: at most `351/14/475`.

SigType ×10 and UseOutproxyPlugin ×4 remain independent blockers. These are ceilings, not claims.

## 8. Containment/security rules

Accepted authority remains ADR-0001 through ADR-0005, M061/M062 and M093.

- Proposal/admin policy stays in I2PControl wherever possible.
- Neutral core seams require exact owners and Proposal-free APIs.
- No broad crypto/netdb/i2np/destination/primitives/transport waiver.
- No direct-clearnet fallback.
- No signing-suite fallback or plaintext/unsecreted/unauthenticated LeaseSet downgrade.
- Secret/private/auth material never becomes diagnostic/response-facing merely to simplify integration.
- Yosemite remains the exact accepted optional pin unless separately superseded.
- External/upstream activity remains read-only.

## 9. Final-state rule

M152 may declare full support only when the mechanically recomputed matrix has zero applicable blockers. Otherwise it may close the current line as safe partial/terminal only when every remaining blocker has an explicit accepted architecture/security disposition and no dependency-ready safe implementation plan remains.

## 10. Current handoff

No milestone is currently registered. Only M162 may be registered next (with exact-path amendment).

Do not begin M162 until its registration is authored and the registry explicitly advances the handoff.