# M156 — Neutral Red25519 and Ed25519-Blinding Primitive

Status: **deferred / unregistered; hard-depends on M155 closure**

Class: neutral cryptographic infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

## Objective

Implement the smallest standard I2P cryptographic primitive required to derive and use type-11 Red25519 blinded signing keys from an existing type-7 Ed25519 Destination for Encrypted LeaseSet2.

This milestone does not reopen M147/M148, does not make type 11 a persistent/user-selectable Destination SigType, and must not add legacy DSA/ECDSA generation.

## Registration gate

M156 MUST NOT be registered until M155 closure freezes:

- exact I2P blinding/Red25519 formulas and reference vectors;
- exact files and APIs;
- exact direct dependency/version/features if required;
- no-std posture;
- constant-time/security review;
- confirmation that type-7 Destinations are sufficient inputs for the modern encrypted-LS2 path.

At registration, M061/M062 must authorize only exact production files actually required.

## Required neutral API properties

The API must be Proposal-free and difficult to misuse. Expected capabilities:

- derive daily blinding scalar `alpha` from unblinded signing public key, input/output sig-type codes, UTC date, and optional secret;
- blind an Ed25519 public key by Edwards-point addition;
- derive the matching blinded private scalar from an Ed25519 signing seed/private scalar plus `alpha`;
- Red25519 sign with cryptographically random nonce input according to the I2P construction;
- verify Red25519 signatures;
- derive the blinded storage-key preimage/hash inputs required by M157;
- expose exact 32-byte/64-byte typed values rather than generic arbitrary-length byte vectors where practical;
- zeroize private scalars/intermediate secret material.

No generic runtime algorithm registry is allowed.

## Security invariants

- no bespoke curve arithmetic or bignum implementation;
- scalar reduction and Edwards operations come from a maintained constant-time Rust crypto implementation frozen by M155;
- production Red25519 signatures use secure RNG; deterministic RNG injection is test-only;
- private scalar/alpha/seed material is never Debug/loggable;
- malformed/non-canonical public points/scalars fail closed;
- all derivation domains include exact I2P domain separation and sig-type codes;
- UTC-day boundary behavior is deterministic and covered by vectors;
- ordinary Ed25519 signing behavior remains byte-compatible and unchanged.

## Tests

Require known-answer/cross-reference vectors for:

- alpha derivation with and without lookup secret;
- blinded public key;
- blinded private scalar/public-key agreement;
- Red25519 signature verification;
- wrong key/message rejection;
- day rollover;
- malformed point/scalar rejection;
- no-std compilation if the touched owner supports no_std;
- zero regression to existing Ed25519/LeaseSet2 tests.

## Stop conditions

Close blocked if no maintained constant-time dependency can implement the required operations safely, if exact vectors disagree with the pinned Java/spec behavior, or if implementation requires generalizing persistent Destination signature types.

## Closure evidence

Record exact changed files/dependencies, vector provenance/results, no-std/security review, M061/M062 reconciliation, zero matrix delta, implementation SHA, and M157 readiness.