# ADR-0003: Proposal 170 Tunnel Runtime Completion and Application-Filter Boundary

Status: accepted

Date: 2026-08-14

Decision owners: project maintainers

Supersedes, only for newly authorized missing-tunnel runtime scope:

- the deferred/missing-data-plane portions of `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- the runtime-eligibility restriction in `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md` that limited real backends to generic `client` and `server`.

ADR-0001 and ADR-0002 remain historical authority for exhaustive registration, truthful unsupported behavior, startup/control-plane ownership separation, generic client/server lifecycle, and server-secret ownership unless this ADR explicitly changes a point.

Related governance and canonical planning:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`, revision created/updated `2026-05-20`;
- established SAM v3 streaming/datagram behavior;
- Java I2PTunnel behavior as read-only security/interoperability reference only.

## Context

The Proposal 170 control plane now has an exhaustive twelve-type tunnel model, durable definitions, lifecycle dispatch, real generic `client` and `server` backends, and explicit unsupported backends for ten remaining types:

- `httpclient`;
- `ircclient`;
- `socks`;
- `socksirc`;
- `connectclient`;
- `streamrclient`;
- `httpserver`;
- `httpbidirserver`;
- `ircserver`;
- `streamrserver`.

The earlier project direction intentionally deferred these data planes so Proposal 170 contract work could close without broad runtime implementation. Maintainer direction now explicitly reopens these ten declared Proposal 170 tunnel types as internal-fork implementation scope.

This is not authority to add any non-Proposal-170 tunnel type, router protocol, frontend, base I2PControl method, RouterInfo source, or upstream contribution activity.

Research against the existing Emissary/Yosemite boundary and the Java I2PTunnel reference shows an important architectural distinction:

1. the remaining TCP client types can establish independent SAM/Yosemite streaming sessions and local listeners from I2PControl without router-core changes;
2. security-sensitive server types must not use blind SAM `STREAM FORWARD`, because HTTP and IRC require application-layer inspection before bytes reach the local service;
3. SAM/Yosemite application-visible stream acceptance provides the required interception point and peer destination identity without adding a router-core socket/stream API;
4. Streamr is a small datagram-oriented subsystem and can be implemented through existing SAM/Yosemite datagram support plus Tokio UDP;
5. several Java tunnel families are compositions rather than independent networking stacks: SOCKS-IRC reuses SOCKS negotiation plus IRC filters, and HTTP bidirectional server composes an HTTP server with a no-outproxy HTTP client.

The largest security risk is treating specialized server tunnels as generic byte forwarders. HTTP and IRC protocols can disclose local/server identity or carry attacker-controlled metadata across the anonymity boundary. Their filters are therefore part of the tunnel's minimum correct data plane, not optional hardening.

## Decision drivers

- Complete the remaining ten Proposal 170 tunnel types without changing the public Proposal 170 model.
- Keep implementation business logic under `emissary-cli/src/i2pcontrol/**` wherever technically possible.
- Avoid new `emissary-core/**` production changes for tunnel completion.
- Preserve the accepted source/dependency containment authorities from M061-M063.
- Treat filtering and normalization as mandatory for HTTP/IRC specialized tunnels.
- Prevent hidden-service tunnels from becoming identity-leak, request-smuggling, open-proxy, SSRF-to-LAN, or protocol-confusion paths.
- Reuse the existing SAM/Yosemite application boundary rather than exposing additional router internals.
- Keep startup-managed services separate from control-plane-owned tunnel instances.
- Reject recognized-but-unimplemented security-sensitive options instead of persisting them and silently ignoring them at runtime.
- Keep verification local/package-scoped and proportional; do not create new CI/release machinery.

## Considered options

### Option A — Extend generic client/server runners until all types fit

Rejected.

A raw generic server uses SAM forwarding and cannot inspect HTTP/IRC application bytes before they reach the local target. Adding protocol filtering into generic startup tunnel modules would contaminate unrelated upstream-derived runtime paths and weaken the I2PControl containment boundary.

### Option B — Add router-core stream/peer APIs for specialized servers

