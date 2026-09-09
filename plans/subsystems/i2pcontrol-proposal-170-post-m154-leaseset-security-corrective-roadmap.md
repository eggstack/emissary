# I2PControl Proposal 170 Post-M154 LeaseSet-Security Corrective Roadmap

Status: **active / partial; M155-M159 closed, M160 deferred (hard dep satisfied, registration pending)**

This roadmap supersedes the LeaseSet-security execution ordering in the older residual and post-M146 roadmaps while preserving M146/M147/M154 closures as historical evidence.

Current authority:

- whole-surface qualification: M153;
- M095: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M146 `UseOutproxyPlugin` ×4 closed blocked;
- M154 configurable Destination `SigType` ×10 closed blocked;
- M155-M159 closed with zero Proposal promotions;
- **no registered successor; M160 deferred with satisfied hard dep, registration pending**.

## 1. Corrective architecture

Modern Encrypted LeaseSet2 is independent of the blocked general configurable-Destination-SigType path. The active line retains ordinary type-7 Ed25519 Destination identity and uses the closed type-11 Red25519 blinding/signing primitive internally.

The old M149 -> M150 -> M151 ordering remains superseded. The remaining LeaseSet fields are coupled through ten `EncryptLeaseSet` values, lookup secret, PSK/DH authorization and persistent control-plane custody. Neutral crypto/publication primitives remain zero-promotion until M162 integrates the complete Proposal field contracts.

Legacy `encrypted (aes)` stays isolated behind M161 and never aliases to modern type 5.

## 2. Frozen standard mappings

Direct Proposal/reference source establishes:

```text
legacy AES:
  i2cp.encryptLeaseSet=true

modern modes:
  i2cp.leaseSetType=5

lookup secret:
  i2cp.leaseSetSecret=Base64(UTF8(secret))

PSK:
  i2cp.leaseSetAuthType=2
  i2cp.leaseSetPrivKey=Base64(32B)
  i2cp.leaseSetClient.psk.N=Base64(UTF8(name)) + ":" + Base64(32B)

DH:
  i2cp.leaseSetAuthType=1
  i2cp.leaseSetPrivKey=Base64(32B X25519 private)
  i2cp.leaseSetClient.dh.N=Base64(UTF8(name)) + ":" + Base64(32B X25519 public)
```

Non-per-user PSK/DH modes still use the persistent base key; indexed entries are additional only for per-user modes.

## 3. Containment rules

- M061/M062/M093 remain binding.
- Proposal policy/persistence stays under `emissary-cli/src/i2pcontrol/**` where possible.
- Neutral crypto/publication code remains Proposal-free and in exact owners only.
- A milestone may use a stricter subset of already-realized M061 paths without creating a redundant waiver; plan + registry must enumerate the subset exactly.
- No broad crypto/netdb/i2np/destination/primitives permission.
- No plaintext, unsecreted or unauthenticated downgrade.
- No secret/PSK/DH/private key material in logs/debug/Get/rawConfig outside exact safe Proposal semantics.
- M147/M148 and M146 remain separate blocked lines.

## 4. Closed handoff — M159 (realized record)

M159 exact production set (realized):

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

All four are already realized M061 owners. M159 added no file/dependency/Cargo/lock/Yosemite/I2PControl production change.

M159 added standard PSK layer-1 authorization only, with:

- auth flags `0x03`;
- fresh auth cookie/auth salt per regenerated object;
- `ELS2PSKA` 52-byte HKDF schedule;
- client ID + encrypted-cookie records (duplicates preserved);
- auth-cookie-bound L2 (L1 unchanged);
- randomized multi-client order;
- pinned Java 4096-byte encrypted-data ceiling as the allocation/O(N) work bound;
- standard parser extraction/redaction of `leaseSetPrivKey` and indexed PSK properties;
- zero Proposal promotions.

Closure: `plans/closure/i2pcontrol-proposal-170/159-closure.md`.

## 5. Deferred plans reviewed/pre-corrected

### M160 — DH/X25519

Deferred with satisfied hard dep (M159 closed); registration pending. Its owner/dependency envelope revalidates cleanly to the same four exact files and zero new dependencies.

It must use existing X25519 support, standard DH flags/ephemeral-key construction, explicit all-zero shared-secret rejection, the same 4096-byte O(N) bound, randomized records, duplicate preservation, and no persistent core secret state.

M159 closure preserves these owners, so M160 may be registered directly with exact M061/M062 authorization; no generic exact-path research milestone is required.

### M161 — legacy AES/LS1 gate

Corrected dependency: **hard-depends on M160 closure**, not merely M155. It is zero-production and runs only after the modern authorization owner graph is frozen.

Possible outcomes remain A (separate exact implementation successor), B (valid but blocked under current architecture/security policy), or C (reference-incoherent/dead operational value; still blocked absent authoritative applicability correction).

### M162 — Proposal integration

Deferred until M160 + M161 disposition. Its contract is pre-corrected to require:

- exact ten-mode machine-readable mapping;
- typed Proposal fields rather than raw generic I2CP/custom smuggling;
- persistent lookup/base-key/per-user secret custody inside I2PControl;
- definition + secret-generation transactionality across create/edit/restart/delete;
- correct PSK/DH base-key semantics and per-user property formatting;
- complete-field promotion only after all valid values/uses are operational;
- no lower-layer changes; missing neutral primitive means stop/split.

### M152 — final requalification

Rebased to treat M156-M158 as closed lineage and explicitly requalify M159/M160 authenticated ELS2 bounds/negative behavior plus M162 secret transactionality. Zero production/promotions.

## 6. Dependency graph

```text
M153 current-head requalification              [CLOSED]
  |
  v
M154 SigType refreeze                          [CLOSED; M147/M148 BLOCKED]
  |
  v
M155 semantic/owner refreeze                   [CLOSED]
  |
  v
M156 Red25519/blinding                         [CLOSED]
  |
  v
M157 modern Encrypted LS2                      [CLOSED]
  |
  v
M158 lookup-secret + blinded address           [CLOSED]
  |
  v
M159 PSK client authorization                  [CLOSED]
  |
  v
M160 DH/X25519 client authorization            [DEFERRED; HARD DEP SATISFIED, REGISTRATION PENDING]
  |
  v
M161 legacy AES/LS1 feasibility                [DEFERRED; ZERO PRODUCTION]
  |
  v
M162 Proposal field integration                [DEFERRED; CONDITIONAL PROMOTIONS]
  |
  v
M152 final whole-surface requalification       [DEFERRED; ZERO PROMOTIONS]
```

No milestone is executable now; only M160 may be registered next.

## 7. Promotion ceilings

From `336/29/475`:

- OptionalLookup + LeaseSetClientAuths fully operational while legacy AES keeps EncryptLeaseSet blocked: at most `346/19/475`;
- complete `EncryptLeaseSet` enum domain too: at most `351/14/475`.

These are ceilings, not claims. SigType ×10 and UseOutproxyPlugin ×4 remain independent blockers.

## 8. Completion conditions

This line completes only when:

- M159-M162 close according to their security/interop gates;
- legacy AES has a recorded disposition;
- M152 requalifies the actual head;
- M095/M105/registry/roadmaps agree;
- any residual blockers have explicit accepted architecture/security dispositions rather than missing planning.