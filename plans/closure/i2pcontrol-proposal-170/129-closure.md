# M129 Closure — Non-Loopback Managed-TLS Fail-Closed Corrective

Status: **closed**

Date: 2026-09-04

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/129-nonloopback-managed-tls-fail-closed-corrective.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Implementation commit:

- `39ccdd7` (`fix(i2pcontrol): non-loopback managed-TLS fail-closed (M129)`)

Planning baseline: `9948cfd0782a3defbd5f68cf2d4523603bdc7940` for plan
creation; implementation/review baseline is the closed-M128 head
(`855cdf1`), so remote-TLS validation inherits the corrected M127
token-lifetime semantics and the M128 bounded batch dispatch plus their
regression suites.

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open;
- existing I2PControl HTTPS transport requirement and accepted local
  managed-certificate architecture.

Current Proposal matrix at closure: `284 apply / 96 blocked_primitive / 460 not_applicable` (unchanged).

## 1. Executive disposition

M129 is closed. Remote I2PControl exposure is now fail-closed when the
operator has not supplied explicit TLS identity material appropriate for
remote clients:

- managed TLS is accepted only for loopback binds (`localhost`,
  `127.0.0.1`, `::1` SAN set unchanged);
- every non-loopback bind — IPv4, IPv6, and wildcard/unspecified
  (`0.0.0.0`, `::`, which are not loopback) — requires both an explicit
  certificate path and an explicit private-key path;
- non-loopback with no explicit paths, certificate-only, or key-only
  fails during `I2pControlConfig::validate`, before listener bind, before
  service-task creation, and before managed certificate
  generation/reuse/mutation;
- the rejection error states the explicit-material requirement and the
  loopback-only managed identity without leaking passwords, tokens,
  private-key material, or managed filesystem internals;
- complete explicit material passes validation and proceeds to the
  unchanged explicit TLS loader for non-loopback serving;
- loopback/default installations are unchanged; explicit TLS failures
  never fall back to managed TLS or plaintext; plaintext remains
  unreachable behind the TLS acceptor.

M129 supersedes only the affected M126/M108 managed-TLS qualification
claim (a non-loopback warning was treated as sufficient). Historical
M126/M127/M128 closures remain unchanged.

## 2. Production architecture

Before (planning baseline): `I2pControlConfig::validate` checked only
empty-password rejection and emitted a non-loopback warning, then
returned `Ok`. A non-loopback bind without explicit paths proceeded to
`build_tls_config`, which generated/reused the loopback-only managed
identity, then bound the listener and started service tasks.

After (M129 head):

```text
I2pControlConfig::validate
  -> empty password => Config error (unchanged)
  -> enabled + non-loopback bind + incomplete explicit material
       => Config error ("non-loopback ... requires explicit TLS
          certificate and private-key paths; managed TLS identity is
          loopback-only"), before any side effect
  -> enabled + non-loopback bind + complete explicit material
       => warn (explicit remote exposure) + Ok
  -> otherwise => Ok (loopback managed/explicit unchanged)

init_server
  -> config.validate()?                      // M129 boundary first
  -> SAM observation check
  -> build_tls_config (explicit loader | managed loopback loader)
  -> address-book/tunnel store setup
  -> production state construction
  -> TcpListener::bind
  -> serve (TLS acceptor only; no plaintext path)
```

`TlsConfig::has_complete_explicit_material` (new small helper in
`emissary-cli/src/i2pcontrol/tls.rs`) expresses the complete-vs-partial
explicit state so the validator reads as the documented policy. No
`emissary-core/**`, `emissary-util/**`, Yosemite, tunnel/proxy/router/
transport/frontend, config-schema, manifest, or lockfile change occurred.

## 3. Requirement-to-evidence table