Rejected.

The existing SAM/Yosemite application interface already provides sufficient stream acceptance and peer identity. New core APIs would increase audited-core surface without a protocol need.

### Option C — Reuse startup HTTP/SOCKS managers directly

Rejected as the primary design.

Those services have startup-oriented ownership and configuration that differs from control-plane-created TunnelManager definitions. Refactoring them into shared lifecycle authority would widen unrelated code changes and risk ownership ambiguity. Existing source may be used as behavioral evidence, but control-plane runtimes should remain I2PControl-owned unless a later implementation proves one tiny neutral helper is necessary.

### Option D — Implement composable I2PControl-owned application adapters over SAM/Yosemite

Accepted.

Client adapters own local listeners and outbound SAM/Yosemite streams. Filtered server adapters own persistent I2P destination sessions, accept I2P streams in-process, apply protocol-specific filtering, and only then connect to local services. Datagram adapters own Streamr-specific UDP/subscription logic.

## Decision

### 1. Scope

The Proposal 170 workstream is expanded to make all twelve declared tunnel types operational where the pinned protocol and available SAM/Yosemite capabilities permit a correct implementation.

The ten previously unsupported types are now authorized implementation scope. This authorization is limited to their Proposal 170-compatible runtime semantics.

### 2. Primary ownership boundary

New specialized tunnel runtime, filter, lifecycle, option-capability, and protocol-adapter code SHOULD live under:

`emissary-cli/src/i2pcontrol/**`

The preferred structure is an I2PControl-owned backend/runtime/filter hierarchy. Exact filenames are implementation details, but the ownership must remain clear.

No new `emissary-core/**` production change is authorized by this ADR for missing-tunnel implementation. If an implementation agent proves a core change is unavoidable, it must stop and create a separate corrective/architecture plan rather than silently widening a milestone.

Changes to other `emissary-cli/src/**` production paths are also exceptional. Existing generic client/server seams from ADR-0002 remain accepted; new specialized types should not require refactoring startup managers.

### 3. Filtered server architecture

`httpserver` and `ircserver` MUST use an application-visible accepted-stream path, not blind forwarding to the local service.

Required shape:

```text
remote I2P peer
    -> SAM/Yosemite accepted stream
    -> I2PControl-owned bounded protocol parser/filter
    -> local TCP target
```

The local TCP connection MUST NOT be opened until the security-critical initial protocol material has been validated and sanitized where the protocol permits this ordering.

Peer identity used for trusted injected metadata MUST come from the accepted I2P stream/session, never from client-supplied application headers or fields.

### 4. HTTP server minimum security contract

`httpserver` is not operational unless all of the following are implemented and tested:

- bounded request-line/header parsing with time, per-line, header-count, and aggregate-size limits;
- rejection of malformed headers, control-character injection, obsolete folding, ambiguous framing, conflicting `Content-Length`, and unsafe `Transfer-Encoding`/`Content-Length` combinations;
- removal of caller-supplied I2P identity headers before optional injection of trusted peer-derived identity headers;
- safe `Host` handling / configured website-hostname rewriting;
- explicit handling/removal of forwarding/proxy identity headers so remote input cannot impersonate a trusted reverse proxy;
- hop-by-hop header handling;
- request-policy enforcement for Proposal 170 HTTP/server controls that are declared supported;
- connection and POST/request throttling keyed by trusted I2P peer identity where applicable;
- response-header parsing and removal of server-fingerprinting/proxy headers before returning data to I2P;
- loopback-safe local target defaults and no request-controlled target selection;
- bounded tasks and cancellation; one malformed/slow peer cannot indefinitely retain unbounded resources.

Transparent compression optimization is not required for initial correctness unless needed by the pinned contract. Security and protocol correctness precede optimization.

### 5. IRC minimum security contract

`ircclient` MUST use line-oriented bidirectional filtering sufficient to prevent common local-address and DCC/CTCP leaks. Initial implementation MUST fail closed for DCC rather than creating auxiliary DCC tunnels.

At minimum:

- bound IRC line sizes and registration/initial read time;
- normalize or reject unsafe `USER` hostname/servername fields;
- prevent PING/PONG forms from reflecting local/proxy addressing where applicable;
- sanitize PART/QUIT text where the reference security behavior requires it;
- allow ordinary IRC/IRCv3 commands needed for normal clients while rejecting unknown dangerous forms rather than default-passing them;
- permit safe CTCP ACTION while rejecting unsupported CTCP and DCC address/port negotiation;
- preserve CAP/SASL/IRCv3 message-tag compatibility needed by contemporary clients.

`ircserver` MUST separately filter the initial registration sequence before forwarding to the local IRC daemon. It must derive any presented peer hostname/cloak from actual I2P peer identity, reject obvious cross-protocol input, and impose strict line/count/total-time bounds.

WEBIRC and DCC are not minimum completion requirements. If requested through configuration before implemented, the backend must return an explicit unsupported-option error and must not silently ignore the request.

### 6. SOCKS and CONNECT boundaries

`socks` MUST implement a bounded local proxy whose direct-I2P mode does not perform arbitrary local DNS resolution. Initial supported command surface may be limited to SOCKS4a/SOCKS5 TCP CONNECT; BIND and UDP ASSOCIATE remain outside the initial backend unless separately justified by the milestone.

`socksirc` MUST compose the SOCKS frontend with the exact same IRC filtering implementation used by `ircclient` rather than fork a second IRC sanitizer.

`connectclient` MUST be CONNECT-only. For direct I2P targets, extra browser/proxy metadata is not forwarded merely because it was present in the client request. Clearnet access requires an explicitly configured I2P outproxy; direct local/LAN target access must fail closed.

### 7. HTTP client boundary

`httpclient` MUST sanitize privacy-sensitive request metadata before sending it over I2P. It must not perform local DNS resolution for `.i2p` routing and must not silently route clearnet without an explicitly configured I2P outproxy.

Anonymity-sensitive options must have conservative defaults. If an option is accepted by TunnelManager and relevant to the running backend, it must either be applied or rejected as not implemented.

### 8. HTTP bidirectional composition

`httpbidirserver` is implemented only after `httpserver` and `httpclient` close. It MUST be composition, not a third HTTP stack:

- server side uses the accepted `httpserver` filter/runtime;
- local proxy side uses the accepted `httpclient` filter/runtime;
- outproxy capability is disabled for the bidirectional proxy role;
- the server destination/session identity is shared where required by the design;
- no new HTTP parser/filter fork is introduced.

The type remains supported because Proposal 170 declares it even if the reference implementation treats it as deprecated.

### 9. Streamr boundary

`streamrclient` and `streamrserver` may use existing SAM/Yosemite datagram capability plus Tokio UDP.

The Streamr implementation must include bounded subscriber state, explicit subscribe/unsubscribe control semantics, expiry/keepalive behavior, packet-size limits, cancellation, and no arbitrary amplification to unbounded subscriber sets.

Streamr remains isolated from the streaming TCP abstractions; do not generalize a cross-protocol framework merely to share supervision code.

### 10. Option capability enforcement

Every real backend MUST declare or implement a deterministic validation boundary for runtime-relevant Proposal 170 options.

For a real backend:

- implemented relevant options are applied;
- irrelevant options are rejected when the contract/type combination is invalid;
- recognized relevant but not-yet-implemented options fail before resource allocation;
- security-sensitive options are never accepted and silently ignored;
- secret values remain redacted from errors, logs, `Debug`, and ordinary responses.

Persistence may remain lossless, but `start` must validate the effective runtime capability set before opening listeners, creating sessions, or connecting targets.

### 11. Startup/runtime ownership

ADR-0002 ownership rules remain:

- startup-managed definitions remain externally owned unless separately migrated;
- control-plane-created definitions own only their own tasks/listeners/sessions;
- stop/restart/delete target exact named control-plane resources;
- no startup task is adopted or cancelled through TunnelManager merely because it has a similar type/configuration.

### 12. Dependency and containment rule

