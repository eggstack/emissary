# M159 — Encrypted LeaseSet PSK Client-Authorization Primitive

Status: **deferred / unregistered; M158 hard dependency satisfied, amendment pending**

> M158 closed complete (`plans/closure/i2pcontrol-proposal-170/158-closure.md`),
> satisfying this milestone's hard dependency. Registration additionally
> requires an exact-path amendment (PSK format/ownership, work limits,
> custody boundaries, integration seams, exact files/dependencies, interop
> vectors) authored at M159 registration. This file is otherwise unchanged;
> no production work is authorized by this note.

Class: neutral cryptographic authorization infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

## Objective

Implement the standard Encrypted LeaseSet2 pre-shared-key client authorization mode on top of M157/M158, independently of DH authorization and independently of Proposal parsing.

## Required behavior

- bounded list of 32-byte PSK authorization keys with deterministic duplicate/error semantics frozen from the reference;
- exact client-ID/cookie derivation, HKDF and ChaCha20 processing from the I2P encrypted-LS2 specification;
- include authorization material in the correct encrypted layer without exposing labels or raw keys on the wire beyond the standard format;
- authorized reference client decrypts the intended LS2; wrong/unknown key fails without downgrade;
- renewals and UTC-day rollover preserve the configured authorization set;
- generation changes rotate authorization state atomically;
- persistent private material is redacted and transactionally stored only where required.

## Security/work bounds

- validate entry count and key length before cryptographic work;
- cap memory/CPU by an exact bound frozen at registration;
- no secret-dependent logging/error differentiation;
- constant-time comparisons/derivation primitives where applicable;
- no unauthenticated/plaintext fallback on failure;
- no reuse of PSK material as Destination, lookup-secret or transport keys.

## Registration gate

M159 requires M158 closure plus exact I2P PSK vectors, exact entry limits/reference semantics, exact changed files/dependencies and exact M061/M062 authorization. No broad crypto/LeaseSet path is authorized by this draft.

## Tests

Known-answer derivation/encryption vectors; authorized/wrong-key cases; zero/max/max+1 entries; duplicate/malformed inputs; renewal/day rollover; restart/rotation transactionality; no-downgrade; reference-client interoperability; no-std/regression checks for touched owners.

## Stop conditions

Stop/split if PSK mode requires broad generic keyring infrastructure, unbounded per-client work/state, or cannot interoperate with the pinned reference.

## Closure evidence

Exact paths/dependencies, vector/interoperability results, bounds/secret review, zero matrix delta, implementation SHA, M061/M062 reconciliation, and M160 readiness.