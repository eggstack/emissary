# M162 — Proposal LeaseSet-Security Field Integration

Status: **deferred / unregistered; hard-depends on M160 closure and M161 disposition; integration contract pre-frozen**

Class: I2PControl capability integration

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Target fields/families:

- `EncryptLeaseSet` × 5 server families;
- `OptionalLookup` × 5 server families;
- `LeaseSetClientAuths` × 5 server families.

## Objective

Integrate only already-proven neutral M156-M160 primitives with exact Proposal-170 validation, persistent secret/key custody, edit/restart transactionality and running server-session behavior.

M162 is the **first** milestone allowed to turn the neutral M158-M160 primitives into Proposal field support. It must not implement new cryptography, NetDB behavior, legacy LS1, general SigType support, or a second server-session data plane. Missing lower primitives are a stop condition.

## Entry gate

M162 may register only after:

- M155 semantic/field-coupling freeze is closed;
- M156 Red25519/blinding closed;
- M157 modern Encrypted LS2 publication closed;
- M158 lookup-secret/address primitive closed;
- M159 PSK auth closed;
- M160 DH auth closed;
- M161 has an explicit A/B/C legacy-AES disposition;
- any M161 outcome-A implementation successor has closed before `EncryptLeaseSet` promotion is considered;
- the exact I2PControl owner set and secret-store schema changes are revalidated against the then-current head and frozen in M061/M062 before registration.

## Direct-source mapping authority

Pinned Proposal-170 PR `ServiceTunnelCreator` is the authoritative field-to-standard-property mapping.

### Modern/legacy selector split

```text
legacy "encrypted (aes)":
    i2cp.encryptLeaseSet = true
    modern type-5 selector absent

modern blinded / PSK / DH modes:
    i2cp.leaseSetType = 5
    i2cp.encryptLeaseSet is NOT the modern selector
```

M161 owns the legacy flag/value contract. M162 must never alias legacy AES to type 5.

### Lookup secret

Proposal `OptionalLookup` plaintext maps to the standard property:

```text
i2cp.leaseSetSecret = Base64(UTF8(OptionalLookup))
```

This is the exact M158 runtime representation. Empty/absent Proposal value must follow the direct Proposal PR mode table rather than being silently invented for a mode that requires a lookup secret.

### PSK authorization

For PSK modes:

```text
i2cp.leaseSetAuthType = "2"
i2cp.leaseSetPrivKey = Base64(32-byte persistent base PSK)
```

For per-user PSK modes, each `LeaseSetClientAuths` entry additionally maps to a contiguous indexed property:

```text
i2cp.leaseSetClient.psk.N = Base64(UTF8(Name)) + ":" + Key
```

where `Key` MUST Base64-decode to exactly 32 bytes.

The base `leaseSetPrivKey` remains an authorized PSK even when no indexed per-user entries exist. Therefore non-per-user PSK modes are operationally distinct from per-user PSK modes and MUST NOT be modeled as “PSK list required.”

### DH authorization

For DH modes:

```text
i2cp.leaseSetAuthType = "1"
i2cp.leaseSetPrivKey = Base64(32-byte persistent X25519 private key)
```

For per-user DH modes:

```text
i2cp.leaseSetClient.dh.N = Base64(UTF8(Name)) + ":" + Key
```

where `Key` MUST Base64-decode to exactly 32 bytes of X25519 public key material. The public key derived from `leaseSetPrivKey` is part of the reference authorized-client set.

## Ten-mode table requirement

Before any M162 production edit, materialize a machine-readable table for all ten exact `EncryptLeaseSet` strings recording at least:

- exact string spelling;
- legacy flag value/presence;
- `leaseSetType` value/presence;
- auth type 0/1/2 and neutral lower primitive;
- whether lookup secret is required, forbidden or absent;
- whether indexed client auth entries are required, allowed or forbidden;
- PSK vs DH key interpretation;
- persistent base-key role;
- extended-B32 `secret_required` / `auth_required` flags;
- secret/key generation vs import rules;
- create/edit/restart semantics;
- actual published DatabaseStore type and storage key;
- M161 dependency for legacy AES.

The table is executable acceptance authority. Tests must iterate it across all five target server families.

## Control-plane ownership model

### Typed domain, not rawConfig

`EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` must be modeled as typed Proposal fields, not smuggled through generic `I2CPOptions`/`CustomOptions` or response-facing raw config.

Secret-bearing values must use redacted/non-debug representations. `Get` may return only the exact Proposal-defined safe representation; if the Proposal/reference does not require returning raw secret/key material, it must remain redacted/omitted according to existing secret-store convention.

### Dedicated server LeaseSet-security secret state

Do not put lookup passwords, base PSKs/private keys, or per-user key material into `ServerDestinationStore` destination identity blobs unless that store's schema is explicitly extended with a separate typed secret record and transactional semantics.

Preferred ownership is an I2PControl-owned server LeaseSet-security secret record keyed by the same stable tunnel/server identity used by the server backend, with:

- lookup secret;
- generated/imported base PSK or X25519 private key as applicable;
- per-user `{name,key}` entries;
- generation/revision metadata needed for atomic edits;
- no raw secret serialization into ordinary `TunnelDefinition.raw_config`.

If extending `server_secret_store.rs` is sufficient without creating a second store, prefer that. A new secret-store file requires explicit exact-path amendment before M162 registration.

### Transaction ordering

Create/edit/restart must be generation-transactional:

1. validate complete mode/table combination and all key lengths before allocation;
2. prepare new secret state off to the side;
3. persist secret state atomically or fail with the last-known-good record intact;
4. commit definition/runtime generation only when its matching secret revision is available;
5. cancel/stop stale generation before it can republish superseded authorization material;
6. failed runtime start does not destroy the last-known-good secret generation unless the definition transaction itself was never committed;
7. deleting the tunnel removes its LeaseSet-security secrets through the same deletion transaction/cleanup discipline used for destination secrets.

No split-brain definition/secret generation is acceptable.

## Expected owner set for later exact-path freeze

The then-current head must be revalidated, but M162 should stay within existing I2PControl canonical owners. The expected set is limited to exact files in these roles:

- `domain/tunnel.rs` — typed/redacted Proposal fields;
- `backends/options.rs` — common fail-before-allocation field/coupling validation;
- `backends/runtime/session.rs` — canonical standard-property translation into Yosemite/SAM session options;
- the five real server-family backend owners (`server`, `httpserver`, `httpbidirserver`, `ircserver`, `streamrserver`) only where each needs capability declaration/runtime composition;
- existing server secret-store owner for LeaseSet-security custody if extension is sufficient;
- TunnelManager/control-plane persistence transaction owner only if required to atomically commit definition + secret revision;
- M095/M105 tests/matrix/docs after real promotion.

This list is **not executable authority** while M162 is deferred. Registration must enumerate concrete file paths after M160/M161 close. Any need for lower-layer core changes is a stop/split condition, not a reason to broaden this list.

## Promotion rules

### `OptionalLookup`

Promote up to five cells only if every valid use has:

- exact Proposal plaintext-to-standard Base64(UTF8) mapping;
- real M158 secret-aware blinding/publication;
- persistent redacted custody;
- create/edit/restart/delete transactionality;
- extended B32 secret flag correctness;
- reference interoperability;
- no downgrade on missing/wrong secret.

A stored string or Yosemite property alone is insufficient.

### `LeaseSetClientAuths`

Promote up to five cells only if the full Proposal list contract is operational for every mode in which it is applicable:

- exact `{Name,Key}` parsing and 32-byte Base64 key validation;
- exact PSK/DH property prefix/value formatting;
- bounded M159/M160 4096-byte work contract;
- persistent names/key custody and redaction;
- deterministic edit/restart semantics;
- authorized/unauthorized reference interoperability;
- no inert entries, sparse-index truncation or silent client dropping.

If a security bound causes a syntactically valid Proposal list to be rejected, closure must decide explicitly whether the field remains blocked rather than claiming full support.

### `EncryptLeaseSet`

Promote up to five cells only if **every valid Proposal enum value** is operational for that family. If M161 leaves legacy `encrypted (aes)` valid but unsupported, all five `EncryptLeaseSet` cells remain blocked even when all modern type-5 modes work.

No partial enum-domain support may be represented as `apply`.

## Matrix ceilings

From `336/29/475`:

- if OptionalLookup + LeaseSetClientAuths fully close but EncryptLeaseSet remains blocked: at most `346/19/475`;
- if EncryptLeaseSet also fully closes: at most `351/14/475`;
- the 10 SigType and 4 UseOutproxyPlugin blockers remain independent.

These are ceilings, not promised outcomes.

## Security/transactionality invariants

- raw lookup secrets, base PSKs/private keys and per-user keys never appear in logs/errors/metrics/Get/rawConfig unless exact Proposal semantics explicitly require a safe public field;
- user names are not cryptographic secrets but must not be exposed through unrelated diagnostics;
- unsupported/invalid mode combinations fail before listener/session/publication allocation;
- no downgrade to ordinary, unsecreted or unauthenticated publication;
- httpbidirserver reuses the canonical server destination/session owner rather than duplicating cryptographic state;
- stale/cancelled generations cannot republish superseded secret/auth material;
- secrets never participate in `Debug`/Display compatibility keys except through a non-reversible identity/fingerprint where sharing semantics require distinction;
- shared session compatibility must distinguish different LeaseSet-security generations without logging their material.

## Required tests

Table-driven tests must cover all ten modes × five server families, including:

- create, edit, restart, stop/start and process restart;
- missing/extra OptionalLookup;
- PSK and DH per-user structures;
- non-per-user PSK/DH base-key modes;
- malformed Base64, wrong lengths, duplicate/sparse entries;
- secret-store write failure/cancellation/rollback;
- Get/rawConfig/log/debug redaction;
- extended B32 flags;
- exact standard-property wire output;
- reference interoperability;
- deletion/secret cleanup;
- fail-before-allocation on invalid modes;
- no modern mode setting legacy `i2cp.encryptLeaseSet=true`;
- legacy AES never silently mapping to type 5;
- all lower-layer M157-M160 regressions.

Run full core/I2PControl/M061/M062/M095/M105 qualification. Every promoted cell requires direct runtime evidence.

## Stop conditions

Stop/split if:

- integration discovers missing crypto/NetDB/LeaseSet behavior;
- a valid enum/list value can only be accepted inertly;
- a failure path downgrades confidentiality/authentication;
- exact definition+secret transactionality cannot be preserved;
- the existing server-family architecture requires broad non-I2PControl changes;
- M161 outcome A has an unclosed legacy implementation successor;
- promotion would depend on broadening the accepted M159/M160 security/work bounds without a separate review.

## Closure evidence

Record the final ten-mode machine table, exact changed paths and M061/M062 authorization, secret-store schema/transaction design, standard-property wire fixtures, all interoperability/redaction/failure results, per-field/per-family promotion decisions, exact matrix delta, implementation SHA, residual blockers and M152 readiness.