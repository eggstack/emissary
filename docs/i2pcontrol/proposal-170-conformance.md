# Proposal 170 Conformance Matrix

Status: final internal conformance record; partial Proposal 170 support

Proposal 170 remains Open. This workstream is pinned to the revision created
and last updated on `2026-05-20`.

The matrix previously stored in this file was used by the M019A internal
closure. That closure and matrix are no longer normative because later audit
found material defects in the base I2PControl contract, method schemas,
runtime ownership, persistence atomicity, source classification, and fixture
selection.

Authoritative invalidation:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

Corrective roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Matrix rebuild owner:

- M025, `plans/implementation/i2pcontrol-proposal-170/025-routerinfo-contract-and-source-reconciliation.md`

Final literal conformance and independent review record:

- M027, `plans/implementation/i2pcontrol-proposal-170/027-proposal-170-conformance-and-reclosure.md`
- Closure: `plans/closure/i2pcontrol-proposal-170/027-closure.md`

M027's replacement literal fixture suite is
`emissary-cli/tests/m027_literal_fixtures.rs`. Compatibility fixtures remain
in separately named modules and are not included in canonical fixture counts.

## Why the prior matrix is invalid

The prior matrix included or accepted claims now known to be incorrect or
insufficiently evidenced, including:

- nonstandard mandatory `Username` authentication;
- string-valued authentication `API` response;
- header-only token transport instead of standard `params.Token`;
- incomplete I2PControl-specific authentication/version errors;
- notifications discarded before execution;
- invalid request-ID coercion;
- direct base RouterInfo compatibility not preserved cleanly;
- a canonical TunnelManager `get` shape that did not match Proposal 170
  `result.info` and nested `rawConfig`;
- non-atomic tunnel rename persistence;
- secret-bearing raw configuration handling without sufficient negative evidence;
- AddressBook success backed by an administrative shadow disconnected from
  runtime lookup;
- no production startup-managed tunnel inventory;
- proxy lifecycle state that could remain listening after task exit;
- sticky SAM observation overflow;
- RouterInfo source classifications and support claims that contradicted
  unavailable production sources;
- fixtures that proved current serializers rather than literal external wire
  expectations.

The former detailed rows remain available in repository history as historical
evidence. They must not be copied forward without revalidation.

## Final replacement matrix

M025 created, and M027 independently revalidated, one reviewed machine-readable
contract/source table covering exactly the 43 Proposal 170 RouterInfo additions.
For every selector it records
record:

- exact key;
- exact JSON type and nullability;
- direct parameter-presence behavior;
- authoritative production source owner;
- source disposition: available, protocol-permitted neutral, or unavailable;
- serializer;
- result count/byte bounds;
- literal fixture identifier;
- compatibility/base separation;
- residual limitation.

The replacement must also preserve a separate existing-I2PControl inventory
and a separate Emissary compatibility inventory. Neither may be counted in the
43 Proposal 170 additions.

## Method-level conformance required before M027

### Base I2PControl and JSON-RPC

M020 must establish exact authentication, token placement, error codes,
notification execution, request IDs, and direct base RouterInfo compatibility.

### TunnelManager

M021 must establish exact action parameters, option validation, structured
operation results, canonical `info/rawConfig`, atomic persistence, secret
handling, and explicit unsupported runtime behavior. M023 must add truthful
startup-managed inventory and only narrowly justified existing lifecycle
adapters.

### AddressBook

M022 must establish one coherent actual runtime/durable owner, exact canonical
entry/subscription/config behavior, destination validation, restart/failure
atomicity, and exact RouterInfo source objects.

### ClientServicesInfo

M023/M024 must establish actual listener/task lifecycle, truthful I2PTunnel
address provenance, startup inventory, and recoverable bounded SAM session
observation.

### RouterInfo

M025 must freeze exact wire/source classification. M026 may add only feasible
bounded read-only snapshots for authoritative state that already exists.
Unavailable fields must remain explicit and cannot be replaced by fabricated
zero/empty values.

## Support dimensions

Every replacement row and final claim must separate:

| Dimension | Meaning |
|---|---|
| Wire | exact request/response contract |
| Source | truthful current production source |
| Runtime | real operational backend |
| Persistence | durable and failure-atomic mutation |
| Evidence | literal fixture plus failure/restart/composition proof |

Compatibility aliases, parser acceptance, administrative shadow state,
unsupported runtime stubs, and unavailable sources are not full operational
implementation.

## Final-status rule

M027 restored the normative conformance matrix and final subsystem status.

Possible dispositions are:

- `closed internally against pinned revision` when exact wire behavior and
  every claimed source/runtime dimension have evidence;
- `partial Proposal 170 support` when one or more pinned sources remain
  truthfully unavailable;
- `corrective pass required` for unresolved high/medium defects;
- `blocked` when necessary evidence cannot be obtained.

The selected status is `partial Proposal 170 support`: exact wire behavior and
claimed source/runtime/persistence dimensions are evidenced, while unavailable
sources and unsupported data planes are listed explicitly. No status implies
upstream review, acceptance, certification, adoption, approval, or merge.

## Internal-only boundary

All work is internal to `eggstack/emissary`.

No corrective plan authorizes upstream issues, pull requests, merge requests,
reviews, discussions, submissions, patches, maintainer outreach, contribution
packages, or writes to any upstream or third-party repository. External
specifications and reference sources may be inspected read-only solely for
internal correctness.
