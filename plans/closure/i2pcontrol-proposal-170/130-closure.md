# M130 Closure — Post-M127–M129 Corrective Requalification

Status: **closed**

Date: 2026-09-04

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/130-post-m127-m129-corrective-requalification.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Implementation commit:

- `fe1a981` (`test(i2pcontrol): add M130 post-corrective requalification evidence`)

Planning baseline: `9948cfd0782a3defbd5f68cf2d4523603bdc7940` for plan
creation; implementation/review baseline is the closed-M129 head
(`579a22c`), so requalification inherits the corrected M127
token-lifetime semantics, the M128 bounded batch dispatch, and the M129
fail-closed TLS boundary plus their regression suites. M130 adds only the
integrated requalification suite plus the M062 dependency-budget entry;
no production change occurs.

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open;
- the existing I2PControl transport/authentication/version behavior required by the extension surface;
- JSON-RPC 2.0 envelope/request-ID/notification/batch semantics used by that surface.

Current Proposal matrix at closure: `284 apply / 96 blocked_primitive / 460 not_applicable` (unchanged, mechanically recomputed).

## 1. Executive disposition

M130 is closed. The corrected I2PControl service is operationally and
security-qualified on the actual post-M129 head, with particular emphasis
on the defects that M126 incorrectly closed:

- finite one-day monotonic token lifetime with atomic expiry removal and
  exact expired (`-32004`) versus unknown (`-32003`) errors;
- bounded JSON-RPC batch handling (`MAX_BATCH_ELEMENTS = 32`) with
  per-element authentication and correct notification suppression, rather
  than blanket rejection;
- fail-closed remote/non-loopback TLS configuration (managed identity is
  loopback-only; every non-loopback bind requires complete explicit
  certificate/key material before listener/filesystem side effects).

Composition and surrounding production service were re-exercised without
regressing previously qualified behavior. M130 restores the clean
current-head “implemented subset qualified” statement for the corrected
shared control plane plus representative Proposal production. It does not
equal full Proposal 170 completion while the 96 applicable residuals
remain blocked.

M130 supersedes only the affected M126 shared-control-plane
qualification claim for current authority. Historical M126–M129 closures
remain unchanged.

## 2. Reviewed head and predecessor table (WP1)

Exact M127, M128, M129 implementation commits verified present in the
reviewed head via `git merge-base --is-ancestor`:

| Milestone | Implementation | Closure | Scope resolved |
|---|---|---|---|
| M127 | `098c9d1` | `plans/closure/i2pcontrol-proposal-170/127-closure.md` | C10 finite token lifetime, expired/unknown distinction |
| M128 | `0ed60eb` | `plans/closure/i2pcontrol-proposal-170/128-closure.md` | C11 bounded batch conformance, per-element auth |
| M129 | `39ccdd7` | `plans/closure/i2pcontrol-proposal-170/129-closure.md` | C12 non-loopback managed-TLS fail-closed |
| M130 | `fe1a981` | this record | integrated current-head requalification |

Reviewed head for behavior: `fe1a981` (closed-M129 head `579a22c` plus
only `emissary-cli/tests/m130_post_corrective_requalification.rs`).

Registry/roadmap/implementation-README status reconciled to M130 closed;
M095 matrix counts recomputed mechanically (see §4); no M127–M129 change
to Proposal option applicability/support was found — all three closures
record `284 / 96 / 460` unchanged and the M130 recomputation agrees.

## 3. Requirement-to-evidence table

