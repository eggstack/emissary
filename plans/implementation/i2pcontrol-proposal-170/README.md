# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; tunnel-runtime completion phase active; M064/M065 closed; M066 next

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative planning references:

- `plans/000-long-term-specification.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`

Pinned Proposal 170 revision: `2026-05-20`.

Planning production baseline for M064-M072: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, reference implementations, and upstream source are read-only evidence. No plan authorizes upstream submissions, review requests, maintainer contact, contribution preparation, or repository writes outside the fork.

## Canonical scope amendment

The original Proposal 170 phase intentionally left missing tunnel data planes behind explicit unsupported backends. Maintainer direction on 2026-08-14 changes that long-term scope.

ADR-0003 now authorizes bounded implementation of the ten remaining Proposal 170 tunnel types while preserving the established containment and startup/control-plane ownership rules. ADR-0001 and ADR-0002 remain historical authority except for their statements that these ten data planes must remain deferred/ineligible.

The implementation target is:

- keep specialized runtime/filter policy in `emissary-cli/src/i2pcontrol/**`;
- avoid new `emissary-core/**` production changes for missing-tunnel implementation;
- treat HTTP/IRC filtering as required correctness/security behavior;
- reject runtime-relevant options that are recognized but not implemented instead of silently ignoring them;
- retain explicit unsupported backends until each real family independently closes.

## Current handoff

M065 is closed. M066 is the next registered dependency-ready implementation plan:

- `065-i2pcontrol-tunnel-runtime-primitives.md` — **closed**; closure:
  `plans/closure/i2pcontrol-proposal-170/065-closure.md`.
- `066-irc-client-server-tunnel-family.md` — **ready**.

M064 repairs the existing feature-disabled/no-events unused-parameter regression in `emissary-core/src/events.rs` and establishes a clean baseline before new tunnel runtime work. It adds no capability.

M064 closure: `plans/closure/i2pcontrol-proposal-170/064-closure.md`.

Per `plans/003-planning-process.md`, future plans are prewritten for continuity but are not registered ready until their hard dependencies close.

## Tunnel runtime completion sequence

| Handoff | Status | Scope | Dependencies |
|---|---|---|---|
| M064 | closed | narrow current-head no-events/core-feature corrective | M063 closed |
| M065 | closed | I2PControl-owned client/accepted-server runtime primitives + option-capability validation | M064 closed |
| M066 | ready | common IRC filter + real `ircclient` and `ircserver` | M065 |
| M067 | blocked — dependency-ready but not next registered handoff | secure filtered `httpserver` | M065 |
| M068 | blocked — dependency-ready but not next registered handoff | real `httpclient` + strict `connectclient` | M065 |
| M069 | blocked | SOCKS4a/5 CONNECT + `socksirc` composed with M066 filter | M065, M066 |
| M070 | blocked | `httpbidirserver` composition of M067/M068 | M067, M068 |
| M071 | blocked — dependency-ready but not next registered handoff | bounded Streamr client/server datagram family | M065 |
| M072 | blocked | integrated twelve-type runtime/security/containment reclosure | M066-M071 |

After M065 closes, M066, M067, M068, and M071 are dependency-ready successors. Project convention registers only M066 as the next handoff; M069 waits for the common IRC filter, M070 waits for both HTTP halves, and M072 waits for all families.

## Security-critical family rules

### HTTP server

`httpserver` and the inbound half of `httpbidirserver` must use application-visible accepted I2P streams and sanitize the request before connecting/writing to the local HTTP service. The backend must cover bounded parsing, request-framing ambiguity, spoofed I2P/proxy identity headers, Host policy, configured access/throttle controls, response fingerprint/proxy-header filtering, target confinement, cancellation, and resource bounds.

Blind SAM forwarding is not an acceptable implementation for these types.

### IRC

`ircclient` and `socksirc` use one common line-oriented anonymity filter. Initial completion blocks DCC and unsupported CTCP rather than creating auxiliary DCC tunnels. `ircserver` separately sanitizes the bounded registration sequence and derives presented peer identity from the actual accepted I2P peer before the local IRCd receives registration.

WEBIRC is not required by the initial family plan and must be rejected if requested before implemented.

### Proxy safety

HTTP client, CONNECT, and SOCKS direct-I2P modes must not use local OS DNS. Clearnet access requires explicitly configured I2P outproxy behavior. Localhost/LAN/private/link-local direct targets fail closed. Non-loopback proxy exposure follows explicit authentication/safety policy and must not become an accidental open proxy.

### Streamr

Streamr remains a small datagram producer/consumer subsystem with hard subscriber, expiry, packet, and task bounds. It must not drive a generalized UDP transport framework.

## Current containment authority

M061 remains the accepted source-path containment authority. New production code for M065-M072 should stay below the existing `emissary-cli/src/i2pcontrol/**` policy root, so the preferred tunnel-completion path does not expand the non-I2PControl source boundary.

M062 plus the strengthened M063 test remain the dependency-policy authority:

- an I2PControl-only direct dependency must be optional and activated through `i2pcontrol`;
- unrelated local features must not activate it directly or indirectly;
- default/feature-disabled builds must remain free of the specialized runtime behavior.

M064 is the only planned `emissary-core/**` edit in the new runtime series, and it is a semantically neutral repair to an already accepted observation setter. M065-M072 do not authorize new core production paths.

## Accepted unrelated Proposal 170 state

Tunnel runtime completion does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook `SetConfig`/unrelated base-I2PControl limitations remain separate and must continue to be documented truthfully.

## Verification discipline

Use focused local/package-scoped Rust checks and deterministic fake-SAM/local service fixtures. Existing M061/M062 containment tests remain required where relevant.

Do not add hosted CI jobs, release machinery, broad platform matrices, fuzz infrastructure, coverage gates, long-running soak systems, or public-network certification harnesses merely to implement these tunnel types.

Security parser tests may be extensive, but should remain ordinary deterministic local Rust tests unless a concrete finding proves another tool is necessary.

## Final status rule

The repository may continue to call the Proposal 170 contract partial/runtime-partial until the relevant closure evidence exists. Replacing an unsupported backend in code is not closure by itself.

M072 may describe tunnel runtime completion only if all twelve production types are real, HTTP/IRC filtering is non-bypassable, runtime-relevant options are applied or truthfully rejected, lifecycle/persistence/containment/default-build evidence is green, and no high/medium correctness or security finding remains.

No upstream review or acceptance is implied or authorized.
