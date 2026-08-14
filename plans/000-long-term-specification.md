# Emissary Proposal 170 Long-Term Specification

Status: normative for the Proposal 170 workstream; tunnel-runtime scope amended 2026-08-14 by ADR-0003

This document defines the required end state for implementing I2P Proposal 170 in Emissary. The keywords MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, and MAY are normative.

## 1. Purpose

Emissary MUST expose the I2PControl API additions described by I2P Proposal 170 without expanding the proposal, redesigning router behavior, or coupling the work to frontend state.

The original contract-completion phase deliberately used explicit runtime stubs for tunnel data planes that Emissary did not yet implement. Maintainer direction on 2026-08-14 intentionally expands the long-term end state: the twelve Proposal 170 tunnel types are now expected to become operational through bounded application-layer backends where the existing SAM/Yosemite interfaces are sufficient.

The target is contract-complete and progressively runtime-complete Proposal 170 support:

- every specified method, selector, parameter, action, tunnel type, response key, JSON type, and validation rule exists;
- data already available from Emissary is returned truthfully;
- missing read-only data is exposed only through bounded inspection interfaces or remains truthfully unavailable;
- every tunnel type remains registered through a real or explicit unsupported backend at every intermediate revision;
- unsupported backends remain truthful until their independently reviewed real replacement closes;
- specialized HTTP/IRC tunnel types include the security filtering required to preserve the anonymity/application boundary;
- future/runtime backend replacement does not change the public Proposal 170 API or persistence schema.

## 2. Normative external references

The implementation MUST preserve compatibility with:

- I2P Proposal 170, `I2PControl API Expansion`, pinned internally to revision `2026-05-20`;
- the existing I2PControl JSON-RPC API and authentication/version behavior needed by the implemented extension surface;
- JSON-RPC 2.0 response-envelope and request-ID semantics;
- SAM/Yosemite streaming and datagram behavior used as the application/router boundary.

The proposal is the authority for Proposal 170 fields and actions. The established I2PControl API is the authority for shared transport, authentication, token, version, and JSON-RPC behavior already within scope. Reference implementations MAY clarify ambiguity and security intent but MUST NOT silently expand the protocol.

Java I2PTunnel source is read-only behavioral/security reference material. Implementation MUST be independently authored and MUST NOT line-for-line translate reference code without a separate licensing decision.

## 3. Scope boundary

### 3.1 Required capability

The workstream owns:

- an independently runnable I2PControl service;
- authentication and JSON-RPC dispatch required to expose Proposal 170;
- Proposal 170 additions to `RouterInfo`;
- `AddressBook`;
- `TunnelManager`;
- `ClientServicesInfo`;
- persistent Proposal 170 administrative state;
- read-only router inspection required by Proposal 170;
- exhaustive tunnel backend registration;
- real generic `client` and `server` control-plane backends;
- real implementations, through bounded staged milestones, for the remaining declared Proposal 170 tunnel types: `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient`, `httpserver`, `httpbidirserver`, `ircserver`, and `streamrserver`;
- protocol, persistence, security, restart, containment, and conformance tests.

### 3.2 Explicit non-goals

The workstream MUST NOT:

- add I2P wire messages or extend Proposal 170;
- alter routing, peer selection, NetDB behavior, tunnel construction, transport behavior, congestion control, exploratory-tunnel policy, or cryptographic protocol behavior merely to implement tunnel adapters;
- add tunnel types not declared by the pinned Proposal 170 contract;
- redesign existing startup proxy/tunnel managers merely to make control-plane lifecycle convenient;
- migrate/adopt startup-managed tasks into TunnelManager unless a separate accepted plan explicitly authorizes it;
- change runtime address-book resolution precedence as an incidental tunnel-runtime refactor;
- add frontend controls, pages, views, or frontend-owned state;
- report an unsupported or partially implemented tunnel as active or traffic-capable;
- fabricate values to make selectors or runtime options appear implemented;
- accept a security-sensitive runtime option and silently ignore it;
- initiate or prepare upstream review, merge, adoption, or contribution activity.

## 4. Architectural invariants

### 4.1 Administrative ownership

I2PControl is an administrative control surface. Its HTTP listener, authentication, request parsing, persistence, method handlers, control-plane tunnel supervision, and specialized Proposal 170 tunnel adapters SHOULD be owned by `emissary-cli/src/i2pcontrol/**` or an equivalent optional application-layer component.

Core router crates MUST NOT acquire HTTP/JSON-RPC/application-proxy dependencies solely for I2PControl.

The preferred completion target is zero new `emissary-core/**` production changes for the missing-tunnel runtime work. A claimed need for such a change blocks the affected implementation milestone pending a separate architecture/corrective plan.