| Plan acceptance criterion | Evidence | Result |
|---|---|---|
| 1. M127–M129 individually closed and present in reviewed head | `m130_post_corrective_requalification::m127_m128_m129_commits_are_present_in_reviewed_head`; predecessor table above | pass |
| 2. shared auth/JSON-RPC/TLS black-box qualified at current head | `token_storage_has_finite_expiry_behavior`, `token_expired_is_reachable_and_mapped_distinctly`, `valid_batch_arrays_do_not_regress_to_blanket_rejection`, `batch_dispatch_stays_sequential_without_sharing_or_fanout`, `non_loopback_managed_tls_is_rejected_loopback_remains`, `tls_rejection_precedes_side_effects_and_never_falls_back`, `rejected_remote_creates_no_side_effects_live`, `plaintext_never_reaches_dispatch_tls_only`, `resource_limits_remain_effective`; live runtime phases A/B/G green; adversarial suite green | pass |
| 3. token expiry, batch handling, remote TLS fail-closed have durable regression evidence | M127/M128/M129 suites retained green (50 tests across m126/m127/m128/m129/m130 gate); new M130 composition guards fail on expiry-state loss, `TOKEN_EXPIRED` collapse, batch blanket-rejection, all-notification body emission, over-cap side effects, non-loopback managed acceptance, plaintext dispatch | pass |
| 4. no high/medium shared-control-plane defect remains open | full verification green; §10 findings table | pass |
| 5. AddressBook/TunnelManager/RouterInfo/ClientServicesInfo representative production evidence remains green | `production_composition_has_no_fake_fallback`; live runtime phases C/D/E/F green (Add/Lookup/Delete, subscriptions, SetConfig path guards, restart round trip, tunnel CRUD/bind recovery/startup ownership, RouterInfo selectors, ClientServicesInfo inventory); `router_info_truthfulness`, `production_adapter`, `persistence_concurrency` suites green inside the 2051-test package run | pass |
| 6. M123 cancellation and application/filter boundaries intact | package suite green incl. M123 cancellation tests; `no_unrelated_base_method_parity_is_smuggled`; containment guards green | pass |
| 7. blocked options remain fail-before-effect | M095/M105 green; `proposal_matrix_is_mechanically_recomputed` (284/96/460); no cell promoted | pass |
| 8. matrix counts mechanically reproduced | `proposal_matrix_is_mechanically_recomputed`; M095 `current_matrix_counts_are_explicit_and_exact` and `matrix_is_exhaustive_and_truthful_at_the_current_baseline` green | pass |
| 9. no unexplained production change outside containment | `production_changes_stay_under_i2pcontrol` (baseline `9948cfd` diff: core/util/config/dependency empty; `emissary-cli/src` diffs confined to `i2pcontrol/`); M061/M062 green; `yosemite_alias_remains_optional_exact_and_isolated` green | pass |
| 10. docs/registry/roadmap identify M130 as current authority and retain partial wording | `active_authority_retains_partial_support_and_m130_lineage`; registry/roadmap/implementation-README/docs README updated in closure commit | pass |
| 11. unrelated base parity not smuggled | `no_unrelated_base_method_parity_is_smuggled` (`GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings` remain `UnsupportedBase`; `PROTECTED_DISPATCH` length 6) | pass |
| 12. broad verification no unexplained regression | §8 command/outcome table; 2051 package tests pass; live runtime passes; clippy clean; fmt drift pre-existing only | pass |

Focused-test coverage map (plan §8): token finite-expiry state; `TOKEN_EXPIRED`
reachability and distinct mapping; valid batch execution; single-request
preservation; empty/over-cap/non-object edge cases with zero over-cap
effect; sequential dispatch without fan-out or token sharing; notification
suppression with no-content all-notification batches; loopback managed and
explicit acceptance; non-loopback/wildcard rejection with explicit-only
pass; validate-before-TLS/bind/store ordering; no managed-file/listener
mutation on rejection; loopback-only SAN set without remote synthesis;
explicit no-fallback posture; TLS-only serving with contained handshake
failures; body/connection/concurrency/batch/throttle/handshake/deadline
bounds; secret-safe error paths; fake-free production composition;
mechanical 284/96/460 authority; partial-support wording with M130 lineage;
`i2pcontrol`-only production boundary; exact optional Yosemite pin.

## 4. Current matrix recomputation

Mechanical recomputation from
`095-full-support-matrix.toml` at the reviewed head:

- proposal `170`, revision `2026-05-20`, status `Open`, source SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`;
- RouterInfo: 43 rows (42 `available`, 1 `neutral`, 0 `unavailable`);
- AddressBook SetConfig: 13 rows (12 operational plus contract-defined
  `theme` administrative metadata);
- canonical tunnel types: 12;
- ClientServicesInfo selectors: 6;
- TunnelManager cells: `284 apply / 96 blocked_primitive / 460 not_applicable`;
- declared `current_matrix_counts` agrees: `284 / 96 / 460`.

The 96 residuals remain exactly:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells, including 18
  `Close`/`CloseTime`/`NewDest` cells;
- 19 server presentation/routing/LeaseSet cells.

No M127–M129 cell-level reclassification occurred; M130 promotes none.

## 5. Shared-control-plane black-box results (WP2)

Against the real child-process TLS server (`i2pcontrol_live_runtime`
`live_runtime_interoperability`, 65 tests with `adversarial`):

- Phase A: production TLS listener ready (loopback managed identity).
- Phase B: wrong-password `-32001`; issued token authorizes before expiry;
  all six protected methods reject missing (`-32002`) and invalid
  (`-32003`) credentials; conflicting header/parameter tokens rejected;
  bounded single batch executes with per-element auth; mixed
  valid/invalid batch keeps independent result/error; empty batch is a
  single invalid-request; request IDs preserved; notification executes
  with `204 No Content`; explicit-null ID remains a request.
- Phase G: malformed `not-json` body isolated; shutdown diagnostics
  contain no password.

Deterministic lower-level evidence (no 24-hour sleeps):

- `auth.rs` unit tests: validity at issuance, expiry exactly at
  `TOKEN_LIFETIME`, atomic removal on first expired observation, second
  use unknown, concurrent validators yield no post-expiry success,
  capacity reclaims expired entries within `MAX_TOKENS`, oversized input
  fails before allocation without echo.
- `rpc.rs`/`server.rs` unit tests: ordering, notification suppression,
  mixed/invalid entries, over-cap zero-effect, per-element expired
  (`-32004`) versus unknown (`-32003`), no token propagation, no task
  fan-out, single-request golden fixtures unchanged.
- M129 unit/integration tests: loopback truth table, non-loopback and
  wildcard rejection, complete-explicit pass, validate-before-TLS/bind/
  store ordering, no-file/no-bind/no-mutation proofs, loopback-only SAN
  with remote-verification failure proof, explicit no-fallback proof,
  explicit wildcard serving with verified TLS (no
  `danger_accept_invalid_certs`).
- M130 live spot check: `rejected_remote_creates_no_side_effects_live`
  proves `init_server` rejection creates no `i2pcontrol-certs`,
  `addressbooks`, or `tunnels` state.

Limits effective at current head: body 1 MiB
(`RequestBodyLimitLayer`), concurrency 64 (`Semaphore`), connection
tasks 128, batch 32 (≤64 budget; body cap independent), tokens 1024
with deterministic oldest eviction, presented credentials 256 B,
throttle 256 entries / 60 s window / 25 ms–1 s delay, handshake 30 s,
request deadline 60 s.

## 6. Representative production-owner traces (WP3–WP5)

Production composition (`production_composition_has_no_fake_fallback`):

```text
init_server
  -> config.validate()? (M129 fail-closed boundary first)
  -> SAM observation check
  -> build_tls_config (explicit | managed loopback)
  -> address-book/tunnel store setup (real stores, load before serve)
  -> production state (no Fake fallback; ctx.address_book_handle required)
  -> TcpListener::bind -> serve (TLS acceptor only)
