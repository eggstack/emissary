# M155 — Post-M154 LeaseSet-Security Semantic and Exact-Owner Re-freeze

Status: **closed as complete**
(see `plans/closure/i2pcontrol-proposal-170/155-closure.md`)

Class: invariant / architecture-security qualification

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Baseline:

- repository head at planning start: `5a9754cdd2226ed23336feb61e6fa77118d2939e`;
- current qualification authority: M153;
- current M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M146 `UseOutproxyPlugin` remains blocked (4 cells);
- M147/M148 configurable Destination `SigType` path remains blocked (10 cells);
- M149-M151 are superseded for execution by this corrective line until their assumptions are re-frozen.

Promotion budget: **zero Proposal cells**.

Production budget: **zero production Rust/dependency/Yosemite changes**.

## 1. Objective

Re-freeze the remaining LeaseSet-security contract after M154 proved that configurable Destination `SigType` cannot be the prerequisite for Encrypted LeaseSet2 work.

M155 must determine the smallest truthful continuation that separates:

1. configurable persistent Destination signature-suite support (`SigType`) from the narrower Ed25519-to-Red25519 blinding/signing primitive required by Encrypted LeaseSet2;
2. modern Encrypted LeaseSet2 type-5 publication from legacy LS1 `i2cp.encryptLeaseSet` AES behavior;
3. `EncryptLeaseSet`'s ten-value Proposal domain from the independently consumable `OptionalLookup` and `LeaseSetClientAuths` fields;
4. publication-side secret/auth behavior from any unnecessary generic client-side NetDB subsystem.

No implementation may begin from this plan. M155 exists to freeze exact semantics, exact files, dependencies, test vectors, and the corrected dependency graph.

## 2. Pinned reference facts to verify and record

M155 must record direct read-only evidence for each statement below against the pinned Java/I2P snapshot and Proposal-170 PR:

### 2.1 Proposal `EncryptLeaseSet` value domain

Freeze all ten accepted strings and their exact Java property mapping:

- `disable`;
- `encrypted (aes)`;
- `blinded`;
- `blinded with lookup password`;
- `encrypted (psk)`;
- `encrypted with lookup password (psk)`;
- `encrypted with per-user key (psk)`;
- `encrypted with lookup password and per-user key (psk)`;
- `encrypted with per-user key (dh)`;
- `encrypted with lookup password and per-user key (dh)`.

For every value record:

- `i2cp.encryptLeaseSet`;
- `i2cp.leaseSetType`;
- `i2cp.leaseSetSecret`;
- `i2cp.leaseSetAuthType`;
- `i2cp.leaseSetKey` / `i2cp.leaseSetPrivKey`;
- per-client PSK/DH properties;
- required destination signing type constraints;
- actual LeaseSet wire/storage format.

### 2.2 Legacy AES contradiction

Freeze whether `encrypted (aes)` is an operational contract in the pinned Java runtime or a legacy LS1-only path that is incoherent with current forced-LS2 behavior.

The audit must reconcile, from source rather than inference:

- Proposal PR maps this mode to `i2cp.encryptLeaseSet=true` without type-5 LS2;
- current Java `requiresLS2()` behavior;
- `LeaseSet.encrypt(SessionKey)` legacy LS1 behavior;
- `LeaseSet2.encrypt(SessionKey)` behavior;
- whether an actual current I2PTunnel service using this value publishes a valid encrypted object;
- whether implementing it in Emissary would require adding a legacy LeaseSet1 publication stack.

Allowed dispositions:

A. operational and bounded — exact LS1 implementation requirements are frozen for a later milestone;
B. reference-incoherent/dead compatibility value — leave `EncryptLeaseSet` blocked unless an authoritative Proposal correction permits another disposition;
C. unresolved — leave `EncryptLeaseSet` blocked and stop any plan that assumes full value-domain completion.

M155 MUST NOT silently reclassify or remove the value.

### 2.3 Narrow Red25519 requirement

Freeze that modern Encrypted LS2 can use the already-supported type-7 Ed25519 Destination and derive a type-11 Red25519 blinded key without making type 11 a persistent user-selectable Destination `SigType`.

Record exact formulas/vectors for:

- daily alpha derivation;
- Ed25519 scalar conversion/reduction;
- blinded public key `A + alpha*B`;
- blinded private scalar `(a + alpha) mod L`;
- Red25519 randomized signing;
- verification;
- blinded DHT storage key;
- UTC-day rollover;
- optional blinding secret contribution.

