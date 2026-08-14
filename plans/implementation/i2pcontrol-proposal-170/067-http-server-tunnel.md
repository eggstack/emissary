# M067 — HTTP Server Tunnel

Status: blocked — hard dependency on M065 closure

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Authority:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

Hard dependency:

- M065 accepted-stream server runtime + option-capability validation closed.

## 1. Objective

Implement `httpserver` as a secure-by-design accepted-stream HTTP reverse tunnel that validates and sanitizes remote HTTP requests before they reach the configured local service and filters identifying response headers before they return over I2P.

This is the highest-risk missing tunnel family. A raw byte-forwarding implementation is explicitly insufficient and must remain unsupported rather than land partially.

## 2. Security model

Remote I2P peers are untrusted HTTP clients. The local HTTP service may trust loopback/reverse-proxy metadata and may have parsing behavior different from the tunnel's parser. M067 therefore owns a strict protocol normalization boundary.

Required path:

```text
remote I2P peer
    -> Yosemite accepted stream + trusted peer identity
    -> bounded HTTP request parser/normalizer
    -> access/throttle policy
    -> local TCP target
    -> bounded HTTP response-header parser/filter
    -> remote I2P peer
```

No local target connection should be opened until the initial request line/header block is complete, valid, bounded, and policy-accepted.

## 3. Reference behavior adopted as security intent

The Java HTTP server reference demonstrates important categories of protection:

- header/line/count/aggregate bounds;
- header read timeouts;
- removal of spoofable I2P destination identity headers before trusted injection;
- Host rewriting;
- optional inproxy/Referer/User-Agent policy;
- POST/PUT throttling keyed by I2P peer;
- response stripping of server/proxy/fingerprint headers;
- connection close/upgrade handling;
- protection against proxy-header attacks.

M067 must independently implement the behavior categories required for security and Proposal 170 options. It must not copy Java source.

## 4. Classification

Primary class: capability / security.

Type promoted on closure:

- `httpserver`.

`httpbidirserver` remains unsupported until M070.

## 5. Hard invariants

- all HTTP-server-specific production logic under `emissary-cli/src/i2pcontrol/**`;
- no `emissary-core/**` change;
- use M065 accepted-stream path, never generic SAM forwarding;
- remote request bytes cannot reach local service before request headers pass the sanitizer;
- attacker-supplied `X-I2P-*` identity headers are never trusted;
- trusted I2P peer identity is derived only from accepted stream/session;
- request headers cannot select arbitrary local target host/port;
- local target defaults to loopback-safe policy;
- HTTP framing ambiguities fail closed before local connect;
- parser/resource bounds are explicit and tested;
- request body is streamed after validated headers and is not buffered unboundedly;
- response body is streamed after validated response headers and is not buffered unboundedly;
- no security-sensitive Proposal 170 option is accepted but ignored;
- server destination secret uses existing backend-owned secret store;
- lifecycle remains exact-name/generation-safe;
- no startup HTTP proxy/server ownership changes.

## 6. Explicit non-goals

Do not:

- implement `httpclient` or `httpbidirserver` here;
- add a general HTTP framework/server dependency if existing parser utilities suffice;
- proxy arbitrary protocols through HTTP server;
- implement transparent `x-i2p-gzip` optimization unless independently required for correctness;
- implement arbitrary TLS termination;
- inspect/modify application bodies beyond framing/streaming requirements;
- add WAF/content filtering unrelated to anonymity/protocol correctness;
- expose arbitrary backend/LAN routing based on request Host;
- create new core peer/session APIs.

## 7. Proposed implementation surface

Expected local modules:

```text
emissary-cli/src/i2pcontrol/backends/filters/http.rs
emissary-cli/src/i2pcontrol/backends/http_server.rs
emissary-cli/src/i2pcontrol/backends/registry.rs
```

If M068 later needs common HTTP request helpers, keep reusable pieces generic only where roles genuinely share semantics; server response filtering remains server-specific.

## 8. Request parser and normalization requirements

