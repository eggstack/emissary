# M157 — Modern Encrypted LeaseSet2 Publication Primitive

Status: **deferred / unregistered; hard dependency on M156 closure satisfied by `plans/closure/i2pcontrol-proposal-170/156-closure.md`, exact-path amendment still required at registration**

Class: neutral LeaseSet/NetDB infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

## Objective

Implement standard type-5 Encrypted LeaseSet2 construction and publication from the existing real type-7 Destination, real inbound leases, and M156 blinded Red25519 key material.

This milestone establishes modern encrypted publication independently of Proposal option parsing. It must not implement legacy LS1 AES, PSK/DH client authorization, or claim `EncryptLeaseSet` support.

## Registration gate

M157 requires M156 closure and an exact-path amendment freezing:

- Encrypted LeaseSet2 layer formats and crypto/KDF domains;
- DatabaseStore type-5 representation;
- blinded DHT/storage key derivation;
- publication/storage-verification owner;
- UTC-day rollover/republish owner;
- exact files/dependencies;
- interoperability vectors/fixtures.

## Required runtime behavior

- construct a valid ordinary inner LeaseSet2 from current truthful inbound leases;
- construct the encrypted outer type-5 object using exact I2P nested layer format;
- sign with the current day's blinded Red25519 key;
- publish DatabaseStore as EncryptedLeaseSet/type 5 under the exact blinded storage key;
- storage verification queries the same blinded key and validates the returned object;
- republish automatically when the UTC-day blinding key rolls, even if tunnels have not otherwise changed;
- lease renewal/expiry continues to use current real leases and the existing M135 desired-count semantics;
- ordinary unencrypted LeaseSet2 publication remains unchanged for ordinary destinations;
- requested encrypted mode can never downgrade to ordinary LS2 on crypto/publication failure.

## Expected owner boundaries

M155 closure must replace this section with exact files. Candidate existing owners only:

- `emissary-core/src/primitives/lease_set.rs` for exact wire structures;
- `emissary-core/src/i2np/database/store.rs` for type-5 DatabaseStore representation;
- `emissary-core/src/destination/lease_set.rs` for publication/storage verification and rollover;
- M156's exact neutral crypto module.

No broad NetDB, routing, tunnel-building, transport, RouterInfo, frontend or I2PControl policy change is authorized by this draft.

## Security invariants

- no plaintext fallback;
- blinded storage key and outer signature correspond to the same UTC epoch/secret inputs;
- only current real inbound leases appear in the inner LS2;
- malformed/tampered layer data fails closed;
- time rollover cannot publish mixed old/new blinded key material;
- stale generation cannot republish after successor shutdown/restart;
- keys/secrets are not logged or exposed through Debug;
- encryption uses exact standard algorithms/nonce/KDF rules, not ad-hoc constructions.

## Tests

Require deterministic known-answer vectors for every layer plus:

- parse/serialize round trip;
- reference implementation accepts published object;
- wrong/tampered outer signature fails;
- wrong layer key/nonce/KDF input fails;
- DatabaseStore type and key exact;
- day rollover changes blinded key/store key and republishes once through canonical owner;
- tunnel expiry/renewal preserves encryption mode and truthful leases;
- ordinary LS2 byte/behavior regression;
- cancellation/restart race tests;
- no-std checks for touched core paths.

## Stop conditions

Split/stop if publication requires a second NetDB subsystem, broad routing rewrite, fabricated LeaseSet state, or an unreviewed cryptographic construction.

## Closure evidence

Exact changed paths/dependencies, vector/interoperability evidence, rollover/publication/storage-verification results, no-downgrade/security review, zero matrix delta, M061/M062 update, implementation SHA, and M158 readiness.