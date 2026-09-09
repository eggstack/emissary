# M158 — LeaseSet Lookup-Secret and Blinded-Address Primitive

Status: **registered / dependency-ready; M157 closed complete**

Class: neutral/application LeaseSet privacy infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Baseline:

- repository head at exact-path research start: `a18fba7fa307d195f2a6e7c2cae57c554b07eba8`;
- M157 closure: `plans/closure/i2pcontrol-proposal-170/157-closure.md`;
- current whole-surface qualification authority: M153;
- current M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- remaining LeaseSet blockers stay 5 `EncryptLeaseSet` + 5 `OptionalLookup` + 5 `LeaseSetClientAuths` before M158;
- M158 promotion budget: **zero Proposal cells**.

## 1. Objective

Extend the closed M156/M157 modern Encrypted LeaseSet2 path with the standard optional lookup/blinding secret and the standard encrypted-service extended `.b32.i2p` address format.

M158 is still neutral infrastructure. It proves that a real standard type-5 SAM session can:

- accept a standard `i2cp.leaseSetSecret` value without leaking it through generic option/debug surfaces;
- derive daily blinded key/storage-key material from that secret;
- publish and rotate the type-5 object through the existing M157 owner;
- expose the corresponding canonical extended B32 address for the unblinded type-7 public key with the `secret required` flag;
- recreate the same generation behavior from the same destination + standard secret input without persistent secret state inside core.

M158 MUST NOT:

- promote Proposal `OptionalLookup` yet;
- modify I2PControl production source or its secret stores;
- persist the secret in core;
- add a NetDB client lookup/decryption subsystem;
- implement PSK/DH client authorization;
- implement legacy AES/LS1;
- reopen configurable Destination `SigType`.

Field-level Proposal persistence/edit/restart semantics remain M162 work. M158 establishes the runtime primitive M162 will consume.

## 2. Pinned standard/reference contract

### 2.1 Standard property representation

The Java I2P/I2PTunnel path stores the lookup secret in standard session options as:

```text
i2cp.leaseSetSecret = Base64(UTF8(secret))
```

Direct Java evidence:

- I2PTunnel `TunnelConfig.setBlindedPassword()` writes `Base64.encode(DataHelper.getUTF8(secret.trim()))`;
- `RequestLeaseSetMessageHandler` reads `i2cp.leaseSetSecret`, Base64-decodes it, converts the result to UTF-8, and passes the resulting string into `EncryptedLeaseSet.setSecret()` **before** destination assignment;
- `ClientMessageEventListener` performs the same Base64 -> UTF-8 decode for encrypted-LS2 lookup/decryption.

M158 therefore consumes the **standard SAM/I2CP property representation** above. It does not consume the Proposal JSON plaintext field directly. M162 later maps Proposal `OptionalLookup` plaintext to this exact standard property.

Invalid Base64 or invalid UTF-8 MUST fail before session allocation/activation.

An absent or empty decoded secret means no secret and preserves the M157 unsecreted type-5 path.

Do not invent a new semantic secret-length limit. The existing bounded SAM command/input framing remains the allocation/work bound; M158 must not silently truncate otherwise-valid UTF-8 secret bytes.

### 2.2 Blinding contribution

M156 already implements the complete primitive:

```text
alpha = GENERATE_ALPHA(A, YYYYMMDD, secret_utf8_bytes)
```

and validates that `secret` is UTF-8. M158 MUST reuse `crypto/red25519.rs` unchanged.

The same secret bytes must be used for:

- current-day blinded public key;
- current-day blinded private key;
- blinded DHT storage key;
- every M157 UTC-day rollover derivation.

Changing only the secret while destination/day remain equal must change the blinded public/storage key deterministically.

### 2.3 Extended encrypted-service B32

M158 implements the current encrypted-LS B32 format for the supported type-7 -> type-11 domain.

Binary payload is exactly 35 bytes:

```text
byte 0: flags
  bit 0 = two-byte sigtypes (0 in M158)
  bit 1 = secret required
  bit 2 = per-client auth required
  bits 7..3 = zero
byte 1: unblinded sigtype = 7
byte 2: blinded sigtype   = 11
bytes 3..34: unblinded Ed25519 public key (32 bytes)
```

Checksum/post-processing:

```text
crc = CRC-32(data[3..35])          # IEEE CRC-32, Java java.util.zip.CRC32 behavior
wire[0] = flags ^ low8(crc)
wire[1] = 7     ^ low8(crc >> 8)
wire[2] = 11    ^ low8(crc >> 16)
wire[3..] = public key
hostname = I2P-Base32(wire) + ".b32.i2p"
```

For the current 35-byte supported form the encoded label is exactly 56 characters, followed by `.b32.i2p`.

Decoder requirements:

- ASCII case-insensitive hostname input;
- exact `.b32.i2p` suffix;
- exact 35-byte decoded current-domain payload / 56-character label;
- reverse CRC XOR before interpreting flags/types;
- reject reserved flag bits;
- reject two-byte sigtype flag for this milestone;
- require unblinded sigtype 7 and blinded sigtype 11;
- validate the 32-byte Ed25519 point with the existing M156/public-key validation path before treating it as an address identity;
- no trailing bytes, embedded secret, or embedded private key;
- expose `secret_required` and `auth_required` only as public address metadata.

M158 runtime emission uses `auth_required=false`; the codec may round-trip the auth-required bit now so M159/M160 can reuse the canonical address format without another codec.

No CRC crate is required: implement the small exact IEEE CRC-32 helper in the existing ELS2 module and pin it against Java/reference vectors.

## 3. Secret ownership / non-leak contract

The current M157 parser leaves generic session options inside `SamCommand::CreateSession`, whose derived `Debug` representation can include option strings. Therefore M158 MUST NOT simply permit a nonempty Base64 secret to remain in the generic `options` map.

The exact ownership correction is:

1. `sam/parser.rs` validates and removes `i2cp.leaseSetSecret` from the generic options map before creating the command;
2. it Base64-decodes and UTF-8-validates the value;
3. decoded secret bytes move into a dedicated zeroizing/non-`Debug` neutral secret type owned by `crypto/els2.rs`;
4. that secret travels inside `DestinationContext`, whose custom `Debug` already omits its fields;
5. `sam/session.rs` moves it once into `EncryptedPublicationConfig`;
6. `LeaseSetManager` owns it for the generation and uses it only for daily blinding derivation;
7. shutdown/drop zeroizes it naturally.

The generic `options` map retained by `SamSession` MUST NOT contain the secret in plaintext or Base64 form after parsing.

No log/error/metric/event may contain secret bytes or their Base64 representation.

## 4. Exact production path amendment

M158 authorizes **exactly four existing production files**. No new production source file is authorized.

### 4.1 `emissary-core/src/crypto/els2.rs`

Owner: neutral modern ELS2 crypto/address helper.

Authorized additions only:

- zeroizing `LookupSecret` type with no `Debug`/`Display`;
- secret-aware `blinded_day_material(..., secret)` using the already-secret-capable M156 `generate_alpha()`;
- canonical 35-byte/56-character type7->type11 extended-B32 encode/decode;
- exact IEEE CRC-32 helper local to the codec;
- public metadata structure for decoded `{unblinded_public_key, secret_required, auth_required}`;
- deterministic test helpers/vectors.

Existing M157 layer encryption, credential/subcredential, UTC-day conversion, and signing-seed behavior must remain unchanged except for threading the secret into the existing blinded-day call.

No new generic CRC API, generic naming subsystem, generic signature registry, or client-auth crypto is authorized.

### 4.2 `emissary-core/src/destination/lease_set.rs`

Owner: existing M157 encrypted publication/rollover state machine.

Authorized additions only:

- `EncryptedPublicationConfig` owns the optional zeroizing lookup secret for exactly one destination generation;
- current-day and rollover derivation call the secret-aware ELS2 helper;
- changing/recreating configuration from a different secret produces a clean independent key generation;
- no fallback from secret-required publication to empty-secret publication on error;
- no new timer/task/state machine beyond the M157 owner-local rollover mechanism.

No NetDB path changes are authorized: M157 already publishes/verifies under whatever blinded key this owner provides.

### 4.3 `emissary-core/src/sam/parser.rs`

Owner: SAM SESSION CREATE pre-allocation validation / secret extraction.

Authorized additions only:

- extend the M157 type-5 no-auth gate to accept absent/empty or valid Base64-encoded UTF-8 `i2cp.leaseSetSecret`;
- extract/remove that property from generic options before constructing `SamCommand`;
- place the decoded secret in `DestinationContext` using the dedicated non-`Debug` type;
- continue rejecting nonzero auth type, PSK/DH client entries, legacy AES flag, legacy LeaseSet key companions, unsupported LeaseSet types, and unpublished type-5 combinations;
- malformed Base64/UTF-8 rejects before allocation.