### 8.1 Bounds

Define explicit constants/validated configurable bounds for:

- first request-line timeout;
- header completion timeout;
- maximum request-line length;
- maximum individual header-line length/value length;
- maximum header count;
- maximum aggregate request-line + headers size;
- maximum concurrent accepted connections;
- maximum pending local target connects;
- body inactivity/failsafe timeout appropriate to GET/HEAD versus upload methods.

Reference-scale defaults such as ~8 KiB individual line, tens of headers, and tens of KiB aggregate are reasonable, but exact values must be documented and independently justified.

### 8.2 Request line

Parse method, request-target, and HTTP version without accepting ambiguous whitespace/control characters.

Initial supported methods should include normal HTTP server methods needed for compatibility. If implementation chooses a restricted method set, document/reject unsupported methods deterministically rather than accidentally raw-forwarding them.

Reject:

- embedded NUL/CTL in request line;
- invalid HTTP version syntax;
- overlong URI/request line;
- malformed absolute-form/origin-form that cannot be normalized safely.

### 8.3 Header syntax

Reject:

- header without colon;
- invalid field-name characters;
- embedded CR/LF/NUL in field values;
- obsolete line folding/continuation;
- header count/size overflow.

Normalize header-name comparison case-insensitively while preserving or emitting canonical safe spelling.

### 8.4 Framing / request-smuggling defenses

Before connecting the local backend, enforce deterministic framing rules:

- duplicate `Content-Length` accepted only if exact same numeric value and implementation deliberately chooses that compatibility rule; otherwise reject duplicates;
- conflicting Content-Length values reject;
- invalid/non-numeric/overflow Content-Length rejects;
- any `Transfer-Encoding` + `Content-Length` ambiguity rejects unless a single well-defined safe normalization is implemented;
- unsupported transfer-coding rejects;
- malformed chunked framing must not be translated into a different message for backend;
- hop-by-hop `Connection` nominated headers are removed/normalized according to role;
- proxy-specific connection headers removed.

The tunnel parser and local backend must not see materially different framing interpretations.

### 8.5 Trusted identity headers

Before any injection, remove all caller-supplied variants of the supported trusted identity names, case-insensitively.

If this backend exposes Java-compatible identity metadata to the local server, values must be generated solely from the accepted I2P peer identity, e.g.:

- destination hash;
- B32 destination;
- full public destination only if intentionally supported and bounded.

Do not allow a request to preserve a user-provided value under alternative casing/duplication.

### 8.6 Proxy/forwarding headers

Remote input must not impersonate a trusted reverse proxy.

At minimum, explicitly handle/remove:

- `Forwarded`;
- `Via`;
- `X-Forwarded-For`;
- `X-Forwarded-Host`;
- `X-Forwarded-Server`;
- `Proxy`;
- `Proxy-Connection`;
- related configured inproxy detection headers.

If `BlockAccessInProxies` requires detection, inspect these headers before stripping and reject according to policy. Otherwise do not pass attacker-controlled proxy identity to the local application by default.

### 8.7 Host policy

`WebsiteHostname`/`SpoofedHost` equivalent must drive backend Host rewriting when configured.

Safe rule:

- local target selection is entirely from tunnel definition;
- Host presented to backend is configured/policy-derived;
- incoming Host never changes target address;
- multiple/ambiguous Host fails closed;
- if no explicit website hostname is configured, use a deterministic safe host derived from server identity/config rather than trusting arbitrary authority as privileged routing metadata.

### 8.8 Referer/User-Agent policy

Implement Proposal 170 fields/options that control:

- Referer blocking/filtering;
- User-Agent blocking/filtering;
- inproxy blocking;
- optional access lists.

Where exact semantics are underspecified, document the adopted conservative behavior and test it. Unsupported modes must reject start rather than silently no-op.

## 9. Request throttling / connection limits

Implement relevant Proposal 170 server controls with trusted I2P peer identity as the key where per-client semantics apply.

At minimum audit and disposition:

- `MaxConcurrentConns`;
- per-client connection limits/windows/ban durations if present in the pinned option set;
- total connection limits;
- `PostLimit`/`PostLimitTime` and related POST/PUT throttling fields;
- access list semantics.

Requirements:

- bounded maps with expiry/eviction;
- no attacker-controlled unbounded key retention;
- monotonic time source;
- no lock held across network I/O;
- deterministic rejection response/close behavior;
- restart clears ephemeral throttling state unless Proposal 170 explicitly requires persistence.

## 10. Local target connection

Only after request headers and access policy pass:

- resolve `TargetHost` from admin configuration only;
- default/restrict to loopback according to accepted policy;
- validate target port;
- connect with timeout;
- send a freshly serialized sanitized request line/header block;
- stream body according to validated framing.

If target connect fails, send a bounded generic HTTP error over I2P without disclosing local target IP/path/internal error detail.

## 11. Response filtering

Before any local response headers are returned over I2P:

- bound response status/header read size/time;
- validate basic HTTP response header syntax;
- remove hop-by-hop/proxy-only headers according to connection semantics;
- remove at least the adopted server fingerprint headers such as `Server`, `X-Powered-By`, `X-Runtime`, `Proxy`, and `Proxy-Connection`;
- decide/document `Date` behavior consistently with anonymity/reference behavior;
- sanitize Connection/upgrade handling;
- preserve Content-Length/Transfer-Encoding framing without creating ambiguity;
- after sanitized header output, stream payload without unbounded buffering.

If websocket/upgrade is supported, filter initial request/response headers first, then switch to raw bidirectional relay only after successful safe upgrade. If not supported initially, reject upgrade deterministically.

## 12. Option-capability matrix

Before runtime allocation, create explicit disposition for all server/HTTP/I2CP Proposal 170 options relevant to `httpserver`.

At minimum cover:

- target host/port;
- website hostname/spoof host;
- access list;
- connection/throttle fields;
- referer/inproxy/user-agent filter controls;
- gzip/compression controls;
- SSL/TLS-related flags;
- custom options;
- tunnel length/variance/quantity;
- signature/encryption/LeaseSet options supported by Yosemite session configuration.

Recognized but unimplemented security/behavioral fields reject `start` before destination/session/listener allocation.

## 13. Ordered work packages

### WP1 — HTTP filter unit layer

Implement request parser/normalizer and response-header filter with exhaustive table-driven negative cases.

### WP2 — accepted-stream server wiring

Use M065 accepted-server runtime and existing secret store. Prove no generic `run_single_server`/STREAM FORWARD path is used for `httpserver`.

### WP3 — access/throttle controls

Implement the accepted option set with bounded peer-keyed state.

### WP4 — local target/body/response streaming

Wire validated headers to local service, stream bodies safely, parse/filter response headers, support or reject upgrade explicitly.

### WP5 — adversarial integration tests

Use fake SAM/accepted streams and a local capture server to prove the exact sanitized bytes received by backend and returned to peer.

### WP6 — registry/docs/closure

Promote only `httpserver`. Leave `httpbidirserver` unsupported. Update support docs and option matrix.

## 14. Required adversarial tests

At minimum:

- spoofed `X-I2P-DestHash/B32/B64` in multiple casing/duplicates cannot survive trusted injection;
- spoofed `X-Forwarded-For`, `Forwarded`, `Via`, `Proxy`, `Proxy-Connection` handling;
- duplicate/conflicting Host;
- duplicate/conflicting Content-Length;
- TE+CL ambiguity;
- invalid chunked/transfer coding;
- header without colon;
- obs-fold;
- CRLF injection/control chars;
- overlong request line/header line;
- too many/too-large headers;
- first-line/header slowloris timeout;
- backend connection not attempted for every pre-validation rejection;
- configured Host rewrite reaches backend;
- remote Host cannot change target host/port;
- per-peer connection/POST throttling works and does not affect unrelated peer;
- `Server`, `X-Powered-By`, `X-Runtime`, proxy headers stripped from response;
- backend target error returns generic remote error without local details;
- connection task cancellation closes both sides;
- restart preserves server identity but not stale connection/throttle task state.

