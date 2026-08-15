# M070 — HTTP Bidirectional Server Composition

Status: closed

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Authority:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

Hard dependencies:

- M067 `httpserver` closed;
- M068 `httpclient`/`connectclient` closed.

## 1. Objective

Implement Proposal 170 `httpbidirserver` strictly as composition of the already accepted HTTP server and HTTP client runtime/filter components.

No third HTTP parser, sanitizer, request policy, response filter, or proxy implementation is authorized by M070.

The reference family is deprecated in contemporary Java I2P but remains a declared Proposal 170 tunnel type. Runtime completeness therefore requires a truthful implementation or an explicit accepted blocker. This plan assumes composition is feasible with the M067/M068 interfaces.

## 2. Required topology

Target behavior:

```text
                 one control-plane server identity/session authority
                                |
          +---------------------+---------------------+
          |                                           |
remote I2P HTTP -> M067 server filter -> local HTTP service
          |
          +-> local no-outproxy HTTP proxy role -> M068 client filter -> I2P
```

The exact SAM/Yosemite session-sharing mechanics may vary with the accepted M065/M067 implementation. The essential invariants are:

- inbound server role has exactly M067 security behavior;
- local proxy role has exactly M068 HTTP-client sanitization behavior;
- bidirectional proxy role cannot use clearnet outproxying;
- lifecycle owns/cancels both halves as one named TunnelManager backend;
- no duplicate destination/identity authority appears.

## 3. Classification

Primary class: capability / composition.

Type promoted on closure:

- `httpbidirserver`.

## 4. Hard invariants

- no new HTTP parser/filter policy implementation;
- use accepted M067 request/response filters unchanged or through explicitly exposed stable interfaces;
- use accepted M068 HTTP client request sanitization unchanged;
- outproxy behavior disabled for bidirectional local proxy role;
- one backend name owns all resources;
- one server destination identity persists across stop/restart;
- failure of one half transitions the composite backend truthfully and shuts down/isolates resources according to defined policy;
- no `emissary-core/**` change;
- all new composition logic under `emissary-cli/src/i2pcontrol/**`;
- no startup HTTP proxy/server adoption;
- option-capability validation before allocation;
- no public Proposal 170 change.

## 5. Explicit non-goals

Do not:

- add new HTTP filtering behavior that belongs in M067/M068 corrective work;
- enable clearnet outproxy from the bidirectional proxy role;
- refactor startup HTTP proxy/server ownership;
- add arbitrary shared-session framework for unrelated tunnel types;
- add new router/core session-sharing API;
- implement performance optimization beyond needed composition;
- broaden deprecated type behavior beyond the pinned Proposal 170/reference role.

## 6. Readiness audit

Before coding, inspect the closed M067/M068 interfaces and answer:

1. Can the M067 server runtime expose or compose the server session/destination authority without leaking private material?
2. Can an M068 HTTP-client connection handler operate using the same appropriate I2P session/identity context or a tightly scoped sibling session while preserving the intended bidirectional semantics?
3. Can outproxy capability be disabled structurally rather than by a fragile runtime flag ignored deep in the stack?
4. Can both halves share one cancellation/generation supervisor without changing their standalone behavior?
5. Can all required `httpbidirserver` Proposal 170 options map to already implemented M067/M068 capabilities?

If any answer requires a new core API or a new HTTP stack, stop and create a corrective architecture plan rather than forcing M070.

## 7. Composition requirements

### 7.1 Server half

Use the exact M067 accepted-stream path:

- same trusted peer identity handling;
- same header/request-smuggling protections;
- same Host/identity/proxy-header policy;
- same connection/throttle rules applicable to bidirectional server;
- same response filter.

No bypass via generic server forwarding.

### 7.2 Local proxy half

Use M068 HTTP client logic with explicit bidirectional restrictions:

- local listener interface/port from bidirectional definition;
- direct I2P only;
- no clearnet HTTP outproxy selection;
- same anonymity-sensitive header sanitizer;
- same local proxy authentication/exposure policy as applicable;
- same bounded parser/task semantics.

A test must prove a clearnet request cannot escape through configured global/startup outproxy state.

### 7.3 Identity/session semantics

Prefer a single server identity authority. Do not create a second persistent private destination merely because the client helper normally owns an independent session.

If Yosemite cannot safely reuse one session for both roles, document the exact semantics of any sibling client session and prove that it does not undermine the Proposal 170/reference bidirectional identity expectation. A material identity mismatch blocks closure and requires replanning.

Private destination material stays in the existing backend-owned secret store and is not copied into generic raw configuration.

## 8. Lifecycle semantics

Start:

1. validate composite option set;
2. obtain/validate server identity;
3. establish inbound server session/runtime;
4. establish local proxy listener/runtime;
5. report running only after both required halves are ready.

