# M151 — LeaseSet ClientAuths Completion

Status: **deferred / unregistered; hard-depends on M150 closure**

Class: infrastructure + capability / LeaseSet authorization security

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- `LeaseSetClientAuths` for `server`, `httpserver`, `httpbidirserver`, `ircserver`, and `streamrserver`.

Historical authority:

- M131 `PB-LEASESET-AUTH-01`.

## 1. Objective

Implement the exact pinned LeaseSet client-authorization modes on top of the M149 encrypted LeaseSet and M150 lookup/blinding primitives, including real authorization key material, publication/decryption interoperability, restart-safe custody and bounded client-entry handling.

A parsed authorization list, stored key, or Yosemite property alone is not support.

## 2. Registration gate

M151 MUST NOT be registered until M150 is closed and this plan is amended with:

- exact Proposal `LeaseSetClientAuths` schema/value representation;
- exact Java/I2PTunnel mode mapping and distinction between PSK and DH authorization where applicable;
- maximum/reference client entry semantics and any label/identifier behavior;
- exact I2P encrypted LeaseSet client-auth cryptographic specification;
- exact key generation/import/export requirements for server and authorized clients;
- exact Emissary files/dependencies required for encryption/publication and client decryption/interoperability;
- secret-store format and migration implications.

No broad crypto/NetDB/LeaseSet path authorization is granted. Exact files only after the audit.

## 3. Neutral authorization contract

The lower layer must model standard I2P encrypted LeaseSet client authorization independent of Proposal terminology. Required capabilities for the selected pinned modes include:

- validate/generate/import authorization key material at exact lengths/types;
- encode bounded authorized-client metadata/key material into the encrypted LeaseSet format;
- decrypt/authenticate from an authorized client fixture/reference path;
- reject unauthorized/wrong-mode/wrong-key clients without revealing which secret component failed;
- preserve destination/signing/encryption identity separation;
- renew/rebuild LeaseSets with the same authorization policy until config changes;
- persist only required private authorization material atomically and redacted;
- leave ordinary encrypted LeaseSets from M149 unchanged when client auth is absent.

If PSK and DH modes have materially different primitives/owners, split M151 before registration rather than combining them under a generic map.

## 4. Security invariants

- Authorization keys/secrets never appear in logs, errors, Get/rawConfig, metrics or public inspection.
- Entry count, labels/identifiers and key lengths are bounded before allocation.
- Duplicate/conflicting entries have deterministic pinned behavior.
- Secret/key comparison/derivation uses appropriate constant-time library primitives.
- No unauthorized client can decrypt/access the LeaseSet through fallback to non-authenticated mode.
- Failure to build an authenticated LeaseSet never publishes a less restricted LeaseSet.
- Removing auth through a successful edit/restart follows exact reference semantics; failure preserves last-known-good restricted publication.
- Key rotation/restart cannot mix old/new authorization sets within one published generation.
- M149 confidentiality and M150 lookup secrets remain separate from client-auth key custody unless the I2P format explicitly derives one from the other.
- No user-controlled client list can cause unbounded cryptographic work or persistent state growth.

## 5. I2PControl mapping and storage

For the five target server families:

- parse the exact Proposal structure and reject malformed/noncanonical entries before allocation;
- normalize only as the pinned reference permits; do not invent aliases;
- store private/auth material in approved redacted secret storage, not response-facing `raw_config`;
- preserve safe public/non-secret round-trip fields if the Proposal requires Get visibility;
- map required standard Yosemite/session options only after neutral runtime support is real;
- enforce compatibility with `EncryptLeaseSet`/`OptionalLookup` modes before start/restart;
- `httpbidirserver` reuses the server destination authorization owner.

## 6. Candidate path categories — not authorization

Before registration replace with exact files:

- M149 encrypted LeaseSet structures/crypto reused;
- M150 blinding/lookup primitives reused;
- exact client-auth crypto/serialization owner;
- exact server LeaseSet publication owner;
- I2PControl server secret store/config validation;
- reference-client interoperability fixture.

