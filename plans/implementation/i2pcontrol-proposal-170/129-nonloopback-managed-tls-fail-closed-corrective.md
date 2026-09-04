# M129 — Non-Loopback Managed-TLS Fail-Closed Corrective

Status: **closed**

Closure authority: `plans/closure/i2pcontrol-proposal-170/129-closure.md`

Class: corrective / TLS configuration / operational security

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Predecessor authority:

- M107/M108 managed-TLS hardening;
- M126 plan and closure;
- M127 authentication token-lifetime corrective;
- M128 JSON-RPC batch corrective;
- M061/M062 containment authority.

Planning baseline: `9948cfd0782a3defbd5f68cf2d4523603bdc7940`.

Pinned authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- existing I2PControl HTTPS transport requirement and accepted local managed-certificate architecture.

Current Proposal matrix entering M129: `284 apply / 96 blocked_primitive / 460 not_applicable`.

## 1. Objective

Make remote I2PControl exposure fail closed when the operator has not supplied explicit TLS identity material appropriate for remote clients.

At the planning baseline, enabling I2PControl on a non-loopback bind emits a warning but is accepted. When no explicit certificate/key paths are configured, Emissary generates/reuses a managed self-signed certificate whose SAN set is limited to `localhost`, `127.0.0.1`, and `::1`. A standards-validating remote client connecting to a non-loopback address cannot authenticate that managed certificate for the remote address/name.

The dangerous failure mode is operational rather than cryptographic: a seemingly supported remote configuration can pressure operators or client code to disable certificate verification. M129 must reject that configuration before listener startup. Non-loopback serving remains possible only with complete explicit operator-provided TLS material.

M129 does not inspect or certify operator-provided certificate hostname/SAN policy, does not add mTLS, and changes no Proposal 170 matrix cell.

## 2. Readiness and dependency policy

M129 is queued and unregistered while earlier RPC/auth corrective work is active. It is implementation-independent of M128, but should be executed after M127/M128 to keep closure history linear around one shared control-plane boundary.

Registration rule: promote M129 only after M128 closure unless a maintainer explicitly reorders the independent milestones.

Registration note (2026-09-04): M128 closed (`plans/closure/i2pcontrol-proposal-170/128-closure.md`), so M129 is promoted to **ready / registered** as the single active handoff per `plans/003-planning-process.md`.

## 3. Why prior verification missed the defect

M108/M126 verified:

- production TLS-only serving;
- managed key/certificate generation and reuse;
- owner-only managed-directory/key permissions on Unix;
- symlink/special-file rejection;
- no plaintext fallback;
- a warning for non-loopback bind.

They did not test the semantic relationship between the configured bind address and the identity set encoded in the generated managed certificate. Consequently, the implementation proved “TLS is present” without proving “the default TLS identity is usable for the advertised remote endpoint.”

M129 regression evidence must bind those two configuration facts together.

## 4. Ownership and containment

Expected production paths:

- `emissary-cli/src/i2pcontrol/server.rs` for configuration validation;
- `emissary-cli/src/i2pcontrol/tls.rs` only if a small helper is needed to express managed-vs-explicit material state;
- I2PControl tests/docs/planning.

No `emissary-core/**`, `emissary-util/**`, Yosemite, tunnel/proxy/router/transport/frontend, or dependency changes are authorized.

Do not implement a router-wide certificate manager or generic remote administration layer.

## 5. Hard invariants

M129 MUST preserve:

- TLS-only production serving;
- current safe rustls protocol defaults;
- current managed certificate SANs for loopback service unless a separate compatibility need is proven;
- managed certificate/key persistence and permissions;
- managed symlink/special-file confinement;
- explicit certificate and private-key loading behavior;
- no plaintext fallback after any TLS/configuration failure;
- no automatic certificate generation containing wildcard, host-derived, DNS-resolved, interface-enumerated, or request-controlled remote names/addresses;
- no automatic trust-store modification;
- no disabling hostname/certificate verification on clients;
- no Proposal-specific production change outside `i2pcontrol`;
- no matrix change.

## 6. Required production semantics

### 6.1 Loopback bind

A loopback bind may use either:

- complete explicit certificate + private-key paths; or
- the current managed loopback certificate/key path.

Managed certificate behavior remains otherwise unchanged.

### 6.2 Non-loopback bind

If `I2pControlConfig.enabled` and the bind IP is not loopback, configuration validation must require both an explicit certificate path and an explicit private-key path.

A non-loopback bind with:

- no explicit TLS paths;
- certificate only; or
- private key only

must fail before listener bind and before generating/reusing managed TLS material.

The error must clearly state that remote/non-loopback I2PControl requires explicit TLS certificate and key material, without leaking filesystem internals beyond operator-supplied path context already used by explicit TLS errors.

### 6.3 Operator-provided identity

M129 does not parse certificate SANs to decide whether they match the bind IP/hostname. That would require a separate exact policy for DNS names, reverse proxies, wildcard certificates, and interface addresses.

The security guarantee is narrower and explicit: Emissary will no longer present its loopback-only managed identity as though it were a supported remote-service identity.

## 7. Ordered work packages