Do not parse Proposal `OptionalLookup` vocabulary here.

### 4.4 `emissary-core/src/sam/session.rs`

Owner: canonical session composition and public server-destination event.

Authorized additions only:

- move the extracted secret from `DestinationContext` into `EncryptedPublicationConfig` for valid type-5 publication;
- emit the canonical extended encrypted-service B32 address for every published type-5 server destination instead of the ordinary 52-character Destination-hash address;
- set extended-B32 `secret_required=true` iff the decoded lookup secret is nonempty;
- set `auth_required=false` for M158 runtime sessions;
- ordinary/non-type5 event address behavior remains byte-for-byte unchanged;
- canonical ordinary inner LS2 generation remains unchanged.

`events.rs` is not authorized: the existing event API already carries an opaque address `String` and does not need a semantic change.

## 5. Explicitly unauthorized production paths

M158 does **not** authorize changes to:

- `emissary-core/src/crypto/red25519.rs` — already supports secret bytes;
- `emissary-core/src/crypto/mod.rs` — `els2` is already public;
- `emissary-core/src/destination/mod.rs` — existing config bridge is sufficient;
- `emissary-core/src/events.rs` — existing address String seam is sufficient;
- `emissary-core/src/i2np/**`;
- `emissary-core/src/netdb/**`;
- `emissary-core/src/primitives/**`;
- `emissary-core/src/destination/session/mod.rs`;
- any router/tunnel/transport/frontend file;
- `emissary-cli/src/i2pcontrol/**`;
- `emissary-cli/src/tunnel/**`;
- Yosemite;
- Cargo manifests or `Cargo.lock`.

If implementation needs any such path, stop before editing and amend M158/M061/M062.

## 6. Dependency posture

M158 adds **no dependency**.

Reuse:

- existing I2P Base32 codec in `crypto/mod.rs`;
- M156 Red25519/blinding;
- M157 ELS2 helper and rollover owner;
- existing Base64 codec;
- existing zeroize;
- core/alloc only for CRC and address parsing.

No Cargo/lockfile/Yosemite change is authorized.

## 7. Runtime behavior

### 7.1 No-secret type-5 session

Must remain M157-compatible:

```text
i2cp.leaseSetType=5
(no secret)
  -> alpha(day, empty)
  -> blinded store key
  -> type-5 publication
  -> extended B32 with secret_required=false, auth_required=false
```

This corrects the user-facing encrypted-service address from ordinary 52-char hash form to the required 56-char extended form without changing publication crypto.

### 7.2 Secret type-5 session

```text
i2cp.leaseSetType=5
i2cp.leaseSetSecret=Base64(UTF8(secret))
  -> pre-allocation decode/extract
  -> generic option map contains no secret
  -> alpha(day, secret)
  -> different blinded public/store key
  -> type-5 publication under secret-derived key
  -> extended B32 with secret_required=true
```

The secret itself is **not** embedded in the B32 address. The address only signals that a secret is required.

### 7.3 Rollover

The same generation-local secret is used at every M157 UTC-day rollover. A rollover must never accidentally substitute empty secret.

### 7.4 Recreation / restart boundary

Core owns no persistence. A newly created SAM generation supplied with the same persistent destination and same standard property must deterministically derive the same same-day blinded identity/address flags. A generation supplied with a changed secret must derive a different blinded key while the public extended B32 remains the same except for the `secret_required` bit (which stays true for any nonempty secret).

I2PControl persistence, edit transactionality, and redacted secret-store reload are explicitly deferred to M162 because M158 has zero Proposal promotions and no I2PControl source authorization.

## 8. Extended-B32 reference requirements

Pin at least one byte-exact vector against Java `net.i2p.crypto.Blinding.encode/decode` for type 7 -> 11.

Required vectors:

- fixed type-7 public key, no secret/auth flags;
- same key with `secret_required=true`;
- same key with `auth_required=true` to prove codec future compatibility;
- exact 35 decoded bytes;
- exact 56-character label + `.b32.i2p` hostname;
- exact CRC value and XORed first three bytes.

Decoder adversarial cases:

- old 52-char B32 rejected as extended form;
- missing/wrong suffix;
- invalid Base32;
- wrong decoded length;
- reserved flag bits;
- two-byte sigtype flag;
- wrong unblinded/blinded sigtype;
- invalid/small-order Ed25519 public key;
- checksum corruption;
- trailing/extra bytes.

## 9. Secret/privacy tests

Require tests proving:

- standard property Base64 -> exact UTF-8 bytes;
- invalid Base64 rejects before allocation;
- invalid UTF-8 rejects before allocation;
- generic `SamCommand`/options debug output cannot contain the source Base64 secret;
- `DestinationContext` debug remains redacted/non-exhaustive;
- empty/absent secret reproduces M157 alpha/store key;
- nonempty secret matches direct `red25519::generate_alpha(..., secret)` known answer;
- different secrets produce different alpha/blinded/store key;
- same destination/day/secret is deterministic;
- rollover retains secret contribution;
- no-secret and secret-required publication both remain type 5;
- failure never publishes under the empty-secret key;
- drop/replacement generation does not inherit predecessor secret.

## 10. Proposal promotion decision

M158 promotes **zero** Proposal cells.

Reason:

- Proposal `OptionalLookup` still requires I2PControl field validation/mapping, persistent secret custody, edit/restart transactionality, response redaction and five-family integration;
- those are intentionally kept inside `emissary-cli/src/i2pcontrol/**` and belong to M162;
- promoting here would conflate a neutral standard runtime primitive with completed administrative semantics.

M095 therefore must remain exactly `336/29/475`.

## 11. M061/M062 registration amendment

The registration commit MUST:

- add M158 `[registered_pending]` authority for exactly the four production paths in §4;
- record zero new files;
- record exact owner evidence for each path;
- keep all four paths in the already-realized ordinary M061 allowlist from M157/M060 ancestry;
- update the M061 pending guard from its closed M157-specific assertion to exact M158 / zero-new-file assertions;
- add M062 registered-pending bookkeeping for the same four paths;
- record `new_direct_dependencies=[]`, `manifest_changes=[]`, `lockfile_change=false`, `yosemite_change=false`, `i2pcontrol_source_change=false`;
- add/adjust M062 guard logic so the registered M158 zero-dependency budget is machine-checked;
- leave M159-M162/M152 unregistered.

No historical closure file is modified.

## 12. Verification baseline

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

Also run focused M156/M157/M158 crypto, parser, publication, rollover and event-address tests.

## 13. Acceptance criteria

M158 closes complete only when:

- actual production diff is exactly the four §4 files;
- no dependency/manifest/lock/Yosemite/I2PControl source change occurred;
- no raw/Base64 lookup secret remains in generic options after SESSION CREATE parsing;
- secret-aware blinded-day/store-key derivation is exact and rollover-safe;
- no-secret M157 behavior remains unchanged except for canonical extended B32 address emission;
- secret-required sessions emit canonical extended B32 with the correct public flag and never embed the secret;
- Java/reference extended-B32 vectors interoperate byte-for-byte;
- M061/M062 guards pass with exact M158 authority;
- M095 remains `336/29/475`;
- no medium/high secret-leak, crypto, publication, or address-codec finding remains.

## 14. Stop/amend conditions

Stop and amend before implementation continues if:

- any production path outside §4 is required;
- any dependency/Cargo/lockfile/Yosemite change appears necessary;
- I2PControl secret persistence must be changed to make the neutral primitive work;
- a new NetDB resolver/query subsystem is required;
- secret data cannot be removed from generic debug-capable option state before activation;
- the extended B32 current type7->type11 format cannot be implemented without a broader naming subsystem;
- reference Java/spec vectors disagree with the planned CRC/flags/sigtype format;
- runtime failure can downgrade from secret-required blinding to empty-secret or ordinary publication.

## 15. Closure evidence

Record:

- implementation/closure heads;
- exact four production paths and all test/planning paths;
- M061/M062 registration/reconciliation diff;
- zero dependency/Cargo/lock/Yosemite/I2PControl-source proof;
- Java standard-property Base64/UTF-8 evidence;
- lookup-secret alpha/blinded/store-key known answers;
- Java/reference extended-B32 vectors and adversarial decode results;
- secret-redaction/debug audit;
- rollover/recreation traces;
- no-downgrade evidence;
- unchanged M095 hash/counts;
- unresolved findings;
- M159 readiness disposition.

External specifications/reference repositories remain read-only.