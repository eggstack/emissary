# M137 Closure — M133 Corrective: SAM Idle Close and Reasoned Termination

Status: **closed as complete**

Date: `2026-09-04`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/137-m133-corrective-sam-idle-close-and-reasoned-termination.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Planning baseline:

- M136 closure `plans/closure/i2pcontrol-proposal-170/136-closure.md` (gate satisfied);
- pre-M137 matrix `305 apply / 67 blocked_primitive / 468 not_applicable`;
- M132 closure `plans/closure/i2pcontrol-proposal-170/132-closure.md` (blocked, zero promotions);
- M133 closure `plans/closure/i2pcontrol-proposal-170/133-closure.md` (blocked).

Reviewed head (implementation):

- implementation commit closing M137 on the current branch (production + tests +
  M062/M095/M105 guards + matrix/ledger/docs + planning records).

Production-behavior baselines (retained):

- M130 implementation head `fe1a981`;
- M130 closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`;
- M135 primitive (desired inbound/outbound targets, dynamic LeaseSet desired
  count, bounded destination-scoped coordination) unchanged and requalified
  by 30 `m135_*` tests still green;
- M136 idle decrease/restore owner unchanged in behavior when close is
  disabled and requalified by 17 `m136_*` core + 9 `m136_*` CLI tests still green.

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`;
- read-only Java reference snapshot
  `i2p/i2p.i2p@2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- M130 runtime/security qualification; M131 residual primitive authority;
  M135 neutral primitive closure; M136 canonical activity/timer closure.

Current Proposal matrix at closure (mechanically recomputed):

- `319 apply / 53 blocked_primitive / 468 not_applicable`;
- `095-full-support-matrix.toml` SHA-256
  `40509da612bd1430910295524bd88cf4e7350c56b859ba1c4e20db4a295c49a1`.

## 1. Executive finding

M137 is closed as complete. Emissary now extends the same M136
generation-local SAM session idle activity owner with exact standard I2CP
close-on-idle semantics (default 30 minutes / 1800000 ms, minimum 5 minutes /
300000 ms, close evaluated before reduction, reduction suppressed when close
threshold is less than or equal to the reduce threshold) and drives real
canonical session/destination/pool teardown at the close threshold. A neutral
authoritative generation-local termination cause (`IdlePolicy` / `Requested` /
`Failure` / `Unknown`) is recorded at the winning transition and carried
through the existing SAM server lifecycle seam to the passive observation
event. Proposal `Close`/`CloseTime` are validated fail-before-allocation and
mapped through Yosemite's existing validated generic additional-session-option
path with exact `SESSION CREATE` serialization. All 14 client `Close*` cells
(six TCP families plus Streamr) promote to `apply` with end-to-end evidence;
five server families per option remain `not_applicable`. No `NewDest`
behavior is added; M134/M138 remains the sole NewDest successor.

## 2. Requirement-to-evidence matrix

| Plan requirement | Evidence | Result |
|---|---|---|
| §2 registration gate (9 M136 items) | M136 closure proves one canonical activity timestamp/state machine, one monotonic generation-local timer owner, real reduction/restore through M135, no local-TCP-handler heuristic, shared-member aggregation, deterministic shutdown/replacement isolation, stable standard parsing, no high/medium defect, explicit readiness mark; M137 rebased on that API without semantic expansion | **pass** |
| §3 direct reference freeze (§3.1–§3.5) | I2CP `closeOnIdle` switch, 300000 ms minimum, 1800000 ms default, close-before-reduce ordering, close<=reduce suppression, `destroySession` teardown shape via existing shutdown chain, primary/subsession aggregation at owning session, Streamr/datagram sessions use same generic `I2PSession` owner | **pass** |
| §4 state-machine extension (Active/Reduced/Closing, one scheduler) | Same `SamSession` owner extended: `idle_closing` terminal flag, `close_enabled`/`close_idle_time` policy, `remaining_delay`/`initial_delay`/`reduce_effective` helpers, single actor-local `R::Timer`, close checked before reduce on same `idle_last_activity` clock, suppression from frozen policy, no second scheduler; tests `m137_*_suppresses/suppression/after_reduce` | **pass** |
| §5 canonical teardown contract | Narrow `request_idle_close` trigger calls existing `stream_manager.shutdown()`, `destination.shutdown()`, socket close (stop accepting commands), drops timer without rescheduling; `SamServer` removal, destination-map cleanup, and passive `SessionRemoved` publication reuse the existing path; no second pool shutdown, no wire extension; map recorded in §3 | **pass** |
| §6 neutral termination reason | `SamTerminationReason::{IdlePolicy,Requested,Failure,Unknown}` generation-local, first-wins recording, idle recorded only when idle transition wins, manual/failure never labeled idle, simultaneous races resolve by poll order or `Unknown`, no secrets/addresses/Proposal fields, publication failure never blocks teardown, available in-process via `SamSessionResult` + `SessionRemoved{reason}` for later NewDest, never persisted; tests `m137_idle_cause/manual/failure/stale` | **pass** |
| §7 standard option consumption (`i2cp.close*`, no Proposal vocab in core) | `IdlePolicy::parse` deterministic: `closeOnIdle` true iff case-insensitive `true`, else disabled fail-safe; close default 1800000, min 300000 clamp; reduce suppression derived, not parsed; core diagnostics grep clean for `Proposal`/`I2PControl`/`TunnelManager`/`JsonRpc`; tests `m137_close_policy_*` | **pass** |
| §8 I2PControl mapping (existing generic path, no Yosemite change) | `parse_close_policy` + `build_session_options` mapping: `Close`→`i2cp.closeOnIdle`, `CloseTime`→`i2cp.closeIdleTime` (ms) via `add_session_option`; invalid/unsupported fail before listener/session allocation; `CloseTime` without `Close=true` fails (no silent enable, no accept-inert); server families reject; fake-SAM test proves exact `SESSION CREATE` keys with no raw injection | **pass** |
| §9 shared-session behavior | Core primary/subsessions share one `SamSession` owner (same as M136); I2PControl `CompatibilityKey` + `additional_options_identity` carries exact Close identity automatically (differing policies do not share, proven by `m137_differing_close_policies_do_not_share`); one member release does not close remaining (registry `release` only drops on last member); final explicit release is `Requested`, not idle; creator cancellation drops reservation without stranding timer (M116/M123 unchanged) | **pass** |
| §10 path budget respected | Realized diff: `emissary-core/src/sam/session.rs`, `emissary-core/src/sam/mod.rs`, `emissary-core/src/lib.rs`, `emissary-cli/src/i2pcontrol/backends/runtime/session.rs`, `client.rs`, `connect_client.rs`, `http_client.rs`, `socks.rs`, `sam_observer.rs`, plus focused tests, M062/M095/M105 guards, matrix/ledger/docs/closure/registry/roadmaps, `AGENTS.md`/`README.md`; no Cargo/lockfile/Yosemite/frontend/NetDb/crypto/transport/peer-selection change | **pass** |
| §11 all 23 focused tests | 14 core `m137_*` + 9 CLI `m137_*` plus updated M121/M136 guards (see §4); all pass; items 1–23 map in §4 | **pass** |
| §12 failure/cancellation/restart | Timer/reason/closing state generation-local, never persisted; restart fresh active with `None` reason; shutdown wins (closing flag stops reschedule, timer dropped, `Failure` only when no winner); no lock across I/O; stale generations isolated by owned timer + generation id + closing flag; failed controls explicit | **pass** |
| §13 security review | Per-destination isolation; exploratory/participating unreachable; no hop/peer/clearnet change; proxy/DNS/loopback confinement unchanged; no fabricated leases; no secrets in logs/Debug/reason (vocabulary test); no user-controlled string becomes a reason; malformed Proposal fails before effect; core contains no Proposal/I2PControl names | **pass** |
| §14 broad verification | §5 commands/results | **pass (with pre-existing historical drift recorded, no M137 regression)** |
| §15 matrix `305/67/468` → `319/53/468` (14 promotions, 0 partial) | Mechanically recomputed; all seven client families proven including Streamr datagram trace; servers remain N/A | **pass** |
| §16 NewDest successor gate | Stable consumer contract in §10 (generation id, authoritative cause, idle-boolean boundary, new-generation rule, no replay); decision: amend/rebase historical M134 only if assumptions match, else create M138; neither registers automatically | **pass** |

## 3. Production implementation evidence

### Canonical teardown map (WP2)

Existing authoritative SAM session-generation removal sequence in current
Emissary (recorded before edits):

- transition that stops accepting commands: `SamSession.socket = None`
  breaks the socket-command loop; when `idle_closing`, the receiver-command
  loop is additionally skipped so no new `Connect`/`Accept`/`Forward`/
  `SendDatagram` is accepted for the generation;
- stream/datagram manager shutdown: `StreamManager::shutdown()` sends
  `ShutDown` to active streams and starts the shutdown handler; its
  `ShutDown` event calls `Destination::shutdown()`;
- destination/tunnel pool shutdown: `Destination::shutdown()` calls
  `TunnelPoolHandle::shutdown()`, which marks quantity control closed and
  sends the pool shutdown signal; the pool emits `TunnelPoolShutDown`,
  which `Destination` surfaces and `SamSession` converts to `Ready`;
- SAM server/session map removal: `SamSession` future `Ready(result)` yields
  `SamSessionResult{session_id, reason}` through `SessionContext` futures;
  `SamServer` removes the sender map entry, retains sub-session cleanup,
  removes `session_id_destinations`/`active_destinations`;
- passive observation publication: `SamServer` publishes
  `SamObservationEvent::SessionRemoved{session_id, reason}` via
  `publish_observation_event` (warn-and-continue, never blocks teardown);
- owner wake/removal behavior: `JoinSet` yields the result; wakers are held
  only for short copies and woken after lock release.

M137 adds the narrow internal trigger `request_idle_close()` which records
`IdlePolicy` only on winning the race, sets `idle_closing`, drops the timer,
and invokes the existing `stream_manager.shutdown()` /
`destination.shutdown()` / socket-close steps above in order. No second pool
shutdown path, no Yosemite change, no SAM wire extension.

### Close policy state (WP3)

- `IdlePolicy` gains `close_enabled`/`close_idle_time` with reference
  defaults/bounds; `reduce_effective()` derives close<=reduce suppression;
  `initial_delay()`/`remaining_delay()` compute the single next deadline;
  `enabled_idle_timer()` uses the minimum of enabled thresholds.
- `poll_idle_timer()` evaluates close before reduce on the same
  `idle_last_activity` clock; after reduction with close enabled a close
  timer remains; after close no reschedule occurs.
- `note_qualifying_activity()` resets both clocks via the single timer and
  restores base targets when reduced; once closing it is ignored.

### Reasoned teardown (WP4)

- `SamTerminationReason` (neutral, `Copy`, no secrets) and `SamSessionResult`
  extend the existing `sam/mod.rs` lifecycle seam; `SessionRemoved` carries
  the reason; `lib.rs` re-exports both.
- `SamSession` holds `idle_closing: bool` + `termination_reason:
  Option<SamTerminationReason>`; `record_termination_reason()` implements
  first-wins; `termination_result()` defaults to `Unknown`, never idle.
- Winning map: `Quit`/socket-close → `Requested`; `TunnelPoolShutDown`
  with no winner → `Failure`; receiver/stream/destination/lookup spontaneous
  ends with no winner → `Unknown`; idle timer → `IdlePolicy` only when no
  winner. Poll order makes simultaneous causes deterministic; ambiguous
  cases use `Unknown`.
- `sam_observer.rs` ignores the reason for snapshot identity (no leak, no
  mislabeling) while remaining exhaustive over the extended event.

### I2PControl mapping (WP6)

- `parse_close_policy()` enforces Proposal-valid `Close`/`CloseTime`
  (boolean master switch, ms minimum 300000, default 1800000, no silent
  enable, servers reject, malformed fail).
- `build_session_options()` maps through Yosemite generic
  `add_session_option` (`i2cp.closeOnIdle=true`,
  `i2cp.closeIdleTime=<ms>`); `client_lifecycle_config()` validates the
  same policy fail-before-allocation while keeping the local
  handler-count `run_idle_closer` disabled.
- Backend allowlists (`client`, `connect_client`, `http_client`, `socks`
  covering `socksirc`) admit `Close`/`CloseTime`; `irc_client`/`streamr`
  ride the common builder path (Streamr already permissive, now with real
  effect).

## 4. Focused tests

28 new/updated deterministic tests, all passing:

- core (14 `m137_*` in `sam/session.rs`): parsing/bounds/fail-safe,
  no-admin-vocabulary, no-close-preserves-M136, no-teardown-before-threshold,
  exact-threshold canonical teardown + no-reschedule, activity resets both
  clocks, close<=reduce suppression, close>reduce reduce-then-close,
  reduced restore postpones close, idle-wins-only + activity-ignored-after-close,
  manual/failure/unknown not idle, stale-generation isolation, shared one
  clock, datagram same owner;
- CLI (9 `m137_*` in `backends/runtime/session.rs`): absent/false disabled,
  defaults mapping, custom time, Time-without-Close fails, malformed fails,
  servers reject, all seven clients translate end-to-end with lifecycle
  heuristic disabled, differing policies do not share (same share), exact
  wire keys without injection;
- updated guards: M121 Close tests now assert M137 applied behavior
  (lifecycle validates, local closer stays disabled, below-minimum still
  fails, NewDest still blocked); `m136_close_remains_blocked` replaced by
  the M137 suite.

Plan §12 items 1–23 map: 1→no-close-preserves-M136 + absent/false;
2→no-teardown-before-threshold; 3→exact-threshold teardown;
4→activity-resets-both + shared/datagram same-owner;
5→activity-resets + restore-postpones; 6→suppression;
7→reduce-then-close; 8→restore-postpones; 9→no-reschedule-after-teardown;
10→idle-wins-only; 11→manual-not-idle; 12→replacement-fresh (restart not
idle); 13→failure-not-idle; 14→first-wins race test;
15→stale-generation isolation; 16→shared one clock; 17→registry release
only on last member (existing M116/M123 + `m137_differing` identity);
18→final release `Requested` (record API + server removal path);
19→datagram same owner; 20→malformed CLI + core fail-safe parsing;
21→exact wire keys; 22→seven families end-to-end CLI translate + core
datagram trace; 23→M061/M062 green for M137 paths (see §5).

## 5. Verification executed

| Command | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo test -p emissary-core --no-fail-fast` | **pass**: `1136 passed, 2 ignored (5 suites)` (1122 pre-existing + 14 new) |
| `cargo test -p emissary-core --lib -- m136` | **pass**: `18 passed` (17 M136 + 1 M137 name overlap) |
| `cargo test -p emissary-core --lib -- m137` | **pass**: `14 passed` |
| `cargo test -p emissary-core --lib -- m135` | **pass**: `30 passed` (M135 still correct) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` (workspace) | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `776 passed` (758 pre-existing + 9 M136 retained + 9 new) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib -- m137` | **pass**: `9 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib backends::runtime::session` | **pass**: `33 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit` | **pass**; matrix `319/53/468` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | **pass** (M130 live qualification inherited for unaffected surfaces) |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass** |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **pass except pre-existing** `chunks_exact_to_as_chunks` in untouched `backends/filters/proxy.rs`; M137 files clean |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide; M137 touched files formatted via stable before commit |
| `git diff --check` | **pass** (no whitespace errors) |

