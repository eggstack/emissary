# Proposal 170 Long-Term Roadmap

Status: active; tunnel-runtime completion authorized 2026-08-14 by ADR-0003; full-support completion authorized 2026-08-27 by ADR-0004

This roadmap orders the work required to implement I2P Proposal 170 without expanding the public protocol, changing router behavior merely for API convenience, or coupling the work to frontend state.

The workstream is pinned to Proposal 170 revision created/updated `2026-05-20`. The proposal remains Open. Completion statements are therefore revision-specific.

Normative references:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`
- `plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`

Detailed subsystem roadmaps:

- historical source/truthfulness baseline: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`;
- containment: `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`;
- tunnel runtime completion: `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`;
- tunnel security hardening: `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- active full-support completion: `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## Phase 1 — Contract and truthful partial runtime

Historical dependency graph:

```text
M001 Contract matrix and I2PControl foundation
    |
    v
M002 Control-plane domain and persistence
    |
    +------------------+-------------------+
    |                  |                   |
    v                  v                   v
M003 AddressBook   M004 TunnelManager   M005 RouterInfo inspection
                       and stubs             |
                         |                   |
                         +---------+---------+
                                   |
                                   v
                         M006 ClientServicesInfo
                                   |
                                   v
                         M007 Conformance and closure
```

M001-M007 established the initial contract. Subsequent corrective/source/containment milestones refined it and established a truthful partial Proposal 170 state.

### Phase-1 durable outcomes

- exact Proposal 170 method/selector/type/action/domain model;
- frontend-independent authenticated JSON-RPC service;
- versioned administrative persistence;
- AddressBook CRUD and subscription architecture;
- exhaustive TunnelManager registration and persistent definitions;
- RouterInfo source inventory and bounded read-only inspection;
- ClientServicesInfo selectors;
- explicit unavailability rather than fabricated values;
- source/dependency containment ultimately fixed by M061-M063.

The later M045-M057 source sequence closed the pre-ADR-0004 RouterInfo baseline as 43 total / 37 available / 1 protocol-permitted neutral / 5 unavailable. That state remains truthful historical/current evidence until the new source-completion milestones close; it is no longer the intended final state.

## Phase 2 — Proposal 170 tunnel runtime completion and security

ADR-0003 intentionally expanded the original stub-only end state to make all twelve declared tunnel families real through bounded I2PControl-owned application adapters while preserving startup/control-plane separation and avoiding router-core tunnel-type logic.

Historical implementation dependency graph:

```text
M064 baseline feature corrective
    |
    v
M065 I2PControl runtime/filter foundation
    |
    +------------------+------------------+------------------+
    |                  |                  |                  |
    v                  v                  v                  v
M066 IRC family    M067 HTTP server   M068 HTTP client/  M071 Streamr
                                         CONNECT
    |                                     |
    v                                     |
M069 SOCKS + SOCKS-IRC                    |
    |                                     |
    +------------------+------------------+
                       |
                       v
                 M070 HTTP bidirectional
                       |
                       v
                 M072 integrated reclosure
                       |
                       v
                 M073 option-truthfulness corrective
```

M074-M094 then hardened and independently reclosed the server/application security boundary, including admission, HTTP/IRC identity/framing/filtering, Streamr bounds, lifetime/half-close behavior, containment rollback after the unauthorized M091 expansion, and final planning reconciliation.

### Phase-2 durable outcomes

- all twelve Proposal 170 tunnel types have real production backends;
- all seven canonical actions operate over persistent control-plane definitions;
- HTTP/IRC server types use accepted application-visible I2P streams and mandatory filters;
- trusted peer identity is Yosemite-derived and canonicalized;
- server admission/rate state is bounded and transactional at the accepted post-accept boundary;
- server local targets remain literal-loopback/confined;
- Streamr remains a bounded datagram subsystem;
- backend-owned persistent server identity/secrets remain redacted;
- runtime-relevant options are applied or rejected before allocation;
- M093 remains the current tunnel production/security reclosure authority;
- M088's lower-layer pre-accept resource/timing residual and Streamr's finite-subscriber availability limitation remain accepted unless separately reopened by direct evidence.

