# M127 Closure — Base I2PControl Authentication Token-Lifetime Corrective

Status: **closed**

Date: 2026-09-04

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/127-base-auth-token-lifetime-corrective.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Implementation commit:

- `098c9d1` (`fix(i2pcontrol): enforce finite auth token lifetime (M127)`)

Planning baseline: `9948cfd0782a3defbd5f68cf2d4523603bdc7940`.

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open;
- the existing I2PControl API-1 authentication/error contract required by the Proposal 170 extension surface;
- the reference I2PControl token lifecycle as read-only compatibility evidence.

Current Proposal matrix at closure: `284 apply / 96 blocked_primitive / 460 not_applicable` (unchanged).

## 1. Executive disposition

M127 is closed. Every issued authentication token now has finite
in-process validity, expired credentials are removed atomically and return
the already-defined `-32004 TOKEN_EXPIRED` on first use after expiry, and
later uses return `-32003 INVALID_TOKEN`. No protected request can succeed
after expiry. Capacity, input, throttle, conflict-rejection, shutdown, and
secret-safety bounds are preserved. No production change occurred outside
`emissary-cli/src/i2pcontrol/**` and no Proposal 170 matrix cell changed.

M127 supersedes only M126's affected authentication-lifetime qualification
claim. Historical M126 closure (`plans/closure/i2pcontrol-proposal-170/126-closure.md`)
remains unchanged.

## 2. Selected lifetime and authority/rationale

- Exact lifetime: `TOKEN_LIFETIME = 24 * 60 * 60` seconds (one day),
  defined as a named constant in `emissary-cli/src/i2pcontrol/auth.rs`.
- Rationale: the reference I2PControl implementation uses finite token
  validity and removes expired tokens, and the RPC layer already declared
  standard `TOKEN_EXPIRED` (`-32004`) behavior that production validation
  could never reach. One day matches the established reference behavior
  per the plan's compatibility rule; no pinned base contract evidence
  required another exact value. The constant carries this rationale
  in-code so future review adjudicates the value rather than a magic number.

## 3. Before/after token-store state model

Before (planning baseline):

```text
TokenStore { tokens: HashMap<String, ()>, order: VecDeque<String> }
validate(&str) -> bool  // read lock, membership only, no expiry
```

- Tokens valid until explicit invalidation, capacity eviction, shutdown, or restart.
- `TOKEN_EXPIRED` declared in `rpc.rs` but unreachable from production validation.

After (M127 head):

```text
TokenStore { tokens: HashMap<String, Instant /* monotonic expiry */>, order: VecDeque<String> }
TokenClock { now: Arc<dyn Fn() -> Instant + Send + Sync> }  // monotonic; manual only under cfg(test)
TokenValidation { Valid, Expired, Unknown }
issue()    // write lock: remove_expired (bounded) -> evict oldest live at capacity -> insert now+TOKEN_LIFETIME
validate() // bound length before lookup; write lock: Valid | Expired-and-removed | Unknown
```

- Expiry decided with `Instant` (monotonic); wall-clock jumps cannot extend or shorten validity.
- `remove_expired` is lazy/bounded (`MAX_TOKENS` entries), called from issuance; no background scanner, timer task, file, or new dependency.
- Expired lookup removes under the same write lock used to decide expiry; concurrent validators cannot both authorize after expiry.
- Presented credentials over `MAX_PRESENTED_TOKEN_LEN = 256` bytes (issued shape is 64 hex chars) fail as `Unknown` before hashing/lookup/allocation with no echo. Opaque semantics preserved; no new hex-syntax requirement.
- `TokenService::new()` (production) always binds the real monotonic clock. Deterministic time control is `new_manual_for_test` under `#[cfg(test)]`, unavailable to production composition; `I2pControlState::set_token_service_for_test` is likewise test-gated.

## 4. Requirement-to-evidence table

