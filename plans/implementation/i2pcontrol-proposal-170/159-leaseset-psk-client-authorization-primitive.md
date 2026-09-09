# M159 — Encrypted LeaseSet PSK Client-Authorization Primitive

Status: **registered / dependency-ready; M158 closed complete**

Class: neutral modern Encrypted LeaseSet2 client-authorization infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Baseline:

- M158 implementation/closure head: `57473dc4d729e1a298471b4e20fc062df19f7aeb`;
- M158 closure: `plans/closure/i2pcontrol-proposal-170/158-closure.md`;
- current whole-surface qualification authority: M153;
- current M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M159 Proposal-promotion budget: **zero cells**.

## 1. Objective

Extend the closed M157/M158 type-5 publication owner with the standard PSK client-authorization layer used by Encrypted LeaseSet2, while retaining the same bounded publication/storage-verification/UTC-rollover owner and without adding Proposal/I2PControl policy to core.

M159 proves the neutral standard SAM/session path for:

- `i2cp.leaseSetType=5`;
- `i2cp.leaseSetAuthType=2` (PSK in the pinned Java I2CP property contract);
- required `i2cp.leaseSetPrivKey` 32-byte Base64 key material;
- optional contiguous `i2cp.leaseSetClient.psk.N` 32-byte per-client entries;
- optional M158 `i2cp.leaseSetSecret` in combination with PSK auth;
- authenticated layer-1 construction with fresh auth cookie/salt per publication;
- `auth-required` extended-B32 emission;
- UTC rollover and ordinary LeaseSet regeneration without confidentiality/authentication downgrade.

M159 does **not** implement Proposal `LeaseSetClientAuths` persistence or promote that field. Proposal JSON parsing, names, persistent secret/key custody, create/edit/restart transactions and five-family backend integration remain M162 work.

## 2. Pinned reference contract

### 2.1 Standard Java I2CP properties

Pinned Java `RequestLeaseSetMessageHandler` defines:

```text
i2cp.leaseSetAuthType = "2"          # PSK
i2cp.leaseSetPrivKey = Base64(32B)   # required base PSK
i2cp.leaseSetClient.psk.N = [Base64(UTF8(name)) ":"] Base64(32B)
```

The Proposal-170 PR's `ServiceTunnelCreator` maps PSK modes as follows:

- modern mode selector: `i2cp.leaseSetType=5`;
- PSK auth selector: `i2cp.leaseSetAuthType=2`;
- `addLeaseSetPrivKey(..., true)` creates/reuses one 32-byte Base64 key in `i2cp.leaseSetPrivKey`;
- per-user PSK modes additionally write indexed `i2cp.leaseSetClient.psk.N` values as `Base64(UTF8(name)) + ':' + Key`;
- non-per-user PSK modes still authorize the base `i2cp.leaseSetPrivKey` key and therefore MUST NOT require indexed entries.

M159 consumes this standard representation only. It does not generate Proposal-layer names or persistent keys.

### 2.2 Layer-1 PSK format

Pinned Java `EncryptedLeaseSet` and the Encrypted LeaseSet specification establish:

```text
outerSalt[32]
ChaCha20_L1(
    flags[1] = 0x03,
    authSalt[32],
    clientCount[u16 BE],
    repeated clientCount times:
        clientID[8],
        encryptedAuthCookie[32],
    innerSalt[32],
    ChaCha20_L2(type=0x03 || signedInnerLeaseSet2)
)
```

`0x03` is the standard PSK client-auth flags value: per-client authorization plus the PSK scheme bits.

For every publication generation:

```text
authCookie = CSRNG(32)
authSalt   = CSRNG(32)

pskInput_i = psk_i || subcredential || published_BE
okm_i      = HKDF-SHA256(authSalt, pskInput_i, "ELS2PSKA", 52)
clientKey  = okm_i[0..32]
clientIV   = okm_i[32..44]
clientID   = okm_i[44..52]
encryptedAuthCookie_i = ChaCha20(clientKey, clientIV, authCookie)

layer2Input = authCookie || subcredential || published_BE
layer2      = HKDF-SHA256(innerSalt, layer2Input, "ELS2_L2K", 44)
layer1Input = subcredential || published_BE
layer1      = HKDF-SHA256(outerSalt, layer1Input, "ELS2_L1K", 44)
```

The auth cookie MUST affect the inner layer and MUST NOT affect the outer layer.

### 2.3 Exact size/work bound

