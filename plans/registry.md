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
| I2PControl Proposal 170 tunnel runtime completion | active; M064-M068 closed; M069 ready | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | M069 — SOCKS + SOCKS-IRC | M070/M071 are dependency-ready but not the next registered handoff; M072 remains blocked on later family closure |

## Canonical scope amendment for tunnel runtimes

Maintainer direction on 2026-08-14 intentionally reopens the ten Proposal 170 tunnel data planes that ADR-0001/ADR-0002 previously deferred.

New controlling decision:

- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

ADR-0001 and ADR-0002 remain historical/controlling for exhaustive registration, truthful unsupported behavior, startup/control-plane ownership separation, generic client/server runtime ownership, server secret storage, and internal-only scope. ADR-0003 supersedes only their statements that the ten remaining Proposal 170 tunnel types must remain deferred/ineligible for real backends.

The preferred implementation boundary is `emissary-cli/src/i2pcontrol/**`. M065-M072 do not authorize new `emissary-core/**` production paths. M064 is a separate narrow correction to an already accepted `events.rs` observation setter and adds no tunnel capability.

## Dependency-ready implementation plans

Exactly one plan is currently registered as dependency-ready:

| Handoff | Status | Plan | Objective |
|---|---|---|---|
| M069 — SOCKS + SOCKS-IRC | ready | `plans/implementation/i2pcontrol-proposal-170/069-socks-and-socks-irc-tunnels.md` | implement bounded SOCKS TCP CONNECT and compose the accepted IRC filter |

Per `plans/003-planning-process.md`, only the next dependency-ready implementation plan is registered as ready. Future handoffs are prewritten but remain blocked until their hard dependencies close.

## Prewritten blocked tunnel-runtime successors

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M066 — IRC client/server family | closed | `plans/implementation/i2pcontrol-proposal-170/066-irc-client-server-tunnel-family.md` | M065 closed; closure accepted |
| M067 — HTTP server | closed | `plans/implementation/i2pcontrol-proposal-170/067-http-server-tunnel.md` | M065 closed; closure accepted |
| M068 — HTTP client + CONNECT | closed | `plans/implementation/i2pcontrol-proposal-170/068-http-client-and-connect-tunnels.md` | M065 closed; closure accepted |
| M069 — SOCKS + SOCKS-IRC | ready | `plans/implementation/i2pcontrol-proposal-170/069-socks-and-socks-irc-tunnels.md` | M065 + M066 closed |
| M070 — HTTP bidirectional server composition | blocked — dependency-ready but not the next registered handoff | `plans/implementation/i2pcontrol-proposal-170/070-http-bidirectional-server-composition.md` | M067 + M068 closed |
| M071 — Streamr client/server | blocked — dependency-ready but not the next registered handoff | `plans/implementation/i2pcontrol-proposal-170/071-streamr-client-server-tunnels.md` | M065 closed |
| M072 — integrated tunnel-runtime reclosure | blocked | `plans/implementation/i2pcontrol-proposal-170/072-tunnel-runtime-completion-reclosure.md` | M066-M071 closed |

After M068 closes, M069, M070, and M071 are dependency-ready. Project convention registers M069 as the next handoff. M070 waits for both accepted HTTP halves. M072 waits for every runtime-family milestone.

## Tunnel-runtime security boundary

The new scope is not permission to turn specialized tunnels into raw forwarding.

Durable rules:

- `httpserver` and the inbound half of `httpbidirserver` must use application-visible accepted I2P streams so request filtering occurs before local-service forwarding;
- HTTP server completion requires bounded parsing, request-framing/request-smuggling defenses, trusted peer-derived I2P identity metadata, spoofed proxy/identity-header removal, safe Host/target handling, supported access/throttle controls, and response fingerprint/proxy-header filtering;
- `ircclient` and `socksirc` use one common anonymity filter; DCC/unsupported CTCP remain fail-closed unless a later accepted plan explicitly implements them;
- `ircserver` filters bounded registration and derives presented client hostname/cloak from actual I2P peer identity before local IRCd forwarding;
- `socksirc` may not have a raw/unfiltered payload path;
- HTTP client, CONNECT, and SOCKS direct-I2P routing must not use local OS DNS or direct LAN/localhost routing; clearnet requires explicitly configured I2P outproxy behavior;
- Streamr subscriber/packet/task state is hard bounded;
- a real backend must apply or reject runtime-relevant Proposal 170 options before allocation; security-sensitive persist-but-ignore behavior is forbidden.

## Current production baseline and containment authority

Tunnel-runtime planning production baseline:

`a1296b018ce98d26a019bd5064dff9f4b47e0ad6`.

At that baseline:

- production registry has real generic `client` and `server` backends;
- the other eight specialized Proposal 170 types are explicit unsupported backends;
- M064 repaired the feature-disabled/no-events unused-parameter regression in `emissary-core/src/events.rs::set_ipv4_testing/set_ipv6_testing`;
- no specialized real backend is claimed yet.

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

M051 remains blocked with the accepted news/ban semantic limitation. Tunnel runtime completion does not create news/ban owners or reopen transit/error source decisions.

AddressBook `SetConfig`, unrelated base-I2PControl method limitations, and any environmental qualification in the support document remain separate. Completing tunnel runtimes must not be misrepresented as complete historical I2PControl support if those limitations remain.

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

M051 is independent of M064-M072.

## Verification policy for the new runtime series

Keep verification local/package-scoped and proportional:

- focused backend/filter/unit/integration tests;
- fake/local SAM and local capture services where practical;
- feature-disabled and feature-enabled `cargo check`;
- retained M061 and M062/M063 containment tests;
- bounded live child-process tests already present when relevant.

Do not add hosted CI jobs, release/publishing machinery, coverage gates, fuzz infrastructure, soak farms, broad platform matrices, or public-network interoperability harnesses merely for this workstream.

## Registry maintenance rules

1. Only the next dependency-ready plan is normally marked/registered ready.
2. M064-M068 are closed; M069 is the current dependency-ready handoff.
3. After M068, M069/M070/M071 are dependency-ready independently; registry should reflect whichever handoff(s) are actually assigned/active without rewriting future plan requirements.
4. Preserve ADR-0003 scope: implement only the ten pinned Proposal 170 families, not adjacent tunnel/protocol features.
5. Keep new specialized runtime/filter code under I2PControl wherever technically possible.
6. No M065-M072 plan may add a new `emissary-core/**` production path without stopping and creating separate architecture/corrective planning.
7. Preserve M061/M062/M063 containment/dependency authorities unless a separately accepted corrective explicitly supersedes them.
8. Preserve RouterInfo 37/1/5 and M051 blocker unless separate source-owner work changes them.
9. Do not silently accept/ignore runtime-relevant security options.
10. Keep unsupported backends resource-free until the owning real-backend milestone closes.
11. No upstream interaction is authorized. External specification/reference/source access is read-only only.
12. All repository writes remain internal to `eggstack/emissary`.
