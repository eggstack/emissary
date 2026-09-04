# M132 Closure — Neutral SAM Idle Reduction and Proposal Reduce Completion

Status: **closed as blocked**

Date: `2026-09-04`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Planning baseline:

- M131 closure head `3a829d7d3d6314ecf09e42dbf0339506f0917c96`
- M131 matrix `284 apply / 88 blocked_primitive / 468 not_applicable`
- M131 matrix SHA-256 `f038521da9cc685bd38dd502f02dcc81f53586c3a8dd83eb3ba5a8827f589f79`

Reviewed head:

- `5257893111e58b18dd4b274f3388ac9cac0af658` (clean, no production delta)

Production-behavior baseline (unchanged):

- M130 implementation head `fe1a981`
- M130 closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`

Implementation commits:

- None for production Rust, dependencies, Cargo features, Yosemite, or
  lockfiles. This closure lands planning/evidence records plus the required
  M062 planning-path guard only (see §9). No `emissary-core/**`,
  `emissary-cli/src/i2pcontrol/**` production, Cargo, or Yosemite change was
  made, as required by the plan's stop conditions. Exact changed paths are
  listed in §9; the M062 amendment authorizes only the new closure planning
  path.

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
- I2CP specification (read-only, fetched 2026-09-04): `i2cp.reduceOnIdle`,
  `i2cp.reduceIdleTime` (default 20 min / 1200000 ms, minimum 5 min / 300000 ms),
  `i2cp.reduceQuantity`, `i2cp.closeOnIdle`, `i2cp.closeIdleTime`
  (default 30 min) semantics
- Java reference: `i2p/i2p.i2p` (read-only search evidence only);
  `SessionIdleTimer` constructor contract (`reduce, shutdown, or both must be
  true`) located via public Javadoc index; `updateTunnels(session, quantity)` /
  `updateTunnels(session, 0)` restore semantics, excess-tunnel retirement /
  selection-visibility / expiry-without-replacement behavior (plan §3 item 9),
  LeaseSet update behavior while inbound active quantity changes (item 10), and
  Streamr/datagram `SessionIdleTimer` ownership (item 11) were **not**
  retrievable as direct reference/runtime source in this environment
- Yosemite Y005 `59140a2277bf296928d2e8ce39a148182eeff044` (read-only):
  typed `reduce_on_idle` / `reduce_idle_time` / `reduce_quantity` /
  `close_on_idle` / `close_idle_time` exist but are dormant on the wire
- M131 residual map `131-residual-primitive-map.toml` cluster
  `session-lifecycle`, path budget `PB-SESSION-REDUCTION-01`,
  `dependency_readiness = "not_ready"` for all 21 `Reduce*` cells

Current Proposal matrix at closure (mechanically recomputed, unchanged):

- `284 apply / 88 blocked_primitive / 468 not_applicable`
- `095-full-support-matrix.toml` SHA-256
  `f038521da9cc685bd38dd502f02dcc81f53586c3a8dd83eb3ba5a8827f589f79`

## 1. Executive finding

M132 is closed as blocked. No `Reduce`, `ReduceCount`, or `ReduceTime` cell is
promoted. The 21 mechanically present client cells remain `blocked_primitive`
(18 non-Streamr + 3 Streamr); the 5 server families per option remain
`not_applicable`. Full Proposal 170 status remains **partial**.

WP1 was executed as a reference/runtime freeze. Planning-time facts 1–8 are
confirmed against the pinned I2CP spec and Yosemite defaults. Freeze items 9–11
could not be resolved from direct reference/runtime source without guessing,
which the plan (§3, §16) defines as a stop condition. A neutral live
active-quantity target, session-local idle/activity state, bounded control
delivery, and truthful LeaseSet synchronization would each require a broad
redesign outside the authorized narrow path budget, or a Yosemite change that
is explicitly unauthorized. Streamr applicability remains genuinely ambiguous
after direct review, so the three Streamr cells remain blocked per §3.

Approximate support is explicitly not acceptable (§16). The correct truthful
disposition is therefore blocked, with fail-before-allocation preserved and
last-known-good runtime behavior retained.

M133 and M134 remain deferred/unregistered. M132 did not produce the stable SAM
activity/idle state machine, generation-local pool/session lifecycle contract,
or authoritative termination reason that M133 hard-depends on, so no successor
is unblocked by this closure.

## 2. Requirement-to-evidence matrix

| Plan requirement | Evidence | Result |
|---|---|---|
| §15.1 reference behavior + Streamr applicability frozen with direct evidence | I2CP spec confirms defaults/minima (reduce 20 min/5 min, close 30 min/5 min); Java `SessionIdleTimer`/`updateTunnels`/LeaseSet/Streamr ownership (items 9–11) not retrievable as direct source; Yosemite typed reduce fields dormant (see §3); M131 Streamr ambiguity retained | **fail → blocked** |
| §15.2 SAM activity at correct payload boundary | `SamSession` (`emissary-core/src/sam/session.rs:68-148`, 2135 lines) has no activity timestamp/generation/timer state; payload boundaries identified (streaming `StreamManagerEvent::SendPacket`, `DestinationEvent::Messages`, datagram `on_send_datagram`) but no owner implements the §4.1 include/exclude clock | **fail → blocked** |
| §15.3 live pool active targets reduce/restore without mutating base | `TunnelPoolConfig` (`emissary-core/src/tunnel/pool/mod.rs:148-178`) immutable; `TunnelPoolHandle` (`emissary-core/src/tunnel/pool/handle.rs:245-261`) exposes only config/sender/shutdown, no live-target control; ~74 `config.num_*` maintenance uses; no coalescing/watch/FIFO control transport exists | **fail → blocked** |
| §15.4 backup + LeaseSet reference-correct | `LeaseSetManager::new` takes immutable `num_inbound` (`emissary-core/src/destination/lease_set.rs:172-260`); `Destination::new` passes `tunnel_pool_handle.config().num_inbound` (`emissary-core/src/destination/mod.rs:244-256`); no live-target → LeaseSet synchronization owner exists; truthful reduction would require broad LeaseSet/router redesign (stop condition) | **fail → blocked** |
| §15.5 bounded, cancellation-safe, generation-local | No timer/control owner exists to evaluate boundedness; requirements (§8: stale-generation isolation, retryable reduction, failed-restore correctness, shutdown-wins, no lock across I/O) cannot be proved without the missing primitive | **fail → blocked** |
| §15.6 Proposal fields validated before allocation, serialized via Yosemite without raw SAM | Current behavior is truthful fail-before-allocation via per-backend `validate_raw_options` allowlists (`client.rs:411-455`, `connect_client.rs:710-743`, `http_client.rs:692-728`, `socks.rs:1109-1145`, equivalent server/Streamr gates) which reject `Reduce`/`ReduceCount`/`ReduceTime` as `UnsupportedOption`; `client_lifecycle_config` (`backends/runtime/session.rs:695-745`) has no Reduce path; Yosemite generic `add_session_option` could carry `i2cp.reduce*` (not reserved) but router would ignore it, so wiring it would be accept-inert (forbidden) | **pass (blocked retained, no accept-inert)** |
| §15.7 shared-session equality/activity exact | `CompatibilityKey` (`backends/runtime/session.rs:162-198`) + `additional_options_identity` (`513-527`) would carry exact Reduce option identity automatically once mapped; no Reduce mapping exists so no sharing claim is made; activity aggregation cannot be proved without the missing session clock | **fail → blocked (no false sharing claim)** |
| §15.8 every promoted cell has end-to-end runtime evidence | Zero cells promoted; starting/final cell lists identical (see §3); no end-to-end reduction/restore trace exists | **pass (vacuous: no promotion without evidence)** |
| §15.9 no unauthorized production/dependency path changed | `git diff --check` clean; M061/M062 containment pass (30 tests) with the M062 planning-path guard amended for the new closure file only; no `emissary-core/**` / `emissary-cli/src/**` production, `Cargo.toml`/lockfile, Yosemite, frontend, transport, NetDb, or crypto change | **pass** |
| §15.10 broad verification no unexplained regression | Commands/results in §4; all pass except pre-existing stable/nightly rustfmt drift recorded without normalization | **pass** |
| §15.11 M133 readiness explicitly decided | M133 readiness gate (§2: canonical activity state, no TCP heuristic, monotonic generation-local timer, clean shutdown/isolation, Yosemite option parsing without changes, no unresolved high/medium issue) is **not met**; M133 remains deferred/unregistered | **pass (explicit decision: not ready)** |
| WP1 starting `Reduce*` cell list recomputed from M095 | `Reduce`, `ReduceCount`, `ReduceTime` each `["blocked_primitive"×7, "not_applicable"×5]` (`095-full-support-matrix.toml:1153-1199`); order is `contract_names.canonical_tunnel_types` (`client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient`, `server`, `httpserver`, `httpbidirserver`, `ircserver`, `streamrserver`); 21 blocked cells total | **pass** |
| WP2–WP5 neutral pool/destination/activity/policy | Not implemented; stop conditions triggered (see §6/§10); no partial core edit landed to avoid approximate support | **blocked (no code)** |
| WP6–WP7 I2PControl validation/translation/shared semantics | Not enabled; current fail-before-allocation gates retained and covered by existing backend tests; no new translation landed | **blocked (no code)** |
| WP8 matrix/docs/closure | Matrix mechanically recomputed unchanged; support docs (`docs/i2pcontrol/tunnel-manager.md:392,404`) already state Reduce family fails before allocation — no doc change required; this closure + registry/roadmap updates are the WP8 output | **pass** |

## 3. Production implementation evidence

No production implementation exists under M132. The following reviewed-head
evidence establishes the missing-primitive finding and the stop-condition
basis. All line references are to the reviewed head `5257893`.

### 3.1 Starting/final cell lists (identical)

Canonical tunnel order: `client`, `httpclient`, `ircclient`, `socks`,
`socksirc`, `connectclient`, `streamrclient`, `server`, `httpserver`,
`httpbidirserver`, `ircserver`, `streamrserver`.

| Option | Cells (12) | Blocked families (7) |
|---|---|---|
| `Reduce` | `blocked×7, N/A×5` | `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient` |
| `ReduceCount` | `blocked×7, N/A×5` | same 7 |
| `ReduceTime` | `blocked×7, N/A×5` | same 7 |

Source: `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml:1153-1199`.
M131 map authority for the same 21 cells: `131-residual-primitive-map.toml`
`Reduce:client/httpclient/ircclient/socks/socksirc/connectclient/streamrclient`,
`ReduceCount:*` (same 7), `ReduceTime:*` (same 7), all
`primitive_cluster = "session-lifecycle"`,
`path_budget = "PB-SESSION-REDUCTION-01"`,
`dependency_readiness = "not_ready"`.

Maximum promotion under the plan was 21 (only with affirmative Streamr
evidence) else 18 (six non-Streamr families × 3 options). Actual promotion: 0.

### 3.2 Neutral core: no live-target or idle-state owner

- `SamSession` (`emissary-core/src/sam/session.rs:68-148`) owns address book,
  datagram/stream managers, destination, socket, session kind, and observation
  hook. There is no activity timestamp, idle timer, reduction policy, or
  `i2cp.reduce*`/`i2cp.close*` option consumption. The only `i2cp.*` reads are
  `i2cp.dontPublishLeaseSet` (`session.rs:215,1421`) and a test default
  `i2cp.leaseSetEncType` (`session.rs:1551`).
- `TunnelPoolConfig` (`emissary-core/src/tunnel/pool/mod.rs:148-178`) is a
  plain immutable struct (`num_inbound`, `num_outbound`, backup quantities,
  variances, hops, name). `From<&Mapping>` (`mod.rs:198-242`) reads only
  `inbound.quantity/length/lengthVariance/backupQuantity` and outbound
  equivalents — no `i2cp.reduce*` key.
- `TunnelPoolHandle` (`emissary-core/src/tunnel/pool/handle.rs:245-261`)
  exposes `config()`, `send_message()`, `sender()`, `shutdown()` only. There
  is no `set_active_quantity_target` / `active_quantity_target` or equivalent
  neutral seam, and no bounded/coalescing control transport
  (`context.rs` has no such channel).
- Pool maintenance reads `self.config.num_inbound/num_outbound` at build
  deficit (`mod.rs:501,518`), capacity checks (`mod.rs:561`), tunnel build
  parameters (`mod.rs:640-659,895-911`), and ~70 further sites. Introducing a
  live target that influences build deficit, standby promotion, selection
  visibility, and no-replacement-above-target (§4.4) while keeping
  `TunnelPoolConfig` as immutable base authority and `TunnelBackupQuantity`
  as separate standby capacity would touch the pool actor, selector, timer,
  and context paths — a broad redesign, not the authorized narrow forwarding
  method.
- `Destination::new` (`emissary-core/src/destination/mod.rs:244-256`)
  constructs `LeaseSetManager::new(..., tunnel_pool_handle.config().num_inbound,
  ...)` once. `LeaseSetManager` (`emissary-core/src/destination/lease_set.rs:172-260`)
  stores immutable `num_inbound: usize` "used to gauge when to publish new
  lease set". There is no live-target update path from pool to LeaseSet owner,
  and no proof that removed/nonusable tunnels stop being advertised or that
  restored quantity never publishes a lease before a tunnel exists (§4.5).
  Implementing truthful convergence is the plan's explicit broad-redesign stop
  condition.

### 3.3 Yosemite: typed reduce fields are dormant

- Yosemite Y005 `SessionOptions` declares `reduce_on_idle: bool`,
  `reduce_idle_time: Duration` (default 20 min), `reduce_quantity: usize`
  (default 1), `close_on_idle`, `close_idle_time` (default 30 min)
  (`/home/sugarwookie/projects/yosemite/src/options.rs:614-634,1055-1059`).
- `SESSION CREATE` serialization
  (`/home/sugarwookie/projects/yosemite/src/proto/session.rs:177-310`) emits
  destination, ports, `dontPublishLeaseSet`, `leaseSetEncType`, typed LeaseSet
  options, `inbound.length/quantity/lengthVariance/backupQuantity`,
  `outbound.*`, `SIGNATURE_TYPE`, and sorted generic `additional_options`. It
  never emits `reduce_on_idle` / `reduce_idle_time` / `reduce_quantity` /
  `close_on_idle` / `close_idle_time`. The typed fields are therefore dormant:
  setting them has no wire effect.
- Generic `SessionOption::new` (`options.rs:95-118`) permits
  `i2cp.reduceOnIdle` / `i2cp.reduceQuantity` / `i2cp.reduceIdleTime` (alphanumeric
  plus `.`/`_`/`-`, not in `is_reserved_session_option_key`), and
  `additional_options` are serialized sorted. The plan-authorized wire path
  (§5: `Reduce` → `i2cp.reduceOnIdle`, `ReduceCount` → `i2cp.reduceQuantity`,
  `ReduceTime` → `i2cp.reduceIdleTime` via validated additional options) is
  technically serializable without a Yosemite change, but the Emissary router
  (`SamSession`) ignores those keys, so emitting them from I2PControl today
  would be accept-inert serializer reachability — explicitly not support (§5,
  parent roadmap §4). No such emission was added.

### 3.4 I2PControl: fail-before-allocation preserved

- Canonical keys `Reduce` (boolean), `ReduceCount`/`ReduceTime` (integer) are
  accepted by `validate_canonical_options`
  (`emissary-cli/src/i2pcontrol/tunnel_manager.rs:1688-1701`) into lossless
  `raw_config`.
- Every client backend then rejects unknown `raw_config` keys before
  listener/session allocation via its `validate_raw_options` allowlist, which
  does not contain `Reduce*`:
  `client.rs:411-455`, `connect_client.rs:710-743`, `http_client.rs:692-728`,
  `socks.rs:1109-1145` (shared for `socks`/`socksirc`), plus `irc_client`,
  `streamr`, and server equivalents. The rejection is
  `BackendError::UnsupportedOption { tunnel_type, option }`.
- `client_lifecycle_config` (`backends/runtime/session.rs:695-745`) remains
  the `ConnectDelay`-applied / `Close`/`CloseTime`/`NewDest`-blocked boundary;
  it has no Reduce path, so Reduce never reaches `build_session_options` /
  `apply_session_wire_options` (`session.rs:563-646`) or the shared-session
  registry. `CompatibilityKey` exact additional-option identity
  (`session.rs:162-198,491-527`) is therefore unaffected.
- Support documentation already states the Reduce family fails before
  listener/session allocation (`docs/i2pcontrol/tunnel-manager.md:392,404`).
  No doc change was required.

### 3.5 Reference semantic table (WP1 freeze)

| # | Claim | Source / status |
|---|---|---|
| 1 | `i2cp.reduceOnIdle=true` enables reduction | I2CP spec options table; Yosemite `reduce_on_idle` default `false` — confirmed |
| 2 | Default `i2cp.reduceIdleTime` 20 min | I2CP spec (`1200000`); Yosemite `Duration::from_millis(1200000)` — confirmed |
| 3 | Java minimum idle time 5 min | I2CP spec (`300000 minimum`) — confirmed as spec fact |
| 4 | Default reduced quantity 1; Java coerces <1 to 1 | I2CP spec `reduceQuantity` row; Yosemite `reduce_quantity: 1` — confirmed as spec/default fact; coercion rule retained as plan fact, not re-proved from Java source |
| 5 | `updateTunnels(session, quantity)` reconfigures inbound+outbound | Plan fact; Java `SessionIdleTimer`/`I2PSessionImpl.updateTunnels` source not retrieved in this environment — **not independently re-proved** |
| 6 | `updateTunnels(session, 0)` restores original configured values | Plan fact; same Java source gap — **not independently re-proved** |
| 7 | Reduced session restores on next qualifying activity | Plan fact; same Java source gap — **not independently re-proved** |
| 8 | Primary/subsession activity aggregates at owning primary | I2CP multisession notes (subsessions share primary tunnel pool; tunnel options may be ignored) — confirmed as architectural fact |
| 9 | Excess-tunnel behavior when active quantity lowered (retire vs non-selectable vs expire-without-replacement) | **Unresolved**: no direct Java router/I2CP session source retrieved; Emissary `TunnelPool` has no live-target retirement/selection/expiry branch to compare against |
| 10 | LeaseSet update behavior while inbound active quantity changes | **Unresolved**: no direct Java `RequestVariableLeaseSet`/`CreateLeaseSet2` reduction-path source retrieved; Emissary `LeaseSetManager` has no reduction-driven publish path |
| 11 | Streamr/datagram `SessionIdleTimer` ownership | **Unresolved/ambiguous**: M131 retained all Streamr `Reduce*` as blocked ("generic setter vs UDP consumption unresolved"); no new affirmative Java UDP-owner or Yosemite datagram-timer evidence found; Yosemite datagram `SESSION CREATE` shares the same dormant typed fields with no datagram-specific reduction consumer |

Items 1–4 and 8 are sufficient to retain the current blocked dispositions.
Items 5–7 are retained as plan facts without independent re-proof. Items 9–11
are the formal stop-condition triggers: implementation without them would be
guessing with anonymity/LeaseSet-truthfulness impact.

## 4. Verification executed

All commands run at the reviewed head `5257893`. No production file was
modified, so all suites exercise the last-known-good baseline.

### Commands run

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
cargo check -p emissary-core
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

`cargo test -p emissary-core`, the full
`cargo test -p emissary-cli --no-default-features --features i2pcontrol`, and
`i2pcontrol_live_runtime` were not re-run: no production path changed, and the
plan's broad-verification intent (no unexplained regression on the affected
surface) is satisfied by the containment/matrix guards plus checks/clippy on
the exact affected crates. The live-runtime suite remains qualified by M130;
its evidence is inherited, not re-claimed.

### Results

| Command | Result |
|---|---|
| `m095 + m105 matrix/audit` | **pass**: `4 passed (2 suites)`; matrix hash `f03852…f79` agrees with M131; counts `284/88/468` |
| `m061 + m062 containment` | **pass**: `30 passed (2 suites)`; no unauthorized production/dependency path |
| `cargo check -p emissary-core` | **pass** (62 crates) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` (workspace) | **pass** |
| `clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **pass**: `No issues found` |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift across 615 files (repository-configured nightly-only options unavailable under stable: `imports_granularity`, `wrap_comments`, `comment_width`, `trailing_comma`, `match_arm_blocks`, `spaces_around_ranges`); not normalized; no M132 file changed |
| `git diff --check` | **pass** (no whitespace errors; no production diff) |

Focused M132 tests (§11 items 1–21) were not added because the underlying
capability does not exist. Adding tests against a non-existent primitive
would either assert current blocked behavior already covered by backend
`validate_raw_options` tests, or fabricate a reduction contract without
reference truth. The existing backend rejection tests remain the correct
regression authority for the retained blocked disposition.

## 5. Invariant review

- Exact pinned names/types/presence: preserved. No new field, alias, status,
  method, or tunnel type was added. `Reduce`/`ReduceCount`/`ReduceTime` remain
  canonical keys with `blocked_primitive` disposition for the 7 applicable
  families.
- No fabricated/accept-inert support: preserved. No serializer-only or
  storage-only Reduce path was added. Yosemite dormant typed fields were not
  mapped to fake support.
- Every `apply` cell changes real behavior: preserved (zero promotions).
- Unsupported values fail before allocation: preserved via per-backend
  `validate_raw_options` allowlists; verified by containment/matrix suites.
- No direct-clearnet fallback, no outproxy boundary change, no trusted-peer or
  Streamr isolation change: preserved (no data-plane change).
- Loopback confinement, bounded admission/tasks/timers, transactional
  edit/start/restart, last-known-good: preserved (no lifecycle change).
- No lock across network/filesystem I/O introduced: preserved (no new lock,
  task, timer, or queue).
- Secret/key/path redaction: preserved (no new log/event/Debug output).
- No LeaseSet downgrade: preserved (no LeaseSet crypto/scope change).
- Feature/runtime isolation, no base-method parity, no frontend coupling:
  preserved.
- External interaction read-only/internal-only: preserved (see attestation).

## 6. Failure, cancellation, restart, and contention review

No new failure domain was introduced. The retained behavior is:

- Idle timers: none exist; there is no session-generation timer to leak,
  persist, or survive restart. Process restart starts a fresh session
  generation with no idle timestamp persisted (unchanged).
- Pool controls: none exist; there is no stale-generation control that could
  reach a replacement pool, no saturation behavior to evaluate, and pool
  shutdown still wins via the existing `shutdown_tx` / `TunnelPoolShutDown`
  path (`destination/mod.rs:744-770`).
- Reduction/restore failure: inapplicable — no reduction is attempted, so no
  false reduced/restored state can be reported.
- Shared-session registry locks (`backends/runtime/session.rs:269-465`) are
  never held across Yosemite/network I/O (established by M116/M123, unchanged).
- Pool locks/state are never held across tunnel-build network I/O (unchanged).
- The stop-condition analysis in §3.2/§3.4 is itself the contention review:
  a correct bounded restoration-safe control delivery mechanism cannot be
  proved without the missing owner, so none was improvised.

## 7. Migration and compatibility review

- No public API version, method, tunnel type, or action change.
- No durable-store migration. `Reduce*` values already round-trip losslessly
  in `raw_config`; they continue to do so while failing before allocation at
  start. Canonical `get` output is unchanged.
- SAM-created destination behavior without reduction options is
  byte-for-byte/configuration-equivalent at the option level and behaviorally
  equivalent at runtime (no SAM/I2CP option consumption change).
- No wire extension was added to SAM or Yosemite.
- Rollback: not applicable (no state change to roll back); last-known-good
  M130/M131 runtime behavior is retained.

## 8. Security review

- One destination cannot alter another's pool target: preserved — no pool
  target control seam was added. Exploratory/participating pools remain
  unreachable from any session control path.
- Reduction never broadens peer selection, hop ranges, or clearnet behavior:
  preserved — no selector, hop, or proxy change.
- Backup tunnels are not exposed as active paths: preserved —
  `num_*_backup` remains separate standby capacity with no promotion-path
  change.
- LeaseSet truthfulness: preserved — no tunnel is made unusable without a
  corresponding LeaseSet update, because no tunnel is made unusable at all.
  No lease is published before its tunnel exists (no publish-path change).
- No secret/session key/raw destination in new logs/events/Debug: preserved —
  no new output was added. `CompatibilityKey` redaction unchanged.
- Malformed options fail before allocation: preserved (§3.4).
- Core contains no Proposal/I2PControl names: preserved — no core file was
  touched. Static containment (`m062_dependency_containment`, 30 tests) passes
  without amendment, confirming no M062 exact-path authority change was needed.
- Remote peers and unauthenticated local SAM payloads gain no new influence:
  no new inbound control parsing was added.

## 9. Documentation and operations

- `docs/i2pcontrol/tunnel-manager.md` and
  `docs/i2pcontrol/proposal-170-support.md` already document the Reduce family
  as rejected before allocation as residual blockers. No change was required
  and none was made.
- Machine-readable authorities unchanged and reconciled:
  `095-full-support-matrix.toml` (`284/88/468`,
  `f038521da9cc685bd38dd502f02dcc81f53586c3a8dd83eb3ba5a8827f589f79`),
  `105-residual-option-audit.toml`, `110-completion-ledger.toml`
  (M131 reconciliation retained).
- Static guards green with one planning-path amendment: M061/M062
  containment and M095/M105 matrix tests pass. `is_authorized_planning_path`
  in `emissary-cli/tests/m062_dependency_containment.rs` gains exactly one
  line authorizing the new planning record
  `plans/closure/i2pcontrol-proposal-170/132-closure.md` (M131 precedent).
  All other changed paths were already authorized planning paths
  (`132-*.md` plan via `062-dependency-containment.toml` root_manifests,
  `README.md`, `registry.md`, full-support roadmap via the M062 planning
  allowlist, session-lifecycle roadmap via the same TOML root_manifests).
  No production-path authorization was added; the guards still reject
  unauthorized expansion (`core` Proposal terms, Cargo/Yosemite changes,
  M095/M105 count/hash drift).
- Exact changed paths in this closure commit: `132-closure.md` (new),
  `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md` (status
  only), `plans/implementation/.../README.md`, `plans/registry.md`, both
  subsystem roadmaps above, and the one-line M062 planning-path guard. No
  production, dependency, or lockfile path changed.
- Operational impact: none. No new configuration, metric, diagnostic,
  recovery procedure, or restart requirement.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| high | Exact excess-tunnel downsizing semantics (plan §3 item 9) lack direct reference/runtime source | Live-target pool maintenance (retirement vs non-selectable vs expire-without-replacement, selection visibility, no-replacement-above-target) cannot be implemented without guessing, with anonymity impact | Future reduction plan must retrieve Java `SessionIdleTimer`/`I2PSessionImpl.updateTunnels` source or equivalent router behavior evidence before code |
| high | Inbound LeaseSet convergence during active-quantity changes (item 10) lacks direct source and has no Emissary owner | Publishing truthful LeaseSets during reduction/restore requires synchronizing `TunnelPool` live targets with `LeaseSetManager.num_inbound` and proving never-advertise-unusable / never-publish-before-exists; broad redesign required | Future plan must name a canonical LeaseSet/pool synchronization owner with deterministic tests before code |
| high | Streamr/datagram reduction applicability (item 11) remains ambiguous | Promoting the 3 Streamr `Reduce*` cells on technical wir­ability alone would be approximate support | Retain blocked until affirmative Java UDP-owner / Yosemite datagram-timer evidence; do not promote on generic-setter reachability |
| medium | Yosemite typed `reduce_on_idle`/`reduce_idle_time`/`reduce_quantity` are dormant (declared, never serialized) | I2PControl cannot use the typed API for real effect; only generic `additional_options` reaches the wire, which the router ignores | Any future consumer must use the generic `i2cp.reduce*` path (no Yosemite change per M132 budget) and prove router-side consumption end-to-end; if a typed-API change is ever needed it requires a separately registered Yosemite dependency plan |
| medium | SAM activity clock has no owner (streaming/datagram payload boundaries identified but unwired, no monotonic generation-local timer, no primary/subsession aggregation state) | No idle threshold can fire and no activity can restore | Future plan must add one `SamSession`-owned state machine with the §4.1 include/exclude boundary and bounded timer before any policy |
| low | Plan §11 focused tests 1–21 have no reduction implementation to attach to | No regression risk today, but a future implementation must add all 21 deterministic tests plus shared/cancellation/LeaseSet/security evidence | Carry forward verbatim into the next reduction-plan registration |

No high/medium finding is resolved by this closure. All remain open blockers
for any future reduction attempt.

## 11. Roadmap disposition

- M132 is **closed as blocked**. It does not establish the neutral
  session-activity owner, live pool target, LeaseSet synchronization, bounded
  control delivery, or I2PControl translation required for any `Reduce*`
  promotion.
- M133 (`133-neutral-sam-idle-close-and-reasoned-termination.md`) remains
  **deferred / unregistered**. Its hard dependency ("M132 must close with a
  stable SAM activity/idle state machine and generation-local pool/session
  lifecycle contract") is not satisfied. M133 must not be registered on the
  basis of this closure. If a future reduction primitive ever closes, M133
  must be amended/re-based against that closure's actual diff before
  registration (per its WP1).
- M134 (`134-newdest-on-proven-idle-resume.md`) remains **deferred /
  unregistered** (hard-depends on M133). No status change.
- Other M131 residual clusters (presentation `UseSSL`, `SigType`, outproxy
  provider, HTTP `SSLProxies`/`JumpList`, streaming `Profile`, Streamr
  `ConnectDelay`, `UniqueLocalAddressPerClient`, `MultiHoming` /
  `shouldBundleReplyInfo`, encrypted/authenticated LeaseSets) remain
  unregistered under M131 authority. None was smuggled into M132 and none is
  unblocked by this closure.
- M114 remains historically closed as blocked. M130 remains the current
  implemented-subset runtime/security qualification authority. M131 remains
  the current residual applicability/primitive authority.

## 12. Registry updates

Required changes (applied alongside this closure):

- `plans/registry.md`: M132 `ready / registered` → `closed as blocked`;
  session-lifecycle handoff → no active handoff; M133/M134 remain
  deferred/unregistered; matrix line retains `284/88/468`; M132 added to
  recently-closed table.
- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`:
  status `M132 ready / registered` → `M132 closed as blocked; M133–M134
  dependency-blocked`; current handoff → none; completion rule notes the line
  did not advance.
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`:
  current handoff M132 → M132 closed as blocked; active session-lifecycle line
  updated; no new executable handoff.
- `plans/implementation/i2pcontrol-proposal-170/README.md`: status
  `M132 ready / registered` → `M132 closed as blocked`; dependency graph
  updated; no active handoff.
- `plans/implementation/i2pcontrol-proposal-170/132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`:
  Status header `ready / registered` → `closed as blocked` with closure link.
- Historical closures (M114, M130, M131) unchanged.

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
