# M146 Closure — Outproxy Provider / UseOutproxyPlugin

Status: **closed as blocked**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/146-outproxy-provider-useoutproxyplugin-completion.md`
  (Status now `closed as blocked` with closure link; hard dependency on M145
  satisfied by `plans/closure/i2pcontrol-proposal-170/145-closure.md`).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## Planning baseline

- M145 closure head `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2` (clean worktree
  verified before M146 edits).
- Current M095 matrix after M145: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M095 matrix SHA-256 `81851be26af3ed1e4d3f0526d914aaae3d061224d8b78c5cc985cd586d84b20d`
  (unchanged by this closure).

Reviewed head:

- `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2` (clean, no production delta).
  This closure lands planning/evidence records plus the required M062
  planning-path guard only (see §9). No `emissary-core/**`,
  `emissary-cli/src/i2pcontrol/**` production, Cargo, lockfile, Yosemite,
  frontend, or workflow change was made, as required by the plan's stop
  conditions.

Pinned authority (all accessed read-only):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`;
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`;
- Java I2P/I2PTunnel reference snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`;
- I2P configuration specification (read-only, fetched 2026-09-09):
  `tunnel.N.option.i2ptunnel.useLocalOutproxy=true|false`, HTTP-client-only,
  default `true`;
- Java I2PTunnel outproxy-plugin evidence (read-only search/fetch):
  `I2PTunnelHTTPClientBase.PROP_USE_OUTPROXY_PLUGIN = "i2ptunnel.useLocalOutproxy"`,
  `Outproxy.NAME = "outproxy"`, `Outproxy.connect(host, port)` via
  `ClientAppManager.getRegisteredApp`, `SOCKSServer.shouldUseOutproxyPlugin`
  default `true`, `GeneralHelper.getUseOutproxyPlugin` default `true`.

Current Proposal matrix at closure (mechanically recomputed, unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable`;
- `095-full-support-matrix.toml` SHA-256
  `81851be26af3ed1e4d3f0526d914aaae3d061224d8b78c5cc985cd586d84b20d`.

## 1. Executive finding

M146 is closed as blocked. No `UseOutproxyPlugin` cell is promoted. The four
mechanically present client/proxy cells remain `blocked_primitive`:

- `UseOutproxyPlugin:httpclient`;
- `UseOutproxyPlugin:socks`;
- `UseOutproxyPlugin:socksirc`;
- `UseOutproxyPlugin:connectclient`.

The eight other families per the M095 row remain `not_applicable`. Full
Proposal 170 status remains **partial**.

WP1 (provider feasibility / reference freeze) was executed. The reference
semantics are frozen below, and the current-code inventory proves there is no
real bounded I2P-routed local outproxy provider within the M146 path budget.
Adding one would require either a general direct-clearnet networking subsystem
(explicitly prohibited) or a dummy/registry-only provider (explicitly zero
support value). Per plan §13, M146 must stop and leave cells blocked. No dummy
provider was created to promote cells.

## 2. Reference freeze (pinned before coding)

- Canonical property: `i2ptunnel.useLocalOutproxy`, boolean, default `true`
  (I2P configuration specification; `SOCKSServer.shouldUseOutproxyPlugin`
  `Boolean.parseBoolean(..., "true")`; `GeneralHelper.getUseOutproxyPlugin`
  default `true`).
- Provider boundary: the application `ClientAppManager` registered app named
  `Outproxy.NAME` (`"outproxy"`) exposing `Socket connect(host, port)`.
  Registration is at composition/startup (single registered app; duplicate
  registration fails); lookup is via `getRegisteredApp`; calls have the
  caller's socket timeout/cancellation context.
- Selection order: when the option is `true` and a provider is registered, the
  request uses `outproxy.connect(host, remotePort)` via
  `I2PTunnelOutproxyRunner` and bypasses the ordinary `selectProxy()` /
  `ProxyList` path for that request. When the option is `false`, or when it is
  `true` but no provider is registered (`mgr == null` or
  `getRegisteredApp == null`), the tunnel falls through to the ordinary
  configured-I2P-outproxy path (`selectProxy()`; `ERR_NO_OUTPROXY` / 503 when
  no `ProxyList` is configured).
- Provider failure after selection does not fall back to `ProxyList` within
  the same request: the `usingInternalOutproxy` branch connects via the
  provider and runs the outproxy runner directly. An `IOException` from
  `connect` fails that request; there is no unbounded retry/fallback cycle.
- Protocol coverage: HTTP (`I2PTunnelHTTPClient`, including `CONNECT` via the
  same base), CONNECT (`I2PTunnelConnectClient`), and SOCKS
  (`SOCKSServer.shouldUseOutproxyPlugin`) consume the same
  `PROP_USE_OUTPROXY_PLUGIN` property and the same `Outproxy.connect` API.
  SOCKS-IRC inherits the SOCKS provider path and retains IRC filtering (the
  Emissary `SocksIrcTunnelBackend` composes the shared SOCKS runtime with the
  M066 IRC relay filter; no separate provider API exists in reference).
- Credentials: outproxy authentication (`outproxyAuth`/`outproxyUsername` /
  `outproxyPassword`, `Proxy-Authorization` for the next hop) applies only to
  the ordinary `ProxyList` (`usingWWWProxy`) path. The provider path opens a
  raw `Socket` to the local provider; no provider-specific Proposal
  credentials exist. Failures are sanitized per-connection with bounded error
  pages; no credentials/destinations leak into diagnostics beyond the existing
  redacted boundaries.

## 3. Current-code inventory (WP1 answers)

1. Is there already an in-process service/provider that can act as a local
   outproxy while preserving I2P egress? **No.** Emissary has no
   `ClientAppManager`, no `Outproxy` interface, no `Outproxy.NAME`
   registration, and no provider registry under
   `emissary-cli/src/i2pcontrol/backends/runtime/`. The only outproxy
   mechanisms are configured remote I2P destinations: the ordinary
   `ProxyList` single outproxy (`OutproxyTarget { destination, port }`,
   I2P-destination-validated, default port 4444) consumed by `http_client`,
   `connect_client`, `socks` (via shared HTTP-client helpers), and
   `socks_irc` (via the shared SOCKS runtime); plus the M142 HTTP-client-only
   `SSLProxies` bounded I2P-only HTTPS/CONNECT selector and `JumpList`
   address-helper response (pure metadata, never fetched). There is no
   loopback service, plugin host, or PortMapper equivalent that could serve
   as a provider without new networking.
2. Which request classes/protocols are covered for HTTP, CONNECT, SOCKS and
   SOCKS-IRC? **None by a provider.** Ordinary-`ProxyList` coverage exists
   for all four families (HTTP plain/CONNECT via `HttpClientRequest::parse`
   + handler; CONNECT-only via `ConnectRequest` + strict CONNECT parsing;
   SOCKS5 via `TargetRoute::Outproxy` + `connect_via_outproxy`; SOCKS-IRC via
   the same SOCKS route plus the IRC relay filter), but that is the
   configured-outproxy path, not a local provider. A provider aliasing the
   same `ProxyList` destination would have no distinct observable effect and
   would collapse the two Proposal options.
3. Can the provider be invoked through an I2PControl-local trait/registry
   without modifying router core? **Mechanically yes, but with zero support
   value alone.** A bounded registry under
   `emissary-cli/src/i2pcontrol/backends/runtime/` plus four family consumers
   plus shared filter helpers fits the plan's preferred ownership and needs
   no `emissary-core`, Cargo, Yosemite, NetDB, transport, frontend, or
   workflow change. The plan explicitly states such infrastructure without a
   real provider has zero support value and must not change the matrix.
4. What does Java do when the option is true but no provider is registered?
   **Falls through to the ordinary path.** `usingInternalOutproxy` stays
   false; the request proceeds to `selectProxy()` / configured outproxies, or
   fails with `ERR_NO_OUTPROXY` when none is configured. Emissary's current
   fail-before-allocation for any supplied `UseOutproxyPlugin` value is
   strictly more conservative (explicit unsupported) and is the correct
   blocked behavior: it never silently coerces `true` to ordinary routing
   nor accepts `false` as an inert no-op.
5. How are provider failures/fallback to configured I2P outproxies ordered?
   **Provider first when selected; no per-request fallback on provider
   failure; ordinary path only when the provider is absent or the option is
   false.** Any future provider must preserve this ordering, stay I2P-only on
   fallback, never fall back to public DNS/direct TCP, and never bypass
   protocol-specific filters (HTTP sanitizer, CONNECT-only enforcement, SOCKS
   command scope, SOCKS-IRC relay filter).

## 4. Why no safe provider fits the M146 budget

- A direct OS TCP/DNS provider (plain `TcpStream::connect` / `getaddrinfo`
  for arbitrary clearnet hosts) is prohibited for this workstream and would
  violate the cross-cutting invariants (no direct-I2P-to-clearnet fallback by
  proxy/TLS/plugin/address-helper work; literal-loopback confinement;
  explicit-I2P-outproxy-only clearnet).
- The Java local outproxy plugin is an in-JVM `Socket` provider whose
  downstream egress is outside the I2PTunnel data plane. Emissary has no
  equivalent in-process clearnet-fetch service, no plugin host, and no
  independently useful neutral owner for one. Introducing a general clearnet
  networking subsystem from the router/control process is out of scope and
  would require a plan amendment plus a full security review that M146 does
  not authorize.
- Wrapping the existing `ProxyList` destination as a "local provider" would
  not be a real provider: selection would be indistinguishable from ordinary
  routing, `true` would have no observably distinct effect, and the matrix
  promotion would be fabricated. The plan forbids promoting cells from a
  registry, parser flag, callback interface, or empty provider list alone.
- Legacy startup proxies (`--http-outproxy` / `--socks-outproxy`,
  `emissary-cli/src/config.rs`) are explicitly excluded from the M146
  production budget and are themselves configured-I2P-outproxy consumers, not
  providers. No other candidate (M142 SSL selector, JumpList, presentation
  TLS, Streamr datagram contract) is a general outproxy provider.

## 5. Stop conditions triggered

Per plan §13, M146 stops and leaves cells blocked because:

- no real safe provider exists within bounded I2PControl-local scope;
- implementation would require general direct-clearnet networking;
- a registry-only/dummy provider is the only feasible in-budget result;
- protocol-specific filters would have to be re-proven across a new provider
  boundary with no independent provider to test against.

No production code was written to work around these stops.

## 6. Blocked behavior evidence (no production change)

- All four families reject any supplied `UseOutproxyPlugin` value before
  listener/session allocation via their existing raw gates (not in
  `SUPPORTED`; servers via explicit `REJECTED`): `httpclient`
  (`http_client::validate_raw_options`), `socks`/`socksirc`
  (`socks::validate_raw_options` via `config_for`, composed by
  `socks_irc`), `connectclient` (`connect_client::validate_raw_options` via
  `config`). `validate_start` / `start` return `UnsupportedOption`
  naming only `UseOutproxyPlugin`; `inspect` stays `Stopped`; no value echo.
- Omitted `UseOutproxyPlugin` preserves the existing configured-outproxy
  behavior byte-for-byte (ordinary `ProxyList` still passes preflight;
  clearnet without an outproxy still fails at the filter with
  `clearnet target requires an I2P outproxy`; I2P targets never need an
  outproxy). No listener, session, task, timer, cache, or retry was added.
- New guard `emissary-cli/tests/m146_outproxy_provider_blocked.rs` (5 tests)
  pins: matrix `336/29/475` with the four cells `blocked_primitive`;
  residual 29-cell inventory intact; all four families reject
  `true`/`false`/string/numeric/null before allocation with `Stopped`
  inspect; ordinary `ProxyList` without the flag still passes while the same
  config plus the flag fails; ordinary clearnet without an outproxy never
  selects direct egress via `HttpClientRequest::parse`.
- Existing guards continue to pin the blocked cells: `m095` (`336/29/475`,
  M112 ownership/disposition for `UseOutproxyPlugin`), `m105` (residual
  audit subtracts only applied cells), `m142`/`m143`/`m144`/`m145` (each
  asserts `UseOutproxyPlugin:httpclient` stays blocked).

## 7. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m146_outproxy_provider_blocked --no-fail-fast` | **pass**: `5 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: matrix `336/29/475`; residual/containment green |
| `git diff --check` | **pass** |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide (unchanged unrelated files); all M146-touched files are individually nightly-rustfmt-clean; no unrelated normalization |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **evidence only**: sole error is the known pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60`; all M146 files otherwise clean |

Full `cargo test -p emissary-cli --features i2pcontrol` (all suites) retains
historical failures independent of M146, all count/wording drift (not
behavioral regressions): `m126`/`m127`/`m128`/`m129`/`m130`/`m139`/`m140`/
`m141`/`m142`/`m143`/`m144` suites assert old `325`/`327`/`329`/`330`/`334`-era
counts/wording. Those closures are immutable history; M146 supersedes none of
them for counts while M139 remains the runtime/security qualification
authority. Recorded here as historical drift, not M146 regressions. No
M146-required suite fails.

### Production-path diff (bounded)

`git diff --name-only` plus untracked-file inspection proves only the exact
budget changed (see §9). No `emissary-core/**`, `emissary-util/**`,
`emissary-cli/src/**`, manifest, lockfile, Yosemite, frontend, or workflow
path changed or added. `110-completion-ledger.toml` gains no entry (zero
promotions). `061` gains no file. Current unsupported values keep their
fail-before-allocation/no-effect behavior (Streamr datagram limits —
16-subscriber, 60s expiry, 1200-byte payload, 4095-byte transport buffer, 15s
refresh, bounded shutdown, loopback-only UDP — untouched; remote datagrams
never choose a local UDP destination).

## 8. Requirement-to-evidence matrix

| Plan requirement (§12) | Evidence | Result |
|---|---|---|
| pinned provider selection/fallback semantics are frozen | §2 freeze (default true, provider-first ordering, absent-fallback to ProxyList, failure-fails-request, shared API, SOCKS-IRC inheritance, credential scoping) | **pass** |
| a real bounded production provider exists | §3–§4 inventory proves none exists in budget and none was invented | **blocked (stop)** |
| all promoted family paths observably use it | zero promotions; no family claims provider use | **blocked (stop)** |
| no direct-clearnet fallback/DNS path exists | no provider/network path added; clearnet-without-outproxy still fails via filter; new guard pins it | **pass** |
| registry/task/state bounds and cancellation are proven | no registry/task/timer added; existing generation/cancellation semantics untouched | **pass** |
| production changes remain I2PControl-local unless amended | zero production changes | **pass** |
| machine matrix/docs/registry and containment evidence match runtime | M095 `336/29/475` recomputed; `m095`/`m105`/`m146` green; `AGENTS.md`, registry, README, both roadmaps, all four docs state `336/29/475` and partial support with M146 blocked | **pass** |
| no medium/high proxy-routing/security defect remains | invariant/failure review below; only low historical-drift findings | **pass** |

## 9. Changed paths (exact budget only)

- `plans/implementation/i2pcontrol-proposal-170/146-outproxy-provider-useoutproxyplugin-completion.md`
  (Status `deferred / unregistered` → `closed as blocked` with closure link);
- new `plans/closure/i2pcontrol-proposal-170/146-closure.md` (this file);
- new `emissary-cli/tests/m146_outproxy_provider_blocked.rs` (5 blocked-behavior guards);
- `emissary-cli/tests/m062_dependency_containment.rs` (exact
  `is_authorized_m146_path` for planning/test/docs paths);
- `plans/registry.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `AGENTS.md`;
- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md`, `docs/i2pcontrol/tunnel-backends.md`.

`git diff --check` passes.

## 10. Documentation and operations

Machine authorities unchanged: `095-full-support-matrix.toml` (`336/29/475`,
SHA `81851be26af3ed1e4d3f0526d914aaae3d061224d8b78c5cc985cd586d84b20d`),
`110-completion-ledger.toml` (no new entry; zero promotions), `061`
(no new file), `m095` still expects `336/29/475` with M112
`UseOutproxyPlugin` ownership/disposition, `m105` still subtracts zero M146
cells, `m062` gains `is_authorized_m146_path`, new `m146` guard owns the
blocked/residual/validation/containment checks. Support docs
(`proposal-170-support.md`, `tunnel-manager.md`, `tunnel-backends.md`,
`README.md`), `AGENTS.md`, registry, implementation README, and both
residual/full-support roadmaps agree on `336/29/475`, partial support, M146
blocked, no registered successor beyond the deferred M147-M152 chain, and the
29-cell residual inventory. Operational impact: none; absent values preserve
prior behavior.

## 11. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | `cargo clippy --tests` reports pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60` | none on M146 files (clean) | record, separate corrective if needed |
| low | `m126`/`m127`/`m128`/`m129`/`m130`/`m139`/`m140`/`m141`/`m142`/`m143`/`m144` suites assert old `325`/`327`/`329`/`330`/`334`-era counts/wording and fail on the M146 `336/29/475` head | none on M146 behavior; those closures are immutable history (M139 remains runtime/security authority) | record as historical drift; future requalification may rebase those suites the way M139 rebased M126-M130 |
| low | Remaining 29 blocked cells (10 SigType, 15 LeaseSet crypto/lookup/auth, 4 UseOutproxyPlugin) | partial Proposal 170 support remains; no M146 scope expansion | separate deferred M147-M152 clusters; only M147 is next (still unregistered) |
| info | Emissary rejects any supplied `UseOutproxyPlugin` value (including `false`), while Java accepts `false` as ordinary-routing and `true` without a provider as ordinary-fallback | no security issue; strictly more conservative fail-closed blocked behavior; no silent coercion or inert acceptance | documented; a future provider plan must implement the exact `false`-accept / absent-fallback semantics when a real provider exists |
| info | A future safe provider must still prove I2P-routed egress, bounded registry, timeout/cancellation, filter preservation, and credential redaction before any cell promotes | no action in M146 | carry forward verbatim into any future outproxy-provider registration |

No high/medium correctness defect remains.

## 12. Registry updates

Applied alongside this closure:

- `146-*.md` plan: Status `deferred / unregistered` → `closed as blocked`
  with closure link;
- `plans/registry.md`: M146 → closed as blocked (`336/29/475` unchanged);
  handoff M146 blocked with no registered successor; residual table stays 29
  (`UseOutproxyPlugin` retained); execution chain updated; M147 constraint
  retained (hard dep on M146 closure now satisfied but still needs its own
  algorithm/owner freeze and explicit registration decision);
- `plans/implementation/.../README.md`: handoff M146 blocked, no registered
  successor; authority/M095/counts/residual inventory updated;
- residual + full-support roadmaps: M146 blocked (`336/29/475`); M147 still
  deferred; counts/inventory/registration discipline updated;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m062`: gains `is_authorized_m146_path`; `m061`: unchanged (no production
  paths).

## Future-plan unblock determination

M146 closure satisfies the hard-dependency gate for M147 (`hard-depends on
M146 closure`), but M147 still requires its own pre-registration audit
(exact signature-type domain, security disposition, exact crypto files, and
M061/M062 exact-file authorization) plus an explicit registration decision
before execution. M146 does not register M147. M148-M152 remain
deferred/unregistered behind M147. No other future-plan status required a
change beyond the M146-blocked handoff. File presence alone never authorizes
production work.

## Internal-only / read-only-upstream attestation

- External sources (Proposal 170 text at `https://i2p.net/proposals/170-i2pcontrol-expansion.txt`,
  I2P configuration specification at `https://i2p.net/en/docs/specs/configuration/`,
  Java I2P/I2PTunnel snapshots and i2pplus commit diffs
  (`d054e129521c81f74314878b2dc1559141fd1aa0`,
  `0bc6c23ac90644f151bf8a3e241492eba2d7f8bc`,
  `7cb5dab67f99a9eb8255af45d44ea11fc25ccd62` via read-only web fetch for
  evidence, Java I2PControl Proposal-170 head
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
