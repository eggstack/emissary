# Proposal 170 Long-Term Roadmap

Status: active; tunnel-runtime completion phase authorized 2026-08-14

This roadmap orders the work required to implement I2P Proposal 170 without expanding the public protocol, altering router behavior, or coupling the work to frontend state.

The original phase deliberately stopped at contract completeness with explicit unsupported tunnel backends. Maintainer direction on 2026-08-14 intentionally adds a second phase: implement the ten remaining Proposal 170 tunnel families through bounded application-layer backends while preserving the established I2PControl containment boundary.

Normative references:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`

Detailed subsystem roadmaps:

- source/truthfulness: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`;
- containment: `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`;
- tunnel runtime completion: `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

## Phase 1 dependency graph — contract and truthful partial runtime

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

M001-M007 are historical canonical phases. Subsequent corrective/source/containment milestones refined them and established the current partial Proposal 170 state.

## Milestone M001 — Contract matrix and I2PControl foundation

Primary class: invariant / infrastructure

Established:

- an exact Proposal 170 conformance matrix;
- base I2PControl authentication and version behavior required by the extension;
- a frontend-independent JSON-RPC listener and dispatcher;
- exact protocol/error DTOs;
- bounded request handling and security defaults;
- a typed method registry and control-plane interface boundary;
- contract fixtures for later milestones.

## Milestone M002 — Control-plane domain and persistence

Primary class: invariant / infrastructure

Established:

- canonical tunnel definitions covering every Proposal 170 option;
- exhaustive tunnel type and action enums;
- backend registry contracts;
- administrative address-book models;
- versioned, atomic, restart-safe persistence;
- explicit ownership and internal state models;
- fake control-plane implementations for method tests.

## Milestone M003 — AddressBook

Primary class: capability

Implemented the Proposal 170 AddressBook contract to the repository's current truthful support level without changing runtime resolver precedence. Any remaining `SetConfig` limitation is separate from the tunnel-runtime completion phase.

## Milestone M004 — TunnelManager and explicit stubs

Primary class: capability / infrastructure

Established:

- exact parsing for every declared tunnel type and option;
- create, edit, get, delete, start, stop, restart, and permitted `All` behavior;
- exhaustive backend registration;
- persistent definitions and deterministic status/error mapping;
- explicit unsupported backends for missing data planes;
- later real generic `client` and `server` backends through ADR-0002 and follow-up milestones.

The unsupported-backend design remains the safe intermediate state for a type until its new runtime milestone independently closes.

## Milestone M005 — RouterInfo inspection

Primary class: capability / infrastructure

The accepted current matrix is 43 canonical Proposal 170 additions / 37 available / 1 protocol-permitted neutral / 5 unavailable. Tunnel runtime work MUST NOT reopen those source-owner decisions.

## Milestone M006 — ClientServicesInfo

Primary class: capability

Implemented exact service selectors using real listener/session state where available and truthful inactive/unavailable state otherwise. As new control-plane tunnel families become real, ClientServicesInfo integration may be updated only where the pinned selector semantics require it.

## Milestone M007 — Conformance, hardening, and strict closure

Primary class: invariant / polish

Established protocol/security/persistence/containment closure for the contract-complete partial-runtime state. It is not historical authority that permanently forbids later real tunnel backends; ADR-0003 explicitly reopens that bounded runtime scope.

## Phase 2 — Proposal 170 tunnel runtime completion

Primary class: capability / security / containment

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Implementation handoffs are numbered M064-M072 to continue the repository's active planning sequence.

```text
M064 current-head baseline/core-feature corrective
    |
    v
M065 I2PControl-owned runtime/filter foundation
    |
    +------------------+------------------+------------------+
    |                  |                  |                  |
    v                  v                  v                  v
M066 IRC family    M067 HTTP server   M068 HTTP client/  M071 Streamr family
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
       +---------------+-------------------+
       |               |                   |
       +------- M066-M071 all closed ------+
                       |
                       v
                 M072 integrated reclosure
```

M067, M068, M069, and M071 may proceed in parallel after M066 closes. M069
uses the accepted common IRC filter from M066 for `socksirc`. M070 requires
closed HTTP server and HTTP client implementations and must be composition-only.
M072 depends on every runtime-family milestone.

### Phase-2 invariants

- No new Proposal 170 method, field, action, tunnel type, or wire status is introduced.
- New specialized runtime/filter code remains under `emissary-cli/src/i2pcontrol/**` wherever technically possible.
- No new `emissary-core/**` production change is authorized for missing-tunnel implementation.
- Existing startup services/tunnels are not adopted into control-plane ownership.
- `httpserver` and `ircserver` use application-visible accepted I2P streams so filtering occurs before local-service forwarding.
- HTTP/IRC security filters are minimum functionality, not post-completion polish.
- Real backends reject runtime-relevant options they do not implement; security-sensitive persist-but-ignore behavior is forbidden.
- Clearnet proxying requires an explicitly configured I2P outproxy; no arbitrary local DNS/LAN access is introduced.
- DCC may remain explicitly unsupported in the initial IRC implementation rather than creating auxiliary tunnel machinery.
- HTTP bidirectional support reuses closed HTTP server/client implementations rather than forking a third HTTP stack.
- Streamr remains a small bounded datagram implementation rather than driving a generalized transport abstraction.
- Verification remains local/package-focused; no new CI/release/fuzz/coverage apparatus is required by default.
- No upstream interaction, submission, review, adoption, or contribution preparation is authorized.

## Deferred work outside this roadmap

The following remain separate unless a later maintainer directive changes them:

- runtime lifecycle migration/adoption of existing startup-managed tunnels and proxies;
- tunnel types not declared by pinned Proposal 170;
- runtime resolver-precedence changes for the four Proposal 170 address books;
- the blocked RouterInfo news/banned-peer sources and other accepted unavailable source rows;
- unrelated base I2PControl methods not required by the pinned Proposal 170 scope;
- frontend management of I2PControl resources;
- new I2PControl methods or fields;
- cross-router interoperability certification beyond the focused behavior needed to validate Proposal 170 tunnel families;
- upstream contribution/review/merge activity.

## Roadmap completion rule

The contract-complete partial-runtime state remains a valid intermediate state. The newly authorized tunnel-runtime phase closes only after M072 demonstrates that all newly implemented families are operational and secure within their declared capability sets, unsupported option behavior is truthful, application filters are non-bypassable, default/feature-disabled Emissary behavior remains unaffected, and the final support documentation accurately distinguishes any remaining limitations.
