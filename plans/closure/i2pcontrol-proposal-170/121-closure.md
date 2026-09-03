# M121 Closure — M111/M112 Semantic Truthfulness Corrective

Status: **closed**

Plan: `plans/implementation/i2pcontrol-proposal-170/121-m111-m112-semantic-truthfulness-corrective.md`

Implementation commit: `21f4070cbe4eb71a7482e01ec0fdf512e2b2536b`

Closure date: 2026-09-03

Proposal authority: I2P Proposal 170, pinned revision `2026-05-20`, status Open.

## Disposition

M121 is closed with both semantic areas demoted to `blocked_primitive`.
No local correction inside the accepted I2PControl ownership boundary can
make the current behavior reference-equivalent, so truthfulness requires the
matrix reduction. Historical M111/M112 closures are untouched; this record
supersedes only the affected support claims.

Authoritative matrix: `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`

Post-closure matrix SHA-256: `98c2dd9a9dfe0a10a2d67b9a799d9af43b45b16a30003f1e1d7bc2700e79366c`

| State | Apply | Blocked primitive | Not applicable |
|---|---:|---:|---:|
| Before M121 (post-M112/M113) | 312 | 70 | 458 |
| After M121 | 284 | 98 | 458 |
| Delta | -28 | +28 | 0 |

The 28 demoted cells are:

- `SigType` × ten families (`client`, `httpclient`, `ircclient`, `socks`,
  `socksirc`, `connectclient`, `server`, `httpserver`, `httpbidirserver`,
  `ircserver`) — M121 Outcome C;
- `Close` × six TCP client families;
- `CloseTime` × six TCP client families;
- `NewDest` × six TCP client families — M121 §5.2.

`ConnectDelay` × six TCP clients remains applied; it is an independent
bounded delay before remote session use with no idle-observation dependency.
Streamr `not_applicable` cells, M111 `UseSSL` cells, and M113 server cells are
unchanged.

## WP1 — independent semantic evidence table

| Option | Proposal text | Java/I2PTunnel reference | Yosemite capability | Emissary capability | Current (pre-M121) behavior |
|---|---|---|---|---|---|
| `SigType` | `destination signing type`, `string`, identity/key-affecting (`095-full-support-matrix.toml:901-907`); audit: changes identity/key material, must be sent on destination/session creation (`105-residual-option-audit.toml:821`); reference behavior: Java requires explicit SAM `SIGNATURE_TYPE` (`105-residual-option-audit.toml:826`) | Requires explicit `SIGNATURE_TYPE` for destination creation; accepts the reference numeric value set, not a singleton | Fork `yosemite-i2pcontrol` at `8026f5b` serializes any `u16` (`SIGNATURE_TYPE=<value>`, proven with `11` in M117/session adapter regressions) and generates any numeric `DEST GENERATE` type | Core `SigningPrivateKey` is exclusively `Ed25519` (`emissary-core/src/crypto/mod.rs:529`); `SigningKeyKind` only `EdDsaSha512Ed25519` (type 7); transient generation is `SigningPrivateKey::random` → Ed25519 (`emissary-core/src/sam/pending/connection.rs:427-428`); SAM parser accepts only `"7"` for `DEST GENERATE` (`emissary-core/src/sam/parser.rs:867-876`, rejects `1337`) | Accepts exactly canonical string `"7"` (`options.rs`, `client_secret_store.rs:398-403` double guard numeric + string equality); all other values fail before allocation with no fallback |
| `Close` | `close idle client`, `boolean` | `i2cp.closeOnIdle` is an I2P-session idle policy (session bytes/messages), not local-socket presence; open-but-inactive local socket does not reset it | `SessionOptions` declares `close_on_idle`/`close_idle_time` but the accepted `SESSION CREATE` serializer never emits them (declaration-only; no idle observation, byte counter, or close callback) | No session-activity observation primitive exists below I2PControl | Counts accepted local TCP handler tasks (`ConnectionActivity:client_listener.rs:240-270`); `run_idle_closer` closes only after count stays zero for `CloseTime` (`:272-297`) |
| `CloseTime` | `idle-close delay`, `duration` | I2P-session idle delay coupled to the same session-activity trigger | Same declaration-only fields, no wire/observation effect | Same missing observation primitive | Bounded millisecond timer on handler-count idle; generation-local, monotonic, cancellable |
| `NewDest` | `explicit destination/key generation`, identity-affecting; coupled to close-on-idle/resume, incompatible with `PersistentClientKey` | `newDestOnResume` allocates a transient successor identity only on resume after an actual idle close; never on manual staging/start; failed resume must not resurrect the old generation | No resume/observation primitive | Same missing trigger | Staged identity + `DestinationKind::Transient` resume options after owned idle close; `NewDest` requires `Close=true`, rejects `PersistentClientKey`/`Shared` combinations |

