# M162 — Proposal LeaseSet-Security Field Integration

Status: **deferred / unregistered; hard-depends on M160 closure and M161 disposition**

Class: I2PControl capability integration

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Target fields/families:

- `EncryptLeaseSet` × 5 server families;
- `OptionalLookup` × 5 server families;
- `LeaseSetClientAuths` × 5 server families.

## Objective

Integrate only already-proven neutral M156-M160 primitives with Proposal-170 validation, persistence and running server-session behavior. Field-level promotion is conditional on the full valid contract of that field, not on presence of a serializer/property.

M162 must not implement new cryptography, NetDB behavior, legacy LS1, or general SigType support. Missing lower primitives are a stop condition.

## Entry gate

M162 may register only after:

- M155 semantics/field-coupling freeze is closed;
- M156 Red25519/blinding closed;
- M157 modern Encrypted LS2 publication closed;
- M158 lookup-secret/address primitive closed;
- M159 PSK auth closed;
- M160 DH auth closed;
- M161 has an explicit A/B/C legacy-AES disposition;
- any M161 outcome-A implementation successor has closed before `EncryptLeaseSet` promotion is considered;
- exact I2PControl files/secret-store fields and M061/M062 production paths are frozen.

## Proposal mapping requirements

For each of the ten `EncryptLeaseSet` strings, build a machine-readable table recording:

- required lower primitive;
- exact standard session properties;
- required/forbidden `OptionalLookup` value;
- required/forbidden client auth entries;
- auth type (none/PSK/DH);
- secret/key generation/import rules;
- restart/edit semantics;
- actual published LeaseSet format/store key.

Validation of invalid or incomplete combinations must fail before allocation/listener/publication side effects.

## Promotion rules

### `OptionalLookup`

Promote up to 5 cells only if every valid use of the field has real lookup-secret/blinding publication semantics, restart-safe secret custody, reference interoperability and no downgrade. A stored string or Yosemite option is insufficient.

### `LeaseSetClientAuths`

Promote up to 5 cells only if all valid PSK/DH per-user uses defined by the Proposal/reference are implemented with bounded entries, correct key semantics, restart behavior and authorized/unauthorized interoperability.

### `EncryptLeaseSet`

Promote up to 5 cells only if **every valid Proposal value** is operational for that family. If legacy `encrypted (aes)` remains a valid but unsupported value, all five `EncryptLeaseSet` cells remain blocked even if all modern modes work.

No partial enum-domain support may be represented as `apply`.

## Expected matrix ceilings

From `336/29/475`:

- if OptionalLookup + ClientAuths fully close but EncryptLeaseSet remains blocked: at most `346/19/475`;
- if EncryptLeaseSet also fully closes: at most `351/14/475`;
- 10 SigType and 4 UseOutproxyPlugin remain separate blocked clusters unless independently resolved.

These are ceilings, not guaranteed outcomes.

## I2PControl containment

Expected work remains in exact `emissary-cli/src/i2pcontrol/**` backend/options/secret-store/session-generation paths plus tests/matrix/docs. Any new lower-layer runtime requirement discovered here means M162 must stop and create a separate neutral corrective plan.

## Security/transactionality

- secret/auth material never appears in Get/rawConfig/logs/errors/metrics;
- edits/restarts are generation-transactional;
- failed secret/auth persistence preserves last-known-good running restricted generation;
- unsupported modes/combinations fail before allocation;
- no downgrade to ordinary/blinded-without-secret/unauthenticated publication;
- httpbidirserver reuses the canonical server destination owner rather than duplicating cryptographic state;
- cancellation/stale generation cannot republish superseded key/auth state.

## Tests

Require table-driven tests covering all ten EncryptLeaseSet strings across all five target families, every valid/invalid OptionalLookup coupling, PSK/DH client-auth structures, omitted/default behavior, edits/restarts, secret-store redaction/failure, reference interoperability, and exact fail-before-allocation behavior.

Run full core/I2PControl/M061/M062/M095/M105 qualification. Any promoted cell must have direct runtime evidence.

## Stop conditions

Stop/split if integration discovers missing crypto/NetDB/LeaseSet behavior, if a valid enum value can only be accepted inertly, if any failure path downgrades confidentiality/authentication, or if exact secret transactionality cannot be preserved.

## Closure evidence

Record the final ten-mode table, exact changed paths, secret-store schema changes, all interoperability results, per-field promotion decisions, matrix delta, containment/dependency evidence, implementation SHA, unresolved blockers, and M152 readiness.