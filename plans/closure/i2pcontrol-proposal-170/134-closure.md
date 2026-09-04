# M134 Closure — NewDest on Proven Idle Resume

Status: **closed as complete**

Date: `2026-09-04`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/134-newdest-on-proven-idle-resume.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Planning baseline:

- M137 closure `plans/closure/i2pcontrol-proposal-170/137-closure.md` (hard-gate
  satisfied via §12 consumer contract);
- pre-M134 matrix `319 apply / 53 blocked_primitive / 468 not_applicable`;
- M132 closure `plans/closure/i2pcontrol-proposal-170/132-closure.md` (blocked);
- M133 closure `plans/closure/i2pcontrol-proposal-170/133-closure.md` (blocked).

Reviewed head (implementation):

- implementation commit closing M134 on the current branch (production + tests +
  M062/M095/M105 guards + matrix/ledger/docs + planning records).

Production-behavior baselines (retained):

- M130 implementation head `fe1a981`;
- M130 closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`;
- M135 primitive (desired inbound/outbound targets, dynamic LeaseSet desired
  count, bounded destination-scoped coordination) unchanged;
- M136 idle decrease/restore owner unchanged in behavior when close is disabled;
- M137 idle close/teardown plus neutral `IdlePolicy`/`Requested`/`Failure`/
  `Unknown` reason unchanged and consumed (not modified) by M134.

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`;
- read-only Java reference snapshot
  `i2p/i2p.i2p@2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- M130 runtime/security qualification; M131 residual primitive authority;
  M135 neutral primitive; M136 canonical activity/timer; M137 close/reason.

Current Proposal matrix at closure (mechanically recomputed):

- `325 apply / 47 blocked_primitive / 468 not_applicable`;
- `095-full-support-matrix.toml` SHA-256
  `557c37638fd72d2e4161b5b93221f53ee406e6fb0d2c7fc0a2b984203a427be1`.

## 1. Executive finding

M134 is closed as complete. Historical M134 design material was mechanically
rebased against the proven M137 §12 reason/reopen contract before code, so no
corrective M138 was required. Emissary now implements exact Proposal `NewDest`
semantics for the six non-Streamr TCP client families (`client`, `httpclient`,
`ircclient`, `socks`, `socksirc`, `connectclient`): a fresh successor is staged
only when reopening after the immediately preceding owning generation was
authoritatively closed by the configured idle-close policy. Manual Stop/Start,
explicit Restart, process restart, transport/router failure, failed or cancelled
resume, stale reasons and unrelated edits never rotate. `NewDest=true` requires
`Close=true` and conflicts with `PersistentClientKey`/`PrivKeyFile` before
allocation; `NewDest=false` is explicit disabled; Streamr and servers stay
`not_applicable`. Shared policies require compatible `NewDest`/close identity
and yield one synthetic successor, never one per member. All six `NewDest`
cells promote to `apply` with end-to-end evidence; no other cell changes.

## 2. Requirement-to-evidence matrix

| Plan requirement | Evidence | Result |
|---|---|---|
| §2 readiness gate (stable M133/M137 idle-close reason, generation, teardown, stale protection) | M137 closure §12 contract (`idle_generation`, `SamTerminationReason`, `IdlePolicy`-only idle boolean, new-generation boundary, no replay); M134 rebased without semantic expansion, no core change | **pass** |
| §3 ownership (I2PControl-owned, no new core identity API) | New `idle_resume.rs` tracker, `client_secret_store.rs` generation-aware stage/commit plus synthetic shared entries, `backends/runtime/session.rs` validation/compatibility/resume acquisition, `production.rs` start/commit coordination, existing SAM observation seam forwarding; `emissary-core/**` untouched | **pass** |
| §4 lifecycle model (no qualifying close; G idle-eligible for one resume; successor staged G+1; committed; consumed; concurrent serialize; one-shot) | Tracker dedicated/shared generations with `eligible`/`settled` first-wins, one-shot consume on commit, per-name lifecycle locks plus per-policy creation reservations, `try_reserve`/`wait` single-winner; tests `dedicated_*`, `shared_*`, `observation_*`, production `m134_dedicated/shared` | **pass** |
| §5.1 `Close` prerequisite | `parse_newdest_policy` requires `Close=true` for `NewDest=true` (via `parse_close_policy`); `CloseTime`-without-`Close` still fails; `NewDest=false` needs no prerequisite; tests `m134_newdest_true_requires`, `m134_newdest_proven_resume`, production `m134_conflicts` | **pass** |
| §5.2 `PersistentClientKey` conflict | `validate_common_options` plus `parse_newdest_policy` reject `NewDest=true` with persistent before allocation; `NewDest=false` stays compatible; tests `m134_new_dest_applies`, `m134_newdest_true_requires` | **pass** |
| §5.3 `PrivKeyFile` conflict | Same gates reject `NewDest=true` with any import before allocation; imported source file never overwritten or mutated (secret-store import path untouched, synthetic names namespaced, user `__emissary_shared_client_` names rejected); tests as above | **pass** |
| §5.4 `Shared` (one identity, one reason, one successor, compatible policy) | `CompatibilityKey.new_dest_policy` separates incompatible sharing via the wire-exact key; `shared_policy_key` (Transient build, no secrets) keys one synthetic successor; per-policy reservation yields one successor; member mapping propagates one observation to one policy; tests `m134_differing`, `m134_shared_policy_key`, `m134_shared_single_successor`, secret `m134_shared_policy` | **pass** |
| §6 identity transaction (validate; reserve/serialize; stage fresh transient successor only when qualified; construct outside locks; discard on failure/cancel; commit + consume on success; publish only when coherent; retryable; no orphan) | `stage_with_resume`/`shared_stage` (fresh only when qualifying, otherwise reuse/initial), Yosemite construction outside bookkeeping locks, `discard`/`shared_discard` on every pre-commit failure, `commit`/`shared_commit` then tracker `commit_*` on success, `CreationReservation`-style shared reservation with Drop-cancel safety; tests `m134_qualifying_resume`, `m134_failed_resume`, production end-to-end | **pass** |
| §7 manual lifecycle (Stop clears stale; Start after manual reuses; Restart preserves; restart reconstructs durable only; Delete removes; Edit reconciles) | `note_manual_stop` settles without destroying a won idle fact (Stop preserves qualifying for immediate resume, blocks late-race idle otherwise); `Restart` reuses unless qualifying; process restart volatile-clears tracker but reuses durable; `note_delete`/`remove_shared`/`shared_remove` bounded cleanup; `note_edit`/`rename_dedicated` settle + unregister; tests `dedicated_manual_stop_preserves`, `dedicated_delete_and_edit`, `m134_process_restart`, `m134_edit_clears`, production `m134_edit/restart` | **pass** |
| §8 path budget (I2PControl-only, no core/Cargo/Yosemite/startup/frontend/NetDb/crypto) | Realized diff: `idle_resume.rs` (new), `mod.rs`, `client_secret_store.rs`, `backends/runtime/session.rs`, `backends/runtime/client_listener.rs`, `backends/options.rs`, `client.rs`, `http_client.rs`, `connect_client.rs`, `socks.rs`, `streamr.rs` (one-line policy flag), `production.rs`, `sam_observer.rs`, `server.rs` + `main.rs` composition seams sharing one tracker, `tunnel_manager.rs` synthetic-name guard, plus focused tests, M062/M095/M105 guards, matrix/ledger/docs/closure/registry/roadmaps, `AGENTS.md`; no `emissary-core/**`, Cargo/lockfile, Yosemite, NetDb, crypto, frontend, transport change | **pass** |
| §12 all 20 focused tests | 31 new M134-focused tests plus updated guards (see §4); all pass; items 1–20 map in §4 | **pass** |
| §13 broad verification | §5 commands/results | **pass (with pre-existing historical drift recorded, no M134 regression)** |
| §14 matrix `319/53/468` → `325/47/468` (6 promotions, 0 partial) | Mechanically recomputed; six TCP `NewDest` proven; Streamr/servers remain N/A | **pass** |
| §15 security/static guards | §9 review | **pass** |

## 3. Production implementation evidence

### Rebase on M137 (WP1)

M137 §12 provides `idle_generation` isolation, `SamTerminationReason`
first-wins at the winning transition carried in `SamSessionResult` +
`SessionRemoved{reason}`, `reason == IdlePolicy` only when the idle transition
won, fresh generations on restart/replacement/re-acquire with no replay, and no
persistence. M134 consumes exactly this contract: the tracker arms only on an
explicit `IdlePolicy` record for the current generation, settles first-wins so
manual/failure races cannot fabricate idle, ignores stale generations, and
holds no state across process restart. No core production change was required
or made.

### Generation eligibility owner (WP2)

`idle_resume.rs` owns dedicated per-name generations plus shared per-policy
generations, each with `eligible` (one-shot qualifying fact) and `settled`
(first-winner marker), bounded to 1000 entries per namespace with deterministic
eviction, in-memory only. `record_observation` forwards neutral
`SessionRemoved` facts (dedicated by `SESSION CREATE ID` == tunnel name;
shared via member name → stable policy mapping) with first-wins settling.
`note_manual_stop` settles without destroying a won idle fact so an immediate
resume Start still rotates; ordinary Starts reuse. `commit_*` advances and
consumes; `note_edit`/`rename`/`note_delete` reconcile; `try_reserve`/`wait`
serializes one shared successor with Drop-cancel safety. The tracker holds no
secrets (names, generations, policy keys, neutral reasons only).

### Validation conflicts (WP3)

`validate_common_options` allows `NewDest` only for the six TCP families, with
`NewDest=true` conflicting with persistent/import; `parse_newdest_policy`
(additionally gated in `client_lifecycle_config` and `build_session_options`)
enforces `NewDest=true` requires `Close=true`, both values rejected for
Streamr/servers, `false` disabled without prerequisites. Backend raw allowlists
admit `NewDest` for the six TCP families only. All fail before listener/session
allocation and before `DEST GENERATE` (explicit lifecycle preflight precedes
staging in `start_locked`).

### Successor secret transaction (WP4)

`stage_with_resume` (dedicated) and `shared_stage` (synthetic
`__emissary_shared_client_<fnv>` in the same atomic envelope, user collision
rejected) generate fresh via `DEST GENERATE` only when qualifying, otherwise
reuse or initially generate, always persisting `NewDest` successors for
stability. Construction happens outside bookkeeping locks; every pre-commit
failure/cancellation discards the staged successor and preserves eligibility
for retry; commit then tracker advance consumes exactly once. No lock spans
network/build I/O; no orphan keys (pending overwritten, durable atomic via
`publish_with_backup`).

### Shared successor (WP5)

`CompatibilityKey.new_dest_policy` keeps incompatible lifecycle policies from
sharing the same Yosemite session while staying off the wire.
`shared_policy_key` (Transient build, no identity material) keys one synthetic
successor and one eligibility per policy. The per-policy creation reservation
plus the registry per-key reservation guarantee concurrent members yield one
`SESSION CREATE`, never one per member. Final explicit release surfaces as
`Requested` and settles (never idle); creator cancellation drops the
reservation without stranding or consuming.

### Manual/edit/restart cleanup (WP6)

Covered in §2 §7 row and §7 review below.

## 4. Focused tests

44 M134-scoped tests, all passing (plus updated M095/M105/M062 guards):

- tracker (14 in `idle_resume.rs`): preceding-generation gate, first-winner
  manual/failure races, manual preserves won idle, stale/future ignored,
  one-shot consume, failed-retryable, delete/edit, observation dedicated +
  shared mapping with first-wins, shared one-shot/scoped, single-winner
  reservation with cancellation safety, synthetic stability/bounds/namespace,
  no-secret Debug, bounded eviction;
- session (6 new `m134_*` in `backends/runtime/session.rs` plus updated
  `m134_newdest_proven_resume` lifecycle test): prerequisite/conflicts for six
  families, Streamr/server N/A, differing `NewDest` separation with redaction,
  stable policy key excluding identity, six-family end-to-end translate, exact
  wire keys with no `NewDest` injection;
- options (1 `m134_*`): six-family allow with persistent/import conflicts,
  `false` compatible, Streamr/servers N/A;
- secret store (5 `m134_*`): reuse without qualifying, rotate-once-then-stable,
  failed-discard with retry, shared one-successor with discard safety,
  redaction;
- production (5 `m134_*`): dedicated rotate-once end-to-end with Stop/Restart
  preservation and no second rotation (fake SAM DEST + SESSION counting);
  shared one successor with incompatible separation; restart reuse without
  replay; edit clearing; conflicts fail before allocation without secrets;
- guards: `m095` expects `325/47/468` with M134 `NewDest` apply; `m105`
  subtracts six M134 cells; `m062` gains `is_authorized_m134_path`.

Plan §12 items 1–20 map: 1→secret reuse + production dedicated initial;
2→secret rotate + production dedicated resume; 3→secret stable + production no
second rotation; 4→tracker manual-preserves + production Stop/Start;
5→production Restart; 6→production restart-reuse; 7→secret discard + tracker
retryable + production failed path; 8→tracker stale + observation settled;
9→secret discard + tracker retryable; 10→reservation Drop + discard paths with
no double-commit (commit only after backend success, discard on every
pre-commit failure, reservation Drop wakes without consuming);
11→tracker one-shot + reservation single-winner + production dedicated
per-name serialization and shared one-successor counting; 12→options/session/
production conflicts; 13→same PrivKeyFile; 14→prerequisite tests at all three
gates; 15→production shared + secret shared; 16→compatibility differing +
production incompatible; 17→tracker edit/delete + production edit/delete with
shared cleanup; 18→redaction tests at tracker/secret/compatibility plus error
paths asserting no secret echo and matrix/docs containing no key material;
19→Streamr N/A at options/session/lifecycle/matrix; 20→M061/M062 green for M134
paths (see §5).

## 5. Verification executed

| Command | Result |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` (workspace) | **pass** |
| `cargo test -p emissary-core --no-fail-fast` | **pass**: `1136 passed, 2 ignored (5 suites)` (no core change, still green) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `805 passed` (774 pre-existing + 31 new M134-focused) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: `34 passed (4 suites)`; matrix `325/47/468` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | **pass** (M130 live qualification inherited for unaffected surfaces) |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass** |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **pass except pre-existing** `chunks_exact_to_as_chunks` in untouched `backends/filters/proxy.rs`; M134 files clean |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide; M134 touched files retain repo style, no unrelated normalization |
| `git diff --check` | **pass** (no whitespace errors) |

Full `cargo test -p emissary-cli ... --no-fail-fast` (all suites) shows the same
pre-existing historical failures recorded in M137 §5 (`m060` budget,
`m126`–`m130` token/batch/TLS/matrix lineage asserting pre-M131/M135 baselines
and failing on current master independent of M134). M134's own guards (`m095`/
`m105` with new counts, `m062` M134 paths, focused `m134_*`, lib suite, live
runtime) are green; no M134 regression.

## 6. Invariant review

- Exact pinned names/types/presence preserved; six promotions only, no new
  field/alias/status/method/tunnel-type;
- No fabricated/accept-inert support: every `apply` changes real behavior
  (fail-before-allocation validation plus tracker-proven rotation with wire-
  exact `Close` mapping and secret transaction); `NewDest=true` without
  `Close=true` fails rather than being ignored; `NewDest=false` is explicit
  disabled, never inert-accepted as `true`;
- Unsupported values fail before allocation (common options, lifecycle,
  session gates, raw allowlists, production preflight before `DEST GENERATE`);
- No direct-clearnet, outproxy, trusted-peer, or Streamr isolation change
  (Streamr uses same bounded datagram/session contract, limits untouched;
  remote datagrams never choose local UDP destination);
- Loopback confinement, bounded admission/tasks/timers, transactional
  lifecycle, last-known-good preserved;
- No lock across network/build I/O; secret/key/path redaction preserved;
- No LeaseSet crypto/scope change; no publish of nonexistent tunnels;
- Feature/runtime isolation, no base-method parity, no frontend coupling.

## 7. Failure, cancellation, restart, and contention review

- Eligibility/generation/`resuming` state volatile-only; restart starts fresh
  with `None` eligibility and reuses durable committed/shared successors;
- `IdlePolicy` arms only for the current unsettled generation; manual/failure
  settles first-wins; stale/future ignored; ambiguous races resolve to
  ineligible rather than surprise rotation;
- Manual Stop after a won idle preserves for immediate resume; ordinary
  Stop/Restart without qualifying fact reuses; explicit Restart reloads after
  exact stop with non-overlapping generations;
- Failed decrease/close semantics unchanged (M136/M137); failed NewDest resume
  discards staged successor, keeps eligibility retryable, leaves committed
  untouched; failed shared commit stops the just-started backend and discards;
- Stale generations cannot reach replacements (tracker generations plus core
  `idle_generation` isolation; sync ignores mismatches);
- Shared creation/resume overload serializes to one winner (per-policy
  reservation + registry per-key reservation + `Notify`); waiters reuse the
  single committed successor without advancing generation; cancellation drops
  the reservation, wakes waiters, and preserves eligibility;
- No filesystem/persistent-store lock spans Yosemite construction/network I/O;
  per-name lifecycle locks serialize dedicated starts (concurrent same-name
  Start fails `InvalidState`, one successor);
- Observation publication failure never blocks authoritative teardown
  (forwarding is passive, infallible from SAM's perspective).

## 8. Migration and compatibility review

- No public API version, method, tunnel type, or action change;
- No durable-store migration: `NewDest` successors reuse the existing
  `client-destinations/current.json` envelope (`version = 1`); synthetic shared
  entries (`__emissary_shared_client_<fnv>`) are new keys in the same atomic
  map, ignored (and pruned as unreferenced) by old builds, empty by default on
  old files via existing load path — backward compatible, no version bump;
- Existing definitions with `NewDest` remain round-trippable; previously
  rejected `NewDest` values now validate only with the full prerequisite set,
  otherwise fail before allocation with the exact option named;
- Without `NewDest=true` plus `Close=true`, behavior is equivalent to the
  M137-qualified runtime (disabled → no rotation; shared compatibility now
  additionally keys `NewDest` policy, which only separates previously
  conflated incompatible shares);
- New APIs additive (`IdleResumeTracker`, `shared_policy_key`,
  `stage_with_resume`, shared secret methods, `new_with_resume_tracker`,
  tracker sharing in composition); rollback via replacement generation by
  construction;
- SAM clients without `NewDest`/`Close` see no behavior change.

## 9. Security review

- Per-destination ownership: dedicated per-name plus per-policy shared entries,
  each bounded (1000) with deterministic eviction; isolation tests for
  handles/pools/destinations plus generations/policy keys;
- Exploratory/participating pools unreachable from destination seam;
- Rotation cannot force zero-hop/direct-clearnet (hop/variance/peer selection
  untouched) and cannot weaken proxy/DNS/loopback confinement;
- No general router shutdown: tracker cannot close sessions, only consumes
  neutral facts; one session cannot rotate another (per-name/policy isolation,
  server map removal by session id only in core);
- Backup never promoted beyond desired active target;
- LeaseSet never advertises nonexistent tunnels (real-lease-only);
- No key/secret/path/address in new logs/Debug/reason/matrix/docs (vocabulary
  and redaction tests; tracker holds no secrets; reason is a 4-variant enum;
  synthetic names are hashes; errors name options, never values);
- No user-controlled string becomes a lifecycle reason or synthetic name
  collision (synthetic prefix rejected for user tunnels; FNV hash fixed-seed;
  reason is a fixed enum, never parsed from input);
- No new unbounded queue/task/timer; no SAM wire field/status/event added;
  `NewDest` emits no wire option (exact `Close` keys only, proven no-injection);
- Termination reason cannot be spoofed by remote peer or unauthenticated
  local SAM payload (recorded only by the authoritative session owner at the
  winning transition; I2PControl only consumes via the passive hook);
- Changed core: none (added-line grep clean by construction; only standard
  `i2cp.close*` keys on the wire);
- Changed CLI contains Proposal names only inside `i2pcontrol/` policy root
  plus `main.rs`/`server.rs` composition seams sharing the tracker (M061/M062
  green for M134 paths).

## 10. Documentation and operations

- Machine authorities updated: `095-full-support-matrix.toml`
  (`325/47/468`), `110-completion-ledger.toml` (`post_m134`, six cells);
- Static guards updated: `m095` expects `325/47/468` + M134 `NewDest` `apply`,
  `m105` subtracts six M134 cells, `m062` gains `is_authorized_m134_path`;
- Support docs: `docs/i2pcontrol/tunnel-manager.md` (`NewDest` applied table +
  runtime paragraphs), `docs/i2pcontrol/proposal-170-support.md` (47 blocked
  breakdown + M134 history);
- `AGENTS.md` pruned to the current `325/47/468` baseline;
- Operational impact: none (no config/metric/diagnostic/restart change;
  disabled definitions have no eligibility; enabled definitions rotate at most
  once per proven idle close).

## 11. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | `cargo clippy -p emissary-cli ...` reports pre-existing `chunks_exact_to_as_chunks` in untouched `backends/filters/proxy.rs` | none on M134 files (clean) | record, separate corrective if needed |
| low | Full historical suites (`m060`, `m126`–`m130`, `m061` upstream diff, `m062` pre-existing paths) assert pre-M131/M135 baselines and fail on master independent of M134 | none on M134 behavior; M134 guards green | record as historical drift; future requalification may rebase those suites |
| low | Remaining 47 blocked cells (4 UseSSL, 10 SigType, 14 client proxy/lifecycle, 19 server LeaseSet/presentation) | partial Proposal 170 support remains; no M134 scope expansion | separate M131 residual clusters, unregistered; no successor authorized by M134 |

No high/medium correctness defect remains.

## 12. Roadmap disposition and future-plan audit

- M134 is **closed as complete** with matrix `325/47/468`.
- M134 registration gate (historical M133 hard dependency) was satisfied by
  corrective M137 closure §12 before code; implementation preserves all gate
  items with no core change.
- **Session-lifecycle roadmap completes with M134**: `Reduce*` (M136, 21
  cells), `Close*` (M137, 14 cells) and `NewDest` (M134, six cells) are now all
  operational with evidence for every applicable family; `NewDest:streamrclient`
  correctly remains `not_applicable` per M131 affirmative gates. Whole-surface
  Proposal 170 completion remains governed by the parent full-support roadmap
  (partial, 47 residuals outside the lifecycle line).
- **No future plan is unblocked by M134**: the remaining 47 residuals
  (`UseSSL`, `SigType`, outproxy/plugin, HTTP `SSLProxies`/`JumpList`,
  streaming `Profile`, Streamr `ConnectDelay`, `UniqueLocalAddressPerClient`,
  `MultiHoming`/`shouldBundleReplyInfo`, encrypted/authenticated LeaseSets)
  stay unregistered under M131 authority with dependency/architecture blockers
  M134 does not satisfy. No M138 is required (historical M134 assumptions
  matched the M137 contract after rebase). M132/M133 closures are immutable
  history.

## 13. Registry updates

Applied alongside this closure:

- `134-*.md` plan: Status `deferred / unregistered` → `closed as complete`
  with closure link and M137-rebase note;
- `plans/registry.md`: handoff NewDest future → M134 closed as complete;
  matrix `325/47/468`; M134 added to recently-closed; no active handoff
  (lifecycle line complete; residuals unregistered under M131);
- `plans/implementation/.../README.md`: handoff M134 complete; dependency
  graph updated;
- session-lifecycle + full-support roadmaps: M134 complete, six promotions;
  lifecycle line closed, parent remains partial;
- `110-completion-ledger.toml`: `post_m134` six cells.

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