No unrelated transport, tunnel-building, frontend, startup proxy or RouterInfo change is expected.

## 7. Failure, cancellation, restart and contention

- Full list/key validation occurs before key generation/publication/listener readiness.
- Authorization list size bounds cap CPU/memory before cryptographic work.
- Key generation/import and persistence are transactional; cancellation cannot commit a partial set as current.
- LeaseSet publication switch from old to new auth set is generation-consistent.
- No lock spans filesystem/network I/O or bulk crypto across all entries.
- Failed edit/restart leaves prior restricted generation and key set intact.
- Stale generation cannot republish old authorization state after successor becomes current.

## 8. Work packages

### WP1 — exact mode/schema/spec freeze

Freeze PSK/DH modes, Proposal representation, limits, key formats and exact owners. Split the plan if mode independence warrants separate milestones.

### WP2 — neutral client-auth crypto/format primitive

Implement bounded authorization-entry representation, required derivation/encryption/decryption and known-answer vectors.

### WP3 — publication integration

Build renewed encrypted LeaseSets with the configured authorization set and prove no downgrade on errors.

### WP4 — secret persistence/config mapping

Integrate I2PControl parsing, redacted storage, restart and edit transactionality for five families.

### WP5 — interoperability/adversarial tests

Authorized client succeeds; unauthorized/wrong key/wrong mode/tampered object fails; entry floods bounded; rotation/restart consistent.

### WP6 — matrix/containment/closure

Promote only end-to-end proven families; reconcile sensitive exact paths/dependencies, machine matrix/docs/registry.

## 9. Focused tests

For every selected auth mode:

- known-answer key/derivation/encryption fixture;
- valid authorized client decrypts/authenticates the intended LeaseSet;
- unauthorized, wrong key, wrong mode, tampered data fail with sanitized indistinguishable errors where appropriate;
- max-entry boundary succeeds and max+1 rejects pre-allocation;
- duplicate/conflicting entry behavior exact;
- malformed key length/encoding rejects with no secret echo;
- published LeaseSet never downgrades to unauthenticated on construction failure;
- renewal preserves authorization set;
- successful rotation changes auth set atomically;
- failed edit/restart retains old restricted generation;
- process restart reloads required private material;
- no key in diagnostics/Get/rawConfig;
- five family gates exact;
- reference/live authorized-client interoperability evidence.

## 10. Matrix promotion budget

Maximum: **5 cells**, independently proven from M150 closure baseline.

Shared crypto/storage infrastructure does not promote a family until an actual authorized client can interoperate with that family's published LeaseSet.

## 11. Verification

Registered amendment must add exact core/no-std/dependency checks. Minimum:

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

M151 closes complete only when:

- exact Proposal schema/auth modes and I2P cryptographic specs are frozen;
- exact sensitive paths/dependencies were authorized before coding;
- real authorized-client interoperability works for every claimed mode/family;
- unauthorized access and downgrade are impossible under tested failure paths;
- entry/work/state bounds, secret custody, rotation, restart and cancellation are proven;
- M149/M150 semantics remain intact;
- M061/M062/M095/M105/docs/registry match behavior;
- no medium/high authorization/crypto/privacy defect remains.

## 13. Stop conditions

Stop/split and leave affected cells blocked if:

- PSK/DH mode requirements cannot be established;
- required cryptography/dependency fails security review;
- client interoperability cannot be demonstrated;
- only storage/wire mapping can be completed;
- authorization failure would require fallback to unauthenticated/plain LeaseSet;
- bounded client-entry semantics cannot preserve the required contract.

## 14. Closure evidence required

Record amended mode/schema/path table, known-answer vectors, authorized/unauthorized interoperability, list bounds, no-downgrade tests, secret/rotation/restart/cancellation review, changed paths/dependencies, matrix delta, containment update, implementation SHA, unresolved findings and M152 readiness. External access remains read-only.