Reference sources were read-only (Proposal inventory, M116 Java snapshot note at
`094624f0990d545526674c3267ce0e6d9985d8b2`, Yosemite local source, I2CP/streaming
option semantics). No upstream contact occurred.

## WP2 — SigType disposition (Outcome C — demote)

Questions (§4):

1. Best-effort subset or configurable support? The required runtime effect is
   "the requested signing type must produce the destination with matching key
   material and be observable on the emitted SAM/destination creation path."
   No affirmative Proposal/reference text permits a singleton `{7}` domain as
   a configurable option. Outcome A is therefore unavailable.
2. Is `{7}`-only truthful support or inert? It is inert: `"7"` equals the
   Yosemite/router default, so an explicit `"7"` is observationally identical
   to omitting the option. A one-value domain is a fixed field, not a
   configurable `SigType`.
3. What can Emissary generate/use end-to-end (transient + persistent)? Only
   Ed25519 (type 7) — see evidence table. No RSA/ECDSA/RedDSA/GOST private
   signing primitive exists in the accepted owner.
4. Numeric string only, or names/aliases? Reference SAM accepts numeric values
   (and the fork serializes any `u16`); Emissary additionally rejects
   noncanonical spellings (`"07"`, `" 7"`, names such as
   `"EdDSA_SHA512_Ed25519"`, `""`). The demotion does not depend on resolving
   the alias question because no additional type is producible either way.

Outcome B is unavailable (no accepted lower-layer primitive for another type;
new crypto is an explicit non-goal). Outcome C applies: all ten `SigType`
cells return to `blocked_primitive` with missing primitive "router
signing-key generation/signing for non-Ed25519 reference types and SAM DEST
GENERATE/SESSION CREATE handling for the full reference value set." No
`accept_inert`, coercion, or fallback to 7 exists: any supplied value,
including `"7"`, fails before session/listener allocation and before client
secret staging (production validates before stage; `build_session_options`
re-validates).

Yosemite wire plumbing for arbitrary `u16` (e.g. `SIGNATURE_TYPE=11`) and the
client-secret `DEST GENERATE` plumbing remain as dependency evidence; they are
not Proposal support and are no longer reachable via any validated Proposal
path.

## WP3 — session idle disposition (§5.2 — demote all 18)

Reference trace:

- Idle time resets on I2P-session activity (bytes/messages on the streaming
  session), not on accepted local socket lifetime.
- An open but inactive local TCP connection (no I2P traffic) is idle at the
  reference layer, yet the local handler count keeps the Emissary session
  non-idle — the opposite predicate.
- Zero local connections with recent I2P activity, concurrent connections with
  mixed activity, and failed/cancelled connects all diverge between the two
  predicates.
- `newDestOnResume` allocates exactly one transient successor identity on
  resume after the actual idle close; manual stop/start, failed successor
  creation, and successor-generation cancellation must not rotate or resurrect
  identity.

Preferred local correction (§5.1) would require a bounded session-activity
signal (last-I2P-byte/message timestamp/counter or close callback) owned
generation-locally, with no lock across I/O. Yosemite exposes no such
observation: `close_on_idle`/`close_idle_time` are declaration-only and never
reach the wire. Wrapping proxied bytes would still measure the wrong layer
(local TCP bytes ≠ I2P-session activity, including protocol/tunnel traffic)
and would need per-byte bookkeeping or stream-termination semantics outside
the accepted owner. The stop conditions therefore trigger (exact idle
semantics require router/core instrumentation; lifecycle correction would need
a broader session owner).

All 18 coupled cells (`Close`/`CloseTime`/`NewDest` × six TCP families) demote
together; reference evidence supports no narrower split. `ConnectDelay` is
unaffected (no idle-observation dependency) and remains applied. The existing
generation-local `ConnectionActivity`/`run_idle_closer` machinery is retained
in `client_listener.rs` but is unreachable via any validated Proposal path
(`client_lifecycle_config` never returns `close_on_idle=true`); no timer can
fire from Proposal input.

