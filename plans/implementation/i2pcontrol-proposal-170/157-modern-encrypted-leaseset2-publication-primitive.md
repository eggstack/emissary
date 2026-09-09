# M157 — Modern Encrypted LeaseSet2 Publication Primitive

Status: **registered / dependency-ready; M156 closed complete**

Class: neutral LeaseSet/NetDB infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Baseline:

- repository head at amendment research start: `1678790cc74d4075e800425c57312da89e1169fe`;
- current qualification authority: M153;
- M156 closure: `plans/closure/i2pcontrol-proposal-170/156-closure.md`;
- current M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M146 `UseOutproxyPlugin` remains blocked (4 cells);
- M147/M148 configurable Destination `SigType` remains blocked (10 cells);
- all 15 LeaseSet-security cells remain blocked before M157.

Promotion budget: **zero Proposal cells**.

## 1. Objective

Implement the smallest neutral modern type-5 Encrypted LeaseSet2 publication path on top of:

- the existing real type-7 Ed25519 Destination and signing seed;
- the existing ordinary signed LeaseSet2 produced from real inbound leases;
- M156's exact Ed25519 -> Red25519 blinding/signing primitive;
- the existing bounded destination LeaseSet publication/storage-verification state machine;
- the existing NetDB/floodfill message paths.

M157 is infrastructure only. It MUST NOT:

- implement or claim Proposal `EncryptLeaseSet`, `OptionalLookup`, or `LeaseSetClientAuths`;
- implement legacy LS1 `i2cp.encryptLeaseSet` AES;
- implement lookup-secret contribution;
- implement PSK or DH client authorization;
- make Red25519 a persistent/selectable Destination `SigType`;
- add a second NetDB subsystem or generic signature/crypto registry.

The matrix remains `336/29/475` throughout M157.

## 2. Pinned protocol contract

M157 implements the no-client-auth, no-lookup-secret modern Encrypted LS2 subset of the current I2P Encrypted LeaseSet specification.

### 2.1 Layer 0 / outer object

The DatabaseStore type byte is **5** (`EncryptedLeaseSet`). It is not part of the serialized LeaseSet payload but **is** the first byte covered by the outer Red25519 signature.

The type-5 payload contains, in order:

1. blinded public-key signature type — `u16` big-endian, exactly `11`;
2. blinded Red25519 public key — 32 bytes;
3. published timestamp — `u32` seconds since Unix epoch;
4. expires — `u16` offset from published timestamp in seconds;
5. flags — `u16`, zero in M157 (offline keys are out of scope);
6. `lenOuterCiphertext` — `u16` big-endian;
7. outer ciphertext — exact length;
8. Red25519 signature — 64 bytes over `0x05 || all preceding layer-0 payload bytes`.

M157 MUST reject nonzero/reserved flags, unsupported blinded sig types, malformed lengths, noncanonical Red25519 signatures, expired objects, and trailing/short data.

### 2.2 Layer 1 / no-auth middle layer

M157 implements only no-client-auth layer 1:

- flags byte = `0`;
- no DH/PSK auth section;
- remaining bytes = encrypted layer 2.

Any request carrying auth flags/standard auth companions is rejected before session activation until M159/M160 explicitly extend the contract.

### 2.3 Layer 2 / ordinary LS2 inner layer

Layer 2 plaintext is:

- type byte `3`;
- the complete already-signed ordinary LeaseSet2 payload produced by the existing SAM/session owner, including its header and Ed25519 signature.

M157 MUST NOT create a second ordinary LeaseSet2 builder. It reuses the canonical inner object produced from the current truthful inbound leases.

### 2.4 Credential/subcredential and encryption

Given unblinded signing public key `A`, unblinded sig type `7`, blinded sig type `11`, and current-day blinded public key `A'`:

```text
keydata       = A || 0x0007 || 0x000b
credential    = SHA-256("credential" || keydata)
subcredential = SHA-256("subcredential" || credential || A')
```

Layer 1:

