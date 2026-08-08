# Proposal 170 Support Status

Status: partial Proposal 170 support

Proposal 170 remains Open. This status is pinned to the `2026-05-20` revision.

Historical invalidation:

- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`
- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md` (resolved by M040–M044)

Current roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Closed handoffs:

- M028 closed for implementation: `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`
- M029 closed: `plans/implementation/i2pcontrol-proposal-170/029-in-scope-conformance-reclosure.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/029-closure.md`
- M030 closed: `plans/implementation/i2pcontrol-proposal-170/030-addressbook-destination-owner-coherence.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/030-closure.md`
- M031 closed: `plans/implementation/i2pcontrol-proposal-170/031-client-tunnel-runtime-backend.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/031-closure.md`
- M032 closed: `plans/implementation/i2pcontrol-proposal-170/032-server-tunnel-runtime-backend.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/032-closure.md`
- M033 closed: `plans/implementation/i2pcontrol-proposal-170/033-tunnel-lifecycle-reconciliation.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/033-closure.md`
- M034 closed: `plans/implementation/i2pcontrol-proposal-170/034-addressbook-setter-truthfulness.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/034-closure.md`; disposition:
  `plans/closure/i2pcontrol-proposal-170/034-implementation-disposition.md`
- M035 closed: `plans/implementation/i2pcontrol-proposal-170/035-base-compatibility-and-selector-overlap.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/035-closure.md`; disposition:
  `plans/closure/i2pcontrol-proposal-170/035-implementation-disposition.md`
- M036 closed: `plans/implementation/i2pcontrol-proposal-170/036-auth-and-publication-hardening.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/036-closure.md`
- M037 closed: `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/037-closure.md`
- M038 closed: `plans/implementation/i2pcontrol-proposal-170/038-live-runtime-interoperability.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/038-closure.md`; disposition:
  `plans/closure/i2pcontrol-proposal-170/038-implementation-disposition.md`
- M039 closed: `plans/implementation/i2pcontrol-proposal-170/039-operational-reclosure.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/039-closure.md`
- M039's final disposition was invalidated by
  `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md` and
  corrected by M040–M044:
  - M040: `plans/closure/i2pcontrol-proposal-170/040-closure.md`
  - M041: `plans/closure/i2pcontrol-proposal-170/041-closure.md`
  - M042: `plans/closure/i2pcontrol-proposal-170/042-closure.md`
  - M043: `plans/closure/i2pcontrol-proposal-170/043-closure.md`
  - M044: `plans/closure/i2pcontrol-proposal-170/044-closure.md`

M019 is superseded and non-controlling. M020–M027 remain retained corrective
evidence, while M027's final disposition is historical invalidated evidence.
M028's implementation disposition and closure record contain the
feature-isolation correction evidence; M030's implementation disposition and
final-head closure are now controlling for AddressBook destination coherence.

## Status model

Support is reported separately as:

| Dimension | Meaning |
|---|---|
| Wire | exact public request/response names, casing, presence semantics, and JSON types |
| Source | truthful current Emissary source exists |
| Runtime | a real backend performs the requested operation |
| Persistence | process-crash atomic/recoverable mutation; qualified power-loss durability |
| Feature isolation | disabled/default execution is unaffected by the administrative feature |
| Evidence | literal external-contract, failure, restart, composition, and transition proof exists |

Parser acceptance, compatibility aliases, stored administrative definitions,
unavailable sources, and unsupported runtime stubs are not full operational
implementation.

## Current overall disposition

The repository is closed for the authorized in-scope dimensions. M044 reviewed
the corrected final head and accepted the final disposition as partial Proposal
170 support. M039 remains a historical invalidated closure.

M028 owns the status/feature-boundary correction. M030 owns the destination
authority, lookup precedence, bounded import/repair, and independent final-head
review. M034 owns live subscription replacement, bounded refresh control, and
truthful configuration rejection. M035 owns the base method inventory and the
mode-specific RouterInfo compatibility boundary. M038 owns production-composition
and live child-process interoperability evidence. M040–M043 own the corrective
implementation and regression evidence; M044 owns the accepted independent
final-head review.

Expected final disposition under the authorized scope remains
`partial Proposal 170 support` because 26 of the 43 RouterInfo additions lack
bounded authoritative sources and missing tunnel data planes remain explicit
unsupported runtimes.

## M038 live-runtime evidence

The live evidence is layered rather than presented as a universal network
certification:

- unit and fixture evidence remains in the retained M020–M037 suites;
- production-composition evidence launches the feature-enabled `emissary-cli`
  binary with its real TLS/authentication, AddressBook, RouterInfo,
  ClientServicesInfo, and TunnelManager owners;
