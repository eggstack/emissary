# M142 — HTTP Client SSLProxies and JumpList Completion

Status: **deferred / unregistered; hard-depends on M141 closure**

Class: capability / HTTP proxy routing and presentation

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- `SSLProxies:httpclient`;
- `JumpList:httpclient`.

Historical authority:

- M131 `PB-HTTP-OUTPROXY-TLS-01` and `PB-HTTP-JUMPLIST-01`;
- M131 already reclassified the SOCKS/SOCKS-IRC/CONNECT counterparts to `not_applicable` with affirmative HTTP-only reference evidence.

## 1. Objective

Implement the two remaining HTTP-client-only Proposal fields as real HTTP proxy behavior without creating a general clearnet networking path:

1. `SSLProxies` selects configured I2P outproxy destinations for HTTPS/CONNECT-style clearnet requests using the pinned HTTP-client selection/failure semantics.
2. `JumpList` supplies the bounded HTTP address-helper/jump-server list used by the HTTP client's destination-not-found/address-helper response path.

Neither option may be accepted solely for persistence or serializer round-trip.

## 2. Hard dependency and readiness

Hard dependency:

- M141 closed and M095/current docs reconciled.

Registration also requires the current HTTP client to retain:

- a single I2PControl-owned request/filter path under `backends/http_client.rs` and `backends/filters/http_client.rs`;
- existing I2P destination resolution/connection primitives that never directly resolve/connect a public clearnet hostname;
- bounded request parsing, listener admission and generation cancellation;
- current outproxy credential redaction and proxy-auth invariants.

If these changed materially, amend the plan before registration.

## 3. Pinned semantic contract — SSLProxies

Pinned Java `I2PTunnelHTTPClientBase` behavior establishes:

- property `i2ptunnel.httpclient.SSLOutproxies`;
- list parsing from comma/semicolon/whitespace-separated configured I2P proxy names;
- no configured SSL proxy means no SSL-outproxy selection;
- one configured proxy is returned directly;
- multiple configured proxies are selected using bounded/randomized selection;
- selection is cached by target clearnet hostname;
- the last failed SSL proxy is avoided when alternatives exist;
- the SSL-outproxy list is distinct from the ordinary HTTP outproxy list;
- config is read on the active request path rather than represented as a router-global route.

M142 must preserve the semantic distinction while using Emissary's existing safe I2P destination resolution. A configured outproxy entry must resolve as an I2P destination; it must never become a direct TCP/DNS clearnet target.

### Required bounded-state adaptation

Java's cache shape is not itself a license for unbounded state. Emissary must use a bounded hostname -> selected-I2P-proxy cache with deterministic capacity/eviction or no cache if exact behavior can be preserved without one. If bounded adaptation changes observable reference semantics materially, record and resolve that before promotion.

Failure memory must be bounded (reference tracks only the most recently failed SSL proxy). A single failed proxy must not permanently poison all future requests.

## 4. Pinned semantic contract — JumpList

Pinned Java HTTP client uses `i2ptunnel.httpclient.jumpServers` in HTTP address-helper / destination-not-found response handling. It is not a SOCKS route list and not a router peer-selection input.

M142 must freeze before coding:

- accepted separators and normalization;
- maximum entry count and per-entry length for Emissary's bounded implementation;
- exact circumstances under which jump links are offered;
- whether an entry is an I2P hostname/base32 destination or a URL prefix in the pinned implementation;
- escaping/encoding rules when embedding target host/destination in generated HTTP content;
- behavior when the list is absent/empty/malformed.

JumpList must not cause the router/control process to fetch a jump service or perform direct network navigation. It is response-generation/address-helper metadata only unless the pinned runtime proves otherwise.

## 5. Initial production path budget

Expected I2PControl-local paths:

- `emissary-cli/src/i2pcontrol/backends/http_client.rs`;
- `emissary-cli/src/i2pcontrol/backends/filters/http_client.rs`;
- `emissary-cli/src/i2pcontrol/backends/filters/http.rs` only if shared HTTP-safe escaping/response helpers belong there;
- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/domain/tunnel.rs` and/or `tunnel_manager.rs` only if typed bounded storage is preferable to validated raw-config consumption;
- existing I2PControl HTTP tests plus focused M142 tests.

No `emissary-core/**`, Cargo/dependency, Yosemite, startup HTTP proxy, frontend, NetDB/transport/crypto or `.github/**` production change is authorized.

The legacy startup HTTP proxy is not to be modified to complete the I2PControl backend.

## 6. Invariants

- Clearnet target hostnames are never passed to OS DNS or direct `TcpStream::connect` from the I2PControl data plane.
- Outproxy entries resolve through the accepted I2P destination/address-book path only.
- Proxy credentials remain typed/redacted; cache keys/diagnostics contain no password/token.
- Cache/list cardinality and string lengths are bounded before allocation/effect.
- `SSLProxies` is applied only to the reference request class; ordinary I2P requests are not rerouted.
- `JumpList` cannot inject headers, HTML/script, CRLF, arbitrary schemes, or unescaped target text.
- Missing/failed outproxy selection yields the existing bounded HTTP error path, never a clearnet fallback.
- Per-generation cache/failure state is discarded on stop/restart; stale generations cannot affect successors.
- Existing HTTP request sanitization/address-helper security stays at least as strict.
- Non-HTTP M131 N/A classifications remain unchanged.

## 7. Validation and persistence

Before listener/session allocation:

- validate each supplied option's JSON type;
- enforce list/string size bounds;
- reject control characters and structurally invalid entries;
- define duplicate/case-normalization semantics from the pinned reference;
- ensure edit validation completes before a running generation is replaced;
- preserve exact Proposal get/round-trip spelling without reflecting secrets.

Do not store both typed and raw copies if that could let validation and runtime read different values. Establish one canonical runtime representation.

## 8. Work packages

### WP1 — Reference freeze

Produce a compact source table for `selectSSLProxy`, failure update behavior, JumpList parsing and HTTP error/address-helper output. Resolve the exact list/escaping semantics before production edits.

### WP2 — Bounded configuration model

Add one bounded parsed runtime representation for SSL outproxies and jump servers. Validate before start/restart allocation.

### WP3 — SSL outproxy selection

Implement target-host-scoped selection and last-failure avoidance through existing I2P-only destination connection machinery. Keep state generation-local and bounded.

### WP4 — JumpList response behavior

Implement only the pinned HTTP response/address-helper effect, using strict escaping and no network side effect.

### WP5 — Failure/cancellation/persistence tests

Exercise cache capacity, repeated proxy failures, all-proxies-failed behavior, malformed jump entries, malicious host text, restart/edit rollback and cancellation.

### WP6 — Matrix/containment/closure

Promote only cells with observable end-to-end behavior; reconcile M095/M105/docs/registry and exact M061/M062 bookkeeping.

## 9. Failure, cancellation, restart and contention

- Outproxy resolution/connect uses the existing bounded Yosemite/session/stream timeout contract; add no unbounded retry loop.
- Last-failed state changes only after an authoritative failed selected-proxy attempt.
- Cancellation must prevent a stale generation from updating successor cache/failure state.
- No mutex is held across destination resolution, SAM/I2P stream connection, request forwarding or response body I/O.
- Edit/restart failure preserves the last-known-good running definition according to existing transactionality.
- JumpList output generation is pure/bounded and does not spawn work.

## 10. Focused tests

Required:

- empty SSLProxies -> no SSL outproxy;
- one entry -> exact selected I2P proxy;
- several entries -> every selection belongs to configured set;
- last failed proxy avoided when an alternative exists;
- only one proxy failing does not panic or create an empty-random-range path;
- bounded cache does not grow under attacker-controlled Host churn;
- clearnet hostname never reaches OS resolver/direct TCP path;
- malformed/non-I2P proxy entry rejected before allocation;
- restart drops prior generation selection/failure state;
- JumpList absent -> existing error behavior;
- bounded valid JumpList -> exact safe generated response behavior;
- CRLF/HTML/script/scheme injection attempts are rejected/escaped;
- JumpList does not perform a fetch/connect;
- HTTP-only family gates remain exact;
- Get/edit/restart round-trip is deterministic.

At least one live/fake-SAM integration must prove an HTTPS-class request selects the configured I2P outproxy destination rather than the public hostname.

## 11. Matrix promotion budget

Maximum: **2 cells**.

Promote `SSLProxies:httpclient` and `JumpList:httpclient` independently. Shared parser/config work does not require both to promote together.

Post-counts are derived from the M141 closure baseline; do not hard-code a speculative matrix.

## 12. Broad verification

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 13. Acceptance criteria

M142 closes as complete only when:

- both implemented options have exact pinned HTTP-only semantics;
- no direct clearnet fallback/DNS path exists;
- state/list/cache bounds are explicit and tested;
- failure/restart/cancellation behavior is deterministic;
- JumpList output is injection-safe and side-effect-free except the reference response behavior;
- actual production changes remain within the I2PControl-local path budget;
- promoted rows have end-to-end evidence;
- M061/M062/M095/M105/docs/registry are reconciled;
- no high/medium proxy-routing or HTTP security defect remains.

## 14. Stop conditions

Stop and leave the affected cell blocked if:

- exact reference behavior requires the legacy startup HTTP proxy or router-global routing state;
- a proposed implementation needs public DNS/direct TCP fallback;
- bounded cache adaptation cannot preserve required semantics;
- JumpList semantics cannot be established from pinned source;
- credentials or target data would have to cross an unsafe diagnostic/persistence boundary.

## 15. Closure evidence required

Record pinned source excerpts/SHAs, parsed bounds, routing proof, no-clearnet-fallback tests, cache/failure tests, injection tests, changed paths, matrix deltas, containment updates, broad verification, implementation SHA, unresolved findings and M143 readiness. External access remains read-only.