```text
outerInput      = subcredential || publishedTimestamp_BE
outerSalt       = CSRNG(32)
outerKeys       = HKDF-SHA256(outerSalt, outerInput, "ELS2_L1K", 44)
outerKey        = outerKeys[0..32]
outerIV         = outerKeys[32..44]
outerCiphertext = outerSalt || ChaCha20(counter=1, outerKey, outerIV, outerPlaintext)
```

Layer 2, with no client authorization:

```text
innerInput      = subcredential || publishedTimestamp_BE
innerSalt       = CSRNG(32)
innerKeys       = HKDF-SHA256(innerSalt, innerInput, "ELS2_L2K", 44)
innerKey        = innerKeys[0..32]
innerIV         = innerKeys[32..44]
innerCiphertext = innerSalt || ChaCha20(counter=1, innerKey, innerIV, innerPlaintext)
```

The implementation MUST use the existing core HMAC/SHA-256, secure RNG, zeroization, and `crypto::chachapoly::ChaCha::with_iv()` primitive. The existing ChaCha helper already seeks to block/counter 1. M157 adds no new cryptographic dependency and does not modify `chachapoly.rs` unless a proven correctness defect forces a plan amendment.

### 2.5 Blinded storage key and UTC rollover

M156 remains authoritative for:

```text
alpha = GENERATE_ALPHA(A, current UTC YYYYMMDD, secret="")
A'    = BLIND_PUBKEY(A, alpha)
a'    = BLIND_PRIVKEY(ed25519_seed, alpha)
key   = SHA-256(0x000b || A')
```

M157 uses an empty optional blinding secret. Lookup-secret contribution is M158.

The owner MUST:

- derive the current UTC day from `Runtime::time_since_epoch()` without a new wall-clock/calendar dependency;
- derive the next UTC-day boundary deterministically;
- rotate `alpha`, blinded private/public key, and DHT storage key exactly once when the UTC day changes;
- publish the current still-valid inner LS2 under the new key even if tunnels did not otherwise change;
- reject stale old-key storage-verification replies after the switch;
- never mix an old storage key with a new blinded signature or vice versa.

A small pure integer Gregorian epoch-day conversion may live in the new M157 ELS2 helper; `primitives/date.rs` is not authorized.

## 3. Reference correction carried forward from direct Proposal-PR source

For modern Proposal modes, the Java Proposal-170 implementation selects type-5 ELS2 with `i2cp.leaseSetType=5`.

The direct `ServiceTunnelCreator` source sets:

```text
i2cp.encryptLeaseSet = true only for ENCRYPT_LEASE_SET_AES
```

and therefore sets it **false** for modern blinded/PSK/DH type-5 modes.

For M157 neutral activation:

- `i2cp.leaseSetType=5` is the selector;
- `i2cp.encryptLeaseSet=true` is treated as the separate legacy-AES contract and MUST NOT be aliased to type 5;
- this correction governs future execution even though the immutable M155 closure contains an older planning table. Historical closure evidence is not edited.

M161 remains the authority for the legacy-AES disposition; M162 must use this corrected mapping.

## 4. Exact production path amendment

M157 authorizes **only** the following production paths. No directory prefix/glob is authorized.

### 4.1 New neutral crypto helper

1. `emissary-core/src/crypto/els2.rs` — **new**

Owner: modern Encrypted LeaseSet2 protocol crypto helper.

Authorized responsibilities only:

- credential/subcredential derivation;
- RFC-5869 HMAC-SHA256 expand for the exact 44-byte ELS2 key schedules;
- no-auth layer-1/layer-2 encryption/decryption helpers needed for deterministic self-validation;
- secure 32-byte salts through caller-provided `CryptoRng`;
- UTC epoch-seconds -> `YYYYMMDD` and next-day-boundary helper;
- a zeroizing, non-`Debug` type-7 signing-seed wrapper suitable for deriving daily M156 blinded keys;
- composition of M156 types/functions without exposing Proposal vocabulary.

It MUST NOT implement PSK/DH auth, extended B32, secret persistence, generic HKDF API, or a signature-suite registry.

2. `emissary-core/src/crypto/mod.rs`

