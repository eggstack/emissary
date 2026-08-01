# Proposal 170 Support Status

Status: corrective pass required

Proposal 170 remains Open. This status is pinned to the `2026-05-20` revision.

The prior M019A `closed internally against pinned revision` disposition is invalidated by:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

Current corrective roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Next executable handoff:

- M025, `plans/implementation/i2pcontrol-proposal-170/025-routerinfo-contract-and-source-reconciliation.md`

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

The repository contains substantial retained Proposal 170 infrastructure, but it is not currently considered complete.

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

Material corrective work remains in M025–M027; M023's scoped startup/client-service correction and M024's SAM source correction are closed.

## Base I2PControl/JSON-RPC

Current status: implementation complete; closure recorded in M020.

M020 establishes canonical `API`/`Password` authentication, numeric `API`
responses, standard `params.Token` extraction with the header retained only as
an unambiguous compatibility path, distinct I2PControl authentication and
version errors, notification execution with response suppression, strict
request-ID validation, and direct base RouterInfo selector compatibility.

The subsystem remains open while M023–M027 complete method-specific and source
truthfulness work.

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

Current status: runtime authority bridged; final source-matrix review remains
owned by M025.

The runtime `AddressBookHandle` is now the single durable/mutable authority for
the four books, metadata, and lookup publication. Successful mutations are
visible through normal runtime lookup before success is returned. Legacy
administrative generations are migration input only and collision failures are
fail-closed.

Subscription/config selectors now return the pinned `{path, entries}` object
shape. Emissary has no actual path-backed source for these metadata objects, so
`path: null` is returned rather than a fabricated filesystem path.

Owner: M022, with final matrix integration in M025.

## ClientServicesInfo

Current status: direct wire scaffold retained; bounded SAM source correction closed; final source matrix remains

| Selector | Retained behavior | Corrective requirement |
|---|---|---|
| `I2PTunnel` | shared live startup/control-plane inventory | M023 implemented ownership, collision, bound, and address provenance rules |
| `HTTPProxy` | bind/listening observation | M023 publishes inactive state on task exit |
| `SOCKS` | bind/listening observation | M023 publishes inactive state on task exit |
| `SAM` | bounded active-session source | recovered incomplete-state semantics; final matrix review remains |
| `BOB` | exact boolean `false` | retained |
| `I2CP` | actual listener state | revalidate in final source matrix |

Owner: M024 for recoverable SAM observation; M023's tunnel/proxy source work is
implemented. M025 owns final selector/source reconciliation.

## RouterInfo

Current status: exact 43-key inventory scaffold retained; source/claim reconciliation required

The repository currently recognizes exactly 43 Proposal 170 additions, but many are unavailable and several source/type classifications contradict serializers or documentation. Unqualified completion is therefore invalid.

M025 will freeze one exact matrix containing key, JSON type, source owner, source status, serializer, and fixture.

M026 may add bounded read-only snapshots only where the authoritative state already exists. It must not add new historical samplers, polling loops, algorithms, peer categories, or fabricated defaults.

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
| M020 | closed | base I2PControl authentication/token/error and JSON-RPC correctness |
| M021 | closed | TunnelManager exact wire, atomic persistence, secret boundary |
| M022 | closed internally against pinned revision | actual AddressBook runtime authority and source objects |
| M023 | closed internally against pinned revision | startup tunnel inventory and client-service lifecycle/address truthfulness |
| M024 | closed internally against pinned revision | recoverable bounded SAM observation |
| M025 | ready | exact RouterInfo contract/source matrix |
| M026 | blocked | feasible bounded read-only router inspection sources |
| M027 | blocked | literal conformance, documentation reconciliation, independent closure |

See `plans/implementation/i2pcontrol-proposal-170/README.md` for dependencies and handoff rules.

## Final-status rule

Only M027 may restore a final status.

Possible dispositions:

- `closed internally against pinned revision` when exact wire behavior and every claimed source/runtime dimension have evidence;
- `partial Proposal 170 support` when one or more pinned sources remain truthfully unavailable;
- `corrective pass required` for unresolved high/medium defects;
- `blocked` when necessary evidence cannot be obtained.

No internal status implies upstream review, acceptance, certification, adoption, approval, or merge.

## Internal-only boundary

All work is internal to `eggstack/emissary`.

No corrective plan authorizes upstream issues, pull requests, reviews, discussions, submissions, patches, maintainer outreach, merge preparation, or writes to any upstream/third-party repository. External sources may be inspected read-only for internal correctness only.