Full `cargo test -p emissary-cli ... --no-fail-fast` and `m061`/`m062` historical
suites assert pre-M131/M135 baselines and fail on current master independent
of M137 (same pre-existing drift recorded in M136 §5). M137's own guards
(`m095`/`m105` with new counts, `m062` M137 paths, focused `m137_*`) are
green.

## 6. Invariant review

- Exact pinned names/types/presence preserved; 14 promotions only, no new
  field/alias/status/method/tunnel-type;
- No fabricated/accept-inert support: every `apply` changes real behavior
  (timer-driven canonical teardown + wire mapping); `CloseTime` without
  `Close` fails rather than being ignored;
- Unsupported values fail before allocation (backend allowlists +
  `parse_close_policy` + `client_lifecycle_config` validation +
  `validate_canonical_options` gates);
- No direct-clearnet, outproxy, trusted-peer, or Streamr isolation change
  (Streamr uses same bounded datagram/session contract, limits untouched;
  remote datagrams never choose local UDP destination);
- Loopback confinement, bounded admission/tasks/timers, transactional
  lifecycle, last-known-good preserved;
- No lock across network/build I/O; secret/key/path redaction preserved;
- No LeaseSet crypto/scope change; no publish of nonexistent tunnels;
- Feature/runtime isolation, no base-method parity, no frontend coupling.

