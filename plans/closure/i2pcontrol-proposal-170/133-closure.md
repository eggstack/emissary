# M133 Closure — Neutral SAM Idle Close and Reasoned Termination

Status: **closed as blocked**

Date: `2026-09-04`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/133-neutral-sam-idle-close-and-reasoned-termination.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Planning baseline:

- M131 closure head `3a829d7d3d6314ecf09e42dbf0339506f0917c96`
- M131 matrix `284 apply / 88 blocked_primitive / 468 not_applicable`
- M131 matrix SHA-256 `f038521da9cc685bd38dd502f02dcc81f53586c3a8dd83eb3ba5a8827f589f79`
- M132 closure head `6618c49a4bcf962a1ee263fa97fa95a3b70f1ad2` (closed as blocked, zero promotions)

Reviewed head:

- `6618c49a4bcf962a1ee263fa97fa95a3b70f1ad2` (clean, no production delta)

Production-behavior baseline (unchanged):

- M130 implementation head `fe1a981`
- M130 closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`

Implementation commits:

- None for production Rust, dependencies, Cargo features, Yosemite, or
  lockfiles. This closure lands planning/evidence records plus the required
  M062 planning-path guard only (see §9). No `emissary-core/**`,
  `emissary-cli/src/i2pcontrol/**` production, Cargo, or Yosemite change was
  made, as required by the plan's stop conditions (§15). Exact changed paths
  are listed in §9; the M062 amendment authorizes only the new closure
  planning path.

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
- I2CP specification (read-only): `i2cp.closeOnIdle`,
  `i2cp.closeIdleTime` (default 30 min / 1800000 ms, minimum 5 min /
  300000 ms), `i2cp.reduceOnIdle` / `i2cp.reduceIdleTime` (default 20 min /
  1200000 ms) interaction semantics
- Java reference: `i2p/i2p.i2p` (read-only search evidence only);
  `SessionIdleTimer` constructor contract (`reduce, shutdown, or both must be
  true`) located via public Javadoc index; exact close-threshold evaluation
  against the same activity clock, close <= reduce suppression ordering,
  primary/subsession close aggregation, and authoritative
  session/pool-teardown sequence (plan §3) were **not** retrievable as direct
  reference/runtime source in this environment
- Yosemite Y005 `59140a2277bf296928d2e8ce39a148182eeff044` (read-only):
  typed `close_on_idle` / `close_idle_time` (defaults `false` / 30 min) exist
  but are dormant on the wire; typed `reduce_on_idle` / `reduce_idle_time` /
  `reduce_quantity` are likewise dormant
- M131 residual map `131-residual-primitive-map.toml` cluster
  `session-lifecycle`, path budget `PB-SESSION-IDLE-CLOSE-01`,
  `dependency_readiness = "not_ready"` for all 14 `Close`/`CloseTime` cells
- M132 closure `plans/closure/i2pcontrol-proposal-170/132-closure.md`:
  hard dependency not satisfied — no stable SAM activity/idle state machine,
  no generation-local pool/session lifecycle contract, no monotonic timer
  owner, no live-target/LeaseSet synchronization owner

Current Proposal matrix at closure (mechanically recomputed, unchanged):

- `284 apply / 88 blocked_primitive / 468 not_applicable`
- `095-full-support-matrix.toml` SHA-256
  `f038521da9cc685bd38dd502f02dcc81f53586c3a8dd83eb3ba5a8827f589f79`

## 1. Executive finding

M133 is closed as blocked. No `Close` or `CloseTime` cell is promoted. The
14 mechanically present client cells remain `blocked_primitive` (7 `Close` +
7 `CloseTime` across `client`, `httpclient`, `ircclient`, `socks`,
`socksirc`, `connectclient`, `streamrclient`); the 5 server families per
option remain `not_applicable`. Full Proposal 170 status remains **partial**.

WP1 (rebase after M132) was executed. The M133 readiness gate (§2) requires,
before registration, a stable M132 interface:

- one canonical activity timestamp/state machine in `SamSession`;
- no local-TCP-handler heuristic;
- generation-local monotonic timer behavior;
- clean session/pool shutdown and replacement-generation isolation;
- a stable way to parse standard I2CP session options without Yosemite
  changes;
- no unresolved high/medium correctness issue in the M132 session owner.

M132 closure proves none of these: `SamSession`
(`emissary-core/src/sam/session.rs:68-148`, 2135 lines) has no activity
timestamp, generation, timer, reduction policy, or `i2cp.close*`/`i2cp.reduce*`
consumption; pool maintenance still reads immutable `config.num_*` at ~74
sites with no live-target control; `LeaseSetManager` still takes immutable
`num_inbound` with no live-target synchronization owner; Streamr ownership
remains ambiguous. M132 §10/§11 carry five open high/medium blockers forward
verbatim.

Extending a non-existent M132 state machine with a close deadline (§4.1),
performing canonical session/pool teardown at a close threshold (§4.2), or
exposing a neutral authoritative termination reason through the lifecycle
path (§4.3) would therefore require guessing at the activity predicate,
timer ownership, teardown ordering, and reason authority — each explicitly a
stop condition (§15: stale owner, broad SAM server redesign, nonstandard SAM
wire extension, non-authoritative races, new dependency/Yosemite change, path
expansion). Approximate support is not acceptable.

The correct truthful disposition is therefore blocked, with
fail-before-allocation preserved and last-known-good runtime behavior
retained.

M134 remains deferred/unregistered. M133 did not produce the authoritative
generation-local idle-close reason or stable reopen boundary that M134
hard-depends on, so no successor is unblocked by this closure.

## 2. Requirement-to-evidence matrix

| Plan requirement | Evidence | Result |
|---|---|---|
| §14.1 M132 is closed and its activity owner is reused | M132 closed as blocked (`132-closure.md` §1/§11); `SamSession` has no activity/timer owner (§3.2); no reuse target exists | **fail → blocked** |
| §14.2 exact close/reference ordering is frozen | I2CP spec confirms close defaults/minima (30 min/5 min); Java `SessionIdleTimer` same-clock evaluation, close <= reduce suppression, subsession aggregation, teardown sequence not retrievable as direct source; Yosemite typed close fields dormant (see §3) | **fail → blocked** |
| §14.3 close-on-idle performs real bounded session/pool teardown | No idle-close owner exists; `SamSession` has only ordinary socket/session teardown via `SamServer` removal; no second pool-shutdown implementation was improvised; `ClientSessionOwner::close_if_idle` (`client_listener.rs:220-231`) drops only a Yosemite-session generation, not a core SAM session/pool | **fail → blocked** |
| §14.4 a neutral authoritative idle-close reason exists in-process | `SamObservationEvent` (`sam/mod.rs:89-128`) has only `SessionActivated`/`SocketActivated`/`SocketRemoved`/`SessionRemoved` with no reason field; hook (`mod.rs:138-141`) is passive and publication failure never changes lifecycle state (`mod.rs:151-161`); no `IdlePolicy`/`Requested`/`Failure`/`Unknown` type exists | **fail → blocked** |
| §14.5 manual/failure teardown is not mislabeled | No reason type exists so no mislabeling is possible; retained behavior reports only generic `SessionRemoved` with no reason inference | **pass (vacuous: no reason to mislabel)** |
| §14.6 Proposal Close/CloseTime validation/translation is fail-before-allocation and raw-SAM-free | Current behavior is truthful fail-before-allocation via `client_lifecycle_config` (`session.rs:695-745`) rejecting any `Close`/`CloseTime` key as `UnsupportedOption`, plus per-backend `validate_raw_options` allowlists; `ClientLifecycleConfig` always returns `close_on_idle: false` (`session.rs:739-744`); Yosemite generic `add_session_option` could carry `i2cp.close*` (not reserved) but router would ignore it, so wiring it would be accept-inert (forbidden) | **pass (blocked retained, no accept-inert)** |
| §14.7 shared/cancellation/restart races are covered | `CompatibilityKey` (`session.rs:162-198`) + `additional_options_identity` (`513-527`) would carry exact Close option identity automatically once mapped; no Close mapping exists so no sharing claim is made; `ClientSessionOwner` state machine (`client_listener.rs:47-62,143-218`) serializes Yosemite-session creation without holding locks across I/O, but it is not the core SAM activity/close owner and cannot prove §8 races for the missing primitive | **fail → blocked (no false sharing/race claim)** |
| §14.8 every promoted matrix cell has end-to-end evidence | Zero cells promoted; starting/final cell lists identical (see §3); no end-to-end close-teardown trace exists | **pass (vacuous: no promotion without evidence)** |
| §14.9 no unauthorized path/dependency changes occur | `git diff --check` clean; M061/M062 containment pass (30 tests) with the M062 planning-path guard amended for the new closure file only; no `emissary-core/**` / `emissary-cli/src/**` production, `Cargo.toml`/lockfile, Yosemite, frontend, transport, NetDb, or crypto change | **pass** |
| §14.10 M134 receives a stable idle-close/reopen interface or remains blocked | No stable reason/reopen boundary exists; M134 remains deferred/unregistered with explicit decision (see §11) | **pass (explicit decision: M134 remains blocked)** |
| WP1 rebase after M132 | Reviewed M132 closure head `6618c49`, changed paths, missing-owner findings (§3.2–§3.5); readiness gate fails on all six bullets; no registration amendment can cure a missing owner without a future proven reduction primitive | **pass (rebase proves blocked)** |
| WP2 close semantics freeze | Reference table in §3.5 records planning-time facts vs unresolved items; reduce+close suppression, same-clock evaluation, and teardown sequence retained as plan facts without independent re-proof | **blocked (freeze without guessing)** |
| WP3–WP4 idle state machine + reasoned termination | Not implemented; stop conditions triggered (see §6/§10); no partial core edit landed to avoid approximate support and a second competing timer | **blocked (no code)** |
| WP5–WP6 I2PControl translation + shared/cancellation evidence | Not enabled; current fail-before-allocation gates retained and covered by existing `client_lifecycle_config` and backend rejection tests; `NewDest` remains blocked throughout (§5) | **blocked (no code)** |
| WP7 matrix/docs/closure | Matrix mechanically recomputed unchanged; support docs already state Close family fails before allocation — no doc change required; this closure + registry/roadmap updates are the WP7 output | **pass** |

## 3. Production implementation evidence

No production implementation exists under M133. The following reviewed-head
evidence establishes the missing-primitive finding and the stop-condition
basis. All line references are to the reviewed head `6618c49`.

### 3.1 Starting/final cell lists (identical)

Canonical tunnel order: `client`, `httpclient`, `ircclient`, `socks`,
`socksirc`, `connectclient`, `streamrclient`, `server`, `httpserver`,
`httpbidirserver`, `ircserver`, `streamrserver`.

| Option | Cells (12) | Blocked families (7) |
|---|---|---|
| `Close` | `blocked×7, N/A×5` | `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient` |
| `CloseTime` | `blocked×7, N/A×5` | same 7 |

Source: `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml:1201-1231`.
M131 map authority for the same 14 cells: `131-residual-primitive-map.toml`
`Close:*` (7 families) and `CloseTime:*` (7 families), all
`primitive_cluster = "session-lifecycle"`,
`path_budget = "PB-SESSION-IDLE-CLOSE-01"`,
`dependency_readiness = "not_ready"`.

Maximum promotion under the plan was 14 (only with affirmative reference
evidence and end-to-end teardown traces; Streamr applicability must be
rechecked per §12 and no Streamr cell may be promoted solely because the
core primitive can technically apply). Actual promotion: 0.

### 3.2 Neutral core: no idle state machine to extend

- `SamSession` (`emissary-core/src/sam/session.rs:68-148`) owns address book,
  datagram/stream managers, destination, socket, session kind, and
  observation hook. There is no activity timestamp, idle timer, close policy,
  termination reason, or `i2cp.close*`/`i2cp.reduce*` option consumption. The
  only `i2cp.*` reads are `i2cp.dontPublishLeaseSet` (`session.rs:215,1421`)
  and a test default `i2cp.leaseSetEncType` (`session.rs:1551`).
- The plan (§4.1) requires extending the M132 state machine rather than
  introducing a second competing timer, with active/reduced/closing states
  computed from the same last-activity value and actor-local bounded
  rescheduling. No such owner exists, so no close deadline can be added
  without creating the forbidden second timer subsystem.
- The plan (§4.2) requires reusing the canonical teardown path (stop
  accepting commands for the generation, shut down stream
  manager/destination/pool in existing order, remove session/subsession
  mappings through the server owner, emit no fake success). The ordinary
  `SessionRemoved` path exists (`sam/mod.rs:961-1000`), but there is no idle
  trigger wired to it and no proof that a future idle trigger would preserve
  the existing order without a second pool-shutdown implementation.
- Do not infer close from lack of local TCP handlers (§3): the only
  idle-closer in the tree counts accepted local TCP handler guards
  (`client_listener.rs:240-270`), sleeps `idle_time`, and drops a Yosemite
  session generation (`client_listener.rs:220-231,272-297`). That is exactly
  the heuristic the plan forbids as the Proposal activity predicate, and it
  is unreachable via Proposal `Close` because `client_lifecycle_config`
  always returns `close_on_idle: false`.

### 3.3 Neutral termination reason: no in-process fact exists

- `SamObservationEvent` (`emissary-core/src/sam/mod.rs:89-128`) carries only
  session/socket identifiers and sanitized addresses. `SessionRemoved`
  carries only `session_id` with no reason, generation, or policy fact.
- The hook trait (`mod.rs:134-141`) is synchronous and passive; publication
  failure is logged and never changes lifecycle state (`mod.rs:151-161`).
  Extending it with a neutral `IdlePolicy` / `Requested` / `Failure` /
  `Unknown` enum (plan §4.3) would be the preferred narrow seam, but no such
  type exists and no authoritative removal path carries a reason. Inventing a
  reason without the winning-transition owner (§8: idle vs manual vs failure
  race, `Unknown` when indistinguishable) would fabricate lifecycle facts.
- M133 must not invent a new SAM wire response/event solely for I2PControl
  (§4.3). No wire field was added.

### 3.4 Yosemite: typed close fields are dormant

- Yosemite Y005 `SessionOptions` declares `close_on_idle: bool` (default
  `false`) and `close_idle_time: Duration` (default 30 min / 1800000 ms)
  (`options.rs:629-634,1058-1059`), alongside dormant `reduce_on_idle` /
  `reduce_idle_time` / `reduce_quantity`.
- `SESSION CREATE` serialization (`proto/session.rs:177-310`) emits
  destination, ports, `dontPublishLeaseSet`, `leaseSetEncType`, typed
  LeaseSet options, `inbound.*/outbound.*`, `SIGNATURE_TYPE`, sorted generic
  `additional_options`, and LeaseSet typed options. It never emits
  `close_on_idle` / `close_idle_time` / `reduce_*`. The typed fields are
  therefore dormant: setting them has no wire effect.
- Generic `SessionOption::new` (`options.rs:95-118`) permits
  `i2cp.closeOnIdle` / `i2cp.closeIdleTime` (alphanumeric plus `.`/`_`/`-`,
  not in `is_reserved_session_option_key`), and `additional_options` are
  serialized sorted. The plan-authorized wire path (§5: `Close` →
  `i2cp.closeOnIdle`, `CloseTime` → `i2cp.closeIdleTime` via validated
  additional options) is technically serializable without a Yosemite change,
  but the Emissary router (`SamSession`) ignores those keys, so emitting
  them from I2PControl today would be accept-inert serializer reachability —
  explicitly not support. No such emission was added.

### 3.5 I2PControl: fail-before-allocation preserved, NewDest still blocked

- Canonical keys `Close` (boolean) and `CloseTime` (integer/duration) are
  accepted by `validate_canonical_options`
  (`emissary-cli/src/i2pcontrol/tunnel_manager.rs:1688-1701`) into lossless
  `raw_config`.
- `client_lifecycle_config` (`backends/runtime/session.rs:695-745`) rejects
  any `Close`/`CloseTime` key before listener/session allocation as
  `BackendError::UnsupportedOption`, and rejects `NewDest` via
  `definition.options.new_dest`. `ClientLifecycleConfig` always returns
  `close_on_idle: false` with `DEFAULT_CLOSE_IDLE_TIME` (30 min) as a
  placeholder, never as an enabled policy.
- Every client backend additionally rejects unknown `raw_config` keys before
  allocation via its `validate_raw_options` allowlist, which does not
  contain `Close`/`CloseTime`/`NewDest` (`client.rs:411-455`,
  `connect_client.rs:710-743`, `http_client.rs:692-728`,
  `socks.rs:1109-1145`, plus `irc_client`, `streamr`, and server
  equivalents). `CompatibilityKey` exact additional-option identity
  (`session.rs:162-198,491-527`) is therefore unaffected.
- Support documentation already states the Close family fails before
  listener/session allocation (`docs/i2pcontrol/tunnel-manager.md:362-368,392`;
  `docs/i2pcontrol/proposal-170-support.md:513-515`). No doc change was
  required.
- `NewDest` remains blocked throughout M133 as required (§5): no NewDest
  gate was removed, no successor identity is staged, and M134 is the only
  milestone allowed to remove that gate.

### 3.6 Reference semantic table (WP2 freeze)

| # | Claim | Source / status |
|---|---|---|
| 1 | `i2cp.closeOnIdle=true` enables idle session destruction | I2CP spec options table; Yosemite `close_on_idle` default `false` — confirmed |
| 2 | `i2cp.closeIdleTime` is milliseconds | I2CP spec option unit; Yosemite `Duration::from_millis(1800000)` — confirmed |
| 3 | Java default close time is 30 minutes | I2CP spec (`1800000`); Yosemite default — confirmed as spec/default fact |
| 4 | Java minimum is 5 minutes | I2CP spec (`300000 minimum`) — confirmed as spec fact |
| 5 | Close is evaluated against the same session activity clock used by reduction | Plan fact; no M132 activity clock exists to share; Java `SessionIdleTimer` same-clock source not retrieved — **not independently re-proved** |
| 6 | When reduce and close are both enabled and close time <= reduce time, Java suppresses reduction | Plan fact; Java `SessionIdleTimer` ordering source not retrieved — **not independently re-proved** |
| 7 | Close-on-idle destroys the I2P session, tearing down its tunnel pool | I2CP session-destruction architecture; Emissary ordinary `SessionRemoved` path exists but no idle trigger — confirmed as architectural fact, not as implemented idle behavior |
| 8 | Activity before the close threshold postpones close | Plan fact; same Java source gap as items 5–6 — **not independently re-proved** |
| 9 | After close, a later owner may reopen a new session; reopen policy is outside the neutral close primitive | Plan scoping fact; no reopen was implemented — confirmed as scope boundary |
| 10 | Close must not be inferred from local TCP handler count | M121 §5.2 + M132 §4.1; the only idle-closer in-tree (`client_listener.rs`) is exactly that heuristic and is unreachable via Proposal `Close` — confirmed |
| 11 | Primary/subsession activity aggregation; closing primary closes dependents | I2CP multisession notes (subsessions share primary pool); Java aggregation source not retrieved — **retained as plan fact** |
| 12 | Shared definitions using one Yosemite session share close policy/clock; incompatible policy must not share | `CompatibilityKey` mechanism exists but no Close mapping exists — confirmed as mechanism, not as implemented sharing |

Items 1–4, 7 (architecture), 9–10 are sufficient to retain the current
blocked dispositions. Items 5–6, 8, 11 are retained as plan facts without
independent re-proof. Together with the missing M132 owner they are the
formal stop-condition triggers: implementation without them would be
guessing with session-lifecycle and anonymity impact.

## 4. Verification executed

All commands run at the reviewed head `6618c49`. No production file was
modified, so all suites exercise the last-known-good baseline.

### Commands run

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib backends::runtime::session
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib backends::runtime::client_listener
cargo check -p emissary-core
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

`cargo test -p emissary-core`, the full
`cargo test -p emissary-cli --no-default-features --features i2pcontrol`, and
`i2pcontrol_live_runtime` were not re-run: no production path changed, and
the plan's broad-verification intent (no unexplained regression on the
affected surface) is satisfied by the containment/matrix guards plus
checks/clippy and the focused `session`/`client_listener` suites on the
exact affected crates. The live-runtime suite remains qualified by M130;
its evidence is inherited, not re-claimed.

### Results

| Command | Result |
|---|---|
| `m095 + m105 matrix/audit` | **pass**: `4 passed (2 suites)`; matrix hash `f03852…f79` agrees with M131/M132; counts `284/88/468` |
| `m061 + m062 containment` | **pass**: `30 passed (2 suites)`; no unauthorized production/dependency path |
| `--lib backends::runtime::session` | **pass**: `15 passed`; `Close`/`CloseTime`/`NewDest` fail-before-allocation gates green |
| `--lib backends::runtime::client_listener` | **pass**: `12 passed`; TCP-heuristic idle-closer isolated from Proposal path |
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` (workspace) | **pass** |
| `clippy -p emissary-core --all-targets -- -D warnings` | **pass**: `No issues found` |
| `clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **pass**: `No issues found` |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift across 615 files (repository-configured nightly-only options unavailable under stable: `imports_granularity`, `wrap_comments`, `comment_width`, `trailing_comma`, `match_arm_blocks`, `spaces_around_ranges`); not normalized; no M133 file changed |
| `git diff --check` | **pass** (no whitespace errors; no production diff) |

Focused M133 tests (§10 items 1–15) were not added because the underlying
capability does not exist. Adding tests against a non-existent primitive
would either assert current blocked behavior already covered by
`client_lifecycle_config` / `validate_raw_options` tests, or fabricate a
close/reason contract without reference truth. The existing backend
rejection tests remain the correct regression authority for the retained
blocked disposition.

## 5. Invariant review

- Exact pinned names/types/presence: preserved. No new field, alias, status,
  method, or tunnel type was added. `Close`/`CloseTime` remain canonical
  keys with `blocked_primitive` disposition for the 7 applicable families;
  `NewDest` remains blocked throughout.
- No fabricated/accept-inert support: preserved. No serializer-only or
  storage-only Close path was added. Yosemite dormant typed fields were not
  mapped to fake support.
- Every `apply` cell changes real behavior: preserved (zero promotions).
- Unsupported values fail before allocation: preserved via
  `client_lifecycle_config` plus per-backend `validate_raw_options`
  allowlists; verified by focused session suites and containment/matrix
  suites.
- No direct-clearnet fallback, no outproxy boundary change, no trusted-peer
  or Streamr isolation change: preserved (no data-plane change).
- Loopback confinement, bounded admission/tasks/timers, transactional
  edit/start/restart, last-known-good: preserved (no lifecycle change).
- No lock across network/filesystem I/O introduced: preserved (no new lock,
  task, timer, or queue).
- Secret/key/path redaction: preserved (no new log/event/Debug output; no
  destination/key material in any new type).
- No LeaseSet downgrade: preserved (no LeaseSet crypto/scope change).
- Feature/runtime isolation, no base-method parity, no frontend coupling:
  preserved.
- External interaction read-only/internal-only: preserved (see attestation).

## 6. Failure, cancellation, restart, and contention review

No new failure domain was introduced. The retained behavior is:

- Idle timers: none exist at the SAM/I2CP activity boundary; there is no
  session-generation close timer to leak, persist, or survive restart.
  Process restart starts a fresh session generation with no idle timestamp
  persisted (unchanged).
- The I2PControl-local `run_idle_closer` (`client_listener.rs:272-297`) is a
  TCP-handler heuristic explicitly excluded from Proposal semantics. It is
  unreachable via `Close` (always `false` from `client_lifecycle_config`),
  generation-local via `ClientSessionOwner`, aborted on stop/drain
  (`client_listener.rs:529-532,554-557`), and never reports a neutral
  `IdlePolicy` reason. It cannot leak into a replacement generation as a
  Proposal close fact.
- Pool/session shutdown: unchanged. `close_if_idle` drops only a
  Yosemite-session `SessionResource`; core SAM session/pool shutdown still
  wins via the existing `SamServer` removal path. No double-remove was
  introduced because no idle trigger was added.
- Termination reason: inapplicable — no reason is emitted, so no race can
  mislabel manual/failure teardown as idle, no stale generation reason can
  qualify a replacement generation, and no observer failure can block
  teardown (there is no new observer path).
- Shared-session registry locks (`backends/runtime/session.rs:269-465`) are
  never held across Yosemite/network I/O (established by M116/M123,
  unchanged).
- Pool locks/state are never held across tunnel-build network I/O
  (unchanged).
- The stop-condition analysis in §3.2/§3.3 is itself the contention review:
  authoritative close-vs-manual-vs-failure arbitration and one-teardown
  idempotence cannot be proved without the missing owner, so none was
  improvised.

## 7. Migration and compatibility review

- No public API version, method, tunnel type, or action change.
- No durable-store migration. `Close`/`CloseTime` values already round-trip
  losslessly in `raw_config`; they continue to do so while failing before
  allocation at start. Canonical `get` output is unchanged.
- SAM-created destination behavior without close options is
  byte-for-byte/configuration-equivalent at the option level and behaviorally
  equivalent at runtime (no SAM/I2CP option consumption change).
- No wire extension was added to SAM or Yosemite.
- Existing definitions containing blocked `Close`/`CloseTime` values remain
  round-trippable; successful start becomes available only after a future
  validated runtime exists.
- Rollback: not applicable (no state change to roll back); last-known-good
  M130/M131 runtime behavior is retained.

## 8. Security review

- One session cannot close another session through the new primitive:
  preserved — no close-control seam was added. Exploratory/participating
  pools remain unreachable from any session control path.
- Core reason types contain no Proposal/I2PControl concepts or secrets:
  preserved — no core type was added; `SamObservationEvent` carries only
  sanitized identifiers/addresses.
- No public/general router-control endpoint is introduced: preserved.
- No new SAM wire field/status/event is added: preserved.
- Termination reason cannot be spoofed by a remote peer or local
  unauthenticated SAM payload: preserved — no new inbound control parsing
  was added and no reason is emitted at all.
- No secret/session key/raw destination in new logs/events/Debug: preserved
  — no new output was added. `CompatibilityKey` redaction unchanged.
- Malformed/below-minimum/overflow Proposal `CloseTime` fails before
  allocation: preserved (§3.5); exact bounds await a future frozen contract
  but no value is accepted today.
- Core contains no Proposal/I2PControl names: preserved — no core file was
  touched. Static containment (`m062_dependency_containment`, 30 tests)
  passes without production amendment, confirming no M062 exact-path
  authority change was needed beyond the closure planning path.
- Remote peers and unauthenticated local SAM payloads gain no new influence:
  no new inbound control parsing was added.
- M132 behavior remains green (inherited containment/matrix/checks/clippy).

## 9. Documentation and operations

- `docs/i2pcontrol/tunnel-manager.md` and
  `docs/i2pcontrol/proposal-170-support.md` already document the Close family
  as rejected before allocation as residual blockers (M121 §5.2). No change
  was required and none was made.
- Machine-readable authorities unchanged and reconciled:
  `095-full-support-matrix.toml` (`284/88/468`,
  `f038521da9cc685bd38dd502f02dcc81f53586c3a8dd83eb3ba5a8827f589f79`),
  `105-residual-option-audit.toml`, `110-completion-ledger.toml`
  (M131 reconciliation retained), `131-residual-primitive-map.toml`
  (`PB-SESSION-IDLE-CLOSE-01`, `not_ready` retained).
- Static guards green with one planning-path amendment: M061/M062
  containment and M095/M105 matrix tests pass. `is_authorized_planning_path`
  in `emissary-cli/tests/m062_dependency_containment.rs` gains exactly one
  line authorizing the new planning record
  `plans/closure/i2pcontrol-proposal-170/133-closure.md` (M132 precedent).
  All other changed paths were already authorized planning paths
  (`133-*.md` plan via `062-dependency-containment.toml` root_manifests,
  `README.md`, `registry.md`, full-support roadmap via the M062 planning
  allowlist, session-lifecycle roadmap via the same TOML root_manifests).
  No production-path authorization was added; the guards still reject
  unauthorized expansion (`core` Proposal terms, Cargo/Yosemite changes,
  M095/M105 count/hash drift).
- Exact changed paths in this closure commit: `133-closure.md` (new),
  `133-neutral-sam-idle-close-and-reasoned-termination.md` (status only),
  `plans/implementation/.../README.md`, `plans/registry.md`, both
  subsystem roadmaps above, and the one-line M062 planning-path guard. No
  production, dependency, or lockfile path changed.
- Operational impact: none. No new configuration, metric, diagnostic,
  recovery procedure, or restart requirement.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| high | M132 activity/timer owner still missing (M133 readiness gate §2, all six bullets) | No close deadline can be computed from the same last-activity value; no single state machine can represent active/reduced/closing without a second competing timer | Future reduction plan must land the M132 `SamSession` activity state, monotonic generation-local timer, and pool/session lifecycle contract first; M133 must then be amended/re-based against that closure diff before any registration (per its WP1) |
| high | Exact close/reference ordering lacks direct source (same-clock evaluation, close <= reduce suppression, subsession aggregation, teardown sequence) | Close threshold, reduce+close interaction, and primary/subsession teardown cannot be implemented without guessing | Future plan must retrieve Java `SessionIdleTimer`/session-destruction source or equivalent router behavior evidence before code |
| high | Neutral termination reason has no authoritative owner (`SessionRemoved` carries no reason/generation) | M134 cannot distinguish intentional idle-policy destruction from manual/failure termination without inferring from elapsed time, handler count, or error strings (all forbidden) | Future plan must extend the authoritative lifecycle/removal path with a neutral `IdlePolicy`/`Requested`/`Failure`/`Unknown` fact where the owner can prove it, with `Unknown` fallback and observer-failure-never-blocks-teardown |
| medium | Yosemite typed `close_on_idle`/`close_idle_time` are dormant (declared, never serialized) | I2PControl cannot use the typed API for real effect; only generic `additional_options` reaches the wire, which the router ignores | Any future consumer must use the generic `i2cp.close*` path (no Yosemite change per M133 budget) and prove router-side consumption end-to-end; if a typed-API change is ever needed it requires a separately registered Yosemite dependency plan |
| medium | I2PControl-local `run_idle_closer` is a TCP-handler heuristic, not the reference predicate | Reusing it for Proposal `Close` would violate §3–§4 and M121 truthfulness; it must remain unreachable via `Close` until a real activity clock exists | Future plan must keep Proposal close disabled until the core activity owner lands; must not promote the heuristic to support |
| medium | Shared-session close-policy compatibility has no mapping to participate in `CompatibilityKey` | Compatible-policy sharing vs incompatible-policy separation (§4.4) cannot be proved | Future plan must include Close/CloseTime in the exact compatibility key with regression evidence once core support exists |
| low | Plan §10 focused tests 1–15 have no close implementation to attach to | No regression risk today, but a future implementation must add all 15 deterministic tests plus shared/cancellation/reason/race evidence | Carry forward verbatim into the next close-plan registration |

No high/medium finding is resolved by this closure. All remain open
blockers for any future close attempt and transitively for M134.

## 11. Roadmap disposition

- M133 is **closed as blocked**. It does not establish the neutral idle
  state machine extension, bounded session/pool teardown trigger, neutral
  termination reason, or I2PControl `Close`/`CloseTime` translation required
  for any `Close*` promotion.
- M134 (`134-newdest-on-proven-idle-resume.md`) remains **deferred /
  unregistered**. Its hard dependency ("M133 must close with an authoritative
  generation-local idle-close reason and stable reopen boundary") is not
  satisfied. M134 must not be registered on the basis of this closure. If a
  future reduction primitive ever closes and a future close primitive then
  closes, M134 must be amended/re-based against those closures' actual diffs
  before registration (per its WP1).
- Other M131 residual clusters (presentation `UseSSL`, `SigType`, outproxy
  provider, HTTP `SSLProxies`/`JumpList`, streaming `Profile`, Streamr
  `ConnectDelay`, `UniqueLocalAddressPerClient`, `MultiHoming` /
  `shouldBundleReplyInfo`, encrypted/authenticated LeaseSets) remain
  unregistered under M131 authority. None was smuggled into M133 and none is
  unblocked by this closure.
- M114 remains historically closed as blocked. M130 remains the current
  implemented-subset runtime/security qualification authority. M131 remains
  the current residual applicability/primitive authority.

## 12. Registry updates

Required changes (applied alongside this closure):

- `plans/registry.md`: M133 `deferred / unregistered` → `closed as blocked`;
  session-lifecycle handoff → still no active handoff; M134 remains
  deferred/unregistered; matrix line retains `284/88/468`; M133 added to
  recently-closed table.
- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`:
  status `M132 closed as blocked; M133–M134 dependency-blocked` → `M132
  closed as blocked; M133 closed as blocked; M134 dependency-blocked`;
  current handoff → none; completion rule notes the line did not advance.
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`:
  current handoff M132 closed as blocked → M132/M133 closed as blocked;
  active session-lifecycle line updated; no new executable handoff.
- `plans/implementation/i2pcontrol-proposal-170/README.md`: status
  `M132 closed as blocked` → `M132/M133 closed as blocked`; dependency graph
  updated; no active handoff.
- `plans/implementation/i2pcontrol-proposal-170/133-neutral-sam-idle-close-and-reasoned-termination.md`:
  Status header `deferred / unregistered; hard-depends on M132 closure` →
  `closed as blocked` with closure link.
- Historical closures (M114, M130, M131, M132) unchanged.

## Internal-only / read-only-upstream attestation

- External sources (I2P Proposal 170, I2CP specifications, Java/GitHub
  Javadoc indexes, Yosemite source) were accessed read-only for evidence.
- No upstream or third-party repository, issue, pull request, discussion,
  review, or maintainer channel was opened, drafted, updated, commented on,
  or contacted.
- No commit, branch, tag, patch, release, or artifact was pushed to any
  upstream remote. All writes are internal to `eggstack/emissary` on the
  current branch.
- No upstream review, approval, feedback, adoption, or merge was requested.
- No upstream contribution package, patch series, or submission checklist was
  prepared.
- Violation of the above would invalidate this closure per
  `plans/003-planning-process.md` §11; no such violation occurred.

(End of file)
