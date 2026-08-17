# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies and interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency or evidence requirement prevents progress.
- **closing** — implementation landed and independent closure evidence is being gathered.
- **closed** — closure record accepted.
- **closed internally against pinned revision** — internal closure accepted against an explicitly named revision of an open external specification; does not imply upstream review or acceptance.
- **partial Proposal 170 support** — exact supported dimensions are closed, but one or more pinned source/runtime capabilities remain truthfully unavailable.
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 source/truthfulness | partial Proposal 170 support; M057 closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | no source-completion handoff | M051 remains blocked by absent substantive news/ban owners; accepted RouterInfo matrix remains 37/1/5 |
| I2PControl Proposal 170 containment | closed | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | no containment corrective handoff | M061 source containment and M062/M063 dependency containment remain accepted authorities |
| I2PControl Proposal 170 tunnel runtime completion | closed through M074 | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no runtime-completion handoff | M072 accepted after M073; M074 is separately closed in the security-hardening sequence |
| I2PControl Proposal 170 tunnel security hardening | closed | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | no active handoff | M079 final-head reclosure accepted |

## Canonical scope amendment for tunnel runtimes

Maintainer direction on 2026-08-14 intentionally reopens the ten Proposal 170 tunnel data planes that ADR-0001/ADR-0002 previously deferred.

New controlling decision:

- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

ADR-0001 and ADR-0002 remain historical/controlling for exhaustive registration, truthful unsupported behavior, startup/control-plane ownership separation, generic client/server runtime ownership, server secret storage, and internal-only scope. ADR-0003 supersedes only their statements that the ten remaining Proposal 170 tunnel types must remain deferred/ineligible for real backends.

The preferred implementation boundary is `emissary-cli/src/i2pcontrol/**`. M065-M079 do not authorize new `emissary-core/**` production paths. M064 is a separate narrow correction to an already accepted `events.rs` observation setter and adds no tunnel capability.

## Dependency-ready implementation plans

No implementation plan is currently registered as dependency-ready:

| Handoff | Status | Plan | Objective |
|---|---|---|---|
| — | — | — | — |

Per `plans/003-planning-process.md`, only the next dependency-ready
implementation plan is registered as ready. M079 is closed and no downstream
security plan is prewritten or newly unblocked.

## Prewritten blocked tunnel-runtime successors

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M066 — IRC client/server family | closed | `plans/implementation/i2pcontrol-proposal-170/066-irc-client-server-tunnel-family.md` | M065 closed; closure accepted |
| M067 — HTTP server | closed | `plans/implementation/i2pcontrol-proposal-170/067-http-server-tunnel.md` | M065 closed; closure accepted |
| M068 — HTTP client + CONNECT | closed | `plans/implementation/i2pcontrol-proposal-170/068-http-client-and-connect-tunnels.md` | M065 closed; closure accepted |
| M069 — SOCKS + SOCKS-IRC | closed | `plans/implementation/i2pcontrol-proposal-170/069-socks-and-socks-irc-tunnels.md` | M065 + M066 closed; closure accepted |
| M070 — HTTP bidirectional server composition | closed | `plans/implementation/i2pcontrol-proposal-170/070-http-bidirectional-server-composition.md` | M067 + M068 closed; closure accepted |
| M071 — Streamr client/server | closed | `plans/implementation/i2pcontrol-proposal-170/071-streamr-client-server-tunnels.md` | M065 closed; closure accepted |
| M072 — integrated tunnel-runtime reclosure | closed | `plans/implementation/i2pcontrol-proposal-170/072-tunnel-runtime-completion-reclosure.md` | M066-M071 closed; accepted after M073 |
| M073 — generic tunnel option truthfulness corrective | closed | `plans/implementation/i2pcontrol-proposal-170/073-generic-tunnel-option-truthfulness-corrective.md` | M072 corrective finding; closure accepted |

M071-M079 are closed. No registered tunnel-security handoff remains.

## Prewritten blocked tunnel-security successors

The post-M072 security/anonymity review established a separate bounded
corrective sequence. M073-M079 are closed; no later plan leapfrogs the accepted
reclosure.

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M074 — shared server admission and rate-limit hardening | closed | `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md` | M073 closed; closure accepted |
| M075 — generic server accepted-stream hardening | closed | `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md` | M073 + M074 closed; closure accepted |
| M076 — HTTP server anonymity and POST-throttle hardening | closed | `plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md` | M073 + M074 closed; closure accepted |
| M077 — IRC server lifetime and exhaustion hardening | closed | `plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md` | M073 + M074 + M076 closed; closure accepted |
| M078 — Streamr local-boundary hardening | closed | `plans/implementation/i2pcontrol-proposal-170/078-streamr-local-boundary-hardening.md` | M075-M077 closed; closure accepted |
| M079 — integrated tunnel-security reclosure | closed | `plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md` | M074-M078 closed; closure accepted |

