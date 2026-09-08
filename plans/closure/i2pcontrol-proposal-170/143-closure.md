# M143 Closure — Retained Streaming Profile Runtime Completion

Status: **closed as complete**

Date: `2026-09-08`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/143-streaming-profile-runtime-completion.md`
  (Status now `closed as complete` with closure link; registered via
  `M143-AMEND-01` pre-implementation amendment with exact retained set,
  pinned semantics and exact-file core budget).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## 1. Closure decision

M143 is closed as complete. It is an infrastructure + capability milestone
with a maximum 1-cell promotion budget, the single cell promoted. The current
matrix is now exactly:

- `330 apply`;
- `35 blocked_primitive`;
- `475 not_applicable`;
- `840` total cells.

Promoted cell (exact observable neutral streaming-window effect proven
end-to-end):

- `Profile:client` (plain client only; `bulk`/`interactive` to neutral
  `i2p.streaming.maxWindowSize` 128/16 with bounded StreamManager/Stream
  enforcement).

No high- or medium-severity correctness, streaming or containment issue
remains. Full Proposal 170 support remains partial.

## 2. Reviewed commits and scope

Planning baseline (per M142 closure, M143 hard dependency):

- `48028657065d33241f946020ff412615f71dec2d`.

Implementation starting point (HEAD before M143 edits, clean worktree
verified):

- `48028657065d33241f946020ff412615f71dec2d`.

Implementation plus closure land in a single commit on the current branch
(see git log for the M143 closure commit). The reviewed production-behavior
baseline extends the lifecycle head `e4f217cb1459e26bf011da46b67fc2c83cd192b5`
(M095 `current_production_head`, retained; M143 adds the neutral
streaming-window owner plus the I2PControl-local Profile mapping without
touching NetDB/transport/crypto/tunnel-pool/Cargo/Yosemite/frontend or
workflows).

Changed paths (exact amended budget only; no broader `sam/**` waiver):

- `emissary-core/src/sam/protocol/streaming/config.rs` (neutral
  `i2p.streaming.maxWindowSize` bounds `2..=128`, default `128`, fail-safe
  parsing; `Profile` Clone/Copy; default `max_window_size` 12 → 128 to preserve
  effective bulk behavior when consumed; core-only parsing/bounds tests);
- `emissary-core/src/sam/session.rs` (parse neutral window before activation,
  `StreamManager::new_with_max_window`; no global mutable; no Proposal
  vocabulary);
- `emissary-core/src/sam/protocol/streaming/mod.rs` (generation-local immutable
  `max_window_size`, `new`/`new_with_max_window` + accessor, spawn passes
  `StreamConfig { max_window_size }`; manager window tests);
- `emissary-core/src/sam/protocol/streaming/stream/active.rs` (immutable per-stream
  `max_window_size`, consumes `StreamConfig`, caps `window_size` growth at
  `min(max_window, MAX_WINDOW_SIZE)` with `EXP_GROWTH_STOP_THRESHOLD` respect;
  `max_window_size()`/`window_size()` accessors; interactive-vs-bulk growth
  test);
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` (M143 `StreamProfile`
  validation: exact `"bulk"`/`"interactive"` only, plain-client-only, fail
  before allocation with no echo; `interactive` → generic
  `add_session_option("i2p.streaming.maxWindowSize", "16")`, `bulk`/omitted →
  absent; shared-session compatibility via existing `additional_options_identity`);
- `emissary-cli/src/i2pcontrol/backends/client.rs` (retained-family `SUPPORTED`
  gate adds `"Profile"` only);
- `emissary-cli/tests/m143_streaming_profile.rs` (new; 14 integration guards:
  matrix promotion, residual delta, omitted/bulk/interactive mapping, invalid
  rejection without echo, all 11 N/A families rejection, shared equal/unequal,
  restart successor-only + last-known-good, cancellation no-partial, wire reaches
  owner, observable default-vs-interactive, exact containment, docs agreement,
  no-global/clearnet);
- `emissary-cli/tests/m095_full_support_matrix.rs` (M143 apply expectations,
  `330/35/475` counts, Profile `continue` for apply row);
- `emissary-cli/tests/m105_residual_option_audit.rs` (subtract M143 cell);
- `emissary-cli/tests/m062_dependency_containment.rs` (exact
  `is_authorized_m143_path` for the six production files plus M143
  planning/test/docs paths);
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`
  (add the two not-yet-allowed core files with owner evidence:
  `config.rs` neutral parsing, `stream/active.rs` window enforcement);
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
  (`330/35/475`, Profile row `M143` / `apply_or_not_applicable`);
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml`
  (new `post_m143` 1-cell record);
- `plans/implementation/i2pcontrol-proposal-170/143-streaming-profile-runtime-completion.md`
  (Status `deferred` → `registered` via `M143-AMEND-01` → `closed as complete`);
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `AGENTS.md`;
- `docs/i2pcontrol/proposal-170-support.md`, `docs/i2pcontrol/tunnel-manager.md`,
  `docs/i2pcontrol/tunnel-backends.md`;
- new `plans/closure/i2pcontrol-proposal-170/143-closure.md` (this file).

`git diff --name-only` proves no `emissary-util/**`, manifest, lockfile,
Yosemite, frontend, or workflow path changed. `git diff --check` passes.

## 3. Requirement evidence

### Pinned reference (all accessed read-only)

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (Proposal lists `Profile` string client-management option without a
  per-family value partition; family applicability frozen by M140);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`:
  `TunnelRequestParser.getProfile` returns raw string or null;
  `ClientTunnelCreator.setTunnelManagementOptions` maps exact
  `"interactive".equals(profile)` to
  `option.i2p.streaming.maxWindowSize = "16"` (from
  `TunnelSupport.PROP_DEFAULT_STREAMING_MAX_WINDOW_SIZE = "16"`), else removes;
  `TunnelSupport.PROP_DELAY_DEFAULT_ACTIVE = "500"`;
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`:
  `I2PTunnelClient` performs no streaming override (plain-client survival);
  HTTP/IRC/SOCKS families force bulk via `remove(maxWindowSize)` (M140);
  `StreamrConsumer` extends UDP `I2PTunnelUDPClientBase` (no streaming socket);
  `TunnelController` defaults absent Profile to bulk;
  `GeneralHelper.isInteractive` is `maxWindowSize == 16`;
  `ConnectionOptions` default when absent is `Connection.MAX_WINDOW_SIZE = 128`,
  `setMaxWindowSize` clamps `<2` to `2` and `>256` to `256`, `setProfile`
  documented `Warning: unused`, consumed knobs are `maxWindowSize`/`connectDelay`;
  `Connection` window/inboundBuffer derive from `getMaxWindowSize`,
  `MAX_WINDOW_SIZE = 128`;
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (transport-only generic `add_session_option` path; no M143 dependency change).

### Exact reference freeze applied before coding (M143-AMEND-01)

- Accepted Proposal domain: exactly `"bulk"` / `"interactive"` (exact lowercase,
  case-sensitive per Java `equals`); omitted means bulk default (no wire option);
- `"interactive"` → neutral `i2p.streaming.maxWindowSize = "16"`;
  `"bulk"` → absent (same effective as omitted, distinct persisted string);
- All other types/values fail before allocation with no echo;
- Session-manager default (one effective window per SAM STREAM session
  generation); no per-socket override, no live mutation;
- Family overrides: plain client survives; HTTP/IRC/SOCKS forced-bulk removals
  and Streamr UDP ownership exclude all other families (M140, no inheritance);
- Core neutral bounds `2..=128`, default `128` (current effective cap), fail-safe
  to default on absent/malformed, clamp out-of-range (reference `<2` → `2`
  preserved; Emissary upper `128` preserves current hard cap vs reference `256`
  which would be inert beyond `128`).

Dedup/bulk-absent equivalence (explicit `"bulk"` vs omitted share the same
effective window and compatibility key) is the sole observable bounded
adaptation versus Java `else removed`; it preserves distinct persisted strings
for round-trip while keeping effective behavior identical and is recorded here.

### Pre/post matrix rows and counts

Pre-M143 (M142-qualified): `329/36/475` (matrix SHA before M143 edits is the
M142 SHA `e5fe8e2b28103bbceea3a92aa58c2e296c98337ce5c5df4625abb26f791a3576`).
Post-M143: `330/35/475` (matrix SHA
`cdd38453b6402df7e9612bb62c2b3afad42e2db3d25da239cb4896e164c5fadd`).
Recomputed from cells: `total=840 apply=330 blocked=35 na=475`; declared
counts match recomputation.

Changed cell (only this one; no other cell changed):

- `Profile:client` `blocked_primitive -> apply`.

Row now owns `completion_owner = "M143"`,
`current_or_planned_disposition = "apply_or_not_applicable"`, no
`blocked_primitive`/`blocking_milestone`, cells `[0]=apply` with M143 neutral
window notes; all other Profile cells remain `not_applicable` per M140.

### Neutral lower-layer contract

- Initialized before the manager becomes active (`SamSession::new_inner`
  parses `options` then `StreamManager::new_with_max_window`; no global or
  router-wide mutable);
- Immutable for the generation (`StreamManager.max_window_size` and
  `Stream.max_window_size` set once at construction, never mutated; restart
  creates a successor generation, stale streams retain only the old generation
  until teardown);
- Bounded to protocol-safe values (`2..=128`, default `128`, clamp; no
  per-packet logging, no unbounded tracking);
- Shared sessions have one compatible effective window or reject before
  allocation (I2PControl `additional_options_identity` distinguishes `16` vs
  absent; equal windows share, incompatible never silently share);
- Omitted preserves current upstream-compatible default (`128`, the current
  hard cap; `StreamConfig::default` 12 → 128 aligns the previously ignored
  default with the effective cap);
- Core contains no Proposal/I2PControl/TunnelManager/JsonRpc vocabulary
  (static guards + `m143_effective_window_reaches_stream_owner`).

### I2PControl validation and shared-session compatibility

- TunnelManager canonical validation already enforces string type for `Profile`
  (`validate_string`); malformed types fail at the control plane before backend
  allocation;
- `client::validate_raw_options` promotes only the retained `"Profile"` key to
  `SUPPORTED`; `build_session_options` validates exact `"bulk"`/`"interactive"`
  via `parse_profile_policy` so non-string, empty, whitespace, case-variant and
  unknown strings fail with `UnsupportedOption` before any runtime reservation;
  `config()` + `lifecycle` + `session_options` all run before supervisor
  `reserve`, so a bad edit never replaces the running generation;
- Non-target families (all 11 others) reject any supplied value via their
  existing `UnsupportedOption` raw gates plus `parse_profile_policy`
  family check before allocation (proven by
  `m143_non_client_families_keep_exact_rejection` for all families);
- `Get`/persistence round-trip preserves exact strings via raw config
  (deterministic round-trip covered by restart test with same-name definitions);
  `TunnelDefinition` Debug redacts raw values and never logs passwords;
- Compatibility: `omitted`/`bulk` (no wire option) share one key;
  `interactive`/`interactive` (both `16`) share one key; `bulk` vs
  `interactive` have different keys and never share (proven by
  `m143_shared_sessions_distinguish_effective_windows`).

### Observable behavior

- Core-only: `config::parse_stream_max_window_size` unit tests (omitted →
  default, `16` accepted, malformed fail-safe, out-of-range clamped,
  case-sensitive key); `StreamManager` window tests (default `128`,
  interactive `16`, clamp, distinguish); `Stream` growth test (same start,
  synthetic acked packets with `next_seq_nro` held valid, interactive caps at
  `16` while bulk grows beyond, both `≤128`);
- I2PControl: omitted → no wire option; `bulk` → no wire option (accepted);
  `interactive` → `i2p.streaming.maxWindowSize=16` (single sorted
  `additional_options` entry); invalid → fail (no echo);
- End-to-end: fake-SAM `SESSION CREATE` path exercised via
  `ClientTunnelBackend::start` with bulk then interactive (restart test);
  wire contains `i2p.streaming.maxWindowSize=16` only for interactive (asserted
  via `additional_options` wire strings + core owner static guards).

### Failure, cancellation, restart, and contention evidence

- `Profile` validation is synchronous/pre-allocation (pure `parse_profile_policy`
  + Yosemite `add_session_option` validation, no allocation/I/O);
- Session construction failure leaves no listener or committed replacement
  generation (`build_session_options` fails before `reserve`; `validate_start`
  pattern retained; `m143_invalid...` proves `inspect` stays `Stopped`);
- Cancellation during session setup cannot publish a partially configured
  manager (`CreationReservation` drop reopens the key; `m143_cancellation...`
  proves invalid start leaves `Stopped`; supervisor `reserve` only after
  validation, `stop_generation` drops the `Arc`);
- Restart creates a new generation with the new effective window; stale streams
  retain only the old generation until teardown (proven by stop → `Stopped` →
  start with different Profile → `Running` with different wire options; failed
  edit preserves last-known-good `Running`);
- No lock held across SAM connection, destination resolution or stream I/O
  (`select`/`note_result` short locks retained; core window is immutable data,
  no lock; `RuntimeMap` locks short-lived around map mutation only);
- Shared-session admission/rejection deterministic under concurrent starts
  (existing per-key `creating` + `notify` reservation, unchanged; different
  windows have different keys so no silent sharing race).

### Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** (neutral owners participate in no-std) |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1076 passed` (includes 5 new `config` parsing tests + 2 new manager window tests + 3 new stream window tests) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `816 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --test m143_streaming_profile --test m061_containment --test m062_dependency_containment --no-fail-fast` | **pass**: `48 passed (5 suites)`; matrix `330/35/475`; M143 promotion/residual/containment checks green (m143 alone: `14 passed`) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | **pass** (existing live-runtime smoke, no new tunnel formation claimed) |
| `git diff --check` | **pass** |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide (unchanged unrelated files); all ten M143-touched/added files are individually stable-rustfmt-clean after `rustfmt --edition 2021` on those files only; no unrelated normalization (one unintended `listener.rs` formatting reverted) |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **evidence only**: `1 error` — sole error is the known pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60`; all M143 files otherwise clean |

Full `cargo test -p emissary-cli --features i2pcontrol` (all suites) retains
nine expected historical failures independent of M143, all count/wording
drift (not behavioral regressions):

- `m126_requalification` (2), `m127_token_lifetime`, `m128_jsonrpc_batch`,
  `m129_nonloopback_tls`, `m130_post_corrective_requalification` assert the
  old `325`-era counts/wording;
- `m139_post_lifecycle_requalification::current_matrix_is_exhaustive_and_residuals_are_exact`
  asserts the M139-head `325/47/468` blocked set;
- `m140_residual_streaming_applicability::m140_matrix_reconciliation_is_exact_and_contained`
  asserts the M140-head `325/40/475` counts;
- `m141_unique_local_source::m141_matrix_promotes_exactly_two_unique_local_cells`
  and `m141_residual_inventory_subtracts_two_cells` assert the M141-head
  `327/38/475` counts;
- `m142_httpclient_sslproxies_jumplist::m142_matrix_promotes_exactly_two_httpclient_cells`
  and `m142_residual_inventory_subtracts_two_cells` assert the M142-head
  `329/36/475` counts.

M139/M140/M141/M142 closures are immutable history; M143 supersedes them for
counts while M139 remains the runtime/security qualification authority.
Recorded here as historical drift, not M143 regressions. No M143-required
suite fails.

### Production-path diff (bounded)

`git diff --name-only` plus untracked-file inspection proves only the amended
exact budget changed (see §2). No `emissary-util/**`, manifest, lockfile,
Yosemite, frontend, or workflow file changed or added.
`110-completion-ledger.toml` gains `post_m143` (1 cell). `061` gains the two
not-yet-allowed core files with owner evidence. Current unsupported values
keep their fail-before-allocation/no-effect behavior (Streamr datagram limits —
16-subscriber, 60s expiry, 1200-byte payload, 4095-byte transport buffer, 15s
refresh, bounded shutdown, loopback-only UDP — untouched; remote datagrams
never choose a local UDP destination).

## 4. Requirement-to-evidence matrix

| Plan requirement (§12) | Evidence | Result |
|---|---|---|
| M140-retained cells and no others are implemented | retained `Profile:client` ×1 verified via M140 map (not assumed); six N/A Profile + Streamr ConnectDelay rejection tests; matrix diff shows only `Profile:client` changed | **pass** |
| a neutral core streaming primitive is consumed by the actual runtime owner | `config::parse_stream_max_window_size` → `SamSession` handoff → `StreamManager::new_with_max_window` → `Stream` ceiling (`handle_acks` cap + `window_size` init); `new` default preserves bulk; static owner guards + core parsing/manager/stream tests | **pass** |
| effective behavior matches pinned Profile semantics and is deterministically observable | `"interactive"` → `16`, `"bulk"`/omitted → `128`; `GeneralHelper isInteractive (==16)` / `MAX_WINDOW_SIZE 128` / clamp `<2` preserved; core growth test proves `16` caps while `128` exceeds; I2PControl wire test proves `16` vs absent | **pass** |
| default behavior is unchanged when omitted | omitted emits no wire option; core default `128` equals current hard cap; `StreamConfig::default` 12 → 128 alignment; omitted test + manager default test | **pass** |
| shared-session compatibility is exact | equal windows share (same `additional_options`), incompatible never share (different lengths/keys); `additional_options_identity` exact; restart successor-only + last-known-good + cancellation no-partial | **pass** |
| core contains no Proposal/I2PControl terminology or policy | static no-`Proposal 170`/`I2PControl`/`TunnelManager`/`JsonRpc` guards on all four core files; neutral `i2p.streaming.maxWindowSize` only; M061/M062 exact-path (no glob) | **pass** |
| actual non-policy changed paths are exact-file authorized in M061/M062 | `061` adds `config.rs` + `stream/active.rs` with evidence; `m062` adds `is_authorized_m143_path` for six production files + planning/test/docs; `m061`/`m062` green (23 + containment) | **pass** |
| matrix/docs/registry match runtime evidence | M095 `330/35/475` recomputed; `m095`/`m105`/`m143` green; `AGENTS.md`, registry, README, both roadmaps, all three docs state `330/35/475` and partial support | **pass** |
| no high/medium streaming correctness or containment defect remains | invariant/failure/compatibility reviews below; only low historical-drift findings | **pass** |

## 5. Invariant review

- Exact pinned names/types/presence preserved; one `apply` promotion only,
  zero `not_applicable` moves, no new field/alias/status/method/tunnel-type;
- No fabricated/accept-inert support: every `apply` changes real behavior
  (interactive caps window at `16` vs bulk `128` via the actual connection
  owner); N/A cells retain affirmative non-applicability proof; Streamr
  datagram contract untouched (no invented window/delay timer);
- Unsupported values keep fail-before-allocation/no-effect behavior with no
  echo;
- No direct-clearnet, outproxy, trusted-peer, Streamr isolation, loopback
  confinement, bounded admission/tasks/timers, transactional lifecycle,
  last-known-good, lock discipline, secret/key/path redaction, LeaseSet
  crypto/scope, or feature/runtime isolation change beyond the additive
  neutral window + retained-family mapping;
- M061/M062 exact-path/dependency evidence amended only for M143
  planning/test/docs plus the six production files; no broad prefix waiver;
  Yosemite exact pin and optional `yosemite-i2pcontrol` ownership preserved;
- Full Proposal 170 support not claimed; partial-support wording retained
  everywhere.

## 6. Failure, cancellation, restart, and contention review

- `Profile` validation inherits the existing bounded Yosemite/session/stream
  timeout contract; timeout and bind/connect errors map to per-connection
  failures with no retry loop (single generation attempt per start);
- cancellation of the tunnel generation cancels/drains accepted tasks under
  existing bounded task-group semantics (untouched `run_generic_client`;
  no new task/timer/queue; core window is immutable data);
- failed validation closes nothing and records no state (proven by invalid
  tests with `Stopped` inspect and by malformed-option test that
  `build_session_options` fails while `inspect` stays `Stopped`, never stale
  `Running`);
- restart/edit failure preserves last-known-good: `build_session_options`
  gates `start`; supervisor `reserve` happens only after validation, so a bad
  edit never replaces the running generation (existing M120 order retained);
- no `Mutex`/`parking_lot` guard held across destination resolution, SAM/I2P
  stream connection, request forwarding, or response body I/O (window cap is
  lock-free immutable; `select`/`note_result` short-lived only).

## 7. Compatibility, migration, and security review

- No public API version, method, tunnel type, or action change; `Get` returns
  the exact persisted strings via raw config; no wire/schema change beyond the
  additive neutral `i2p.streaming.maxWindowSize=16` for interactive;
- No durable-store migration; existing definitions without the key default to
  bulk (`128`, backward compatible); explicit `"bulk"` shares the same
  effective window as omitted but round-trips distinctly; no dependency change;
- No migration or rollback operation required (additive string + data-plane
  cap);
- No secret material in selection/tests/docs/guards (Profile values are
  non-secret window labels; rejections never echo values; cache keys are
  window sizes only; raw config carries only the non-secret Profile string);
- No lock across I/O or crypto; no new network/filesystem/DNS/TLS/handshake
  path beyond the bounded I2P `connect_to` and loopback listener;
- Containment: preferred production ownership remains
  `emissary-cli/src/i2pcontrol/**` plus the four exact neutral core owners;
  deferred M144-M152 path budgets remain non-executable until registered (no
  registered successor exists after M143; M144 hard dep satisfied but still
  needs its own registration decision).

## 8. Documentation and operations

Machine authorities updated: `095-full-support-matrix.toml` (`330/35/475`,
SHA `cdd38453b6402df7e9612bb62c2b3afad42e2db3d25da239cb4896e164c5fadd`),
`110-completion-ledger.toml` (`post_m143` 1 cell), `061` (+2 core files),
`m095` expects `330/35/475` with M143 ownership/disposition checks, `m105`
subtracts M143 cell, `m062` gains `is_authorized_m143_path`, new `m143` guard
owns promotion/residual/validation/containment checks. Support docs
(`proposal-170-support.md`, `tunnel-manager.md`, `tunnel-backends.md`),
`AGENTS.md`, registry, implementation README, and both residual/full-support
roadmaps agree on `330/35/475`, partial support, M143 closure, no registered
successor, and the 35-cell residual inventory. Operational impact: none beyond
the additive retained-family option; absent values preserve prior behavior.

## 9. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | `cargo clippy --tests` reports pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60` | none on M143 files (clean) | record, separate corrective if needed |
| low | `m126`/`m127`/`m128`/`m129`/`m130`/`m139`/`m140`/`m141`/`m142` suites assert old `325`/`327`/`329`-era counts/wording and fail on the M143 `330/35/475` head | none on M143 behavior; those closures are immutable history (M139 remains runtime/security authority, M140 remains streaming-applicability authority, M141 remains unique-local authority, M142 remains SSL/Jump authority) | record as historical drift; future requalification may rebase those suites the way M139 rebased M126-M130 |
| low | Remaining 35 blocked cells (10 SigType, 15 LeaseSet crypto/lookup/auth, 4 UseSSL, 4 UseOutproxyPlugin, 2 MultiHoming) | partial Proposal 170 support remains; no M143 scope expansion | separate deferred M144-M152 clusters; no successor registered by this closure |
| info | Explicit `"bulk"` vs omitted share the same effective window (`128`) but round-trip distinctly; Java `else removed` collapses them identically on the wire | no security issue; effective behavior identical, persistence distinct | documented; no action |

No high/medium correctness defect remains.

## 10. Registry updates

Applied alongside this closure:

- `143-*.md` plan: Status `deferred` → `registered` via `M143-AMEND-01` →
  `closed as complete` with closure link;
- `plans/registry.md`: M143 → closed as complete (`330/35/475`); handoff M143
  closed with no registered successor; residual table `36` → `35`
  (Profile removed); execution chain updated; M144 constraint retained (hard
  dep satisfied but still needs its own registration decision);
- `plans/implementation/.../README.md`: handoff M143 closed, no registered
  successor; authority/M095/counts/residual inventory updated;
- residual + full-support roadmaps: M143 closed (`330/35/475`); M144 still
  deferred; counts/inventory/registration discipline updated;
- `110-completion-ledger.toml`: `post_m143` 1 cell added; matrix SHA recorded.

## Future-plan unblock determination

M143 unblocks no future plan for execution:

- **No plan registered**: M144 hard-depends on M143, and that hard dependency
  is now satisfied, but M144 still requires its own semantic freeze/registration
  decision (TLS trust/certificate contract per plan §46; file presence alone
  never authorizes production work). M143 does not satisfy that decision, so
  M144 remains deferred/unregistered. M145-M152 remain deferred/unregistered
  behind M144.
- No other future-plan status required a change beyond the M143-closed handoff.
  The sole next step is for a maintainer to freeze M144 semantics and
  explicitly register it; file presence alone never authorizes production work.

## Internal-only / read-only-upstream attestation

- External sources (Proposal 170 text at `https://i2p.net/proposals/170-i2pcontrol-expansion.txt`,
  Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  (`TunnelRequestParser.java` `getProfile`, `ClientTunnelCreator.java`
  `setTunnelManagementOptions`, `TunnelSupport.java` `16`), Java
  I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`
  (`I2PTunnelClient.java` no override, `I2PTunnelHTTPClientBase.java`/
  `I2PTunnelIRCClient.java`/`socks/I2PSOCKSTunnel.java` forced-bulk removals,
  `streamr/StreamrConsumer.java` UDP ownership, `TunnelController.java`
  default-bulk, `ui/GeneralHelper.java` `isInteractive`, `streaming/impl/`
  `ConnectionOptions.java`/`Connection.java` `MAX_WINDOW_SIZE 128`/clamp),
  Yosemite revision `59140a2277bf296928d2e8ce39a148182eeff044`, all fetched via
  read-only raw/API URLs for evidence) were accessed read-only for evidence;
- No upstream or third-party repository, issue, pull request, discussion,
  review, or maintainer channel was opened, drafted, updated, commented on, or
  contacted;
- No commit, branch, tag, patch, release, or artifact was pushed to any upstream
  remote before this closure commit. All writes are internal to
  `eggstack/emissary` on the current branch;
- No upstream review, approval, feedback, adoption, or merge was requested;
- No upstream contribution package, patch series, or submission checklist was
  prepared;
- Violation would invalidate this closure per `plans/003-planning-process.md`
  §11; no such violation occurred.

(End of file)
