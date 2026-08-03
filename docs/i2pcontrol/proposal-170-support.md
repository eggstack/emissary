# Proposal 170 Support Status

Status: closed against the pinned 2026-05-20 Proposal 170 revision

Proposal 170 remains Open. This status is pinned to the `2026-05-20` revision.

The prior M019A `closed internally against pinned revision` disposition is invalidated by:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

Current corrective roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Final closure:

- M027, `plans/closure/i2pcontrol-proposal-170/027-closure.md`

## Status model

Support is reported separately as:

| Dimension | Meaning |
|---|---|
| Wire | exact public request/response names, casing, presence semantics, and JSON types |
| Source | truthful current Emissary source exists |
| Runtime | a real backend performs the requested operation |
| Persistence | mutation is durable and failure-atomic |
| Evidence | literal external-contract, failure, restart, and production-composition proof exists |

Parser acceptance, compatibility aliases, administrative shadow state, unavailable sources, and unsupported runtime stubs are not counted as full operational implementation.

## Current overall disposition

M027 is closed as `partial Proposal 170 support`. The canonical wire
surfaces, available sources, persistence dimensions, and required negative
behavior have internal evidence. The disposition is intentionally partial:
26 of the 43 pinned RouterInfo additions remain unavailable because no bounded
authoritative Emissary owner exists, and missing tunnel data planes remain
explicitly unsupported under ADR-0001.

Retained candidate implementation includes:

- HTTPS I2PControl service with bounded request and connection handling;
- typed twelve-tunnel administrative inventory and exhaustive unsupported
  backend registry (only the existing generic startup client/server managers
  have runtime data-plane support);
- durable generation stores;
- direct Proposal 170 parameter forms;
- passive service registry;
- bounded SAM observation handle;
- exact 43-key RouterInfo inventory scaffold;
- live event counters and log ring sources;
- explicit unavailable/unsupported behavior in several paths.

M023's scoped startup/client-service correction, M024's SAM source correction,
M025's source reconciliation, M026's bounded-source audit, and M027's final
conformance/reclosure are closed. No successor handoff is currently blocked on
M027.

## Base I2PControl/JSON-RPC

Current status: implementation complete; closure recorded in M020.

M020 establishes canonical `API`/`Password` authentication, numeric `API`
responses, standard `params.Token` extraction with the header retained only as
an unambiguous compatibility path, distinct I2PControl authentication and
version errors, notification execution with response suppression, strict
request-ID validation, and direct base RouterInfo selector compatibility.

M027 independently rechecked this surface. The base contract is wire- and
evidence-supported; token state remains intentionally in-memory and therefore
does not claim persistence.

## TunnelManager

Current status: M023 implementation complete and closed for its bounded
source/lifecycle correction; the overall subsystem remains corrective-pass work

Retained:

- seven lowercase canonical action parser;
- twelve exact tunnel type parser;
- durable administrative definitions;
- explicit unsupported backend for every missing data plane;
- safe inactive behavior for unsupported runtimes.

M021 now also provides:

- exact canonical `result.status`/`result.info`/`rawConfig` response shapes;
- strict canonical action/option/type/range validation;
- one-publication create/edit/rename/delete persistence;
- restrictive-permission enforcement and temporary-file cleanup on failure;
- secret-safe typed persistence and response serialization;
- separate compatibility serializers for capitalized actions and `List`.

M023 corrections:

- startup-configured generic client/server tunnels are now mapped at
  composition into a bounded, read-only `StartupManaged` source shared with
  production TunnelManager and ClientServicesInfo;
- persisted startup-name collisions fail closed, and control-plane create,
  rename, delete, and lifecycle operations cannot target startup ownership;
- no safe named lifecycle adapter exists: the current generic managers own
  retrying task sets rather than independently cancellable named tasks, so
  startup lifecycle remains explicitly externally managed;
- proxy task exit publishes `Stopped` with generation fencing;
- ClientServicesInfo uses client remote destinations and server session-published
  destinations only, with explicit error behavior for unknown address state.

Closure: `plans/closure/i2pcontrol-proposal-170/023-closure.md`.

### Missing tunnel data planes

The following remain intentionally out of scope for this Proposal 170 corrective sequence:

- HTTP client/server and bidirectional server;
- IRC client/server;
- SOCKS-IRC and CONNECT variants;
- Streamr client/server;
- any other missing listener/destination/LeaseSet/traffic implementation.