Security-hardening planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

The plans pin read-only behavioral/security reference snapshots from Java I2P and I2P+ and remain independently authored. No upstream interaction is authorized.

## Tunnel-runtime security boundary

The new scope is not permission to turn specialized tunnels into raw forwarding.

Durable rules:

- `httpserver` and the inbound half of `httpbidirserver` must use application-visible accepted I2P streams so request filtering occurs before local-service forwarding;
- accepted-stream server families must use bounded global and peer-specific concurrency plus peer/aggregate connection-rate admission so one authenticated peer cannot monopolize the global pool;
- when server admission/accounting state reaches its memory bound, a new attacker-controlled identity must fail closed rather than evicting active/throttled state;
- generic control-plane `server` was migrated under M075 from blind `STREAM FORWARD` to the same peer-aware accepted-stream boundary while remaining a raw byte relay after admission;
- HTTP server completion requires bounded parsing, request-framing/request-smuggling defenses, trusted peer-derived I2P identity metadata, spoofed proxy/identity-header removal, safe Host/target handling, supported access/throttle controls, and response fingerprint/proxy-header filtering;
- HTTP response hardening must remove `Date` at minimum and the independently adopted I2P+ backend/provider/cache/trace fingerprint set where it does not alter HTTP framing; request-side reverse-proxy identity such as `X-Real-IP` must not reach the loopback application;
- HTTP POST limiter/accounting state must be bounded and churn-safe; active abusive state may not be evicted simply to admit a new identity;
- `ircclient` and `socksirc` use one common anonymity filter; DCC/unsupported CTCP remain fail-closed unless a later accepted plan explicitly implements them;
- `ircserver` filters bounded registration and derives presented client hostname/cloak from actual I2P peer identity before local IRCd forwarding; M077 adds a 10-minute activity-resetting post-registration inactivity bound rather than a fixed total session lifetime;
- `socksirc` may not have a raw/unfiltered payload path;
- HTTP client, CONNECT, and SOCKS direct-I2P routing must not use local OS DNS or direct LAN/localhost routing; clearnet requires explicitly configured I2P outproxy behavior;
- Streamr subscriber/packet/task state is hard bounded; M078 makes the local UDP producer/client boundary loopback-only and aligns the subscriber ceiling to 10;
- a real backend must apply or reject runtime-relevant Proposal 170 options before allocation; security-sensitive persist-but-ignore behavior is forbidden;
- timing hardening must remove controllable resource/fingerprint signals rather than adding random/fixed sleeps or response jitter.

## Current production baseline and containment authority

Tunnel-runtime planning production baseline:

`a1296b018ce98d26a019bd5064dff9f4b47e0ad6`.

Security-hardening planning baseline:

`04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

At the M072 reclosure head:

- production registry has real generic `client` and `server` backends;
- after M071, all ten formerly specialized Proposal 170 types are real bounded backends, including the dedicated Streamr producer/consumer runtime;
- M064 repaired the feature-disabled/no-events unused-parameter regression in `emissary-core/src/events.rs::set_ipv4_testing/set_ipv6_testing`;
- M072 is reclosed with a corrective disposition; M073 owns the generic client/server option-truthfulness repair;
- the later security review found additional server admission/fairness, generic-server accepted-stream, HTTP fingerprint/POST limiter, IRC idle-lifetime, and Streamr local-boundary hardening work; M074-M079 own those corrections without reopening unrelated Proposal 170 source scope.

Accepted containment authorities remain:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` + `m061_containment.rs` for source paths;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` + strengthened `m062_dependency_containment.rs` for direct dependency ownership and transitive local-feature activation.

M062/M063 durable dependency rule:

A direct dependency whose only direct consumer is `feature = "i2pcontrol"` code must be optional and feature-owned by `i2pcontrol`. An unrelated local feature must not activate that dependency directly or indirectly through local feature composition.

## Accepted unrelated Proposal 170 support state

The RouterInfo source matrix remains exactly:

- 43 canonical Proposal 170 RouterInfo additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable: transit 15s, news, banned peers, and both v4/v6 network-error rows.

M051 remains blocked with the accepted news/ban semantic limitation. Tunnel runtime/security hardening does not create news/ban owners or reopen transit/error source decisions.

AddressBook `SetConfig`, unrelated base-I2PControl method limitations, and any environmental qualification in the support document remain separate. Completing tunnel runtimes or the security hardening sequence must not be misrepresented as complete historical I2PControl support if those limitations remain.

## Recently closed containment/corrective milestones

| Subsystem | Handoff | Status | Implementation plan | Closure record |
|---|---|---|---|---|
| I2PControl Proposal 170 containment | M063 — M062 closure consistency and indirect feature-activation guard corrective | closed | `plans/implementation/i2pcontrol-proposal-170/063-m062-closure-and-feature-guard-corrective.md` | `plans/closure/i2pcontrol-proposal-170/063-closure.md` |
| I2PControl Proposal 170 containment | M062 — dependency-surface containment corrective | closed (closure/evidence corrected by M063) | `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md` | `plans/closure/i2pcontrol-proposal-170/062-closure.md` |
| I2PControl Proposal 170 containment | M061 — independent containment reclosure | closed | `plans/implementation/i2pcontrol-proposal-170/061-containment-reclosure.md` | `plans/closure/i2pcontrol-proposal-170/061-closure.md` |
| I2PControl Proposal 170 tunnel runtime completion | M066 — IRC client/server family | closed | `plans/implementation/i2pcontrol-proposal-170/066-irc-client-server-tunnel-family.md` | `plans/closure/i2pcontrol-proposal-170/066-closure.md` |
| I2PControl Proposal 170 tunnel runtime completion | M068 — HTTP client + CONNECT family | closed | `plans/implementation/i2pcontrol-proposal-170/068-http-client-and-connect-tunnels.md` | `plans/closure/i2pcontrol-proposal-170/068-closure.md` |
| I2PControl Proposal 170 tunnel runtime completion | M065 — runtime/option foundation | closed | `plans/implementation/i2pcontrol-proposal-170/065-i2pcontrol-tunnel-runtime-primitives.md` | `plans/closure/i2pcontrol-proposal-170/065-closure.md` |
| I2PControl Proposal 170 tunnel runtime completion | M064 — tunnel-runtime baseline corrective | closed | `plans/implementation/i2pcontrol-proposal-170/064-proposal-170-tunnel-runtime-baseline-corrective.md` | `plans/closure/i2pcontrol-proposal-170/064-closure.md` |

## Blocked source successor

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M051 — router news and banned peers | blocked with accepted semantic limitation | `plans/implementation/i2pcontrol-proposal-170/051-routerinfo-news-and-banned-peer-semantics.md` | substantive news/ban owners absent; no current owner-specific plan authorized |

M051 is independent of M064-M079.

## Verification policy for the tunnel runtime/security series

Keep verification local/package-scoped and proportional:

- focused backend/filter/unit/integration tests;
- fake/local SAM and local capture services where practical;
- paused-time deterministic limiter/expiry/idle tests where relevant;
- feature-disabled and feature-enabled `cargo check`;
- retained M061 and M062/M063 containment tests;
- bounded live child-process tests already present when relevant.

Do not add hosted CI jobs, release/publishing machinery, coverage gates, fuzz infrastructure, soak farms, broad platform matrices, public-network deanonymization experiments, or interoperability harnesses merely for this workstream.

## Registry maintenance rules

1. Only the next dependency-ready plan is normally marked/registered ready.
2. M064-M079 are closed; no dependency-ready tunnel-security handoff remains.
3. M079 independently owns and closes the final security reclosure.
4. M072 remains the integrated runtime reclosure authority for its historical head, but tunnel-runtime security may not be considered fully reclosed until M079 closes the post-M072 security findings.
5. Preserve ADR-0003 scope: implement only the pinned Proposal 170 families, not adjacent tunnel/protocol features.
6. Keep new runtime/filter/admission code under I2PControl wherever technically possible.
7. No M073-M079 plan may add a new `emissary-core/**` production path without stopping and creating separate architecture/corrective planning.
8. Preserve M061/M062/M063 containment/dependency authorities unless a separately accepted corrective explicitly supersedes them.
9. Preserve RouterInfo 37/1/5 and M051 blocker unless separate source-owner work changes them.
10. Do not silently accept/ignore runtime-relevant security options; underspecified fields must be rejected rather than guessed.
11. Do not use artificial response jitter/fixed delays as an anonymity substitute for admission fairness, bounded lifetime, and fingerprint suppression.
12. No upstream interaction is authorized. External specification/reference/source access is read-only only.
13. All repository writes remain internal to `eggstack/emissary`.