| Plan acceptance criterion | Evidence | Result |
|---|---|---|
| 1. managed TLS accepted only for loopback binds | `m129_loopback_managed_is_allowed`; `loopback_binds_accept_managed_and_explicit`; live loopback managed suite green | pass |
| 2. every non-loopback bind requires complete explicit cert/key | `m129_non_loopback_managed_is_rejected`; `m129_non_loopback_partial_is_rejected`; `m129_non_loopback_complete_explicit_passes_validation`; `non_loopback_and_wildcard_reject_managed_and_partial` incl. `0.0.0.0`/`::` | pass |
| 3. rejection before listener/task/TLS-file side effects | `rejection_runs_before_tls_generation_and_listener_bind` (static ordering: validate < TLS < stores < bind); `rejected_remote_creates_no_managed_files_and_binds_nothing`; `rejected_remote_does_not_mutate_existing_managed_material`; `validation_is_fail_closed_not_warn_only` | pass |
| 4. loopback managed TLS remains operational | `managed_tls_generates_and_loads` + `managed_certificate_validates_all_loopback_server_names` retained green; new `managed_identity_fails_remote_verification_but_serves_loopback` loopback arm; full `i2pcontrol_live_runtime` loopback managed phases green | pass |
| 5. explicit TLS remains operational and never silently falls back | `explicit_tls_never_falls_back_to_managed_or_plaintext`; `explicit_material_serves_non_loopback_bind` (verified TLS to wildcard bind via operator trust anchor); `m129_loopback_partial_passes_config_and_fails_at_tls_load` (loader-owned partial rejection) | pass |
| 6. plaintext remains unreachable | TLS-only `serve` + `TlsAcceptor` + handshake-timeout guards retained; `explicit_tls_never_falls_back_to_managed_or_plaintext` safe-default/no-fallback assertions; live malformed/plaintext isolation phase green | pass |
| 7. security/configuration docs describe the new boundary exactly | `docs/i2pcontrol/README.md` (Security notes + HTTPS behavior: loopback-only managed, explicit remote requirement, no SAN synthesis/trust modification, no fallback); `docs/i2pcontrol/security.md` (Remote TLS exposure section) | pass |
| 8. production changes wholly inside `i2pcontrol` | `production_changes_stay_under_i2pcontrol` (baseline diff: core/util/config/dependency empty; `emissary-cli/src` diffs confined to `i2pcontrol/`); M061/M062 green | pass |
| 9. Proposal matrix `284 / 96 / 460` | `proposal_matrix_unchanged_by_tls_fail_closed` (mechanical recomputation); M095/M105 green | pass |
| 10. broad verification no unexplained regression | §6 command/outcome table; 2034 package tests pass; live runtime passes; clippy clean; fmt drift pre-existing only | pass |

Focused-test coverage map (plan §10): IPv4 loopback + managed allowed;
IPv6 loopback + managed allowed; non-loopback IPv4 + managed rejected;
non-loopback IPv6 + managed rejected; wildcard/unspecified
(`0.0.0.0`, `::`) treated as non-loopback and rejected without explicit
material; complete explicit passes validation for non-loopback; partial
explicit rejects (both loopback-loader and non-loopback-policy paths);
rejected remote creates no managed TLS files/listener and mutates no
existing managed material; loopback managed SAN set remains
localhost/loopback-only with remote verification failure proof; explicit
TLS failure never falls back to managed or plaintext; error/log output
contains no secret material.

WP4 runtime qualification adds: loopback managed still starts and serves
verified TLS clients (unit handshake + live child-process suite);
non-loopback without explicit terminates at configuration validation
(init_server `Err` plus ordering guard; no listener/task/files);
explicit material serves a wildcard bind in a controlled local topology
with verification enabled (reqwest + operator trust anchor, no
`danger_accept_invalid_certs`); plaintext/handshake failures remain
contained behind the TLS acceptor.

## 4. Deterministic validation results

Unit (in-crate):

- `i2pcontrol::server`: 8 new M129 truth-table/secret-safety tests
  (`loopback_managed`, `loopback_explicit`, `loopback_partial_loader`,
  `non_loopback_managed`, `non_loopback_partial`, `non_loopback_complete`,
  `disabled_non_loopback`, `rejection_no_secret`).