Prefer existing dependencies already present in `emissary-cli` (`tokio`, Yosemite/SAM support, HTTP parser utilities, URL handling, etc.). Do not add a dependency solely for convenience if a small bounded implementation is clearer.

If a new direct dependency is truly necessary only for I2PControl, it must obey the accepted M062/M063 rule: optional, default-features-disabled where practical, and activated exclusively by `feature = "i2pcontrol"`, including transitive local-feature reachability.

### 13. Reference-source licensing boundary

Java I2PTunnel source may be inspected read-only to understand security intent, protocol hazards, interoperability behavior, and test cases.

Implementation agents MUST NOT line-for-line translate or copy GPL-covered reference code into this fork unless repository maintainers separately resolve licensing compatibility. Plans should specify required behavior independently and use independently authored Rust implementations and fixtures.

## Consequences

### Positive

- all Proposal 170 tunnel types can become operational without redesigning TunnelManager wire/storage contracts;
- security filters stay at the application boundary where they can inspect bytes and peer identity;
- the audited router core does not need new tunnel-type logic;
- the existing unsupported backend remains a safe fallback until each family independently closes;
- HTTP/IRC leak prevention becomes a closure requirement instead of post-hoc hardening;
- specialized tunnel families can be reviewed and reverted independently.

### Negative

- `emissary-cli/src/i2pcontrol/**` becomes a larger optional subsystem with real networking/application protocol code;
- some behavior overlaps conceptually with startup HTTP/SOCKS services rather than being factored into shared upstream-style abstractions;
- operational completeness requires several milestones and security-focused tests;
- some Proposal 170 options may remain explicitly rejected until their semantics are implemented.

### Neutral/deferred

- RouterInfo 37/1/5 source disposition is unchanged;
- AddressBook `SetConfig` limitations are unchanged by this ADR;
- base I2PControl methods outside the Proposal 170 scope remain unchanged;
- frontend integration remains out of scope;
- upstream contribution/review/adoption remains prohibited.

## Compatibility and migration

- public Proposal 170 method, action, tunnel-type, status, and persistence schemas remain unchanged;
- persisted definitions for previously unsupported types require no schema migration merely to gain a real backend;
- a definition becomes startable only when its type backend and requested option set are operational;
- existing unsupported-state records must continue to load;
- startup configuration remains authoritative for startup-owned services.

## Security and reliability implications

- specialized server filtering is part of correctness and cannot be bypassed by selecting a different lifecycle path;
- filters must process untrusted network input with explicit bounds and timeouts;
- peer-derived identity is trusted only from the SAM/Yosemite stream/session boundary;
- local backend connections must not become request-controlled SSRF/open-proxy primitives;
- error strings must remain secret-free;
- task/session/listener counts must be bounded by existing tunnel inventory/resource limits;
- cancellation must close exact resources and prevent stale-generation tasks from reporting current state.

## Verification

The runtime-completion roadmap must establish evidence that:

1. no new missing-tunnel work requires `emissary-core/**` production changes;
2. each real backend validates its runtime option capability set before allocation;
3. HTTP server request and response filtering is mandatory and non-bypassable;
4. IRC client/server filtering blocks address-leaking features such as unsupported DCC;
5. SOCKS-IRC uses the common IRC filter;
6. CONNECT rejects non-CONNECT and unsafe target paths;
7. HTTP client anonymity-sensitive headers and routing are deterministic;
8. bidirectional HTTP is composed from already-closed HTTP primitives;
9. Streamr subscription state and packet handling are bounded;
10. lifecycle start/stop/restart/delete remains exact-name and ownership-safe;
11. unsupported types remain resource-free until their owning milestone closes;
12. feature-disabled/default Emissary behavior remains unaffected;
13. no upstream/third-party write or contribution preparation occurs.

## Supersession

ADR-0001 and ADR-0002 remain accepted historical records. This ADR supersedes only their statements that the ten missing Proposal 170 tunnel data planes are deferred/out of scope or ineligible for real backend registration. Their truthful-stub, exhaustive-registry, ownership, secret-storage, and internal-only constraints remain in force.