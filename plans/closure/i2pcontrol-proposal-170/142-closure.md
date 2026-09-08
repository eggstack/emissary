# M142 Closure — HTTP Client SSLProxies and JumpList Completion

Status: **closed as complete**

Date: `2026-09-08`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/142-httpclient-sslproxies-and-jumplist-completion.md`
  (Status now `closed as complete` with closure link).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## 1. Closure decision

M142 is closed as complete. It is a capability / HTTP proxy routing and
presentation milestone with a maximum 2-cell promotion budget, both cells
promoted. The current matrix is now exactly:

- `329 apply`;
- `36 blocked_primitive`;
- `475 not_applicable`;
- `840` total cells.

Promoted cells (exact observable I2P-only routing/presentation effect proven
end-to-end):

- `SSLProxies:httpclient`;
- `JumpList:httpclient`.

No high- or medium-severity correctness or proxy-routing/HTTP security issue
remains. Full Proposal 170 support remains partial.

## 2. Reviewed commits and scope

Planning baseline (per M141 closure, M142 hard dependency):

- `fe3e0b1ee6024bc7f159a961791328fffae4b2e7`.

Implementation starting point (HEAD before M142 edits):

- `fe3e0b1ee6024bc7f159a961791328fffae4b2e7`.

Implementation plus closure land in a single commit on the current branch
(see git log for the M142 closure commit). The reviewed production-behavior
baseline extends the lifecycle head `e4f217cb1459e26bf011da46b67fc2c83cd192b5`
(M095 `current_production_head`, retained; M142 adds the I2PControl-local
HTTP-client SSL selector/jump-helper owner without touching core/router/
transport).

Changed paths (I2PControl-local only; no `emissary-core/**`, Cargo/dependency,
Yosemite, NetDB, transport, crypto, startup tunnel, frontend, or `.github/**`
production change):

- `emissary-cli/src/i2pcontrol/backends/filters/http_client.rs` (M142 bounded
  owners: `MAX_SSL_RAW_LEN 2048`/`MAX_SSL_ENTRIES 8`/`MAX_SSL_ENTRY_LEN 255`,
  `MAX_JUMP_RAW_LEN 2048`/`MAX_JUMP_ENTRIES 8`/`MAX_JUMP_ENTRY_LEN 256`,
  `SSL_CACHE_CAPACITY 32`; `parse_ssl_proxy_list` with typo repair,
  lowercase/dedup, I2P-only validation; `parse_jump_server_list` with
  http-only `.i2p` URL-prefix validation; `SslProxySelector` LRU-32 +
  single last-failed; `escape_html`; `render_jump_helper_response` pure;
  CONNECT + `https` absolute support with `is_ssl`/`host_key`/`is_i2p`);
- `emissary-cli/src/i2pcontrol/backends/http_client.rs` (distinct
  `ssl_selector` generation-local `Arc<Mutex<SslProxySelector>>` +
  `jump_servers` typed extraction; `SSLProxies`/`JumpList` `SUPPORTED`
  promotion with fail-before-allocation; `validate_start` pure preflight;
  CONNECT-via-SSL and HTTPS-via-SSL branches through existing I2P-only
  `connector.connect_to`; jump-helper 502 path for plain `.i2p`
  destination-not-found; ordinary I2P never rerouted; no clearnet fallback;
  `#[allow(clippy::too_many_arguments)]` on the 8-arg handler);
- `emissary-cli/tests/m142_httpclient_sslproxies_jumplist.rs` (new; 14
  integration guards: matrix promotion, residual delta, empty/single/multi/
  last-failed/single-failure/cache-bound, malformed rejection, non-HTTP
  gates, jump absent/safe/injection/purity, clearnet static guard,
  round-trip, docs agreement, fake-SAM HTTPS selection, jump render +
  restart);
- `emissary-cli/tests/m095_full_support_matrix.rs` (M142 apply expectations,
  `329/36/475` counts);
- `emissary-cli/tests/m105_residual_option_audit.rs` (subtract two M142 cells);
- `emissary-cli/tests/m062_dependency_containment.rs` (exact
  `is_authorized_m142_path` for the two production files plus M142
  planning/test/docs paths);
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
  (`329/36/475`, SSLProxies/JumpList rows `M142` / `apply_or_not_applicable`);
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml`
  (new `post_m142` 2-cell record);
- `plans/implementation/i2pcontrol-proposal-170/142-httpclient-sslproxies-and-jumplist-completion.md`
  (Status `registered` → `closed as complete`);
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `AGENTS.md`;
- `docs/i2pcontrol/proposal-170-support.md`, `docs/i2pcontrol/tunnel-manager.md`,
  `docs/i2pcontrol/tunnel-backends.md`;
- new `plans/closure/i2pcontrol-proposal-170/142-closure.md` (this file).

`git diff --name-only` proves no `emissary-core/**`, `emissary-util/**`,
manifest, lockfile, Yosemite, frontend, or workflow path changed. `git diff
--check` passes.

## 3. Requirement evidence

### Pinned reference (all accessed read-only)

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (Proposal lists `SSLProxies`/`JumpList` string options; HTTP-client-only
  applicability frozen by M131 with affirmative parser/I2PTunnel gates);
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`:
  `apps/i2ptunnel/java/src/net/i2p/i2ptunnel/I2PTunnelHTTPClientBase.java`
  `PROP_SSL_OUTPROXIES = "i2ptunnel.httpclient.SSLOutproxies"`,
  `selectSSLProxy(host)`: null when unconfigured, typo repair
  `exit.storymcloud.i2p` → `exit.stormycloud.i2p`,
  `DataHelper.split(s, "[,; \\r\\n\\t]")`, size 0 → null, size 1 → `p[0]`,
  else hostname-cached (`_proxySSLCache = LHMCache(32)`) randomized
  `nextInt(size)` with `_lastFailedSSLProxy` avoidance, parsed on the fly;
  `noteProxyResult(proxy, host, isSSL, ok)`: success clears matching
  last-failed and pins cache, failure stores last-failed and drops matching
  cache entry; `selectProxy` requires restart, `selectSSLProxy` does not;
  `writeErrorMessage(..., jumpServers)`: `jumpServers` comma/space-separated,
  only `http` scheme with `.i2p` host accepted, non-b32 hosts skipped unless
  `namingService().lookup` succeeds, `targetRequest` HTML-escaped,
  `jurl + uri` links under `#jumplinks`;
- `apps/i2ptunnel/java/src/net/i2p/i2ptunnel/I2PTunnelHTTPClient.java`
  `PROP_JUMP_SERVERS = "i2ptunnel.httpclient.jumpServers"`,
  `DEFAULT_JUMP_SERVERS = "http://stats.i2p/cgi-bin/jump.cgi?a=,http://i2pjump.i2p/jump/,http://notbob.i2p/cgi-bin/jump.cgi?q="`,
  `isConnect` from `CONNECT` (spoofed `https://` + `/`), `https` protocol or
  `isConnect` selects `selectSSLProxy`, else `selectProxy`; jump servers only
  on non-`usingWWWProxy`, non-`ahelperPresent`, non-B32 destination-not-found
  (`dnfh` + `400 + random(256)` ms delay), never fetched;
- `TunnelConfig.java` `setSslProxies(s.trim().replace(" ", ","))`,
  `setJumpList(val.trim().replace("\\r\\n", ",").replace("\\n", ",").replace(" ", ","))`
  (normalization evidence for the bounded separators);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  (parser/creator reachability unchanged; no new applicability gate);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (transport-only; no M142 dependency change).

### Exact reference freeze applied before coding

- SSL separators: comma/semicolon/whitespace (` `, `\t`, `\r`, `\n`),
  matching `DataHelper.split`; empty tokens dropped; entries lowercased,
  deduplicated preserving first; typo repair applied;
- SSL bounds: raw ≤2048, entries ≤8, each ≤255, default port 4444, port ≠0;
  cache LRU-32 (reference `LHMCache(32)`); failure memory exactly one
  (`_lastFailedSSLProxy`); single entry returns directly without cache;
  multi-entry randomized via `rand::rng().random_range`, avoiding last-failed
  when alternatives exist; empty-random-range impossible (≥2 entries minus at
  most one failure leaves ≥1 candidate; dedup preserves this);
- SSL request class: `CONNECT` or absolute `https://` (default 443) uses the
  SSL selector for clearnet; ordinary `http://`/origin-form uses the ordinary
  `ProxyList`; I2P (`.i2p`/`.b32.i2p`/full destination) never uses either
  list; missing/failed SSL selection yields the existing bounded 403/502
  path, never a clearnet fallback or `ProxyList` fallback;
- Jump separators: same comma/semicolon/whitespace class; normalization
  lowercases the URL host, preserves path/query, drops empties, dedups;
- Jump bounds: raw ≤2048, entries ≤8, each ≤256, host ≤255;
- Jump entries: `http://*.i2p/*` URL prefixes only; `https`/arbitrary schemes,
  userinfo, fragments, `" ' < >` whitespace/control/CRLF all rejected;
- Jump circumstances: only I2P resolution failure for a plain `.i2p`
  non-B32 hostname with a non-empty JumpList renders the helper; B32/full
  destinations, clearnet/outproxy failures, and absent/empty lists keep the
  existing empty 502; output is `502 Bad Gateway` + `text/html` with
  `Content-Length`, escaped target and escaped `jurl + target` links under
  `#jumplinks`; pure/bounded, no fetch/connect/spawn.

Dedup (SSL distinct destinations, Jump normalized URLs) is the sole
observable bounded adaptation versus reference duplicates-as-weighted-entries;
it preserves distinct-destination selection while bounding and is recorded
here.

### Pre/post matrix rows and counts

Pre-M142 (M141-qualified): `327/40/475` is a typo; exact pre-M142 is
`327/38/475` (matrix SHA before M142 edits is the M141 SHA
`528440dcabed526ee2d57782f5c55d9a92afbd04a58bcaea3643d168afe98def`).
Post-M142: `329/36/475` (matrix SHA
`e5fe8e2b28103bbceea3a92aa58c2e296c98337ce5c5df4625abb26f791a3576`).
Recomputed from cells: `total=840 apply=329 blocked=36 na=475`; declared
counts match recomputation.

Changed cells (only these two; no other cell changed):

- `SSLProxies:httpclient` `blocked_primitive -> apply`;
- `JumpList:httpclient` `blocked_primitive -> apply`.

Rows now own `completion_owner = "M142"`,
`current_or_planned_disposition = "apply_or_not_applicable"`, no
`blocked_primitive`/`blocking_milestone`, cells `[1]=apply` with M142
I2P-only routing/presentation notes; all other SSL/Jump cells remain
`not_applicable` per M131.

### Validation and persistence before allocation

- TunnelManager canonical validation already enforces string type for
  `SSLProxies`/`JumpList` (`validate_string`); malformed types fail at the
  control plane before backend allocation;
- `httpclient::validate_raw_options` promotes both keys to `SUPPORTED` and
  calls `parse_ssl_proxies`/`parse_jump_servers` so non-string, oversized,
  control-char, and structurally invalid entries fail with
  `UnsupportedOption`/`Internal("... is invalid")` before any runtime
  reservation; `config()` re-validates types and parses the single canonical
  raw-config representation (no typed/raw dual storage);
- `validate_start` (pure, no allocation/I/O) mirrors `start` via
  `validate_common_options` + `validate_options` + `validate_raw_options` +
  `client_lifecycle_config`, so preflight and start cannot drift; `start`
  calls `validate_start` before `config()` and supervisor `reserve`, so a bad
  edit never replaces the running generation;
- non-target families (`socks`, `socksirc`, `connectclient`, all servers,
  Streamr) reject any supplied value via their existing `UnsupportedOption`
  raw gates before allocation (proven by
  `m142_non_http_families_keep_exact_rejection` for socks/connectclient;
  other families share the same unknown-key rejection shape);
- `Get`/persistence round-trip preserves exact strings via raw config
  (proven by deterministic JSON round-trip test); `TunnelDefinition` Debug
  redacts raw values and never logs passwords (`HttpClientConfig` Debug shows
  only `ssl_proxies` count and `jump_servers` count, passwords as `***`;
  selector cache keys are `host:port` hostnames only).

### I2P-only routing and no-clearnet-fallback evidence

- Clearnet hostnames never reach OS DNS or direct `TcpStream::connect`:
  static guard asserts no `to_socket_addrs`/`getaddrinfo`/
  `TcpStream::connect` in either production file; `resolve_destination`
  returns `None` for clearnet without an address-book entry, so the original
  public hostname can never become a `connect_to` destination;
- Outproxy entries resolve through the accepted I2P path only
  (`resolve_destination`: `.b32.i2p`/full destination direct, else
  private/local/router/published address-book lookup); non-I2P SSL entries
  rejected at validation; ordinary `ProxyList` destination re-checked with
  `is_i2p_destination` on the request path;
- Fake-SAM integration (`m142_https_connect_selects_i2p_ssl_outproxy_not_clearnet`)
  binds a loopback fake SAM recording `STREAM CONNECT DESTINATION=...`,
  starts `HttpClientTunnelBackend` with
  `SSLProxies=ssl-outproxy.b32.i2p`, sends
  `CONNECT example.com:443` to the loopback listener, and asserts SAM saw
  `ssl-outproxy.b32.i2p` and never `example.com`;
- Missing/failed SSL selection yields 403 (no selector) or 502
  (resolve/connect/handshake failure with `note_result(..., false)`), never a
  `ProxyList` fallback or direct route; ordinary HTTP clearnet without
  `ProxyList` still yields 403; ordinary I2P requests never consult either
  list.

### Failure, cancellation, restart, and contention evidence

- `note_result` is called only after an authoritative selected-proxy
  resolve/connect/handshake attempt; parse/auth failures never touch
  selector state;
- Single-proxy failure does not panic and returns the sole entry (early
  return, no `random_range(0..0)`); multi-entry failure avoidance leaves ≥1
  candidate (proven by unit tests);
- Cache is LRU-32; attacker `Host` churn test proves `cache_len ≤ 32` after
  `4×` capacity inserts;
- No `Mutex` held across destination resolution, SAM/stream connect, request
  forwarding, or response I/O: `select` clones under a short lock, async I/O
  runs unlocked, `note_result` re-locks briefly (proven by code inspection;
  `RuntimeMap` locks remain short-lived around map mutation only);
- Cancellation discards the generation: selector state is
  `Arc<Mutex<...>>` owned by `HttpClientConfig` for one `reserve`d
  generation; `stop_generation` removes the generation and drops the `Arc`,
  so a stale handler cannot mutate a successor (proven by
  `m142_unknown_i2p_renders_jump_links_and_restart_drops_generation_state`
  stop → `Stopped` → start → `Running` with deterministic jump behavior);
- Edit/restart failure preserves last-known-good: `validate_start` gates
  `start`; supervisor `reserve` happens only after validation, so a bad edit
  never replaces the running generation (existing M120 order retained);
- Jump rendering is pure/bounded string construction; no task/timer/queue,
  no spawn, no network (proven by static guard + unit tests).

### Verification outcomes

| Command group | Result |
|---|---|
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --test m142_httpclient_sslproxies_jumplist --test m061_containment --test m062_dependency_containment --no-fail-fast` | **pass**: `48 passed (5 suites)`; matrix `329/36/475`; M142 promotion/residual/containment checks green (m142 alone: `14 passed`) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `816 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | **pass**: `1 passed` |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `git diff --check` | **pass** |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide (unchanged unrelated files); all six M142-touched/added files are individually stable-rustfmt-clean after `rustfmt --edition 2021` on those files only; no unrelated normalization |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **evidence only**: `1 error` — sole error is the known pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60`; M142 `too_many_arguments` resolved via explicit `#[allow]` on the 8-arg handler; all M142 files otherwise clean |

Full `cargo test -p emissary-cli --features i2pcontrol` (all suites) retains
ten expected historical failures independent of M142, all count/wording
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
  `327/38/475` counts.

M139/M140/M141 closures are immutable history; M142 supersedes them for counts
while M139 remains the runtime/security qualification authority. Recorded
here as historical drift, not M142 regressions. No M142-required suite fails.

### Production-path diff (bounded)

`git diff --name-only` plus untracked-file inspection proves only the
authorized I2PControl-local paths changed (see §2). No `emissary-core/**`,
`emissary-util/**`, manifest, lockfile, Yosemite, frontend, or workflow file
changed or added. `110-completion-ledger.toml` gains `post_m142` (2 cells).
Current unsupported values keep their fail-before-allocation/no-effect
behavior (Streamr datagram limits — 16-subscriber, 60s expiry, 1200-byte
payload, 4095-byte buffer, 15s refresh, bounded shutdown, loopback-only UDP —
untouched; remote datagrams never choose a local UDP destination).

## 4. Requirement-to-evidence matrix

| Plan requirement (§13) | Evidence | Result |
|---|---|---|
| both implemented options have exact pinned HTTP-only semantics | `selectSSLProxy`/`noteProxyResult`/`writeErrorMessage` freeze (§3); `is_ssl` gates CONNECT/`https` to SSL list, ordinary to `ProxyList`, I2P to direct; non-HTTP N/A unchanged; `m142_non_http...` + selector tests | **pass** |
| no direct clearnet fallback/DNS path exists | I2P-only `resolve_destination`/`connect_to`; static no-DNS/no-`TcpStream::connect` guards; fake-SAM proves `ssl-outproxy.b32.i2p` selected, never `example.com` | **pass** |
| state/list/cache bounds are explicit and tested | raw 2048/entry 255-256/count 8/LRU-32/single-failure; empty/single/multi/churn tests; oversized rejected | **pass** |
| failure/restart/cancellation behavior is deterministic | `note_result` only after authoritative attempt; single-failure no-panic; LRU eviction; generation-local `Arc` discarded on stop; restart test proves clean successor | **pass** |
| JumpList output is injection-safe and side-effect-free except the reference response behavior | http-only `.i2p` validation rejects CRLF/HTML/script/schemes/userinfo/fragment; `escape_html` on target + jump URLs; absent→empty 502, valid→exact 502 + `#jumplinks`; purity static guard | **pass** |
| actual production changes remain within the I2PControl-local path budget | §2 path list; `m062` `is_authorized_m142_path`; no core/dependency change | **pass** |
| promoted rows have end-to-end evidence | fake-SAM HTTPS selection + loopback jump render + restart; `14 passed` M142 suite | **pass** |
| M061/M062/M095/M105/docs/registry are reconciled | M095 `329/36/475` recomputed; `m095`/`m105`/`m142` green; `AGENTS.md`, registry, README, both roadmaps, all three docs state `329/36/475` and partial support | **pass** |
| no high/medium proxy-routing or HTTP security defect remains | invariant/failure/compatibility reviews below; only low historical-drift findings | **pass** |

## 5. Invariant review

- Exact pinned names/types/presence preserved; two `apply` promotions only,
  zero `not_applicable` moves, no new field/alias/status/method/tunnel-type;
- No fabricated/accept-inert support: every `apply` changes real behavior
  (SSL selection routes CONNECT/`https` clearnet to the selected I2P
  destination; JumpList renders helper links on plain-`.i2p`
  destination-not-found); N/A cells retain affirmative non-applicability proof;
  Streamr datagram contract untouched;
- Unsupported values keep fail-before-allocation/no-effect behavior;
- No direct-clearnet, outproxy, trusted-peer, Streamr isolation, loopback
  confinement, bounded admission/tasks/timers, transactional lifecycle,
  last-known-good, lock discipline, secret/key/path redaction, LeaseSet
  crypto/scope, or feature/runtime isolation change beyond the additive
  HTTP-client branches;
- M061/M062 exact-path/dependency evidence amended only for M142
  planning/test/docs plus the two production files; no broad prefix waiver;
  Yosemite exact pin and optional `yosemite-i2pcontrol` ownership preserved;
- Full Proposal 170 support not claimed; partial-support wording retained
  everywhere.

## 6. Failure, cancellation, restart, and contention review

- SSL resolve/connect/handshake inherits the existing bounded
  Yosemite/session/stream timeout contract; timeout and bind/connect errors
  map to per-connection 403/502 with no retry loop (single selected-proxy
  attempt per request);
- cancellation of the tunnel generation cancels/drains accepted tasks under
  existing bounded task-group semantics (untouched `run_client_listener`;
  no new task/timer/queue);
- failed resolve/connect closes only that accepted connection and records
  `lastFailed`; generation state untouched (proven by 502 tests with no
  accept on failure and by malformed-option test that `validate_start` fails
  while `inspect` stays `Stopped`, never stale `Running`);
- restart/edit failure preserves last-known-good: `validate_start` (pure, no
  allocation/I/O) gates `start`; `config()` validates before
  destination/store/session work; supervisor `reserve` happens only after
  validation, so a bad edit never replaces the running generation (existing
  M120 transactional order retained);
- no `Mutex`/`parking_lot` guard held across destination resolution, SAM/I2P
  stream connection, request forwarding, or response body I/O
  (`select`/`note_result` are short-lived map mutations only).

## 7. Compatibility, migration, and security review

- No public API version, method, tunnel type, or action change; `Get` returns
  the exact persisted strings via raw config; no wire/schema change;
- No durable-store migration; existing definitions without the keys default to
  no-SSL/empty-jump (backward compatible); no dependency change;
- No migration or rollback operation required (additive strings + data-plane
  branches);
- No secret material in selection/tests/docs/guards (usernames appear only as
  existing proxy-auth inputs; passwords redacted as `***`; cache keys are
  `host:port`; raw config carries only the two non-secret strings);
- No lock across I/O or crypto; no new network/filesystem/DNS/TLS/handshake
  path beyond the bounded I2P `connect_to` and loopback listener;
- Containment: preferred production ownership remains
  `emissary-cli/src/i2pcontrol/**`; M142 authorizes exactly the two backend
  files; deferred M143-M152 path budgets remain non-executable until
  registered (no registered successor exists after M142; M143 still needs its
  exact-file amendment).

## 8. Documentation and operations

Machine authorities updated: `095-full-support-matrix.toml` (`329/36/475`,
SHA `e5fe8e2b28103bbceea3a92aa58c2e296c98337ce5c5df4625abb26f791a3576`),
`110-completion-ledger.toml` (`post_m142` 2 cells), `m095` expects
`329/36/475` with M142 ownership/disposition checks, `m105` subtracts two
M142 cells, `m062` gains `is_authorized_m142_path`, new `m142` guard owns
promotion/residual/validation/containment checks. Support docs
(`proposal-170-support.md`, `tunnel-manager.md`, `tunnel-backends.md`),
`AGENTS.md`, registry, implementation README, and both residual/full-support
roadmaps agree on `329/36/475`, partial support, M142 closure, no registered
successor, and the 36-cell residual inventory. Operational impact: none beyond
the two additive HTTP-client options; absent values preserve prior behavior.

## 9. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | `cargo clippy --tests` reports pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60` | none on M142 files (clean via `#[allow]` for the 8-arg handler) | record, separate corrective if needed |
| low | `m126`/`m127`/`m128`/`m129`/`m130`/`m139`/`m140`/`m141` suites assert old `325`/`327`-era counts/wording and fail on the M142 `329/36/475` head | none on M142 behavior; those closures are immutable history (M139 remains runtime/security authority, M140 remains streaming-applicability authority, M141 remains unique-local authority) | record as historical drift; future requalification may rebase those suites the way M139 rebased M126-M130 |
| low | Remaining 36 blocked cells (10 SigType, 15 LeaseSet crypto/lookup/auth, 1 Profile, 4 UseSSL, 4 UseOutproxyPlugin, 2 MultiHoming) | partial Proposal 170 support remains; no M142 scope expansion | separate deferred M143-M152 clusters; no successor registered by this closure |
| info | IPv6 literals and public-IP clearnet hostnames route via the selected I2P outproxy like any clearnet name; no direct path exists | no security issue; I2P-only routing preserved | documented; no action |

No high/medium correctness defect remains.

## 10. Registry updates

Applied alongside this closure:

- `142-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `plans/registry.md`: M142 → closed as complete (`329/36/475`); handoff M142
  closed with no registered successor; residual table `38` → `36`
  (SSLProxies/JumpList removed); execution chain updated; M143 constraint
  retained (amendment with exact neutral files still required before its
  registration);
- `plans/implementation/.../README.md`: handoff M142 closed, no registered
  successor; authority/M095/counts/residual inventory updated;
- residual + full-support roadmaps: M142 closed (`329/36/475`); M143 retained-set
  frozen (amendment still pending); counts/inventory/registration discipline
  updated;
- `110-completion-ledger.toml`: `post_m142` 2 cells added; matrix SHA recorded.

## Future-plan unblock determination

M142 unblocks no future plan for execution:

- **No plan registered**: M143 hard-depends on M142 + M140, but its required
  pre-registration amendment (exact neutral streaming files for retained
  `Profile:client` × 1) is still missing. M142 does not satisfy that
  amendment, so M143 remains deferred/unregistered. M144-M152 remain
  deferred/unregistered behind M143.
- No other future-plan status required a change beyond the M142-closed handoff.
  The sole next step is for a maintainer to amend M143 with exact files and
  explicitly register it; file presence alone never authorizes production work.

## Internal-only / read-only-upstream attestation

- External sources (Proposal 170 text at `https://i2p.net/proposals/170-i2pcontrol-expansion.txt`,
  Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`
  (`I2PTunnelHTTPClientBase.java` SSL/jump branches and
  `I2PTunnelHTTPClient.java` CONNECT/jump branches, `TunnelConfig.java`
  setters), Java I2PControl PR head
  `45bb593000408071dd376b78848fdc246dccd964`, Yosemite revision
  `59140a2277bf296928d2e8ce39a148182eeff044`, all fetched via read-only
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
