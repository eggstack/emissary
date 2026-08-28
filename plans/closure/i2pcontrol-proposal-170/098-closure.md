# M098 Closure — Client Proxy, Management, and HTTP Option Completion

Status: closed as a bounded independent slice

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/098-client-proxy-management-and-http-option-completion.md`

Evidence authorities:

- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
- `plans/closure/i2pcontrol-proposal-170/095-closure.md`
- `plans/closure/i2pcontrol-proposal-170/097-closure.md`
- accepted M068/M069 client, CONNECT, SOCKS, and SOCKS-IRC runtimes

Implementation commit: `cc51a69` (`feat(i2pcontrol): complete M098 client proxy option slice`)
Review date: 2026-08-28

## 1. Disposition and dependency correction

M098 is formally closed for the client/proxy/HTTP slice that is owned by existing
I2PControl backends and filters. M097 was used as interface/evidence authority, not as a
milestone-wide hard dependency. Its unresolved shared-session, destination/key-lifecycle,
private-key-import, and Yosemite SAM serializer primitives remain blocked and were not
approximated here.

M099 is now unblocked from its integration-order dependency and is the current handoff.
M104 remains blocked because residual applicable TunnelManager cells still exist.

## 2. Changed paths

Production:

- `emissary-cli/src/i2pcontrol/backends/connect_client.rs`
- `emissary-cli/src/i2pcontrol/backends/filters/http_client.rs`
- `emissary-cli/src/i2pcontrol/backends/http_client.rs`
- `emissary-cli/src/i2pcontrol/backends/socks.rs`
- `emissary-cli/src/i2pcontrol/domain/tunnel.rs`
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`

Tests and planning/documentation:

- `emissary-cli/tests/m095_full_support_matrix.rs`
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
- `plans/implementation/i2pcontrol-proposal-170/098-client-proxy-management-and-http-option-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/closure/i2pcontrol-proposal-170/098-closure.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- `docs/i2pcontrol/README.md`
- `docs/i2pcontrol/tunnel-manager.md`

No `emissary-core/**`, `emissary-util/**`, Cargo manifest/lockfile, startup-managed
tunnel path, frontend, workflow, or upstream path changed. No store migration was added:
the M062 containment budget does not authorize a new persistence path. New canonical
outproxy passwords enter the typed redacted option boundary; legacy raw compatibility
values remain excluded from canonical response output.

## 3. M095 cell reconciliation and counts

Before implementation, the 23 M098-owned option rows contained 276 cells: 4 `apply`,
108 `planned_apply`, and 164 `not_applicable`. After implementation and reconciliation,
the same rows contain 35 `apply`, 8 `planned_apply` (the M099 server-role handoff for
the three HTTP privacy flags and their two server roles), 68 `blocked_primitive`, and
165 `not_applicable` cells. M098 owns the 35 applied cells; the 68 blocked cells were
transferred to `residual-option-line` with named primitives and blocking milestones.

The matrix guard now pins these M098 classifications and verifies that no applicable
client cell is left ambiguously `planned_apply`.

## 4. Requirement-to-runtime evidence

| Option family | Applied cells and evidence |
|---|---|
| `ProxyList` | `httpclient`, `socks`, `socksirc`, and `connectclient` parse exactly one bounded endpoint, reject empty/comma-separated lists and port zero, require an I2P destination, and fail before listener/session allocation. |
| `ProxyAuth`, `ProxyUsername`, `ProxyPassword` | The same four local proxy roles validate complete, non-empty credentials with a 255-byte field bound; non-loopback listeners still require authentication. Passwords use the typed redacted option where supplied. |
| `OutproxyAuth`, `OutproxyUsername`, `OutproxyPassword` | The same four roles keep outproxy credentials separate from local listener credentials, reject incomplete authentication, and generate authorization only for the configured outproxy path. `OutproxyPassword` is typed/redacted and canonical `get` omits both secret values. |
| `OutproxyType` | HTTP accepts `http`; CONNECT accepts `http`/`connect`; SOCKS and SOCKS-IRC accept `socks`/`socks5`. Unsupported types and type-without-`ProxyList` fail before allocation. |
| `AllowUserAgent`, `AllowReferer`, `AllowAccept` | The accepted `httpclient` path uses the existing HTTP filter. False flags normalize/strip the corresponding caller fields; true flags forward only permitted fields, with same-origin Referer validation and mandatory proxy-identity stripping. Server-role cells remain M099-owned. |

The `client`, `ircclient`, and `streamrclient` proxy cells are not applicable under the
pinned backend contract. `AllowInternalSSL` is not applicable to `httpclient` because
the accepted outbound client is HTTP-only; server-role cells remain with M099. No
Emissary-owned outproxy plugin/provider exists, so `UseOutproxyPlugin` is blocked.
`SSLProxies` is blocked pending a trust-preserving TLS-capable outbound proxy data plane,
and `JumpList` is blocked pending a bounded multi-outproxy selection/failover owner.

## 5. Routing, failure, and security evidence

- Direct `.i2p`, B32, and validated full-destination requests remain I2P-routed and do
  not invoke local clearnet DNS.
- Clearnet HTTP/CONNECT/SOCKS requests require an explicit configured I2P outproxy;
  HTTP and CONNECT resolve that outproxy through the address-book owner before dialing.
  Missing/invalid/unresolved outproxy state fails locally or returns a bounded 502 and
  cannot open a direct clearnet socket.
- Local/private/unspecified targets remain rejected by the existing filters.
- Proxy and outproxy credentials are destination-scoped, bounded, and redacted from
  backend debug output and canonical `get`; direct I2P serialization never adds
  outproxy authorization.
- Validation and authentication failures occur before listener/session reservation.
  Existing per-name supervisor generation semantics handle stop/restart; no management
  timer was admitted without an exact session owner, so no timer-cancellation claim is
  made for the blocked management family.
- SOCKS command scope and SOCKS-IRC reuse remain unchanged. No BIND, UDP ASSOCIATE,
  DCC, WEBIRC, plugin architecture, LAN-target relaxation, or trust bypass was added.

## 6. Residual blocked cells

The following M098 rows are explicit residuals, not inertly accepted options:

- `UseOutproxyPlugin`: no Emissary-owned plugin/provider concept;
- `SSLProxies`: no accepted trust-preserving TLS outbound proxy data plane;
- `JumpList`: no bounded multi-outproxy selection/failover owner;
- `ConnectDelay`: no exact control-plane destination/session lifecycle authority;
- `Profile`: no destination store and exact session/profile selection authority;
- `DelayOpen`: no generation-local destination/session opening boundary;
- `Reduce`, `ReduceCount`, `ReduceTime`: no bounded session-owned resource-reduction
  authority;
- `Close`, `CloseTime`: no session-owned idle-close operation and exact generation
  cancellation authority.

Each applicable residual cell is `blocked_primitive`, has a per-type rationale, and
fails before allocation. M097-owned common cells retain the M097 blockers and were not
reclassified by this closure.

## 7. Verification

Passed:

- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol` — 1,746 passed
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix` — 1 passed
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment` — 7 passed
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` — 19 passed
- focused HTTP, CONNECT, and SOCKS/SOCKS-IRC backend tests — 2, 4, and 22 passed
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings`
- `git diff --check`

The historical `m063_feature_reachability` target remains absent in this checkout and
was not invented or substituted. Stable and nightly rustfmt checks report existing
repository-wide differences caused by the configured nightly-only formatting options;
no formatter-only rewrite was made.

## 8. Future-plan disposition and unresolved findings

M099 is advanced from “blocked on M098 integration order only” to **ready/current
handoff**. It can reconcile and implement its server/access/throttle slice, including
the server-role HTTP privacy cells transferred by this matrix update.

M104 remains **blocked**. It still requires M099 closure and resolution of every
applicable residual `planned_apply`/`blocked_primitive` cell before live interoperability
and final reclosure. No new residual implementation plan is registered because the
missing primitives have no demonstrated bounded path under current containment.

Unresolved findings are therefore the named residual primitives above and the M097
blockers. They are recorded in the authoritative M095 matrix; no security regression or
silent acceptance was found.

## 9. Internal-only attestation

All writes were confined to the internal Emissary repository. The pinned Proposal 170
text and existing dependency/runtime evidence were read only. No upstream issue, pull
request, review, merge, adoption request, submission, maintainer contact, or contribution
channel was contacted or mutated.

Disposition: **M098 closed; applicable client proxy/auth/privacy cells applied; residual
plugin/TLS-proxy/jump-list/management cells explicitly blocked; M099 ready; M104 blocked.**
