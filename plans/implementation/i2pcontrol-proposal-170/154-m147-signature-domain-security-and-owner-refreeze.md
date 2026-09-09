# M154 — M147 Signature Domain, Security, and Exact-Owner Re-freeze

Status: **registered / dependency-ready; hard dependency on M153 closure satisfied by `plans/closure/i2pcontrol-proposal-170/153-closure.md`**

Class: invariant / architecture-security qualification

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Target successor:

- `plans/implementation/i2pcontrol-proposal-170/147-neutral-destination-signature-suite-primitive.md`.

Promotion budget: **zero Proposal cells**.

Production budget: **zero production code/dependency changes**.

## 1. Objective

Satisfy M147's explicit pre-registration gate before any security-sensitive destination-signature implementation begins.

M154 freezes, from the pinned Proposal/reference and current Emissary source:

1. the exact Proposal `SigType` value domain relevant to the ten applicable cells;
2. the exact signature algorithms that must be supported for truthful compatibility;
3. the project's security disposition for each algorithm, especially legacy algorithms;
4. current Emissary generate/sign/verify/serialize/persist capability per algorithm;
5. the exact destination/public-certificate/private-key owners and every hardcoded Ed25519 assumption that M147 would need to change;
6. persistent destination/import/export compatibility and migration implications;
7. the smallest exact-file M061/M062 production budget and any dependency changes required for M147;
8. whether M147 is dependency-ready, must be split, or must close blocked under current security/dependency policy.

M154 must not implement crypto and must not promote `SigType` cells.

## 2. Hard dependency and baseline

Hard dependency:

- M153 closed complete as the current post-M146 runtime/security qualification authority.

Entry baseline must be mechanically taken from the M153 closure rather than assumed. Expected pre-crypto state if M153 closes without runtime findings:

- matrix `336 apply / 29 blocked_primitive / 475 not_applicable`;
- 10 `SigType` blockers;
- 15 LeaseSet-security blockers;
- 4 terminal M146 `UseOutproxyPlugin` blockers;
- no registered capability plan.

If M153 finds a production corrective requirement, M154 remains deferred until that corrective closes and current-head authority is restored.

## 3. Pinned evidence to freeze

Use read-only evidence from:

- Proposal 170 revision `2026-05-20`;
- Java I2PControl Proposal-170 reference head `45bb593000408071dd376b78848fdc246dccd964`;
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- applicable I2P signature/key-certificate specifications pinned by content hash/date in the M154 closure;
- current Emissary crypto/primitives/destination persistence source;
- Yosemite Y005 only for exact session/control transport capability, not as evidence of Emissary signing support.

External access remains read-only.

## 4. Exact `SigType` domain table

Produce a machine-readable or closure-embedded table with, for every Proposal/reference accepted type:

- canonical Proposal spelling/value;
- I2P numeric signature type;
- public-key length;
- private-key length;
- signature length;
- key-certificate encoding requirements;
- current Emissary parse/verify support;
- current Emissary generation support;
- current Emissary signing support;
- current persistent destination support;
- maintained Rust primitive/dependency candidate if missing;
- cryptographic security disposition: `required_and_acceptable`, `legacy_compatibility_only`, `rejected_by_policy`, or `not_required_by_pinned_contract`;
- exact reference evidence.

Do not infer capability from an enum constant or verifier alone.

## 5. Current-code owner audit

Trace exact files for:

- `SigningPrivateKey` / signing public-key representation;
- key generation;
- Destination key certificate construction and serialization;
- Destination parsing/type validation;
- LeaseSet/LeaseSet2 signing call sites that depend on destination signing keys;
- streaming/SAM destination identity generation;
- persistent private-destination storage/import/export;
- test fixtures/known-answer material;
- any router-identity use of the same types that must remain untouched.

Every required production file must be listed individually. Broad prefixes such as `emissary-core/src/crypto/**`, `primitives/**`, `destination/**` are prohibited as M147 authorization.

## 6. Security/dependency decision gate

For each missing algorithm:

- identify a maintained Rust implementation or prove an existing dependency already supplies it;
- record no-std/std compatibility where relevant;
- record side-channel/key-zeroization properties relied upon;
- assess dependency ownership and whether it can remain neutral/core-owned rather than I2PControl-owned;
- reject unmaintained, unsafe, FFI-heavy or policy-incompatible dependencies unless separately accepted by architecture/security authority.

If the pinned contract requires an algorithm the project refuses to implement for security reasons, M154 must say so explicitly. Do not silently map it to Ed25519 or narrow the Proposal value domain.

