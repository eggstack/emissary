# I2PControl Proposal 170 Milestone M020 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/020-base-i2pcontrol-and-jsonrpc-interoperability.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline reviewed: `02677cb236d8d127a7eb7c2f0664ccd1a5a83377`

Implementation commit:

- `71a8dc6` — `fix(i2pcontrol): restore base JSON-RPC interoperability`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/020-implementation-disposition.md`

## 1. Executive finding

M020 is complete at the common I2PControl JSON-RPC boundary. Standard
`Authenticate` requests use `API` and `Password`, successful responses use a
numeric `API`, protected calls accept `params.Token`, and existing header
clients remain supported only through the documented compatibility header.
Authentication metadata is removed before method parsing. Notifications execute
the normal path and suppress their response, valid IDs round-trip exactly, and
invalid IDs fail without coercion. Direct RouterInfo requests retain the full
existing selector inventory alongside the Proposal 170 additions.

Proposal 170 remains open and the subsystem remains `corrective pass required`;
M020 closes only this bounded interoperability milestone.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Standard `API` and `Password` authentication without required `Username` | `server::tests::authenticate_uses_standard_params_and_numeric_api`; canonical integration fixtures | pass |
| Numeric `result.API` and string `result.Token` | Server unit fixture, `golden_fixtures::fixture_authenticate_success_envelope`, TLS authentication fixture | pass |
| Missing password, missing API, and unsupported API are distinct | `server::tests::authenticate_distinguishes_password_and_api_errors`; named `-32001`, `-32005`, and `-32006` constants and manifest | pass |
| Standard `params.Token` extraction and metadata removal | `server::tests::protected_authentication_sanitizes_params_and_supports_base_router_info`; TLS protected-request fixture | pass |
| Header compatibility, equal-value acceptance, and conflicting-token rejection | `server::tests::protected_authentication_distinguishes_missing_unknown_and_conflicting_tokens` | pass |
| Missing and unknown token errors are distinct and sanitized | Same server test plus named `-32002`/`-32003` inventory; no token value is reflected | pass |
| Authentication precedes method parsing and unknown-method disclosure | Single protected authentication gate in `handle_jsonrpc` and `dispatch_protected`; missing-token path is tested before RouterInfo dispatch | pass |
| Notifications execute normal side effects and suppress responses | `server::tests::notifications_execute_then_suppress_success_and_error_responses`; notification dispatch result separates execution from emission | pass |
| Valid string, integral, and explicit-null IDs are preserved | `rpc::tests::parse_request_ids_preserve_null_and_notification_status`; existing string/integer fixtures | pass |
| Fractional, boolean, object, array, and out-of-range IDs are rejected | `rpc::tests::parse_request_rejects_invalid_ids_without_coercion` | pass |
| Direct base RouterInfo selectors coexist with Proposal 170 additions | RouterInfo direct validation uses `router_info_keys::ALL`; protected base-selector dispatch fixture | pass |
| Nested `Selector` compatibility remains separate and mixing is rejected | Existing RouterInfo compatibility tests plus post-token-removal direct validation | pass |
| Passwords and tokens are not logged or reflected in errors | Sanitized auth errors, adversarial secret tests, existing observability redaction tests, and no credential-bearing tracing fields in the common gate | pass |

## 3. Request/response fixture table

| Fixture | Expected wire behavior | Evidence |
|---|---|---|
| `Authenticate` with `{API: 2, Password}` | `result.Token` is a string and `result.API` is number `2` | server unit and golden fixture |
| `Authenticate` without `Password` | error `-32001`, `Invalid password provided` | server unit |
| `Authenticate` without `API` | error `-32005` | server unit |
| `Authenticate` with `API: 3` | error `-32006` | server unit |
| Protected RouterInfo with `params.Token` and `i2p.router.version` | selector succeeds; `Token` is not passed to RouterInfo | server unit and TLS integration |
| Protected request without token | error `-32002` | server unit |
| Protected request with unknown token | error `-32003` | server unit |
| Header-only protected request | accepted as compatibility path | server unit |
| Header and parameter token conflict | error `-32003`; neither source overrides the other | server unit |
| Valid notification | handler executes; HTTP `204 No Content`; no JSON body | server unit |
| Invalid notification | validation/authentication executes; HTTP `204 No Content`; no JSON body | server unit |

The `-32004` expired-token constant and message remain part of the named
I2PControl inventory, but this in-memory token service has no expiry state and
therefore does not claim expired-token behavior.

## 4. Production implementation evidence

- `rpc.rs` validates request IDs as strings, integral `i64` numbers, or explicit
  null; arbitrary JSON values are rejected as `INVALID_REQUEST` with a null ID.
- `JsonRpcRequest::is_notification` distinguishes an omitted ID from an
  explicit `RequestId::Null`.
- `server.rs` has one protected authentication gate. It resolves parameter and
  compatibility-header tokens, fails closed on conflicts, validates under the
  existing token-store lock, and removes `Token` before handler dispatch.
- `DispatchResult` stores execution output separately from response emission,
  so notification requests share authentication, bounds, validation, and side
  effects with ordinary requests.
- `router_info_handler.rs` accepts all existing direct selectors plus the exact
  Proposal 170 additions while retaining nested `Selector` as compatibility.

No router lifecycle, protocol, tunnel data-plane, persistence, address-book,
service, telemetry-source, dependency, CI, release, or upstream scope was
entered.

## 5. Verification executed

### Commands run

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol authenticate
cargo test -p emissary-cli --no-default-features --features i2pcontrol token
cargo test -p emissary-cli --no-default-features --features i2pcontrol notification
cargo test -p emissary-cli --no-default-features --features i2pcontrol request_id
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info
cargo +nightly fmt -- --check                 # from emissary-cli
git diff --check
```