Pinned Java `EncryptedLeaseSet.MAX_ENCRYPTED_SIZE` is 4096 bytes. M159 adopts that authenticated encrypted-data ceiling instead of inventing an arbitrary client-count limit.

Before any per-client HKDF/ChaCha work, compute the complete encrypted-data size with checked arithmetic:

```text
32 outerSalt
+ 1 flags
+ 32 authSalt
+ 2 clientCount
+ 40 * N client records
+ 32 innerSalt
+ 1 inner type
+ signedInnerLeaseSet2.len()
<= 4096
```

This simultaneously bounds allocation and PSK work. `N` must also fit `u16`, but the 4096-byte bound is stricter. If the current inner LeaseSet leaves insufficient room for all configured keys, activation/publication fails closed; entries are never silently dropped or truncated.

## 3. Exact production-path freeze

M159 may modify exactly these four existing production files:

1. `emissary-core/src/crypto/els2.rs`
2. `emissary-core/src/destination/lease_set.rs`
3. `emissary-core/src/sam/parser.rs`
4. `emissary-core/src/sam/session.rs`

All four are already individually present in M061's realized exact source boundary from M157/M158. M159 therefore creates **no new M061 path waiver**.

No production file outside this four-file subset is authorized.

Explicitly forbidden without amendment:

- `emissary-core/src/crypto/red25519.rs` or `crypto/mod.rs`;
- `emissary-core/src/destination/mod.rs` or `destination/session/mod.rs`;
- `emissary-core/src/primitives/**`, `i2np/**`, `netdb/**`;
- router/tunnel/transport/event files;
- `emissary-cli/src/i2pcontrol/**` or `emissary-cli/src/tunnel/**`;
- Yosemite.

## 4. Dependency freeze

M159 adds **no dependency**. Reuse only the already accepted HMAC-SHA256, ChaCha20, RNG, zeroize, Base64, M156 Red25519, and M157/M158 ELS2/blinding/B32 stack.

M159 authorizes no Cargo manifest, lockfile, Yosemite, feature, I2PControl production-source, or new-source-file change. M062 records this exact zero-dependency budget.

## 5. Secret/key ownership

### 5.1 Dedicated neutral types

`crypto/els2.rs` must introduce or extend narrow non-`Debug`, zeroizing types for one 32-byte PSK and the generation-local bounded PSK authorization set.

The base `leaseSetPrivKey` PSK is always the first logical authorized key. Indexed per-user keys follow. Runtime does not need or retain user names.

### 5.2 Parser extraction

`sam/parser.rs` owns fail-before-allocation extraction. For a type-5 PSK request it must:

1. require auth type exactly `2`;
2. require `i2cp.leaseSetPrivKey` to decode from Base64 to exactly 32 bytes;
3. scan indexed `i2cp.leaseSetClient.psk.N` entries from zero;
4. permit an optional single `name:` prefix and validate the suffix as Base64 exactly 32 bytes;
5. reject malformed suffixes, index gaps followed by later entries, mixed DH entries, unsupported auth selectors, or incompatible companion keys;
6. reject duplicate PSK bytes rather than emit duplicate client IDs;
7. enforce a checked pre-allocation absolute entry ceiling derived from the 4096-byte minimum framing and re-check exact serialized size before per-key crypto;
8. remove `leaseSetPrivKey` and all PSK entry values from generic debug-capable options before constructing `SamCommand` or retained session state.

The optional M158 lookup secret remains in its existing dedicated zeroizing handoff and may coexist with PSK auth. Names are not retained in core; Proposal-layer names remain M162 state.

### 5.3 No persistence in core

Core does not persist PSKs. A new standard SAM generation must supply the same standard key properties to recreate the authorization set. Persistent key creation/custody and transactional edit/restart behavior are M162 authority.

## 6. Crypto owner work

`crypto/els2.rs` owns the neutral wire/crypto extension:

- exact `ELS2PSKA` HKDF-SHA256 52-byte derivation;
- fresh 32-byte auth cookie and auth salt from `CryptoRng` for every regenerated outer object;
- exact client-ID and encrypted-cookie layout;
- auth-cookie-bound L2 derivation;
- unchanged M157/M158 L1 derivation;
- deterministic fixed-cookie/fixed-salt construction only for tests or clearly test-oriented helpers;
- zeroization of PSKs, auth cookie, derived client keys/IVs and temporary plaintext where retained;
- strict checked length arithmetic before allocation/crypto;
- a decrypt/selection helper sufficient for KATs and negative tests, not a production NetDB lookup subsystem.