Phase 2 closed tunnel data-plane functionality and security. It did not claim all optional/applicable Proposal 170 option semantics, remaining RouterInfo sources, operational AddressBook SetConfig, or full public-network interoperability.

## Phase 3 — Full support against pinned Proposal 170 revision

Primary class: capability / invariant / containment / operations.

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

ADR-0004 authorizes this phase. It changes the intended final completeness target while preserving historical closure evidence for earlier partial states.

Implementation handoffs continue at M095-M104.

```text
M095 exact full-support matrix + containment budget
  |
  +------------------+-------------------+------------------+
  |                  |                   |                  |
  v                  v                   v                  v
M096 AddressBook   M097 common tunnel  M100 transit 15s   M101 router news
SetConfig          session/key opts    source             source
                       |
                       +----------------------+
                       |                      |
                       v                      v
                 M098 client/proxy      M099 server/
                 management/HTTP opts   LeaseSet/access opts

M095 ----------------------------------------------+
  |                                                |
  v                                                v
M102 canonical network-error owner            M103 banned-peer semantic closure
  |                                                |
  +----------------------+-------------------------+
                         |
M096-M103 all closed ----+
                         |
                         v
                 M104 live interoperability +
                 full Proposal 170 reclosure
```

Only the dependency-ready plan named in `plans/registry.md` is executable. At phase creation, M095 is the sole ready handoff. M096-M104 are prewritten continuity plans and remain blocked until their hard dependencies close.

### M095 — Exact full-support matrix and containment budget

Primary class: invariant / infrastructure.

Create one machine-readable matrix covering:

- all 43 Proposal 170 RouterInfo additions;
- all 13 AddressBook SetConfig keys;
- every canonical TunnelManager option crossed with all 12 tunnel types and classified from actual runtime behavior;
- all 6 ClientServicesInfo selectors;
- explicit out-of-scope classification for unrelated base I2PControl methods;
- exact owner/path budgets for every remaining completion milestone.

No production behavior changes under M095.

### M096 — AddressBook SetConfig operational completion

Primary class: capability / security / persistence.

Implement all thirteen pinned SetConfig keys with one durable configuration authority, bounded worker-generation semantics, and path-confined administrative file ownership. Behaviorally meaningful settings must be consumed; harmless reference UI metadata may round-trip without creating frontend coupling.

No arbitrary remote host filesystem authority, global logger reconfiguration, or incidental resolver-precedence redesign.

### M097 — Common tunnel session/key option completion

Primary class: capability / infrastructure / security.

Complete common tunnel/session/key/persistence options through existing supported Yosemite/SAM/session primitives. Examples expected to include tunnel length/variance/quantity/backup, signing/encryption types, shared/persistent/new destination semantics, confined private-key references, and bounded CustomOptions.

No Proposal-170-shaped core API, Yosemite vendoring/forking, or dependency provenance change is authorized. A missing primitive is a stop condition.

### M098 — Client/proxy/management/HTTP option completion

Primary class: capability / security.

Complete applicable client proxy/outproxy/authentication, lifecycle/management timers, and HTTP privacy options while preserving no-local-DNS, explicit-I2P-outproxy, listener authentication, SOCKS scope, and existing HTTP/IRC filtering.

### M099 — Server/access/throttle/LeaseSet option completion

Primary class: capability / security.

Complete applicable server access/filter/presentation/throttle/POST and LeaseSet encryption/authentication options while preserving M093's trusted peer/admission/filter/loopback boundaries. Tunnel-local temporary denial must not become router-wide peer banning.

### M100 — Request-independent transit 15-second source

Primary class: capability / infrastructure.

Add a feature-gated I2PControl-owned bounded sampler over the existing authoritative cumulative transit-byte counter. Sampling cadence/history is independent of RouterInfo requests. No router-core timer or traffic instrumentation.

### M101 — Router news source

Primary class: capability / security / operations.

Establish the exact pinned/reference news source/format, then implement a bounded I2PControl-owned source/cache/parser with truthful failure/staleness semantics. No arbitrary public-web substitute, per-request fetch, or core news subsystem.

### M102 — Canonical IPv4/IPv6 network-error owner

Primary class: capability / containment.

