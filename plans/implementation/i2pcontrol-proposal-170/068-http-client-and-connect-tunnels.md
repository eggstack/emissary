# M068 — HTTP Client and CONNECT Client Tunnels

Status: closed

Closure: `plans/closure/i2pcontrol-proposal-170/068-closure.md`.

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Authority:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

Hard dependency:

- M065 control-plane client listener/session and option-capability primitives closed.

## 1. Objective

Implement real `httpclient` and `connectclient` backends as bounded local proxies over I2P while preserving anonymity-sensitive request normalization, explicit outproxy routing, strict local/LAN target safety, and truthful runtime option semantics.

These backends remain fully I2PControl-owned. Existing startup HTTP proxy code is behavioral evidence, not lifecycle authority.

## 2. Current/reference evidence

The existing startup HTTP proxy already proves that Emissary's application layer has the dependencies needed for:

- local TCP listener ownership;
- HTTP request parsing;
- `.i2p`/B32 routing and address-book resolution;
- outbound Yosemite streaming connections;
- privacy-sensitive request-header sanitization.

The Java HTTP client reference similarly treats HTTP proxying as a privacy adapter rather than raw forwarding. The CONNECT reference is intentionally smaller: it accepts only CONNECT, strips/ignores extra proxy headers for direct I2P requests, and turns into raw relay only after successful target connection.

M068 should reuse concepts and existing dependencies but must not refactor startup services into shared ownership merely for deduplication.

## 3. Classification

Primary class: capability / security.

Types promoted on closure:

- `httpclient`;
- `connectclient`.

`httpbidirserver` remains unsupported until M070.

## 4. Hard invariants

- production logic stays under `emissary-cli/src/i2pcontrol/**`;
- no `emissary-core/**` change;
- no direct local DNS resolution for `.i2p` or arbitrary clearnet requests;
- clearnet requests route only through an explicitly configured I2P outproxy;
- localhost/LAN/router-console targets fail closed unless a separately reviewed explicit policy allows them;
- privacy-sensitive headers are sanitized deterministically;
- CONNECT accepts only CONNECT and does not forward arbitrary client proxy metadata to direct I2P destinations;
- non-loopback listener exposure follows explicit authentication/safety policy and cannot silently become an open proxy;
- relevant Proposal 170 options are applied or rejected before listener/session allocation;
- no startup HTTP proxy listener/task adoption;
- request parser limits/timeouts explicit and bounded;
- secret proxy credentials never appear in logs/errors.

## 5. Explicit non-goals

Do not:

- implement `httpserver` or `httpbidirserver`;
- implement a general-purpose clearnet HTTP proxy outside configured I2P outproxy behavior;
- implement FTP/mailto/other proxy schemes;
- add local DNS fallback;
- add browser UI/helper pages except protocol errors necessary for proxy behavior;
- create a shared startup/control-plane HTTP manager;
- add arbitrary TLS MITM/termination;
- expand I2PControl methods/fields;
- implement every Java HTTP-proxy feature if not required by Proposal 170/security correctness.

## 6. Proposed implementation surface

Expected modules:

```text
emissary-cli/src/i2pcontrol/backends/filters/http_client.rs   # or shared HTTP filter module
emissary-cli/src/i2pcontrol/backends/http_client.rs
emissary-cli/src/i2pcontrol/backends/connect_client.rs
emissary-cli/src/i2pcontrol/backends/registry.rs
```

M068 may reuse a common bounded HTTP-header utility from M067 only if M067 already closed and semantics align. M068 must not create a hard dependency on M067; both are intended to proceed independently after M065.

## 7. `httpclient` request model

### 7.1 Initial parsing and bounds

Use bounded header parsing with explicit:

- initial request timeout;
- request-line/header size/count limits;
- malformed/CTL/obs-fold rejection;
- deterministic method support;
- request-target/Host consistency validation;
- no unbounded buffering of request bodies.

Methods needed for normal HTTP proxy use should be supported explicitly. CONNECT may be delegated to the dedicated `connectclient` role if the Proposal 170 type distinction requires separate listener behavior; do not accidentally expose a second divergent CONNECT path unless intentionally tested.

### 7.2 Target classification

Classify target into:

- `.i2p` hostname requiring existing approved address-book/I2P resolution;
- `.b32.i2p` direct I2P destination;
- full destination form if existing APIs safely support it;
- clearnet hostname requiring configured I2P HTTP outproxy;
- forbidden local/LAN/router-console address.

Rules:

- never use host OS DNS to resolve `.i2p`;
- clearnet without configured I2P outproxy fails locally;
- IP-literal/localhost/private-network targets are forbidden in direct/local mode;
- an outproxy itself must be an explicitly configured I2P destination/name, not an arbitrary local TCP proxy unless Proposal 170 explicitly defines otherwise.

### 7.3 Request sanitization

Independently define anonymity policy for outgoing requests.

At minimum:

- replace or suppress original User-Agent according to Proposal 170 `AllowUserAgent`/related options;
- suppress Referer by default unless explicitly allowed and safe for same-target routing;
- suppress `From`, `Via`, `Forwarded`, `X-Forwarded-*`, `Proxy-*`, and similar client/network identity headers;
- handle `Accept*` family according to Proposal 170 options and a conservative fingerprinting policy;
- rewrite Host for direct I2P requests so local address-book aliases are not leaked where appropriate;
- preserve original Host for clearnet request forwarded through an HTTP outproxy when required for correct outproxy semantics;
- normalize Connection/hop-by-hop headers;
- strip local proxy authorization before direct destination forwarding; only separately configured outproxy credentials may be sent to outproxy;
- do not log full Authorization/Cookie/proxy credential values.

### 7.4 Address-helper/local proxy special behavior

Do not expand scope into Java-specific proxy helper UI unless the pinned Proposal 170 contract requires it. If current startup proxy has helper behavior not represented by Proposal 170 options, it is not automatically required for M068.

## 8. `connectclient` model

### 8.1 Strict CONNECT parser

Accept only a bounded CONNECT request line/header block.

Required validation:

- method exactly CONNECT case-insensitively according to HTTP method semantics;
- target host and optional/required port valid;
- no zero/out-of-range port;
- header count/size/time bounds;
- proxy authorization extracted only for local proxy authentication;
- other extra headers are ignored/stripped for direct I2P target rather than forwarded blindly;
- request body before tunnel establishment is rejected.

### 8.2 Target policy

For direct `.i2p`/B32 targets:

- resolve through I2P mechanisms;
- open Yosemite stream with destination port;
- only after stream succeeds send local `200 Connection Established` equivalent;
- then use raw bidirectional relay.

For clearnet:

- require configured I2P CONNECT/HTTP outproxy semantics explicitly supported by options;
- forward only the reconstructed CONNECT request/necessary outproxy auth, not arbitrary local proxy headers;
- no local DNS/LAN direct connect.

For localhost/private/link-local/router-console targets: fail closed.

## 9. Listener exposure and proxy authentication

Audit Proposal 170 fields such as:

- `ListenOn`/listen interface;
- `Port`/listen port;
- `ProxyUsername`/`ProxyPassword` or equivalent;
- outproxy credentials.

Policy:

- loopback listener may support no-auth only when configuration permits;
- if binding non-loopback, require a configured authentication mode unless a separately documented Proposal 170 compatibility reason exists;
- compare credentials using an appropriate constant-time primitive already available under the `i2pcontrol` feature where meaningful;
- rate-limit or delay repeated auth failures only if it can be done boundedly without creating blocking resource exhaustion;
- never include credential values in diagnostics.

If the exact proposal permits non-loopback/no-auth and project policy chooses to allow it, documentation must make the exposure explicit; default remains safe/loopback.

## 10. Option-capability matrix

Before resource allocation, explicitly disposition relevant fields for each type:

HTTP client:

- listen interface/port;
- outproxy list;
- proxy username/password;
- allow User-Agent/Referer/Accept fields;
- internal SSL policy;
- I2CP tunnel length/variance/quantity and supported session options;
- custom options.

CONNECT client:

- listen interface/port;
- outproxy list/type;
- proxy auth;
- target/default port rules;
- I2CP options;
- custom options.

Recognized relevant but unimplemented fields fail `start` before listener/session allocation.

## 11. Ordered work packages

### WP1 — bounded request/target/sanitization unit layer

Build deterministic table-driven tests for target classification and header sanitization independent of network runtime.

### WP2 — `httpclient` backend

Wire M065 local listener/session primitive, safe resolution/outproxy path, request serialization/body streaming, auth/exposure policy, lifecycle, and focused e2e fake-SAM test.

### WP3 — `connectclient` backend

Implement strict CONNECT parser, direct I2P connection, outproxy case if supported, local success/error responses, then raw relay after establishment.

### WP4 — security/adversarial tests

Test header leakage, DNS/LAN restrictions, malformed/oversized requests, proxy auth, outproxy credentials, and connection cancellation.