- the child-process scenario is
  `emissary-cli/tests/i2pcontrol_live_runtime.rs` and exercises restart,
  persistence, bind failure/recovery, unsupported types, startup ownership,
  malformed requests, and bounded cleanup;
- local I2P data-plane formation is not claimed: this loopback run has no
  reseeded peer set, so client/server traffic and public server destination
  identity stability remain explicitly blocked at the formation boundary;
- subscription replacement is reported as unavailable when no HTTP downloader
  is composed, matching the existing documented `-32603` failure rather than
  claiming a successful refresh.

The run uses temporary loopback state, process-local credentials, generated
runtime-only destination material, and temporary service TLS material. No
secrets or generated state are committed.

## Base method and RouterInfo compatibility

The exact method support inventory is maintained in
`rpc.rs::methods::SUPPORT_INVENTORY`. Implemented base methods are
`Authenticate` and `RouterInfo`; `AddressBook`, `TunnelManager`, and
`ClientServicesInfo` are Proposal 170 methods; and `SetSubscriptions` and
`SetConfig` remain shipped compatibility aliases. `GetKeys`, `GetRate`,
`RouterManager`, `NetworkSetting`, and `AdvancedSettings` return standard
`METHOD_NOT_FOUND` errors and are not claimed as implemented.

RouterInfo direct requests use Proposal 170 presence/source semantics. The
historical nested `Selector` form accepts only base selectors, uses truthy
boolean selection, and retains legacy serializers. The three exact overlaps
(`i2p.router.news`, `i2p.router.addressbook.subscriptions`, and
`i2p.router.addressbook.config`) are explicitly table-driven and tested.

## Retained implementation

Retained candidate evidence includes:

- HTTPS I2PControl service with bounded request and connection handling;
- standard I2PControl authentication, token placement, error codes, JSON-RPC
  notification execution, and strict request IDs;
- exact Proposal 170 direct method/selector/action/type parsing;
- literal external-contract fixtures;
- typed twelve-tunnel administrative inventory and exhaustive unsupported
  backend registry;
- exact TunnelManager result shapes, validation, atomic persistence, and secret
  handling;
- operational control-plane-owned generic `client` and `server` runtime
  backends with startup ownership isolation; server identities are persistent,
  fixed-path, and redacted;
- startup-managed tunnel inventory and proxy lifecycle observation;
- bounded recoverable SAM observation;
- exact 43-key RouterInfo matrix and explicit unavailable behavior;
- enabled-mode runtime AddressBook authority.

M028 must not reimplement or broaden these areas.

## Base I2PControl and JSON-RPC

Retained status: method-level implementation complete in M020.

Behavior includes:

- `Authenticate` with `API` and `Password`;
- numeric `API` response and opaque `Token`;
- standard `params.Token` protected requests;
- compatibility-only header token with conflict rejection;
- distinct I2PControl authentication/version errors;
- notification execution with response suppression;
- explicit-null request IDs and strict invalid-ID rejection;
- direct base RouterInfo compatibility.

M029 reran the focused evidence after M028; the exact command outcomes are
recorded in `plans/closure/i2pcontrol-proposal-170/029-closure.md`.

## TunnelManager

Retained status: wire/persistence correction complete in M021, startup/source
correction retained from M023, and the generic control-plane `client` and
`server` runtime backends are operational from M031/M032.

Retained behavior:

- seven lowercase canonical actions;
- twelve exact tunnel types;
- exact `status`, `results`, `info`, and nested `rawConfig` shapes;
- strict action/option/type/range validation;
- one-publication create/edit/rename/delete;
- prior-state preservation on publication failure;
- restrictive permissions and temporary cleanup where supported;
- secret-safe persistence and response serialization;
- startup-owned name collision and mutation rejection;
- deterministic resource-free unsupported lifecycle behavior.

### Missing tunnel data planes

The generic `client` and `server` types are the real control-plane lifecycle
backends at this stage. They reuse the existing Yosemite streaming data planes
behind I2PControl-owned, per-name supervisors. Startup-managed client and
server definitions remain externally managed and reject administrative
lifecycle operations.

The following other tunnel families remain intentionally out of scope:

- HTTP client/server and bidirectional server;
- IRC client/server;
- SOCKS-IRC and CONNECT variants;
- Streamr client/server;
- any other missing listener, destination, LeaseSet, or traffic implementation.

Their definitions may parse and persist. Start/restart must return explicit
not-implemented operation status; stop remains safe and inactive. They must not
report running or open resources.