Authorized change: declaration/re-export of the neutral `els2` module/API only. Existing signing abstractions remain unchanged.

### 4.2 Wire/common-structure owner

3. `emissary-core/src/primitives/lease_set.rs`

Authorized responsibilities:

- add a bounded neutral `EncryptedLeaseSet2`/outer representation for type-5 payload data;
- strict parse/serialize framing for layer 0;
- expose published/expiry/blinded-key facts needed by floodfill/publication owners;
- verify the outer Red25519 signature through M156;
- preserve ordinary `LeaseSet2` parse/serialize bytes and behavior unchanged.

No decryption-to-Destination/client lookup is authorized here.

4. `emissary-core/src/primitives/mod.rs`

Authorized change: exact re-export of the new neutral encrypted-LS2 outer type only.

### 4.3 I2NP DatabaseStore owner

5. `emissary-core/src/i2np/database/store.rs`

Authorized responsibilities:

- add an `EncryptedLeaseSet2` payload representation for store type 5;
- parse and build type-5 DatabaseStore messages without changing type-3 semantics;
- preserve exact raw type-5 payload bytes for floodfill forwarding/storage verification;
- expose only the minimum neutral type/kind fact needed by NetDB/Destination.

No DatabaseLookup protocol expansion is authorized.

### 4.4 Existing NetDB floodfill owner

6. `emissary-core/src/netdb/mod.rs`

This path is required, despite the earlier M155 candidate list, because the current floodfill cache stores only raw LeaseSet bytes and re-emits every cached entry as `DatabaseStoreKind::LeaseSet2`. After type-5 parsing exists, leaving this owner unchanged would corrupt encrypted stores into type 3 when re-flooding or answering lookups.

Authorized responsibilities only:

- accept a valid type-5 outer object when acting as a floodfill;
- use its plaintext outer expiration for bounded cache expiry;
- preserve whether each cached raw LeaseSet object is type 3 or type 5;
- re-flood and answer lookups with the **same** DatabaseStore type and exact raw bytes;
- retain the existing opaque 32-byte DHT-key model and bounded cache/query machinery.

M157 does **not** authorize:

- decrypting ELS2 in NetDB;
- a new query engine;
- client-side blinded lookup;
- new cache cardinality/retry policy;
- any other `emissary-core/src/netdb/**` file.

For an active ordinary `QueryKind::LeaseSet`, a received type-5 object must not be misrepresented as ordinary `LeaseSet2`; M158 owns encrypted client lookup/decryption if later required.

### 4.5 Destination publication/storage-verification owner

7. `emissary-core/src/destination/lease_set.rs`

Authorized responsibilities:

- add generation-local publication mode: ordinary vs modern type-5/no-auth;
- retain the canonical ordinary signed inner LeaseSet2 bytes;
- derive/build current-day encrypted outer bytes and storage key from the M157 crypto helper;
- choose type 3/unblinded key vs type 5/blinded key for DatabaseStore publication;
- use the same current key/type for storage verification;
- integrate UTC rollover into the existing bounded `LeaseSetManager` future/state machine;
- clear/re-resolve key-scoped floodfill/query verification state when the blinded key rotates;
- preserve M135 desired-count/real-lease truthfulness and existing retry bounds;
- reject crypto/build failure rather than falling back to ordinary publication.

No second timer loop/thread/task is authorized: rollover uses one owner-local runtime timer/state in the existing future.

8. `emissary-core/src/destination/mod.rs`

Authorized responsibilities:

- carry a narrow neutral publication configuration/signing-seed wrapper from SAM session construction into `LeaseSetManager`;
- keep `SessionManager` on the ordinary inner LS2 bytes;
- route direct type-5 DatabaseStore storage-verification replies to `LeaseSetManager` without accepting the wrong type/key as proof;
- preserve the existing `DestinationEvent::CreateLeaseSet` and `publish_lease_set()` inner-LS2 lifecycle.