M155 must explicitly state that this primitive does not reopen M147/M148 and does not authorize DSA/P256/P384/P521 generation.

### 2.4 Field coupling

Freeze which `EncryptLeaseSet` modes consume:

- `OptionalLookup`;
- `LeaseSetClientAuths` PSK entries;
- `LeaseSetClientAuths` DH entries.

Determine whether `OptionalLookup` and `LeaseSetClientAuths` can truthfully promote independently once their own complete semantics are operational even if `EncryptLeaseSet` remains blocked because one enum value is unavailable.

## 3. Current Emissary owner audit

Record exact current files and relevant functions for:

- type-7 signing private/public keys;
- ordinary LeaseSet2 parse/serialize/sign;
- Destination LeaseSet construction and publication;
- DatabaseStore parsing/building/store-type encoding;
- LeaseSet storage verification lookup key;
- destination/server secret stores and generation transactionality;
- standard SAM/Yosemite session-option transport;
- X25519, HKDF/HMAC/SHA256, ChaCha20 and secure RNG primitives already available;
- any current extended `.b32.i2p` support.

For each future change classify it as:

- reusable existing primitive;
- new neutral primitive in an existing exact owner;
- I2PControl-only policy/storage mapping;
- prohibited/unnecessary broad rewrite.

## 4. Dependency/security freeze

M155 must inspect maintained Rust crypto options for the narrow Red25519 operation, prioritizing:

1. direct use of an already-transitive maintained curve25519 implementation at an exact compatible version;
2. `no_std + alloc` compatibility;
3. audited constant-time scalar/point operations;
4. no generic hazmat exposure to I2PControl policy;
5. no bespoke bignum/curve implementation.

If a new direct dependency is required, record exact package/version/features, transitive impact, license, maintenance posture and why existing dependencies cannot expose the needed safe operations.

No dependency is added in M155.

## 5. Corrected successor decomposition to validate

Expected successor chain, subject to M155 evidence:

```text
M155 semantic/owner refreeze                 [zero production]
  |
  v
M156 narrow Red25519 + Ed25519 blinding      [neutral primitive; zero promotion]
  |
  v
M157 modern Encrypted LS2 publication        [neutral publication; zero promotion]
  |
  v
M158 lookup-secret + blinded-address support [neutral/application primitive]
  |
  v
M159 PSK client-authorization primitive      [neutral crypto; zero promotion]
  |
  v
M160 DH client-authorization primitive       [neutral crypto; zero promotion]
  |
  +--> M161 legacy AES/LS1 feasibility       [zero promotion unless later implementation plan]
  |
  v
M162 Proposal LeaseSet-field integration     [conditional promotions]
  |
  v
M152 final residual requalification          [zero promotion]
```

If evidence shows a smaller or safer split, M155 must amend the roadmap before registering M156.

## 6. Exact-path freeze requirement

Before M156 may be registered, M155 closure must list exact files for M156 and exact candidate files for M157-M160. Broad prefixes such as `emissary-core/src/crypto/**`, `destination/**`, `netdb/**` or `i2np/**` are forbidden.

Planning-only paths may be added to M062 now. Production paths must be authorized only when the corresponding successor is registered.

## 7. Verification / evidence

M155 closure must include:

- exact ten-mode table;
- legacy AES disposition with direct source evidence;
- Red25519/blinding formula table and at least one cross-reference vector source;
- current Emissary owner/capability table;
- dependency review;
- corrected field-coupling table;
- exact proposed files for M156;
- decision whether M149-M151 are fully superseded;
- unchanged mechanically recomputed `336/29/475` matrix;
- zero production/dependency diff proof;
- registry/roadmap/M062 reconciliation.

## 8. Stop conditions

Close blocked and register no implementation successor if:

- modern Encrypted LS2 requires configurable Destination SigType beyond type 7/11 blinding;
- no maintained constant-time Rust primitive can implement required Red25519/blinding safely;
- exact Encrypted LS2 format cannot be frozen;
- the only implementation path requires a broad unbounded crypto/NetDB rewrite;
- Proposal field coupling remains too ambiguous to define truthful promotion criteria.

## 9. Acceptance criteria

M155 closes complete only when the continuation can be handed to an implementation agent without guessing about mode semantics, crypto formulas, exact owners, dependency policy, legacy AES disposition, or matrix promotion rules. M155 itself changes no production behavior and promotes no cells.