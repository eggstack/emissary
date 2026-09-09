# M158 — LeaseSet Lookup-Secret and Blinded-Address Primitive

Status: **deferred / unregistered; hard-depends on M157 closure**

Class: neutral/application LeaseSet privacy infrastructure

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Default promotion budget: **zero Proposal cells**. M155 may authorize up to 5 `OptionalLookup` promotions here only if it proves the field's full valid contract is independently complete at this stage; otherwise promotions are deferred to M162.

## Objective

Add the standard optional lookup/blinding secret to the M156/M157 derivation/publication path and implement the standard encrypted-service extended `.b32.i2p` representation needed for usable interoperability.

This milestone is publication/addressing focused. It must not create a second generic NetDB client subsystem merely to prove a server-side Proposal field.

## Required behavior

- accept an optional secret as exact UTF-8 bytes according to the pinned Java/Proposal mapping;
- feed that secret into the standard daily alpha/blinding derivation;
- derive the corresponding blinded public/store key deterministically;
- encode/decode the standard extended encrypted-service B32 form with exact flags, unblinded sig type and blinded sig type;
- preserve secret-required/auth-required flag semantics required by later modes;
- regenerate/re-publish on secret change or UTC-day rollover through the existing M157 owner;
- restart reloads secret material atomically/redacted where the owning application contract requires persistence;
- wrong/missing required secret never falls back to public/plain lookup/publication.

## Containment

M155/M157 closure must freeze exact files before registration. Expected changes are limited to M156/M157 neutral crypto/address/publication owners and I2PControl secret storage only if needed to prove restart behavior. No generic resolver, second NetDB query engine, transport, or routing subsystem is authorized.

## Security invariants

- raw lookup secrets never enter logs, errors, metrics, Get/rawConfig or non-secret persistence;
- secret-derived identifiers cannot be confused across Destinations;
- B32 parsing validates checksum/flags/type domain and exact length;
- malformed addresses/secrets fail before network publication;
- removing/changing a secret is generation-transactional and cannot mix old/new publication keys;
- no downgrade to unsecreted blinded publication when the secret is required.

## Tests

Require known-answer vectors for secret alpha/store-key derivation and B32 encode/decode, wrong-secret negative tests, checksum/flag/type adversarial tests, restart/rotation tests, UTC rollover, reference lookup/address interoperability, and no ordinary-LS2 regressions.

## Stop conditions

Stop if exact extended-B32/secret semantics cannot be frozen, if only wire storage without a real publication consumer is possible, or if proving the server capability requires inventing a broad new NetDB client subsystem.

## Closure evidence

Record exact paths, secret-store ownership, vectors/interoperability, promotion decision if any, no-downgrade review, M061/M062 state, implementation SHA, and M159 readiness.