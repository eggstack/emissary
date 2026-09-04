# M128 Closure — JSON-RPC 2.0 Batch Conformance Corrective

Status: **closed**

Date: 2026-09-04

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/128-json-rpc-batch-conformance-corrective.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Implementation commit:

- `0ed60eb` (`fix(i2pcontrol): bounded JSON-RPC batch conformance (M128)`)

Planning baseline: `9948cfd0782a3defbd5f68cf2d4523603bdc7940` for plan
creation; implementation/review baseline is the closed-M127 head
(`c16934b`), so batch dispatch inherits the corrected token-lifetime
semantics and its regression suite.

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open;
- JSON-RPC 2.0 request, notification, batch, error, and request-ID
  semantics required by the implemented I2PControl extension surface.

Current Proposal matrix at closure: `284 apply / 96 blocked_primitive / 460 not_applicable` (unchanged).

## 1. Executive disposition

M128 is closed. The blanket rejection of top-level JSON-RPC batch arrays
is replaced with bounded, authenticated JSON-RPC 2.0 batch behavior:

- single request objects and non-empty batch arrays (at most
  `MAX_BATCH_ELEMENTS = 32` entries) are both supported;
- empty and over-cap batches are single invalid-request errors with null
  ID; over-cap batches execute zero elements;
- each batch entry is validated with the exact single-request parser, so
  an invalid entry contributes a per-entry error without disturbing valid
  siblings; non-object entries (scalars, null, nested arrays) are
  invalid requests with null ID;
- each protected element authenticates independently with the M127
  valid/expired/unknown distinction; an `Authenticate` entry never
  propagates its token to siblings;
- structurally valid notifications execute normally but contribute no
  response element; an all-notification batch with no invalid entries
  emits no JSON-RPC body (`204 No Content`);
- elements dispatch sequentially under the single held HTTP in-flight
  permit; no task is spawned per element;
- batching is not a transaction: earlier committed mutations are not
  rolled back because a later element fails;
- single-request shape, method/domain semantics, and the Proposal matrix
  are unchanged; no production change occurred outside
  `emissary-cli/src/i2pcontrol/**`.

M128 supersedes only the affected M126 batch-conformance qualification
claim (valid batches were blanket-rejected). Historical M126/M127
closures remain unchanged.

## 2. Exact batch cardinality bound

- `MAX_BATCH_ELEMENTS: usize = 32`, a named constant in
  `emissary-cli/src/i2pcontrol/rpc.rs` with in-code rationale.
