# Emissary Proposal 170 Long-Term Specification

Status: normative for the Proposal 170 workstream; tunnel-runtime scope amended 2026-08-14 by ADR-0003; full-support completion target amended 2026-08-27 by ADR-0004

This document defines the required end state for implementing I2P Proposal 170 in Emissary. The keywords MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, and MAY are normative.

## 1. Purpose

Emissary MUST expose the I2PControl API additions described by I2P Proposal 170 without expanding the proposal, redesigning router behavior for API convenience, or coupling the work to frontend state.

The workstream is pinned to I2P Proposal 170 revision created/updated `2026-05-20`. The proposal remains Open, so a claim of full support is always revision-specific and does not imply future draft changes, upstream review, or upstream acceptance.

The implementation history deliberately progressed through safe intermediate states:

- contract completeness with truthful unavailable selectors and explicit tunnel stubs;
- all-twelve-tunnel runtime completion through bounded I2PControl-owned application backends;
- security hardening and independent reclosure;
- the current full-support completion phase authorized by ADR-0004.

The required final state is full support for the pinned Proposal 170 revision:

- every specified method, selector, parameter, action, tunnel type, response key, JSON type, and validation rule exists exactly;
- every behaviorally meaningful applicable Proposal 170 configuration/option is operational rather than accepted-and-ignored;
- RouterInfo data is backed by authoritative state or by an explicitly proven semantic such as a by-design empty set, never by source-absence defaults;
- all thirteen pinned AddressBook `SetConfig` keys have an explicit operational or non-runtime-metadata disposition consistent with the reference semantics;
- all twelve tunnel types remain real and operational;
- every applicable TunnelManager option/type cell is applied; `not_applicable` is permitted only with explicit contract/reference rationale;
- specialized HTTP/IRC tunnel types retain the filtering required to preserve the anonymity/application boundary;
- final closure includes live interoperability and restart/persistence evidence rather than parser/unit-test evidence alone;
- Proposal 170-specific business/admin/application policy remains under `emissary-cli/src/i2pcontrol/**` wherever technically possible.

## 2. Normative external references

The implementation MUST preserve compatibility with:

- I2P Proposal 170, `I2PControl API Expansion`, pinned internally to revision `2026-05-20`;
- the existing I2PControl JSON-RPC API and authentication/version behavior needed by the implemented extension surface;
- JSON-RPC 2.0 response-envelope and request-ID semantics;
- SAM/Yosemite streaming and datagram behavior used as the application/router boundary.

The proposal is the authority for Proposal 170 fields and actions. The established I2PControl API is the authority for shared transport, authentication, token, version, and JSON-RPC behavior already within scope. Reference implementations MAY clarify ambiguity and security intent but MUST NOT silently expand the protocol.

Java I2PTunnel/i2pd/I2P reference source is read-only behavioral/security evidence. Implementation MUST be independently authored and MUST NOT line-for-line translate reference code without a separate licensing decision.

Because the proposal is Open, a later upstream proposal revision requires an explicit delta audit. Historical closure against `2026-05-20` remains valid evidence for that pinned revision.

## 3. Scope boundary

### 3.1 Required capability

The workstream owns:

- an independently runnable I2PControl service;
- authentication and JSON-RPC dispatch required to expose Proposal 170;
- all Proposal 170 additions to `RouterInfo`;
- `AddressBook`, including exact CRUD, subscriptions, configuration, persistence, and restart behavior;
- `TunnelManager`;
- `ClientServicesInfo`;
- persistent Proposal 170 administrative state;
- read-only router inspection required by Proposal 170;
- exhaustive tunnel backend registration;
- real implementations for all twelve Proposal 170 tunnel types: `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient`, `server`, `httpserver`, `httpbidirserver`, `ircserver`, and `streamrserver`;
- complete applicable TunnelManager option semantics through existing supported session/application primitives;
- protocol, persistence, security, restart, containment, conformance, and focused live-interoperability tests.

### 3.2 Explicit non-goals

The workstream MUST NOT:

- add I2P wire messages or extend Proposal 170;
- implement unrelated base I2PControl methods such as `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, or `AdvancedSettings` merely to claim Proposal 170 completion;
- alter routing, peer selection, NetDB behavior, tunnel construction algorithms, transport retry/congestion behavior, exploratory-tunnel policy, or cryptographic protocol behavior merely to populate an administrative field;
- add tunnel types not declared by the pinned Proposal 170 contract;
- redesign existing startup proxy/tunnel managers merely to make control-plane lifecycle convenient;
- migrate/adopt startup-managed tasks into TunnelManager unless a separate accepted plan explicitly authorizes it;
- change runtime address-book resolution precedence as incidental configuration work;
- add frontend controls, pages, views, or frontend-owned state;
- report unsupported or absent state as successful through fabricated zero, false, empty, or adjacent values;
- accept a security-sensitive or behaviorally meaningful runtime option and silently ignore it;
- create a router-wide peer-ban algorithm solely to populate `i2p.router.netdb.bannedpeers`;
- initiate or prepare upstream review, merge, adoption, or contribution activity.

## 4. Architectural invariants

### 4.1 Administrative ownership

I2PControl is an administrative/application control surface. Its HTTP listener, authentication, request parsing, persistence, method handlers, administrative configuration, control-plane tunnel supervision, specialized Proposal 170 tunnel adapters, recent-metric samplers, and news-source policy SHOULD be owned by `emissary-cli/src/i2pcontrol/**` or an equivalent optional application-layer component.

Core router crates MUST NOT acquire HTTP/JSON-RPC/application-proxy/news dependencies solely for I2PControl.

A standalone `i2pcontrol` crate is not required merely for aesthetic isolation. The current optional feature/module boundary is acceptable if containment remains exact.

### 4.2 Handler purity

JSON-RPC handlers MUST perform only:

1. authentication and version checks;
2. parameter parsing and validation;
3. invocation of a typed control-plane operation or bounded source snapshot;
4. exact response serialization.

Handlers MUST NOT directly own router tasks, mutate NetDB state, edit arbitrary files, perform one network fetch per getter call, or implement application-protocol filtering inline. Data-plane/filter/source-worker logic belongs behind typed I2PControl-owned adapters or neutral read-only owner snapshots.

### 4.3 Router inspection and owner exceptions

Core changes for selector truthfulness are permitted only when a canonical lower-layer owner is the only truthful place to know a required fact and the change has its own accepted containment evidence.

Such interfaces MUST be:

- neutral and Proposal-170-agnostic;
- bounded and read-only;
- passive observations of existing behavior;
- individually path-budgeted before implementation;
- unable to change peer selection, transport decisions, NetDB state, routing, tunnel building, congestion/retry, or cryptographic behavior.

Inspection interfaces MUST NOT expose mutable subsystem handles, transfer task ownership, consume a single-owner event receiver, or leak private/session key material, sockets, mutable session/tunnel/router objects, or command channels.

The full-support phase MUST exhaust truthful I2PControl-local derivation before introducing a lower-layer owner change. Missing network-error reason state is the principal anticipated neutral-owner exception. A peer-ban engine is not authorized solely for telemetry.

### 4.4 Frontend independence

I2PControl MUST operate in headless and frontend-enabled builds without depending on frontend state or lifecycle. No visual frontend work is part of this workstream.

### 4.5 Startup/control-plane ownership separation

Startup-managed tunnels and services remain externally owned unless separately migrated. Control-plane-created definitions may start, stop, restart, and delete only resources created by their own backend/supervisor generation.

No backend may identify a similarly configured startup service and adopt or cancel it implicitly.

## 5. Protocol exactness

The implementation MUST:

- use JSON-RPC 2.0 envelopes;
- preserve request IDs;
- use named parameters;
- require authentication tokens for all methods except authentication as defined by I2PControl;
- return only requested selector keys where the API uses selector-by-presence behavior;
- preserve exact Proposal 170 key names, value types, action names, and tunnel type names;
- distinguish JSON-RPC/protocol errors from TunnelManager operation status strings;
- avoid aliases, pagination, metadata wrappers, capability fields, or extension status values not defined by the protocol except already-documented internal compatibility aliases.

Compatibility aliases/extensions do not count toward canonical completion and MUST NOT change the behavior of canonical lowercase Proposal 170 requests.

Malformed examples in explanatory text MUST NOT override the established JSON-RPC result envelope.

## 6. Tunnel contract, real backends, and final option semantics

Every Proposal 170 tunnel type MUST be accepted by parsing, validation, persistence, backend dispatch, and real runtime execution.

All twelve types have real backends in the current production baseline. The final full-support phase MUST preserve that architecture and complete option semantics without redesigning the public model.

A real backend MUST validate the effective runtime option set before resource allocation. For runtime-relevant options, the backend MUST either apply the requested behavior or reject the operation during an intermediate implementation revision. Persist-and-ignore is forbidden for security-sensitive or behaviorally significant options.

At final full-support closure:

- every canonical option/type cell is explicitly classified;
- every behaviorally applicable cell is applied by the actual runtime/session/filter/secret-storage path;
- every `not_applicable` cell has a contract/reference rationale;
- no applicable cell remains permanently `unsupported`, `blocked_primitive`, or parser-only;
- secret/path-bearing values remain confined and redacted;
- session/tunnel-construction controls use existing supported Yosemite/SAM/session primitives rather than new Proposal-170-shaped core APIs;
- a genuinely missing lower-level primitive is a stop condition requiring separate architecture/containment review, not authority for an incidental core expansion.

Replacing a current reject disposition with implemented semantics MUST NOT require a public handler/action/tunnel-type redesign.

## 7. Specialized tunnel security boundary

### 7.1 Filtered server rule

`httpserver` and `ircserver` MUST NOT be blind forwarders to a local TCP target.

Their required architecture remains:

```text
remote I2P peer
    -> application-visible SAM/Yosemite accepted stream
    -> trusted peer identity + bounded admission
    -> I2PControl-owned bounded protocol parser/filter
    -> explicitly configured literal-loopback local target
```

Security-critical initial material MUST be validated/sanitized before the local service receives it where the accepted protocol architecture requires that ordering. Trusted peer identity MUST come from the accepted I2P stream/session, never attacker-supplied application metadata.

The accepted M088 lower-layer/pre-accept resource/timing residual remains documented unless a separately authorized future plan changes it. Full option/source parity does not implicitly reopen lower-layer concurrency work.

### 7.2 HTTP server baseline

An operational `httpserver` MUST retain:

- bounded request-line/header parsing and timeouts;
- malformed/framing/request-smuggling rejection;
- caller-supplied I2P identity-header removal before trusted peer-derived injection;
- safe Host/vhost rewriting;
- forwarding/proxy identity handling that cannot spoof a trusted reverse proxy;
- hop-by-hop header handling;
- applicable Proposal 170 access/throttle controls;
- bounded request concurrency/resource use;
- response-header filtering sufficient to avoid obvious server/proxy fingerprint leakage;
- local-target confinement and no request-driven backend target selection.

### 7.3 IRC baseline

The IRC family MUST retain filters that prevent local-address and direct-connect leakage. DCC/unsupported CTCP may remain fail-closed unless the pinned Proposal 170 contract specifically requires otherwise.

`ircserver` MUST bound and sanitize registration before local IRC daemon forwarding and derive presented peer identity from actual I2P peer identity.

`socksirc` MUST reuse the common IRC filter.

### 7.4 Proxy target safety

HTTP, CONNECT, and SOCKS direct-I2P modes MUST NOT resolve arbitrary requested hostnames through local clearnet DNS. Clearnet routing requires an explicitly configured I2P outproxy. Localhost/LAN access must fail closed unless an explicit separately justified administrative configuration exists.

A non-loopback local proxy listener must not silently become an unauthenticated general-purpose network proxy.

## 8. Existing runtime ownership and server secrets

Existing startup-managed Emissary tunnels MAY be exposed through read-only inspection. Proposal 170 MUST NOT claim lifecycle authority over tasks that do not expose a safe lifecycle control contract.

An operation against an externally managed tunnel MUST either be truthfully supported or return a deterministic operation error. It MUST NOT update a control-plane copy while leaving contrary runtime state unreported.

Control-plane server destinations remain owned by the backend-confined secret-store authority established under ADR-0002. Specialized server types SHOULD reuse that authority rather than create parallel private-key path semantics.

`PrivKeyFile`, filter-file, AddressBook path, or similar remotely supplied path semantics MUST be confined to an explicitly owned administrative root. Authentication is not permission for arbitrary host filesystem access.

## 9. AddressBook boundary

Proposal 170's `private`, `local`, `router`, and `published` books MUST exist as persistent administrative stores with exact CRUD, subscription, and configuration behavior.

All thirteen pinned `SetConfig` keys MUST receive an explicit final disposition:

- behaviorally meaningful values are consumed by the active AddressBook downloader/publication/storage runtime;
- path values are normalized and confined to an I2PControl/AddressBook administrative root;
- harmless non-runtime metadata such as a frontend theme may be durably round-tripped without creating frontend coupling if the pinned semantics support that classification;
- no behaviorally meaningful value may be accepted and stored inertly.

Administrative path changes MUST preserve atomic complete-generation publication and recoverability. They MUST NOT silently change Emissary runtime resolver precedence beyond the accepted AddressBook architecture.

## 10. Truthful state and observability

RouterInfo and ClientServicesInfo MUST use real snapshots, explicit canonical semantics, or protocol-permitted neutral behavior. Missing implementation MUST NOT be disguised as zero, false, an empty collection, or a successful operation.

The historical pre-ADR-0004 RouterInfo baseline is:

- 43 total Proposal 170 additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable: news, transit 15-second bandwidth, IPv4/IPv6 network errors, and banned peers.

That 37/1/5 state remains truthful until the owning full-support milestones close, but it is no longer the intended final state.

Final requirements include:

- transit 15-second bandwidth is request-independent and derived from the authoritative cumulative transit counter through a bounded I2PControl-owned sampler;
- router news has a real adopted bounded source/cache, not arbitrary web content or a per-request fetch;
- network-error codes map from explicit neutral canonical v4/v6 error-reason state; source absence is never `No error`;
- banned peers maps from a real enforced owner or from a rigorously proven by-design-empty router semantic; a new ban algorithm is not introduced solely for the getter.

I2PControl log retrieval MAY use a bounded tracing-backed memory buffer. Clearing that buffer MUST NOT clear or reconfigure unrelated log sinks.

## 11. Persistence and recovery

Proposal 170 administrative data MUST use:

- versioned schemas;
- deterministic serialization;
- validation before activation;
- atomic same-filesystem replacement;
- bounded recovery after interrupted writes;
- explicit handling of unsupported or externally managed definitions during intermediate revisions;
- confined secret/path ownership.

Existing `router.toml` behavior MUST remain compatible unless a later accepted plan explicitly defines an additive configuration change.

Runtime backend state must be reconstructible from persisted definitions plus backend-owned secret state. A failed start/configuration mutation must not corrupt the definition or require manual database deletion.

Recent observational state such as transit 15-second history or transient network-error state need not be persisted unless the pinned semantics specifically require it; restart must have truthful warmup/unknown behavior.

## 12. Security and resource bounds

The implementation MUST provide:

- secure token generation and validation;
- loopback-safe default binding or explicit secure configuration;
- request-body and nesting limits;
- bounded peer, news, log, RouterInfo, AddressBook, and tunnel result construction;
- timeouts and cancellation for request/runtime/fetch/sampler work;
- redaction of credentials, tokens, private keys, proxy passwords, client-auth material, and sensitive destination material;
- path confinement for persistence/secret/filter/config stores;
- deterministic concurrent-edit and per-name lifecycle behavior;
- bounded listener/session/task/subscriber/recent-sample/source-cache counts;
- no lock held across network I/O, sleep, cancellation wait, or task join where avoidable;
- no hidden-service/application filter bypass path;
- no silent LeaseSet/authentication downgrade;
- no new router-wide ban behavior solely for telemetry.

## 13. Dependency and containment discipline

I2PControl-only direct dependencies MUST remain optional and activated only through the `i2pcontrol` feature, including transitive local-feature reachability, as established by M062/M063.

Completion work SHOULD use dependencies already present in `emissary-cli`. Dependency additions require an explicit separate containment decision and must not broaden default/feature-disabled builds.

M061 source containment remains the baseline authority. New files under `emissary-cli/src/i2pcontrol/**` are the preferred location. Any new non-I2PControl production path requires individual pre-implementation justification and an exact containment amendment rather than an incidental edit.

Existing historically accepted non-I2PControl paths are a ceiling, not blanket authorization to modify all of them.

## 14. Completion definition

Historical contract-complete or tunnel-runtime-complete states remain valid intermediate descriptions for the revisions that closed them.

The workstream may describe itself as fully supporting Proposal 170 against the pinned `2026-05-20` revision only when all of the following are true:

1. every Proposal 170 method/selector/key/type/action/value shape is exact and truthful;
2. all 43 Proposal 170 RouterInfo additions have authoritative final semantics with no applicable unavailable row;
3. AddressBook CRUD, SetSubscriptions, and all thirteen SetConfig keys have operational/pinned semantics and restart-safe persistence;
4. all twelve declared tunnel types have real operational backends;
5. every applicable canonical TunnelManager option/type cell is applied and no applicable cell remains unsupported/blocked/parser-only;
6. specialized HTTP/IRC backends retain their required security filtering and cannot bypass it;
7. RouterInfo and ClientServicesInfo values remain truthful and correctly typed;
8. administrative persistence/restart/failure/contention semantics remain coherent;
9. the service runs without a frontend and does not interfere with frontend event consumers;
10. Proposal 170-specific policy remains predominantly under `emissary-cli/src/i2pcontrol/**`, and every lower-layer exception is neutral, exact, and individually justified;
11. default/feature-disabled Emissary behavior remains unaffected;
12. closure evidence covers protocol, security, persistence, restart, cancellation, contention, containment, and compatibility;
13. focused live/reseeded interoperability proves the twelve tunnel families carry their intended traffic and representative reference-router behavior is compatible where practical;
14. unrelated base I2PControl method absence is not misrepresented as Proposal 170 completion work;
15. no upstream interaction has occurred.

Until that closure, the support documentation MUST continue to use a truthful partial-support statement and enumerate remaining cells.

The target final statement is:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision.

That statement MUST be accompanied by the revision pin and acknowledgement that Proposal 170 remains Open. It MUST NOT imply upstream acceptance, upstream merge intent, or general full I2PControl parity.