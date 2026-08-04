# Proposal 170 Support Status

Status: corrective pass required

Proposal 170 remains Open. This status is pinned to the `2026-05-20` revision.

Current invalidation:

- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`

Current roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Current handoffs:

- M028 ready: `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`
- M029 blocked: `plans/implementation/i2pcontrol-proposal-170/029-in-scope-conformance-reclosure.md`

M019 is superseded and non-controlling. M020–M027 remain retained corrective
evidence, but M027's final disposition is invalidated pending the M028
feature-isolation correction and M029 final-head review.

## Status model

Support is reported separately as:

| Dimension | Meaning |
|---|---|
| Wire | exact public request/response names, casing, presence semantics, and JSON types |
| Source | truthful current Emissary source exists |
| Runtime | a real backend performs the requested operation |
| Persistence | mutation is durable and failure-atomic |
| Feature isolation | disabled/default execution is unaffected by the administrative feature |
| Evidence | literal external-contract, failure, restart, composition, and transition proof exists |

Parser acceptance, compatibility aliases, stored administrative definitions,
unavailable sources, and unsupported runtime stubs are not full operational
implementation.

## Current overall disposition

The repository is not currently closed because:

1. the post-M027 merge restored superseded M019 status language; and
2. the Proposal 170 AddressBook control owner is currently constructed and used
   by normal address-book execution even when the service is not runtime-enabled.

The second issue allows default/disabled execution to consult and persist
`addressbook/control-state.json`, and it made `serde_json` unconditional in the
CLI dependency set.

M028 owns only this status/feature-boundary correction. M029 will perform the
independent final-head review.

Expected final disposition under the authorized scope remains
`partial Proposal 170 support` because 26 of the 43 RouterInfo additions lack
bounded authoritative sources and missing tunnel data planes remain explicit
unsupported runtimes.

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

M029 must rerun the focused evidence after M028.

## TunnelManager

Retained status: wire/persistence correction complete in M021, startup/source
correction retained from M023.

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

The following remain intentionally out of scope:

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
private, local, router, and published books, subscription/config metadata, and
normal lookup publication.

Current reopened defect:

- the control owner is not isolated from no-feature and runtime-disabled
  execution;
- normal startup may read retained control state and rebuild legacy lookup from
  it;
- normal downloads may update control state even when no I2PControl service is
  active.

M028 target:

- no-feature and runtime-disabled execution use legacy address files only and
  do not touch control state;
- enabled execution constructs one control owner and preserves M022 behavior;
- disabling preserves but ignores control-state files;
- re-enabling restores them;
- no second authority or schema migration is introduced;
- `serde_json` returns to feature ownership if no independent unconditional
  consumer requires it.

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

M028 does not alter these sources. M029 revalidates them.

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

M028 additionally must prove that disabled/default AddressBook execution cannot
be influenced by stale, corrupt, or attacker-planted Proposal 170 control state.

## Corrective sequence

| Milestone | Status | Scope |
|---|---|---|
| M020–M027 | retained evidence | base/wire/persistence/source corrections and literal review |
| M028 | ready | post-M027 status repair and AddressBook compile/runtime feature isolation |
| M029 | blocked on M028 | independent final-head in-scope conformance review |

## Final-status rule

M029 may select:

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