The existing M145 end-to-end reply-LeaseSet bundling path remains ordinary LS2. This is intentional and specification-consistent: encrypted LS2 is for floodfill publication, while an authenticated end-to-end session may carry the unencrypted LeaseSet in the wrapped garlic message. Therefore `emissary-core/src/destination/session/mod.rs` is **not** authorized for M157.

### 4.6 Neutral SAM activation/composition seams

9. `emissary-core/src/sam/parser.rs`

Authorized responsibilities:

- recognize the standard `i2cp.leaseSetType=5` activation contract before session allocation;
- for M157 type-5/no-auth mode, reject unsupported companion state before activation rather than accepting it inertly:
  - `i2cp.encryptLeaseSet=true` (legacy AES, M161);
  - nonempty `i2cp.leaseSetSecret` (M158);
  - nonzero/nonempty `i2cp.leaseSetAuthType` (M159/M160);
  - `i2cp.leaseSetPrivKey` / legacy LeaseSet key companions not consumed by no-auth type 5;
  - `i2cp.leaseSetClient.psk.*` and `i2cp.leaseSetClient.dh.*` entries;
  - unpublished/dont-publish configuration if it would make type-5 publication accept-inert.
- preserve all existing ordinary SESSION CREATE parsing/default behavior when type 5 is absent.

The parser MUST NOT interpret Proposal `EncryptLeaseSet` strings. It consumes only standard I2CP/SAM properties.

10. `emissary-core/src/sam/session.rs`

Authorized responsibilities:

- keep canonical ordinary inner LeaseSet2 construction/signing exactly at the current owner;
- when validated `i2cp.leaseSetType=5` is present, construct the narrow zeroizing M157 signing-seed/public-key publication config and pass it into `Destination`/`LeaseSetManager`;
- retain ordinary inner LS2 in `SessionManager` for end-to-end session use;
- ensure subsequent `DestinationEvent::CreateLeaseSet` renewals continue to create/sign the ordinary inner object before the publication owner wraps it.

No general signing-key ownership migration is authorized.

## 5. Explicitly unauthorized production paths

M157 does **not** authorize changes to:

- `emissary-core/src/crypto/red25519.rs` — reuse M156 as closed;
- `emissary-core/src/crypto/chachapoly.rs` — existing counter-1 ChaCha API is sufficient;
- `emissary-core/src/primitives/date.rs`;
- `emissary-core/src/error/**` — existing parse-error classes must be reused unless a demonstrated correctness issue forces amendment;
- `emissary-core/src/destination/session/mod.rs`;
- any other `emissary-core/src/netdb/**` or `emissary-core/src/i2np/**` file;
- any tunnel/transport/router/RouterInfo/frontend/startup file;
- `emissary-cli/src/i2pcontrol/**`;
- Yosemite;
- `Cargo.toml`, `emissary-core/Cargo.toml`, `Cargo.lock`, or any dependency declaration.

If implementation requires one of these paths, stop before editing and amend M157/M061/M062 with exact rationale.

## 6. Dependency posture

M157 adds **no dependency**.

Required primitives already exist in core:

- M156 `curve25519-dalek`/Red25519 blinding;
- `hmac` + `sha2` for exact HKDF-SHA256 composition;
- `chacha20` through `crypto::chachapoly::ChaCha::with_iv`;
- `rand`/`CryptoRng`;
- `zeroize`;
- `bytes`/`nom` for existing wire owners.

No Cargo or lockfile mutation is authorized.

## 7. Publication and rollover state-machine contract

### 7.1 Ordinary mode

When type 5 is absent, behavior must remain byte/semantics compatible:

```text
ordinary inner LS2
  -> DatabaseStore type 3
  -> key = Destination hash
  -> existing storage verification
```

No extra rollover timer/crypto work should run for ordinary mode.

### 7.2 Encrypted mode

When type 5 is validated before activation:

```text
ordinary signed inner LS2
  -> M157 no-auth ELS2 wrapper for current UTC day
  -> DatabaseStore type 5
  -> key = M156 blinded storage key
  -> existing bounded publication + storage verification using that key
```

The current inner LS2 remains the truth source for destination, encryption keys, leases, published timestamp, expiry, and signature.