## 7. Failure, cancellation, restart, and contention review

- Desired/timer/reduced/closing/reason state generation-local, never
  persisted; restart re-inits from base + fresh activity + `None` reason;
- Pool shutdown → `Failure` only when no winner; idle `IdlePolicy` and
  manual `Requested` survive subsequent pool events (first-wins);
- Failed decrease changes nothing, drops timer (no spin); failed restore
  keeps reduced for bounded retry on next activity; failed close is
  impossible to mislabel because close records only on winning teardown;
- Stale generations cannot reach replacements (owned timer + generation id
  + closing flag; sync ignores mismatches via ownership);
- Overload coalesces (single slot, newest timer wins; restore cannot be
  lost to saturation);
- Excess expiry/failure idempotent under target changes (M135);
- LeaseSet transitions stay owned by `LeaseSetManager`; no second
  publisher/timer;
- Shared registry M116/M123 cancellation unchanged; creator drop reopens
  key without stranding timer; one member release never closes remaining
  members.

## 8. Migration and compatibility review

- No public API version, method, tunnel type, or action change;
- No durable-store migration; `Close`/`CloseTime` already round-tripped
  losslessly in `raw_config`, now validated/mapped (previously rejected
  for six TCP families, silently allowed for `ircclient`/`streamrclient`
  — now all seven have real effect, no inert accept);
