# M025 Implementation Disposition — RouterInfo Contract and Source Reconciliation

Status: implemented

Implementation commit: `63afb65`

## Disposition

M025 rebuilt the exact pinned Proposal 170 RouterInfo addition matrix and made
the handler, serializers, tests, and support documents consume the same
machine-readable contract. The matrix contains exactly 43 additions: 16
available, 1 protocol-permitted neutral, and 26 unavailable. No unavailable
field emits a fabricated zero, false, empty list, or empty string.

The six address-book additions consume the shared runtime owner established by
M022. The I2PTunnel quick-info addition consumes the bounded startup/control-
plane inventory established by M023. Clock skew is neutral because the
production adapter has no peer estimate and the proposal explicitly permits
`null`. Router news is unavailable because an empty string would be fabricated.

## Requirement evidence

| Requirement | Evidence | Result |
|---|---|---|
| Exact 43-key set and uniqueness | `rpc::PROPOSAL_170_ADDITIONS`, `conformance_manifest_has_exact_types_and_source_counts` | pass |
| Exact nested JSON types | Literal 43-entry type table in `rpc.rs` and integration conformance test | pass |
| One disposition/owner/reason per field | `ContractField` metadata and conformance test | pass |
| Base/canonical/compatibility separation | Separate base aliases, canonical manifest, `Selector` form, and partition tests | pass |
| Direct-presence semantics after Token removal | protected server fixture plus RouterInfo direct request tests | pass |
| Unavailable validation before source reads | canonical validation loop precedes owner snapshot acquisition | pass |
| Requested-key-only output | canonical address-book literal fixture and existing direct selector tests | pass |
| Shared owner queried once per request | pre-acquired per-owner snapshots and existing grouped dispatch | pass |
| AddressBook exact list/object shapes | `canonical_address_book_shapes_are_literal_and_requested_only` | pass |
| I2PTunnel exact quick-info source and bounds | shared `tunnel_definition_to_get_result`, count/byte checks, M023 inventory source | pass |
| Transit semantics | `canonical_transit_bytes_returns_forwarded_counter_only` | pass |
| Total success-rate percentage and zero handling | canonical success-rate fixture and zero-total branch | pass |
| Logs and exact log-clear mutation | `canonical_logs_and_presence_semantics_are_literal` and LogRing adapter | pass |
| Actual serialized bound | `actual_serialized_response_bound_rejects_underestimated_log_payload` | pass |
| Documentation/source-map parity | `router_info_source_map_documents_every_canonical_key_once` | pass |

## Frozen M026 input

The 26 unavailable fields were reviewed by owner group:

- `traffic-metrics`: recent transit and rolling tunnel success rates require
  new history/samplers and remain deferred unavailable;
- `network`: v6 status, error, and testing fields lack exact integer mappings;
- `tunnel-pool`: participating/exploratory/client detail and queue fields have
  no bounded owner source and are out of scope;
- `netdb`, `peer-list`, and `peer-stats`: no bounded current snapshots exist;
- `peer-limits`: no authoritative NTCP/SSU limit owner exists;
- `ban-list`: no authoritative ban owner exists.

No field is currently `M026 feasible`. M026 is nevertheless dependency-ready
to perform the bounded owner audit and close with this explicit deferred set.

## Exact changed-file scope

Production and test changes are limited to the existing I2PControl RouterInfo,
AddressBook serializer, and production adapter seams, plus support/planning
documentation and conformance fixtures. No `emissary-core` production source,
router algorithm, transport, tunnel data plane, dependency, CI, frontend, or
upstream artifact was changed.

## Security and compatibility

RouterInfo remains authenticated by the common protected dispatcher. Token
metadata is removed before selector validation. Requests are read-only except
the exact log-clear selector. Errors expose only stable owner groups/reasons;
they do not expose paths, credentials, keys, payloads, or private state.
Base selectors remain accepted through the existing inventory, compatibility
`Selector` remains separate, and canonical direct values remain presence-based.

## Scope and external attestation

The pinned Proposal 170 page and reference material were inspected read-only.
No upstream repository, issue, pull request, review, maintainer channel, or
third-party write was used or prepared. All repository changes remain within
the user-authorized `eggstack/emissary` repository.

Frozen implementation/test head: `63afb65`