```

`main.rs` supplies `with_address_book_handle` and
`with_startup_tunnel_inventory` before `init_server`. Fake adapters
remain limited to explicit test constructors.

Live runtime representative coverage (real binary, real TLS, real stores):

- AddressBook: private Add → Lookup → SetSubscriptions →
  unsafe-SetConfig rejection → RouterInfo address-book selector →
  Delete → persisted Add → restart Lookup recovery (phases C/F).
- TunnelManager: client create → occupied-port start failure → edit →
  restart → stop; server create/start/stop; unsupported-type create
  then start failure; startup-inventory delete rejection (phase E).
- RouterInfo: available selectors incl. all M045–M050 groups, combined
  multi-group request, `net.error` neutral `-32603` with documented
  message, transit-15s unavailable `-32603` (phase D).
- ClientServicesInfo: live SAM-enabled inventory with BOB `false`
  (phase D).

## 7. Persistence, restart, cancellation, and contention evidence (§6)

- Token expiry races: expired lookup removes atomically under the same
  write lock that decides expiry; concurrent validators cannot both
  authorize after expiry (auth unit test retained green).
- Batch deadline/cancellation: batch dispatch holds the single HTTP
  in-flight permit and runs sequentially under the existing request
  deadline; cancellation stops undispatched elements; already committed
  earlier mutations are not rolled back (documented non-transactional
  batching; method-level transaction guarantees unchanged).
- No batch-level transactional rollback is claimed anywhere in code or
  docs.
- Server shutdown clears all tokens and active request state; restart
  invalidates every token (process-local store) while persisted
  AddressBook/tunnel generations recover (live phase F:
  re-authentication, persisted AddressBook lookup, tunnel `get`,
  RouterInfo health).
- TLS configuration failure precedes service side effects (ordering
  guard plus no-file/no-bind/no-mutation proofs).
- AddressBook concurrent mutation and failed-publication behavior
  retained green via `persistence_concurrency` and production suites.
- TunnelManager per-name lifecycle exclusion and M123 cancellation
  terminalization retained green in the package suite.
- Restart recovery for persisted administrative state and destination
  secrets retained green (live phase F; server-destination
  current/backup rotation unchanged).
- No lock is held across unrelated network I/O for requalification
  instrumentation: M130 adds only synchronous parse/validate guards and
  one bounded `init_server` rejection test; batch dispatch holds no
  global lock across element awaits.

## 8. Verification executed

All commands from the Emissary checkout at the implementation head
(`fe1a981`; closure commit adds only planning/docs). Exit 0 means pass.

| Command | Result |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check` | pass |
| `cargo test -p emissary-core` | pass; 1075 passed, 2 ignored (5 suites) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | pass; 758 tests (unchanged from M129) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | pass; 2051 tests, 31 suites (2034 at M129 + 17 new M130) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial --test i2pcontrol_live_runtime --no-fail-fast` | pass; 65 tests incl. live loopback managed phases |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | pass; 33 tests (4 suites) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m127_token_lifetime --test m128_jsonrpc_batch --test m129_nonloopback_tls --test m130_post_corrective_requalification --test m126_requalification --no-fail-fast` | pass; 50 tests (9 + 8 + 12 + 17 + 4 across 5 suites) |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass; no issues |
| `cargo fmt --all -- --check` | recorded non-zero; stable drift is repo-wide and pre-existing (640 `Diff in` sites on untouched files). M130's new suite is stable-clean after `rustfmt` (zero `m130` diffs). Nightly-only settings drift repo-wide, including untouched files |
| `git diff --check` | pass |

## 9. Secret, logging, and containment review

Changed production paths (implementation commit `fe1a981`): none.
Changed test paths: `emissary-cli/tests/m130_post_corrective_requalification.rs`
(new composition guards) and `emissary-cli/tests/m062_dependency_containment.rs`
(M130 dependency-budget entry folded into the M127 binding).

Baseline `9948cfd` diff for `emissary-core/src`, `emissary-util/src`,
`emissary-cli/src/main.rs`, `emissary-cli/src/config.rs`, manifests, and
lockfile is empty. `emissary-cli/src` production diffs remain confined to
`emissary-cli/src/i2pcontrol/` (the M127–M129 `auth.rs`/`rpc.rs`/
`server.rs`/`tls.rs` changes). M061/M062 green.