### WP1 — configuration truth table

Freeze and test the exact matrix:

| Bind | TLS material | Expected |
|---|---|---|
| loopback | none | allowed; managed TLS |
| loopback | cert + key | allowed; explicit TLS |
| loopback | cert only | rejected by existing explicit TLS completeness rules |
| loopback | key only | rejected by existing explicit TLS completeness rules |
| non-loopback | none | reject before TLS generation/listen |
| non-loopback | cert + key | allowed to proceed to explicit TLS loading |
| non-loopback | cert only | reject |
| non-loopback | key only | reject |

Cover IPv4 and IPv6 loopback/non-loopback addresses.

### WP2 — fail-closed validation

1. Add the narrow validation rule at the earliest configuration boundary.
2. Ensure failure precedes listener bind and managed certificate generation/reuse.
3. Preserve existing explicit cert/key load validation.
4. Keep the existing non-loopback warning only if useful after explicit-material validation; it must not be the sole control.

### WP3 — startup and filesystem side-effect evidence

Prove rejected non-loopback managed-TLS startup:

- does not bind the configured port;
- does not create the managed certificate directory/files if they do not already exist;
- does not mutate existing managed TLS material;
- does not start I2PControl tasks;
- emits no password/token/private-key material.

### WP4 — live runtime regression

Extend the child-process/runtime test with:

- loopback managed TLS still starts and accepts valid TLS clients;
- non-loopback without explicit material terminates configuration/startup cleanly;
- explicit material path remains usable for a non-loopback bind in a controlled local test topology where feasible;
- plaintext still never reaches JSON-RPC dispatch.

Do not weaken test certificate verification merely to make remote startup pass.

### WP5 — documentation and operational guidance

Update I2PControl security/config docs to state:

- managed certificates are loopback-only identities;
- non-loopback serving requires explicit certificate and private-key configuration;
- operators remain responsible for issuing material matching the client-visible endpoint/trust model;
- no automatic remote certificate identity synthesis occurs.

## 8. Failure, cancellation, restart, and contention semantics

- Invalid non-loopback/managed configuration fails synchronously before service task creation.
- No cancellation-sensitive runtime state exists before validation completes.
- Restart re-runs the same configuration rule; a previously valid explicit remote configuration remains valid if its material remains readable/valid.
- Existing managed TLS file publication/locking semantics remain unchanged for loopback configurations.
- Multiple simultaneous startup attempts retain existing listener/TLS behavior; M129 adds no shared mutable state.

## 9. Compatibility and migration

Loopback/default installations are unchanged.

Operators currently relying on non-loopback bind plus auto-generated managed TLS will receive a startup/configuration failure and must provide explicit certificate/key material. This is an intentional fail-closed behavior change.

No persisted state migration occurs. Existing managed loopback certificates remain reusable for loopback operation.

## 10. Focused tests

At minimum prove:

- IPv4 loopback + managed allowed;
- IPv6 loopback + managed allowed;
- non-loopback IPv4 + managed rejected;
- non-loopback IPv6 + managed rejected;
- wildcard/unspecified binds (`0.0.0.0`, `::`) are treated as non-loopback and rejected without explicit material;
- complete explicit material passes configuration validation for non-loopback;
- partial explicit material rejects;
- rejected remote configuration creates no managed TLS files or listener;
- loopback managed SAN set remains localhost/loopback only;
- explicit TLS failure never falls back to managed TLS or plaintext;
- error/log output contains no secret material.

## 11. Broad verification

Run and record:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial --test i2pcontrol_live_runtime --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Record known formatter-toolchain limitations without unrelated churn.

## 12. Acceptance criteria

M129 closes only when:

1. managed TLS is accepted only for loopback binds;
2. every non-loopback bind requires complete explicit certificate/key configuration;
3. rejection occurs before listener/task/TLS-file side effects;
4. loopback managed TLS remains operational;
5. explicit TLS remains operational and never silently falls back;
6. plaintext remains unreachable;
7. security/configuration docs describe the new boundary exactly;
8. production changes remain wholly inside `i2pcontrol`;
9. Proposal matrix remains `284 / 96 / 460`;
10. broad verification reports no unexplained regression.

## 13. Stop conditions

Stop and return for separate planning if correctness appears to require:

- automatic interface/DNS discovery;
- generated remote SANs;
- certificate-authority/trust-store management;
- mTLS/client certificates;
- reverse-proxy protocol support;
- core/router changes;
- weakening any existing TLS material confinement or rustls security default.

## 14. Closure evidence required

Closure must record:

- implementation commit(s);
- bind/TLS-material truth table results;
- filesystem/listener side-effect evidence;
- loopback and explicit remote startup evidence;
- plaintext-failure evidence;
- exact verification outcomes;
- compatibility/migration/security review;
- containment review;
- unresolved findings and next-readiness decision;
- internal-only external-interaction attestation.

## 15. External-interaction boundary

External I2P/TLS/reference sources are read-only evidence. Writes are authorized only to `eggstack/emissary`.

No upstream issue, PR, review, discussion, release, submission, merge/adoption request, maintainer contact, contribution package, or third-party repository mutation is authorized.