Their API definitions may persist and round-trip. Start/restart must return deterministic not-implemented operation status; stop must remain safe and inactive. They must never report running or open resources.

## AddressBook

Current status: runtime authority bridged; final source-matrix review is closed
in M025.

The runtime `AddressBookHandle` is now the single durable/mutable authority for
the four books, metadata, and lookup publication. Successful mutations are
visible through normal runtime lookup before success is returned. Legacy
administrative generations are migration input only and collision failures are
fail-closed.

Subscription/config selectors now return the pinned `{path, entries}` object
shape. Emissary has no actual path-backed source for these metadata objects, so
`path: null` is returned rather than a fabricated filesystem path.

Owner: M022, with final matrix integration closed in M025.

## ClientServicesInfo

Current status: direct wire contract and bounded source behavior closed in M027;
runtime availability remains selector-specific.

| Selector | Retained behavior | Corrective requirement |
|---|---|---|
| `I2PTunnel` | shared live startup/control-plane inventory | M023 implemented ownership, collision, bound, and address provenance rules |
| `HTTPProxy` | bind/listening observation | M023 publishes inactive state on task exit |
| `SOCKS` | bind/listening observation | M023 publishes inactive state on task exit |
| `SAM` | bounded active-session source | recovered incomplete-state semantics; final matrix review remains |
| `BOB` | exact boolean `false` | retained |
| `I2CP` | actual listener state | revalidate in final source matrix |

Owner: M024 for recoverable SAM observation; M023 owns tunnel/proxy source
truthfulness. M027 independently rechecked selector shape and failure behavior.

## RouterInfo

Current status: M025 matrix frozen; source dimensions are explicit.

The repository recognizes exactly 43 Proposal 170 additions. The reviewed
matrix contains 16 available fields, 1 protocol-permitted neutral field, and
26 explicitly unavailable fields, with exact JSON types, owner/reason,
serializer, bound, and fixture metadata. Availability is not inferred from
adjacent counters or from fake/test sources.

M026 audited bounded read-only snapshots only where authoritative state already
exists. No candidate met that threshold, so no new source was added. The 26
remaining fields stay explicitly unavailable or out of scope.

Fields that cannot be sourced without invasive redesign will remain explicitly unavailable and require a final `partial Proposal 170 support` disposition rather than a false complete claim.

## Persistence and security

Retained strengths:

- versioned complete generations;
- deterministic serialization;
- write/sync/rename publication;
- prior-generation fallback;
- bounded retention;
- authentication before protected handler execution;
- request and collection bounds;
- redacted tracing in several paths.

Corrective requirements:

- one generation per logical tunnel edit/rename;
- prior state preserved on publication failure;
- actual AddressBook/runtime consistency;
- secret values stored once and never unintentionally returned/logged;
- restrictive permissions enforced rather than best effort where supported;
- literal negative tests for tokens, passwords, keys, credentials, paths, and destinations.

Owners: M022, rechecked by M027.

## Corrective sequence

| Milestone | Status | Scope |
|---|---|---|
| M001 | Closed | Base protocol, auth, JSON-RPC |
| M002 | Closed | Tunnel domain, persistence, backend trait |
| M003 | Closed | AddressBook handler |
| M004 | Closed | TunnelManager contract and stubs |
| M005 | Superseded | RouterInfo inspection (superseded by M009/M010) |
| M006 | Superseded | ClientServicesInfo (superseded by M011) |
| M007 | Superseded | Conformance and strict closure (superseded by M012/M013) |
| M008 | Closed | Production composition and durable-state integrity |
| M009 | Closed | RouterInfo availability and truthfulness |
| M010 | Closed | Bounded core router inspection |
| M011 | Closed | ClientServicesInfo live state |
| M012 | Closed | Real TLS and request resource hardening |
| M013 | Closed | Production conformance and independent reclosure |
| M014 | Closed | Spec-constrained truthfulness and local hardening |
| M015 | Superseded | Historical reclosure; superseded by M017 |
| M016 | Closed | Bounded SAM session observation corrective pass |
| M017 | Invalidated | Historical final-head review; broad closure superseded by M018/M019 |
| M018 | Closed | Exact wire-contract reconciliation implementation |
| M019 | Closed against pinned revision | Pinned-revision independent reclosure |
