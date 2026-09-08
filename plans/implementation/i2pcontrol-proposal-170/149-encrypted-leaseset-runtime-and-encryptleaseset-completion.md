# M149 — Encrypted LeaseSet Runtime and EncryptLeaseSet Completion

Status: **deferred / unregistered; hard-depends on M148 closure**

Class: infrastructure + capability / LeaseSet cryptographic security

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- `EncryptLeaseSet` for `server`, `httpserver`, `httpbidirserver`, `ircserver`, and `streamrserver`.

Historical authority:

- M131 `PB-LEASESET-CRYPTO-01`;
- M122/M124/Y005 prove selected SAM option serialization but not Emissary end-to-end LeaseSet confidentiality.

## 1. Objective

Implement the minimum neutral Emissary LeaseSet confidentiality/publication runtime required for truthful Proposal `EncryptLeaseSet` behavior, then wire exactly the five server families to that primitive.

M149 must not claim `OptionalLookup` or `LeaseSetClientAuths` merely because shared LS2/blinding primitives are introduced. Those remain M150/M151.

## 2. Registration gate — mode and exact-path freeze

M149 MUST NOT be registered until an explicit reference/spec audit amends this plan with:

1. exact Proposal value/type semantics for `EncryptLeaseSet`;
2. Java I2PTunnel mapping to `i2cp.leaseSetType`, `leaseSetSecret`, `leaseSetKey`, `leaseSetPrivKey`, auth type and related properties;
3. exact I2P LeaseSet format(s) required: legacy LeaseSet, encrypted LS1, LS2, EncryptedLeaseSet/blinded variants as applicable;
4. current Emissary LeaseSet construction, signing, publication and NetDB owners;
5. exact key-generation/storage requirements and restart behavior;
6. exact files/dependencies required, with security review;
7. exact interoperability fixtures/reference peers needed for closure.

M061 currently prohibits broad `emissary-core/src/crypto/`, `lease_set/` and `netdb/` changes. This plan grants no exception. Before registration, authorize exact files only. If the primitive requires a broad NetDB/crypto rewrite, stop and split the infrastructure further.

## 3. Neutral lower-layer contract

Core must model standard I2P LeaseSet confidentiality, not Proposal options. The resulting neutral API must permit the canonical destination/LeaseSet owner to construct and publish the selected format using real keys and real inbound leases.

Required properties:

- format/type is explicit and validated, never inferred from key length alone;
- cryptographic keys are generated/imported with exact required lengths/types;
- LeaseSet content is signed/encoded according to I2P format rules;
- publication uses the canonical NetDB owner and cannot fall back to plaintext when confidentiality is requested;
- desired/current inbound lease truthfulness from M135 remains intact;
- expiration/renewal produces fresh cryptographically valid LeaseSets;
- restart-safe secret/key custody preserves identity/config semantics required by the reference;
- unsupported format/type fails before publication/session readiness;
- ordinary unencrypted LeaseSet behavior remains unchanged when option is omitted/false.

## 4. Security invariants

- No requested encrypted mode silently publishes an ordinary plaintext LeaseSet.
- No fabricated leases, expired tunnels, wrong destination or wrong signing key are published.
- Encryption/blinding secrets and private keys never enter logs, Debug, Get/rawConfig or non-secret persistence.
- Key material is stored atomically with restrictive existing secret-store semantics.
- Cryptographic randomness comes from existing approved secure RNG owners.
- Cipher/KDF/MAC parameters exactly match the pinned I2P specification; no ad hoc construction.
- Decryption/client-lookup functionality not required for server publication is not exposed as completed OptionalLookup.
- M145 reply-bundling must never leak confidential LeaseSet material contrary to the encrypted format's publication semantics.
- Existing ordinary LeaseSet publication, M135 desired-count behavior and RouterInfo/NetDB invariants remain regression-tested.

## 5. I2PControl/session mapping

After the neutral runtime is real:

- validate `EncryptLeaseSet` exactly for five target server families before allocation;
- translate the Proposal field and any required canonical companion values into the accepted Yosemite/session options;
- ensure the server destination's actual published LeaseSet format matches the requested mode;
- preserve server secret-store restart semantics;
- `httpbidirserver` reuses its server destination/session owner;
- invalid/missing required key/secret companion values fail before start/restart side effects;
- no client family is accidentally enabled.

Yosemite wire acceptance remains adapter evidence only; the published NetDB object is the capability evidence.

## 6. Candidate path categories — not authorization

Before registration, replace this section with exact files.

Potential neutral owner categories:

- current LeaseSet data structures/serialization;
- destination LeaseSet manager/build/publication;
- exact cryptographic modules for required encrypted/blinded format;
- exact NetDB publication owner;
- secret key representation/persistence where owned by core or I2PControl.

Expected I2PControl paths:

- server backend/session option validation/mapping;
- server secret store only for Proposal-owned persisted secret companions;
- matrix/tests/docs.

No transport/tunnel-building/frontend/startup proxy or unrelated RouterInfo change is authorized.

## 7. Failure, cancellation, restart and contention

- Mode/key validation precedes server session readiness/publication.
- Encryption/publication failure prevents the new generation from becoming truthfully running if the contract requires a publishable LeaseSet.
- Failed edit/restart preserves last-known-good destination/key material and running generation according to existing transaction authority.
- Cancellation prevents stale generation publication and key-store commit where commit has not already become authoritative.
- LeaseSet regeneration uses bounded timers/events already owned by the destination; no second polling loop.
- No destination/NetDB/secret-store lock spans unrelated network I/O or expensive crypto beyond narrowly required local state transitions.

## 8. Work packages

### WP1 — exact mode/spec/owner audit

Freeze modes, wire properties, I2P format specifications and exact current files. Split M149 if more than one independently large format primitive is required.

### WP2 — neutral LeaseSet confidentiality structures/crypto

Implement standard format parsing/serialization/encryption/signing with deterministic known-answer fixtures.

### WP3 — destination publication integration

Build encrypted LeaseSets from the real current inbound leases and publish through the canonical NetDB owner with ordinary renewal/expiry semantics.

### WP4 — secret persistence/restart

Implement only required restart-safe keys/secrets with atomic redacted custody.

### WP5 — I2PControl five-family mapping

Map exact validated options through server session generation and prove actual published format.

### WP6 — interoperability/adversarial evidence

Use reference vectors or a bounded reference router fixture to validate published encrypted LeaseSet format and failure on malformed/wrong keys.

### WP7 — matrix/containment/closure

Promote only proven five-family cells; reconcile exact sensitive paths, dependencies, M095/M105/docs/registry.

## 9. Focused tests

After mode freeze, require:

- ordinary omitted/false mode stays byte/behavior compatible;
- each selected encrypted mode produces parseable exact-format LeaseSet from real current inbound leases;
- wrong key/type/length rejected without fallback;
- published object cannot be interpreted as ordinary plaintext LeaseSet when encryption requested;
- signing identity matches hosting Destination;
- renewal/expiry maintains mode and valid fresh leases;
- restart reloads required keys/secrets and republishes interoperably;
- failed secret-store write/publication preserves prior generation/keys;
- cancellation prevents stale publication;
- no secret bytes in logs/Get/rawConfig;
- five family gates exact;
- M135 desired inbound count and M145 bundling privacy regressions green;
- reference/live interoperability for at least one mode required by Proposal closure.

## 10. Matrix promotion budget

Maximum: **5 cells**, independently proven from the M148 closure baseline.

Infrastructure alone or a Yosemite property does not promote a cell. The actual published LeaseSet must have the requested confidentiality semantics.

## 11. Verification

The registered amendment must add exact core/no-std/dependency commands for the selected modules. Minimum:

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 12. Acceptance criteria

M149 closes complete only when:

- exact encrypted LeaseSet mode semantics/specs are frozen;
- exact sensitive core files/dependencies were authorized before implementation;
- real published LeaseSets provide requested confidentiality with current truthful leases;
- renewal/restart/secret custody/failure behavior is proven;
- no plaintext downgrade or secret leak is possible;
- all promoted families consume the real primitive;
- OptionalLookup/ClientAuths remain unclaimed;
- M061/M062/M095/M105/docs/registry match actual behavior;
- no medium/high cryptographic/NetDB/LeaseSet issue remains.

## 13. Stop conditions

Stop/split and leave cells blocked if:

- exact mode mapping or I2P format cannot be established;
- implementation requires broad unbounded NetDB/crypto redesign;
- required cryptographic primitive/dependency fails security review;
- confidentiality would be best-effort or downgrade on failure;
- real current LeaseSets cannot be published without fabricating/rewriting unrelated router behavior.

## 14. Closure evidence required

Record amended exact mode table/spec references, exact changed paths/dependencies, known-answer format/crypto fixtures, actual publication/renewal/restart evidence, secret/redaction review, no-downgrade tests, interoperability result, matrix delta, containment/dependency update, implementation SHA, unresolved findings and M150 readiness. External access remains read-only.