### 4.2 Handler purity

JSON-RPC handlers MUST perform only:

1. authentication and version checks;
2. parameter parsing and validation;
3. invocation of a typed control-plane operation;
4. exact response serialization.

Handlers MUST NOT directly own router tasks, mutate NetDB state, edit arbitrary files, or implement application-protocol filtering inline. Tunnel data-plane/filter logic belongs behind typed backends/runtime adapters.

### 4.3 Router inspection

Core changes for selector truthfulness are permitted only when they expose bounded, read-only snapshots required by Proposal 170 and have their own accepted containment evidence.

Inspection interfaces MUST NOT:

- expose mutable subsystem handles;
- transfer task ownership;
- modify peer profiles, bans, NetDB entries, tunnels, transports, or queues;
- consume an event stream needed by another frontend or subsystem;
- block router progress on unbounded serialization.

Missing-tunnel runtime implementation MUST NOT use selector/inspection exceptions as a pretext for new core networking APIs when SAM/Yosemite already provides the needed application boundary.

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
- avoid aliases, pagination, metadata wrappers, capability fields, or extension status values not defined by the protocol except already-documented internal compatibility aliases that predate this amendment.

Malformed examples in explanatory text MUST NOT override the established JSON-RPC result envelope.

## 6. Tunnel contract, staged backends, and option semantics

Every Proposal 170 tunnel type MUST be accepted by parsing, validation, persistence, and backend dispatch.

At every intermediate revision, a type has exactly one registered backend that is either real or explicitly unsupported.

For a tunnel type without a closed runtime implementation:

- `create`, `edit`, `get`, and `delete` operate on persistent administrative state;
- `start` and `restart` reach an explicit unsupported backend and return a deterministic Proposal 170-compatible `error - ... not implemented` status;
- `stop` is safe and idempotent for an inactive definition;
- status inspection MUST NOT report the tunnel as running;
- no listener, LeaseSet, session, destination, or traffic path may be simulated;
- unsupported state remains internal and maps to an existing wire-level inactive state where required.

A real backend MUST validate the effective runtime option set before resource allocation. For runtime-relevant options, the backend MUST either apply the requested behavior or reject the operation as unsupported/invalid. Persist-and-ignore is forbidden for security-sensitive or behaviorally significant options.

Replacing an unsupported backend with a real backend MUST NOT require public API, handler, action, tunnel-type, or persistence-schema redesign.

## 7. Specialized tunnel security boundary

### 7.1 Filtered server rule

`httpserver` and `ircserver` MUST NOT be implemented as blind SAM forwarding to a local TCP target.

Their required architecture is:

```text
remote I2P peer
    -> application-visible SAM/Yosemite accepted stream
    -> bounded I2PControl-owned protocol parser/filter
    -> explicitly configured local target
```

Security-critical initial material MUST be validated/sanitized before the local service receives it. Trusted peer identity MUST come from the accepted I2P stream/session, never from attacker-supplied application metadata.

### 7.2 HTTP server baseline

An operational `httpserver` MUST include:

- bounded request-line/header parsing and timeouts;
- malformed/framing/request-smuggling rejection;
- caller-supplied I2P identity-header removal before trusted peer-derived injection;
- safe Host/vhost rewriting;
- forwarding/proxy identity handling that cannot spoof a trusted reverse proxy;
- hop-by-hop header handling;
- supported Proposal 170 access/throttle controls;
- bounded request concurrency/resource use;
- response-header filtering sufficient to avoid obvious server/proxy fingerprint leakage;
- local-target confinement and no request-driven backend target selection.

Optimization such as I2P-specific transparent compression is secondary and not a prerequisite unless required by the pinned contract.

### 7.3 IRC baseline

An operational IRC family MUST include the anonymity filters necessary to prevent local-address and direct-connect leakage. Initial completion MAY reject DCC and unsupported CTCP rather than implementing auxiliary DCC tunnels.

`ircserver` MUST bound and sanitize the registration phase before connecting/forwarding it to the local IRC daemon and derive any presented client hostname/cloak from actual I2P peer identity.

`socksirc` MUST reuse the common IRC filter rather than fork a second sanitizer.

### 7.4 Proxy target safety

HTTP, CONNECT, and SOCKS direct-I2P modes MUST NOT resolve arbitrary requested hostnames through local clearnet DNS. Clearnet routing requires an explicitly configured I2P outproxy. Localhost/LAN access must fail closed unless an explicit, separately justified administrative configuration allows it.

A non-loopback local proxy listener must not silently become an unauthenticated general-purpose network proxy when Proposal 170 authentication options require protection.

## 8. Existing runtime ownership and server secrets