| Plan acceptance criterion | Evidence | Result |
|---|---|---|
| 1. every issued token has finite in-process validity | `TOKEN_LIFETIME` constant + `auth::token_lifetime_is_one_day_compatibility_constant`; `token_is_valid_immediately_after_issuance`; `token_expires_at_exact_lifetime_boundary` | pass |
| 2. compatibility lifetime explicitly justified and tested | §2 above; in-code rationale; one-day boundary test at `issued_at + lifetime - 1ns` (Valid) vs exactly at boundary (Expired) | pass |
| 3. expired tokens removed atomically | `expired_validation_removes_token_and_second_use_is_unknown` (count 0, order cleaned); `concurrent_validators_cannot_both_authorize_after_expiry` (8 threads: exactly 1 Expired, 7 Unknown, 0 Valid) | pass |
| 4. first expired observation `-32004`, later unknown `-32003` | `server::protected_dispatch_returns_expired_then_unknown`; `m127_token_lifetime::dispatch_maps_expired_and_unknown_distinctly`; `token_expired_error_contract_is_declared` | pass |
| 5. no protected request succeeds after expiry | `server::expired_token_never_authorizes_protected_dispatch`; concurrent test contains no `Valid`; live runtime pre-expiry success retained | pass |
| 6. capacity and attacker input bounded | `issuance_reclaims_expired_entries_before_evicting_live_ones`; `issuance_at_capacity_with_mixed_expiry_prefers_expired_cleanup`; `oversized_presented_credentials_fail_before_lookup_without_echo`; `server::oversized_token_fails_as_unknown_without_echo`; `MAX_TOKENS=1024` oldest-eviction retained | pass |
| 7. no token/password/expiry secret exposed | secret/logging review (§7); error paths return static messages only; `error_and_log_paths_do_not_echo_token_material`; live child diagnostics contain no password (existing live test) | pass |
| 8. restart/shutdown behavior correct | `clear_tokens` / `token_clear_on_restart`; `serve` still clears on shutdown (unchanged); process-local store, no persistence added | pass |
| 9. no production path outside `i2pcontrol` | containment review (§8); `m127_token_lifetime::production_changes_stay_under_i2pcontrol`; M061/M062 green | pass |
| 10. matrix remains `284/96/460` | `m127_token_lifetime::proposal_matrix_unchanged_by_token_lifetime`; M095/M105 green | pass |
| 11. broad verification no unexplained regression | §6 command/outcome table; 1948 package tests pass; live runtime passes; clippy clean; fmt drift pre-existing only | pass |
| 12. closure supersedes only affected M126 claim | this record; plan status `closed` with closure authority; M126 file untouched | pass |

Focused-test coverage map (plan §9): immediate-valid, exact-boundary expiry, removal, second-use-unknown, dispatch `-32004` then `-32003`, 8-way expiry concurrency, mixed/capacity reclaim without exceeding `MAX_TOKENS`, oversized fail-fast without echo, header-only/params-only/equal-both success, conflict rejection (pre-existing test retained green), shutdown/restart clearing, throttle unchanged (all throttle tests green).

## 5. Deterministic expiry/concurrency results

Unit (in-crate, manual monotonic clock, no sleeps):

- `i2pcontrol::auth`: 31 passed, including `token_expires_at_exact_lifetime_boundary`, `expired_validation_removes_token_and_second_use_is_unknown`, `concurrent_validators_cannot_both_authorize_after_expiry` (exactly 1×Expired / 7×Unknown / 0×Valid), `issuance_reclaims_expired_entries_before_evicting_live_ones` (count collapses 1024→1, never exceeds cap), `oversized_presented_credentials_fail_before_lookup_without_echo` (257 B and 1 MiB both Unknown).
- `i2pcontrol::server`: 34 passed, including `protected_dispatch_returns_expired_then_unknown` (`-32004` then `-32003` with exact static messages), `expired_token_never_authorizes_protected_dispatch` (no sanitized request produced), `oversized_token_fails_as_unknown_without_echo`, `valid_header_only_params_only_and_equal_both_still_work`, `live_issued_token_authorizes_before_expiry`.

Integration:

- `m127_token_lifetime`: 9 passed — error contract, live pre-expiry authorize, oversized-as-unknown, expiry-state guard (fails if storage regresses to `HashMap<String, ()>` or loses `TOKEN_LIFETIME`/`TokenValidation`), dispatch-mapping guard (fails if Expired collapses into `INVALID_TOKEN`), no-scanner guard, no-echo guard, containment guard, matrix guard.

## 6. Verification executed

All commands from the Emissary checkout at the implementation head. Exit 0 means pass.

| Command | Result |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | pass; 725 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | pass; 1948 tests, 28 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | pass; 1 real child-process/TLS runtime test |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m126_requalification --no-fail-fast` | pass; 37 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m127_token_lifetime --no-fail-fast` | pass; 42 tests |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass; no issues |
| `cargo fmt --all -- --check` | recorded non-zero; installed stable formatter cannot apply the repository's nightly-only settings (628 pre-existing diffs, none in M127 files `auth.rs`/`server.rs`/`m127_token_lifetime.rs`/`m062_dependency_containment.rs`/`adversarial.rs`/`i2pcontrol.rs`); no unrelated churn introduced |
| `git diff --check` | pass |

No 24-hour live test was added, per plan (deterministic time control used instead).

