# M160 — Encrypted LeaseSet DH Client-Authorization Primitive

Status: **deferred / unregistered; M159 hard dependency satisfied, registration pending**

> M159 closed complete (`plans/closure/i2pcontrol-proposal-170/159-closure.md`),
> satisfying this milestone's hard dependency. The pre-frozen envelope
> revalidates cleanly (same four exact files, existing `x25519-dalek`
> sufficient, zero-dependency budget intact). Registration must still be
> authored in a separate commit with exact M061/M062 authorization; this
> file is otherwise unchanged and no production work is authorized by this
> note.

Class: neutral modern Encrypted LeaseSet2 client-authorization infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

## Objective

Implement the standard Encrypted LeaseSet2 X25519/DH client-authorization mode on the same owner graph as M159, independently of Proposal parsing/persistence.

M160 is intentionally pre-frozen now so successful M159 closure can advance it without another open-ended exact-path research milestone. Registration must only revalidate that M159 did not move the accepted owners or dependency surface.

## Standard property contract

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

## Exact DH wire/crypto contract

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

## Exact work/size bound

Reuse M159's pinned Java 4096-byte authenticated encrypted-data ceiling. Before any per-client X25519 operation, use checked arithmetic to ensure the complete encrypted-data object can fit within 4096 bytes.

The 40-byte/client record size plus fixed framing gives a strict finite upper bound on DH operations; no arbitrary independent count limit is needed. Never truncate or deduplicate clients to fit.

## Pre-frozen exact production envelope

If M159 closes within its registered owner set, M160 is expected to modify exactly the same four existing files:

1. `emissary-core/src/crypto/els2.rs`
2. `emissary-core/src/destination/lease_set.rs`
3. `emissary-core/src/sam/parser.rs`
4. `emissary-core/src/sam/session.rs`

No new file is expected. These are already realized M061 owners. Registration after M159 must verify this exact envelope still holds; any fifth production path requires amendment before M160 is registered.

Explicitly out of scope: `crypto/mod.rs`, `red25519.rs`, `destination/mod.rs`, NetDB/I2NP/primitives, events, router/tunnel/transport, any I2PControl production file, Yosemite, and client-side NetDB lookup/decryption.

## Dependency envelope

No new dependency is expected or authorized by this draft. Reuse the repository's existing `x25519-dalek`, HMAC-SHA256, ChaCha20, RNG, zeroize and M156-M159 helpers. No second curve/X25519 implementation.

M160 registration after M159 must confirm:

- no Cargo/lockfile/Yosemite/feature changes;
- the existing X25519 API can provide ephemeral private/public derivation and shared-secret bytes without a new module;
- all-zero shared-secret rejection is explicit at the ELS2 boundary.

If that is false, amend before registration rather than silently adding dependencies/files.

## Secret ownership

- parser extracts and removes `leaseSetPrivKey` and all DH client-key values from generic debug-capable options before activation;
- base private key and ephemeral private key use zeroizing/non-`Debug` wrappers;
- indexed client names are not retained by core;
- public authorized client-key entries, including duplicates, may be retained generation-locally but never become response-facing via this primitive;
- core persists nothing; M162 owns persistent key generation/custody and transactionality.

Sparse indexed keys, malformed Base64, wrong lengths, mixed PSK/DH properties and unsupported auth types fail before activation. Duplicate public-key entries are preserved as the reference does and tested explicitly.

## Publication/session behavior

Reuse the M159 publication owner and configuration shape rather than adding another scheduler/state machine. Day rollover, inner-LS renewal, retry rebuild and stale-generation rules are identical to M159. Authenticated generations never fall back to no-auth/type3.

M158 extended-B32 emission sets `auth_required=true`; `secret_required` remains independently controlled by the optional lookup secret.

## Tests and interoperability

Require:

- deterministic X25519/HKDF/client-ID/cookie KATs with fixed ephemeral key/auth cookie/salts;
- one/multiple clients and both absent/present lookup secret;
- duplicate-entry preservation;
- all-zero shared-secret rejection and maintained-library malformed/low-order cases;
- wrong-client/tamper/no-match failures;
- 4096-byte boundary/overflow checked before O(N) X25519 work;
- randomized record-order evidence;
- renewal/UTC rollover/stale-generation tests;
- no-auth/M158/M159/ordinary regressions;
- pinned Java/reference interoperability without adding a production resolver;
- no-std and full M061/M062/M095/M105 checks.

## Proposal-support disposition

M160 has zero promotion budget. `LeaseSetClientAuths`, `EncryptLeaseSet` and Proposal-layer `OptionalLookup` remain blocked until M162 integrates the full five-family persistent control-plane contract.

## Registration rule

After M159 closes, M160 may be registered directly if and only if:

1. M159 closure confirms the four-file owner graph remains valid;
2. the existing X25519 dependency is sufficient with no manifest/lock change;
3. M062 is updated from M159 to M160 with the same zero-dependency envelope;
4. registry/roadmap/AGENTS advance only M160.

No additional generic exact-path research milestone is required if those four facts hold.

## Stop conditions

Stop/amend if M160 needs a fifth production file, new dependency, I2PControl change, another publication owner, unbounded DH work, a client-side NetDB subsystem, or any confidentiality/authentication downgrade.

## Closure evidence

Record exact realized paths/dependencies, duplicate-preservation behavior, 4096-byte/O(N) bound proof, X25519 low-order/all-zero handling, KAT/reference interoperability, secret custody/redaction, zero matrix delta, implementation SHA, M061/M062 evidence, and M161/M162 readiness.