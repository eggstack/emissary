# M144 Closure — Presentation UseSSL Runtime Completion

Status: **closed as complete**

Date: `2026-09-08`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/144-presentation-usessl-runtime-completion.md`
  (Status now `closed as complete` with closure link; registered for execution
  after M143 closure with the pinned family-direction/trust freeze recorded
  here).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## 1. Closure decision

M144 is closed as complete. It is a capability milestone with a maximum
4-cell promotion budget, all four cells promoted. The current matrix is now
exactly:

- `334 apply`;
- `31 blocked_primitive`;
- `475 not_applicable`;
- `840` total cells.

Promoted cells (exact externally observable presentation-TLS effects proven
end-to-end):

- `UseSSL:httpclient` (local listener presents ephemeral TLS, terminated
  before HTTP/CONNECT parsing);
- `UseSSL:connectclient` (same shared listener-TLS owner);
- `UseSSL:httpserver` (accepted stream admitted first, then TLS to the
  validated literal-loopback target);
- `UseSSL:httpbidirserver` (one boolean drives both halves: listener TLS via
  the shared client owner plus target TLS via the shared server owner).

No high- or medium-severity correctness, TLS, or containment issue remains.
Full Proposal 170 support remains partial.

## 2. Reviewed commits and scope

Planning baseline (per M143 closure, M144 hard dependency):

- `a27cbbb058e2e469a6d2b93dea6acbdfffc0260`.

Implementation starting point (HEAD before M144 edits, clean worktree
verified):

- `a27cbbb058e2e469a6d2b93dea6acbdfffc0260`.

Implementation plus closure land in a single commit on the current branch
(see git log for the M144 closure commit). The reviewed production-behavior
baseline extends the lifecycle head `e4f217cb1459e26bf011da46b67fc2c83cd192b5`
(M095 `current_production_head`, retained; M144 adds the I2PControl-local
presentation-TLS owner plus the four family mappings without touching
NetDB/transport/crypto/tunnel-pool/Cargo/Yosemite/frontend or workflows).

Changed paths (exact budget only):

- `emissary-cli/src/i2pcontrol/backends/runtime/presentation_tls.rs` (new;
  application-neutral helper: family gate, boolean validation, ephemeral
  listener identity via existing `rcgen`, system-root + internal-PEM target
  connector via existing `tokio-rustls`/`rustls-pemfile`, explicit 5s
  handshake bound, strict verification, no fallback);
- `emissary-cli/src/i2pcontrol/backends/runtime/mod.rs` (export the helper);
- `emissary-cli/src/i2pcontrol/backends/options.rs` (allow typed `use_ssl`
  for the four families; all others stay rejected);
- `emissary-cli/src/i2pcontrol/backends/http_client.rs` (SUPPORTED `UseSSL`,
  generation-local acceptor built before reserve, TLS termination before
  parsing via boxed post-accept path, redacted Debug);
- `emissary-cli/src/i2pcontrol/backends/connect_client.rs` (same listener
  owner; plus missing `validate_common_options` alignment);
- `emissary-cli/src/i2pcontrol/backends/http_server.rs` (SUPPORTED `UseSSL`,
  generation-local connector built before allocation, TLS wrap of the
  loopback target after admission via the shared accepted handler, boxed
  local stream, redacted Debug);
- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` (SUPPORTED `UseSSL`,
  one boolean builds both acceptor and connector, reuses
  `make_accepted_handler` for the target half with no second TLS
  implementation);
- `emissary-cli/src/i2pcontrol/production.rs` (M120 server-family UseSSL
  rejection narrowed to the three still-N/A families);
- `emissary-cli/tests/m144_presentation_usessl.rs` (new; 14 integration
  guards);