### Results

- Package check passed.
- Full feature-gated package tests passed: `1149 passed` across `15 suites`.
- Clippy passed with `-D warnings` and no issues.
- Focused filters passed: `authenticate` `10`, `token` `22`, `notification`
  `8`, `request_id` `2`, and `router_info` `91` tests.
- Crate-local CI-equivalent nightly formatting passed.
- `git diff --check` passed.
- The unqualified root `cargo fmt --all -- --check` remains non-zero because
  the repository baseline has unrelated stable-rustfmt and workspace-wide
  differences. Root nightly formatting also reports pre-existing differences
  across `emissary-core`, `emissary-util`, UI, and other untouched files. No
  unrelated file was formatted or included in the implementation commit.

## 6. Invariant and failure review

- Authentication failures return before method-specific destination/config
  parsing, mutation, or persistence.
- Token issue and validation remain bounded and atomic under the existing
  `TokenService` lock; restart still invalidates all tokens.
- Concurrent token validation/invalidation has coherent lock-ordered
  before-or-after behavior.
- Every request, including notifications, acquires the existing concurrency
  permit and remains subject to body, connection, and deadline bounds.
- Invalid JSON and invalid request envelopes fail before authentication or
  handler dispatch. Invalid but syntactically valid notifications execute the
  same validation path and suppress only the response.
- No cancellation or persistence semantics were changed; no durable mutation
  belongs to this milestone.

## 7. Compatibility and migration review

No persisted data or router configuration changes were made. Standard clients
may move from the compatibility header to `params.Token` without migration.
Existing header clients continue to work when no conflicting parameter token
is supplied. A legacy `Username` field is ignored rather than required, so
already-shipped clients remain accepted without changing canonical behavior.

The compatibility nested RouterInfo `Selector` form remains available and is
rejected only when mixed with direct selectors after authentication metadata is
removed.

## 8. Security review

The common gate never includes password or token values in error messages,
tracing fields, or response data. Conflicting credentials fail closed, unknown
tokens do not reveal token prefixes or counts, and unknown methods authenticate
before method behavior is disclosed. Invalid request IDs are not reflected.
The existing constant-time password comparison, in-memory token bound, HTTPS
transport, request limits, and concurrency limits remain in force.

## 9. Documentation and operations

Updated:

- `docs/i2pcontrol/README.md` — canonical authentication, token placement,
  compatibility header, error inventory, and notification semantics;
- `docs/i2pcontrol/proposal-170-support.md` — M020 implementation/closure and
  successor status;
- literal authentication, error, ID, and protected-request fixtures;
- the conformance error-code manifest and static guards.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| informational | Token expiry is not implemented by the existing in-memory token service | `-32004` cannot be produced truthfully | Retain the named inventory; do not claim expiry until a future bounded plan owns it |
| informational | Root workspace formatting baseline is not stable-rustfmt clean | Root formatting command is non-zero outside M020 scope | Use the crate-local nightly formatter for touched-file evidence; do not widen M020 |

No unresolved high- or medium-severity M020 finding remains.

## 11. Roadmap disposition

M020 is closed and its hard dependency is satisfied. M021 is dependency-ready
and may proceed. M022, M023, M024, M025, M026, and M027 remain blocked by their
independent named hard dependencies. Proposal 170 remains open and the
subsystem remains `corrective pass required` until M027 performs final
conformance and independent reclosure.

## 12. Registry updates

The closure/planning update changes:

- M020 implementation plan: `closed`;
- M020 closure record: this file, `closed`;
- M021 implementation plan: `ready`;
- M021 becomes the sole dependency-ready handoff in `plans/registry.md`;
- M022–M027 retain their blocked status and exact dependency names;
- M020 auth/token/error, notification/ID, and RouterInfo compatibility findings
  are marked resolved in the registry.

## 13. Internal-only compliance attestation

All implementation, test, documentation, closure, and planning writes targeted
the internal `eggstack/emissary` repository. The Proposal 170 and I2PControl
reference pages were accessed read-only for protocol verification. No upstream
issue, pull request, review, discussion, merge request, patch, branch, tag,
submission package, maintainer contact, review request, adoption request, or
other upstream repository mutation was created or prepared.
