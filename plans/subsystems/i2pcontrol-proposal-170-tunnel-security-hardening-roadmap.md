# I2PControl Proposal 170 Tunnel Security Hardening Roadmap

Status: planned corrective security work; M075 closed; M076 next; M077 ready; M078-M079 blocked

Planning production baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Source runtime roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`;
- M061 source-containment and M062/M063 dependency-containment authorities;
- M072 closure and M073 generic option-truthfulness corrective.

Pinned external contract:

- I2P Proposal 170, `I2PControl Expansion`, Open revision created/updated `2026-05-20`.

Read-only reference snapshots inspected on `2026-08-15`:

- Java I2P `i2p/i2p.i2p` at `498488b0d01d9f59efe906424e56ff5e25f58a4d`;
- I2P+ `I2PPlus/i2pplus` at `5cd400e7f6d3b4432450d2f401d131897ac6998b`.

Reference source is evidence only. No copying, upstream mutation, review request, submission, or contribution preparation is authorized.

## 1. Purpose

The M066-M071 sequence made the ten previously deferred Proposal 170 tunnel families operational. A subsequent security/anonymity review found that the implementation is bounded in several important ways but is not yet equivalent to the mature server-side abuse and fingerprint defenses present in Java I2PTunnel and I2P+.

This roadmap hardens the newly introduced tunnel data planes against attacker-controlled resource exhaustion, load-state/timing correlation, backend fingerprint leakage, long-lived idle occupancy, limiter-state churn, and unsafe local UDP exposure. It does not expand Proposal 170, add tunnel types, redesign router protocols, or turn I2PControl into a general WAF.

The principal anonymity threat is not that any one observable immediately reveals the host IP. The threat is that an I2P peer can deliberately create a stable externally observable condition — saturation, backend-specific response metadata, application clock information, or injected local traffic — and correlate that condition with a candidate host/service outside I2P.

## 2. Research findings that control this roadmap

### 2.1 Java I2P server admission is layered

Current Java I2P retains server defaults in `TunnelController` for:

- per-peer connections: 30/minute, 80/hour, 200/day;
- aggregate connections: 50/minute, unlimited hour/day by default;
- concurrent streams: 30.

When no server limits are explicitly configured, those defaults are installed. The streaming `ConnectionManager` enforces global concurrent streams plus peer-keyed and aggregate minute/hour/day throttlers against the authenticated remote `Destination`/hash. The base I2PTunnel server also uses bounded handler execution and resets/drops work when its executor cannot accept more.

Security conclusion: a single global semaphore is necessary but not sufficient. Emissary needs peer-keyed and aggregate admission state before a remote peer can consume application handler/local-target resources.

### 2.2 Java HTTP throttling keeps peer and total state separate

`I2PTunnelHTTPServer` uses `ConnThrottler`, which separately tracks per-peer and aggregate POST/PUT activity and has distinct peer/total throttle periods. `ConnThrottler` explicitly describes itself as basic DoS protection rather than a complete solution.

Security conclusion: bounded state must not evict an active abusive peer merely to admit a new attacker-controlled identity. Eviction-based limiter churn converts a memory bound into a bypass.

### 2.3 HTTP response metadata is an anonymity boundary

Java I2P removes at least `Date`, `Server`, `X-Powered-By`, `X-Runtime`, `Proxy`, and `Proxy-Connection` from hidden-service HTTP responses.

Current I2P+ goes further and removes headers including `Age`, `Alt-Svc`, `Date`, `Expires`, `Pragma`, `Server`, `Strict-Transport-Security`, `Via`, cache headers, cloud/hosting trace identifiers, `X-Powered-By`, `X-Runtime`, `X-Served-By`, and related provider-specific identifiers. It also strips additional request-side proxy/privacy metadata including `X-Real-IP`.

Security conclusion: Emissary must at minimum achieve Java parity, and should adopt the stronger I2P+ anonymity-oriented denylist where it does not break HTTP framing semantics. `Date` is a required correction because local clock/time behavior is a correlation signal.

### 2.4 Java IRC bounds both registration and the later connection

Java I2P applies a bounded registration phase and then sets a 10-minute IRC read timeout, with the expectation that the IRC application itself will keep live users active through PING/PONG.

Security conclusion: Emissary's current bounded registration followed by unbounded `io::copy` lifetime permits registered-idle connections to pin accepted-server capacity indefinitely. The correction must be an inactivity deadline that resets on traffic, not a fixed total session lifetime.

### 2.5 Streamr is intentionally small and bounded

Java Streamr keeps at most 10 subscriptions and expires them after 60 seconds. Emissary already has stronger packet-size bounds, exact one-byte control parsing, fixed subscription expiry, and no unbounded send-task fanout, but currently permits administrator-selected non-loopback UDP ingress/egress.

Security conclusion: the remaining anonymity/integrity issue is the local UDP trust boundary. Without an authenticated publisher protocol in Proposal 170, non-loopback UDP exposure should fail closed rather than allow unauthenticated LAN/external traffic to become an I2P-correlatable stream.

## 3. Current Emissary risk inventory

At baseline `04e0c2e`:

1. `run_accepted_server` has a global bounded task group but no peer-keyed admission/fairness layer.
2. `httpserver`, `ircserver`, and inbound `httpbidirserver` can therefore expose a remotely manipulable saturation state.
3. generic `server` still uses SAM `STREAM FORWARD`, bypassing the accepted-stream peer/admission seam entirely.
4. `httpserver` response filtering does not remove `Date` and is substantially narrower than I2P+.
5. the HTTP POST limiter evicts the oldest entry after its map bound, so identity churn can reset active limiter state.
6. `ircserver` has strict registration bounds but no post-registration inactivity limit.
7. Streamr producer/client local UDP endpoints may be configured non-loopback despite no local publisher authentication model.
8. M073 independently owns generic option apply-or-reject truthfulness and remains a hard predecessor. This roadmap must not duplicate or race that correction.

## 4. Security invariants

All M074-M079 work MUST preserve:

- exact Proposal 170 wire fields/actions/types; no security-only public extensions;
- authenticated remote identity from SAM/Yosemite only, never application headers;
- security policy decisions before local target connection where technically possible;
- bounded tasks, peer state, counters, buffers, and shutdown waits;
- no lock held across network I/O, sleeps, target connection, or task joins;
- no active abusive identity may be evicted solely to make room for an untrusted new identity;
- monotonic time for rate/idle state;
- no private destination material in logs/errors/Debug/API output;
- no artificial response jitter or sleep-based pseudo-constant-time defense;
- no local DNS/direct LAN routing expansion;
- no new `emissary-core/**` production path;
- no startup-service ownership refactor;
- no hosted CI/fuzz/soak/release machinery added for this work;
- no upstream or third-party write activity.

## 5. Target architecture

The accepted-stream server boundary should become:

```text
SAM/Yosemite accepted stream
    -> validated TrustedPeerIdentity
    -> shared I2PControl-owned ServerAdmissionState
       - global concurrent permits
       - peer concurrent permits
       - peer minute/hour/day counters
       - aggregate minute/hour/day counters
       - bounded deny/expiry state
    -> protocol-specific pre-local validation
    -> fixed administrator-selected loopback target
    -> bounded protocol relay/filter
```

The shared admission component is infrastructure, not a general router service. It belongs under `emissary-cli/src/i2pcontrol/backends/**` and is consumed only by Proposal 170 control-plane tunnel backends.

For generic `server`, M075 replaces blind `STREAM FORWARD` with application-visible accepted streams followed by an otherwise raw relay. The generic server does not gain HTTP/IRC parsing; it gains only peer-aware admission and fixed local-target ownership.

## 6. Timing/correlation policy

This roadmap does not attempt to make interactive network protocols constant-time. Application response time, congestion, and I2P path latency remain observable.

The target is narrower and actionable:

- prevent one peer from deterministically driving global handler saturation;
- prevent indefinite idle pinning where the protocol has an established inactivity bound;
- avoid exposing backend clock/provider/cache identity in HTTP metadata;
- avoid creating distinguishable local-error details;
- reject overload promptly rather than adding attacker-amplifiable sleeps;
- test fairness and saturation behavior directly.

Random jitter, fixed rejection delays, and padding schemes are explicit non-goals unless a later separately researched anonymity design demonstrates measurable benefit.

## 7. Proposal 170 option policy

M074/M075 must apply or reject the existing server controls before allocation. The named minute/hour/day fields have direct Java streaming analogues and should be implemented:

- `MaxConcurrentConns`;
- `ClientPerMinute`;
- `ClientPerHour`;
- `ClientPerDay`;
- `TotalInPerMinute`;
- `TotalInPerHour`;
- `TotalInPerDay`.

Reference-scale defaults should be 30 global concurrent streams, 30/80/200 per peer, and 50/0/0 aggregate minute/hour/day when no explicit values are supplied, unless current persisted semantics require a compatibility-preserving distinction. The hard maximum may remain the existing Emissary 128 ceiling; increasing it is out of scope.

Proposal 170 currently lists `PerClientPeriod`, `TotalPeriod`, and `TotalBanTime` without enough normative unit/precedence detail to justify inventing behavior. Implementation MUST search the pinned reference/config sources again. If exact semantics remain unavailable, these fields MUST stay explicit fail-before-allocation unsupported options rather than receive guessed semantics.

`FilterFilePath`, `UniqueLocalAddressPerClient`, and `MultiHoming` likewise remain apply-or-reject fields. This roadmap does not implement a new filter-file language, network namespace/address allocator, or multi-home router feature merely for checkbox parity.

## 8. Milestones and dependency graph

```text
M073 generic option truthfulness corrective
    |
    v
M074 shared accepted-server admission/rate hardening
    |\
    | +--------------------+
    v                      v
M075 generic server     M076 HTTP anonymity/POST hardening
accepted-stream            |
hardening                  +------------------+
    |                                         |
    +------------------+                      v
                       |                  M077 IRC lifetime hardening
                       |
                       +--------------------------+
                                                  \
M073 ----------------------------------------------> M078 Streamr local-boundary hardening

M074 + M075 + M076 + M077 + M078
    |
    v
M079 integrated tunnel-security reclosure
```

Dependency classes:

- M073 -> M074: hard. Do not add new server-option semantics while generic option truthfulness is still unresolved.
- M074 -> M075: hard. Generic accepted-stream migration must reuse the accepted admission component rather than create a second limiter.
- M074 -> M076/M077: hard for shared admission behavior; HTTP/IRC protocol-specific hardening remains locally owned.
- M073 -> M078: hard for current registry sequencing; technically Streamr is independent, but only one dependency-ready handoff is registered at a time.
- M074-M078 -> M079: hard.

## 9. Milestone summary

### M074 — Shared server admission and rate-limit hardening

Implement the common peer-aware admission layer and integrate it into `httpserver`, `ircserver`, and inbound `httpbidirserver`. Introduce a finite per-peer concurrent ceiling in addition to the configured global ceiling; default no higher than 8 per peer, with no new public field. Apply the supported Proposal 170 connection-rate controls and safe reference-scale defaults. Denied streams must not reach protocol handlers/local targets.

### M075 — Generic server accepted-stream hardening

Replace control-plane generic server blind `STREAM FORWARD` with accepted-stream raw relay so generic servers participate in peer-aware admission. Preserve fixed loopback target and byte transparency after admission. No router-core API change.

Status: closed; closure: `plans/closure/i2pcontrol-proposal-170/075-closure.md`.

### M076 — HTTP server anonymity and POST-throttle hardening

Bring response filtering to Java parity and adopt the non-framing I2P+ fingerprint denylist; include `Date` and request-side `X-Real-IP`/equivalent proxy identity handling. Replace eviction-bypassable POST limiter state with bounded fail-closed state.

Status: ready; next registered handoff after M075.

### M077 — IRC server lifetime/exhaustion hardening

Add a 10-minute post-registration inactivity deadline that resets on successful traffic, target-connect timeout, and cancellation-safe relay behavior. Preserve the accepted registration filter and raw post-registration IRC semantics.

### M078 — Streamr local-boundary hardening

Make local UDP producer/client targets loopback-only for this Proposal 170 implementation, reduce maximum subscribers to the Java reference ceiling of 10, preserve 60-second expiry/15-second refresh/1200-byte payload bound, and revalidate destination/control bounds.

### M079 — Integrated tunnel-security reclosure

Independently re-audit all newly introduced tunnel families and the generic server after the corrective sequence. The workstream may not close with a high/medium anonymity, resource-exhaustion, option-truthfulness, containment, or lifecycle finding.

## 10. Verification discipline

Use focused deterministic tests, fake/local SAM endpoints, local TCP/UDP capture services, Tokio paused-time tests, package-scoped `cargo test`/`cargo check`, existing M061/M062/M063 containment suites, feature-disabled checks, Clippy, scoped nightly rustfmt where the repository already requires it, and `git diff --check`.

Do not create public-network certification, load-test farms, hosted CI jobs, generalized fuzz infrastructure, or benchmark gates. Small deterministic stress tests such as 31 concurrent streams, peer-cap exhaustion, limiter-map saturation, and paused-time expiry are sufficient for these correctness/security contracts.

## 11. Stop conditions

Stop the affected milestone and record a corrective/architecture blocker if:

- peer-aware accepted streaming requires a new `emissary-core/**` API;
- a required Proposal 170 field cannot be given authoritative semantics and cannot be rejected truthfully;
- generic server accepted-stream relay changes wire payload semantics;
- response filtering would require body inspection/content rewriting;
- IRC idle timeout cannot be implemented as inactivity rather than total lifetime;
- Streamr non-loopback exposure is required for compatibility but cannot be authenticated safely within the pinned schema;
- a dependency addition breaks M062/M063 feature ownership;
- a high/medium finding remains after M079.

## 12. Final closure rule

M073 alone is no longer sufficient to declare the tunnel-runtime security phase complete. The post-M072 security review establishes additional corrective work. The tunnel runtime/security workstream remains corrective until M079 closes with explicit evidence that the new server types no longer expose the identified controllable saturation, idle-pinning, HTTP fingerprint, limiter-churn, or local-UDP trust-boundary defects.
