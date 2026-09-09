# M147 — Neutral Destination Signature-Suite Primitive

Status: **closed as blocked (path blocked by M154 disposition C; see `plans/closure/i2pcontrol-proposal-170/154-closure.md`). No production, dependency, or M061/M062 budget is authorised. Re-opening requires a separate explicit architecture/security decision and plan.**

Former gate (superseded): ~~deferred / unregistered; hard-depends on M154 closure/readiness disposition~~

Class: infrastructure / cryptographic identity security

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`;
- historical source line: `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Promotion budget: **zero Proposal cells**.

Historical authority:

- M121 Outcome C proved that fixed Ed25519/type-7 support is not truthful configurable `SigType` support;
- M131 `PB-DEST-SIGTYPE-01` records the missing neutral signing/key-generation primitive;
- M154 is the mandatory pre-registration audit for signature domain, algorithm security, persistence and exact-file ownership.

## 1. Objective

Implement the minimum neutral Emissary destination signing/key-generation infrastructure required for a later truthful Proposal `SigType` implementation, without putting Proposal policy into core and without promoting any M095 row in this milestone.

The primitive must support actual key generation, Destination certificate/type encoding, signing and verification interoperability for the exact signature-type domain accepted by the M154 closure. Merely parsing a type code or carrying it through Yosemite is not infrastructure completion.

## 2. Hard dependency and registration gate

M147 MUST NOT be registered merely because M146 is closed.

Hard dependency:

- M154 closed complete with a disposition that explicitly selects one bounded M147 implementation rather than split/block.

Before M147 registration, M154 must have amended this plan with:

1. exact signature-type numeric/name domain required by the pinned Proposal/reference;
2. exact subset accepted by project cryptographic security policy;
3. current Emissary generation/sign/verify/serialize/persist capability per type;
4. maintained cryptographic primitive/dependency selected per missing type;
5. exact files owning private signing keys, Destination certificates/serialization, signing, verification and persistent destination handling;
6. persistence/import/export compatibility or migration contract;
7. exact-file M061/M062 production authorization;
8. exact known-answer/reference fixtures and no-std obligations.

If M154 chooses to split the work, this plan remains unregistered and the split successors replace it in the dependency graph. If M154 closes the path blocked, M147 remains unregistered and the ten SigType cells remain blocked.

No broad `emissary-core/src/crypto/**`, `primitives/**` or `destination/**` authorization is permitted.

## 3. Neutral architecture contract

Core must express standard I2P cryptographic identities, not Proposal values. The preferred shape is a small closed signature-suite/type abstraction owned by existing primitive/crypto code with operations required by real destinations:

- generate private/public signing key pair;
- derive/serialize the Destination signing public key and key certificate;
- parse/validate encoded keys/certificates;
- sign the I2P structures/messages currently signed by destination identity;
- verify corresponding signatures;
- expose type/length information needed by serialization without unchecked dynamic allocation;
- preserve existing Ed25519 behavior exactly when the new abstraction is not selected.

Avoid a generic plugin-style crypto framework when a closed I2P signature-suite enum is sufficient.

M147 must not modify router identity algorithms unless M154 proves an unavoidable shared neutral owner and explicitly authorizes the exact file. Destination SigType does not imply router-identity migration.

## 4. Security invariants

- Private signing keys never expose raw bytes via Debug/Display/errors/logs.
- Secret buffers follow existing zeroization/secret-storage policy; any new dependency requires explicit M154/M062 authorization.
- Algorithm/type and key lengths validate before allocation/use.
- No implicit fallback to Ed25519 or another algorithm on parse/generation/sign failure.
- Parsed certificate type must match actual key material and signature implementation.
- Invalid/malleable encodings are rejected according to the pinned I2P specification.
- Existing Ed25519 destination byte compatibility remains regression-tested.
- Core contains no Proposal/I2PControl/TunnelManager/JSON-RPC vocabulary.
- No algorithm is enabled solely because a numeric code exists if M154 rejected it on security/maintenance grounds.
- Random generation uses the existing approved runtime RNG/cryptographic randomness boundary.

## 5. Persistence and compatibility

M154 must freeze the exact persistence contract before registration. M147 then implements only that accepted contract.

Required invariants:

- existing Ed25519 persisted destinations remain readable and byte-compatible;
- new suites persist all required type/certificate/private-key metadata atomically;
- import validates actual key/certificate type and cannot be relabeled by the caller;
- export/round-trip, where supported, preserves suite and secret material exactly;
- malformed/truncated/wrong-length keys fail before replacing last-known-good identity;
- no automatic migration rewrites existing identities merely because M147 lands;
- cancellation/failure cannot leave a half-written identity considered valid.

If M154 determined an incompatible storage migration is needed, that migration must already be explicitly described and authorized before M147 starts.

## 6. Production ownership

The exact path list is **not defined by this template plan**. M154 closure must insert the final file-by-file budget before registration.

Candidate categories only:

- destination primitive/certificate serialization;
- destination signing private/public key abstraction;
- exact crypto modules for selected suites;
- exact destination/LeaseSet signing call sites currently hardcoded to Ed25519;
- persistent destination storage/import/export owner;
- focused fixtures/tests.

Adjacent router identity, transport handshake, NetDB, tunnel-pool and unrelated crypto files remain prohibited unless individually proven necessary and added by plan amendment before coding.

No I2PControl production change is expected in M147 except tests/planning bookkeeping. M148 owns Proposal mapping.

## 7. Failure, cancellation, restart and contention

Most cryptographic primitives are synchronous. Where identity generation/persistence uses async/filesystem work:

- validate/generate completely before committed identity replacement;
- atomic write/rename and file permissions follow existing secret-store rules;
- cancellation cannot leave partial identity state accepted;
- no lock spans expensive random generation, filesystem I/O or cryptographic work unless a narrowly reviewed existing owner requires it;
- concurrent creation for the same persistent identity uses deterministic existing transaction semantics;
- restart cannot silently reinterpret a stored suite as another suite.

## 8. Work packages

### WP1 — consume M154 freeze

Verify the registered M154 closure/amendment exactly matches this plan's production/dependency budget. Stop on mismatch rather than widening scope.

### WP2 — closed neutral suite abstraction

Implement the smallest selected suite abstraction plus key generation/sign/verify/type/length operations.

### WP3 — Destination certificate/serialization integration

Encode and parse exact standard I2P key-certificate/signing-key representation for each selected suite.

### WP4 — signing call-site integration

Replace only the hardcoded Ed25519 assumptions identified by M154 that are necessary for actual destination operation.

### WP5 — persistent identity integration

Implement transient and persistent generation/import/round-trip according to the frozen storage contract.

### WP6 — interoperability/security evidence

Use deterministic known-answer/reference fixtures and cross-type/wrong-length/tamper negatives for every selected suite.

### WP7 — closure with zero Proposal promotion

Run core/no-std/containment verification. M095 cell counts MUST remain unchanged. M148 becomes dependency-ready only if the complete selected primitive works end-to-end.

## 9. Focused tests

For every selected suite require:

- key generation produces exact expected key lengths;
- Destination serialization/certificate type parses round-trip;
- sign + verify succeeds for valid data;
- modified message/signature/key fails;
- wrong suite/type/key length cannot coerce/fallback;
- known-answer/reference fixture verifies;
- existing Ed25519 fixture remains byte-compatible;
- transient destination generation uses selected suite;
- persistent save/reload preserves suite and identity;
- malformed import cannot replace valid stored identity;
- private key absent from Debug/error/log output;
- no Proposal matrix row changes.

Add regression coverage for every hardcoded assumption identified by M154.

## 10. Broad verification

The registered M154 amendment must specify exact packages/features for any new crypto dependency. Minimum:

```text
cargo check -p emissary-core
cargo check -p emissary-core --no-default-features --features no_std
cargo test -p emissary-core --lib --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run dependency-specific security/known-answer tests required by M154. Clippy/fmt drift outside touched files must not be normalized opportunistically.

## 11. Acceptance criteria

M147 closes complete only when:

- every M154-selected suite can actually generate, serialize, sign, verify and persist a destination identity;
- exact core paths/dependencies were authorized before implementation;
- existing Ed25519 behavior remains compatible;
- secret/persistence/failure invariants are proven;
- no broad containment waiver or Proposal-specific core API exists;
- no hidden router-identity/network behavior changes occurred;
- M095 remains at the incoming counts with zero promotions;
- no high/medium cryptographic correctness/security issue remains.

## 12. Stop conditions

Stop blocked rather than broaden/approximate if:

- implementation requires a suite M154 rejected;
- a maintained/safe primitive is unavailable;
- a broad crypto/router identity rewrite is required beyond the exact M154 budget;
- persistence compatibility cannot be preserved under the approved contract;
- a type can be encoded but cannot actually generate/sign/verify;
- no-std or target-platform constraints invalidate the selected primitive;
- exact-file containment cannot be maintained without a broad waiver.

## 13. Matrix and successor rule

M147 has zero support-promotion budget. Successful infrastructure does not itself change `SigType` cells.

M148 may be registered only after M147 closure proves the real neutral primitive and the registry explicitly advances it. If M147 closes blocked or only a subset of required suites is acceptable, M148 must be amended/split or remain blocked accordingly.

## 14. External-interaction boundary

All specification/reference/source research remains read-only. M147 authorizes no upstream issue/PR/review/contact/submission. Repository writes remain internal to `eggstack/emissary`.

## 15. Closure evidence required

Record:

- M154 closure/amendment authority;
- exact changed paths/dependencies;
- per-suite known-answer/reference fixtures;
- key/certificate serialization and transient/persistent round-trip evidence;
- secret/redaction review;
- no-std status;
- M061/M062 exact diff;
- proof of zero M095 promotions;
- implementation SHA;
- unresolved security findings;
- M148 readiness disposition;
- external read-only attestation.