- `i2pcontrol::tls`: 1 new completeness test (`both paths required`;
  existing `is_explicit`, generation/reuse, permission, symlink, and
  loopback-handshake tests retained green).

Integration:

- `m129_nonloopback_tls`: 12 passed — loopback accept, non-loopback +
  wildcard reject/accept matrix, fail-closed-vs-warning ordering,
  validate-before-TLS/bind/store ordering, no-file/no-bind/no-mutation
  side-effect proofs, loopback-only SAN + no-synthesis guards, managed
  loopback-serves/remote-fails handshake proof, explicit no-fallback
  proof, explicit wildcard serving with verified TLS, containment guard,
  matrix guard.
- `i2pcontrol_live_runtime`: loopback managed child-process phases pass
  unmodified against the real TLS server (readiness, auth/protected/
  batch/notification/ID, AddressBook, RouterInfo/ClientServicesInfo,
  tunnel CRUD, restart/recovery, malformed isolation, no password leak).

## 5. Filesystem/listener side-effect evidence

`rejected_remote_creates_no_managed_files_and_binds_nothing`
(`0.0.0.0:<ephemeral>` + managed `None`/`None` via `init_server`):

- returns `Config` error containing `non-loopback` + `explicit` +
  `loopback-only`, containing no password, no `Token`, no
  `i2pcontrol-certs` internals;
- `i2pcontrol-certs/` absent, `addressbooks/` absent, `tunnels/` absent
  (validation precedes all of them);
- the configured port rebinds successfully afterward (no listener held,
  no instance exists so no service task was started).

`rejected_remote_does_not_mutate_existing_managed_material`
(pre-generated managed `cert.pem`/`key.pem` + `192.0.2.10` managed):

- `init_server` rejects; both files are byte-identical afterward.

## 6. Verification executed

All commands from the Emissary checkout at the implementation head
(`39ccdd7`). Exit 0 means pass.

| Command | Result |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | pass; 758 tests (749 at M128 + 9 new unit tests) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | pass; 2034 tests, 30 suites (2004 + 30: 9 unit × lib+bin targets + 12 `m129_nonloopback_tls`) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial --test i2pcontrol_live_runtime --no-fail-fast` | pass; 65 tests incl. live loopback managed phases |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m127_token_lifetime --test m128_jsonrpc_batch --test m129_nonloopback_tls --no-fail-fast` | pass; 62 tests (50 at M128 + 12 new) |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass; no issues |
| `cargo fmt --all -- --check` | recorded non-zero; stable drift is repo-wide and pre-existing (verified by stashing in M128; same pattern here). M129's own added lines are stable-clean: remaining sites in touched files are pre-existing lines only (`server.rs` 1853/2469/2483, `tls.rs` 176/613, all untouched per `git diff`); new `m129_nonloopback_tls.rs` is stable-clean after `rustfmt`. Nightly-only settings drift repo-wide, including untouched files |
| `git diff --check` | pass |

## 7. Secret/logging review

- Rejection errors interpolate no password, token, private-key, or
  path material: the message is a static requirement statement naming
  only the `certificate`/`private-key` field requirement and the
  loopback-only managed identity.
- Unit + integration guards assert the error contains no password, no
  `Token`, and no `i2pcontrol-certs` internals.
- Existing secret handling (constant-time password comparison, bounded
  token store, redacted option values, sanitized TLS/store errors, no
  password in child diagnostics) remains green via adversarial/live
  suites.
- Live restart/shutdown diagnostics contain no password (existing live
  test retained green).

## 8. Containment/path review

Changed production paths (implementation commit `39ccdd7`):

- `emissary-cli/src/i2pcontrol/server.rs` — fail-closed validation +
  init_server fail-closed documentation + truth-table unit tests.
