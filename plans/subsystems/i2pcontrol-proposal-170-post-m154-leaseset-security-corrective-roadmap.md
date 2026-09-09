# I2PControl Proposal 170 Post-M154 LeaseSet-Security Corrective Roadmap

Status: **active / partial; M155-M157 closed, M158 registered**

This roadmap supersedes the LeaseSet-security execution ordering in the older residual and post-M146 roadmaps while preserving M146/M147/M154 closures as historical evidence.

Current authority:

- current whole-surface qualification: M153;
- current M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M146 closed blocked (`UseOutproxyPlugin` ×4);
- M154 disposition C left configurable Destination `SigType` blocked ×10;
- M155 closed semantic/owner re-freeze;
- M156 closed neutral Red25519/blinding primitive;
- M157 closed modern Encrypted LeaseSet2 publication with zero promotions;
- **M158 is the sole registered successor**.

## 1. Corrective architecture

M154 correctly blocked a general configurable destination-signature-suite primitive, but modern Encrypted LeaseSet2 does not require that entire capability. The active line uses the existing type-7 Ed25519 Destination and the closed M156 type-11 Red25519 blinded-key primitive without reopening M147/M148.

The old M149 -> M150 -> M151 ordering remains superseded because `EncryptLeaseSet` is a ten-value mode selector whose lookup-password and PSK/DH modes consume the independently visible `OptionalLookup` and `LeaseSetClientAuths` fields.

Legacy `encrypted (aes)` remains a separate M161 compatibility gate; it must not block modern ELS2 infrastructure, but it may continue to block all five `EncryptLeaseSet` cells if that enum value remains valid and unsupported.

## 2. Direct Proposal/reference corrections

Direct Proposal-170 PR source establishes:

- `i2cp.encryptLeaseSet=true` only for the legacy AES mode;
- modern blinded/PSK/DH modes select `i2cp.leaseSetType=5` and do not use the legacy flag as the modern selector.

Direct Java I2P/I2PTunnel source additionally establishes the lookup-secret standard property:

```text
i2cp.leaseSetSecret = Base64(UTF8(secret))
```

and the encrypted-service extended B32 implementation uses type-7 -> type-11 one-byte sigtypes with secret/auth flags and CRC-32/XOR header protection.

These direct-source contracts govern M158 and later M162 integration. Historical closure evidence remains immutable.

## 3. Containment rules

- M061/M062/M093 remain binding.
- Proposal policy and secret/config mapping stay in `emissary-cli/src/i2pcontrol/**` where possible.
- Red25519, ELS2 serialization/publication, lookup-secret derivation and auth crypto are neutral lower-layer primitives with Proposal-free APIs.
- Every production milestone requires an exact-file budget; no broad crypto/netdb/i2np/destination/primitives prefix authority.
- A later milestone may use a stricter subset of already-realized M061 exact paths without creating a duplicate source-boundary waiver; the milestone plan and registry must enumerate that subset exactly.
- No downgrade from requested encrypted/authenticated publication to ordinary public LeaseSet2 or empty-secret blinding.
- Secret, PSK, DH and blinding private material is never response-facing or logged.
- M147/M148 remain blocked; no general signature-suite migration is smuggled into this line.
- M146 remains blocked; no clearnet egress/provider work is mixed into this line.

### M157 containment reconciliation — closed

M157 realized its exact ten-path production budget and reconciled every new path into M061's ordinary `[allowed]`/`[[evidence]]` ledger. It added no dependency, Cargo/lockfile, Yosemite, or I2PControl production source change.

### M158 containment registration — current

M158 uses exactly four existing M061-authorized owners:

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

No new source path is authorized. M062 records zero new files, dependencies, manifests, lockfile, Yosemite, and I2PControl-source changes.

The secret must be extracted from generic SAM options before `SamCommand` construction and carried through a dedicated zeroizing/non-`Debug` type. This prevents the standard Base64 secret from escaping through debug-capable generic option state.

## 4. Dependency graph

```text
M153 current-head requalification                   [CLOSED]
  |
  v
M154 SigType domain/security refreeze               [CLOSED; M147 PATH BLOCKED]
  |
  v
M155 LeaseSet semantic/owner refreeze               [CLOSED; ZERO PRODUCTION]
  |
  v
M156 narrow Red25519/blinding primitive             [CLOSED; ZERO PROMOTION]
  |
  v
M157 modern Encrypted LeaseSet2 publication         [CLOSED; ZERO PROMOTION]
  |
  v
M158 lookup-secret + blinded-address primitive      [REGISTERED; ZERO PROMOTION]
  |
  v
M159 PSK client-authorization primitive             [DEFERRED; ZERO PROMOTION]
  |
  v
M160 DH client-authorization primitive              [DEFERRED; ZERO PROMOTION]
  |
  +--> M161 legacy AES/LS1 feasibility              [DEFERRED; ZERO PRODUCTION]
  |
  v
M162 Proposal LeaseSet-field integration            [DEFERRED; CONDITIONAL PROMOTIONS]
  |
  v
M152 final residual requalification                 [DEFERRED; ZERO PROMOTION]
```

Only M158 is executable now.

## 5. Milestone intents

### M157 — modern encrypted-LS2 publication

Closed complete. Type-5 outer framing, nested no-auth encryption, blinded storage key, DatabaseStore type preservation, canonical publication/storage verification and UTC-day rollover use real current inner LeaseSet2 bytes. Zero Proposal promotions.

### M158 — lookup secret and blinded address

Registered. Add the standard Base64(UTF8) lookup-secret contribution to the existing daily blinding/publication path and implement the canonical encrypted-service extended `.b32.i2p` format.

M158 does not persist I2PControl secrets, add client-side NetDB lookup/decryption, or promote `OptionalLookup`. Those administrative/runtime-integration semantics remain M162 work.

### M159 — PSK client authorization

Deferred. Implement bounded PSK layer-1 authorization with reference interoperability. Zero promotions.

### M160 — DH client authorization

Deferred. Implement bounded X25519 DH layer-1 authorization with per-client work limits and reference interoperability. Zero promotions.

### M161 — legacy AES/LS1 feasibility

Deferred. Resolve the Proposal's `encrypted (aes)` legacy contract. Zero production. If real LS1 implementation is required and acceptable, create a separate successor rather than hiding it inside M162.

### M162 — Proposal integration

Deferred. Map real completed primitives into the five server families and promote only fields whose complete valid administrative/runtime contract is operational.

### M152 — final requalification

Deferred. Recompute all 840 cells and establish final full/safe-partial status. Zero promotions.

## 6. Promotion ceilings

From `336/29/475`:

- modern lookup/auth complete but legacy AES still a valid unsupported value: at most `346/19/475`;
- all ten `EncryptLeaseSet` values complete: at most `351/14/475`.

These remain ceilings, not claims. `SigType` ×10 and `UseOutproxyPlugin` ×4 remain independent blockers.

## 7. Completion conditions

This corrective line completes when:

- M158-M162 close according to their gates;
- modern ELS2 publication/auth/lookup primitives are implemented or truthfully blocked;
- legacy AES has an explicit final disposition;
- M152 requalifies the actual current head;
- M095/M105/registry/roadmaps agree;
- any remaining blockers are explicit architecture/security blockers rather than missing planning.