- Rationale: one batch holds a single HTTP in-flight permit while its
  elements dispatch sequentially, so a batch must never demand more
  logical requests than the server serves concurrently
  (`MAX_CONCURRENT_REQUESTS = 64` in `server.rs`, cited in the
  constant's documentation). Thirty-two keeps worst-case per-connection
  logical work at half that concurrent budget while covering realistic
  batch clients. The 1 MiB total body cap remains independently
  authoritative.
- Compile-time guard pins `0 < MAX_BATCH_ELEMENTS <= 64`; contract
  tests pin the exact value `32` and the envelope accept/reject
  boundaries at 32/33 entries.

## 3. Single/batch parser architecture

Before (planning baseline): `parse_request` accepted only a top-level
object; every array (including valid batches) was `INVALID_REQUEST`.

After (M128 head), in `emissary-cli/src/i2pcontrol/rpc.rs`:

```text
parse_envelope(&str)
  -> Ok(Single(JsonRpcRequest))      // top-level object via parse_single_object
  -> Ok(Batch(Vec<serde_json::Value>)) // non-empty array, len <= MAX_BATCH_ELEMENTS
  -> Err(PARSE_ERROR/null)           // invalid JSON
  -> Err(INVALID_REQUEST/null)       // empty array | over-cap array | other scalar
parse_batch_entry(&Value)
  -> parse_single_object(map)        // object entries: exact single-request rules
  -> Err(INVALID_REQUEST/null)       // scalar/null/nested-array entries
parse_single_object(&Map)            // the one shared rule set (version/method/id/named-params)
parse_request(&str)                  // single-only entry point, delegates to parse_envelope;
                                     // arrays stay INVALID_REQUEST (contract preserved byte-for-byte)
```

In `emissary-cli/src/i2pcontrol/server.rs`, `handle_jsonrpc` keeps its
single permit acquisition, body-size check, and deadline posture, then
splits on the envelope:

- `Single` → `handle_single_request` (previous behavior, factored
  unchanged through the shared `dispatch_one`);
- `Batch` → `handle_batch_request`: sequential per-entry
  `parse_batch_entry` + `dispatch_one`, input-ordered
  `Vec::with_capacity(entries.len())` response collection, array
  response when non-empty, `204 No Content` when empty.

`dispatch_one` is the single-request auth/dispatch path used by both
envelopes: `Authenticate` via `handle_authenticate_with_source`,
protected methods via `authenticate_protected_request` (per-element
header/params reconciliation, Token stripping) + `dispatch_protected`.
Structural entry errors always emit (even without an ID);
successfully parsed notifications execute then suppress.

Serialization failures in error envelopes fall back to a sanitized
static internal error (`error_value`); presented tokens are never
interpolated.

## 4. Requirement-to-evidence table

| Plan acceptance criterion | Evidence | Result |
|---|---|---|
| 1. single and bounded batch envelopes supported | `parse_envelope` single/batch arms; `valid_two_request_batch_returns_two_ordered_responses`; `single_request_shape_is_unchanged_by_batch_support`; live batch phases return arrays | pass |
| 2. empty/invalid/oversized fail-closed | `empty_array_returns_single_invalid_request`; `invalid_entries_produce_per_entry_errors_without_suppressing_siblings`; `over_cap_batch_executes_zero_elements`; live empty-batch object check | pass |
| 3. each protected element passes the auth boundary | `protected_batch_elements_each_require_valid_auth` (`-32002`/`-32003` siblings); `conflicting_token_in_one_element_cannot_affect_sibling`; live mixed-batch `-32002` | pass |
| 4. notification suppression incl. all-notification | `mixed_notification_and_request_returns_only_request_response`; `all_notification_batch_returns_no_content` (executes: 2 tokens issued); `notification_with_invalid_auth_executes_but_emits_no_response` (throttle count 1) | pass |
| 5. no implicit intra-batch sharing | `no_implicit_token_propagation_from_authenticate_element` (`-32002` sibling, exactly 1 token); static guard: batch region contains no `token_service`/`.issue()`/`"Token"` | pass |
| 6. no unbounded fan-out/allocation | static guard: batch/dispatch regions contain no spawn/JoinSet/join_all/buffered; `with_capacity(entries.len())` bound; one HTTP permit per batch; max-size batch accepted, 33 rejected | pass |
| 7. prior single-request behavior correct | `parse_request` delegation byte-identical errors; golden/adversarial/`i2pcontrol` suites green; single shape test; M127 expiry mapping tests green | pass |
| 8. method transaction/cancellation unchanged | sequential dispatch, no lock spans awaits, no rollback semantics added; documented non-transactional batching in code + user docs | pass |
| 9. production under `i2pcontrol/**` | `m128_jsonrpc_batch::production_changes_stay_under_i2pcontrol` (baseline diff); M062 budget extended with `is_authorized_m128_path`; M061/M062 green | pass |
| 10. matrix `284 / 96 / 460` | `proposal_matrix_unchanged_by_batch_conformance`; M095/M105 green | pass |
| 11. broad verification clean | §6 command/outcome table; 2004 package tests pass; live runtime passes; clippy clean; fmt drift pre-existing only | pass |

Focused-test coverage map (plan §10): two-request batch with
independent IDs/types; mixed notification/request; all-notification no
body; empty array invalid; per-entry invalid errors with valid sibling
survival; over-cap zero side effects; per-element auth
(missing/unknown/conflicting/expired); equal header+params per element;
expired `-32004` for the affected element only with `-32003` sibling;
no propagation from `Authenticate`; null/duplicate ID preservation;
single golden shape; max-size (32) accepted; task/request bounds via
static guards.

WP4 adversarial qualification adds: scalar/null/nested-array entries;
positional-params entry preserving `INVALID_PARAMS` with its own ID;
header-vs-params conflict isolated per element; invalid-auth
notification executing throttle accounting while emitting nothing;
deterministic M127-clock expiry inside a batch.

## 5. Deterministic dispatch results

Unit (in-crate):

- `i2pcontrol::rpc`: 10 new envelope/entry tests, including 32/33
  accept/reject boundaries and the `parse_request`-still-rejects-arrays
  contract test.
- `i2pcontrol::server`: 14 new batch dispatch tests through the real
  `handle_jsonrpc` handler (status + body), including zero-token
  over-cap proof, 32-element max acceptance, per-element
  missing/unknown/conflict/expired matrix, no-propagation proof with
  token-count and re-validation, and null/duplicate-ID preservation.

Integration:

- `m128_jsonrpc_batch`: 8 passed — bound/budget guard, WP5
  anti-regression guard (valid batch parses as batch; single entry
  point still rejects arrays), envelope distinctions, shared-parser
  guard, sequential-dispatch guard, per-element-auth guard,
  containment guard, matrix guard.
- `i2pcontrol_live_runtime`: extended batch phases pass against the
  real child-process TLS server — single-element batch returns a
  one-element array with result; mixed valid/invalid batch returns
  `[result, -32002]`; empty batch returns a single `-32600` object.

## 6. Verification executed

All commands from the Emissary checkout at the implementation head
(`0ed60eb`). Exit 0 means pass.

| Command | Result |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | pass; 749 tests (725 at M127 + 24 new unit tests) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | pass; 2004 tests, 29 suites (1948 + 56: 24 unit × lib+bin targets + 8 `m128_jsonrpc_batch`) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial --test i2pcontrol_live_runtime --no-fail-fast` | pass; 65 tests incl. extended live batch phases |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m127_token_lifetime --test m128_jsonrpc_batch --no-fail-fast` | pass; 50 tests (42 at M127 + 8 new) |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass; no issues (2 doc-continuation + 2 const-assert findings fixed during implementation) |
| `cargo fmt --all -- --check` | recorded non-zero; stable drift is repo-wide and pre-existing (627 sites with pristine M128-adjacent files, incl. untouched core/util; verified by stashing). M128's own added lines are stable-clean: remaining sites in touched files are pre-existing lines only (`server.rs` 1680/2296/2310, `live_runtime` 312/350, all untouched per `git diff`). Nightly-only settings (`match_arm_blocks`, `wrap_comments`) drift repo-wide, including untouched files |
| `git diff --check` | pass |

## 7. Secret/logging review

- Batch plumbing never reads, logs, or forwards token material: static
  guard asserts the batch region contains no `token_service`,
  `.issue()`, or `"Token"`; sanitization stays inside the shared
  per-element auth path.
- Error envelopes use static messages; `error_value` fallback is a
  static internal error. No attacker-controlled input is echoed (the
  over-cap message interpolates only the entry count, not content).
- Authenticate-notification throttle accounting executes without
  emitting, preserving the M127 throttle guarantees.
- Live runtime shutdown diagnostics contain no password (existing live
  test retained green).

## 8. Containment/path review

Changed production paths (implementation commit `0ed60eb`):

- `emissary-cli/src/i2pcontrol/rpc.rs` — envelope type, bound,
  shared single/batch parser.
- `emissary-cli/src/i2pcontrol/server.rs` — single/batch split,
  sequential dispatcher, shared `dispatch_one`, sanitized error values.

I2PControl-focused tests/docs:

- `emissary-cli/tests/m128_jsonrpc_batch.rs` (new guards);
- `emissary-cli/tests/m062_dependency_containment.rs` (M128 budget
  entry, folded into the M127 binding so the reviewed chains are
  untouched);
- `emissary-cli/tests/i2pcontrol_live_runtime.rs` (batch phases prove
  execution instead of rejection);
- `docs/i2pcontrol/README.md` (batch bound + semantics section).

No `emissary-core/**`, `emissary-util/**`, Yosemite, proxy, tunnel,
transport, router, frontend, config, manifest, or lockfile change.
`m128_jsonrpc_batch::production_changes_stay_under_i2pcontrol` diffs
planning baseline `9948cfd…` for core/util/config/dependency paths
(empty) and confines `emissary-cli/src` production diffs to
`i2pcontrol/`. M061/M062 green.

## 9. Compatibility/migration review

- Additive transport compatibility: single-request shapes are unchanged
  (success object, error object, `204` notification); existing
  single-request tests (golden, adversarial, `i2pcontrol`,
  `m027_literal_fixtures`) pass unmodified.
- Clients sending valid batches change from deterministic `-32600`
  rejection to execution — the intended conformance correction,
  documented in `docs/i2pcontrol/README.md` with the exact bound and
  non-transactional semantics.
- Per-element auth may return `-32002`/`-32003`/`-32004` per entry
  under M127 rules; no new error code, method, selector, action, type,
  option, or response field is added; matrix mechanically recomputed as
  `284 / 96 / 460`.
- No persistence migration; no lock, task-budget, body-cap, deadline,
  or throttle weakening. Unrelated base methods remain explicit
  `METHOD_NOT_FOUND` per canonical scope.

## 10. Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| deferred capability | 96 applicable Proposal cells remain `blocked_primitive` (4 UseSSL, 10 SigType, 63 client lifecycle/proxy, 19 server LeaseSet/presentation) | Truthfully retained; no owner dependency-ready; not promoted by M128 |
| low | Stable `cargo fmt --check` fails repo-wide on pre-existing drift (627 sites with M128-adjacent files pristine); installed nightlies additionally reflow repo-wide (`match_arm_blocks`, `wrap_comments`), including untouched files | Recorded tooling limitation; M128's added lines are stable-clean; no churn introduced |
| low | Clippy `doc_lazy_continuation` / `assertions_on_constants` findings in new code during implementation | Fixed before closure (blank-line separation; compile-time const assert) |
| high/medium Proposal-scoped defect | None remaining in M128 scope | No M131+ required from M128 evidence |

## 11. Successor readiness decision

M128 closure unblocks exactly one successor:

- **M129** (`129-nonloopback-managed-tls-fail-closed-corrective.md`):
  promoted from queued/unregistered to **ready / registered**. It is
  implementation-independent of M128 but sequencing-gated behind it;
  the gate is now satisfied.
- **M130** remains blocked/unregistered on M129 closure (plus closed
  M127/M128).
- **C11** (valid batches blanket-rejected) is resolved; **C12**
  (non-loopback managed-TLS identity) remains open under M129.

No residual capability implementation is unblocked: the 96 blocked
cells have no newly implementable owner.

## 12. Internal-only external-interaction attestation

Pinned Proposal 170 and JSON-RPC 2.0 sources were treated as read-only
evidence. Repository writes are confined to `eggstack/emissary` for
this plan. No upstream issue, pull request, review, discussion,
release, submission, merge/adoption request, maintainer contact,
contribution package, or third-party repository mutation was created,
requested, or prepared.

## 13. Disposition

M128 is **closed**. Active planning authority (registry, post-M114
roadmap, implementation README) now records M128 closed, M129
ready/registered, C11 resolved, and partial `284 / 96 / 460` support
unchanged.