- `emissary-cli/src/i2pcontrol/tls.rs` — complete-material helper +
  completeness unit test.

I2PControl-focused tests/docs:

- `emissary-cli/tests/m129_nonloopback_tls.rs` (new guards);
- `emissary-cli/tests/m062_dependency_containment.rs` (M129 budget
  entry, folded into the M127 binding so the reviewed chains are
  untouched);
- `docs/i2pcontrol/README.md` (loopback-only managed + explicit remote
  requirement + no-synthesis/no-fallback operational guidance);
- `docs/i2pcontrol/security.md` (Remote TLS exposure section).

No `emissary-core/**`, `emissary-util/**`, Yosemite, proxy, tunnel,
transport, router, frontend, config-schema, manifest, or lockfile change.
`m129_nonloopback_tls::production_changes_stay_under_i2pcontrol` diffs
planning baseline `9948cfd…` for core/util/config/dependency paths
(empty) and confines `emissary-cli/src` production diffs to
`i2pcontrol/`. M061/M062 green.

## 9. Compatibility/migration review

- Loopback/default installations are unchanged: loopback + managed and
  loopback + explicit both still validate and serve; managed SANs,
  persistence, permissions, symlink confinement, and rustls defaults
  are unchanged.
- Intentional fail-closed behavior change: operators currently relying
  on non-loopback bind plus auto-generated managed TLS now receive a
  startup/configuration failure and must provide explicit
  certificate/key material matching the client-visible
  endpoint/trust model. No persisted state migration occurs; existing
  managed loopback certificates remain reusable for loopback operation.
- Explicit remote TLS remains supported and is now the only accepted
  non-loopback path (proven with verified TLS in a controlled local
  wildcard topology).
- No new error code, method, selector, action, type, option, or response
  field is added; the rejection uses the existing `Config` error
  channel surfaced as a startup error. Matrix mechanically recomputed as
  `284 / 96 / 460`.
- No persistence migration; no lock, task-budget, body-cap, deadline,
  throttle, batch, or managed-file publication/locking weakening.
  Unrelated base methods remain explicit `METHOD_NOT_FOUND` per canonical
  scope.

## 10. Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| deferred capability | 96 applicable Proposal cells remain `blocked_primitive` (4 UseSSL, 10 SigType, 63 client lifecycle/proxy, 19 server LeaseSet/presentation) | Truthfully retained; no owner dependency-ready; not promoted by M129 |
| low | Stable `cargo fmt --check` fails repo-wide on pre-existing drift; installed nightlies additionally reflow repo-wide, including untouched files | Recorded tooling limitation; M129's added lines are stable-clean; no churn introduced |
| high/medium Proposal-scoped defect | None remaining in M129 scope | No M131+ required from M129 evidence |

## 11. Successor readiness decision

M129 closure unblocks exactly one successor:

- **M130** (`130-post-m127-m129-corrective-requalification.md`):
  promoted from blocked/unregistered to **ready / registered**. Its hard
  dependencies (closed M127, closed M128, closed M129) are now all
  satisfied; it freezes the actual merged post-M129 head and becomes the
  only milestone allowed to restore a clean current-head implemented-
  subset qualification statement.
- **C12** (non-loopback managed-TLS identity) is resolved; **C10** and
  **C11** remain resolved under closed M127/M128.

No residual capability implementation is unblocked: the 96 blocked
cells have no newly implementable owner.

## 12. Internal-only external-interaction attestation

Pinned Proposal 170 and TLS/JSON-RPC sources were treated as read-only
evidence. Repository writes are confined to `eggstack/emissary` for
this plan. No upstream issue, pull request, review, discussion,
release, submission, merge/adoption request, maintainer contact,
contribution package, or third-party repository mutation was created,
requested, or prepared.

## 13. Disposition

M129 is **closed**. Active planning authority (registry, post-M114
roadmap, implementation README) now records M129 closed, M130
ready/registered, C12 resolved, and partial `284 / 96 / 460` support
unchanged.
