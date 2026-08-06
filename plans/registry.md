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
| I2PControl Proposal 170 | partial Proposal 170 support | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | None; M040–M044 closed | M039 remains historical-invalidated; no successor is registered |

## Dependency-ready implementation plans

| Subsystem | Handoff | Status | Implementation plan | Dependencies |
|---|---|---|---|---|
| — | None | — | — | No implementation plan is dependency-ready |

## Registered successor handoffs

| Subsystem | Handoff | Status | Plan | Hard dependency |
|---|---|---|---|---|
| I2PControl Proposal 170 | M040 — Startup server cancellation-owner correction | closed | `plans/implementation/i2pcontrol-proposal-170/040-startup-server-cancellation-correction.md` | M039 invalidation accepted |
| I2PControl Proposal 170 | M041 — Authentication throttle source/accounting correction | closed | `plans/implementation/i2pcontrol-proposal-170/041-auth-throttle-source-accounting.md` | M040 closed |
| I2PControl Proposal 170 | M042 — AddressBook subscription commit boundary | closed | `plans/implementation/i2pcontrol-proposal-170/042-addressbook-subscription-commit-boundary.md` | M041 closed |
| I2PControl Proposal 170 | M043 — Corrective runtime regression validation | closed | `plans/implementation/i2pcontrol-proposal-170/043-corrective-runtime-regression-validation.md` | M040–M042 closed |
| I2PControl Proposal 170 | M044 — Corrective final-head reclosure | closed | `plans/implementation/i2pcontrol-proposal-170/044-corrective-final-head-reclosure.md` | M043 closed |

## Active closure work

| Subsystem | Handoff | Status | Evidence | Closure record |
|---|---|---|---|---|
| — | None | — | No active closure work; M044 accepted at reviewed head `342420e` | `plans/closure/i2pcontrol-proposal-170/044-closure.md` |

## Current corrective findings

| Finding | Severity | Owner | State |
|---|---|---|---|
| Startup `ServerTunnelManager` drops the only watch sender and may self-cancel before SAM session/forward setup | high correctness/regression | M040 | closed |
| Failed-auth throttle uses full `SocketAddr`, so ephemeral-port churn resets state | medium security | M041 | closed |
| Failed-auth delay is read before failure reservation, so concurrent attempts undercount | medium security | M041 | closed |
| `SetSubscriptions` may return failure after durable/active mutation commit | medium operation truthfulness | M042 | closed |
| Accepted evidence omitted exact startup-server, port-churn/concurrency, and post-commit worker-failure paths | high evidence gate | M043 | closed |
| Independent corrected final-head review | high evidence gate | M044 | closed |

## Closure invalidation

| Record | Status | Document | Consequence |
|---|---|---|---|
| M039 operational final-head closure | historical-invalidated | `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md` | M044 restored the final `partial Proposal 170 support` disposition; M039 remains non-controlling historical evidence |

The invalidation retains unaffected M020–M039 evidence. It specifically makes
non-controlling:

- M032/M039 startup-server behavior-preservation claims;
- M036/M039 failed-auth throttle effectiveness claims;
- M034/M039 single-boundary subscription-result claims.

## Corrective scope guard

### M040

Authorized production path:

- `emissary-cli/src/tunnel/server.rs`

Purpose: retain the startup runtime watch sender and add direct
`ServerTunnelManager` regression evidence.

### M041

Authorized production paths:

- `emissary-cli/src/i2pcontrol/auth.rs`
- `emissary-cli/src/i2pcontrol/server.rs`

Purpose: normalize throttle identity to source IP and reserve failure count
atomically before delay.

### M042

Authorized production paths:

- `emissary-cli/src/address_book.rs`
- narrowly related `emissary-cli/src/i2pcontrol/address_book_runtime.rs`
- `emissary-cli/src/i2pcontrol/address_book.rs` only if exact response
  translation is directly affected

Purpose: define one durable subscription mutation linearization point and make
refresh scheduling post-commit follow-up work.

### M043/M044

No production changes. A material defect requires a new corrective plan.

## Prohibited corrective scope

- new HTTP, HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr,
  or other tunnel data planes;
- startup task adoption/control;
- new RouterInfo sources or fabricated values;
- router, transport, streaming, LeaseSet, cryptographic, routing, or
  tunnel-building changes;
- new `emissary-core/**` behavior;
- frontend work;
- broad crate/service refactors;
- persistent accounts, proxy-header trust, distributed bans, or firewall
  integration;
- AddressBook scheduler/event bus/second authority/arbitrary paths;
- `.github/workflows/**`, remote CI, release/publishing, coverage, fuzz, soak,
  platform matrices, or generated evidence bundles;
- upstream issues, pull requests, reviews, submissions, adoption, merge,
  maintainer contact, or contribution preparation.

## Retained closed evidence

| Milestone | Retained scope | Corrective qualification |
|---|---|---|
| M020–M030 | wire/auth/base behavior, persistence, RouterInfo matrix, AddressBook owner/isolation | retained unless a new direct defect is shown |
| M031 | generic client backend and per-name supervisor | retained |
| M032 | generic server backend and fixed secret identity | startup behavior-preservation claim invalidated |
| M033 | lifecycle reconciliation and StartOnLoad | retained |
| M034 | live subscription owner and unsupported SetConfig | post-commit result-boundary claim invalidated |
| M035 | compatibility inventory and overlap behavior | retained |
| M036 | constant-time comparison and publication hardening | throttle effectiveness claim invalidated |
| M037 | containment reduction and passive SAM hook | retained subject to M040 startup adapter correction |
| M038 | bounded live child-process evidence | exact-path coverage gap recorded |
| M039 | final review record | final disposition invalidated |

RouterInfo source classification remains:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

Tunnel backend classification remains:

- real: generic `client`, generic `server`;
- explicit unsupported: the other ten Proposal 170 tunnel families.

## Pinned authority

Current work is pinned to:

- proposal: `I2PControl Expansion`, Proposal 170;
- status: `Open`;
- created: `2026-05-20`;
- last updated: `2026-05-20`;
- canonical page: `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`;
- existing I2PControl authentication/error documentation: `https://i2p.net/en/docs/api/i2pcontrol`.

A changed proposal revision blocks affected implementation/closure and requires a
contract-rebase plan.

## Registry maintenance rules

1. M040–M044 are closed; no implementation plan is currently dependency-ready.
2. Preserve unaffected M020–M039 evidence unless a new direct defect is demonstrated.
3. Keep startup and control-plane runtime ownership separate.
4. Keep production changes outside `i2pcontrol/**` limited to the M040 server adapter correction.
5. Unsupported tunnel families and unavailable RouterInfo sources remain explicit.
6. Verification remains local and package-scoped; no CI/release expansion.
7. M039 remains historical-invalidated after M044; do not delete or rewrite its records.
8. No upstream interaction is authorized.
