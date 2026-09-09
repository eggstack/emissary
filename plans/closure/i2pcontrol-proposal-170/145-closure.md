# M145 Closure — LeaseSet Reply Bundling / MultiHoming Completion

Status: **closed as complete**

Date: `2026-09-08`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/145-leaseset-reply-bundling-multihoming-completion.md`
  (Status now `closed as complete` with closure link; registered for execution
  after M144 closure via `M145-AMEND-01` with the pinned session/bundling freeze
  and exact-file budget recorded in the plan).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## 1. Closure decision

M145 is closed as complete. It is an infrastructure + capability milestone
with a maximum 2-cell promotion budget, both cells promoted. The current
matrix is now exactly:

- `336 apply`;
- `29 blocked_primitive`;
- `475 not_applicable`;
- `840` total cells.

Promoted cells (exact observable neutral reply-bundling effects proven
end-to-end):

- `MultiHoming:httpserver` (omitted/`true` bundles, `false` suppresses
  `ExistingSession` updates via neutral `shouldBundleReplyInfo`);
- `MultiHoming:httpbidirserver` (same server-half owner; client half never
  inherits).

No high- or medium-severity correctness, privacy, or containment issue
remains. Full Proposal 170 support remains partial.

## 2. Reviewed commits and scope

Planning baseline (per M144 closure, M145 hard dependency):

- `e71abf0`.

Implementation starting point (HEAD before M145 edits, clean worktree
verified):

- `e71abf0`.

Implementation plus closure land in a single commit on the current branch
(see git log for the M145 closure commit). The reviewed production-behavior
baseline extends the lifecycle head `e4f217cb1459e26bf011da46b67fc2c83cd192b5`
(M095 `current_production_head`, retained; M145 adds the neutral
reply-bundling owner plus the two family mappings without touching
NetDB/transport/crypto/tunnel-pool/Cargo/Yosemite/frontend or workflows).

Changed paths (exact amended budget only):

- `emissary-core/src/destination/session/mod.rs` (neutral
  `bundle_reply_lease_set: bool`, default `true`, setter clears stale pending
  on disable; `register_lease_set`, `encrypt` ES path, and
  `publish_local_lease_set` gate on the flag; `NewSession` retains mandatory
  handshake bundling; `parse_bundle_reply_lease_set` fail-safe parser +
  `BUNDLE_REPLY_LEASE_SET_OPTION`; core policy tests);
- `emissary-core/src/destination/mod.rs` (already M061-allowed; narrow
  `set_bundle_reply_lease_set` forwarding bridge);
- `emissary-core/src/sam/session.rs` (already M061-allowed as `SamSession`
  owner; parse `shouldBundleReplyInfo` before activation and plumb to
  `Destination`);
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` (M145
  `parse_multihoming_policy`: two server families only, exact boolean,
  omitted/`true` → no wire option, `false` → generic
  `add_session_option("shouldBundleReplyInfo", "false")`; shared-session
  compatibility via existing `additional_options_identity`);
- `emissary-cli/src/i2pcontrol/backends/http_server.rs` (SUPPORTED
  `MultiHoming`, `multihoming_enabled` boolean validation before allocation);
- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` (SUPPORTED
  `MultiHoming`, server-half validation; client half never inherits);
- `emissary-cli/tests/m145_leaseset_reply_bundling.rs` (new; 10 integration
  guards);
- `emissary-cli/tests/m095_full_support_matrix.rs` (`336/29/475` counts,
  MultiHoming `M145` ownership/disposition);
- `emissary-cli/tests/m105_residual_option_audit.rs` (subtract M145 cells);
- `emissary-cli/tests/m062_dependency_containment.rs` (exact
  `is_authorized_m145_path` for production/test/docs paths);
- `emissary-cli/tests/m061_containment.rs` (no change; new core file covered
  by amended `061` TOML);
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
  (`336/29/475`, MultiHoming row `M145` / `apply_or_not_applicable`);
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml`
  (new `post_m145` 2-cell record);
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`
  (add `destination/session/mod.rs` with owner evidence);
- `plans/implementation/i2pcontrol-proposal-170/145-leaseset-reply-bundling-multihoming-completion.md`
  (Status `deferred` → `registered` via `M145-AMEND-01` → `closed as complete`);
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `AGENTS.md`;
- `docs/i2pcontrol/proposal-170-support.md`, `docs/i2pcontrol/tunnel-manager.md`,
  `docs/i2pcontrol/tunnel-backends.md`;
- new `plans/closure/i2pcontrol-proposal-170/145-closure.md` (this file).

`git diff --name-only` proves no `emissary-util/**`, manifest, lockfile,
Yosemite, frontend, or workflow path changed. `git diff --check` passes.

## 3. Requirement evidence

### Pinned reference (all accessed read-only)

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (Proposal lists `MultiHoming` boolean server option without a per-family
  value partition; family applicability frozen by M095/M131);
- Java I2P snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`:
  `OutboundClientMessageOneShotJob.java` `BUNDLE_REPLY_LEASESET =
  "shouldBundleReplyInfo"`, defaults to true (`allow == null ||
  Boolean.parseBoolean(allow)`); per-message `SendMessageOptions.getSendLeaseSet`
  ANDs with the session option; `getReplyLeaseSet(false)` sends only when not
  already acked (cache date check); `replyLeaseSet != null` forces `wantACK`;
  no valid LS → punt (send without fabrication); unpublished still bundles
  directly for reply possibility; HTTP servers may set `false` to save
  bandwidth when the far end already has the LeaseSet;
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  (pass-through boolean to the tunnel controller; no SAM mapping);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (generic `add_session_option` path carries `shouldBundleReplyInfo`; no M145
  dependency change; `shouldBundleReplyInfo` is not reserved).

### Exact reference freeze applied before coding (M145-AMEND-01)

- Accepted Proposal domain: exact boolean `true`/`false`; omitted means bundle
  default (no wire option, preserves current always-bundle);
- `true`/omitted → no wire option (same effective, distinct persisted strings
  for round-trip, shared session; per M143 bulk/omitted precedent);
- `false` → neutral `shouldBundleReplyInfo = "false"` (suppress
  `ExistingSession` updates);
- Only `httpserver`/`httpbidirserver` applicable; all ten other families reject
  any presence (affirmative HTTP-server-presentation-role evidence);
- All other types/values fail before allocation with no echo;
- Session policy (one effective value per SAM STREAM session generation); no
  per-message override (no Emissary SAM equivalent layer; documented, no
  invented flag);
- Core neutral bounds: fail-safe to enabled on absent/malformed; explicit
  `false` (case-insensitive per `Boolean.parseBoolean`) disables;
- `NewSession` retains mandatory bundling for liveness (Emissary inbound
  rejects `NewSession` without `DatabaseStore`; documented bounded
  fail-closed adaptation); the policy gates only `ExistingSession` updates;
- Throttle is ack-gated via existing `ActiveSession.lease_set`
  pending-until-ack plus 5s publish timer (bounded per-session, one pending
  `Option`, no unbounded history);
- No fabricated, expired, unpublished-via-NetDB, or nonexistent leases bundled;
  unpublished still bundles directly to the peer (not via NetDB); encrypted/
  blinded not yet implemented (M149-M151) → no downgrade; no new NetDB lookup
  to fabricate sender info; bounded message growth via existing garlic builder.

### Pre/post matrix rows and counts

Pre-M145 (M144-qualified): `334/31/475` (matrix SHA before M145 edits is the
M144 SHA `af01bc0efb4723f471e442e4cdf5dd8e70f6a9473ac5fe0be022ae9cadfa34c9`).
Post-M145: `336/29/475` (matrix SHA
`81851be26af3ed1e4d3f0526d914aaae3d061224d8b78c5cc985cd586d84b20d`).
Recomputed from cells: `total=840 apply=336 blocked=29 na=475`; declared
counts match recomputation.

Changed cells (only these two; no other cell changed):

- `MultiHoming:httpserver` `blocked_primitive -> apply`;
- `MultiHoming:httpbidirserver` `blocked_primitive -> apply`.

Row now owns `completion_owner = "M145"`,
`current_or_planned_disposition = "apply_or_not_applicable"`, no
`blocked_primitive`/`blocking_milestone`, cells `[8]=[9]=apply` with M145
neutral-policy notes; all other MultiHoming cells remain `not_applicable`.

### Neutral lower-layer contract

- Initialized before the destination becomes active (`SamSession::new_inner`
  parses `options` then `Destination::set_bundle_reply_lease_set` before
  `publish_lease_set`; no global or router-wide mutable);
- Generation-local (one flag per `SessionManager`/`Destination` generation;
  `set_bundle_reply_lease_set(false)` clears stale pending; restart creates a
  successor generation, stale sessions retain only the old generation until
  teardown);
- Bounded (one pending `Option` per active session, one 5s timer per updated
  session; no per-packet logging, no unbounded tracking);
- Shared sessions have one compatible effective policy or reject before
  allocation (I2PControl `additional_options_identity` distinguishes `false`
  vs absent; omitted/`true` share, `false` never silently shares);
- Omitted preserves current upstream-compatible default (enabled, always
  bundle; `NewSession` mandatory preserved);
- Core contains no Proposal/I2PControl/TunnelManager/JsonRpc/`MultiHoming`
  vocabulary (static guards + `m145_exact_containment_holds_for_touched_owners`).

### I2PControl validation and shared-session compatibility

- TunnelManager canonical validation already enforces boolean type for
  `MultiHoming` (`validate_boolean`); malformed types fail at the control plane
  before backend allocation;
- `http_server::multihoming_enabled` + `http_bidir` server-half validation
  promote only the two target `"MultiHoming"` keys to SUPPORTED;
  `build_session_options` validates exact boolean via `parse_multihoming_policy`
  so non-boolean, null, numeric, and string values fail with `UnsupportedOption`
  before any runtime reservation; `config()` + `lifecycle` + `session_options`
  all run before supervisor `reserve`, so a bad edit never replaces the running
  generation;
- Non-target families (all ten others) reject any supplied value via their
  existing `UnsupportedOption` raw gates plus `parse_multihoming_policy`
  family check before allocation (proven by
  `m145_non_server_families_keep_exact_rejection` for all families);
- `Get`/persistence round-trip preserves exact booleans via raw config
  (omitted vs `true` vs `false` distinct persisted; omitted/`true` share
  effective wire-absent; `false` carries wire `"false"`);
  `TunnelDefinition` Debug redacts raw values and never logs secrets;
- Compatibility: omitted/`true` (no wire option) share one key; `false`
  (`shouldBundleReplyInfo=false`) has a different key and never shares (proven
  by `m145_shared_sessions_distinguish_effective_policies`).

### Observable behavior

- Core-only: `parse_bundle_reply_lease_set` unit tests (omitted → enabled,
  `true`/`TRUE` → enabled, `false`/`FALSE` → disabled, malformed fail-safe to
  enabled, exact option key); `SessionManager` policy tests (default enabled,
  disable clears pending, disabled `register_lease_set` records no timer,
  disabled `encrypt` sends without `DatabaseStore`/ack-request and decrypts
  cleanly);
- I2PControl: omitted → no wire option; `true` → no wire option (accepted,
  same effective); `false` → `shouldBundleReplyInfo=false` (single sorted
  `additional_options` entry); invalid → fail (no echo);
- End-to-end: fake-SAM `SESSION CREATE` path exercised via
  `HttpServerTunnelBackend::validate_start` with enabled then disabled
  (restart test pattern); wire contains `shouldBundleReplyInfo=false` only for
  disabled (asserted via `additional_options` wire strings + core owner static
  guards); core message-decode test proves disabled updates carry `Data` with
  no `DatabaseStore` clove.

### Failure, cancellation, restart, and contention evidence

- `MultiHoming` validation is synchronous/pre-allocation (pure
  `parse_multihoming_policy` + Yosemite `add_session_option` validation, no
  allocation/I/O);
- Session construction failure leaves no listener or committed replacement
  generation (`build_session_options` fails before `reserve`; `validate_start`
  pattern retained; `m145_invalid_fails_before_allocation_without_echo` proves
  `inspect` stays `Stopped`);
- Cancellation during session setup cannot publish a partially configured
  manager (`CreationReservation` drop reopens the key; supervisor `reserve`
  only after validation, `stop_generation` drops the `Arc`);
- Restart creates a new generation with the new effective policy; stale
  sessions retain only the old generation until teardown (proven by stop →
  `Stopped` → start with different policy pattern; failed edit preserves
  last-known-good `Running`);
- No lock held across SAM connection, destination resolution or garlic I/O
  (`select`/`note_result` short locks retained; core flag is immutable-after-set
  data, no lock; `RuntimeMap` locks short-lived around map mutation only);
- Shared-session admission/rejection deterministic under concurrent starts
  (existing per-key `creating` + `notify` reservation, unchanged; different
  policies have different keys so no silent sharing race).

### Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1080 passed` (includes 4 new bundling-policy tests) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `821 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --test m145_leaseset_reply_bundling --test m061_containment --test m062_dependency_containment --no-fail-fast` | **pass**: matrix `336/29/475`; M145 promotion/residual/containment checks green (m145 alone: `10 passed`) |
| `git diff --check` | **pass** |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide (unchanged unrelated files); all M145-touched files are individually stable-rustfmt-clean; no unrelated normalization |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **evidence only**: sole error is the known pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60`; all M145 files otherwise clean |

Full `cargo test -p emissary-cli --features i2pcontrol` (all suites) retains
historical failures independent of M145, all count/wording drift (not
behavioral regressions): `m126`/`m127`/`m128`/`m129`/`m130`/`m139`/`m140`/
`m141`/`m142`/`m143`/`m144` suites assert old `325`/`327`/`329`/`330`/`334`-era
counts/wording. M139/M140/M141/M142/M143/M144 closures are immutable history;
M145 supersedes them for counts while M139 remains the runtime/security
qualification authority. Recorded here as historical drift, not M145
regressions. No M145-required suite fails.

### Production-path diff (bounded)

`git diff --name-only` plus untracked-file inspection proves only the amended
exact budget changed (see §2). No `emissary-util/**`, manifest, lockfile,
Yosemite, frontend, or workflow file changed or added.
`110-completion-ledger.toml` gains `post_m145` (2 cells). `061` gains the one
not-yet-allowed core file with owner evidence. Current unsupported values
keep their fail-before-allocation/no-effect behavior (Streamr datagram limits —
16-subscriber, 60s expiry, 1200-byte payload, 4095-byte transport buffer, 15s
refresh, bounded shutdown, loopback-only UDP — untouched; remote datagrams
never choose a local UDP destination).

## 4. Requirement-to-evidence matrix

| Plan requirement (§13) | Evidence | Result |
|---|---|---|
| exact pinned `MultiHoming`/`shouldBundleReplyInfo` semantics are frozen | §3 pinned freeze (default true, direct mapping, per-message N/A, ack-gated throttle, garlic DatabaseStore in NS/ES, unpublished-direct, no-LS punt, expiry via LeaseSetManager) | **pass** |
| exact neutral core files were authorized before implementation | `M145-AMEND-01` names `destination/session/mod.rs` (+ already-allowed `destination/mod.rs`, `sam/session.rs`); `061` adds the one file with evidence; `m062` adds `is_authorized_m145_path`; `m061`/`m062` green | **pass** |
| actual outbound messages observably include/exclude truthful sender LeaseSet info | core `disabled_policy_sends_updates_without_database_store` (enabled pends + timer, disabled clears + no timer + encrypt without DatabaseStore/ack, decrypt clean); I2PControl wire test proves `false` vs omitted/`true` | **pass** |
| destination/privacy/freshness/message-size invariants are proven | wrong-destination isolation via existing session maps (policy is per-`SessionManager`, never cross-destination); no fabrication (punt when no LS); freshness via `LeaseSetManager` (only current valid LS); size via existing garlic builder; unpublished never published; encrypted/blinded N/A (no downgrade) | **pass** |
| core remains Proposal-free | static no-`Proposal 170`/`I2PControl`/`TunnelManager`/`JsonRpc`/`MultiHoming` guards on all three core files; neutral `shouldBundleReplyInfo` only; M061/M062 exact-path (no glob) | **pass** |
| no broad M061 prefix allowance was introduced | one exact file added; no `crypto/`/`i2np/`/`netdb/`/`lease_set/` prefix waiver; prohibited prefixes still block | **pass** |
| matrix/docs/registry match runtime evidence | M095 `336/29/475` recomputed; `m095`/`m105`/`m145` green; `AGENTS.md`, registry, README, both roadmaps, all three docs state `336/29/475` and partial support | **pass** |
| no medium/high privacy, routing or LeaseSet truthfulness defect remains | invariant/failure/compatibility reviews below; only low historical-drift findings | **pass** |

## 5. Invariant review

- Exact pinned names/types/presence preserved; two `apply` promotions only,
  zero `not_applicable` moves, no new field/alias/status/method/tunnel-type;
- No fabricated/accept-inert support: every `apply` changes real behavior
  (disabled suppresses `ExistingSession` DatabaseStore updates via the actual
  bundling owner; `NewSession` handshake retained for liveness with documented
  adaptation); N/A cells retain affirmative non-applicability proof; Streamr
  datagram contract untouched;
- Unsupported values keep fail-before-allocation/no-effect behavior with no
  echo;
- No direct-clearnet, outproxy, trusted-peer, Streamr isolation, loopback
  confinement, bounded admission/tasks/timers, transactional lifecycle,
  last-known-good, lock discipline, secret/key/path redaction, LeaseSet
  crypto/scope, or feature/runtime isolation change beyond the additive
  neutral flag + two-family mapping;
- M061/M062 exact-path/dependency evidence amended only for M145
  planning/test/docs plus the six production files (three core incl. two
  already-allowed + three I2PControl); no broad prefix waiver; Yosemite exact
  pin and optional `yosemite-i2pcontrol` ownership preserved;
- Full Proposal 170 support not claimed; partial-support wording retained
  everywhere.

## 6. Failure, cancellation, restart, and contention review

- `MultiHoming` validation inherits the existing bounded Yosemite/session
  timeout contract; timeout and bind/connect errors map to per-connection
  failures with no retry loop (single generation attempt per start);
- cancellation of the tunnel generation cancels/drains accepted tasks under
  existing bounded task-group semantics (no new task/timer/queue; core flag is
  immutable data; one 5s publish timer per updated session, pre-existing);
- failed validation closes nothing and records no state (proven by invalid
  tests with `Stopped` inspect and by malformed-option test that
  `build_session_options` fails while `inspect` stays `Stopped`, never stale
  `Running`);
- restart/edit failure preserves last-known-good: `build_session_options`
  gates `start`; supervisor `reserve` happens only after validation, so a bad
  edit never replaces the running generation (existing M120 order retained);
- no `Mutex`/`parking_lot` guard held across destination resolution, SAM/I2P
  stream connection, TLS handshake, request forwarding, or response body I/O
  (flag is lock-free immutable; `select`/`note_result` short-lived only).

## 7. Compatibility, migration, and security review

- No public API version, method, tunnel type, or action change; `Get` returns
  the exact persisted booleans via raw config; no wire/schema change beyond
  the additive neutral `shouldBundleReplyInfo=false` for disabled;
- No durable-store migration; existing definitions without the key default to
  bundle (`true`, backward compatible); explicit `"true"` shares the same
  effective policy as omitted but round-trips distinctly; no dependency change;
- No migration or rollback operation required (additive boolean + data-plane
  gate);
- No secret material in selection/tests/docs/guards (MultiHoming values are
  non-secret booleans; rejections never echo values; cache keys are policy
  strings only; raw config carries only the non-secret boolean);
- No lock across I/O or crypto; no new network/filesystem/DNS/TLS/handshake
  path beyond the bounded I2P `connect_to` and loopback listener;
- Containment: preferred production ownership remains
  `emissary-cli/src/i2pcontrol/**` plus the three exact neutral core owners;
  deferred M146-M152 path budgets remain non-executable until registered (no
  registered successor exists after M145; M146 hard dep satisfied but still
  needs its own registration decision).

## 8. Documentation and operations

Machine authorities updated: `095-full-support-matrix.toml` (`336/29/475`,
SHA `81851be26af3ed1e4d3f0526d914aaae3d061224d8b78c5cc985cd586d84b20d`),
`110-completion-ledger.toml` (`post_m145` 2 cells), `061` (+1 core file),
`m095` expects `336/29/475` with M145 ownership/disposition checks, `m105`
subtracts M145 cells, `m062` gains `is_authorized_m145_path`, new `m145` guard
owns promotion/residual/validation/containment checks. Support docs
(`proposal-170-support.md`, `tunnel-manager.md`, `tunnel-backends.md`),
`AGENTS.md`, registry, implementation README, and both residual/full-support
roadmaps agree on `336/29/475`, partial support, M145 closure, no registered
successor, and the 29-cell residual inventory. Operational impact: none beyond
the additive two-family option; absent values preserve prior behavior.

## 9. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | `cargo clippy --tests` reports pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60` | none on M145 files (clean) | record, separate corrective if needed |
| low | `m126`/`m127`/`m128`/`m129`/`m130`/`m139`/`m140`/`m141`/`m142`/`m143`/`m144` suites assert old `325`/`327`/`329`/`330`/`334`-era counts/wording and fail on the M145 `336/29/475` head | none on M145 behavior; those closures are immutable history (M139 remains runtime/security authority, M140 remains streaming-applicability authority, M141 remains unique-local authority, M142 remains SSL/Jump authority, M143 remains Profile authority, M144 remains UseSSL authority) | record as historical drift; future requalification may rebase those suites the way M139 rebased M126-M130 |
| low | Remaining 29 blocked cells (10 SigType, 15 LeaseSet crypto/lookup/auth, 4 UseOutproxyPlugin) | partial Proposal 170 support remains; no M145 scope expansion | separate deferred M146-M152 clusters; only M146 is next (still unregistered) |
| info | `NewSession` retains mandatory LeaseSet bundling even when disabled (Emissary inbound rejects `NewSession` without `DatabaseStore`); explicit `true` vs omitted share the same effective wire-absent policy but round-trip distinctly | no security issue; fail-closed toward connectivity for the handshake, bandwidth saving preserved for updates; effective behavior identical, persistence distinct | documented; no action |
| info | Per-message `SendMessageOptions.getSendLeaseSet` has no Emissary SAM equivalent; session policy alone controls | no security issue; session-level suppression is strictly coarser than per-message (disabling suppresses all updates, enabling preserves default) | documented; no action |

No high/medium correctness defect remains.

## 10. Registry updates

Applied alongside this closure:

- `145-*.md` plan: Status `deferred` → `registered` via `M145-AMEND-01` →
  `closed as complete` with closure link;
- `plans/registry.md`: M145 → closed as complete (`336/29/475`); handoff M145
  closed with no registered successor; residual table `31` → `29`
  (MultiHoming removed); execution chain updated; M146 constraint retained (hard
  dep satisfied but still needs its own registration decision);
- `plans/implementation/.../README.md`: handoff M145 closed, no registered
  successor; authority/M095/counts/residual inventory updated;
- residual + full-support roadmaps: M145 closed (`336/29/475`); M146 still
  deferred; counts/inventory/registration discipline updated;
- `110-completion-ledger.toml`: `post_m145` 2 cells added; matrix SHA recorded.

## Future-plan unblock determination

M145 unblocks no future plan for execution:

- **No plan registered**: M146 hard-depends on M145, and that hard dependency
  is now satisfied, but M146 still requires its own provider/registration
  decision (real bounded I2P-routed outproxy provider, per its plan and the
  registry constraint) plus an explicit registration decision before
  execution. M145 does not satisfy that decision, so M146 remains
  deferred/unregistered. M147-M152 remain deferred/unregistered behind M146.
- No other future-plan status required a change beyond the M145-closed handoff.
  The sole next step is for a maintainer to prove an M146 provider and
  explicitly register it; file presence alone never authorizes production
  work.

## Internal-only / read-only-upstream attestation

- External sources (Proposal 170 text at `https://i2p.net/proposals/170-i2pcontrol-expansion.txt`,
  Java I2P snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`
  (`OutboundClientMessageOneShotJob.java` `BUNDLE_REPLY_LEASESET`/ack-gated
  bundling, `SendMessageOptions` per-message override, `OutboundCache`
  lease/ack caches), Java I2PControl Proposal-170 head
  `45bb593000408071dd376b78848fdc246dccd964`,
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