### 7.3 UTC rollover

At the first poll/wake at or after the next UTC day boundary:

1. derive new day string/alpha/blinded keys/storage key;
2. wrap the current still-valid ordinary inner LS2 under the new day material;
3. atomically switch current publication key/object/type;
4. invalidate old-key storage-verification/floodfill-selection state;
5. start publication through the existing owner path;
6. schedule the next UTC boundary.

If the current inner LS2 is expired or invalid, do not publish stale/fabricated state; await the normal inner-LS renewal path.

A late reply for the previous key must be ignored.

## 8. NetDB type-preservation contract

After M157, a floodfill cache entry must retain at minimum:

```text
{ raw_payload, expiration, store_type = ordinary_ls2 | encrypted_ls2 }
```

The cached raw payload is never transformed between type 3 and type 5.

For type 5:

- validity/expiration comes from the verified plaintext outer header;
- the floodfill does not need the Destination or layer-decryption secret;
- a lookup for the blinded key returns the exact type-5 DatabaseStore framing;
- flooding preserves the exact type and bytes.

This is a narrow correction to the existing single owner, not a new NetDB capability surface.

## 9. Failure, cancellation, and secret handling

- Invalid standard type-5 option combinations fail before session allocation/activation.
- No requested type-5 generation may publish type 3 as fallback.
- Failed salt generation, key derivation, encryption, outer serialization/signing, or publish preparation leaves no mixed/current partial state.
- Signing seed, blinded private scalar, HKDF PRK/OKM, salts-before-publication where sensitive, and derived keys are not `Debug`/logged and are zeroized where secret.
- Public blinded key/storage key may be logged only under existing bounded public-key logging policy; default is no new logging.
- No lock spans network I/O or expensive crypto.
- Drop/shutdown cancels the owner-local rollover timer naturally; no detached rollover task exists.
- A successor generation cannot receive/use a predecessor's key state.

## 10. Exact tests/evidence required

### 10.1 Wire/crypto known-answer tests

Deterministically fix:

- Ed25519 seed/public key;
- UTC day;
- M156 alpha/blinded key;
- inner ordinary LS2 bytes;
- outer/inner salts.

Assert exact:

- credential/subcredential;
- L1/L2 HKDF outputs;
- ChaCha ciphertext;
- layer flags/lengths;
- type-5 signed preimage;
- outer Red25519 verification;
- blinded DHT storage key.

Use a pinned Java/reference fixture or independently generated reference bytes for closure. Local self-round-trip alone is insufficient interoperability evidence.

### 10.2 Parser and DatabaseStore tests

- valid type-5 payload parses and verifies;
- malformed/truncated lengths fail;
- nonzero reserved flags fail;
- bad blinded sig type fails;
- bad/noncanonical signature fails;
- type-5 DatabaseStore builder emits store type exactly 5;
- type-3 builder/parser bytes remain unchanged.

### 10.3 Floodfill preservation tests

- valid type 5 stored under opaque blinded key;
- expiry enforced from outer header;
- flood/reply preserves type 5 and exact raw payload;
- ordinary type 3 remains type 3;
- no type-5 active ordinary-query result is surfaced as a `LeaseSet2`.

### 10.4 Publication/storage-verification tests

- type5 session publishes only type5 under current blinded key;
- verification request uses the same key;
- correct type/key reply completes verification;
- wrong type, old key, or malformed reply does not complete it;
- crypto/build failure never emits ordinary DatabaseStore fallback.

### 10.5 Rollover tests

With paused/mock time:

- no early rotation;
- exact UTC boundary rotates once;
- old and new store keys differ;
- one fresh publication begins under new key even with unchanged tunnels;
- stale old-key acknowledgement is ignored;
- next boundary is scheduled;
- leap-day/month/year transitions derive correct `YYYYMMDD`;
- no rollover work exists for ordinary mode.

### 10.6 Inner-LS and M145 composition tests