## 7. Secret/logging review

- `Authenticate` success returns only the newly issued token; no other path returns token, password, or expiry internals.
- Protected-dispatch errors use static `INVALID_TOKEN_MESSAGE` / `TOKEN_EXPIRED_MESSAGE`; the presented value is never interpolated (asserted by unit + integration guards).
- No `tracing`/logging of token material in `auth.rs` production section; `server.rs` logs only `Authenticate successful` without identity.
- Live runtime shutdown diagnostics contain no password (existing live test phases F/G retained green).
- Password comparison still uses the reviewed bounded constant-time primitive; entropy (32 random bytes, hex) unchanged; token shape test pins 64 hex chars.

## 8. Containment/path review

Changed production paths (implementation commit `098c9d1`):

- `emissary-cli/src/i2pcontrol/auth.rs` — lifetime model, outcomes, bounds, cleanup.
- `emissary-cli/src/i2pcontrol/server.rs` — dispatch mapping + test-only setter.

I2PControl-focused tests/docs:

- `emissary-cli/tests/m127_token_lifetime.rs` (new static/contract + live guards);
- `emissary-cli/tests/adversarial.rs`, `emissary-cli/tests/i2pcontrol.rs` (bool→`TokenValidation` updates);
- `emissary-cli/tests/m062_dependency_containment.rs` (M127 budget + repair of the pre-existing M062 gap where the reopened-line planning files `127`–`130` had no authorizing entry);
- `docs/i2pcontrol/README.md` (finite lifetime, `-32004`→`-32003` + re-authenticate, M127 supersession note).

No `emissary-core/**`, `emissary-util/**`, Yosemite, proxy, tunnel, transport, router, frontend, or dependency change. `m127_token_lifetime::production_changes_stay_under_i2pcontrol` diffs planning baseline `9948cfd…` for core/util/config/dependency paths (empty) and confines `emissary-cli/src` production diffs to `i2pcontrol/`. M061/M062 green.

## 9. Compatibility/migration review

- No persisted schema; tokens remain process-local, so no migration. Restart invalidates every token exactly as before.
- Clients holding a token past one day now receive standard `-32004` and must re-authenticate; later uses return `-32003`. This is the intended compatibility correction, documented in `docs/i2pcontrol/README.md`.
- Single-request shape, API-1-only behavior, TLS-only serving, header/params conflict rejection, token stripping before domain validation, throttle, capacity, notification/ID semantics, and all six protected methods are unchanged except expiry mapping.
- No Proposal method/selector/action/type/option/status/response added; matrix mechanically recomputed as `284 / 96 / 460`.
- Unrelated base methods (`GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings`) remain explicit `METHOD_NOT_FOUND`, per canonical scope.

## 10. Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| deferred capability | 96 applicable Proposal cells remain `blocked_primitive` (4 UseSSL, 10 SigType, 63 client lifecycle/proxy, 19 server LeaseSet/presentation) | Truthfully retained; no owner dependency-ready; not promoted by M127 |
| low | Stable rustfmt cannot verify nightly-only settings (628 pre-existing diffs) | Recorded tooling limitation; M127 files themselves are fmt-clean; no churn introduced |
| pre-existing gap repaired | M062 budget had no entry for reopened-line planning files `127`–`130`, so it failed even on the clean pre-M127 head | Repaired by the M127 budget entry in this workstream; M062 now green |
| high/medium Proposal-scoped defect | None remaining in M127 scope | No M131+ required from M127 evidence |

## 11. Successor readiness decision

M127 closure unblocks exactly one successor:

- **M128** (`128-json-rpc-batch-conformance-corrective.md`): promoted from queued/unregistered to **ready / registered**. It inherits the corrected token-lifetime semantics and must preserve the valid/expired/unknown distinction per element. No router/core dependency; sequencing gate satisfied.
- **M129** remains queued/unregistered behind M128 (one active handoff at a time per registry rules).
- **M130** remains blocked/unregistered on M127–M129 closure.

No residual capability implementation is unblocked: the 96 blocked cells have no newly implementable owner.

## 12. Internal-only external-interaction attestation

Pinned Proposal 170, base I2PControl, and reference token-lifecycle sources were treated as read-only evidence. Repository writes are confined to `eggstack/emissary` for this plan. No upstream issue, pull request, review, discussion, release, submission, merge/adoption request, maintainer contact, contribution package, or third-party repository mutation was created, requested, or prepared.

## 13. Disposition

M127 is **closed**. Active planning authority (registry, post-M114 roadmap, implementation README) now records M127 closed, M128 ready/registered, and partial `284 / 96 / 460` support unchanged.