After M095 freezes the code/source/writer table, add only the smallest neutral lower-layer error-reason state if existing accepted inspection cannot already provide it. Proposal 170/i2pd integer mapping remains in I2PControl. Observation must not alter transport/reachability behavior.

### M103 — Banned-peer semantic completion

Primary class: capability / invariant.

Exhaustively determine whether Emissary already has a router-wide enforced ban/exclusion owner. If yes, expose a bounded neutral snapshot. If the router has no possible banned state by design, an authoritative empty result may be used only with durable proof. Do not create a peer-ban algorithm solely for telemetry. If neither semantic is truthful, full-support work remains blocked pending a new architecture decision.

### M104 — Live interoperability and full reclosure

Primary class: invariant / capability / operations.

Independently reconcile the final matrix and run focused real-network/reseeded interoperability, including real data-plane traffic for all twelve tunnel families, all RouterInfo additions, AddressBook SetConfig/restart, ClientServicesInfo, persistence, security regression, containment, and default-feature review.

Only M104 may change the top-level support statement to:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision.

That statement does not imply general I2PControl parity or upstream review/acceptance.

## Phase-3 invariants

- Proposal 170 remains pinned to revision `2026-05-20`; later draft changes require a separate delta audit.
- No new Proposal 170 method, selector, field, alias, action, tunnel type, or public status is introduced.
- New administrative/application/source policy stays under `emissary-cli/src/i2pcontrol/**` wherever technically possible.
- Existing M061/M062/M063 containment authority remains in force; lower-layer changes require exact pre-implementation path budgets.
- Existing non-I2PControl paths are a ceiling, not blanket permission.
- M102 is the only currently anticipated neutral-core owner addition; it is blocked until M095 proves the exact need/writers/paths.
- M103 cannot create router ban behavior solely for a getter.
- AddressBook/config/key/filter paths are confined to owned administrative roots.
- Runtime options remain fail-before-allocation until their exact semantics close; persist-and-ignore stays forbidden.
- Existing HTTP/IRC/Streamr/server security boundaries are not weakened for option parity.
- Full support requires live interoperability evidence, not parser/unit tests alone.
- Verification remains focused/local/package-scoped; no new hosted CI/fuzz/coverage/release apparatus is required by default.
- No upstream interaction, submission, review, adoption, merge, contribution preparation, or maintainer contact is authorized.

## Deferred work outside this roadmap

The following remain separate unless a later explicit maintainer directive changes them:

- unrelated base I2PControl methods not added by Proposal 170;
- runtime lifecycle migration/adoption of existing startup-managed tunnels and proxies;
- tunnel types not declared by pinned Proposal 170;
- frontend management/UI for I2PControl resources;
- DCC/WEBIRC/SOCKS BIND/UDP ASSOCIATE or other non-required protocol subfeatures;
- broad router peer-ban/reputation policy unrelated to existing canonical semantics;
- changes to router protocol, peer selection, NetDB, tunnel-building algorithms, transport retry/congestion, or cryptographic wire behavior;
- broad hosted CI/release infrastructure;
- upstream contribution/review/merge activity.

The previously deferred RouterInfo news/banned-peer/transit/error rows, AddressBook SetConfig, full applicable TunnelManager option semantics, and focused cross-router/live interoperability are no longer deferred: ADR-0004 moves them into Phase 3 under M095-M104.

## Roadmap completion rule

The current partial state remains truthful until the relevant Phase-3 milestones close.

The roadmap closes as `closed internally against pinned revision` only when M104 demonstrates:

- complete exact matrix coverage for the pinned proposal;
- no applicable unavailable RouterInfo row;
- operational AddressBook SetConfig across all thirteen keys;
- all twelve real tunnel types and all applicable option/type cells operational;
- six ClientServicesInfo selectors exact/current;
- security/persistence/restart/failure/contention invariants preserved;
- minimal/explicit containment with no unplanned core/dependency expansion;
- focused live interoperability/data-plane evidence;
- default/feature-disabled Emissary unaffected;
- no upstream interaction.

If a genuine missing router capability prevents a required Proposal 170 semantic without unacceptable scope expansion, the roadmap remains blocked rather than weakening the full-support definition or fabricating data.