## WP4 — compatibility/identity tests

- `NewDest` rotation on accepted resume is vacated with the demotion: any
  `NewDest` (true or false) fails in `validate_common_options` (earliest
  production gate, before secret staging) and in `client_lifecycle_config`
  (backend preflight), so no staging/start/resume path can rotate.
- Manual staging/start never rotates: without `NewDest`, staging follows the
  persistent/transient/default branches only.
- `PersistentClientKey`/`Shared` constraints remain fail-closed: `Shared`
  semantics are unchanged (M110/M116); `Close`-with-`Shared` now fails on
  `Close` before reaching the shared check, preserving the invariant that one
  member cannot close another's session.
- `client_secret_store` `DEST GENERATE` plumbing tests (type 7 wire,
  non-7 rejection) are retained as dependency evidence, not Proposal support.

## WP5 — matrix/ledger reconciliation

- `095-full-support-matrix.toml`: counts `284 / 98 / 458`; `SigType` row is
  `blocked_primitive_or_not_applicable` with 10 blocked cells and M121
  `blocked_primitive`/`blocking_milestone`; `Close`/`CloseTime`/`NewDest` rows
  have 7 blocked cells each (6 TCP + Streamr) with M121 primitives/milestones;
  `ConnectDelay` unchanged; `current_production_head` is the M121
  implementation commit.
- `105-residual-option-audit.toml`: historical 164-row inventory untouched;
  summary gains `post_m121_*` fields (284/98/458, 28 demoted cells/families).
- `110-completion-ledger.toml`: gains `[post_m121]` (312/70/458 →
  284/98/458, 28 reclassified cells, reason, closure pointer).
- Docs: `docs/i2pcontrol/tunnel-manager.md` (SigType demotion, Close-family
  demotion, `ConnectDelay`-only lifecycle row) and
  `docs/i2pcontrol/proposal-170-support.md` (98 blocked = 4 UseSSL + 10
  SigType + 63 client lifecycle/proxy + 21 server; M111/M112/M121 rows).
- Registry, corrective roadmap, and implementation README are updated to the
  post-M121 counts with M121 closed; M122 remains blocked on Y004.

## WP6 — containment/closure

Changed paths (implementation commit `21f4070`):

- `emissary-cli/src/i2pcontrol/backends/options.rs` — SigType/NewDest demotion;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` — lifecycle demotion + focused tests;
- `emissary-cli/src/i2pcontrol/backends/client.rs` — raw allowlist demotion;
- `emissary-cli/src/i2pcontrol/backends/http_client.rs` — raw allowlist demotion;
- `emissary-cli/src/i2pcontrol/backends/connect_client.rs` — raw allowlist demotion;
- `emissary-cli/src/i2pcontrol/backends/socks.rs` — raw allowlist demotion + `ConnectDelay` retention;
- `emissary-cli/tests/m095_full_support_matrix.rs` — 284/98/458 + demotion assertions;
- `emissary-cli/tests/m105_residual_option_audit.rs` — post-M121 reconciliation;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`;
- `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`;
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `docs/i2pcontrol/tunnel-manager.md`;
- `docs/i2pcontrol/proposal-170-support.md`.

`irc_client`/`socks_irc` need no raw change (no allowlist / shared `config_for`);
their Proposal path is gated by the same lifecycle/common validation.
`client_listener.rs` is intentionally unchanged (dead idle-closer retained,
unreachable via validation). No `emissary-core/**`, `emissary-util/**`,
Cargo/dependency, Yosemite, frontend, workflow, release, or startup-tunnel
production change. M061 manifest unchanged (all production paths remain under
the I2PControl policy root); M062 unchanged (no dependency change).

Historical M111/M112 closures are not rewritten.

## Focused verification (demotion evidence)

New/updated deterministic tests (all fail-before-allocation, secret-safe):

- `options.rs::m121_sigtype_is_blocked_for_all_applicable_families_without_fallback`
  and extended `session_wire_values_are_strictly_bounded_and_router_supported`:
  `"7"`, `"07"`, `" 7"`, names, `"0"/"1"/"11"`, `""` all reject without echo.
- `session.rs::m121_sigtype_seven_is_blocked_before_allocation_without_fallback`:
  `build_session_options` rejects every SigType value including `"7"`;
  omitted SigType still builds with the Yosemite default (router behavior, not
  Proposal support).