## AddressBook

Retained enabled-mode status: M022 established one runtime/durable authority for
private, local, router, and published books and normal lookup publication. M034
replaces the former inert subscription/configuration setter behavior.

M034 additionally proves:

- `SetSubscriptions` reaches the active downloader through one bounded typed
  command seam and publishes complete generations durably;
- restart restores the last accepted source set;
- pre-commit unavailability preserves the prior generation, while post-commit
  refresh-worker unavailability cannot turn the completed mutation into an
  error response;
- URL/count/aggregate bounds are enforced before mutation;
- every pinned `SetConfig` key has an explicit path/unsupported disposition;
- non-empty configuration requests never persist or report success;
- disabled/default execution still does not construct or consult the control
  command seam.

TunnelManager lifecycle reconciliation is now operational for control-plane
generic `client` and `server` definitions. `StartOnLoad` is honored only for
those definitions after durable state loads; startup-managed and unsupported
definitions remain explicit non-auto-start boundaries.

M028-corrected defect:

- the control owner had not been isolated from no-feature and runtime-disabled
  execution;
- normal startup could read retained control state and rebuild legacy lookup
  from it;
- normal downloads could update control state even when no I2PControl service
  was active.

M028 result:

- no-feature and runtime-disabled execution use legacy address files only and
  do not touch control state;
- enabled execution constructs one control owner and preserves M022 behavior;
- disabling preserves but ignores control-state files;
- re-enabling restores them;
- no second authority or schema migration is introduced;
- `serde_json` returned to feature ownership because no independent
  unconditional consumer requires it.

## ClientServicesInfo

Retained behavior:

| Selector | Retained source/runtime behavior |
|---|---|
| `I2PTunnel` | bounded startup/control-plane inventory with actual destination provenance |
| `HTTPProxy` | actual listener state and inactive publication on task exit |
| `SOCKS` | actual listener state and inactive publication on task exit |
| `SAM` | bounded active-session source with incomplete/recovery semantics |
| `BOB` | exact `false` |
| `I2CP` | actual listener state |

M028 does not alter these sources. M029 revalidated them.

## RouterInfo

Retained source matrix:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

Available selectors have bounded current owners. Clock skew uses `null` only
when the protocol permits it. Unavailable selectors fail with sanitized errors
before assembly and never return fabricated zero, false, empty, partial, or
semantically adjacent values.

M026 found no additional in-scope authoritative source. M028/M029 do not repeat
that audit or authorize new telemetry/core inspection.

## Persistence and security

Retained strengths:

- versioned complete generations;
- deterministic serialization;
- write/sync/rename publication;
- prior-generation fallback;
- bounded retention and response size;
- authentication before protected work;
- request and collection bounds;
- redacted diagnostics and response filtering;
- explicit resource-free unsupported backends.

M028 additionally proved that disabled/default AddressBook execution cannot be
influenced by stale, corrupt, or attacker-planted Proposal 170 control state.

## Corrective sequence

| Milestone | Status | Scope |
|---|---|---|
| M020–M027 | retained evidence | base/wire/persistence/source corrections and literal review |
| M028 | closed for implementation | post-M027 status repair and AddressBook compile/runtime feature isolation |
| M029 | historical invalidated closure | retained non-AddressBook evidence |
| M030 | closed; partial Proposal 170 support | AddressBook destination/lookup coherence and final-head review |
| M034 | closed | AddressBook setter truthfulness and runtime subscription control |
| M040 | closed | startup server cancellation-owner correction |
| M041 | closed | source-IP throttle identity and atomic reservation |
| M042 | closed | AddressBook post-commit refresh boundary |
| M043 | closed | corrective runtime regression validation |
| M044 | closed; partial Proposal 170 support | corrected final-head independent reclosure |

## Final-status rule

M030 selected:

- `partial Proposal 170 support` when all implemented/claimed dimensions pass
  but one or more sources/runtimes remain unavailable;
- `closed internally against pinned revision` only if every source/runtime
  dimension is actually available and evidenced;
- `corrective pass required` for unresolved high/medium defects;
- `blocked` when the proposal changed or required evidence cannot be obtained.

Under the current scope, `partial Proposal 170 support` is the expected honest
result. Explicit errors and unsupported stubs are not full operational support.

## Internal-only boundary

All work is internal to `eggstack/emissary`.

No plan authorizes upstream issues, pull requests, reviews, discussions,
submissions, patches, maintainer outreach, contribution preparation, adoption
requests, or merge activity. External specifications and reference sources may
be inspected read-only solely for internal correctness.
