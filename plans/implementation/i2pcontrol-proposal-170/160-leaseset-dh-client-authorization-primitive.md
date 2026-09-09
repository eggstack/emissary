# M160 — Encrypted LeaseSet DH Client-Authorization Primitive

Status: **registered / dependency-ready; M159 closed complete**

Class: neutral modern Encrypted LeaseSet2 client-authorization infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Baseline:

- M159 implementation/closure head: `c0bbf9d4` (verify with `git log --oneline -3` at implementation start and record the exact SHA in the closure);
- M159 closure: `plans/closure/i2pcontrol-proposal-170/159-closure.md`;
- current whole-surface qualification authority: M153;
- current M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M160 Proposal-promotion budget: **zero cells**.

## 1. Objective

Extend the closed M157/M158/M159 type-5 publication owner with the standard DH (X25519) client-authorization layer used by Encrypted LeaseSet2, while retaining the same bounded publication/storage-verification/UTC-rollover owner and without adding Proposal/I2PControl policy to core.

M160 proves the neutral standard SAM/session path for:

- `i2cp.leaseSetType=5`;
- `i2cp.leaseSetAuthType=1` (DH in the pinned Java I2CP property contract);
- required `i2cp.leaseSetPrivKey` 32-byte Base64 X25519 private key material;
- optional contiguous `i2cp.leaseSetClient.dh.N` 32-byte per-client X25519 public keys;
- optional M158 `i2cp.leaseSetSecret` in combination with DH auth;
- authenticated layer-1 construction with fresh ephemeral X25519 keypair and fresh auth cookie per publication;
- `auth-required` extended-B32 emission;
- UTC rollover and ordinary LeaseSet regeneration without confidentiality/authentication downgrade.

M160 does **not** implement Proposal `LeaseSetClientAuths` persistence or promote that field. Proposal JSON parsing, names, persistent secret/key custody, create/edit/restart transactions and five-family backend integration remain M162 work.

## 2. Standard property contract

Pinned Java behavior:

```text
i2cp.leaseSetType = 5
i2cp.leaseSetAuthType = "1"             # DH
i2cp.leaseSetPrivKey = Base64(32B X25519 private key)
i2cp.leaseSetClient.dh.N = [Base64(UTF8(name)) ":"] Base64(32B X25519 public key)
```

Java includes the public key derived from `leaseSetPrivKey` in the authorized client set and then appends indexed DH client public keys. Therefore a DH generation requires the base private key but need not require an indexed per-user entry.

Pinned Java appends configured client entries without deduplicating equal public-key bytes. M160 must preserve duplicate configured entries, each consuming one record and the bounded 4096-byte work budget; do not silently deduplicate.

Core consumes the standard property representation only; Proposal-layer names, key generation/persistence and edit/restart transactions remain M162.

## 3. Exact DH wire/crypto contract

Use the pinned Encrypted LeaseSet specification / Java `EncryptedLeaseSet` construction:

- layer-1 auth flags = `0x01` for DH;
- one fresh ephemeral X25519 keypair per regenerated ELS2;
- layer-1 header carries the 32-byte ephemeral public key and `u16` client count;
- for each authorized client public key `cpk_i`:
  - `shared_i = X25519(ephemeral_private, cpk_i)`;
  - reject an all-zero shared secret;
  - `authInput_i = shared_i || cpk_i || subcredential || published_BE`;
  - `okm_i = HKDF-SHA256(ephemeral_public, authInput_i, "ELS2_XCA", 52)`;
  - client key = first 32 bytes, IV = next 12, client ID = final 8;
  - encrypt the common fresh 32-byte auth cookie with ChaCha20;
- layer-2 input is `authCookie || subcredential || published_BE`;
- layer-1 outer key input remains `subcredential || published_BE`.

Fresh auth cookie and ephemeral keypair are generated for every regenerated authenticated outer object. Multi-client record order is randomized.

## 4. Exact work/size bound

Reuse M159's pinned Java 4096-byte authenticated encrypted-data ceiling. Before any per-client X25519 operation, use checked arithmetic to ensure the complete encrypted-data object can fit within 4096 bytes.

The 40-byte/client record size plus fixed framing gives a strict finite upper bound on DH operations; no arbitrary independent count limit is needed. Never truncate or deduplicate clients to fit.

## 5. Exact production-path freeze

M160 may modify exactly these four existing production files:

1. `emissary-core/src/crypto/els2.rs`
2. `emissary-core/src/destination/lease_set.rs`
3. `emissary-core/src/sam/parser.rs`
4. `emissary-core/src/sam/session.rs`

All four are already individually present in M061's realized exact source boundary from M157/M158/M159. M159 closed within this exact owner set, so M160 creates **no new M061 path waiver**.

No production file outside this four-file subset is authorized.

Explicitly forbidden without amendment:

- `emissary-core/src/crypto/red25519.rs` or `crypto/mod.rs`;
- `emissary-core/src/destination/mod.rs` or `destination/session/mod.rs`;
- `emissary-core/src/primitives/**`, `i2np/**`, `netdb/**`;
- router/tunnel/transport/event files;
- `emissary-cli/src/i2pcontrol/**` or `emissary-cli/src/tunnel/**`;
- Yosemite.

