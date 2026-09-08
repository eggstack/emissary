# M140 Closure — Residual Streaming Applicability Re-Freeze

Status: **closed as complete**

Date: `2026-09-08`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/140-residual-streaming-applicability-refreeze.md`
  (Status now `closed as complete` with closure link).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## 1. Closure decision

M140 is closed as complete. It is an invariant/qualification/matrix-truthfulness
milestone with zero `apply` promotion budget and no production behavior change.
Seven currently blocked cells are reclassified to `not_applicable` with
affirmative pinned actual-runtime evidence; one cell is retained as
`blocked_primitive`. The current matrix is now exactly:

- `325 apply`;
- `40 blocked_primitive`;
- `475 not_applicable`;
- `840` total cells.

No high- or medium-severity correctness or security defect was found. No
corrective implementation plan is required by M140. Full Proposal 170 support
remains partial.

## 2. Reviewed commits and scope

Planning baseline (per M140 plan):

- `7b51da725e83c3ec4a7b806a4e242730b7cb93d7`.

Implementation starting point (clean worktree, verified before edits):

- `a02d269d35b312c33061a3d797dabd9961cf295a`.

The M095 pre-M140 SHA-256 at that head (identical to the M139-qualified head) is:

- `c06236cb3eb014cbe7f04b3b065155da85720e66c6c7a106bdb7624405d5ff9d`.

Implementation plus closure land in a single commit on the current branch
(planning/test/docs only; see §3). The reviewed production-behavior baseline is
unchanged:

- lifecycle implementation head `e4f217cb1459e26bf011da46b67fc2c83cd192b5`
  (M095 `current_production_head`, untouched).

Changed paths (planning/test/docs only; no production Rust, Cargo, lockfile,
Yosemite, router, transport, NetDB, crypto, frontend, or hosted-CI path):

- `AGENTS.md`;
- `docs/i2pcontrol/proposal-170-support.md`, `docs/i2pcontrol/tunnel-manager.md`;
- `emissary-cli/tests/m095_full_support_matrix.rs`,
  `emissary-cli/tests/m105_residual_option_audit.rs`,
  `emissary-cli/tests/m062_dependency_containment.rs` (exact M140 planning-path
  bookkeeping only), and new
  `emissary-cli/tests/m140_residual_streaming_applicability.rs`;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`;
- new `plans/implementation/i2pcontrol-proposal-170/140-residual-streaming-applicability-map.toml`;
- `plans/implementation/i2pcontrol-proposal-170/140-residual-streaming-applicability-refreeze.md`
  (Status `registered` → `closed as complete`);
- `plans/implementation/i2pcontrol-proposal-170/141-http-unique-local-source-address-completion.md`
  (Status `deferred` → `registered / dependency-ready`; hard dependency M140
  satisfied, readiness seams verified, no amendment required);
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- new `plans/closure/i2pcontrol-proposal-170/140-closure.md` (this file).

`git diff --name-only` proves no `emissary-cli/src/**`, `emissary-core/**`,
`emissary-util/**`, manifest, lockfile, Yosemite, or workflow path changed.

## 3. Requirement evidence

### Eight-row evidence map (cell-complete)

`140-residual-streaming-applicability-map.toml` contains exactly eight records,
each with canonical option/family, starting disposition, Proposal
type/applicability evidence, Java I2PControl parser/creator behavior, Java
actual tunnel class and constructor hierarchy, exact property emitted, exact
runtime consumer, constructor/runtime override, Yosemite wire capability
(separated), current Emissary data-plane type, final disposition, rationale and
source locations, and an explicit future-primitive flag. The artifact records
source SHAs and pre/post counts. Its SHA-256 is:

- `75351c450d203103883de63d3d509e0df1d5612829a64c86a18eb813b88f99d7`.

