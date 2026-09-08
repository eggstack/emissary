# M148 — Proposal SigType Completion

Status: **deferred / unregistered; hard-depends on M147 closure**

Class: capability / destination identity security

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- `SigType` for `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `server`, `httpserver`, `httpbidirserver`, and `ircserver`.

Historical authority:

- M121 Outcome C;
- M131 `PB-DEST-SIGTYPE-01`;
- M147 neutral signature-suite primitive closure.

## 1. Objective

Expose Proposal `SigType` only after M147 proves real end-to-end generation/signing support for the exact accepted suite domain. The selected value must determine the actual destination identity/key certificate and signatures used by the running tunnel; no value may silently fall back to Ed25519/type 7.

## 2. Hard dependency and registration amendment

M147 must be closed as complete. Before M148 registration, amend this plan with:

- exact supported numeric/textual SigType values proven by M147;
- exact Proposal/Java parser normalization rules;
- M147 persistent/transient identity API used by I2PControl;
- exact incoming M095 baseline;
- any Yosemite session-option mapping required for the router session to agree with the generated destination type.

If M147 closes with a security-approved subset smaller than the Proposal-required domain, M148 must explicitly decide whether that subset can truthfully satisfy the configurable field. Do not repeat M111's singleton-domain mistake.

## 3. Runtime contract

For a supplied valid SigType:

- transient client identity generation uses exactly that suite;
- persistent client/server destination generation uses exactly that suite;
- imported `PrivKeyFile` / stored destination material is validated against its intrinsic signature type and cannot be relabeled;
- Yosemite/SAM session creation receives the matching standard signature-type setting if the pinned wire requires it;
- shared client sessions can share only when identity/signature-suite requirements are compatible;
- Get/round-trip returns the canonical Proposal representation;
- unsupported/noncanonical values fail before listener/session/key allocation or persistent identity mutation.

For an omitted SigType, preserve the current default destination suite exactly.

## 4. Initial path budget

Expected I2PControl paths:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` and accepted-server session composition as needed;
- `emissary-cli/src/i2pcontrol/client_secret_store.rs` / `server_secret_store.rs` only where suite-aware generation/import belongs to existing I2PControl identity custody;
- ten applicable backend files only where family-specific identity construction requires explicit wiring;
- `domain/tunnel.rs` / `tunnel_manager.rs` for exact validation/round-trip;
- matrix/tests/docs.

M148 should consume M147's neutral API without new core crypto behavior. Any additional core/crypto change is a stop condition requiring M147 corrective/amendment.

No new dependency or Yosemite source change is pre-authorized.

## 5. Invariants

- No fallback/coercion to type 7.
- Generated Destination key certificate and signing behavior match requested type.
- Imported/stored key material is authoritative for its intrinsic type; conflicting config fails.
- Existing identities are not regenerated on ordinary restart/edit unless current identity policy already requires replacement.
- NewDest/idle-resume rotation preserves configured SigType for the successor identity.
- PersistentClientKey preserves both identity and SigType across process restart.
- Shared definitions cannot combine incompatible SigType/identity requirements.
- Secret key bytes remain redacted and atomically stored.
- Streamr families remain outside the ten-cell target unless machine authority changes through a separate applicability plan.
- Existing `SignatureType`/other similarly named fields are not conflated with Proposal `SigType` unless pinned authority says they are the same field.

## 6. Failure, cancellation, restart and contention

- SigType validation completes before any key/session/listener allocation.
- Persistent key generation is transactionally committed only after validation and according to existing secret-store atomicity.
- Failed session start after newly generated persistent identity follows existing commit/rollback authority; do not orphan or rotate keys outside the documented generation transaction.
- Cancellation cannot publish a partially created identity/session.
- Concurrent starts for a shared/persistent identity use deterministic existing store/session ownership and cannot generate competing committed keys.
- No store/session lock spans SAM/network I/O.

## 7. Work packages

### WP1 — consume M147 domain

Freeze exact accepted Proposal values and their M147 suite mapping. Reject aliases not explicitly accepted by pinned Proposal/reference.

### WP2 — validation and identity generation/import

Replace the current blanket SigType rejection with family-gated exact validation and suite-aware transient/persistent/import handling.

### WP3 — session wire/runtime agreement

Ensure Yosemite/session creation receives matching standard type metadata where required. Decode the resulting public Destination to prove the actual suite rather than asserting command text only.

### WP4 — lifecycle/shared identity integration

Re-prove persistent restart, NewDest successor, shared session compatibility and server destination publication with non-default selected suites.

### WP5 — matrix/containment/closure

Promote only families with end-to-end actual selected identity behavior, reconcile current machine/docs/registry and exact M061/M062 paths.

## 8. Focused tests

For each M147-supported Proposal value and representative client/server families:

- exact valid input accepted;
- canonical public Destination reports expected key certificate/type;
- real sign/verify path uses selected suite;
- unsupported/noncanonical/overflow/name alias fails pre-allocation with no value echo;
- omitted value retains default Ed25519/current behavior;
- imported identity matching type succeeds;
- imported identity conflicting type fails without mutation;
- persistent identity reload retains same Destination and suite;
- NewDest successor changes identity but retains configured suite;
- shared session same suite compatible, conflicting suite rejected;
- Get/round-trip exact;
- server published HostingDestination matches selected suite;
- no private key appears in diagnostics/responses.

At least one end-to-end fake/live SAM test must inspect the actual generated Destination, not only the serialized `SIGNATURE_TYPE` field.

## 9. Matrix promotion budget

Maximum: **10 cells**, promoted independently from the M147 closure baseline.

If a target family cannot generate/use the selected suite end-to-end, leave that family blocked even if common parsing is complete.

## 10. Verification

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 11. Acceptance criteria

M148 closes complete only when:

- the accepted value domain is explicitly justified by M147 + pinned Proposal/reference;
- every promoted family uses the requested suite in actual destination/signature behavior;
- imports/persistence/NewDest/shared-session semantics are exact;
- no fallback/inert acceptance remains;
- no new unplanned core crypto behavior was added;
- M061/M062/M095/M105/docs/registry match actual paths/support;
- no medium/high identity/cryptographic defect remains.

## 12. Stop conditions

Stop and leave affected cells blocked if:

- M147's supported domain is insufficient for truthful configurable semantics;
- Yosemite/router session cannot use a generated supported suite end-to-end;
- imported/persistent format cannot unambiguously preserve type;
- implementation would require additional core crypto outside M147 closure;
- any value must be silently coerced/fallback.

## 13. Closure evidence required

Record exact accepted domain, generated Destination/certificate fixtures, actual signing evidence, import/persistence/NewDest/shared-session tests, changed paths, matrix deltas, containment review, broad verification, implementation SHA, unresolved suite limitations and M149 readiness. External reference access remains read-only.