- inner LS uses the current real leases and M135 desired-count behavior;
- tunnel renewal updates inner LS and encrypted outer without losing mode;
- `SessionManager` continues to carry ordinary LS2 only inside established end-to-end garlic/session wrapping;
- no public/floodfill path receives ordinary LS2 for an encrypted-mode destination;
- `shouldBundleReplyInfo=false` behavior remains unchanged.

### 10.7 Standard option gates

- `i2cp.leaseSetType=5` + no-auth/no-secret accepted;
- absent type 5 preserves ordinary behavior;
- legacy `i2cp.encryptLeaseSet=true` is not aliased to type 5;
- secret/auth/PSK/DH companions reject before activation until successor milestones;
- direct Proposal strings are absent from core source.

## 11. Verification baseline

At minimum:

```text
cargo check -p emissary-core
cargo check -p emissary-core --no-default-features --features no_std
cargo test -p emissary-core --lib --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m153_post_m146_requalification --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Also run focused M156 Red25519 tests and all new M157 wire/floodfill/publication/rollover tests under both std and no-std-compatible compilation.

## 12. M061/M062 registration amendment

The registration commit MUST simultaneously:

- add `crypto/els2.rs`, `primitives/lease_set.rs`, `primitives/mod.rs`, `i2np/database/store.rs`, and `netdb/mod.rs` as individually named M061 owner paths;
- extend the existing evidence for `crypto/mod.rs`, `destination/lease_set.rs`, `destination/mod.rs`, `sam/parser.rs`, and `sam/session.rs` with M157 purpose/reference;
- update the M061 guard's prohibited-prefix exception helper to enumerate only the exact sensitive files now accepted (`crypto/mod.rs`, `crypto/red25519.rs`, `crypto/els2.rs`, `i2np/database/store.rs`, `netdb/mod.rs`), never a prefix/glob;
- update M062 status bookkeeping to M156 closed / M157 sole registered;
- record explicitly that M157 adds no dependency/Cargo/lockfile change;
- leave M158-M162 candidate paths planning-only and unauthorized.

## 13. Acceptance criteria

M157 closes complete only when:

- every exact path in §4 is the actual changed production set, with no undeclared production path;
- no forbidden path/dependency expansion was required;
- type-5 outer/wire/crypto known-answer evidence passes;
- current Emissary floodfill behavior preserves type 5 exactly instead of rewriting it to type 3;
- encrypted publication/storage verification uses the current blinded key and never plaintext-falls back;
- UTC rollover is deterministic, bounded, owner-local, and stale-key safe;
- ordinary LS2 and M135/M145 behavior remain green;
- standard type-5 unsupported companion options fail pre-activation;
- the direct Proposal-PR mapping correction in §3 is preserved for later M162 work;
- M095 remains exactly `336/29/475`;
- no medium/high crypto/NetDB/privacy defect remains.

## 14. Stop/amend conditions

Stop before implementation or before any additional edit if:

- a production file outside §4 is required;
- a new dependency/Cargo/lockfile change appears necessary;
- type5 needs a second NetDB/query subsystem;
- public publication would require sending ordinary LS2 as fallback;
- current floodfill behavior cannot preserve type5 without a broader NetDB rewrite;
- encrypted mode requires changing M145's end-to-end session bundling contrary to the specification note that authenticated clients may receive unencrypted LS2 inside wrapped garlic;
- only serializer reachability can be demonstrated without real publication/storage verification;
- reference interoperability contradicts local vectors.

Any such finding requires an explicit M157/M061/M062 amendment before coding continues.

## 15. Closure evidence required

Record:

- implementation and closure heads;
- exact changed production/test/planning paths;
- exact M061/M062 diff;
- no-dependency/Cargo/lockfile proof;
- type-5 format + crypto KAT/reference fixture table;
- DatabaseStore/floodfill preservation evidence;
- publication/storage-verification trace;
- UTC rollover/stale-key trace;
- inner-LS/M135/M145 composition evidence;
- no-downgrade and secret-zeroization review;
- direct Proposal-PR mapping correction carried forward;
- complete verification output;
- unchanged M095 hash/counts;
- unresolved findings;
- M158 readiness disposition.

External specification/reference access remains read-only.