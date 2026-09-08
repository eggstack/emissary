# M150 — LeaseSet OptionalLookup Completion

Status: **deferred / unregistered; hard-depends on M149 closure**

Class: infrastructure + capability / blinded lookup and secret custody

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- `OptionalLookup` for `server`, `httpserver`, `httpbidirserver`, `ircserver`, and `streamrserver`.

Historical authority:

- M131 `PB-LEASESET-LOOKUP-01`.

## 1. Objective

Implement the exact pinned LeaseSet optional/blinded lookup behavior after M149 establishes the required encrypted LeaseSet publication primitives. The capability must include real lookup-secret/blinded-key derivation and NetDB lookup/decrypt policy where required by the Proposal/reference, not only storage or SAM option serialization.

## 2. Registration gate

M150 MUST NOT be registered until M149 is closed and this plan is amended with:

- exact Proposal value/type semantics for `OptionalLookup`;
- exact Java/I2PTunnel property mapping and relation to encrypted/blinded LeaseSet modes;
- exact I2P blinded-key/lookup-secret derivation specification;
- current Emissary NetDB lookup path and negative-cache/retry ownership;
- exact secret persistence requirements;
- exact core/crypto/NetDB files and dependencies needed.

No broad `netdb/`, `lease_set/` or `crypto/` authorization is granted. Exact files only, after audit.

## 3. Neutral runtime contract

The lower layer must provide standard I2P lookup behavior independent of Proposal terminology:

- derive the correct blinded/lookup key from destination + secret inputs;
- query through the canonical NetDB owner with existing network bounds;
- validate/decrypt the returned LeaseSet according to the selected format;
- distinguish not-found, invalid, unauthorized/decryption failure and cancellation without leaking secrets;
- use bounded negative caching/retry semantics consistent with current NetDB policy;
- never fall back from secret/blinded lookup to a public/plain lookup when the configured mode forbids it;
- preserve ordinary lookup behavior for destinations without optional lookup configuration.

M150 must not add a second NetDB client or polling subsystem.

## 4. Security invariants

- Lookup secrets/blinding private material never appear in logs, errors, Get/rawConfig or public inspection.
- Secret-derived lookup keys are scoped to the intended destination and cannot be substituted between tunnels.
- No plaintext/public lookup downgrade when secret/blinded lookup is required.
- Returned LeaseSet must authenticate to the expected destination/blinded identity and pass freshness/signature checks.
- Negative cache keys must not expose raw secret values and cache cardinality/time are bounded.
- Retry/backoff is bounded and cancellation-aware.
- Failed lookup/decrypt does not replace a last-known-good destination identity or publish fabricated state.
- M149 encrypted LeaseSet publication remains byte/behavior compatible for server paths not using OptionalLookup.

## 5. I2PControl mapping

For the five target server families:

- validate exact `OptionalLookup` type/value and required companion secret fields before allocation;
- map the standard session properties through Yosemite only after the neutral runtime consumer is established;
- persist required secret companions only in approved redacted/secret storage;
- ensure restart/reload reconstructs the same lookup behavior;
- reject invalid combinations with unencrypted/incompatible LeaseSet modes according to pinned reference;
- preserve composite `httpbidirserver` ownership without duplicate lookup state.

Any client-side lookup implications discovered in the pinned spec must be planned separately unless they are intrinsically required to prove the server capability end-to-end and fit the same neutral owner.

## 6. Candidate path categories — not authorization

Before registration, replace with exact files:

- M149 LeaseSet/blinding structures reused without duplication;
- exact blinded-key/secret derivation crypto owner;
- exact NetDB lookup/query/cache owner;
- server destination/session secret storage and I2PControl mapping;
- focused interoperability tests.

No transport/tunnel-building/frontend/startup proxy changes are expected.

## 7. Failure, cancellation, restart and contention

- Secret/combination validation precedes session/network activity.
- NetDB lookup timeout/retry uses existing bounded owner semantics; no unbounded loop.
- Cancellation terminates pending lookup/decrypt and stale completion cannot update a successor generation.
- No NetDB/secret-store lock spans network I/O or expensive cryptographic derivation/decryption.
- Restart reloads the same secret/key material atomically.
- Failed edit/restart preserves prior running generation/secret configuration.

## 8. Work packages

### WP1 — lookup semantics/spec/path freeze

Freeze exact mapping, derivation, NetDB behavior and file budget; split if server publication vs client lookup proves materially separate.

### WP2 — neutral blinded/secret lookup primitive

Implement derivation + canonical NetDB query/decrypt/authentication with deterministic vectors and bounded failures.

### WP3 — secret persistence

Store/reload only required secret material atomically/redacted, with malformed/missing secret rejection.

### WP4 — I2PControl five-family integration

Map Proposal values/combinations and prove real lookup/decrypt behavior through the running session/destination path.

### WP5 — adversarial/interoperability tests

Test wrong secret, wrong destination, stale/tampered LeaseSet, not-found, cancellation, negative cache and restart.

### WP6 — matrix/containment/closure

Promote only cells with end-to-end lookup behavior; reconcile exact sensitive paths and machine/docs/registry.

## 9. Focused tests

Required after semantic freeze:

- known-answer blinded/lookup derivation vector;
- correct secret locates/authenticates/decrypts the intended LeaseSet;
- wrong secret does not fall back to public/plain lookup;
- wrong destination/tampered/expired LeaseSet rejected;
- not-found/negative cache bounded and expires per frozen policy;
- repeated attacker-controlled lookup attempts cannot grow state unboundedly;
- cancellation/restart isolates generations;
- persisted secret reload reproduces lookup key/behavior without exposing secret;
- invalid combination with LeaseSet mode fails pre-allocation;
- five family gates exact;
- ordinary M149 encrypted LeaseSet publication path unaffected when OptionalLookup absent;
- bounded reference/live interoperability fixture.

## 10. Matrix promotion budget

Maximum: **5 cells**, independently proven from M149 closure baseline.

A stored secret or Yosemite option has zero promotion value without actual neutral lookup/decrypt behavior.

## 11. Verification

Registered amendment must add exact core/no-std commands. Minimum:

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

M150 closes complete only when:

- exact OptionalLookup semantics and derivation spec are frozen;
- exact sensitive files were authorized before coding;
- real canonical NetDB lookup/decrypt behavior works with correct secrets;
- no downgrade, secret leak or unbounded retry/cache exists;
- restart/cancellation/negative-cache behavior is deterministic;
- every promoted family consumes the primitive;
- M061/M062/M095/M105/docs/registry match actual behavior;
- no medium/high NetDB/crypto/privacy defect remains.

## 13. Stop conditions

Stop/split and leave cells blocked if:

- exact lookup semantics cannot be established;
- neutral implementation requires a second/broad NetDB subsystem;
- required derivation crypto fails security/dependency review;
- only storage/wire mapping can be implemented;
- any failure path must downgrade to ordinary public lookup.

## 14. Closure evidence required

Record amended semantic/path table, known-answer derivation vectors, actual lookup/decrypt/interoperability evidence, wrong-secret/no-downgrade tests, cache/retry/cancellation/restart evidence, secret review, changed paths/dependencies, matrix delta, containment update, implementation SHA, unresolved findings and M151 readiness. External access remains read-only.