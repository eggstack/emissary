# M147 — Neutral Destination Signature-Suite Primitive

Status: **deferred / unregistered; hard-depends on M146 closure**

Class: infrastructure / cryptographic identity security

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Promotion budget: **zero Proposal cells**.

Historical authority:

- M121 Outcome C proved that fixed Ed25519/type-7 support is not truthful configurable `SigType` support;
- M131 `PB-DEST-SIGTYPE-01` records the missing neutral signing/key-generation primitive.

## 1. Objective

Implement the minimum neutral Emissary destination signing/key-generation infrastructure required for a later truthful Proposal `SigType` implementation, without putting Proposal policy into core and without promoting any M095 row in this milestone.

The primitive must support actual key generation, Destination certificate/type encoding, signing and verification interoperability for the exact signature-type domain selected by the pinned I2P/SAM reference and project security policy. Merely parsing a type code or carrying it through Yosemite is not infrastructure completion.

## 2. Registration gate — exact algorithm and owner freeze

M147 MUST NOT be registered until a dedicated pre-implementation audit amends this plan with:

1. the exact signature-type numeric/name domain accepted by the pinned Proposal/Java I2PControl/SAM path;
2. the subset required for truthful Proposal compatibility rather than optional legacy support;
3. current Emissary generation/sign/verify capabilities per type;
4. cryptographic/security disposition for legacy algorithms;
5. exact files owning private signing keys, Destination certificates/serialization, signing and verification;
6. persistence/import/export formats and compatibility implications;
7. exact-file M061/M062 path authorization.

M061 currently prohibits broad changes under `emissary-core/src/crypto/`. This plan does not override that rule. No crypto directory/prefix may be allowed wholesale. Exact files only, after audit.

If the pinned Proposal requires an algorithm the project cannot safely or reasonably implement, record the blocker and leave later SigType cells partial rather than weakening cryptographic policy.

## 3. Neutral architecture contract

Core must express standard I2P cryptographic identities, not Proposal values. The preferred shape is a signature-suite/type abstraction owned by existing primitive/crypto code with operations required by real destinations:

- generate private/public signing key pair;
- derive/serialize the Destination signing public key and key certificate;
- parse/validate encoded keys/certificates;
- sign the I2P structures/messages currently signed by destination identity;
- verify corresponding signatures;
- expose type/length information needed by serialization without dynamic unchecked allocation;
- preserve existing Ed25519 behavior exactly when the new abstraction is not selected.

Avoid a generic pluggable crypto framework if a small closed I2P signature-suite enum is sufficient.

## 4. Security invariants

- Private signing keys never implement Debug/Display/Serialize in a way that exposes raw secret bytes.
- Secret buffers are zeroized using existing project primitives where applicable; adding a dependency requires separate authorization.
- Algorithm/type and key-length validation occurs before allocation/use.
- No implicit fallback to Ed25519 or another algorithm on parse/generation/sign failure.
- Parsed certificate type must match key material and signature implementation.
- Constant-time comparisons/verification primitives are used according to underlying library guarantees.
- Invalid/malleable encodings are rejected according to the I2P spec.
- Existing RouterInfo/router identity behavior is not broadened unless explicitly required by the destination owner; M147 targets destination signing, not router identity migration.
- Existing Ed25519 destination byte compatibility remains regression-tested.
- No Proposal/I2PControl/Yosemite names or type aliases appear in core.
- No algorithm is enabled solely because a numeric code exists if its security or dependency posture is unacceptable.

## 5. Persistence and compatibility

The audit must freeze how existing persistent client/server destination stores represent private key material and whether the format can encode additional signature suites.

Required behavior:

- existing Ed25519 persisted destinations remain readable byte-for-byte;
- new suites persist all required key/certificate type metadata atomically;
- import validates actual key/certificate type and cannot be relabeled by a caller;
- export/round-trip, where currently supported, preserves type and secret material exactly;
- malformed/truncated/wrong-length keys fail before replacing a last-known-good stored identity;
- no automatic migration rewrites existing identities merely because M147 lands.

If persistence requires an incompatible on-disk format change, M147 must include an explicit migration/versioning amendment before registration.

## 6. Candidate path categories — not authorization

Exact paths MUST be filled before registration. Candidate current owner categories include:

- destination primitive/certificate serialization;
- destination private/public signing-key abstraction;
- exact crypto implementation modules for selected suites;
- destination/LeaseSet signing call sites that currently assume Ed25519;
- focused tests/fixtures.

Do not authorize unrelated RouterInfo signing, transport handshakes, NetDB behavior or broad `crypto/`, `primitives/`, `destination/` directories unless the pre-registration audit proves exact necessity file-by-file.

No I2PControl production change is required in M147 other than tests/planning bookkeeping. M148 owns Proposal mapping.

## 7. Failure, cancellation, restart and contention

Most M147 operations are synchronous cryptographic primitives. Where identity generation/persistence is asynchronous or filesystem-backed:

- validation/generation completes before committed identity replacement;
- atomic write/rename and permissions follow existing secret-store authority;
- cancellation cannot leave a half-written identity considered valid;
- no lock is held across expensive random generation/filesystem operations unless existing key-owner design requires a narrowly reviewed guard;
- concurrent generation for the same persistent identity follows deterministic existing store transaction semantics.

## 8. Work packages

### WP1 — algorithm/domain/security freeze

Produce a table of reference type code/name, key/signature lengths, required dependencies/primitives, current support and security decision. Amend this plan with the exact supported target domain.

### WP2 — exact owner/path audit

Trace every hardcoded Ed25519 assumption relevant to destination creation/signing/verification/persistence. Name exact files and update M061/M062 before production work.

### WP3 — closed neutral signature-suite abstraction

Implement the smallest suite abstraction plus key generation/sign/verify/serialization required by the selected domain. Preserve Ed25519 APIs/behavior where possible to minimize churn.

### WP4 — Destination certificate/serialization integration

Prove public Destination bytes encode the correct key certificate/type/length and parse back exactly.

### WP5 — persistent identity integration

Support transient and persistent generation/import/round-trip for selected suites with atomic secret handling.

### WP6 — interoperability/security evidence

Use deterministic known-answer/reference fixtures for each selected suite and negative cross-type/wrong-length signatures.

### WP7 — closure with zero Proposal promotion

Run core and containment verification. M095 counts MUST remain unchanged. M148 becomes ready only if all selected primitives are real end-to-end.

## 9. Focused tests

After the target domain is frozen, require for every selected suite:

- key generation yields exact expected public/private lengths;
- Destination serialization/certificate type parses round-trip;
- sign + verify succeeds for valid data;
- modified message/signature/key fails;
- wrong suite/type/key length cannot be coerced/fallback;
- known-answer/reference fixture verifies where available;
- existing Ed25519 fixture remains byte-compatible;
- transient destination generation uses selected suite;
- persistent destination save/reload preserves suite and identity;
- malformed import cannot replace valid stored identity;
- private key absent from Debug/error/log output;
- no Proposal matrix row changes.

## 10. Broad verification

The registered amendment must specify exact packages/features for any new crypto dependency. Minimum:

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run relevant no-std checks for every changed core crypto/primitive module if supported.

## 11. Acceptance criteria

M147 closes complete only when:

- exact required signature-suite domain and security decisions are documented;
- exact core paths were authorized before implementation;
- actual transient and persistent destinations can be generated/parsed/signed/verified for every selected suite;
- existing Ed25519 behavior remains compatible;
- secret/persistence/failure invariants are proven;
- no broad containment waiver or Proposal-specific core API exists;
- M095 remains at the incoming counts with zero promotions;
- no high/medium cryptographic correctness/security issue remains.

## 12. Stop conditions

Stop blocked rather than broaden/approximate if:

- required algorithms lack acceptable maintained Rust primitives under project dependency/security policy;
- implementation would require a broad crypto/router identity rewrite unrelated to destination SigType;
- persistence compatibility cannot be preserved without an unplanned migration;
- a requested type can be encoded but not actually generated/signed/verified;
- safety review rejects a legacy algorithm required for full compatibility.

## 13. Closure evidence required

Record the amended algorithm table, exact changed paths/dependencies, known-answer/reference fixtures, key/certificate serialization evidence, transient/persistent round-trip, secret/redaction review, no-std status, containment/dependency diff, proof of zero M095 promotions, implementation SHA, unresolved security decisions and M148 readiness. External access remains read-only.