# Proposal 170 Implementation Handoffs

Status: corrective pass required

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

Pinned external authority:

- Proposal 170 `I2PControl Expansion`, Open, created/updated `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

## Internal-only rule

These handoffs are internal to `eggstack/emissary`.

No plan authorizes:

- an upstream issue, pull request, merge request, discussion, review request, or patch submission;
- upstream review, feedback, approval, adoption, or merge solicitation;
- pushing branches, commits, tags, patches, artifacts, or releases to an upstream remote;
- upstream maintainer outreach;
- preparation of an upstream contribution package, patch series, submission checklist, or merge plan;
- connector/API writes against any upstream or third-party repository.

External specifications and reference implementations may be inspected read-only for internal correctness. All writes must remain in `eggstack/emissary` unless a future explicit maintainer directive supersedes the normative planning policy.

Violation is a stop condition and invalidates affected evidence.

## Scope rule

The Proposal 170 corrective sequence owns API correctness and the smallest truthful source/ownership adapters.

It must not implement missing tunnel data planes. The following remain separate security-focused work:

- HTTP client/server and bidirectional server tunnels;
- IRC client/server tunnels;
- SOCKS-IRC and CONNECT variants;
- Streamr client/server tunnels;
- any other listener, destination, LeaseSet, or traffic path not already implemented by Emissary.

Unsupported tunnel types remain explicit administrative definitions with deterministic inactive/not-implemented lifecycle behavior under ADR-0001.

Changes outside `emissary-cli/src/i2pcontrol/**` are permitted only for:

- one purpose-specific runtime AddressBook handle;
- composition-time startup tunnel inventory and existing-handle wiring;
- passive proxy exit observations;
- correction to the already-introduced bounded SAM observation seam;
- bounded read-only RouterInfo snapshots adjacent to existing authoritative owners.

No broad router, protocol, transport, NetDB, peer-selection, tunnel, cryptographic, streaming, resolver, frontend, CI, release, dependency, or formatting project is authorized.

## Closure invalidation

M019A is historical evidence only. Its internal-only/no-upstream attestation remains valid, but its implementation-completeness disposition is invalidated by:

- base I2PControl authentication/token incompatibility;
- JSON-RPC notification and request-ID defects;
- incorrect canonical TunnelManager `get` schema;
- non-atomic rename and secret-boundary defects;
- disconnected AddressBook shadow state;
- missing startup tunnel inventory and stale proxy lifecycle state;
- sticky SAM observation overflow;
- unresolved RouterInfo source/claim contradictions;
- fixtures that validated repository output rather than the full pinned contract.

See `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`.

## Corrective handoffs

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M020 — Base I2PControl and JSON-RPC interoperability | ready | `020-base-i2pcontrol-and-jsonrpc-interoperability.md` | none |
| M021 — TunnelManager exact wire, atomic persistence, and secret boundary | blocked | `021-tunnelmanager-wire-atomicity-and-secrets.md` | M020 |
| M022 — AddressBook runtime bridge and source reconciliation | blocked | `022-addressbook-runtime-bridge.md` | M020, M021 |
| M023 — Startup tunnel inventory and ClientServicesInfo truthfulness | blocked | `023-startup-tunnel-inventory-and-client-services.md` | M021 |
| M024 — Recoverable bounded SAM observation | blocked | `024-recoverable-bounded-sam-observation.md` | M023 |
| M025 — RouterInfo contract and source reconciliation | blocked | `025-routerinfo-contract-and-source-reconciliation.md` | M020, M022, M023, M024 |
| M026 — Bounded router inspection sources | blocked | `026-bounded-router-inspection-sources.md` | M025 |
| M027 — Exact conformance and independent reclosure | blocked | `027-proposal-170-conformance-and-reclosure.md` | M020–M026 |

Earlier plans remain historical as recorded in the subsystem roadmap. M019 remains superseded and non-executable.

## Execution order

```text
M020 base I2PControl and JSON-RPC interoperability
    |
    v
M021 TunnelManager exact contract and persistence safety
    |
    +--------------------------+
    |                          |
    v                          v
M022 AddressBook runtime bridge   M023 startup inventory/client services
    |                          |
    +-------------+------------+
                  v
M024 recoverable SAM observation
                  |
                  v
M025 RouterInfo contract/source matrix
                  |
                  v
M026 feasible bounded owner snapshots
                  |
                  v
M027 exact conformance and independent reclosure
```

Only M020 is dependency-ready at registration time. Successor plans are complete handoffs but must not execute before their hard dependencies close.

## Handoff discipline

Each implementation plan must produce an implementation disposition containing:

- implementation commits;
- exact changed files;
- requirement-to-evidence matrix for its bounded objective;
- focused and broad command outcomes;
- failure/restart/contention evidence;
- compatibility and migration effects;
- security review;
- unresolved findings with severity;
- scope/no-upstream attestation;
- frozen implementation/test head.

A successful implementation commit or broad test count is not closure by itself.

Material deviations require a new plan or explicit corrective disposition. Agents must not weaken requirements to match current code.

## Verification rule

Default CLI package scope:

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Run core package scope only for plans that touch the permitted runtime/SAM/inspection seams:

```bash
cargo check -p emissary-core
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
```

Each plan lists focused filters to run first. Use touched-file formatting when unrelated workspace formatting differences remain.

Remote CI, upstream CI, release checks, platform matrices, coverage gates, fuzz campaigns, network farms, long soak tests, submission checks, and generated evidence bundles are not required.

## Final-status rule

Only M027 may restore a final subsystem disposition.

Possible outcomes:

- `closed internally against pinned revision` when exact wire behavior and every claimed source/runtime dimension have evidence;
- `partial Proposal 170 support` when one or more pinned sources remain truthfully unavailable after bounded-source work;
- `corrective pass required` for unresolved high/medium defects;
- `blocked` when necessary evidence cannot be obtained.

Missing tunnel data planes may remain explicit runtime-unsupported stubs without violating the API scope, but documentation must never count those stubs as real runtime implementation.

No final status implies upstream review, acceptance, certification, adoption, or merge.