Secret review: rejection/auth/batch/TLS error paths interpolate no
password, token, private-key, destination, or path material; static
messages only. Auth performs no logging; TLS performs no SAN synthesis,
trust-store modification, mTLS expansion, or client-verification
weakening. Live shutdown diagnostics contain no password (phases F/G
retained green).

Yosemite: `yosemite-i2pcontrol` remains optional, exact-pinned to Y005
`59140a2277bf296928d2e8ce39a148182eeff044`, activated only by
`i2pcontrol`. Ordinary workspace Yosemite remains the registry package;
no global patch, path replacement, vendoring, or upstream mutation.
M061/M062 green.

## 10. Compatibility and migration review

Intentional behavior corrections (not regressions):

- expired credentials now require re-authentication (`-32004` on first
  post-expiry use, `-32003` thereafter);
- valid JSON-RPC batch clients become supported (additive transport
  compatibility; single-request shape unchanged);
- non-loopback users relying on managed loopback-only identity now
  receive a startup/configuration failure and must configure explicit
  TLS material matching the client-visible endpoint/trust model.

Otherwise compatible:

- single-request and loopback managed-TLS clients remain compatible;
- no Proposal persistence schema migration results from M127–M129 or
  M130 (M130 adds no production or schema change);
- no unrelated base I2PControl methods are added (`GetKeys`, `GetRate`,
  `RouterManager`, `NetworkSetting`, `AdvancedSettings` remain explicit
  `METHOD_NOT_FOUND` per canonical scope).

## 11. Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| deferred capability | 96 applicable Proposal cells remain `blocked_primitive` (4 UseSSL, 10 SigType, 63 client lifecycle/proxy, 19 server LeaseSet/presentation) | Truthfully retained; no owner dependency-ready; not promoted by M130 |
| low | Stable `cargo fmt --check` fails repo-wide on pre-existing drift (640 sites); installed stable formatter cannot apply nightly-only settings | Recorded tooling limitation; M130's added suite is stable-clean; no churn introduced |
| high/medium Proposal-scoped defect | None remaining in M130 scope | No M131+ required from M130 evidence |

## 12. Current-authority and successor-readiness disposition

M130 is **closed** as the current-head requalification authority. The
implemented Proposal 170 subset is operationally/security qualified at
the reviewed head; full Proposal 170 status remains **partial**
(`284 / 96 / 460`).

Successor rule (plan §11): M131+ registers only for a concrete
independently evidenced defect (auth/token bypass, batch auth/resource
bypass, TLS/plaintext/remote regression, fake/shadow state,
success-before-commit, lifecycle/cancellation defect,
source-truthfulness regression, containment regression, or a newly
available residual primitive with exact canonical owner and semantics).
No such defect was found, so **no M131+ is registered**.

Future-plan unblocking determination: no future implementation plan was
blocked on M130 other than M130 itself. The residual capability line
(M111/M112/M113 families) remains blocked for lack of a genuine
canonical owner and exact runtime semantics — M130 changes nothing
there, so no residual plan status is advanced. The full-support
completion line remains blocked by the same 96 residuals. No Yosemite,
containment, or frontend work is unblocked or re-blocked by this
requalification. Active registry/roadmap now record zero registered
Proposal 170 handoffs with M130 as the standing authority.

## 13. Internal-only external-interaction attestation

Pinned Proposal 170 and TLS/JSON-RPC sources were treated as read-only
evidence. Repository writes are confined to `eggstack/emissary` for
this plan. No upstream issue, pull request, review, discussion,
release, submission, merge/adoption request, maintainer contact,
contribution package, or third-party repository mutation was created,
requested, or prepared.

## 14. Disposition

M130 is **closed**. Active planning authority (registry, post-M114
roadmap, implementation README, I2PControl docs) now records M130
closed as the current-head authority, C10/C11/C12 resolved, and partial
`284 / 96 / 460` support unchanged.