## 15. Failure, cancellation, restart, contention semantics

Connection parser/policy failures are per connection and fail closed.

A malformed client cannot crash/stop the server supervisor.

Stop cancels acceptance and all current-generation request/target tasks; partial requests must not leave local sockets open.

Restart retains destination identity through the existing secret store and creates fresh ephemeral parser/throttle state.

Connection limit accounting must be decrement-safe on every normal/error/cancellation path.

No global HTTP-server lock serializes unrelated tunnel names.

## 16. Compatibility and migration

No public/persistence schema migration.

Existing persisted `httpserver` definitions become startable only if their options fit the implemented capability matrix. Definitions using unsupported modes remain persisted/editable but `start` fails with sanitized deterministic operation status.

`httpbidirserver` remains unsupported.

## 17. Verification commands

Minimum local verification:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_server
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_filter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
git diff --check
```

Run bounded end-to-end fake-SAM/local-backend tests. No public I2P network required for closure of parser/security semantics.

## 18. Acceptance criteria

M067 may close only when:

1. M065 is closed;
2. `httpserver` uses accepted-stream filtering, not SAM forwarding;
3. local backend is not connected before request header validation/policy passes;
4. request line/header count/line/aggregate/time bounds exist and are tested;
5. malformed/CTL/obs-fold headers fail closed;
6. request-framing ambiguity (CL conflicts, TE+CL unsafe forms) fails closed;
7. incoming spoofed I2P identity headers are removed case-insensitively;
8. any injected I2P identity derives only from trusted peer identity;
9. proxy/forwarding identity headers cannot spoof a trusted local reverse-proxy identity;
10. Host handling cannot change local target and configured website host rewriting works;
11. relevant referer/inproxy/user-agent/access controls are applied or explicitly rejected;
12. relevant connection/POST throttles are bounded and peer-keyed where required;
13. request body streams without unbounded buffering after validated headers;
14. response headers are parsed before remote exposure and adopted fingerprint/proxy headers are stripped;
15. response body streams without unbounded buffering;
16. upgrade/websocket path is either safely filtered then relayed or explicitly rejected;
17. local target failure does not disclose sensitive local details;
18. lifecycle/cancellation leaves no orphan request/local socket tasks;
19. server destination identity survives restart;
20. option-capability validation rejects unimplemented relevant fields before allocation;
21. secrets/raw sensitive headers are not emitted in errors/logs;
22. only `httpserver` replaces its unsupported backend;
23. `httpbidirserver` remains unsupported;
24. no `emissary-core/**` production change;
25. no unjustified non-I2PControl production change;
26. feature-disabled/default and containment checks pass;
27. docs list supported/rejected options and security behavior accurately;
28. no CI/release/fuzz/coverage/platform expansion;
29. no upstream/third-party write/review/submission/contribution preparation;
30. no high/medium request-smuggling, identity-spoofing, open-proxy/SSRF, or resource-exhaustion finding remains.

## 19. Closure evidence required

`067-closure.md` must include:

- implementation commits/changed paths;
- request/response filter matrix;
- exact parser/resource limits;
- pre-local-connect rejection evidence;
- identity-header spoof/injection evidence;
- framing-smuggling negative evidence;
- throttle/access evidence;
- response fingerprint stripping evidence;
- lifecycle/restart evidence;
- option-capability matrix;
- registry/support-doc changes;
- containment/default-build results;
- security review findings/disposition;
- internal-only attestation.

## 20. Stop conditions

Stop and replan if:

- safe filtering requires local backend to receive raw headers first;
- HTTP parser semantics cannot be made unambiguous with current dependencies;
- a new core socket/peer API appears necessary;
- implementing a Proposal option requires broad startup proxy refactoring;
- the only way to claim support is to silently ignore a security-sensitive option;
- a high/medium smuggling/identity/SSRF/resource finding remains unresolved.