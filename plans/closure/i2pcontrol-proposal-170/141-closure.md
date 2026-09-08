# M141 Closure — HTTP Unique Local Source Address Completion

Status: **closed as complete**

Date: `2026-09-08`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/141-http-unique-local-source-address-completion.md`
  (Status now `closed as complete` with closure link).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## 1. Closure decision

M141 is closed as complete. It is a capability / local-network confinement
milestone with a maximum 2-cell promotion budget, both cells promoted. The
current matrix is now exactly:

- `327 apply`;
- `38 blocked_primitive`;
- `475 not_applicable`;
- `840` total cells.

Promoted cells (exact observable source-bind effect proven end-to-end):

- `UniqueLocalAddressPerClient:httpserver`;
- `UniqueLocalAddressPerClient:httpbidirserver`.

No high- or medium-severity correctness or local-network security issue
remains. Full Proposal 170 support remains partial.

## 2. Reviewed commits and scope

Planning baseline (per M140 closure, M141 hard dependency):

- `36cc18601bb1c7e6707d79d792d59428753f97c8`.

Implementation starting point (HEAD before M141 edits):

- `36cc18601bb1c7e6707d79d792d59428753f97c8`.

Implementation plus closure land in a single commit on the current branch
(see git log for the M141 closure commit). The reviewed production-behavior
baseline extends the lifecycle head `e4f217cb1459e26bf011da46b67fc2c83cd192b5`
(M095 `current_production_head`, retained; M141 adds the I2PControl-local
accepted-stream source-bind owner without touching core/router/transport).

Changed paths (I2PControl-local only; no `emissary-core/**`, Cargo/dependency,
Yosemite, NetDB, transport, crypto, startup tunnel, frontend, or `.github/**`
production change):

- `emissary-cli/src/i2pcontrol/backends/http_server.rs` (single derivation
  owner `derive_unique_local_source`, `unique_local_enabled` typed extraction,
  `connect_to_target` TcpSocket bind-before-connect, `unique_local` config,
  shared `make_accepted_handler` extension, `SUPPORTED` promotion with
  boolean fail-before-allocation);
- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` (composite
  `unique_local` config, shared-handler pass-through, `SUPPORTED` promotion;
  no second derivation algorithm);
- `emissary-cli/tests/m141_unique_local_source.rs` (new; six integration
  guards: matrix promotion, residual delta, boolean validation/round-trip,
  non-target rejection, shared-helper containment, docs/registry agreement);
- `emissary-cli/tests/m095_full_support_matrix.rs` (M141 apply expectations,
  `327/38/475` counts);
- `emissary-cli/tests/m105_residual_option_audit.rs` (subtract two M141 cells);
- `emissary-cli/tests/m062_dependency_containment.rs` (exact
  `is_authorized_m141_path` for the two production files plus M141
  planning/test/docs paths);
- `emissary-cli/src/i2pcontrol/backends/http_server.rs` unit tests (12
  focused tests: IPv4/IPv6 byte layout, determinism, stateless repeat,
  boolean validation, loopback confinement, disabled ordinary path, IPv4
  observable source, header independence, IPv6 bounded failure, malformed
  generation safety);
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
  (`327/38/475`, UniqueLocal row `M141` / `apply_or_not_applicable`);
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml`
  (new `post_m140` history note plus `post_m141` 2-cell record);
- `plans/implementation/i2pcontrol-proposal-170/141-http-unique-local-source-address-completion.md`
  (Status `registered` → `closed as complete`);
- `plans/implementation/i2pcontrol-proposal-170/142-httpclient-sslproxies-and-jumplist-completion.md`
  (Status `deferred` → `registered / dependency-ready`; hard dependency M141
  satisfied, readiness seams verified, no amendment required);
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `AGENTS.md`;
- `docs/i2pcontrol/proposal-170-support.md`, `docs/i2pcontrol/tunnel-manager.md`,
  `docs/i2pcontrol/tunnel-backends.md`;
- new `plans/closure/i2pcontrol-proposal-170/141-closure.md` (this file).

`git diff --name-only` proves no `emissary-core/**`, `emissary-util/**`,
manifest, lockfile, Yosemite, frontend, or workflow path changed. `git diff
--check` passes.

## 3. Requirement evidence

### Pinned reference (all accessed read-only)

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (Server options list names `UniqueLocalAddressPerClient` boolean; no
  per-IP-family partition);
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`:
  `apps/i2ptunnel/java/src/net/i2p/i2ptunnel/I2PTunnelServer.java`
  `PROP_UNIQUE_LOCAL = "enableUniqueLocal"`, `getSocket(Hash from, ...)`:
  `unique && remoteHost.isLoopbackAddress()` → IPv4 `addr[0]=127` +
  `copy hash[0..3] into addr[1..4]`, else `addr[0]=(byte)0xfd` +
  `copy hash[0..15] into addr[1..16]`, `InetAddress.getByAddress(addr)`,
  `new Socket(remoteHost, remotePort, local, 0)`; otherwise ordinary
  `new Socket(remoteHost, remotePort)`; non-loopback targets fall back to
  ordinary (M141 explicitly does not use this to broaden confinement);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  (parser/creator reachability unchanged; no new applicability gate);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (transport-only; no M141 dependency change).

### Exact derivation frozen before coding

- IPv4: `127.<hash[0]>.<hash[1]>.<hash[2]>` byte-for-byte, including `.0`,
  `.255`, repeated addresses, collisions allowed as reference allows;
- IPv6: `fd` + first 15 hash bytes (`octets[0]=0xfd`,
  `octets[1..]=hash[..15]`); trailing `hash[15..]` ignored (proven by unit
  test that mutating index 14 changes the address while mutating 15/16/31
  does not).

### Pre/post matrix rows and counts

Pre-M141 (M140-qualified): `325/40/475` (matrix SHA before M141 edits is the
M140 SHA `dd77613fb302b8bd04f42c8d1fe702b6c8c8920307776e40c4a8d8b1e8e0c44d`).
Post-M141: `327/38/475` (matrix SHA
`528440dcabed526ee2d57782f5c55d9a92afbd04a58bcaea3643d168afe98def`).
Recomputed from cells: `total=840 apply=327 blocked=38 na=475`; declared
counts match recomputation.

Changed cells (only these two; no other cell changed):

- `UniqueLocalAddressPerClient:httpserver` `blocked_primitive -> apply`;
- `UniqueLocalAddressPerClient:httpbidirserver` `blocked_primitive -> apply`.

Row now owns `completion_owner = "M141"`,
`current_or_planned_disposition = "apply_or_not_applicable"`, no
`blocked_primitive`/`blocking_milestone`, cells `[8]=apply [9]=apply` with
M141 source-bind notes; all other UniqueLocal cells remain `not_applicable`.

### Validation and persistence before allocation

- TunnelManager canonical validation already enforces boolean type for
  `UniqueLocalAddressPerClient` (`validate_boolean`); malformed types fail at
  the control plane before backend allocation;
- `httpserver::validate_raw_options` and `http_bidir::validate_raw_options`
  promote the key to `SUPPORTED` and call `unique_local_enabled` so
  non-boolean values fail with `Internal("... UniqueLocalAddressPerClient is
  invalid")` before any runtime reservation; `config_without_destination`
  extracts the bool (absent/false = disabled, true = enabled) before
  destination/store/session work;
- `validate_start` reuses the exact same helpers as `start`, so preflight and
  start cannot drift; `start` validates before identity lookup and supervisor
  reservation;
- non-target families (`server`, `ircserver`, all clients, Streamr) reject any
  supplied value (including explicit `false`) via their existing
  `UnsupportedOption` raw gates before allocation (proven by
  `m141_non_target_families_reject_before_allocation` for server/ircserver;
  other families share the same unknown-key rejection shape);
- `Get`/persistence round-trip preserves the exact boolean via raw config
  (proven by JSON round-trip test); `TunnelDefinition` Debug redacts raw
  values and never logs peer hashes (`TrustedPeerIdentity` Debug redacts both
  fields; `PostPeerKey`/`PeerKey` use fixed 32-byte hashes, never text).

### Target confinement and trusted-peer boundary

- Target host remains literal loopback under existing
  `normalize_loopback_target` (`127.0.0.1`/`localhost` → V4 loopback, `::1`
  → V6 loopback; all else rejected). Hostile configured
  `10.0.0.1`/`192.168.1.1`/`8.8.8.8`/`example.i2p`/`example.com`/`127.0.0.2`
  stay rejected even with the option enabled (unit test);
- derivation uses only `peer.canonical_id()` (`[u8;32]` from structurally
  validated `TrustedPeerIdentity::from_stream`/`from_destination_text`);
  malformed/non-canonical SAM text is rejected at that boundary before any
  derivation/connect (existing `peer_identity_impl` guards plus M141 header
  test);
- HTTP headers/usernames cannot influence the source (integration test sends
  `X-Forwarded-For`/`Forwarded`/`x-i2p-destb64` and still observes the
  peer-derived source);
- no DNS lookup added (static guard asserts no `to_socket_addrs`/
  `getaddrinfo` in either backend); no listener bound; no persistent address
  registry (per-connection `IpAddr` only); no peer Destination/hash in logs
  beyond existing sanitized policy;
- `false` is explicit disabled; `httpbidirserver` reuses the same
  `make_accepted_handler` (static guard proves no second `0xfd`/derivation in
  `http_bidir.rs`); unsupported families remain fail-before-allocation/N/A
  per M095.

### OS/runtime source-bind evidence

- IPv4 `127/8` binds portably on Linux without assignment (proven:
  `127.1.2.3`/`127.255.255.255`/`127.0.0.1` all bind OK; `lo` carries
  `127.0.0.1/8`). Observable integration fixture binds `127.0.0.1:0`,
  drives `handle_http_stream(..., unique_local=true)` with a fixed peer, and
  asserts the listener-observed `peer_addr().ip()` equals
  `derive_unique_local_source(peer.canonical_id(), 127.0.0.1)` and differs
  from the ordinary `127.0.0.1` source. Disabled path proves the ordinary
  source (`127.0.0.1`) with the same harness;
- IPv6 ULA `fd...` requires an OS-assigned address (same as Java). On default
  loopback (`::1/128` only) the exact bind fails with `EADDRNOTAVAIL`
  (errno 99, proven for `fd00::1` and a full ULA). With `sudo ip addr add
  fd00::141/128 dev lo` the same bind succeeds (proven), then the address was
  removed. M141 implements the exact Java bind-or-fail semantics with no
  fallback: unassigned ULA fails closed with `502 Bad Gateway` and the target
  sees no connection (unit test that probes bindability and, when unbound,
  asserts 502 plus no accept). This matches Java, which would throw from
  `new Socket(..., local, 0)` on the same platform. Closure therefore promotes
  both cells with IPv4 observable proof plus exact IPv6 derivation proof and
  documents the ULA-assignment requirement (identical to the reference);
  no partial-support claim is made beyond that documented platform condition.

### Verification outcomes

| Command group | Result |
|---|---|
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --test m141_unique_local_source --test m061_containment --test m062_dependency_containment --no-fail-fast` | **pass**: `40 passed (5 suites)`; matrix `327/38/475`; M141 promotion/residual/containment checks green |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `816 passed` (includes 12 new M141 unit tests) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `git diff --check` | **pass** |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide (unchanged unrelated files); all four M141-touched/added test files are individually stable-rustfmt-clean after `rustfmt --edition 2021` on those files only; no unrelated normalization |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **evidence only**: `0 errors, 1 warning` — sole warning is the known pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60`; M141 `too_many_arguments` resolved via explicit `#[allow]` on the 8-arg handler; all M141 files otherwise clean |

Full `cargo test -p emissary-cli --features i2pcontrol` (all suites) retains
seven expected historical failures independent of M141, all count/wording
drift (not behavioral regressions):

- `m126_requalification`, `m127_token_lifetime`, `m128_jsonrpc_batch`,
  `m129_nonloopback_tls`, `m130_post_corrective_requalification` assert the
  old `325` apply count / `325/47/468` wording;
- `m139_post_lifecycle_requalification::current_matrix_is_exhaustive_and_residuals_are_exact`
  asserts the M139-head `325/47/468` blocked set;
- `m140_residual_streaming_applicability::m140_matrix_reconciliation_is_exact_and_contained`
  asserts the M140-head `325/40/475` counts and blocked-subset containment.

M139/M140 closures are immutable history; M141 supersedes them for counts
while M139 remains the runtime/security qualification authority. Recorded
here as historical drift, not M141 regressions. No M141-required suite fails.

### Production-path diff (bounded)

`git diff --name-only` plus untracked-file inspection proves only the
authorized I2PControl-local paths changed (see §2). No `emissary-core/**`,
`emissary-util/**`, manifest, lockfile, Yosemite, frontend, or workflow file
changed or added. `110-completion-ledger.toml` gains `post_m140` history plus
`post_m141` (2 cells). Current unsupported values keep their fail-before-
allocation/no-effect behavior (Streamr datagram limits — 16-subscriber, 60s
expiry, 1200-byte payload, 4095-byte buffer, 15s refresh, bounded shutdown,
loopback-only UDP — untouched; remote datagrams never choose a local UDP
destination).

## 4. Requirement-to-evidence matrix

| Plan requirement (§12) | Evidence | Result |
|---|---|---|
| both target family gates are exact | `SUPPORTED` promotion only for `httpserver`/`httpbidirserver`; all other families reject via existing `UnsupportedOption` gates; `m141_non_target_families...` plus hostile-target tests | **pass** |
| enabled behavior source-binds the reference-derived address before local loopback connect | `derive_unique_local_source` byte-for-byte Java layout; `TcpSocket::bind(source,0)` then `connect` with `CONNECT_TIMEOUT`; IPv4 observable `peer_addr` test; IPv6 derivation + bind-or-fail test | **pass** |
| disabled behavior is unchanged | absent/`false` → ordinary `TcpStream::connect`; `unique_local_disabled...` test proves `127.0.0.1` source and 502-only-on-filter-failure | **pass** |
| target confinement and trusted-peer boundary unchanged or stricter | `normalize_loopback_target` untouched; hostile-host rejection test; `canonical_id`-only derivation; header-independence test; no-DNS/listener/registry guards | **pass** |
| no persistent/unbounded peer-address state | per-connection `IpAddr` return; no map/cache/timer; `PostLimiter`/`RuntimeMap` bounds untouched; no lock across bind/connect | **pass** |
| deterministic and observable integration tests pass | 12 lib unit tests + 6 integration guards; `40 passed (5 suites)` | **pass** |
| actual changed paths stay within the authorized I2PControl-local budget | §2 path list; `m062` `is_authorized_m141_path`; no core/dependency change | **pass** |
| M061/M062 exact bookkeeping updated without broadening unrelated allowances | `m062` gains exact `is_authorized_m141_path` (two production files + M141 planning/test/docs); `m061` manifest unchanged (no non-policy core path); prohibited patterns still enforced | **pass** |
| M095/M105/docs/registry counts match promoted cells | M095 `327/38/475` recomputed; `m095`/`m105`/`m141` green; `AGENTS.md`, registry, README, both roadmaps, all three docs state `327/38/475` and partial support | **pass** |
| no high/medium correctness or local-network security issue remains | invariant/failure/compatibility reviews below; only low historical-drift findings | **pass** |

## 5. Invariant review

- Exact pinned names/types/presence preserved; two `apply` promotions only,
  zero `not_applicable` moves, no new field/alias/status/method/tunnel-type;
- No fabricated/accept-inert support: every `apply` changes real behavior
  (source bind); N/A cells retain affirmative non-applicability proof;
  Streamr datagram contract untouched;
- Unsupported values keep fail-before-allocation/no-effect behavior;
- No direct-clearnet, outproxy, trusted-peer, Streamr isolation, loopback
  confinement, bounded admission/tasks/timers, transactional lifecycle,
  last-known-good, lock discipline, secret/key/path redaction, LeaseSet
  crypto/scope, or feature/runtime isolation change beyond the additive
  source-bind branch;
- M061/M062 exact-path/dependency evidence amended only for M141
  planning/test/docs plus the two production files; no broad prefix waiver;
  Yosemite exact pin and optional `yosemite-i2pcontrol` ownership preserved;
- Full Proposal 170 support not claimed; partial-support wording retained
  everywhere.

## 6. Failure, cancellation, restart, and contention review

- Source-bind/connect inherits `CONNECT_TIMEOUT` (5s); timeout and bind/connect
  errors map to per-connection `502 Bad Gateway` with no retry loop (existing
  handler has no retry policy);
- cancellation of the tunnel generation cancels/drains accepted tasks under
  existing bounded task-group semantics (untouched `run_accepted_server`;
  no new task/timer/queue);
- failed bind/connect closes only that accepted connection; generation state
  untouched (proven by 502 tests with no accept on failure and by malformed-
  option test that `validate_start`/`start` fail while `inspect` stays
  `Stopped`, never stale `Running`);
- restart/edit failure preserves last-known-good: `validate_start` (pure, no
  allocation/I/O) gates `start`; `config_without_destination` validates the
  boolean before destination/store/session work; supervisor `reserve` happens
  only after validation, so a bad edit never replaces the running generation
  (existing M120 transactional order retained);
- no `Mutex`/`parking_lot` guard held across socket bind/connect or relay
  (`PostLimiter::allow` releases before connect; `RuntimeMap` locks are
  short-lived around map mutation only).

## 7. Compatibility, migration, and security review

- No public API version, method, tunnel type, or action change; `Get` returns
  the exact persisted boolean via raw config; no wire/schema change;
- No durable-store migration; existing definitions without the key default to
  disabled (backward compatible); no dependency change;
- No migration or rollback operation required (additive boolean + data-plane
  branch);
- No secret material in derivation/tests/docs/guards (hashes are fixed test
  vectors; `Debug` redacts peer identity; raw config carries only the
  boolean);
- No lock across I/O or crypto; no new network/filesystem/DNS/TLS/handshake
  path beyond the bounded loopback `TcpSocket` bind/connect;
- Containment: preferred production ownership remains
  `emissary-cli/src/i2pcontrol/**`; M141 authorizes exactly the two backend
  files; deferred M142-M152 path budgets remain non-executable until
  registered (M142 becomes registered by this closure; M143-M152 stay
  deferred).

## 8. Documentation and operations

Machine authorities updated: `095-full-support-matrix.toml` (`327/38/475`,
SHA `528440dcabed526ee2d57782f5c55d9a92afbd04a58bcaea3643d168afe98def`),
`110-completion-ledger.toml` (`post_m141` 2 cells; `post_m140` history note),
`m095` expects `327/38/475` with M141 ownership/disposition checks, `m105`
subtracts two M141 cells, `m062` gains `is_authorized_m141_path`, new `m141`
guard owns promotion/residual/validation/containment checks. Support docs
(`proposal-170-support.md`, `tunnel-manager.md`, `tunnel-backends.md`),
`AGENTS.md`, registry, implementation README, and both residual/full-support
roadmaps agree on `327/38/475`, partial support, M141 closure, M142
registration, and the 38-cell residual inventory. Operational impact: IPv6
ULA targets require an OS-assigned ULA (as in Java); otherwise those
connections fail closed with 502 (documented).

## 9. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | `cargo clippy --tests` reports pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60` | none on M141 files (clean) | record, separate corrective if needed |
| low | `m126`/`m127`/`m128`/`m129`/`m130`/`m139`/`m140` suites assert old `325`-era counts/wording and fail on the M141 `327/38/475` head | none on M141 behavior; those closures are immutable history (M139 remains runtime/security authority, M140 remains streaming-applicability authority) | record as historical drift; future requalification may rebase those suites the way M139 rebased M126-M130 |
| low | Remaining 38 blocked cells (10 SigType, 15 LeaseSet crypto/lookup/auth, 1 Profile, 4 UseSSL, 4 UseOutproxyPlugin, 2 SSLProxies/JumpList, 2 MultiHoming) | partial Proposal 170 support remains; no M141 scope expansion | separate deferred M142-M151 clusters; only M142 registered by this closure |
| info | IPv6 ULA source bind requires OS-assigned ULA; default loopback without assignment fails closed | IPv4 portable; IPv6 matches Java platform requirement; no fallback or security issue | documented; operator assigns ULA for IPv6 unique-local targets if needed |

No high/medium correctness defect remains.

## 10. Registry updates

Applied alongside this closure:

- `141-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `142-*.md` plan: Status `deferred / unregistered` → `registered /
  dependency-ready` (M141 gate satisfied, readiness seams verified — single
  I2PControl-owned HTTP client request/filter path, I2P-only resolution,
  bounded parsing/admission/cancellation, credential redaction — no amendment
  required);
- `plans/registry.md`: M141 → closed as complete (`327/38/475`); handoff M141
  → M142 registered/dependency-ready; residual table `40` → `38` (UniqueLocal
  removed; SSLProxies/JumpList marked M142 target); execution chain updated;
  M143 constraint retained (amendment with exact neutral files still required
  before its registration);
- `plans/implementation/.../README.md`: handoff M141 complete → M142
  registered; authority/M095/counts/residual inventory updated;
- residual + full-support roadmaps: M141 complete (`327/38/475`); M142
  registered; M143 retained-set frozen (amendment still pending);
  counts/inventory/registration discipline updated;
- `110-completion-ledger.toml`: `post_m141` 2 cells added (plus `post_m140`
  history note); matrix SHA recorded.

## Future-plan unblock determination

M141 unblocks exactly one future plan:

- **M142 registered**: hard dependency (M141 closed with reconciled
  `327/38/475` baseline) satisfied; readiness seams verified present with no
  amendment required. M142 becomes the sole next handoff.
- M143-M152 remain deferred/unregistered. M143's required pre-registration
  amendment (exact neutral streaming files for retained `Profile:client` × 1)
  is still required before its own registration; M141 does not satisfy it.
  No other future-plan status required a change beyond the M141→M142 handoff.

## Internal-only / read-only-upstream attestation

- External sources (Proposal 170 text at `https://i2p.net/proposals/170-i2pcontrol-expansion.txt`,
  Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`
  (`I2PTunnelServer.java` unique-local branch), Java I2PControl PR head
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