### Pinned source SHAs and locations (all accessed read-only)

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (verified via fetch sha256sum; client management options list names
  `ConnectDelay`/`Profile` without a per-family partition);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`:
  `TunnelRequestParser.java` `getProfile`/`getConnectDelay`,
  `ClientTunnelCreator.java` `setTunnelManagementOptions` (ConnectDelay
  boolean → `option.i2p.streaming.connectDelay` 500-or-0; Profile interactive
  → `option.i2p.streaming.maxWindowSize` 16 else removed),
  `TunnelSupport.java` `PROP_DELAY_DEFAULT_ACTIVE` 500 and
  `PROP_DEFAULT_STREAMING_MAX_WINDOW_SIZE` 16;
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`:
  `I2PTunnelHTTPClientBase.java` both constructors force bulk
  (`setProperty(connectDelay,200)` + `remove(maxWindowSize)`);
  `I2PTunnelConnectClient.java` extends the HTTP base with no restore;
  `I2PTunnelIRCClient.java` independently forces the same bulk lines;
  `socks/I2PSOCKSTunnel.java` forces the same bulk lines;
  `socks/I2PSOCKSIRCTunnel.java` extends SOCKS with IRC filtering only;
  `I2PTunnelClient.java` performs no streaming override (plain-client survival);
  `streamr/StreamrConsumer.java` extends `udpTunnel/I2PTunnelUDPClientBase.java`
  (datagram `I2PSession` via `I2PClient.createSession`, `I2PSource`/`I2PSink`,
  `_session.connect`, `UDPSink` + `Pinger`; no `I2PSocket`/streaming
  `Connection`);
  `TunnelController.java` defaults absent Profile to bulk and passes
  `option.*` through `setSessionOptions`;
  `ui/GeneralHelper.java` `isInteractive` (`maxWindowSize == 16`) and
  `shouldDelayConnect` (`connectDelay > 0`);
  `streaming/impl/ConnectionOptions.java` (`PROP_CONNECT_DELAY`,
  `PROP_MAX_WINDOW_SIZE`, `PROP_PROFILE`; `setProfile` documented
  `Warning: unused` while `maxWindowSize`/`connectDelay` are the consumed
  knobs);
  `streaming/impl/Connection.java` (`MAX_WINDOW_SIZE` 128; window/inboundBuffer
  from `getMaxWindowSize`);
  `streaming/impl/ConnectionManager.java` `Connect()` conDelay gate;
  `streaming/impl/SchedulerPreconnect.java` SYN delay via `getConnectDelay`;
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`:
  `src/options.rs` `SessionOptions.additional_options` plus
  `add_session_option` validated generic wire path (transport-only, explicitly
  separated from runtime consumption); `src/proto/session.rs` SAM wire.

### Pre/post matrix row diff and counts

Pre-M140 (M139-qualified): `325/47/468` (SHA `c06236cb…5ff9d`).
Post-M140: `325/40/475` (SHA `dd77613fb302b8bd04f42c8d1fe702b6c8c8920307776e40c4a8d8b1e8e0c44d`).
Recomputed from cells: `total=840 apply=325 blocked=40 na=475`; declared counts
match recomputation.

Reclassified `blocked_primitive -> not_applicable` (7 cells, every one among
the eight M140 candidates; no other cell changed):

- `Profile:httpclient` (HTTP-base forced-bulk removal);
- `Profile:connectclient` (inherited HTTP-base removal);
- `Profile:ircclient` (independent bulk override);
- `Profile:socks` (SOCKS bulk override);
- `Profile:socksirc` (inherited SOCKS bulk);
- `Profile:streamrclient` (UDP/datagram ownership, no streaming socket);
- `ConnectDelay:streamrclient` (no streaming SYN-delay event/consumer for
  Streamr despite generic 500-or-0 emission; no UDP start/pinger/send/receive
  effect; no invented datagram timer).

Retained `blocked_primitive` (1 cell):

- `Profile:client` (plain `I2PTunnelClient` performs no override; interactive
  `maxWindowSize` survives to the streaming manager; contemporary
  `setProfile-unused` vs `maxWindowSize`-implemented distinction recorded; the
  Proposal cell follows observable maxWindowSize semantics).

### Proof of zero apply promotions

`apply` count is unchanged from 325 (pre and post). The M140 map records
`apply_promotions = 0`; the M095 `current_or_planned_disposition` for
`ConnectDelay` is now `apply_or_not_applicable` (six TCP apply, six N/A) and
for `Profile` remains `blocked_primitive_or_not_applicable` (one blocked, eleven
N/A). The focused M140 guard asserts no cell moved to `apply` and the blocked
set is a strict subset of the pre-M140 blocked set.

### Production-path diff (no runtime changes)

`git diff --name-only` plus untracked-file inspection proves zero production
paths: no `emissary-cli/src/**`, `emissary-core/**`, `emissary-util/**`,
`Cargo.toml`, `Cargo.lock`, Yosemite, frontend, or workflow file changed or
added. `git diff --check` passes. Current unsupported values keep their
pre-M140 fail-before-allocation/no-effect behavior (TCP allowlist rejections
unchanged; Streamr datagram limits — 16-subscriber, 60s expiry, 1200-byte
payload, 4095-byte buffer, 15s refresh, bounded shutdown, loopback-only UDP —
untouched; remote datagrams never choose a local UDP destination).

### Verification outcomes

| Command group | Result |
|---|---|
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --test m140_residual_streaming_applicability --test m061_containment --test m062_dependency_containment --no-fail-fast` | **pass**: `36 passed (5 suites)`; matrix `325/40/475`; M140 eight-record/N-A/retained/containment checks green |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `git diff --check` | **pass** |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide (unchanged unrelated files); all four M140-touched/added test files are individually stable-rustfmt-clean after `rustfmt --edition 2021` on those files only; no unrelated normalization |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --tests` | **evidence only**: `0 errors, 1 warning` — sole warning is the known pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60`; all M140 files clean |
| Mechanical checks (eight records; every changed disposition among the eight; `apply` unchanged at 325; no production path) | **pass** (see above) |

Full `cargo test -p emissary-cli --features i2pcontrol` (all suites) retains one
expected historical failure independent of M140: `m139_post_lifecycle_requalification::current_matrix_is_exhaustive_and_residuals_are_exact`
still asserts the M139-head `325/47/468` blocked set as current. M139 closure is
immutable history; M140 supersedes it for residual counts while M139 remains the
runtime/security qualification authority. Recorded here as historical drift, not
an M140 regression. No M140-required suite fails.

### Historical-record immutability review

`git diff --name-only` shows no modified file under
`plans/closure/i2pcontrol-proposal-170/` for any pre-M140 closure. M130, M131,
M132, M133, M134, M135, M136, M137, and M139 closure files are untouched. The
only closure addition is this new `140-closure.md`. M131 remains the historical
residual authority except where M140 explicitly supersedes seven cells; M139
remains the current runtime/security authority.

### Exact M141 readiness decision

**Decision: M141 is dependency-ready and is hereby registered as the sole next
handoff.** Its hard dependency (M140 closed with a reconciled `325/40/475`
baseline) is satisfied. Its plan readiness seams were verified present at M140
closure with no amendment required:

- structurally validated `TrustedPeerIdentity` before local-target connect
  (`backends/http_server.rs`, `backends/filters/http.rs`);
- canonical 32-byte `TrustedPeerIdentity::canonical_id()`
  (`backends/runtime/peer_identity_impl.rs`);
- `httpserver`/`httpbidirserver` sharing the existing HTTP accepted-handler
  path (`backends/http_bidir.rs` uses `http_server::make_accepted_handler`);
- literal-loopback-only target normalization
  (`backends/http_server.rs::normalize_loopback_target`, reused by
  `http_bidir.rs`/`irc_server.rs`).

M141 plan Status was flipped `deferred / unregistered` →
`registered / dependency-ready` alongside this closure. M142-M152 remain
deferred/unregistered. M143's required pre-registration amendment input is now
frozen (retained set `Profile:client` × 1 at `325/40/475`), but M143 stays
deferred until M142 closes; its exact neutral streaming-file amendment is still
required before its own registration. No other future-plan status required a
change.

## 4. Requirement-to-evidence matrix

| Plan requirement (§12) | Evidence | Result |
|---|---|---|
| 1. all eight cells have pinned actual-runtime applicability evidence | M140 map eight records, each with Proposal/parser/class/hierarchy/property/consumer/override/wire/data-plane/rationale/sources | **pass** |
| 2. no cell promoted to `apply` | `apply` 325 → 325; map `apply_promotions` 0; M095/M140 guards assert no `apply` gain and blocked-subset containment | **pass** |
| 3. every N/A change meets the affirmative-evidence rule | each of seven N/A rows cites the actual family constructor/runtime path (forced-bulk removals or UDP-ownership exclusion) plus exact override; independent N/A review recorded in map | **pass** |
| 4. M095, current-head tests and active docs agree on final counts | M095 `325/40/475` recomputed; `m095`/`m105`/`m140`/`m061`/`m062` green (36 tests); `AGENTS.md`, registry, README, both roadmaps, `proposal-170-support.md`, `tunnel-manager.md` state `325/40/475` and partial support | **pass** |
| 5. retained Profile implementation set is explicit | map `retained_cells = [Profile:client]`; `counts`/`evidence_review` freeze it for M143; M095 Profile note marks M140-retained; registry/roadmaps/README record the single-cell M143 target | **pass** |
| 6. production and fail-before-allocation behavior unchanged | production-path diff empty; `git diff --check` clean; TCP allowlist rejections and Streamr datagram limits unchanged | **pass** |
| 7. M061/M062 containment at least as strict | `m061`/`m062` green; `m062` gains exact `is_authorized_m140_path` (planning/test/docs only, zero production paths); prohibited patterns still enforced | **pass** |
| 8. M141 identified as next successor only after closure | §3 M141 readiness decision above; M141 registered, M142-M152 deferred | **pass** |

## 5. Invariant review

- Exact pinned names/types/presence preserved; seven N/A moves only, zero
  promotions, no new field/alias/status/method/tunnel-type;
- No fabricated/accept-inert support: every `apply` still changes real behavior;
  N/A cells have affirmative non-applicability proof, not parser absence;
  Streamr datagram contract explicitly forbids inventing a delay/window timer to
  preserve a blocker;
- Unsupported values keep pre-M140 fail-before-allocation/no-effect behavior;
- No direct-clearnet, outproxy, trusted-peer, Streamr isolation, loopback
  confinement, bounded admission/tasks/timers, transactional lifecycle,
  last-known-good, lock discipline, secret/key/path redaction, LeaseSet
  crypto/scope, or feature/runtime isolation change;
- M061/M062 exact-path/dependency evidence amended only for M140
  planning/test/docs paths; no broad prefix waiver; Yosemite exact pin and
  optional `yosemite-i2pcontrol` ownership preserved;
- Full Proposal 170 support not claimed; partial-support wording retained
  everywhere.

## 6. Failure, cancellation, restart, and contention review

M140 changes no runtime state and introduces no new runtime
failure/cancellation surface. Its correctness failure mode is evidence
misclassification, guarded by: every disposition independently source-cited;
every N/A row including the actual family constructor/runtime path; counts
mechanically derived from row changes; no prose-only count edit (map
`no_prose_only_count_edit` plus `m140` recomputation test). No lock, task,
timer, or queue was introduced. If the Proposal revision, reference heads,
production baseline, or matrix hash change, this closure must be reopened and
the eight-cell inventory regenerated.

## 7. Compatibility, migration, and security review

- No public API version, method, tunnel type, or action change;
- No durable-store migration; no wire/schema contract change; no dependency
  change;
- No migration or rollback operation required (planning/test/docs only);
- No secret material in the map/matrix/docs/guards (redacted Debug and raw-key
  policies untouched);
- No lock across I/O or crypto; no new network/filesystem/DNS/TLS/handshake
  path;
- Containment: preferred production ownership remains
  `emissary-cli/src/i2pcontrol/**`; M140 authorizes no production change;
  deferred M141-M152 path budgets remain non-executable until registered.

## 8. Documentation and operations

Machine authorities updated: `095-full-support-matrix.toml` (`325/40/475`,
SHA `dd77613f…0c44d`), new `140-residual-streaming-applicability-map.toml`
(SHA `75351c45…899d7`), `110-completion-ledger.toml` untouched (no promotions;
M140 is not a completion milestone). Static guards updated: `m095` expects
`325/40/475` with M140 Profile/ConnectDelay dispositions and affirmative-note
checks; `m105` subtracts seven M140 N/A cells; `m062` gains
`is_authorized_m140_path`; new `m140` guard owns eight-record/SHA/counts/
containment checks. Support docs (`proposal-170-support.md`,
`tunnel-manager.md`), `AGENTS.md`, registry, implementation README, and both
residual/full-support roadmaps agree on `325/40/475`, partial support, M140
closure, M141 registration, and the single-cell M143 freeze. Operational
impact: none.

## 9. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | `cargo clippy --tests` reports pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60` | none on M140 files (clean) | record, separate corrective if needed |
| low | `m139_post_lifecycle_requalification::current_matrix_is_exhaustive_and_residuals_are_exact` asserts the M139-head `325/47/468` blocked set and fails on the M140 `325/40/475` head | none on M140 behavior; M139 closure is immutable history and remains the runtime/security authority while M140 supersedes it for residual counts | record as historical drift; future requalification may rebase that suite the way M139 rebased M126-M130 |
| low | Remaining 40 blocked cells (10 SigType, 15 LeaseSet crypto/lookup/auth, 1 Profile, 4 UseSSL, 4 UseOutproxyPlugin, 2 SSLProxies/JumpList, 2 UniqueLocalAddressPerClient, 2 MultiHoming) | partial Proposal 170 support remains; no M140 scope expansion | separate deferred M141-M151 clusters; only M141 registered by this closure |

No high/medium correctness defect remains.

## 10. Registry updates

Applied alongside this closure:

- `140-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `141-*.md` plan: Status `deferred / unregistered` → `registered /
  dependency-ready` (M140 gate satisfied, readiness seams verified, no amendment
  required);
- `plans/registry.md`: M140 → closed as complete (`325/40/475`); handoff M140
  → M141 registered/dependency-ready; residual table `47` → `40` with retained
  `Profile:client`; execution chain updated; M143 constraint annotated with the
  frozen single-cell set (amendment with exact neutral files still required
  before its registration);
- `plans/implementation/.../README.md`: handoff M140 complete → M141
  registered; authority/M095/counts/residual inventory updated;
- residual + full-support roadmaps: M140 complete (`325/40/475`); M141
  registered; M143 retained-set frozen (amendment still pending);
  counts/inventory/registration discipline updated;
- `110-completion-ledger.toml`: unchanged (zero promotions; not a completion
  milestone).

## Internal-only / read-only-upstream attestation

- External sources (Proposal 170 text at `https://i2p.net/proposals/170-i2pcontrol-expansion.txt`,
  Java I2PControl PR head `45bb593000408071dd376b78848fdc246dccd964`, Java
  I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`, Yosemite
  revision `59140a2277bf296928d2e8ce39a148182eeff044`, all fetched via read-only
  raw/API URLs for evidence) were accessed read-only for evidence;
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