## 6. Dependency freeze

M160 adds **no dependency**. Reuse only the already accepted `x25519-dalek`
(workspace `3.0.0-pre.6`, features `reusable_secrets`/`static_secrets`/
`zeroize`/`precomputed-tables`, already in `emissary-core/Cargo.toml`),
HMAC-SHA256, ChaCha20, RNG, zeroize, Base64, M156 Red25519, and
M157/M158/M159 ELS2/blinding/B32/PSK stack. No second curve/X25519
implementation.

Registration revalidation (observed, per the §Registration-rule gate):

- no Cargo/lockfile/Yosemite/feature change required: ephemeral keygen via
  `x25519_dalek::StaticSecret::random_from_rng`, public derivation via
  `PublicKey::from`, shared-secret bytes via the existing
  `crypto::SecretKey::diffie_hellman` seam — all without a new module;
- all-zero shared-secret rejection is explicit at the ELS2 boundary
  (frozen contract below).

M160 authorizes no Cargo manifest, lockfile, Yosemite, feature, I2PControl production-source, or new-source-file change. M062 records this exact zero-dependency budget.

## 7. Secret ownership

- parser extracts and removes `leaseSetPrivKey` and all DH client-key values from generic debug-capable options before activation (same redaction seam M159 built for PSK);
- base private key and ephemeral private key use zeroizing/non-`Debug` wrappers;
- indexed client names are stripped and never retained by core;
- authorized client public keys, including duplicates, are retained generation-locally only and never become response-facing via this primitive;
- core persists nothing; M162 owns persistent key generation/custody and transactionality.

Sparse indexed keys, malformed Base64, wrong lengths, mixed PSK/DH properties and unsupported auth types fail before allocation. Duplicate public-key entries are preserved as the reference does and tested explicitly. DH mode and M159 PSK mode are mutually exclusive: PSK entries in a DH request (and DH entries in a PSK request) fail closed.

## 8. Publication/session behavior

Reuse the M159 publication owner and configuration shape rather than adding another scheduler/state machine. Day rollover, inner-LS renewal, retry rebuild and stale-generation rules are identical to M159. Authenticated generations never fall back to no-auth/PSK/type3. The session bridge gains a third type-5 mode (DH) alongside the closed no-auth and PSK modes; all three keep the existing opaque event address seam with `auth_required=true` for DH.

M158 extended-B32 emission sets `auth_required=true`; `secret_required` remains independently controlled by the optional lookup secret.

## 9. Tests and interoperability

Require deterministic KATs with fixed destination/day/inner LS2/outer salt/inner salt/ephemeral key/auth cookie and one/multiple DH clients, independently checking subcredential, `ELS2_XCA` 52-byte outputs, client IDs, encrypted auth cookies, L2 key/IV with auth cookie, complete authenticated ciphertext, and outer Red25519 verification.

Production ordering must be RNG-driven when `N>1`; deterministic test ordering must prove order-independence of authorization semantics. A duplicate-entry test must prove duplicates are preserved as separate records and remain bounded by the same 4096-byte ceiling.

Negative tests must cover missing base key, malformed/non-32-byte Base64, sparse indexed entries, mixed PSK/DH entries, wrong auth type, wrong client/no match, all-zero shared secret, maintained-library malformed/low-order public keys, tampered ephemeral key/ID/cookie/flags, wrong lookup secret, checked-size overflow, >4096 encrypted data, stale day/material, and any attempted no-auth fallback.

Reference interoperability must prove Emissary DH output is consumable under pinned Java/reference semantics, or reproduce/decrypt a pinned reference fixture through an independent path. Deterministic reference randomness is not required if cross-implementation derivation/decryption evidence is stronger. No production client resolver is authorized.

Also run focused M157/M158/M159 regressions for no-auth type 5, lookup-secret type 5, PSK type 5, B32 flags/CRC, UTC rollover, type-preserving NetDB storage/flooding, and ordinary type-3 LS2.

## 10. Verification

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

## 11. Proposal-support disposition

M160 has zero promotion budget. `LeaseSetClientAuths`, `EncryptLeaseSet` and Proposal-layer `OptionalLookup` remain blocked until M162 integrates the full five-family persistent control-plane contract.

## 12. Registration rule (satisfied by this registration)

1. M159 closure (`plans/closure/i2pcontrol-proposal-170/159-closure.md` §10) confirms the four-file owner graph remains valid;
2. the existing X25519 dependency is sufficient with no manifest/lock change (see Dependency freeze);
3. M062 is updated from M159 to M160 with the same zero-dependency envelope in this registration commit;
4. registry/roadmap/AGENTS advance only M160 in this registration commit.

No additional generic exact-path research milestone was required.

## 13. Stop conditions

Stop/amend if M160 needs a fifth production file, new dependency, I2PControl change, another publication owner, unbounded DH work, a client-side NetDB subsystem, or any confidentiality/authentication downgrade.

## 14. Closure evidence

Record exact realized paths/dependencies, duplicate-preservation behavior, 4096-byte/O(N) bound proof, X25519 low-order/all-zero handling, KAT/reference interoperability, secret custody/redaction, zero matrix delta, implementation SHA, M061/M062 evidence, and M161/M162 readiness.