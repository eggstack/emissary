# M098 — Client Proxy, Management, and HTTP Option Completion

Status: blocked on M097

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- ADR-0003 application-filter/proxy boundaries;
- ADR-0004 full-support completion boundary;
- M068/M069 accepted client/CONNECT/SOCKS runtimes;
- M090/M093 retained security authority.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` plus M095/M097 closures when dependency-ready.

Pinned external contract: I2P Proposal 170 revision `2026-05-20`.

Classification: capability / security.

## 1. Objective

Complete every M095-assigned applicable Proposal 170 client-side option across `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient`, and the outbound/client role of `httpbidirserver`, while preserving the already-accepted anonymity, local-target, outproxy, authentication, and application-filter boundaries.

M098 must not broaden a proxy command/protocol surface merely because a configuration option exists. It implements the pinned option semantics for the already-real backends.

## 2. Option classes

The exact set is the M095 matrix. Expected groups include:

### Proxy/outproxy

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

### Client management

- `ConnectDelay`;
- `Profile`;
- `DelayOpen`;
- `Reduce`;
- `ReduceCount`;
- `ReduceTime`;
- `Close`;
- `CloseTime`;
- any common client option not already closed by M097.

### HTTP/privacy

- `AllowUserAgent`;
- `AllowReferer`;
- `AllowAccept`;
- `AllowInternalSSL`;
- `BlockAccessInProxies` where client/proxy semantics apply;
- any M095-assigned HTTP-client-specific canonical key.

Do not implement server-only keys here.

## 3. Proxy/outproxy semantics

### 3.1 Direct I2P remains resolver-safe

Direct `.i2p`/B32 requests continue to use I2P/address-book mechanisms and must not fall through to local clearnet DNS.

### 3.2 Clearnet requires explicit I2P outproxy

`ProxyList`/outproxy settings may select only explicitly configured I2P outproxy destinations/endpoints according to the pinned semantics. An absent/invalid outproxy must fail locally for clearnet requests rather than creating direct clearnet access.

`UseOutproxyPlugin` may be supported only if Emissary has a real compatible plugin/provider concept established by M095. Otherwise the cell must remain blocked/not-applicable rather than inventing a plugin system.

### 3.3 Authentication

Proxy authentication values must affect the local listener before request routing. Non-loopback listener exposure must retain the existing conservative authentication rule.

- credential checks remain constant-time where applicable;
- passwords are never returned by `get` or logs;
- partial username/password config is invalid;
- edits apply at the documented listener generation boundary;
- outproxy credentials are sent only to the configured outproxy and never to direct I2P destinations.

### 3.4 Outproxy type and SSL proxy list

Implement exact selection/transport behavior from M095/reference semantics. Do not treat `SSLProxies` as permission to disable TLS validation or to connect to arbitrary local endpoints.

### 3.5 JumpList

If the pinned behavior is a routing/fallback list, bound entries/length, validate destinations, and use deterministic order/failure semantics. It must not become an unbounded retry amplifier.

## 4. Client management semantics

Implement the pinned meaning of delay/reduce/close/profile controls without new router algorithms.

Preferred implementation is I2PControl-owned lifecycle/session policy around existing backend sessions:

- `ConnectDelay` / `DelayOpen`: bounded delay before outbound connection/session open at the appropriate lifecycle edge;
- `Reduce*`: bounded idle/usage-based reduction of client tunnel/session resources only where the existing SAM/session API supports the requested state transition;
- `Close*`: deterministic inactivity/timeout close behavior owned by the client runtime;
- `Profile`: map only to an existing Yosemite/SAM profile/session option; do not add new router peer/tunnel profile algorithms.

No timer may remain alive after the owning tunnel generation stops. Durations/counts require finite lower/upper bounds.

If an option semantically requires router-level tunnel pool resizing that the current supported interface cannot request, stop that cell rather than approximating it with an unrelated application connection timeout.

## 5. HTTP/privacy option semantics

The existing HTTP client filter remains the mandatory boundary.

- `AllowUserAgent=false` strips or normalizes user-agent according to the adopted privacy policy; true permits caller value within normal parser bounds.
- `AllowReferer=false` removes Referer; true permits it.
- `AllowAccept=false` applies the adopted accept-header normalization/filtering behavior; true preserves accepted caller values.
- `AllowInternalSSL` follows the exact pinned semantics without enabling arbitrary TLS trust bypass.
- forwarding/proxy identity headers remain sanitized regardless of permissive privacy flags unless the pinned contract explicitly says otherwise.
- direct-I2P requests never inherit outproxy credentials/headers.

M098 must use existing filter modules rather than fork a second HTTP parser.

## 6. Backend applicability

Each real client backend must consume only the options M095 marks applicable. Examples:

- generic `client`: session/lifecycle controls, not HTTP proxy headers;
- `httpclient`: proxy/outproxy/auth + HTTP/privacy + applicable lifecycle/session controls;
- `connectclient`: proxy/outproxy/auth and CONNECT-relevant controls only;
- `socks`: SOCKS proxy/auth/outproxy controls; no HTTP headers;
- `socksirc`: same SOCKS controls plus existing common IRC filter; no duplicated IRC policy;
- `ircclient`: session/client-management controls and existing IRC filtering; no HTTP options;
- `streamrclient`: only datagram/session controls applicable to Streamr;
- `httpbidirserver` outbound role: reuse the accepted HTTP client behavior with outproxy disabled where ADR-0003 requires it.

`not_applicable` cells must remain explicit and tested.

## 7. Preferred authorized path boundary

Target changes stay under existing I2PControl backends/filters/runtime, especially:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `http_client.rs`, `connect_client.rs`, `socks.rs`, `socks_irc.rs`, `irc_client.rs`, `client.rs`, `streamr.rs`, `http_bidir.rs` as assigned by M095;
- `backends/filters/http_client.rs`, `backends/filters/proxy.rs`, existing IRC filter only where option behavior changes its existing policy;
- shared I2PControl runtime helpers created/accepted by M097;
- tunnel handler/domain only when exact canonical extraction/serialization needs a typed field;
- focused tests/docs/M095 matrix updates.

No new `emissary-core/**`, startup proxy adoption/refactor, dependency/vendor, frontend, or workflow change is authorized.

## 8. Invariants

1. Clearnet never bypasses explicit I2P outproxy policy.
2. Direct I2P never uses local clearnet DNS.
3. Non-loopback local proxy exposure stays auth-safe.
4. Credentials remain redacted and destination-scoped.
5. HTTP anonymity filtering remains non-bypassable.
6. SOCKS remains within its accepted command scope unless M095 proves Proposal 170 requires more.
7. `socksirc` reuses the common IRC filter.
8. Client timers/resources are generation-local and bounded.
9. Every accepted applicable option changes real runtime behavior.
10. No upstream interaction occurs.

## 9. Explicit non-goals

M098 MUST NOT:

- add SOCKS BIND/UDP ASSOCIATE merely for parity;
- add DCC/WEBIRC;
- create an outproxy plugin architecture if none exists;
- weaken LAN/localhost restrictions;
- change server admission/LeaseSet semantics owned by M099;
- add core tunnel-pool/profile algorithms;
- redesign current client data planes;
- add CI/release machinery;
- interact upstream.

## 10. Ordered work packages

A. Freeze the M098 option/type matrix subset and exact defaults.

B. Implement proxy/outproxy model and secret-safe validation.

C. Apply authentication/exposure behavior to relevant listener generations.

D. Implement client lifecycle timers/policies with cancellation and bounds.

E. Integrate HTTP privacy flags into the existing filter.

F. Apply per-backend capability tables and remove only the corresponding `unsupported` cells after real runtime evidence.

G. Reconcile `get`, edit/restart behavior, support docs, and M095 matrix.

## 11. Failure/cancellation/restart/contention semantics

- validation/auth/outproxy configuration failure occurs before bind/session allocation;
- failed outproxy connection affects only the current request/connection, not durable definition integrity;
- listener-changing edits use exact existing stop/start generation semantics;
- management timers are cancelled on stop/restart and cannot act on a later generation;
- shared/common session ownership from M097 remains authoritative;
- concurrent edit/lifecycle operations preserve existing per-name serialization;
- no lock crosses network I/O, sleep, join, or cancellation wait.

## 12. Compatibility/migration

Use the existing canonical/raw option schema where possible. Existing definitions that omit these options retain accepted defaults. Previously persisted values that were rejected at start become operational only after the exact owning cell closes.

Compatibility aliases remain handler-edge behavior and do not redefine the canonical matrix.

## 13. Tests

At minimum:

- complete M098 option/type applicability fixtures;
- direct I2P vs clearnet/outproxy routing tests;
- proxy/outproxy credential separation/redaction tests;
- non-loopback auth enforcement;
- HTTP User-Agent/Referer/Accept/InternalSSL policy tests;
- no forwarding/proxy identity spoof regression;
- ConnectDelay/DelayOpen/Reduce/Close timer cancellation/restart tests;
- bounds and concurrent lifecycle tests;
- HTTP bidirectional outbound no-outproxy invariant;
- SOCKS/SOCKS-IRC accepted command/filter regressions;
- feature-off/default behavior.

## 14. Verification

Run focused backend/filter tests plus:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
git diff --check
```

## 15. Documentation/static guards

Update the M095 matrix only for proven runtime cells. Update tunnel-manager/support docs with exact per-family option semantics and defaults. Keep overall support status partial until M104.

Static guards should ensure secret-bearing options never enter ordinary `get` output and every canonical M098-owned option is classified for all twelve types.

## 16. Acceptance and stop conditions

M098 closes only when every applicable M098 cell is `apply`, every non-applicable cell has evidence, and no security boundary regresses.

Stop if an option requires a new router algorithm, new unsupported Yosemite primitive, unconfined credential/file behavior, outproxy plugin subsystem, or non-I2PControl production expansion not already budgeted.

## 17. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/098-closure.md` with:

- M095/M097 dependency evidence;
- changed paths;
- per-option/per-type runtime matrix;
- proxy/outproxy/auth/privacy behavior evidence;
- timer/failure/restart/contention evidence;
- security regression results;
- containment results;
- updated M095 matrix;
- unresolved findings;
- internal-only/no-upstream attestation.

## 18. Internal-only rule

All writes remain internal to `eggstack/emissary`; external references are read-only. No upstream issue/PR/review/submission/merge/contribution activity is authorized.