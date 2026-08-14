# M065 — I2PControl Tunnel Runtime and Option-Capability Primitives

Status: blocked — hard dependency on M064 closure

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/ADR authority:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

Hard dependency:

- M064 closed with feature-disabled/no-events baseline green.

## 1. Objective

Create the smallest reusable I2PControl-owned runtime primitives needed by the remaining Proposal 170 tunnel families, without yet replacing any of the ten unsupported production backends.

M065 exists to prevent each protocol family from independently inventing lifecycle, accepted-stream ownership, local-listener ownership, peer-identity transport, and runtime-option validation. It must not become a generalized networking framework.

The key architectural outcome is a tested application-layer server path where I2PControl accepts an I2P stream, receives trusted peer identity, invokes a bounded filter/handler, and only then permits local-service connection/forwarding.

## 2. Current evidence

Current `i2pcontrol/backends` contains real generic `client` and `server` supervisors plus unsupported/fake/registry implementations. The generic server uses the startup-oriented single-server runtime, which ultimately performs SAM `STREAM FORWARD`; that path cannot interpose HTTP/IRC filtering.

The remaining families require two common resource shapes:

1. client/proxy shape: control-plane-owned local TCP listener + independent SAM/Yosemite streaming session + per-connection task set;
2. filtered-server shape: backend-owned persistent destination/session + application-visible accepted I2P streams + trusted peer identity + protocol callback + local TCP connection after validation.

Streamr will need datagrams later, but M065 should not build a generalized datagram abstraction unless a small lifecycle helper is clearly shared.

The current TunnelManager domain already stores all Proposal 170 options, but real generic backends consume only a subset. M065 must add a deterministic backend-level option capability validation contract so later real backends cannot silently accept relevant options they ignore.

## 3. Classification

Primary class: infrastructure / invariant.

M065 does not claim a newly operational tunnel type.

## 4. Hard invariants

- all production changes should remain under `emissary-cli/src/i2pcontrol/**`;
- no `emissary-core/**` production change;
- no new startup proxy/tunnel lifecycle ownership;
- no public Proposal 170 wire/schema/action/type change;
- no persistence schema migration;
- no new direct dependency unless existing dependencies demonstrably cannot provide the required behavior;
- any I2PControl-only direct dependency must satisfy M062/M063 feature ownership;
- no lock across `.await`, network I/O, cancellation wait, or join;
- all runtime maps/task sets bounded by existing tunnel inventory and per-backend connection caps;
- task completion from an old generation cannot overwrite state for a newer generation;
- peer identity is represented in a form that does not expose private key/session material;
- runtime errors must not include destinations' private material, passwords, or arbitrary raw request content;
- unsupported production backend mapping remains unchanged through M065.

## 5. Explicit non-goals

Do not:

- implement HTTP/IRC parsing yet beyond tiny test-only dummy callbacks;
- register `httpclient`, `httpserver`, `ircclient`, `ircserver`, SOCKS, CONNECT, Streamr, or bidirectional HTTP as real;
- refactor `emissary-cli/src/proxy/**` into shared infrastructure;
- move existing generic client/server backends;
- create a framework capable of arbitrary protocols/transports not needed by Proposal 170;
- expose raw Yosemite session/socket objects through broad public APIs;
- add router-core inspection/control paths;
- implement DCC, WEBIRC, SOCKS UDP/BIND, or Streamr semantics;
- alter ClientServicesInfo/RouterInfo behavior except test fixtures strictly required for new internal types.

## 6. Preferred implementation surface

Expected new/modified paths are under:

```text
emissary-cli/src/i2pcontrol/backends/**
emissary-cli/src/i2pcontrol/domain/**          # only if capability model belongs here
emissary-cli/src/i2pcontrol/mod.rs             # module registration if needed
emissary-cli/src/i2pcontrol/production.rs      # composition only if required
emissary-cli/tests/**                           # focused integration/containment tests
```

A likely local structure is:

```text
backends/runtime/client_listener.rs
backends/runtime/accepted_server.rs
backends/runtime/task_group.rs
backends/options.rs
```

Names are illustrative, not mandatory.

If implementation requires an edit outside `emissary-cli/src/i2pcontrol/**`, the agent must first prove why an I2PControl-local implementation is materially worse or impossible and keep the seam neutral. No such external edit is expected.

## 7. Runtime primitive requirements

### 7.1 Client/proxy runtime owner

Provide a bounded primitive that can:

- bind an explicitly validated local interface/port;
- expose actual bound address for status/tests without leaking unrelated state;
- create/own one independent SAM/Yosemite streaming session for the backend instance;
- accept local TCP connections;
- spawn bounded connection tasks;
- hand each connection to a type-specific async handler/callback with session access limited to what is needed to open a remote stream;
- stop accepting on cancellation;
- cancel/drain exact connection tasks within bounded timeout;
- report readiness only after listener + required SAM session are operational;
- fail start cleanly on bind/SAM error without leaving a task/listener behind.

Do not force all future proxies through a single callback if explicit small wrappers are clearer. The primitive should own lifecycle, not protocol semantics.

### 7.2 Accepted-stream server runtime owner

Provide a bounded primitive that can:

- create/load a persistent destination through the existing server secret-store authority;
- establish a server SAM/Yosemite streaming session without blind `STREAM FORWARD`;
- wait for application-visible incoming streams;
- obtain the remote I2P peer destination/hash/base32 identity from the accepted stream/session API;
- pass a sanitized/trusted identity value plus stream to a type-specific handler;
- let the handler validate initial protocol bytes before opening a local TCP target;
- bound concurrent accepted-stream tasks;
- stop accepting and cancel/drain tasks on exact backend stop;
- preserve destination identity across stop/restart;
- isolate one connection panic/failure from the server listener/supervisor;
- report running only after the session is genuinely ready to accept.

The primitive must not connect the local target itself before a protocol-specific handler authorizes that step.

### 7.3 Generation and task semantics

Reuse existing server/client supervisor patterns where possible, but keep shared helpers small.

Required semantics:

- per-name monotonically changing generation/instance token;
- stale task completion ignored after restart;
- duplicate start while starting/running rejected deterministically;
- stop on absent/stopped is idempotent;
- stop sends cancellation, awaits exact tasks, aborts only after bounded timeout;
- completion removes task handles;
- start failure leaves definition durable and retryable;
- delete remains blocked/serialized by higher TunnelManager lifecycle rules while runtime active.

### 7.4 Trusted peer identity type

Define a narrow immutable value passed to server filters containing only required public peer identity representations, e.g. B32/hash/base64 public destination if needed.

Do not expose private destination/session material, router handles, mutable sockets beyond the accepted stream object itself, or arbitrary internal event handles.

The identity must be derived exclusively from the accepted I2P connection.

## 8. Runtime option capability model

Add a deterministic validation step between persisted `TunnelDefinition` and resource allocation.

The model must allow each real backend to classify Proposal 170 fields/options as:

- required and implemented;
- optional and implemented;
- irrelevant/invalid for that type;
- recognized but not implemented for that backend version.

Rules:

1. validation occurs before listener/session/target connection allocation;
2. failure returns a sanitized deterministic backend error mapped through existing TunnelManager status behavior;
3. secret option values never appear in errors;
4. unimplemented security-sensitive options cannot be treated as successful no-ops;
5. raw `CustomOptions` may remain round-tripped, but a real backend must explicitly decide whether a requested custom option namespace is accepted, ignored by documented protocol semantics, or rejected;
6. generic client/server behavior is not retroactively broken by M065—capability enforcement for them may be audited, but broad changes belong to a corrective plan unless a direct safety bug is found.

Implementation should prefer a small per-backend declaration/validator over a large global feature matrix framework.

## 9. Ordered work packages

### WP1 — inspect existing Yosemite interfaces and freeze ownership

Confirm exact APIs for:

- streaming session creation;
- local/outbound connection;
- application-visible accept;
- remote peer identity access;
- cancellation behavior;
- datagram capabilities relevant only to future M071.

Record any mismatch from the roadmap assumptions. If accepted-stream peer identity is unavailable without core changes, stop M065 and replan rather than adding core API.

### WP2 — implement client listener lifecycle primitive

Add the smallest reusable owner with cancellation/readiness/task bounds.

Use test handlers only.

### WP3 — implement accepted-stream server lifecycle primitive

Reuse `ServerDestinationStore` authority and establish the filtered-server interception boundary.

The test handler should prove local target connection can be delayed until after initial bytes/identity are examined.

### WP4 — implement option-capability validation contract

Add backend-facing validation helpers/errors and table/unit tests proving required/applied/rejected classifications.

Do not modify the public JSON model.

### WP5 — failure/cancellation/contention tests

Exercise:

- bind collision;
- SAM session creation failure;
- accepted-stream handler error;
- task panic;
- stop during start;
- stop with active connections;
- restart generation replacement;
- connection-cap exhaustion;
- option rejection before resource allocation;
- secret-redaction behavior.

### WP6 — containment verification and closure

Prove production registry still has only generic client/server as real backends. Run existing M061/M062 containment guards. Update planning status only after closure evidence.

## 10. Failure, cancellation, restart, and contention semantics

Failure is per backend instance/connection. A connection-level parser/target error closes that connection and does not stop the listener unless a session-level fatal error is demonstrated.

Start is successful only after required listener/session readiness.

Stop:

1. marks instance stopping;
2. stops acceptance;
3. signals child tasks;
4. awaits exact current-generation tasks within bound;
5. aborts remaining exact tasks if timeout expires;
6. releases listener/session resources;
7. publishes stopped only for matching generation.

Restart is completed stop then fresh start using the same persisted definition/secret identity.

Per-name operations are serialized through existing TunnelManager/backend supervision. M065 must not add a second global lock that serializes unrelated tunnel names.

## 11. Compatibility, migration, security, and operations

Compatibility: no wire/API change and no newly real type.

Persistence: existing definition/server-secret formats reused; no schema migration expected.

Security: establishes the non-bypassable interception point for future filtered servers and fail-before-allocation option validation.

Operations: feature-disabled/default CLI must not instantiate these primitives.

## 12. Focused tests

Required tests should include:

- local listener bind/ready/stop;
- connection cap enforcement;
- fake SAM session failure cleanup;
- accepted stream exposes trusted peer identity to handler;
- handler can reject before local target connect;
- accepted handler panic isolation;
- destination identity survives restart;
- exact generation cancellation;
- stale completion cannot clobber new state;
- unsupported relevant option rejected before bind/session;
- secret value absent from error/debug text;
- production registry mapping unchanged.

Use fake/bounded local SAM fixtures where possible. No external network required.

## 13. Verification commands

At minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol i2pcontrol::backends
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
git diff --check
```

Also inspect changed paths relative to M064 closure head. No full workspace/remote CI expansion required.

## 14. Acceptance criteria

M065 may close only when:

1. M064 is closed and baseline is green;
2. all production additions are under `emissary-cli/src/i2pcontrol/**` unless a separately documented neutral exception was approved;
3. no `emissary-core/**` production file changes;
4. client listener primitive binds, reports ready, accepts bounded local connections, and shuts down exactly;
5. accepted-stream server primitive establishes a persistent destination/session without blind forwarding;
6. remote peer public identity reaches the handler from trusted SAM/Yosemite state;
7. test proves a handler can reject input before any local target connection occurs;
8. per-instance/per-connection task counts are bounded;
9. duplicate start and stop/restart semantics are deterministic;
10. stale-generation completion cannot overwrite newer state;
11. bind/SAM/handler panic/failure affects only target instance;
12. option-capability validation executes before resource allocation;
13. relevant recognized-but-unimplemented security option can be rejected deterministically;
14. secrets are redacted from validation/runtime errors;
15. production registry still maps the same ten types to unsupported backends;
16. no persistence/public schema change;
17. no startup task ownership change;
18. feature-disabled/default check remains green;
19. M061 and M062/M063 containment tests pass;
20. no unnecessary new dependency; any unavoidable I2PControl-only dependency obeys feature ownership and has explicit closure justification;
21. no CI/release/fuzz/coverage/platform expansion;
22. no upstream/third-party write/review/submission/contribution preparation;
23. closure record explicitly names M066, M067, M068, and M071 as dependency-ready successors, while registry registers only the next handoff according to project convention.

## 15. Closure evidence required

`plans/closure/i2pcontrol-proposal-170/065-closure.md` must include:

- implementation commits;
- exact changed paths;
- requirement-to-evidence matrix;
- lifecycle/failure/contention outcomes;
- accepted-stream peer identity and pre-local-connect rejection proof;
- option fail-before-allocation proof;
- resource-bound values;
- dependency/containment review;
- production-registry unchanged proof;
- default/feature-disabled review;
- internal-only attestation;
- unresolved findings and final disposition.

## 16. Stop conditions

Stop/replan if:

- accepted-stream/peer identity requires new core APIs;
- the only viable design adopts startup-managed sessions/tasks;
- a new dependency would become unconditional/default without an independent consumer;
- the primitive begins embedding HTTP/IRC/SOCKS policy rather than lifecycle;
- production registry would need to promote a tunnel family before its security milestone closes.