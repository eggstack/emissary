# M026 Implementation Disposition — Bounded Router Inspection Sources

Status: closed for implementation; M026 closure accepted; M027 ready

Implementation commit: `fd8cf21`

## Disposition

M026 completed the bounded owner audit against the frozen M025 source matrix.
The matrix contains no `M026 feasible` field: every remaining candidate lacks
an authoritative bounded owner, requires new historical telemetry, requires
semantic mapping that Emissary does not maintain, or is tied to an out-of-scope
tunnel-pool/data-plane boundary. No production owner, sampler, queue, peer
index, ban list, or generic introspection surface was added.

The existing RouterInfo preflight and production adapters already fail closed
for those fields. This disposition adds regression guards for the frozen
16-available / 1-neutral / 26-unavailable counts and for every unavailable
canonical selector returning an explicit error with no result or fabricated
default.

## Requirement evidence

| Requirement | Evidence | Result |
|---|---|---|
| Use the complete M025 input | `025-implementation-disposition.md` and section 13 of the M025 plan | pass |
| Execute only feasible owner groups | Frozen matrix has zero `M026 feasible` fields; no owner implementation was added | pass |
| Preserve unavailable classifications | `conformance_manifest_has_frozen_m026_source_counts`; source-map rows remain 26 unavailable | pass |
| No fabricated unavailable values | `every_frozen_m026_unavailable_field_fails_without_fabricated_result` | pass |
| Validate before source queries and avoid partial results | Existing RouterInfo preflight plus `handle_router_info_no_partial_on_failure` | pass |
| Keep source ownership read-only and bounded | Existing production adapter/DTO boundaries; no new owner or background state | pass |
| Preserve sensitive-data exclusion | No new snapshot or serialized field; existing RouterInfo security tests remain applicable | pass |
| Keep documentation and planning synchronized | RouterInfo source map, support docs, roadmap, registry, and M027 handoff updated | pass |

## Exact changed files

- `emissary-cli/src/i2pcontrol/rpc.rs` — frozen source-count guard
- `emissary-cli/src/i2pcontrol/router_info_handler.rs` — exhaustive unavailable-field guard
- `docs/i2pcontrol/README.md`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/router-info.md`
- `docs/i2pcontrol/router-info-source-map.md`
- `plans/implementation/i2pcontrol-proposal-170/026-bounded-router-inspection-sources.md`
- `plans/implementation/i2pcontrol-proposal-170/027-proposal-170-conformance-and-reclosure.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/026-implementation-disposition.md`
- `plans/closure/i2pcontrol-proposal-170/026-closure.md`

No `emissary-core` source or runtime owner changed.

## Compatibility and security

No wire, storage, configuration, lifecycle, or migration behavior changed.
Unavailable non-null selectors continue to return a sanitized method error;
available zero/empty values remain distinct from unavailable results. No
private key, tunnel key, peer payload, credential, path, or mutable owner
handle is introduced.

## Residual evidence

- The 26 unavailable selectors remain an explicit Proposal 170 source
  limitation and are not counted as operational implementation.
- M027 remains responsible for literal external conformance and final
  independent reclosure.

## Scope and external-interaction attestation

External specifications and reference material were used read-only. No
upstream repository, issue, pull request, review, maintainer channel, or
third-party write was used or prepared. No missing tunnel data plane, router
algorithm, dependency, CI, frontend, sampler, or upstream contribution work
was added.

Frozen implementation/test head: `fd8cf21`
