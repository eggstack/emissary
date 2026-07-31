# M020 — Base I2PControl and JSON-RPC Interoperability

Status: closed

Primary class: invariant/capability corrective pass

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Prior closure defect record:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

Canonical authority:

- existing I2PControl authentication/error contract
- pinned Proposal 170 revision dated `2026-05-20`
- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`

Repository baseline: implementation head recorded by the invalidated M019A closure, plus current planning-only commits.

## 1. Bounded objective

Restore exact existing-I2PControl interoperability at the common JSON-RPC boundary before further Proposal 170 method corrections.

This milestone owns:

- standard `Authenticate` request and response behavior;
- standard token extraction from protected method `params`;
- exact I2PControl authentication/version error distinctions;
- JSON-RPC notification execution with response suppression;
- request-ID validation without silent coercion;
- direct RouterInfo selector compatibility after removing the authentication token;
- literal base-protocol fixtures that all later Proposal 170 milestones consume.

It does not change Proposal 170 method-specific mutation, persistence, tunnel, address-book, service, or telemetry semantics.

## 2. Current evidence and defects

Current production behavior requires a nonstandard username, returns `API` as a string, and accepts authentication only through `X-I2PControl-Token`. Protected methods therefore reject requests from standard I2PControl clients that place `Token` in `params`.

The dispatcher returns a generic application error rather than distinguishing missing, unknown, expired, invalid-password, missing-version, and unsupported-version cases.

Requests with absent/null IDs are returned as HTTP 204 before dispatch, so notifications do not execute. Request IDs outside the supported string/integer/null domain may be coerced to null or zero.

Direct RouterInfo parsing accepts only the Proposal 170 addition set and treats standard `Token` or existing direct selectors as unknown parameters.

## 3. Required invariants

1. `Authenticate` accepts the standard `API` and `Password` parameters.
2. `Username` is not required. An already-shipped username compatibility field may be accepted only when it is ignored or validated without changing canonical behavior.
3. Successful authentication returns `result.Token` as a string and `result.API` as a JSON number.
4. Every protected request may authenticate through `params.Token` exactly as existing I2PControl clients expect.
5. `X-I2PControl-Token` may remain only as a separately documented compatibility extension.
6. Authentication metadata is removed before method-specific selector/mode validation.
7. Missing token and unknown token produce the exact distinct I2PControl error codes and sanitized messages.
8. API version omission and unsupported version produce distinct exact errors.
9. Notifications execute normal validation and side effects but produce no JSON-RPC response body.
10. Invalid request IDs are rejected as invalid requests; they are never silently changed to another ID.
11. Existing direct RouterInfo selectors continue to work alongside Proposal 170 additions.
12. Authentication occurs before expensive destination/config parsing and before mutation.
13. Tokens and passwords never enter logs.

## 4. Explicit non-goals

- no token persistence or cross-restart sessions;
- no new authentication scheme, user database, roles, scopes, OAuth, cookies, or mTLS authorization;
- no rate-limiter redesign;
- no implementation of other base I2PControl methods;
- no Proposal 170 TunnelManager, AddressBook, ClientServicesInfo, or RouterInfo source correction beyond common parsing/token handling;
- no change outside `emissary-cli/src/i2pcontrol/**` except directly affected documentation/tests;
- no dependency, CI, release, or upstream work.

## 5. Expected file boundary

Primary production files:

- `emissary-cli/src/i2pcontrol/rpc.rs`
- `emissary-cli/src/i2pcontrol/auth.rs`
- `emissary-cli/src/i2pcontrol/server.rs`
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`

Focused tests may use:

- existing `emissary-cli/tests/i2pcontrol_*` files;
- one new package-local protocol fixture test if existing integration placement is unsuitable.

Documentation:

- `docs/i2pcontrol/README.md`
- directly affected protocol/conformance documentation only.

Do not touch `emissary-core`, router startup, proxies, tunnel managers, address-book runtime, or `.github/**`.

## 6. Required production changes

### WP1 — Typed request envelope and ID validation

- Parse JSON-RPC request IDs as string, integral JSON number representable by the chosen internal type, or explicit null.
- Reject arrays, objects, booleans, fractional numbers, and out-of-range integers with `INVALID_REQUEST` and a null response ID where required by JSON-RPC.
- Preserve the exact valid request ID in every response.
- Represent notification status independently from `RequestId::Null`; do not conflate absent ID with an explicit response ID during dispatch.

### WP2 — Canonical authentication parameters and response

- Define the canonical authentication DTO as `API` plus `Password`.
- Remove mandatory `Username` from canonical validation.
- Return numeric `API` in `AuthenticateResult`.
- Preserve only explicitly supported API versions; do not claim a version whose complete base contract is intentionally unavailable.
- Use constant-time password comparison already present or a strictly equivalent local implementation.

### WP3 — Standard token extraction

- Extract `Token` from request `params` before method-specific parsing.
- Do not mutate the caller's parsed object in a way that loses evidence needed for errors; construct a sanitized method-parameter map without authentication metadata.
- Define deterministic precedence when both `params.Token` and the compatibility header are present:
  - equal valid values are accepted;
  - conflicting values fail authentication;
  - the compatibility header never overrides an explicit standard token.
- Distinguish missing token from unknown token using the I2PControl error code inventory.
- Keep token lookup bounded and in-memory.

### WP4 — Notification execution

- Dispatch notifications through the same authentication, validation, and method path as requests.
- Suppress the response after execution, including validation or operation errors.
- Preserve HTTP-level request/body limits and server failure behavior.
- Add an internal dispatch result that separates `executed result` from `emit response` rather than adding special cases to every handler.

### WP5 — RouterInfo direct compatibility

- After authentication metadata removal, permit every currently supported existing direct RouterInfo selector plus the exact Proposal 170 addition set.
- Retain the nested `Selector` map only as a separately documented compatibility extension.
- Reject canonical/compatibility mixing only after `Token` is removed.
- Keep direct selector-by-presence semantics; selector values are not used as booleans unless the pre-existing compatibility form requires it.

### WP6 — Exact error inventory

- Add named constants for the existing I2PControl-specific errors consumed by this server.
- Map invalid password, missing token, unknown token, missing API version, and unsupported API version separately.
- Do not expose whether a password prefix, token prefix, or token count matched.
- Preserve standard JSON-RPC errors for parse, request, method, params, and internal failures.

## 7. Failure, cancellation, restart, and contention semantics

- Authentication failure performs no handler work and no persistence.
- Token creation is atomic under the existing token store lock.
- Restart invalidates all tokens; this remains documented behavior.
- Concurrent validation and invalidation must produce a coherent before-or-after outcome.
- Notification execution is subject to the same request deadline and concurrency permit as a normal request.
- Cancellation before mutation returns no success; cancellation after an already-durable mutation follows the method's normal durability semantics.
- A malformed request never reaches authentication or handler dispatch.

## 8. Compatibility and migration

- Standard clients using `params.Token` begin working without configuration migration.
- Existing Emissary clients using the token header continue working unless they send a conflicting standard token.
- Existing clients that supplied `Username: "i2pcontrol"` remain accepted as compatibility behavior if this can be retained without ambiguity.
- No persisted data changes.
- No router configuration changes.

## 9. Security review requirements

- Add negative tests proving token/password values do not appear in errors, tracing fields, or debug serialization.
- Verify conflicting header/parameter tokens fail closed.
- Verify unknown method requests still authenticate before revealing protected method behavior.
- Verify request ID error handling does not reflect arbitrary object/array content.
- Do not add password timing sleeps, random delays, or expensive password hashing unrelated to the current configured-secret model.

## 10. Focused tests

Required fixtures:

1. Standard Authenticate request with only `API` and `Password` succeeds.
2. Successful response contains numeric `API` and string `Token`.
3. Missing password returns invalid-password error without issuing a token.
4. Missing API and unsupported API return distinct exact errors.
5. Protected RouterInfo request with `params.Token` and a base selector succeeds.
6. Protected Proposal 170 request with `params.Token` succeeds and the token is not treated as a method parameter.
7. Missing token and unknown token return distinct errors.
8. Header-only token remains a compatibility path.
9. Conflicting header and parameter tokens fail.
10. A valid notification executes a deterministic test mutation/counter and emits no response.
11. An invalid notification executes validation but emits no response.
12. String and integer IDs round-trip exactly.
13. Fractional, boolean, object, array, and out-of-range IDs are rejected without coercion.
14. Direct base and Proposal 170 RouterInfo selectors can coexist where the method contract permits.
15. Nested compatibility selector cannot be mixed with direct selectors after token removal.

## 11. Verification commands

Focused first:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol authenticate
cargo test -p emissary-cli --no-default-features --features i2pcontrol token
cargo test -p emissary-cli --no-default-features --features i2pcontrol notification
cargo test -p emissary-cli --no-default-features --features i2pcontrol request_id
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info
```

Then package scope:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Run touched-file formatting only when the repository-wide baseline remains unrelatedly dirty.

## 12. Documentation and static guards

- Correct the authentication examples to use `API`, `Password`, and subsequent `params.Token`.
- Document the header as compatibility-only.
- Add a static fixture or manifest assertion that `Token` is authentication metadata, not a RouterInfo selector.
- Document notification execution semantics.
- Do not mark the subsystem closed.

## 13. Acceptance criteria

M020 is implementation-complete only when:

- every invariant in Section 3 has direct test evidence;
- standard authentication and protected-request fixtures pass through the production dispatcher;
- notifications execute and suppress responses;
- invalid IDs cannot be coerced;
- direct existing RouterInfo selectors remain compatible;
- no method-specific Proposal 170 scope was expanded;
- package-scoped check/test/clippy pass or any pre-existing unrelated blocker is recorded precisely;
- touched documentation is accurate;
- an implementation disposition records commits, commands, residual findings, and no-upstream attestation.

## 14. Stop conditions

Stop and record `blocked` or `corrective pass required` if:

- supporting standard token placement requires changing every handler rather than one common boundary;
- a proposed change silently drops existing header clients without a documented compatibility decision;
- an API version is advertised without a written supported-method contract;
- notification execution would require bypassing normal authentication or resource bounds;
- work expands into unrelated base I2PControl methods, router state, CI, or upstream activity.

## 15. Closure evidence required

- exact request/response fixture table;
- authentication/error requirement-to-test matrix;
- notification side-effect and no-response evidence;
- ID rejection matrix;
- changed-file list proving the boundary remained local;
- exact verification commands and outcomes;
- unresolved findings with severity;
- internal-only/no-upstream compliance attestation.

Successful implementation moves M020 to `closing` and makes M021 dependency-ready. It does not close Proposal 170.