A security-policy rejection means the affected `SigType` cells remain blocked and M147 must be split or closed blocked; full Proposal support remains partial.

## 7. Persistence/import/export freeze

Determine the exact format used by current transient and persistent destinations and answer:

- where signature type is encoded;
- whether current private-destination bytes already self-describe enough to round-trip another suite;
- whether existing stored Ed25519 destinations remain byte-compatible;
- whether additional suites require a versioned storage format or only standard I2P key-certificate/private-key bytes;
- how malformed/wrong-type/truncated imports are rejected;
- whether a type change on a running/persistent tunnel creates a new destination, requires restart, or conflicts with persistent-key semantics.

Any incompatible local storage migration must be explicitly planned before M147 registration.

## 8. M147 decomposition decision

M154 closure must choose exactly one disposition:

### A. Register M147 unchanged-but-amended

Allowed only if one bounded neutral signature-suite primitive can cover the required acceptable domain with an exact path/dependency budget.

Amend M147 before registration with:

- exact algorithm table;
- exact-file production paths;
- exact dependencies/features;
- persistence contract;
- known-answer/reference fixtures;
- no-std obligations;
- per-suite stop conditions.

### B. Split M147

Required if algorithms have materially independent dependencies/security/storage semantics. Create bounded infrastructure milestones and keep M148 blocked behind all required ones.

### C. Close M147 path blocked

Required if the pinned required domain cannot be implemented safely within project security policy or compatible persistence boundaries. Keep ten SigType cells blocked and update the residual roadmap/final M152 expectations truthfully.

M154 must not choose A merely because a parser/serializer can encode the type.

## 9. M061/M062 pre-authorization rules

M154 may update planning metadata to name candidate exact paths, but **must not make them executable production authority unless and until M147 is explicitly registered**.

Preferred ceremony:

- M154 closure records the exact candidate M147 path/dependency list;
- M147 plan is amended with the same list;
- registry explicitly registers M147;
- M061/M062 are amended in that registration commit so the executable allowance and registered plan become effective together.

No directory-level exception is permitted.

## 10. Work packages

### WP1 — reference domain freeze

Build the exact SigType/reference table and pin all specification/source hashes.

### WP2 — current Emissary capability inventory

Trace generate/sign/verify/serialize/persist behavior per suite, not just type declarations.

### WP3 — dependency/security review

Evaluate required primitives and security policy, including legacy types.

### WP4 — persistence/migration audit

Freeze transient/persistent/import/export semantics and compatibility.

### WP5 — exact owner/path graph

Name every exact production file M147 would require and explicitly list adjacent prohibited files that do not need modification.

### WP6 — readiness disposition

Amend/split/block M147 as evidence requires. If A, register M147 and only M147. If B, register only the first dependency-ready infrastructure slice. If C, leave SigType blocked and advance no crypto implementation.

### WP7 — closure

Write `plans/closure/i2pcontrol-proposal-170/154-closure.md` with zero production diff and zero matrix promotions.

## 11. Verification

M154 is planning/audit work. At minimum:

```text
cargo check -p emissary-core
cargo check -p emissary-core --no-default-features --features no_std
cargo test -p emissary-core --lib --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
git diff --check
```

No new crypto dependency may be added during M154.

## 12. Acceptance criteria

M154 closes complete only when:

- exact Proposal/reference SigType domain is frozen;
- every required algorithm has an explicit security disposition;
- current Emissary capability per suite is proven at generate/sign/verify/serialize/persist layers;
- exact private/public/certificate/persistence owners are named file-by-file;
- dependency/no-std/security implications are explicit;
- persistence/migration semantics are explicit;
- M147 disposition A/B/C is made with no ambiguity;
- zero production code/dependency changes occurred;
- M095 counts/cells are unchanged;
- no upstream mutation/contact/submission occurred.

## 13. Stop conditions

Stop and leave M147 unregistered if:

- required algorithm/reference behavior remains ambiguous;
- exact file ownership cannot be bounded;
- a broad crypto refactor appears necessary before a bounded plan can be written;
- required maintained Rust primitives cannot be identified;
- storage compatibility cannot be frozen;
- security policy rejects a required type and no accepted compatibility path exists.

## 14. Closure evidence required

Record pinned references/hashes, full algorithm/security table, current capability matrix, exact owner/path/dependency list, persistence findings, M147 A/B/C disposition, any amended successor plan(s), M061/M062 registration changes if and only if a successor is actually registered, zero-promotion proof, zero-production-diff proof, unresolved findings and external read-only attestation.
