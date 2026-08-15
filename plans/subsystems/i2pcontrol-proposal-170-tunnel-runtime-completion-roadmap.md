# I2PControl Proposal 170 Tunnel Runtime Completion Roadmap

Status: active; M064, M065, M066, and M067 closed; M068 is the next registered handoff

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6` — current production head reviewed before this planning series

Pinned external authority:

- I2P Proposal 170, revision created/updated `2026-05-20`;
- SAM v3 streaming/datagram behavior exposed through the repository's Yosemite dependency;
- Java I2PTunnel source/documentation as read-only behavioral/security reference only.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`;
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`.

## 1. Purpose

Complete the ten Proposal 170 tunnel families that were initially explicit unsupported backends while preserving the central containment requirement: Proposal 170-specific runtime policy and application-protocol logic should remain in `emissary-cli/src/i2pcontrol/**`, and the already security-reviewed router core should not acquire new tunnel-type behavior.

This roadmap does not treat byte forwarding as sufficient completion for specialized protocols. HTTP and IRC server/client types are security adapters. Their filtering/normalization behavior is required to prevent local/server identity leaks, spoofed proxy metadata, request-smuggling ambiguity, direct-connect address leakage, and protocol-confusion exposure.

The target production registry evolves monotonically:

```text
current:
  client, server = real
        remaining eight = explicit UnsupportedTunnelBackend

phase progress:
  each family replaces only its own stub after independent closure

final target:
  all twelve Proposal 170 types = real backend
  no public TunnelManager schema/action/type redesign
```

At every intermediate revision, every type must remain exhaustively registered and truthful.

## 2. Current-state evidence

At production baseline `a1296b0`:

- `TunnelType` contains exactly the twelve pinned Proposal 170 types;
- `TunnelBackendRegistry` is exhaustive;
- production registry maps `client` and `server` to real backends and every other type to `UnsupportedTunnelBackend`;
- generic client/server control-plane supervisors already provide named lifecycle isolation;
- control-plane server destination identity is stored through the backend-owned `ServerDestinationStore`;
- specialized runtime files do not yet exist under `i2pcontrol/backends`;
- existing startup HTTP and SOCKS services demonstrate available application dependencies and SAM/Yosemite connectivity but remain startup-owned;
- current generic server runtime uses SAM forwarding and therefore cannot implement HTTP/IRC filtering before the local service sees remote bytes;
- Yosemite/SAM provides the application boundary required for independently accepted streams and datagrams, so no new router-core data-plane API is expected;
- the current head has a small `emissary-core/src/events.rs` feature-disabled warning regression (`testing` is unused when `events` is disabled), which must be repaired separately before new tunnel runtime work so later verification begins from a clean baseline.

Accepted containment state:

- M061 owns the exact non-I2PControl source-boundary authority;
- M062/M063 own the direct-dependency/feature-activation authority;
- new files under `emissary-cli/src/i2pcontrol/**` are the preferred expansion location;
- default `emissary-cli` does not enable the `i2pcontrol` feature.

Accepted Proposal 170 source state remains 43 RouterInfo additions / 37 available / 1 protocol-permitted neutral / 5 unavailable. This roadmap does not reopen those rows.

## 3. Ownership architecture

### 3.1 I2PControl owns specialized application adapters

The preferred implementation shape is:

```text
emissary-cli/src/i2pcontrol/
    backends/
        ... real tunnel-family backends ...
        runtime/
            ... small lifecycle/listener/session helpers ...
        filters/
            ... HTTP/IRC protocol sanitizers ...
```

Exact naming is not normative. Ownership is.

The new code may depend on existing application-level crates/dependencies already used by `emissary-cli`, including Tokio, Yosemite/SAM, HTTP parsing utilities, URL handling, and standard library collections/network types.

### 3.2 No new core runtime surface by default

No M065-M072 implementation plan authorizes a new `emissary-core/**` production path or new core stream/session/control API.

If a family cannot be implemented through existing SAM/Yosemite and application-layer state, the implementation agent must stop that family and document the exact missing primitive. It must not add a convenience core API inside the family plan.

M064 is the sole planned core edit: a narrow correction to an already-existing M050-era observation method so feature-disabled/no-std builds do not fail on an unused parameter. M064 does not add behavior.

### 3.3 Startup services remain separate

Existing `emissary-cli/src/proxy/http/**`, `proxy/socks/**`, and startup tunnel managers may be read as internal behavioral evidence, but new TunnelManager backends do not gain ownership over their listeners/tasks.

Do not refactor startup services into shared lifecycle managers merely to reduce conceptual duplication. A small neutral helper outside `i2pcontrol` is allowed only after the owning implementation plan proves it is simpler, does not expose administrative types/policy, and fits the accepted containment authority; otherwise keep the implementation local.

### 3.4 Server destination ownership

`httpserver`, `ircserver`, `streamrserver`, and `httpbidirserver` should reuse the existing backend-owned server secret-store authority wherever they require persistent published identity.

Do not honor arbitrary private-key filesystem paths from raw configuration. Proposal 170 configuration spelling may round-trip, but runtime secrets remain path-confined and backend-owned.

## 4. Common security and lifecycle invariants

Every family milestone must preserve all of the following:

1. exact Proposal 170 tunnel type/action/field names;
2. no new public capability/status extension fields;
3. unsupported backend remains active for a family until the family's real backend satisfies closure;
4. a real backend validates required fields and runtime option support before binding/listening/session creation;
5. security-sensitive relevant options are applied or rejected, never silently ignored;
6. secrets are redacted from errors/logging/Debug/API responses;
7. startup-managed resources remain externally owned;
8. lifecycle operations target one exact control-plane name/generation;
9. no lock is held across network I/O, sleeps, task joins, or cancellation waits;
10. task/listener/session/subscriber sets are bounded by existing inventory/resource limits;
11. stop is idempotent and restart is completed stop followed by new start;
12. bind/SAM/target-connect/filter failure affects only the target tunnel and leaves durable definition recoverable;
13. no local DNS lookup for direct `.i2p` routing;
14. clearnet access requires an explicitly configured I2P outproxy;
15. localhost/LAN targets are not remotely selectable through application input;
16. default/feature-disabled Emissary behavior remains unaffected;
17. no new CI/release/fuzz/coverage/platform machinery is introduced merely for this work;
18. no upstream or third-party write/review/submission activity occurs.

## 5. Application filter contracts

### 5.1 HTTP request filter

The HTTP parser/filter used by specialized tunnels must be independently authored and bounded. Required properties include:

- request-line timeout and length limit;
- header-completion timeout;
- per-header-line, header-count, and aggregate header-size limits;
- rejection of embedded NUL/control injection and obsolete folded headers;
- deterministic handling/rejection of duplicate `Host`;
- deterministic framing checks for duplicate/conflicting `Content-Length` and `Transfer-Encoding` combinations;
- no ambiguous request target normalization;
- removal of incoming caller-supplied I2P identity headers before trusted injection;
- explicit handling/removal of `Forwarded`, `Via`, `Proxy-*`, `X-Forwarded-*`, and other proxy identity headers according to backend role;
- Host rewriting when `WebsiteHostname`/equivalent configured semantics require it;
- bounded request-body forwarding and cancellation semantics without buffering arbitrary bodies;
- connection upgrade/websocket behavior either correctly supported or explicitly rejected per backend capability.

### 5.2 HTTP response filter

For server tunnels, local HTTP response headers must be parsed before exposure to I2P so obviously identifying/proxy headers can be removed. Minimum identity/fingerprint targets include `Server`, `X-Powered-By`, `X-Runtime`, proxy-only headers, and hop-by-hop state. `Date` handling should follow the adopted anonymity/reference behavior documented by the implementation plan.

Response parsing must not buffer an unbounded body. After a bounded/validated header block, payload streaming may proceed according to framing semantics.

### 5.3 IRC client filter

The common IRC client-side filter must:

- enforce a bounded line length;
- understand optional IRCv3 message tags and command prefixes sufficiently to classify messages safely;
- permit ordinary command/numeric traffic through an explicit allowed policy;
- rewrite or reject `USER` forms that expose client hostname/servername;
- sanitize PING/PONG forms that may expose local/proxy address information;
- sanitize PART/QUIT user-provided text where required by the adopted anonymity policy;
- permit normal PRIVMSG/NOTICE but reject unsafe CTCP except explicitly supported safe forms such as ACTION;
- reject DCC CHAT/SEND/RESUME/ACCEPT during this roadmap unless a later dedicated plan is authorized;
- preserve CAP/SASL and common IRCv3 behavior required for contemporary clients.

### 5.4 IRC server registration filter

Before connecting the accepted I2P stream to the local IRCd, the server filter must:

- apply total registration timeout;
- enforce maximum line length and maximum pre-USER/SERVER line count;
- reject obvious HTTP/BitTorrent/other cross-protocol signatures;
- parse USER/SERVER registration safely;
- derive the presented peer hostname/cloak from actual I2P peer identity;
- never trust a remote-supplied claimed hostname as peer identity;
- support only the explicitly planned USER-cloak mode initially; WEBIRC remains rejected unless separately implemented.

## 6. Proxy-family policies

### HTTP client

- local listener is control-plane-owned;
- `.i2p`/B32 resolution stays within I2P/address-book mechanisms;
- clearnet requires configured I2P HTTP outproxy;
- User-Agent, Referer, Accept-family, forwarding/proxy headers follow explicit anonymity policy and Proposal 170 options;
- proxy authentication is required when configured and should become mandatory for unsafe non-loopback exposure according to the plan's target policy;
- requests unsupported by the implemented parser fail locally rather than becoming raw byte tunnels accidentally.

### CONNECT client

- accepts only CONNECT;
- bounded initial header read;
- direct-I2P request does not forward arbitrary browser proxy headers;
- validates host/port and blocks localhost/LAN/unsafe clearnet paths;
- on success becomes a raw byte tunnel only after target resolution/SAM connection succeeds.

### SOCKS

Initial complete scope:

- SOCKS4a TCP CONNECT;
- SOCKS5 TCP CONNECT;
- domain-name I2P targets;
- optional configured I2P outproxy behavior if Proposal 170 configuration maps cleanly;
- bounded negotiation time/size;
- authentication as required by Proposal 170 options/non-loopback safety policy.

Initial non-goals:

- BIND;
- UDP ASSOCIATE;
- torsocks-specific resolve extensions unless required to satisfy the pinned contract;
- local DNS of arbitrary names.

### SOCKS-IRC

SOCKS negotiation is identical to the accepted SOCKS frontend, followed by the exact accepted IRC filter. No duplicate IRC policy table is allowed.

## 7. Streamr policy

Streamr is a datagram family and remains intentionally separate from streaming TCP helpers.

`streamrserver`/producer responsibilities:

- persistent I2P datagram identity/session;
- bounded local UDP ingest;
- one-byte subscribe/unsubscribe control semantics compatible with adopted reference behavior;
- subscriber identity includes remote I2P destination and relevant from/to port context;
- maximum subscriber count;
- subscription expiry and refresh;
- packet-size/rate bounds preventing trivial amplification/resource exhaustion;
- clean cancellation/restart.

`streamrclient`/consumer responsibilities:

- I2P datagram client session;
- periodic subscribe refresh with bounded cadence;
- explicit unsubscribe on graceful stop where possible;
- forward received payloads only to configured local UDP target;
- no arbitrary packet-driven local target selection.

## 8. Dependency graph

```text
M064 baseline feature-regression corrective
    |
    v
M065 common runtime/option-validation foundation
    |
    +--------------------+--------------------+--------------------+
    |                    |                    |                    |
    v                    v                    v                    v
M066 IRC family      M067 HTTP server    M068 HTTP client/     M071 Streamr
                                          CONNECT
    |
    v
M069 SOCKS + SOCKS-IRC

M067 + M068
    |
    v
M070 HTTP bidirectional composition

M066 + M067 + M068 + M069 + M070 + M071
    |
    v
M072 integrated tunnel-runtime reclosure
```

Dependency classes:

- M064 -> M065: hard, because new work should not build atop a known feature-disabled core regression;
- M065 -> M066/M067/M068/M071: hard, because all families need the common lifecycle/option-capability contract;
- M066 -> M069: interface/hard for `socksirc`; SOCKS negotiation could be written independently, but M069 cannot close until it reuses the accepted IRC filter;
- M067 + M068 -> M070: hard;
- all family milestones -> M072: hard.

Only the next dependency-ready plan is registered in `plans/registry.md`. Future plans exist for handoff continuity but remain blocked until dependencies close.

## 9. Milestones and exit conditions

### M064 — Current-head baseline feature-regression corrective

Plan: `064-proposal-170-tunnel-runtime-baseline-corrective.md`.

Status: closed. Closure: `plans/closure/i2pcontrol-proposal-170/064-closure.md`.

Repair only the existing feature-disabled/no-events unused-parameter regression in `emissary-core/src/events.rs`. Prove feature-off/no-std and normal builds are back to the accepted state. No new runtime feature.

Exit: clean baseline and exact single-defect correction.

### M065 — I2PControl-owned runtime/filter foundation

Plan: `065-i2pcontrol-tunnel-runtime-primitives.md`.

Status: closed; hard dependency M064 is closed. Closure:
`plans/closure/i2pcontrol-proposal-170/065-closure.md`.

Add bounded reusable control-plane primitives for local listener ownership, accepted-stream server ownership, per-name task generation/cancellation where not already reusable, peer identity delivery to filters, and deterministic per-backend option-capability validation. Keep all production work under `i2pcontrol`.

Exit: test backends demonstrate outgoing/accepted stream lifecycle, cancellation, peer identity, and fail-before-allocation option rejection without registering a previously unsupported production type. M066, M067, M068, and M071 were dependency-ready successors; M066 is now closed and M067 is the next registered handoff.

### M066 — IRC client/server family

Plan: `066-irc-client-server-tunnel-family.md`.

Implement common IRC filter, real `ircclient`, and filtered accepted-stream `ircserver`. DCC and WEBIRC fail closed.

Status: closed; closure: `plans/closure/i2pcontrol-proposal-170/066-closure.md`.

Exit: both types are real and independently security-tested; no local-address/registration bypass path is known.

### M067 — HTTP server

Plan: `067-http-server-tunnel.md`.

Implement the security-critical accepted-stream HTTP server with request/response filters, identity-header handling, Host policy, framing checks, time/size bounds, concurrency and configured throttles.

Status: closed; closure: `plans/closure/i2pcontrol-proposal-170/067-closure.md`.

Exit: `httpserver` replaces its stub only after negative security tests prove the local backend never sees unsanitized initial headers.

### M068 — HTTP client and CONNECT client

Plan: `068-http-client-and-connect-tunnels.md`.

Implement control-plane HTTP proxy and CONNECT-only proxy with anonymity-sensitive request sanitization, `.i2p` routing, explicit outproxy handling, local-target restrictions, auth/exposure policy, and strict option capability enforcement.

Exit: `httpclient` and `connectclient` real, with no local DNS/open-LAN route and no silent security-option ignores.

### M069 — SOCKS and SOCKS-IRC

Plan: `069-socks-and-socks-irc-tunnels.md`.

Implement SOCKS4a/5 TCP CONNECT frontend, safe target routing, auth/exposure policy, and `socksirc` composition using M066's IRC filter.

Exit: both types real; unsupported SOCKS commands fail correctly; SOCKS-IRC has no alternate/unfiltered relay path.

### M070 — HTTP bidirectional server composition

Plan: `070-http-bidirectional-server-composition.md`.

Compose closed M067 and M068 runtime/filter components into `httpbidirserver` with no outproxy role and shared server identity/session semantics as required.

Exit: no third HTTP parser/filter implementation exists; lifecycle cancels both halves exactly.

### M071 — Streamr client/server

Plan: `071-streamr-client-server-tunnels.md`.

Implement bounded datagram producer/consumer behavior, subscription refresh/expiry, UDP local endpoints, packet/subscriber limits, and exact lifecycle.

Exit: both Streamr types real with bounded state and no amplification/unbounded subscription issue.

### M072 — Integrated tunnel-runtime reclosure

Plan: `072-tunnel-runtime-completion-reclosure.md`.

Reconcile production registry, all twelve types, option capability matrices, support documentation, containment, feature-disabled/default behavior, persistence/restart, lifecycle contention, and security evidence. No new family implementation should originate here except tiny directly demonstrated corrective fixes within predeclared budgets; material findings generate a new corrective plan.

Exit: final support statement is evidence-backed and no high/medium correctness/security/containment finding remains.

## 10. Verification discipline

Family milestones use focused local tests and only the broad checks needed to prove containment/compatibility. Preferred commands include package-scoped `cargo test`/`cargo check`, feature-disabled checks, existing M061/M062 containment tests, and focused live/fake-SAM fixtures.

Do not add hosted CI jobs, fuzz infrastructure, coverage gates, long-running soak harnesses, Docker interoperability farms, or release automation merely because protocol parsers are being added. Targeted parser/property-style unit cases may be extensive but remain normal Rust tests.

Where reference interoperability matters, use deterministic fixtures or a bounded local Java/I2P test only if already available and necessary. Do not turn this roadmap into an external certification program.

## 11. Documentation and truthful support state

`docs/i2pcontrol/proposal-170-support.md` must be updated as each family closes. It must distinguish:

- real types;
- still-unsupported types;
- supported vs rejected runtime options;
- intentionally unsupported DCC/WEBIRC/SOCKS commands/other subfeatures;
- environmental qualification for live public-network proof.

A backend's type must not be documented operational merely because its parser/tests exist; closure requires a real start/traffic/stop path.

## 12. Risks and mitigations

| Risk | Mitigation |
|---|---|
| HTTP hidden service leaks backend/software identity | mandatory response filtering; trusted identity injection; negative tests |
| Request smuggling reaches local server | bounded parser; framing ambiguity rejection before local connect |
| Remote user spoofs reverse-proxy identity | strip/rebuild forwarding and I2P identity headers |
| IRC leaks local IP through USER/PING/DCC | common fail-closed IRC filter; DCC rejected |
| SOCKS/CONNECT becomes open clearnet/LAN proxy | explicit I2P outproxy only; no local DNS; local-target blocks; auth/exposure checks |
| Specialized runtime contaminates core | zero-new-core-path rule and M061 changed-path review |
| Startup tasks are accidentally controlled | retained ADR-0002 ownership tests |
| Proposal options appear accepted but do nothing | backend option-capability validation before allocation |
| Parser work drives dependency growth | prefer existing dependencies; M062/M063 feature ownership if addition unavoidable |
| HTTP bidir duplicates filters | hard dependency on M067/M068 and composition-only acceptance criterion |
| Streamr creates unbounded multicast state | fixed subscriber/packet/rate/expiry bounds |
| GPL reference implementation is copied | read-only behavioral research; independently authored Rust/fixtures |
| Planning work drifts into upstream activity | Section 11 governance/internal-only attestation in every closure |

## 13. Stop conditions

A family implementation must stop and require corrective/replanning if:

- it needs a new router-core mutation/networking API;
- the pinned Proposal 170 revision materially changes relevant type/options;
- a required security filter cannot be applied before unsafe bytes reach the local service;
- the implementation would require adopting startup-owned tasks;
- a requested option cannot be implemented truthfully and the handler/backend cannot reject it without public protocol expansion;
- dependency additions would contaminate default/feature-disabled builds;
- licensing requires copying incompatible reference source rather than independently implementing behavior;
- a high/medium security finding remains unresolved at closure.

## 14. Final closure statement

M072 may close this roadmap only when the registry has real backends for all ten newly authorized families, their security/option capability sets are truthfully documented, HTTP/IRC filtering is non-bypassable, containment/default-build invariants remain intact, and no upstream interaction occurred. If any declared family remains stubbed, the roadmap remains partial and the exact blocker must be named rather than hidden.