If second-half startup fails, tear down the first half before returning start failure.

Stop:

- mark composite stopping;
- cancel both halves for exact generation;
- await/abort boundedly;
- release listener/session resources;
- preserve server identity secret;
- report stopped only after both halves terminate.

Restart is completed composite stop then start.

If one half terminates unexpectedly while running, default policy should mark the composite failed and cancel the sibling half rather than report a partially working bidirectional tunnel as running.

## 9. Option-capability matrix

Create explicit `httpbidirserver` disposition by composing only capabilities already accepted in M067/M068.

At minimum cover:

- server target host/port;
- server website hostname;
- local proxy listen interface/port;
- HTTP server access/filter/throttle options;
- HTTP client anonymity options;
- proxy authentication;
- outproxy options (must be rejected/disabled for this role);
- I2CP session/tunnel options;
- unsupported TLS/compression/custom modes.

A field accepted by standalone M068 but incompatible with bidirectional no-outproxy semantics must be rejected for this type before allocation.

## 10. Ordered work packages

### WP1 — interface/readiness audit

Freeze exact M067/M068 reusable APIs and record whether any small I2PControl-local visibility/refactor is needed.

### WP2 — composite backend

Implement one `TunnelBackend` owning both halves and shared lifecycle/secret identity.

### WP3 — failure atomicity tests

Exercise first-half success/second-half failure, unexpected sibling termination, stop during start, restart, bind collision, and stale-generation completion.

### WP4 — security composition tests

Prove:

- inbound requests receive exact M067 sanitization;
- local proxy requests receive exact M068 sanitization;
- outproxy/clearnet path disabled;
- no third parser/filter path exists;
- identity persists across restart.

### WP5 — registry/docs/closure

Replace only `httpbidirserver` unsupported backend and document deprecated/composite semantics.

## 11. Required tests

At minimum:

- successful inbound HTTP server request/filtered response through composite;
- successful local proxy -> I2P request through composite;
- clearnet request rejected even if other application/startup outproxy config exists;
- spoofed inbound I2P identity header still removed exactly as M067;
- outbound privacy headers still sanitized exactly as M068;
- second-half bind failure tears down first half;
- server session failure prevents proxy-only running state;
- local proxy failure after running transitions composite failed and cancels server half;
- stop cancels both halves;
- restart retains destination identity;
- duplicate start rejected;
- relevant unsupported composite option rejected before allocation.

## 12. Compatibility and migration

No wire/persistence migration.

Previously persisted `httpbidirserver` definitions become startable only when option set fits the composite capability matrix.

Standalone `httpserver`/`httpclient` behavior must not change merely to enable composition except minimal I2PControl-local visibility/refactor with retained tests.

## 13. Verification commands

Minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_bidir
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_server
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_client
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
git diff --check
```

No public network required.

## 14. Acceptance criteria

M070 may close only when:

1. M067 and M068 are closed;
2. no third HTTP parser/filter implementation is introduced;
3. inbound half uses exact accepted M067 security path;
4. local proxy half uses exact accepted M068 anonymity path;
5. clearnet/outproxy capability is structurally disabled/rejected for bidirectional role;
6. server identity authority is singular and persistent;
7. start reports running only after both halves ready;
8. partial start failure tears down already-started sibling;
9. unexpected runtime loss of one required half cannot leave composite reported running;
10. stop/restart cancels both exact-generation halves;
11. restart retains server destination identity;
12. option-capability matrix is explicit and rejects incompatible outproxy/other fields before allocation;
13. standalone HTTP server/client tests remain green;
14. only `httpbidirserver` replaces its unsupported backend;
15. no `emissary-core/**` production change;
16. no unjustified non-I2PControl production change;
17. feature-disabled/default and containment checks pass;
18. docs describe deprecated/composite/no-outproxy behavior accurately;
19. no CI/release/fuzz/coverage/platform expansion;
20. no upstream/third-party write/review/submission/contribution preparation;
21. no high/medium filter-bypass, partial-lifecycle, identity-duplication, or outproxy-escape finding remains.

## 15. Closure evidence required

`070-closure.md` must include:

- implementation commits/paths;
- proof of M067/M068 code reuse rather than duplicated filters;
- composite startup/shutdown/failure matrix;
- outproxy-disabled negative evidence;
- destination identity restart evidence;
- option matrix;
- registry/docs changes;
- containment/default-build outcomes;
- security findings/disposition;
- internal-only attestation.

## 16. Stop conditions

Stop/replan if:

- composition requires a new HTTP parser/filter fork;
- identity/session semantics cannot match a coherent bidirectional tunnel without core changes;
- outproxy cannot be reliably disabled;
- standalone M067/M068 APIs require broad refactoring outside I2PControl;
- partial runtime cannot be represented truthfully with existing internal state.
