# Proposal 170 Support Status

Status: corrective pass required

Proposal 170 remains Open. This status is pinned to the `2026-05-20` revision.

The prior M019A `closed internally against pinned revision` disposition is invalidated by:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

Current corrective roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Next executable handoff:

- M020, `plans/implementation/i2pcontrol-proposal-170/020-base-i2pcontrol-and-jsonrpc-interoperability.md`

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
- typed twelve-tunnel inventory and exhaustive unsupported backend registry;
- durable generation stores;
- direct Proposal 170 parameter forms;
- passive service registry;
- bounded SAM observation handle;
- exact 43-key RouterInfo inventory scaffold;
- live event counters and log ring sources;
- explicit unavailable/unsupported behavior in several paths.

Material corrective work remains in M021–M027.

## Base I2PControl/JSON-RPC

Current status: implementation complete; closure recorded in M020.

M020 establishes canonical `API`/`Password` authentication, numeric `API`
responses, standard `params.Token` extraction with the header retained only as
an unambiguous compatibility path, distinct I2PControl authentication and
version errors, notification execution with response suppression, strict
request-ID validation, and direct base RouterInfo selector compatibility.

The subsystem remains open while M021–M027 complete method-specific and source
truthfulness work.

## TunnelManager

Current status: M021 implementation complete; M023 remains the source/lifecycle
successor and the overall subsystem remains corrective-pass work

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

Known defects:

- startup-managed tunnel inventory is not production-backed;
- canonical lifecycle status translation still depends on M023 for truthful
  startup-managed runtime sources.

Owner: M023 for the remaining startup/runtime source work.

### Missing tunnel data planes

The following remain intentionally out of scope for this Proposal 170 corrective sequence:

- HTTP client/server and bidirectional server;
- IRC client/server;
- SOCKS-IRC and CONNECT variants;
- Streamr client/server;
- any other missing listener/destination/LeaseSet/traffic implementation.

Their API definitions may persist and round-trip. Start/restart must return deterministic not-implemented operation status; stop must remain safe and inactive. They must never report running or open resources.

## AddressBook

Current status: durable administrative API retained; runtime/source correction required

Known defect:

The current four-book store is disconnected from the running router's normal address-book lookup source. A successful API mutation can therefore leave runtime resolution unchanged.

The corrective target is one narrow runtime owner adapter or one synchronously published authoritative state. Two independently authoritative stores and best-effort synchronization are prohibited.

Subscription/config RouterInfo shapes and source classifications also require correction.

Owner: M022, with final matrix integration in M025.

## ClientServicesInfo

Current status: direct wire scaffold retained; source/lifecycle correction required

| Selector | Retained behavior | Corrective requirement |
|---|---|---|
| `I2PTunnel` | live query of control-plane store | include startup-managed inventory and use actual I2P address provenance |
| `HTTPProxy` | bind/listening observation | publish inactive state on task exit |
| `SOCKS` | bind/listening observation | publish inactive state on task exit |
| `SAM` | bounded active-session source | recover from transient incomplete/overflow state without restart |
| `BOB` | exact boolean `false` | retained |
| `I2CP` | actual listener state | revalidate in final source matrix |

Owners: M023 and M024.

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

Owners: M021 and M022, rechecked by M027.

## Corrective sequence

| Milestone | Status | Scope |
|---|---|---|
| M020 | closed | base I2PControl authentication/token/error and JSON-RPC correctness |
| M021 | ready | TunnelManager exact wire, atomic persistence, secret boundary |
| M022 | blocked | actual AddressBook runtime authority and source objects |
| M023 | blocked | startup tunnel inventory and client-service lifecycle/address truthfulness |
| M024 | blocked | recoverable bounded SAM observation |
| M025 | blocked | exact RouterInfo contract/source matrix |
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