- Without `Close=true`, behavior equivalent to M136 qualified runtime
  (disabled → no close timer; reduce behavior unchanged);
- New APIs additive (`SamTerminationReason`, `SamSessionResult`,
  `SessionRemoved.reason`); `sam_observer` updated exhaustively;
  rollback via replacement generation by construction;
- SAM clients without `close*` see no behavior change.

## 9. Security review

- Per-destination ownership: separate M135 cell per pool + per-session
  timer/closing/reason; isolation tests for handles/pools/destinations +
  generation ids;
- Exploratory/participating pools unreachable from destination seam;
- Close cannot force zero-hop/direct-clearnet (hop/variance/peer selection
  untouched) and cannot weaken proxy/DNS/loopback confinement;
- No general router shutdown: one session cannot close another (owned timer
  + generation + server map removal by session id only);
- Backup never promoted beyond desired active target;
- LeaseSet never advertises nonexistent tunnels (real-lease-only);
- No key/secret/path/address in new logs/Debug/reason (vocabulary test;
  reason is a 4-variant enum, observation sanitizes identifiers);
- No user-controlled string becomes a core lifecycle reason (reason is
  fixed enum, never parsed from input);
- No new unbounded queue/task/timer; no SAM wire field/status/event added;
- Termination reason cannot be spoofed by remote peer or unauthenticated
  local SAM payload (recorded only by the authoritative session owner at
  the winning transition);