### WP5 — registry/docs/closure

Promote `httpclient` and `connectclient`. Leave `httpbidirserver` unsupported. Update support docs and option matrices.

## 12. Required negative tests

At minimum:

- Referer/Forwarded/Via/X-Forwarded/From/Proxy-* leakage blocked per policy;
- user-provided User-Agent replaced/blocked according to options;
- local address-book alias is not leaked in Host when canonical I2P host rewrite is required;
- `.i2p` never causes local DNS lookup;
- clearnet without outproxy fails;
- localhost, loopback IP, RFC1918, link-local, IPv6 loopback/private targets fail in direct mode;
- malformed/obs-fold/overlong request fails;
- CONNECT with GET/POST fails;
- CONNECT extra headers are not forwarded to direct I2P peer;
- CONNECT success is not sent before Yosemite connection succeeds;
- proxy auth failure cannot expose password;
- outproxy auth is not sent to direct I2P target;
- non-loopback unauthenticated exposure fails if adopted policy requires auth;
- cancellation closes local + I2P streams;
- stale task completion cannot corrupt restarted backend.

## 13. Failure, cancellation, restart, contention semantics

- malformed request: bounded local HTTP/proxy error then close;
- destination resolution/connect failure: generic local proxy error without secret/internal details;
- one connection failure does not stop listener/session;
- session fatal failure marks named backend failed;
- stop cancels listener and all current connection tasks;
- restart creates a fresh client SAM session and listener;
- auth state is per listener/backend, not global across unrelated tunnels;
- no lock held during target lookup/connect/body relay.

## 14. Compatibility and migration

No public/persistence schema change.

Persisted definitions become startable only if requested options are in implemented capability sets.

Existing startup HTTP proxy behavior/configuration is unchanged.

## 15. Verification commands

Minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_client
cargo test -p emissary-cli --no-default-features --features i2pcontrol connect_client
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
git diff --check
```

Use fake SAM/local capture proxy tests; no external clearnet/I2P service required.

## 16. Acceptance criteria

M068 may close only when:

1. M065 is closed;
2. `httpclient` local listener/session is real and lifecycle-controlled;
3. `.i2p` target routing never invokes local DNS;
4. clearnet routing requires configured I2P outproxy;
5. localhost/LAN/private/link-local targets fail closed in direct mode;
6. request parser has explicit time/size/count bounds;
7. privacy-sensitive forwarding/proxy headers are removed according to documented policy;
8. User-Agent/Referer/Accept behavior matches implemented Proposal 170 options;
9. Host/address-book alias behavior does not leak unintended local naming information;
10. proxy auth is enforced according to listener-exposure policy;
11. proxy/outproxy credentials are redacted and sent only to intended hop;
12. `connectclient` accepts only CONNECT;
13. direct CONNECT does not forward arbitrary extra client proxy headers;
14. CONNECT success is sent only after I2P connection succeeds;
15. successful CONNECT becomes raw relay only after safe establishment;
16. malformed/oversized/slow initial requests fail without resource leak;
17. relevant unimplemented options reject before allocation;
18. lifecycle stop/restart/cancellation is exact and generation-safe;
19. `httpclient` and `connectclient` replace only their own unsupported backends;
20. `httpbidirserver` remains unsupported;
21. no `emissary-core/**` production change;
22. no unjustified non-I2PControl production change;
23. feature-disabled/default and M061/M062 checks pass;
24. docs accurately record target restrictions, anonymity defaults, auth/outproxy behavior, and option matrix;
25. no CI/release/fuzz/coverage/platform expansion;
26. no upstream/third-party write/review/submission/contribution preparation;
27. no high/medium open-proxy, DNS leak, credential leak, target-confusion, or anonymity-header finding remains.

## 17. Closure evidence required

`068-closure.md` must include:

- implementation commits/changed paths;
- target classification matrix;
- request-header sanitization matrix;
- no-local-DNS/LAN negative evidence;
- proxy/outproxy auth evidence;
- CONNECT strictness/establishment evidence;
- e2e traffic and lifecycle tests;
- option-capability matrix;
- registry/support docs;
- containment/default-build results;
- security review findings;
- internal-only attestation.

## 18. Stop conditions

Stop/replan if:

- correct target routing requires changing global address-book precedence;
- implementation requires local DNS to make `.i2p` work;
- safe CONNECT cannot be isolated from general HTTP parsing;
- a core API is required;
- non-loopback proxy exposure cannot be made truthful/safe within existing options;
- a security-sensitive option would have to be silently ignored.