- `session.rs::m121_close_closetime_newdest_fail_before_allocation_for_all_clients`:
  `Close`/`CloseTime`/`NewDest` reject for all six TCP families via both
  `client_lifecycle_config` and the earlier `validate_common_options` gate.
- Updated `client_lifecycle_controls_are_bounded_and_fail_before_allocation`:
  `ConnectDelay` still applied and bounded; `Close=false`/`CloseTime` values
  also reject (no accept-inert); `NewDest=false` rejects.
- Retained plumbing evidence (not Proposal support):
  `generic_session_wire_adapter_*` (Yosemite `SIGNATURE_TYPE=11` wire),
  `client_secret_store` `DEST GENERATE SIGNATURE_TYPE=7` / non-7 rejection,
  `client_listener::close_on_idle_recreates_the_session_for_a_new_generation`
  (direct listener primitive, unreachable via Proposal validation).
- `socks` raw allowlist now retains `ConnectDelay` (previously missing while
  the matrix claimed apply) and drops `NewDest`; `Close`/`CloseTime` were
  already absent there and remain blocked via lifecycle.
- Matrix/audit guards: `m095_full_support_matrix` (284/98/458, SigType 10
  blocked, Close/CloseTime/NewDest 7 blocked each, milestones M121) and
  `m105_residual_option_audit` (current blocked = post-M112 blocked ∪ 28
  demoted; post-M121 summary fields).

Reference-oriented runtime fixtures for open-but-idle sockets, active-transfer
deadline races, and idle-close resume identity are vacated by the demotion:
no such runtime effect is claimed, so no approximation test is retained as
support evidence.

## Broad verification

From the repository root:

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | PASS |
| `cargo check -p emissary-cli --no-default-features` | PASS |
| `cargo check` | PASS |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | PASS — 701 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | PASS — 1887 tests across 26 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | PASS — 33 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | PASS — 1 test |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | PASS — no issues |
| `cargo fmt --all -- --check` | Pre-existing stable/nightly toolchain drift only (586 diffs pre-change, 591 post-change, all in the known nightly-only style; no bulk rewrite retained) |
| `git diff --check` | PASS |

## Failure, cancellation, restart, contention review

- Every demoted option fails deterministically before listener/session/task
  allocation, secret staging, SAM I/O, or runtime-map mutation (production
  order: common validation → backend preflight → staging → start).
- No new timer, task, lock, or generation owner is added; the retained
  `run_idle_closer` cannot start from Proposal input.
- Cancellation/restart semantics are unchanged; `ConnectDelay` retains its
  bounded monotonic cancellable sleep with generation invalidation.
- Failed/resumed identity hazards are removed with the demotion: no Proposal
  path can stage, commit, or resurrect a rotated destination.

## Security/containment review

- No secret, private key, proxy credential, or custom-option value enters
  diagnostics; rejection strings name only the option and tunnel type
  (asserted).
- No clearnet fallback, proxy-boundary weakening, loopback/SSRF relaxation,
  LeaseSet downgrade, or frontend coupling.
- M110/M116 shared-session compatibility/cancellation ownership untouched;
  server/startup behavior unchanged.
- M061/M062 guards pass with no manifest change (I2PControl-only paths, no
  dependency change).

## Unresolved findings

1. Low (tooling, pre-existing): stable/nightly rustfmt drift across the repo;
   recorded, not rewritten.
2. None (high/medium): no open M121 security, containment, correctness, or
   lifecycle finding. The two stop conditions (non-Ed25519 signing primitive;
   session-activity observation primitive) are recorded as the exact missing
   neutral/lower-layer primitives for any future successor; no such successor
   is registered by this closure.

## M122 readiness

- **M122 remains blocked.** Gates satisfied: M121 is now closed (Emissary
  semantic baseline `284 / 98 / 458` is frozen). Gate outstanding: Yosemite
  Y004 must close with an exact consumer-pinning SHA and clean LeaseSet wire
  findings. M122 cannot be promoted to ready from this closure alone.
- **No other future plan is unblocked.** No M113-successor neutral LeaseSet
  plan is authorized (requires the M122-close capability/crypto ownership
  audit per the corrective roadmap). No final reclosure is ready (98
  applicable blocked cells remain).

## Internal-only attestation

External specifications, Java/I2P reference snapshots, and Yosemite sources
were inspected read-only. No upstream repository, issue, pull request, review,
maintainer channel, release artifact, or external branch/tag was mutated or
requested. All implementation and planning writes are internal to
`eggstack/emissary`.