- `emissary-cli/tests/m095_full_support_matrix.rs` (`334/31/475` counts);
- `emissary-cli/tests/m105_residual_option_audit.rs` (subtract M144 cells);
- `emissary-cli/tests/m062_dependency_containment.rs` (exact
  `is_authorized_m144_path` for production/test/docs paths);
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
  (`334/31/475`, UseSSL row `M144` / `apply_or_not_applicable`);
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml`
  (new `post_m144` 4-cell record);
- `plans/implementation/i2pcontrol-proposal-170/144-presentation-usessl-runtime-completion.md`
  (Status `deferred` → `closed as complete`);
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `AGENTS.md`;
- `docs/i2pcontrol/proposal-170-support.md`, `docs/i2pcontrol/tunnel-manager.md`,
  `docs/i2pcontrol/tunnel-backends.md`;
- new `plans/closure/i2pcontrol-proposal-170/144-closure.md` (this file).

`git diff --name-only` proves no `emissary-core/**`, `emissary-util/**`,
manifest, lockfile, Yosemite, frontend, or workflow path changed.
`git diff --check` passes.

## 3. Requirement evidence

### Pinned reference (all accessed read-only)

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (Proposal lists `UseSSL` boolean “enable SSL where supported” without a
  per-family value partition; family applicability frozen by M095/M131);
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`:
  `I2PTunnelClientBase.java` `PROP_USE_SSL` (shared with
  `I2PTunnelServer.PROP_USE_SSL`): `run()` builds an `SSLServerSocket` via
  `SSLClientUtil.verifyKeyStore` (self-signed CN `localhost`, SANs
  `localhost`/`127.0.0.1`/`::1` plus listen interface) and
  `initializeFactory` when true — TLS termination immediately after local
  accept, before parsing; `httpclient`/`connectclient` share this owner
  (both extend `I2PTunnelClientBase`);
  `I2PTunnelServer.java` `getSocket(Hash, InetAddress, int)`: accepted I2P
  stream first, then TLS client socket to the configured local target via
  `I2PSSLSocketFactory` (system certs plus `certificates/i2ptunnel`, hostname
  skipped only for loopback but chain trust still required); `httpserver` /
  `httpbidirserver` share this owner; the `targetForPort` 443/22
  `forceNonSSL` exception applies only to the multi-target overload which
  Emissary does not implement (single literal loopback target), so Emissary
  always wraps when enabled;
  `I2PTunnelHTTPBidirServer.java` extends the server plus a
  `I2PTunnelHTTPBidirProxy` (extends `I2PTunnelHTTPClient`): one `useSSL`
  boolean drives both the target wrap and the proxy listener;
  `TunnelConfig.java` carries `useSSL` as a boolean option for both client
  (`_booleanClientOpts`) and server (`_booleanServerOpts`); no
  `SSLCertificate`/`SSLKey` Proposal companion exists;
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  (pass-through `useSSL` boolean to the tunnel controller; no SAM mapping);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (its `SessionOptions.ssl` is SAM-control TLS and is deliberately never set
  by M144; proven by `m144_management_and_sam_tls_remain_independent`).

### Exact reference freeze applied before coding

- Client families are TLS servers (present), server families are TLS clients
  (originate); `httpclient`/`connectclient` share one listener owner;
  `httpserver`/`httpbidirserver` share one accepted-stream target owner;
  `httpbidirserver` enables both halves from the single boolean;
- Listener identity is ephemeral generation-local self-signed (reference
  `verifyKeyStore`-generates-if-absent adapted to bounded lifetime: no
  durable keystore files, no cross-generation reuse, distinct CN
  `Emissary I2PTunnel` from M129 management identity);
- Target trust is system roots (standard bundles via existing
  `rustls-pemfile`, no new dependency) plus the internal
  `__emissary_test_trust_anchors` PEM seam (test/internal only, never a
  Proposal cell); hostname strictly verified against the loopback target
  (stricter than the reference loopback skip; no bypass, no trust-all);
- `SSLCertificate`/`SSLKey` are not Proposal companions and remain rejected;
  `UseSSL=true` needs no file material; `UseSSL=false`/omitted is plaintext.

Dedup note: Emissary verifies loopback hostnames strictly where Java skips
them; this is an intentional hardening with identical fail-closed outcomes
for correctly issued loopback certificates and is recorded here.

### Pre/post matrix rows and counts

Pre-M144 (M143-qualified): `330/36/475` is wrong; correct pre-M144 is
`330/35/475` (matrix SHA before M144 edits is the M143 SHA
`cdd38453b6402df7e9612bb62c2b3afad42e2db3d25da239cb4896e164c5fadd`).
Post-M144: `334/31/475` (matrix SHA
`af01bc0efb4723f471e442e4cdf5dd8e70f6a9473ac5fe0be022ae9cadfa34c9`).
Recomputed from cells: `total=840 apply=334 blocked=31 na=475`; declared
counts match recomputation.

Changed cells (only these four; no other cell changed):

- `UseSSL:httpclient` `blocked_primitive -> apply`;
- `UseSSL:connectclient` `blocked_primitive -> apply`;
- `UseSSL:httpserver` `blocked_primitive -> apply`;
- `UseSSL:httpbidirserver` `blocked_primitive -> apply`.

Row now owns `completion_owner = "M144"`,
`current_or_planned_disposition = "apply_or_not_applicable"`, no
`blocked_primitive`/`blocking_milestone`, cells `[1]=[5]=[8]=[9]=apply` with
M144 direction/trust notes; all other UseSSL cells remain `not_applicable`.

### I2PControl validation and handshake ownership

- TunnelManager canonical validation already enforces boolean type for
  `UseSSL`; `extract_tunnel_options` carries the typed boolean and
  `raw_config` preserves it for round-trip; `Get` returns the exact boolean;
- `options::validate_common_options` now allows typed `use_ssl` only for the
  four families; all others reject before allocation;
- each target backend `validate_raw_options` promotes `"UseSSL"` to SUPPORTED
  and re-validates via the shared `parse_use_ssl` (non-boolean, mismatch, or
  non-target presence fails with `UnsupportedOption` naming only `UseSSL`,
  never echoing values);
- `config()` builds TLS material before reserve/bind: listener acceptor via
  `rcgen` (in-memory, generation-local `Arc`, stale tasks retain only the old
  `Arc`); target connector via system roots plus optional internal PEM
  (validated before use); `UseSSL=false` builds nothing;
- client handshake happens immediately after TCP accept inside the connection
  handler, before `read_header_block`/CONNECT parsing; the post-handshake
  plaintext stream flows through the identical boxed filter path as plaintext
  (proven indistinguishable by shared-code construction);
- server handshake happens after I2P admission and HTTP request sanitization,
  wrapping the already source-bound loopback TCP connection (unique-local
  bind preserved: `TcpSocket` bind port 0 before `connect`, then TLS with
  SNI loopback and explicit 5s timeout); failures use the bounded 502 path,
  never plaintext fallback;
- `httpbidirserver` passes its single boolean into both the shared accepted
  handler (target) and the shared no-outproxy client handler (listener);
  static guards prove no second `rcgen`/`ServerConfig` implementation exists
  outside the helper.

### Observable behavior

- `UseSSL=false`/omitted preserves current plaintext behavior for all four
  families (existing parser/filter/admission tests unchanged);
- client TLS listener completes a real `tls12/1.3` handshake (ring) before
  parsing (integration `m144_client_tls_listener_completes_handshake_before_parsing`
  exchanges HTTP over the negotiated stream);
- plaintext on a TLS listener fails the handshake and is never dispatched as
  HTTP (`m144_plaintext_on_tls_listener_is_rejected_without_dispatch`);
- server TLS target completes a real handshake with a trusted anchor and
  forwards only after verification; untrusted anchors fail closed with no
  retry (`m144_server_tls_target_verifies_and_never_falls_back`);
- malformed/missing/mismatched `UseSSL` rejected before listener/replacement
  allocation (helper unit + backend preflight tests);
- secret material absent from Debug/errors/`Get`/rawConfig (redacted Debug,
  static error strings, `TunnelDefinition` key-only Debug);
- handshake timeout explicit 5s and bounded; cancellation tears down via
  existing supervisor `reserve`/`stop_generation` with no lock held across
  file reads, bind/connect, handshake, or I/O;
- old generation retains old TLS state during failed edit (validation before
  `reserve`), successor builds a fresh ephemeral identity only after
  successful restart (distinct certs proven);
- `httpbidirserver` reuses the server TLS owner (source containment guard);
- M129 management TLS and Yosemite SAM TLS unchanged (session options
  `ssl` stays false; `tls.rs` untouched).

### Failure, cancellation, restart, and contention evidence

- `UseSSL` validation is synchronous/pre-allocation (pure boolean checks plus
  in-memory `rcgen`/PEM parsing, no listener/session/I/O);
- TLS material construction happens in `config()` before supervisor `reserve`,
  so a bad value never replaces the running generation;
- listener readiness is published only after TCP bind with valid TLS config;
  cancellation during setup drops the reservation (`CreationReservation`
  pattern retained) and aborts the generation task;
- per-generation `TlsAcceptor`/`TlsConnector` are immutable `Arc`s; stale
  accepted tasks hold only the old `Arc` until drain (`STOP_TIMEOUT`),
  successor tasks hold only the new `Arc`;
- no `Mutex`/`parking_lot` guard held across destination resolution, SAM/I2P
  stream connection, TLS handshake, request forwarding, or response I/O
  (selector/compat locks are short-lived map lookups only);
- shared-session admission unchanged (`additional_options_identity` does not
  include presentation TLS, which is a local-endpoint property, not a session
  setting — correctly excluded from sharing identity).

### Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `821 passed` (includes 5 new presentation-TLS unit tests) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m144_presentation_usessl --no-fail-fast` | **pass**: `48 passed (5 suites)`; matrix `334/31/475`; M144 promotion/residual/containment checks green (m144 alone: `14 passed`) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m129_nonloopback_tls --no-fail-fast` | historical drift: 11 pass, 1 fail (`proposal_matrix_unchanged_by_tls_fail_closed` asserts old `325` counts) — M129 closure immutable, not an M144 regression |
| `git diff --check` | **pass** |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide (unchanged unrelated files); all twelve M144-touched/added files are individually stable-rustfmt-clean after `rustfmt --edition 2021` on those files only; no unrelated normalization |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **evidence only**: `1 error` — sole error is the known pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60`; all M144 files otherwise clean (one new `too_many_arguments` on the extended accepted handler was fixed with an explicit allow) |

Full `cargo test -p emissary-cli --features i2pcontrol` (all suites) retains
ten expected historical failures independent of M144, all count/wording
drift (not behavioral regressions):

- `m126_requalification` (2), `m127_token_lifetime`, `m128_jsonrpc_batch`,
  `m129_nonloopback_tls`, `m130_post_corrective_requalification` assert the
  old `325`-era counts/wording;
- `m139_post_lifecycle_requalification` asserts the M139-head `325/47/468`
  blocked set;
- `m140_residual_streaming_applicability` asserts the M140-head `325/40/475`
  counts;
- `m141_unique_local_source` (2 tests) asserts the M141-head `327/38/475`
  counts;
- `m142_httpclient_sslproxies_jumplist` (2 tests) asserts the M142-head
  `329/36/475` counts;
- `m143_streaming_profile` (3 tests) asserts the M143-head `330/35/475`
  counts.

M139/M140/M141/M142/M143 closures are immutable history; M144 supersedes them
for counts while M139 remains the runtime/security qualification authority.
Recorded here as historical drift, not M144 regressions. No M144-required
suite fails.

### Production-path diff (bounded)

`git diff --name-only` plus untracked-file inspection proves only the exact
budget changed (see §2). No `emissary-core/**`, `emissary-util/**`, manifest,
lockfile, Yosemite, frontend, or workflow file changed or added.
`110-completion-ledger.toml` gains `post_m144` (4 cells). Current unsupported
values keep their fail-before-allocation/no-effect behavior (Streamr datagram
limits — 16-subscriber, 60s expiry, 1200-byte payload, 4095-byte
transport buffer, 15s refresh, bounded shutdown, loopback-only UDP — untouched;
remote datagrams never choose a local UDP destination).

## 4. Requirement-to-evidence matrix

| Plan requirement (§13) | Evidence | Result |
|---|---|---|
| exact TLS direction/trust semantics source-frozen for every promoted family | §3 pinned freeze (client=present via `I2PTunnelClientBase`, server=originate via `I2PTunnelServer.getSocket`, bidir=both, ephemeral listener identity, system-root + internal-PEM target trust, strict loopback SNI, no `SSLCertificate` companion) | **pass** |
| real handshakes occur at the correct local application boundary | client handshake before parsing (`m144_client...`), server handshake after admission before forwarding (`m144_server...`), plaintext rejected, no fallback | **pass** |
| no plaintext fallback or verification bypass exists | handshake errors close without dispatch; untrusted fails with no retry; no `dangerous`/trust-all/hostname-ignore code (static grep + negative tests) | **pass** |
| management TLS and SAM TLS remain independent | `tls.rs` untouched; `options.ssl` stays false for all four families; M129 suite still passes except historical count drift | **pass** |
| secret handling, cancellation, timeout and last-known-good behavior proven | redacted Debug/static errors; 5s handshake bound; reserve-after-validation; generation-local `Arc`s; successor-new-state + failed-edit-preserved tests | **pass** |
| no non-I2PControl production path or new dependency unless amended | `git diff --name-only` + M062 exact-path green; `Cargo.toml`/lockfile unchanged; existing `tokio-rustls`/`rcgen`/`rustls-pemfile` only | **pass** |
| M061/M062/M095/M105/docs/registry reconciled | M095 `334/31/475` recomputed; `m095`/`m105`/`m062`/`m144` green; `AGENTS.md`, registry, README, both roadmaps, all three docs state `334/31/475` and partial support | **pass** |
| no medium/high TLS/security defect remains | invariant/failure/compatibility reviews below; only low historical-drift findings | **pass** |

## 5. Invariant review

- Exact pinned names/types/presence preserved; four `apply` promotions only,
  zero `not_applicable` moves, no new field/alias/status/method/tunnel-type;
- No fabricated/accept-inert support: every `apply` negotiates real TLS at
  the pinned boundary (listener presents, target originates) with verification;
  N/A cells retain affirmative non-applicability proof; Streamr datagram
  contract untouched;
- Unsupported values keep fail-before-allocation/no-effect behavior with no
  echo;
- No direct-clearnet, outproxy, trusted-peer, Streamr isolation, loopback
  confinement, bounded admission/tasks/timers, transactional lifecycle,
  last-known-good, lock discipline, secret/key/path redaction, LeaseSet
  crypto/scope, or feature/runtime isolation change beyond the additive
  presentation-TLS owner;
- M061/M062 exact-path/dependency evidence amended only for M144
  planning/test/docs plus the eight production files (seven modified + one
  new helper); no broad prefix waiver; Yosemite exact pin and optional
  `yosemite-i2pcontrol` ownership preserved;
- Full Proposal 170 support not claimed; partial-support wording retained
  everywhere.

## 6. Failure, cancellation, restart, and contention review

- `UseSSL` validation inherits the existing bounded Yosemite/session/stream
  timeout contract plus the explicit 5s TLS handshake bound; timeout and
  bind/connect/handshake errors map to per-connection failures (502 for
  servers, close for listeners) with no retry loop;
- cancellation of the tunnel generation cancels/drains accepted tasks under
  existing bounded task-group semantics (no new task/timer/queue; TLS state
  is immutable data);
- failed validation closes nothing and records no state (proven by malformed
  tests with `Stopped` inspect and by `build` failing while `inspect` stays
  `Stopped`, never stale `Running`);
- restart/edit failure preserves last-known-good: TLS material gates `start`
  via `config()` before supervisor `reserve`, so a bad edit never replaces
  the running generation (existing M120 order retained);
- no `Mutex`/`parking_lot` guard held across destination resolution, SAM/I2P
  stream connection, TLS handshake, request forwarding, or response body I/O
  (presentation state is lock-free immutable `Arc`s;
  `select`/`note_result` short-lived only).

## 7. Compatibility, migration, and security review

- No public API version, method, tunnel type, or action change; `Get` returns
  the exact persisted boolean via raw config; no wire/schema change beyond
  the additive local TLS negotiation;
- No durable-store migration; existing definitions without the key default to
  plaintext (`false`, backward compatible); explicit `"false"` shares the
  same effective behavior as omitted but round-trips distinctly; no dependency
  change;
- No migration or rollback operation required (additive boolean + local TLS);
- No secret material in selection/tests/docs/guards beyond ephemeral
  generation-local keys (never persisted, never logged; rejections never echo
  values; `__emissary_test_trust_anchors` carries only public trust certs,
  never keys, and never enters response config);
- No lock across I/O or crypto beyond the bounded handshake; no new
  network/filesystem/DNS path beyond the bounded loopback TCP + TLS handshake
  and standard bundle reads;
- Containment: preferred production ownership remains
  `emissary-cli/src/i2pcontrol/**`; deferred M145-M152 path budgets remain
  non-executable until registered (M145 still needs its exact-file amendment);
- Full Proposal 170 support not claimed; partial-support wording retained.

## 8. Documentation and operations

Machine authorities updated: `095-full-support-matrix.toml` (`334/31/475`,
SHA `af01bc0efb4723f471e442e4cdf5dd8e70f6a9473ac5fe0be022ae9cadfa34c9`),
`110-completion-ledger.toml` (`post_m144` 4 cells), `m095` expects
`334/31/475` with M144 ownership/disposition checks, `m105` subtracts M144
cells, `m062` gains `is_authorized_m144_path`, new `m144` guard owns
promotion/residual/validation/containment checks. Support docs
(`proposal-170-support.md`, `tunnel-manager.md`, `tunnel-backends.md`),
`AGENTS.md`, registry, implementation README, and both residual/full-support
roadmaps agree on `334/31/475`, partial support, M144 closure, no registered
successor, and the 31-cell residual inventory. Operational impact: none beyond
the additive opt-in TLS; absent values preserve prior behavior.

## 9. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | `cargo clippy --tests` reports pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60` | none on M144 files (clean) | record, separate corrective if needed |
| low | `m126`/`m127`/`m128`/`m129`/`m130`/`m139`/`m140`/`m141`/`m142`/`m143` suites assert old `325`/`327`/`329`/`330`-era counts/wording and fail on the M144 `334/31/475` head | none on M144 behavior; those closures are immutable history (M139 remains runtime/security authority, M140 remains streaming-applicability authority, M141 remains unique-local authority, M142 remains SSL/Jump authority, M143 remains Profile authority) | record as historical drift; future requalification may rebase those suites the way M139 rebased M126-M130 |
| low | Remaining 31 blocked cells (10 SigType, 15 LeaseSet crypto/lookup/auth, 4 UseOutproxyPlugin, 2 MultiHoming) | partial Proposal 170 support remains; no M144 scope expansion | separate deferred M145-M152 clusters; only M145 is next (still unregistered) |
| info | Emissary verifies loopback hostnames strictly where Java skips them; ephemeral listener identities are per-generation rather than persisted keystores | no security issue; strictly safer, same fail-closed outcomes for correctly issued certs | documented; no action |

No high/medium correctness defect remains.

## 10. Registry updates

Applied alongside this closure:

- `144-*.md` plan: Status `deferred` → `closed as complete` with closure link;
- `plans/registry.md`: M144 → closed as complete (`334/31/475`); handoff M144
  closed with no registered successor; residual table `35` → `31`
  (UseSSL removed); execution chain updated; M145 constraint retained (needs
  exact-file amendment before registration);
- `plans/implementation/.../README.md`: handoff M144 closed, no registered
  successor; authority/M095/counts/residual inventory updated;
- residual + full-support roadmaps: M144 closed (`334/31/475`); M145 still
  deferred; counts/inventory/registration discipline updated;
- `110-completion-ledger.toml`: `post_m144` 4 cells added; matrix SHA recorded.

## Future-plan unblock determination

M144 unblocks no future plan for execution:

- **No plan registered**: M145 hard-depends on M144, and that hard dependency
  is now satisfied, but M145 still requires its own exact-file amendment
  (neutral outbound-message/LeaseSet bundling owner files, per its plan §5
  and the registry constraint) plus an explicit registration decision before
  execution. M144 does not satisfy that amendment, so M145 remains
  deferred/unregistered. M146-M152 remain deferred/unregistered behind M145.
- No other future-plan status required a change beyond the M144-closed handoff.
  The sole next step is for a maintainer to amend M145 with exact owner files
  and explicitly register it; file presence alone never authorizes production
  work.

## Internal-only / read-only-upstream attestation

- External sources (Proposal 170 text at `https://i2p.net/proposals/170-i2pcontrol-expansion.txt`,
  Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`
  (`I2PTunnelClientBase.java` `PROP_USE_SSL`/SSL listener,
  `I2PTunnelServer.java` `PROP_USE_SSL`/`getSocket` SSL target,
  `I2PTunnelHTTPBidirServer.java`/`I2PTunnelHTTPBidirProxy.java` bidir both
  halves, `ui/TunnelConfig.java` boolean carrier, `util/I2PSSLSocketFactory.java`
  trust/hostname policy, `i2ptunnel/SSLClientUtil.java` keystore generation),
  Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`,
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