- Changed core contains no Proposal/I2PControl names (added-line grep
  clean; only standard `i2cp.close*` keys);
- Changed CLI contains Proposal names only inside `i2pcontrol/` policy
  root (M061/M062 green for M137 paths).

## 10. Documentation and operations

- Machine authorities updated: `095-full-support-matrix.toml`
  (`319/53/468`), `110-completion-ledger.toml` (`post_m137`, 14 cells);
- Static guards updated: `m095` expects `319/53/468` + M137 Close `apply`,
  `m105` subtracts 14 M137 cells, `m062` gains `is_authorized_m137_path`;
- Support docs: `docs/i2pcontrol/tunnel-manager.md` (Close applied table +
  runtime paragraphs), `docs/i2pcontrol/proposal-170-support.md` (53 blocked
  breakdown + M137 history);
- `AGENTS.md` + `README.md` pruned to the current `319/53/468` baseline;
- Operational impact: none (no config/metric/diagnostic/restart change;
  disabled sessions have no close timer).

## 11. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | `cargo clippy -p emissary-cli ...` reports pre-existing `chunks_exact_to_as_chunks` in untouched `backends/filters/proxy.rs` | none on M137 files (clean) | record, separate corrective if needed |
| low | Full historical suites (`m060`, `m126`–`m130`, `m061` upstream diff, `m062` pre-existing paths) assert pre-M131/M135 baselines and fail on master independent of M137 | none on M137 behavior; M137 guards green | record as historical drift; future requalification may rebase those suites |
| low | New primitive has no NewDest consumer by design | no key rotation until M134/M138 | M134/M138 consumes the §10 contract |

