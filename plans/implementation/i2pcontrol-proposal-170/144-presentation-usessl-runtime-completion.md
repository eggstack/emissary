# M144 — Presentation UseSSL Runtime Completion

Status: **closed as complete** (closure `plans/closure/i2pcontrol-proposal-170/144-closure.md`; registered for execution after M143 closure with the pinned family-direction/trust freeze recorded in the closure)

Class: capability / TLS identity and trust

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- `UseSSL:httpclient`;
- `UseSSL:connectclient`;
- `UseSSL:httpserver`;
- `UseSSL:httpbidirserver`.

Historical authority:

- M131 `PB-TLS-PRESENTATION-01`.

## 1. Objective

Implement Proposal `UseSSL` as the pinned I2PTunnel **application presentation/local-target TLS** behavior for the four applicable HTTP/CONNECT families, without conflating it with either:

- M129 HTTPS/TLS protecting the I2PControl management listener; or
- Yosemite/session TLS protecting the SAM control connection.

The runtime effect must occur at the application endpoint named by the reference family. A SAM `SSL=true` field is not evidence for this milestone.

## 2. Hard dependency and readiness

Hard dependency:

- M143 closed and current matrix/containment authority reconciled.

Before registration, M144 must re-freeze from the pinned Java source, per target family:

- which side of the local application connection is TLS client vs TLS server;
- certificate/key source and default behavior;
- hostname/SNI and certificate-verification behavior for TLS-to-local-target paths;
- whether `httpclient` and `connectclient` share one listener TLS owner or differ;
- whether `httpserver` and `httpbidirserver` share the accepted-stream local-target TLS owner;
- exact interaction with `SSLCertificate` / `SSLKey` fields if those are part of the pinned Proposal/runtime contract.

If any family semantics remain ambiguous, amend/split M144 before registration rather than approximating all four behind one boolean.

## 3. Existing dependency boundary

The `i2pcontrol` feature already owns optional TLS dependencies including `tokio-rustls`, `rustls-pemfile`, and `rcgen`. M144 is expected to use those existing feature-owned dependencies.

No new dependency or Cargo/lockfile change is pre-authorized. If the exact pinned trust/certificate contract cannot be implemented with the existing dependencies, stop and amend the plan/M062 dependency authority before adding a crate.

## 4. Ownership and initial path budget

Expected production ownership remains entirely under I2PControl:

- `emissary-cli/src/i2pcontrol/backends/http_client.rs`;
- `emissary-cli/src/i2pcontrol/backends/connect_client.rs`;
- `emissary-cli/src/i2pcontrol/backends/http_server.rs`;
- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` only for composite configuration;
- `emissary-cli/src/i2pcontrol/backends/runtime/` for one shared presentation-TLS helper if appropriate;
- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/domain/tunnel.rs` / `tunnel_manager.rs` only for exact typed certificate/key/trust validation/round-trip.

No `emissary-core/**`, Yosemite, transport, NetDB, crypto, startup tunnel, frontend or `.github/**` production change is authorized.

The TLS helper must be application-neutral inside I2PControl and must not reuse management-listener policy objects in a way that couples data-plane and administrative TLS lifecycles.

## 5. Security invariants

- `UseSSL` never changes SAM-control transport security.
- M129 management TLS behavior remains unchanged.
- Private TLS keys are redacted in Debug/errors/response-facing raw config and are never copied into non-secret maps.
- Certificate/key files, if used, are validated before listener/target side effects where feasible and are confined according to existing I2PControl secret-path policy.
- No certificate verification bypass (`dangerous`, accept-all, hostname-ignore) is allowed unless the pinned I2PTunnel semantics explicitly require a trust model that is separately reviewed and documented.
- TLS handshake time and concurrent handshakes are bounded by existing generation/task limits and explicit timeouts.
- Failed handshake cannot fall back to plaintext.
- HTTP server local target remains loopback-confined.
- HTTP/CONNECT client listener remains bound only to its already validated local interface.
- TLS errors do not expose key material, peer certificates, credentials or full target destinations.
- Cancellation tears down handshakes and prevents stale generation publication.

## 6. Required semantic decomposition

### 6.1 Client-side families (`httpclient`, `connectclient`)

Freeze whether `UseSSL=true` means the local browser/application connects to an SSL/TLS listener presented by I2PTunnel. If so:

- TLS termination belongs immediately after local accept and before HTTP/CONNECT parsing;
- the accepted plaintext stream passed into existing filters must be indistinguishable from today's post-accept path;
- certificate identity and key ownership are generation-local/configuration-owned;
- listener readiness must not be reported until TLS configuration is valid.

Do not TLS-wrap the I2P stream to the destination unless the pinned family runtime proves that is the Proposal meaning.

### 6.2 Server-side families (`httpserver`, `httpbidirserver`)

Pinned Java `I2PTunnelServer` chooses a regular or SSL socket for its configured local target. If this is the applicable Proposal path:

- accepted I2P stream filtering/admission occurs first as today;
- local loopback target connection is TLS-wrapped according to the pinned trust behavior;
- no SSL-over-SSL exception or port-specific override is assumed without source evidence;
- failure yields the existing bounded server failure/error path, never plaintext fallback.

`httpbidirserver` must reuse the HTTP server side rather than duplicate TLS-to-target logic.

## 7. Validation, persistence and edit semantics

Before allocation/effect:

- `UseSSL` must be boolean;
- required cert/key/trust companions for a selected family must be complete and valid;
- mismatched cert/key or malformed PEM fails before a replacement generation is committed;
- key path/material remains in redacted/secret storage only;
- `UseSSL=false` is explicit disabled behavior and does not require TLS material;
- changing TLS material or `UseSSL` on a running tunnel follows existing transactional restart/last-known-good rules.

If certificate generation is reference-compatible and required, generated material must have bounded lifetime/storage and atomic file replacement. Do not generate certificates merely because `rcgen` is present if the reference expects configured material.

## 8. Failure, cancellation, restart and contention

- TLS config construction happens before bind/connect when possible.
- Handshake timeout is explicit and bounded.
- No lock is held across file reads, bind/connect, handshake or application I/O.
- Cancellation wins over readiness publication.
- Failed edit/restart preserves the previous running generation and TLS identity according to current transaction rules.
- Stale accepted tasks cannot access successor private key/config state.
- Per-generation TLS acceptor/connector state is immutable after readiness unless pinned semantics require live reload; live reload is otherwise out of scope.

## 9. Work packages

### WP1 — family-direction/trust freeze

Record exact pinned Java class/method behavior for all four cells, certificate/key sources and trust rules. Amend this plan if different families require materially separate owners.

### WP2 — shared bounded TLS configuration helper

Build the minimum I2PControl-local configuration loader/validator and acceptor/connector wrapper using existing feature-owned dependencies. Keep secret data out of general tunnel raw-config outputs.

### WP3 — client listener TLS integration

Integrate only the proven client families and preserve existing HTTP/CONNECT parser/filter behavior after TLS termination.

### WP4 — server local-target TLS integration

Integrate the proven server families through the shared accepted HTTP server path while retaining literal-loopback target confinement.

### WP5 — adversarial/runtime evidence

Test malformed PEM, mismatched key, expired/untrusted target cert according to the chosen trust contract, handshake stall, plaintext attempt against TLS listener, cancellation and restart rollback.

### WP6 — matrix/containment/closure

Promote independently proven cells, reconcile machine/docs/registry and exact M061/M062 paths.

## 10. Focused tests

Required:

- `UseSSL=false` exactly preserves current plaintext-local application behavior;
- client TLS listener completes a real TLS handshake before existing HTTP/CONNECT parsing;
- plaintext on a TLS-enabled listener is rejected and never dispatched as HTTP;
- server TLS target performs a real TLS handshake and forwards only after successful verification per the frozen trust contract;
- failed server TLS handshake never retries plaintext;
- malformed/missing/mismatched cert/key rejected before listener/replacement allocation;
- secret material absent from Debug/errors/Get/rawConfig;
- handshake timeout/cancellation bounded;
- old generation retains old TLS state during failed edit, successor gets new state only after successful restart;
- `httpbidirserver` reuses server TLS owner;
- M129 management listener and Yosemite SAM TLS tests remain unchanged/green;
- non-target families remain blocked/N/A.

## 11. Matrix promotion budget

Maximum: **4 cells**, promoted independently from the M143 closure baseline.

A shared TLS helper alone has zero promotion value until each family has the correct externally observable direction/trust behavior.

## 12. Broad verification

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m129_nonloopback_tls --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 13. Acceptance criteria

M144 closes as complete only when:

- exact TLS direction/trust semantics are source-frozen for every promoted family;
- real handshakes occur at the correct local application boundary;
- no plaintext fallback or verification bypass exists;
- management TLS and SAM TLS remain independent;
- secret handling, cancellation, timeout and last-known-good behavior are proven;
- no non-I2PControl production path or new dependency was needed unless a prior plan amendment explicitly authorized it;
- M061/M062/M095/M105/docs/registry are reconciled;
- no medium/high TLS/security defect remains.

## 14. Stop conditions

Stop and leave affected cells blocked if:

- family direction/trust semantics cannot be established from pinned source;
- exact behavior requires insecure certificate acceptance not explicitly approved by project security authority;
- implementation would weaken loopback/local-listener confinement;
- a new TLS stack or core/router change is required without an amended plan;
- TLS would be accept-inert for any target family.

## 15. Closure evidence required

Record pinned family semantic table, TLS identity/trust model, changed paths, secret-redaction review, real handshake/adversarial tests, timeout/cancellation/restart evidence, matrix deltas, containment/dependency review, verification results, implementation SHA, unresolved findings and M145 readiness. External reference access remains read-only.