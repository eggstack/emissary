# I2PControl Proposal 170 Post-M154 LeaseSet-Security Corrective Roadmap

Status: **active / partial; M155/M156 closed, M157 registered**

This roadmap supersedes the LeaseSet-security execution ordering in the older residual and post-M146 roadmaps while preserving M146/M147/M154 closures as historical evidence.

Current authority:

- current whole-surface qualification: M153;
- current M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M146 closed blocked (`UseOutproxyPlugin` ×4);
- M154 disposition C left configurable Destination `SigType` blocked ×10;
- M155 closed semantic/owner re-freeze;
- M156 closed neutral Red25519/blinding primitive;
- **M157 is the sole registered successor**.

## 1. Corrective architecture

M154 correctly blocked a general configurable destination-signature-suite primitive, but modern Encrypted LeaseSet2 does not require that entire capability. The current line uses the existing type-7 Ed25519 Destination and the closed M156 type-11 Red25519 blinded-key primitive without reopening M147/M148.

The old M149 -> M150 -> M151 ordering remains superseded because `EncryptLeaseSet` is a ten-value mode selector whose lookup-password and PSK/DH modes consume the independently visible `OptionalLookup` and `LeaseSetClientAuths` fields.

Legacy `encrypted (aes)` remains a separate M161 compatibility gate; it must not block modern ELS2 infrastructure, but it may continue to block all five `EncryptLeaseSet` cells if that enum value remains valid and unsupported.

## 2. Direct Proposal-PR mapping correction

Direct `ServiceTunnelCreator` source in the pinned Proposal-170 PR establishes:

- `i2cp.encryptLeaseSet=true` only for the legacy AES mode;
- modern blinded/PSK/DH modes select `i2cp.leaseSetType=5` and do not use the legacy flag as the modern selector.

This direct-source correction governs M157/M162. Historical M155 closure evidence remains immutable.

## 3. Containment rules

- M061/M062/M093 remain binding.
- Proposal policy and secret/config mapping stay in `emissary-cli/src/i2pcontrol/**` where possible.
- Red25519, ELS2 serialization/publication, lookup-secret derivation and auth crypto are neutral lower-layer primitives with Proposal-free APIs.
- Every production milestone requires exact-file registration; no broad crypto/netdb/i2np/destination/primitives prefix authority.
- No downgrade from requested encrypted/authenticated publication to ordinary public LeaseSet2.
- Secret, PSK, DH and blinding private material is never response-facing or logged.
- M147/M148 remain blocked; no general signature-suite migration is smuggled into this line.
- M146 remains blocked; no clearnet egress/provider work is mixed into this line.

### M157 exact containment registration

M157 has an exact ten-path production budget in its plan plus M061/M062 `registered_pending` ledgers.

The registration-time pending ledger deliberately does not pre-populate M061's realized upstream-diff `[allowed]` set with untouched files. The first M157 production commit must atomically reconcile every newly changed path into the ordinary M061 `[allowed]`/`[[evidence]]` ledger.

M157 adds no dependency and authorizes no Cargo, lockfile, Yosemite or I2PControl production source change.

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
M157 modern Encrypted LeaseSet2 publication         [REGISTERED; ZERO PROMOTION]
  |
  v
M158 lookup-secret + blinded-address primitive      [DEFERRED]
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

Only M157 is executable now.

## 5. M157 exact-owner findings

The current source audit expanded the earlier M155 candidate set for concrete reasons:

- `sam/session.rs` owns the Ed25519 signing seed and constructs every ordinary inner LS2;
- `sam/parser.rs` is the neutral fail-before-allocation standard-option gate;
- `destination/mod.rs` joins SessionManager/LeaseSetManager and receives direct DatabaseStore verification replies;
- `destination/lease_set.rs` owns the bounded publication/storage-verification state machine and therefore UTC blinded-key rollover;
- `i2np/database/store.rs` recognizes store type 5 internally but lacks payload/builder support;
- `netdb/mod.rs` currently caches raw LeaseSet bytes without their store type and always re-emits type 3, so it must preserve type 5 exactly after support is introduced;
- `primitives/lease_set.rs`/`primitives/mod.rs` own the type-5 outer common structure;
- a single new `crypto/els2.rs` owns credential/subcredential, exact HKDF schedules, no-auth layer encryption and UTC day conversion while reusing M156 and existing crypto.

M145's `destination/session/mod.rs` is intentionally excluded: the I2P ELS2 specification explicitly permits an authenticated end-to-end session to receive the ordinary unencrypted LeaseSet inside wrapped garlic while floodfill publication is encrypted.

## 6. Milestone intents

### M157 — modern encrypted-LS2 publication

Registered. Implement type-5 outer framing, nested no-auth encryption, blinded storage key, DatabaseStore type preservation, canonical publication/storage verification and UTC-day rollover using real current inner LeaseSet2 bytes. Zero Proposal promotions.

### M158 — lookup secret and blinded address

Deferred until M157 closure. Add optional secret contribution to blinding plus the extended encrypted-service `.b32.i2p` format. Default zero promotions until its entire field contract is proven.

### M159 — PSK client authorization

Deferred. Implement bounded PSK layer-1 authorization with reference interoperability. Zero promotions.

### M160 — DH client authorization

Deferred. Implement bounded X25519 DH layer-1 authorization with per-client work limits and reference interoperability. Zero promotions.

### M161 — legacy AES/LS1 feasibility

Deferred. Resolve the Proposal's `encrypted (aes)` legacy contract. Zero production. If real LS1 implementation is required and acceptable, create a separate successor rather than hiding it inside M162.

### M162 — Proposal integration

Deferred. Map real completed primitives into the five server families and promote only fields whose complete valid contract is operational.

### M152 — final requalification

Deferred. Recompute all 840 cells and establish final full/safe-partial status. Zero promotions.

## 7. Promotion ceilings

From `336/29/475`:

- modern lookup/auth complete but legacy AES still a valid unsupported value: at most `346/19/475`;
- all ten `EncryptLeaseSet` values complete: at most `351/14/475`.

These remain ceilings, not claims. `SigType` ×10 and `UseOutproxyPlugin` ×4 remain independent blockers.

## 8. Completion conditions

This corrective line completes when:

- M157-M162 close according to their gates;
- modern ELS2 publication/auth/lookup primitives are implemented or truthfully blocked;
- legacy AES has an explicit final disposition;
- M152 requalifies the actual current head;
- M095/M105/registry/roadmaps agree;
- any remaining blockers are explicit architecture/security blockers rather than missing planning.