No high/medium correctness defect remains.

## 12. Roadmap disposition and NewDest successor gate

- M137 is **closed as complete** with matrix `319/53/468`.
- M137 registration gate (plan `137-*.md` §2) was satisfied by M136 closure
  before code; implementation preserves all nine gate items.
- **Stable consumer contract for a future NewDest plan (§17):**
  - session generation identifier: `idle_generation` (core) — stale timers
    and reasons never reach a replacement; a replacement starts without
    inherited idle/reduced/closing/reason state;
  - authoritative termination cause: `SamTerminationReason`
    (`IdlePolicy`/`Requested`/`Failure`/`Unknown`) recorded first-wins at
    the winning transition and carried in `SamSessionResult` +
    `SessionRemoved{reason}`;
  - whether the generation ended by idle policy: `reason ==
    SamTerminationReason::IdlePolicy` — true only when the idle transition
    actually won the teardown race;
  - boundary for a new qualifying generation: process restart, explicit
    restart, replacement SAM session creation, or final-member re-acquire
    after removal — each starts a fresh generation with `None` reason and
    fresh activity clock; no persistent/replayed idle reason across restart;
  - no persistence: reason/timer/closing state are never serialized, never
    survive process restart, and observer failure never blocks teardown.
- **Decision: historical M134 may be amended/rebased only if its
  assumptions exactly match this contract; otherwise create a corrective
  M138 rather than silently executing stale M134 authority. Neither
  registers automatically.**
- Other M131 residual clusters remain unregistered. M132/M133 closures are
  immutable history.

## 13. Registry updates

Applied alongside this closure:

- `137-*.md` plan: Status `dependency-ready / unregistered` → `closed as
  complete` with closure link;
- `plans/registry.md`: M137 → closed as complete; handoff M137 → NewDest
  future (M134 rebase or M138); matrix `319/53/468`; M137 added to
  recently-closed;
- `plans/implementation/.../README.md`: handoff M137 complete; dependency
  graph updated;
- session-lifecycle + full-support roadmaps: M137 complete, 14 promotions;
  NewDest successor gated on §12 contract;
- `110-completion-ledger.toml`: `post_m137` 14 cells.

## Internal-only / read-only-upstream attestation

- External sources (Proposal 170 text, I2CP spec, pinned Java snapshot
  paths via roadmap freeze, Yosemite source for generic-path evidence)
  were accessed read-only for evidence;
- No upstream or third-party repository, issue, pull request, discussion,
  review, or maintainer channel was opened, drafted, updated, commented on,
  or contacted;
- No commit, branch, tag, patch, release, or artifact was pushed to any
  upstream remote before this closure commit. All writes are internal to
  `eggstack/emissary` on the current branch;
- No upstream review, approval, feedback, adoption, or merge was requested;
- No upstream contribution package, patch series, or submission checklist was
  prepared;
- Violation would invalidate this closure per `plans/003-planning-process.md`
  §11; no such violation occurred.

(End of file)
