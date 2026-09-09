# M160 — Encrypted LeaseSet DH Client-Authorization Primitive

Status: **deferred / unregistered; hard-depends on M159 closure**

Class: neutral cryptographic authorization infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

## Objective

Implement the standard Encrypted LeaseSet2 X25519/DH client-authorization mode independently from PSK mode and Proposal parsing.

## Required behavior

- validate/import exact 32-byte X25519 authorized-client public keys;
- use the standard server private/client public DH construction and exact HKDF/cookie/client-ID encoding from the encrypted-LS2 specification;
- include the server's required private/public authorization key material with restart-safe ownership;
- authorized reference client decrypts; wrong/unknown client fails without downgrade;
- renewal/day rollover preserve authorization policy without mixing generations;
- server authorization key rotation is explicit and transactional.

## Security/work bounds

DH publication is O(N) in authorized clients. Registration must freeze an exact maximum entry count and prove bounded CPU/memory. No unbounded attacker-controlled client list is permitted.

Private X25519/server keys must be zeroized/redacted and never exposed through logs/Get/rawConfig. Invalid/small-order/noncanonical inputs must follow the maintained X25519 dependency's safe validation policy plus any I2P-specific checks frozen by M155/M160.

## Registration gate

Requires M159 closure, exact reference vectors, exact client-entry bounds, exact changed files/dependencies and exact M061/M062 authorization. Reuse existing x25519-dalek owner where possible; no second curve implementation.

## Tests

Known-answer DH/cookie vectors; authorized/wrong-key cases; max/max+1 work bounds; malformed key cases; renewal/day rollover; restart/key rotation; cancellation/stale generation; no-downgrade; reference-client interoperability; no-std/regression checks.

## Stop conditions

Stop if bounded interoperable DH auth cannot be achieved with the existing maintained X25519 primitive or requires unrelated transport/session-key redesign.

## Closure evidence

Exact paths/dependencies, vectors/interoperability, performance/bounds review, secret custody, zero matrix delta, implementation SHA, M061/M062 reconciliation, and M161/M162 readiness.