The no-auth M157 path and lookup-secret-only M158 path must remain byte/behavior compatible.

## 7. Publication owner work

`destination/lease_set.rs` retains sole floodfill publication authority. Extend `EncryptedPublicationConfig`/owner-local state so one generation may carry signing material, M158 optional lookup secret, and optional PSK authorization.

For PSK mode:

- every regenerated encrypted outer object uses a fresh auth cookie/auth salt and freshly randomized client-record order when more than one key exists;
- daily blinding/store key continues to use the M158 lookup secret exactly;
- storage verification remains keyed to the current blinded day;
- build failure never falls back to no-auth or type 3;
- stale generations cannot publish superseded PSK material;
- no in-place cross-generation mutable key list or second publication scheduler is introduced.

## 8. SAM session composition

`sam/session.rs` may only consume the dedicated parser-owned PSK config, pass it into the existing `EncryptedPublicationConfig` bridge, and emit the M158 extended B32 with `auth_required=true` while `secret_required` independently reflects M158 lookup-secret state.

Raw PSK material must never enter event/log/debug/public destination strings. The existing event API remains unchanged.

## 9. Interoperability and tests

Require deterministic KATs with fixed destination/day/inner LS2/outer salt/inner salt/auth salt/auth cookie and one/multiple PSKs, independently checking:

- subcredential;
- `ELS2PSKA` 52-byte outputs;
- client IDs;
- encrypted auth cookies;
- L2 key/IV with auth cookie;
- complete authenticated ciphertext;
- outer Red25519 verification.

Production ordering must be RNG-driven when `N>1`; deterministic test ordering must prove order-independence of authorization semantics.

Negative tests must cover missing base key, malformed/non-32-byte Base64, duplicate PSKs, sparse indexed entries, mixed PSK/DH entries, wrong auth type, wrong PSK/no client match, tampered ID/cookie/auth salt/flags, wrong lookup secret, checked-size overflow, >4096 encrypted data, stale day/material, and any attempted no-auth fallback.

Reference interoperability must prove Emissary PSK output is consumable under pinned Java/reference semantics, or reproduce/decrypt a pinned reference fixture through an independent path. Deterministic reference randomness is not required if cross-implementation derivation/decryption evidence is stronger. No production client resolver is authorized.

## 10. Proposal-support disposition

M159 promotion budget is **zero**. At closure M095 must remain `336/29/475`; all five `LeaseSetClientAuths`, five `EncryptLeaseSet`, and five `OptionalLookup` Proposal cells remain blocked.

Reason: M159 establishes neutral standard PSK runtime support only. Proposal mapping/names/persistent custody/edit/restart/Get-redaction/five-family composition remain M162, and DH remains M160.

## 11. Verification

Run at minimum:

```bash
cargo fmt --all -- --check
cargo check -p emissary-core
cargo check -p emissary-core --no-default-features --features no_std
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m105_residual_option_audit
```

Also run focused M157/M158 regressions for no-auth type 5, lookup-secret type 5, B32 flags/CRC, UTC rollover, type-preserving NetDB storage/flooding, and ordinary type-3 LS2.

## 12. Stop conditions

Stop and amend before editing if implementation requires:

- any production file outside the exact four-file set;
- any new dependency/manifest/lockfile/Yosemite change;
- I2PControl production changes;
- a new NetDB/query/decryption subsystem;
- a second publication owner/timer;
- generic debug-capable PSK storage;
- authenticated encrypted data larger than the pinned 4096-byte reference ceiling;
- acceptance of malformed/sparse/mixed auth properties;
- plaintext, unsecreted, or no-auth fallback after PSK activation.

If correct interoperability conflicts with this frozen contract, stop and amend rather than approximate.

## 13. Closure evidence

Create `plans/closure/i2pcontrol-proposal-170/159-closure.md` recording implementation SHA/exact paths, standard-property parsing, custody/redaction audit, 4096-byte work-bound evidence, KAT/reference interop, randomized-order and negative tests, M157/M158/ordinary regressions, M061/M062/no-dependency evidence, no-std/lint results, exact `336/29/475`, zero-promotion attestation, and whether M160's pre-frozen envelope remains valid.

On successful closure, remove M159 registration and advance only M160 if its pre-frozen envelope still matches the realized owner graph.