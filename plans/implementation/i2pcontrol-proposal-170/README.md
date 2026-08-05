# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; M039 closed

This directory contains bounded internal implementation and closure handoffs for
the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/030-closure.md`

Pinned external authority:

- Proposal 170 `I2PControl Expansion`, Open, created/updated `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

## Internal-only rule

These handoffs are internal to `eggstack/emissary`.

No plan authorizes:

- an upstream issue, pull request, merge request, discussion, review request, or
  patch submission;
- upstream review, feedback, approval, adoption, or merge solicitation;
- pushing branches, commits, tags, patches, artifacts, or releases to an upstream
  remote;
- upstream maintainer outreach;
- preparation of an upstream contribution package, patch series, submission
  checklist, or merge plan;
- connector/API writes against any upstream or third-party repository.

External specifications and reference implementations may be inspected read-only
for internal correctness. All writes remain in `eggstack/emissary` unless a
future explicit maintainer directive supersedes the normative planning policy.

Violation is a stop condition and invalidates affected evidence.

## Current handoff

No active handoff remains. M039 is formally closed against M038.

M031 established the control-plane runtime supervisor and replaced only the
generic `client` unsupported backend. M032 added the generic `server` backend
and persistent destination identity. M033 closed lifecycle reconciliation and
StartOnLoad. M035 closed the base compatibility boundary, M036 hardened
authentication/publication, and M037 reduced containment coupling. M038 closed
the live-runtime validation gate. These
milestones may not modify `emissary-core`, adopt
startup-managed tasks, or implement another tunnel family.

## Registered successor sequence

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M032 — Generic server backend and destination identity | closed | `032-server-tunnel-runtime-backend.md` | M031 closed |
| M033 — Lifecycle reconciliation and StartOnLoad | closed | `033-tunnel-lifecycle-reconciliation.md` | M031 and M032 closed |
| M034 — AddressBook setter truthfulness | closed | `034-addressbook-setter-truthfulness.md` | M033 closed |
| M035 — Base compatibility and selector overlap | closed | `035-base-compatibility-and-selector-overlap.md` | M034 closed |
| M036 — Authentication and publication hardening | closed | `036-auth-and-publication-hardening.md` | M035 closed |
| M037 — Containment boundary reduction | closed | `037-containment-boundary-reduction.md` | M036 closed |
| M038 — Live-runtime interoperability | closed | `038-live-runtime-interoperability.md` | M031–M037 closed |
| M039 — Operational final-head reclosure | closed | `039-operational-reclosure.md` | M038 closed |

Future plans are written for continuity but are not executable until their named
hard dependencies close and the registry advances them.

## Retained baseline

M030 remains the controlling pre-workstream closure. Retained evidence includes:

| Milestone | Retained result |
|---|---|
| M020 | base authentication/token/error and JSON-RPC correctness |
| M021 | exact TunnelManager wire, validation, atomic definition persistence, secret boundary |
| M022 | enabled AddressBook runtime authority |
| M023 | startup inventory and ClientServicesInfo lifecycle truthfulness |
| M024 | recoverable bounded SAM observation |
| M025 | exact 43-selector RouterInfo contract/source matrix |
| M026 | no feasible additional bounded RouterInfo sources under prior scope |
| M027 | literal contract fixtures; historical final disposition invalidated |
| M028 | compile-time/runtime AddressBook feature isolation |
| M029 | retained non-AddressBook review evidence; final disposition invalidated |
| M030 | full-destination AddressBook owner coherence and partial-support closure |

Current RouterInfo source disposition remains:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

## Runtime tunnel boundary

Under ADR-0002:

- generic `client` and `server` are the only types eligible for real backends in
  this roadmap because Emissary already has those data planes;
- startup-managed client/server tunnels remain externally owned and read-only;
- control-plane-created definitions are supervised separately by I2PControl;
- HTTP, HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, and
  all other missing types remain explicit unsupported backends;
- existing HTTP/SOCKS startup services are not automatically treated as
  Proposal 170 I2PTunnel backends;
- no generic tunnel backend plan may modify `emissary-core/**`.

## Production boundary

Primary production work remains inside:

- `emissary-cli/src/i2pcontrol/**`.

Permitted narrow runtime adapters are milestone-specific:

- M031: `emissary-cli/src/tunnel/client.rs` and one small composition seam;
- M032: `emissary-cli/src/tunnel/server.rs` and one small composition seam;
- M034: `emissary-cli/src/address_book.rs` for one typed active-subscription
  command/overlay seam;
- M037: behavior-preserving reduction of existing AddressBook/SAM coupling,
  including a minimal passive core hook only if it removes broader current core
  machinery.

Every production file outside `i2pcontrol/**` must be named in the implementation
disposition with a direct requirement and before/after justification.

Prohibited throughout:

- new missing tunnel data planes;
- startup task adoption/control;
- RouterInfo samplers/polling/fabricated values;
- router, transport, streaming protocol, LeaseSet, crypto, routing, or tunnel
  algorithm changes;
- frontend work;
- repository-wide crate/service refactors;
- arbitrary request-selected filesystem paths;
- `.github/workflows/**`, remote CI, release/publishing, coverage, fuzz, soak,
  platform matrices, or generated evidence bundles;
- upstream activity.

## Handoff discipline

Each implementation milestone must:

1. inspect the accepted dependency head before editing;
2. add a failing regression or literal evidence for the defect/capability;
3. preserve unrelated retained evidence;
4. stay within its production file budget;
5. run focused tests before the bounded broad matrix;
6. create an implementation disposition under
   `plans/closure/i2pcontrol-proposal-170/`;
7. freeze the implementation/test head;
8. report every unresolved finding with severity;
9. leave final subsystem closure to the independent M039 closure record;
10. attest that no upstream interaction occurred.

A successful commit, compilation result, or broad test count is not closure by
itself.

## Milestone-specific stop rules

### M031/M032/M033

Stop rather than:

- modify core;
- adopt startup-managed tasks;
- duplicate existing generic data planes;
- implement HTTP/SOCKS/IRC/CONNECT/Streamr/bidirectional behavior;
- add public protocol fields or statuses.

### M034

Stop rather than report inert setter success, add arbitrary path control, create
a second AddressBook authority, or introduce a general scheduler/config bus.

### M035

Stop and request a separate plan if compatibility requires implementing a
substantial missing base method or router-control owner.

### M036

Stop rather than add a general authentication/account system or router-wide
storage framework.

### M037

Stop the affected extraction if behavior-preserving containment would require an
unbounded event channel, polling, a second authority, or broader core behavior.

### M038/M039

Validation and closure do not patch production defects. A material defect
requires a new corrective plan.

## Verification rule

The normal bounded matrix is:

```bash
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Each plan adds focused commands for its changed paths. Original CLI module
changes require focused no-feature evidence. M037 alone may require focused core
SAM checks because its purpose is to reduce existing core coupling.

Use targeted formatting and `git diff --check`.

Remote CI, release checks, coverage, fuzzing, soak testing, network farms,
platform matrices, submission checks, and generated evidence bundles are not
required.

## Final-status rule

M039 selected:

- `partial Proposal 170 support`: every implemented/claimed wire, source,
  runtime, persistence, security, and containment dimension is exact and
  evidenced, while RouterInfo sources and tunnel families remain unavailable.

The accepted closure record is
`plans/closure/i2pcontrol-proposal-170/039-closure.md`.

`closed internally against pinned revision` is not expected under this roadmap
because 26 RouterInfo sources and ten tunnel families remain unavailable or
unsupported unless separately authorized future work changes that fact.

No final status implies upstream review, acceptance, certification, adoption, or
merge.
