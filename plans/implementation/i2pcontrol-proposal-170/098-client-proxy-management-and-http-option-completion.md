# M098 — Client Proxy, Management, and HTTP Option Completion

Status: **ready — corrective dependency revision after M097 blocked closure**

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- ADR-0003 application-filter/proxy boundaries;
- ADR-0004 full-support completion boundary;
- M068/M069 accepted client/CONNECT/SOCKS runtimes;
- M090/M093 retained security authority;
- M095 full-support matrix;
- `plans/closure/i2pcontrol-proposal-170/097-closure.md`.

Planning baseline: current `master` after M103 closure (`30cd8bcc9728c286b418cfb534d4f19c6b1eb4f5`).

Pinned external contract: I2P Proposal 170 revision `2026-05-20`.

Classification: capability / security / corrective dependency decomposition.

## 1. Corrective context

The original M098 plan treated successful M097 completion as a milestone-wide hard dependency. M097 has now closed **as blocked** after implementing the common options that have real supported runtime paths (`TunnelLength`, `TunnelQuantity`, typed `EncType`) and identifying the remaining missing primitives.

That closure demonstrates that the original dependency edge was too coarse. A substantial set of M098 client proxy, outproxy, local-listener authentication, HTTP privacy/filter, and potentially generation-local lifecycle behavior is owned by existing I2PControl backends and does not require the unresolved M097 shared-session, client-key, or Yosemite `SESSION CREATE` serializer primitives.

This revision therefore changes M097 from a milestone-wide hard dependency to an **interface/evidence dependency**. M097's blocked cells remain blocked. M098 MUST NOT attempt to solve them.

Why prior planning missed this: M098/M099 were prewritten before M097 executed, so dependency ordering was expressed at milestone granularity rather than at the authoritative M095 option-cell granularity. M097 closure now supplies the evidence required to split those edges safely.

## 2. Objective

Implement every M098-assigned Proposal 170 client/proxy/HTTP option cell that can be given exact runtime semantics using the already-accepted I2PControl client/proxy/filter runtimes, while reclassifying every cell that genuinely requires an unresolved primitive before production work begins.

The milestone closes only the M098-independent slice. It does not claim the remaining M097-dependent TunnelManager matrix is complete and it does not unblock M104 by itself.

## 3. Dependency model

### Hard

- M095 is closed and its machine-readable matrix exists.
- M068/M069 client, CONNECT, SOCKS, and SOCKS-IRC data planes remain accepted.
- M093 remains the current tunnel security authority.

### Interface/evidence

- M097 closure is stable evidence describing the unavailable shared-session, session-wire, destination/key-lifecycle, and private-key-import primitives.

M097 successful completion is **not** a hard prerequisite for the independent M098 slice.

### Integration order

M098 is the current dependency-ready handoff. M099 is serialized behind M098 because both edit the M095 matrix, shared option validation, and some HTTP/filter surfaces. This is an integration-order constraint, not a semantic client-before-server dependency.

## 4. Mandatory pre-code cell reconciliation

Before changing production code, audit every M098-owned `planned_apply` cell in `095-full-support-matrix.toml` against current runtime ownership and M097 closure evidence.

Each applicable M098 cell MUST become exactly one of:

1. `apply` — already operational and evidenced;
2. `planned_apply` owned by M098 — exact behavior is implementable within this plan's existing I2PControl path budget;
3. `blocked_primitive` — exact behavior requires a named unavailable primitive; ownership is transferred out of M098 to the residual blocked line and the blocking primitive is recorded;
4. `not_applicable` — supported by pinned-contract/reference semantics and a per-type rationale.

No production implementation may begin until this reconciliation is represented in the matrix and its guard passes.

Expected independent candidates include the proxy/outproxy/authentication family and HTTP privacy/filter controls. Client-management timers are included only when their pinned behavior can be implemented exactly at the existing generation-local application/session boundary. Do not approximate a router tunnel-pool operation with an unrelated TCP timeout.

## 5. Proxy/outproxy option family

Audit and, where applicable, implement:

- `ProxyList`;
- `UseOutproxyPlugin`;
- `ProxyAuth`;
- `ProxyUsername`;
- `ProxyPassword`;
- `OutproxyAuth`;
- `OutproxyUsername`;
- `OutproxyPassword`;
- `OutproxyType`;
- `SSLProxies`;
- `JumpList`.

Requirements:

- direct `.i2p`/B32 routing never falls through to local clearnet DNS;
- clearnet requests require an explicitly configured I2P-hosted outproxy path;
- invalid or absent outproxy state fails locally rather than opening a clearnet socket;
- credentials are destination-scoped, redacted, bounded, and never exposed through canonical `get`;
- non-loopback listener exposure retains the existing conservative authentication policy;
- `UseOutproxyPlugin` remains blocked/not-applicable if no real Emissary plugin/provider concept exists; this plan does not create one;
- `SSLProxies` cannot disable normal trust validation or authorize arbitrary local/LAN targets;
- `JumpList` is bounded and cannot become an unbounded retry amplifier.

## 6. Client-management option family

Audit:

- `ConnectDelay`;
- `Profile`;
- `DelayOpen`;
- `Reduce`;
- `ReduceCount`;
- `ReduceTime`;
- `Close`;
- `CloseTime`.

Rules:

- a value remains M098-owned only if its exact semantics can be implemented with existing generation-local I2PControl runtime/session ownership;
- timers must be bounded, monotonic, cancelled on stop/restart, and unable to act on a later generation;
- `Reduce*` may not pretend to resize I2P tunnel resources unless the supported session interface actually performs that operation;
- `Profile` may map only to an existing real session/profile primitive;
- options that need the unresolved client destination store, shared-session authority, or Yosemite serializer are transferred to the residual blocked line before coding.

The M097 closure already indicates that `ConnectDelay`/`Profile` may depend on missing client destination/session authority. Preserve that blocker unless current code proves a bounded exact implementation.

## 7. HTTP/privacy option family

Audit and implement the applicable client/proxy roles for:

- `AllowUserAgent`;
- `AllowReferer`;
- `AllowAccept`;
- `AllowInternalSSL` where the pinned client-side role applies.

Use the existing HTTP client filter. Do not create another parser.

Requirements:

- false privacy flags apply the adopted stripping/normalization behavior;
- true flags permit only the corresponding caller field and do not bypass mandatory proxy/I2P identity sanitization;
- direct I2P requests never inherit outproxy credentials or headers;
- `AllowInternalSSL` must not become arbitrary TLS trust bypass;
- any server-role cell discovered under this option family is transferred to the M099 independent server slice or to an explicit residual blocker, rather than being implemented twice.

## 8. Backend applicability

Apply only matrix-authorized behavior to:

- `client`;
- `httpclient`;
- `ircclient`;
- `socks`;
- `socksirc`;
- `connectclient`;
- `streamrclient`;
- the outbound/client role of `httpbidirserver` only where ADR-0003 already composes that role.

No option is accepted merely because it parses or round-trips in raw config.

## 9. Authorized production boundary

Preferred changes are restricted to existing I2PControl paths, especially:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `backends/http_client.rs`;
- `backends/connect_client.rs`;
- `backends/socks.rs`;
- `backends/socks_irc.rs`;
- `backends/irc_client.rs`;
- `backends/client.rs`;
- `backends/http_bidir.rs` only for an already-composed client role;
- `backends/filters/http_client.rs`;
- `backends/filters/proxy.rs`;
- existing generation-local runtime helpers;
- tunnel handler/domain only for typed extraction/serialization;
- focused tests, docs, matrix, registry, and roadmap records.

No new `emissary-core/**`, `emissary-util/**`, Cargo dependency/lockfile, vendored dependency, startup proxy refactor, frontend, or workflow change is authorized.

## 10. Invariants

1. Clearnet never bypasses explicit I2P outproxy policy.
2. Direct I2P never uses local clearnet DNS.
3. Non-loopback local proxy exposure remains auth-safe.
4. Credentials remain redacted and destination-scoped.
5. HTTP anonymity filtering remains non-bypassable.
6. SOCKS remains within its accepted command scope.
7. `socksirc` continues to reuse the common IRC filter.
8. Timers/resources are bounded and generation-local.
9. Every accepted applicable option changes real runtime behavior.
10. M097 blocked primitives remain blocked rather than approximated.
11. No upstream interaction occurs.

## 11. Explicit non-goals

M098 MUST NOT:

- implement `Shared`, `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions`, `NewDest`, `PersistentClientKey`, or `PrivKeyFile` unless a later separately registered plan resolves their M097 blockers;
- vendor, fork, patch, or replace Yosemite;
- add Proposal-170-shaped core APIs;
- add SOCKS BIND/UDP ASSOCIATE;
- add DCC/WEBIRC;
- create an outproxy plugin architecture;
- weaken LAN/localhost restrictions;
- implement server admission/LeaseSet semantics;
- redesign existing client data planes;
- add CI/release machinery;
- interact upstream.