Existing startup-managed Emissary tunnels MAY be exposed through read-only inspection. Proposal 170 MUST NOT claim lifecycle authority over tasks that do not expose a safe lifecycle control contract.

An operation against an externally managed tunnel MUST either be truthfully supported or return a deterministic operation error. It MUST NOT update a control-plane copy while leaving contrary runtime state unreported.

Control-plane server destinations remain owned by the backend-confined secret store established under ADR-0002. Specialized server types SHOULD reuse that authority rather than create parallel private-key path semantics.

## 9. Address-book boundary

Proposal 170's `private`, `local`, `router`, and `published` books MUST exist as persistent administrative stores with exact CRUD, configuration, and subscription behavior to the extent currently supported/claimed.

This workstream MUST NOT change Emissary's runtime resolver precedence merely to implement tunnel backends. A backend that needs name resolution should use the already composed application/address-book boundary or fail closed; broad resolver-ownership changes require separate approval.

## 10. Truthful state and observability

RouterInfo and ClientServicesInfo MUST use real snapshots or explicitly permitted neutral/unavailable behavior. Missing implementation MUST NOT be disguised as zero, false, an empty collection, or a successful operation.

The accepted RouterInfo matrix remains 43 total / 37 available / 1 protocol-permitted neutral / 5 unavailable unless a separately planned source-owner change is approved.

I2PControl log retrieval MAY use a bounded tracing-backed memory buffer. Clearing that buffer MUST NOT clear or reconfigure unrelated log sinks.

## 11. Persistence and recovery

Proposal 170 administrative data MUST use:

- versioned schemas;
- deterministic serialization;
- validation before activation;
- atomic same-filesystem replacement;
- bounded recovery behavior after interrupted writes;
- explicit handling of unsupported or externally managed tunnel definitions.

Existing `router.toml` behavior MUST remain compatible unless a later accepted plan explicitly defines an additive configuration change.

Runtime backend state must be reconstructible from persisted definitions plus backend-owned secret state. A failed start must not corrupt the definition or require manual database deletion.

## 12. Security and resource bounds

The implementation MUST provide:

- secure token generation and validation;
- loopback-safe default binding or explicit secure configuration;
- request-body and nesting limits;
- bounded peer, log, RouterInfo, and tunnel result construction;
- timeouts and cancellation for request/runtime work;
- redaction of credentials, tokens, private keys, proxy passwords, and sensitive destination material;
- path confinement for persistence/secret stores;
- deterministic concurrent-edit and per-name lifecycle behavior;
- bounded listener/session/task/subscriber counts;
- no lock held across network I/O, sleep, cancellation wait, or task join where avoidable;
- no hidden-service/application filter bypass path.

## 13. Dependency and containment discipline

I2PControl-only direct dependencies MUST remain optional and activated only through the `i2pcontrol` feature, including transitive local-feature reachability, as established by M062/M063.

Missing-tunnel implementation SHOULD use dependencies already present in `emissary-cli`. Dependency additions require explicit justification and must not broaden default/feature-disabled builds.

Accepted M061 source containment remains the baseline authority. New files under `emissary-cli/src/i2pcontrol/**` are the preferred location for the new capability. Any additional non-I2PControl production path requires individual justification and, if outside the accepted boundary, an explicit containment-plan amendment rather than an incidental edit.

## 14. Completion definition

The Proposal 170 workstream may describe itself as contract-complete while some runtime types remain explicit stubs, but it may describe tunnel runtime support as complete only when:

1. every Proposal 170 method/selector currently claimed supported remains exact and truthful;
2. all twelve declared tunnel types have real operational backends or a documented, explicitly accepted impossibility against the pinned platform/spec;
3. specialized HTTP/IRC backends include their required security filtering and cannot bypass it;
4. runtime-relevant accepted options are applied rather than silently ignored;
5. RouterInfo and ClientServicesInfo values remain truthful and correctly typed;
6. administrative persistence/restart semantics remain coherent;
7. the service runs without a frontend and does not interfere with frontend event consumers;
8. default/feature-disabled Emissary behavior and audited core routing behavior remain unaffected by the optional tunnel runtime work;
9. closure evidence covers protocol, security, persistence, restart, cancellation, contention, containment, and compatibility;
10. no upstream interaction has occurred.

The accurate intermediate statement remains:

> Emissary implements the Proposal 170 I2PControl contract with real generic client/server tunnels and explicit unsupported backends for tunnel families not yet independently closed.

The target final statement for this newly authorized runtime-completion phase is:

> Emissary implements the Proposal 170 I2PControl tunnel families through bounded control-plane-owned backends, with mandatory HTTP/IRC anonymity and application-boundary filtering and without expanding router-core behavior.