## 12. Ordered work packages

A. Reconcile every M098 matrix cell and transfer genuine residual blockers before code.

B. Implement proxy/outproxy configuration and secret-safe validation.

C. Apply local listener authentication/exposure behavior to relevant generations.

D. Integrate client-management behavior only for cells proven exact with current runtime ownership.

E. Integrate HTTP privacy flags through the existing filter.

F. Apply per-backend capability tables and remove only the corresponding `planned_apply` cells after runtime evidence.

G. Reconcile `get`, edit/restart behavior, docs, matrix, registry, and M099 readiness.

## 13. Failure, cancellation, restart, and contention

- validation/auth/outproxy failures occur before bind/session allocation;
- failed outproxy connection affects only the current request/connection, not durable definition integrity;
- listener-changing edits use existing stop/start generation semantics;
- management timers are cancelled on stop/restart and cannot target a later generation;
- concurrent edit/lifecycle operations retain existing per-name serialization;
- no lock crosses network I/O, sleep, join, or cancellation wait;
- transferred blocked cells continue to fail before allocation.

## 14. Compatibility and migration

Use the existing canonical/raw option schema. Existing definitions omitting these options retain accepted defaults. Secret-bearing values remain redacted from ordinary `get` output. Compatibility aliases remain handler-edge behavior and do not redefine canonical cells.

No new persistent schema is authorized unless a specific M098-independent option demonstrably requires a bounded I2PControl-owned migration; if so, stop and register a separate plan rather than expanding this milestone silently.

## 15. Tests

At minimum:

- exhaustive revised M098 option/type applicability fixtures;
- a regression guard proving no M098-owned applicable cell remains ambiguously dependent on M097;
- direct I2P vs clearnet/outproxy routing tests;
- proxy/outproxy credential separation and redaction;
- non-loopback authentication enforcement;
- HTTP User-Agent/Referer/Accept/InternalSSL policy tests;
- no forwarding/proxy identity spoof regression;
- any admitted generation-local lifecycle timer cancellation/restart tests;
- bounds and concurrent lifecycle tests;
- HTTP bidirectional outbound composition invariants if touched;
- SOCKS/SOCKS-IRC command/filter regressions;
- feature-off/default behavior.

## 16. Verification

Run focused backend/filter tests plus:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

If the historical `m063_feature_reachability` target remains absent, record that limitation rather than inventing replacement scope.

Do not create formatter-only churn across audited core files to satisfy the existing nightly/stable rustfmt mismatch.

## 17. Documentation/static guards

Update `095-full-support-matrix.toml` with exact per-cell ownership before and after implementation. Update tunnel-manager/support docs only for proven runtime semantics.

Static guards must ensure:

- secret-bearing values never enter ordinary canonical `get` output;
- every M098-owned applicable cell is `apply` at closure;
- every transferred residual cell names a blocking primitive and is not silently accepted;
- no new lower-layer/dependency path is introduced.

Overall Proposal 170 status remains partial.

## 18. Acceptance and stop conditions

M098 closes when:

- the pre-code dependency reconciliation is complete;
- every applicable cell still owned by M098 is operational `apply` with runtime evidence;
- every excluded cell is explicitly `not_applicable` or transferred to a named residual blocker;
- no accepted security/containment invariant regresses;
- M099 can be advanced as the next current handoff.

M098 MUST stop on any option requiring a new router algorithm, unsupported Yosemite serializer primitive, unrestricted filesystem/credential behavior, new dependency, or non-I2PControl production expansion.

Closing this revised M098 slice does **not** mean the full TunnelManager option matrix is complete and does not unblock M104 while residual blocked cells exist.

## 19. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/098-closure.md` containing:

- M095 and M097 evidence used for the dependency correction;
- before/after M098 cell ownership and counts;
- exact changed paths;
- per-option/per-type runtime evidence;
- proxy/outproxy/auth/privacy behavior evidence;
- lifecycle/timer evidence for any admitted management cells;
- failure/restart/contention evidence;
- security and containment results;
- updated M095 matrix;
- residual blocked cells and their exact primitives;
- M099 registry transition;
- unresolved findings;
- internal-only/no-upstream attestation.

## 20. Internal-only rule

All writes remain internal to `eggstack/emissary`; external references are read-only. No upstream issue, PR, review, submission, merge, contribution preparation, adoption request, or maintainer contact is authorized.