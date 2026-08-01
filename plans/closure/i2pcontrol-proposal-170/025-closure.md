# I2PControl Proposal 170 Milestone M025 — Closure Status

Status: closed internally against the pinned Proposal 170 revision

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/025-routerinfo-contract-and-source-reconciliation.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline reviewed: `c58f3d1`

Implementation commits or pull requests:

- `63afb65` — implement the exact RouterInfo contract/source reconciliation,
  tests, bounds, adapter behavior, and documentation.
- Closure commit: records this status and the dependency handoff.

## 1. Executive finding

M025 is implemented and closed internally. The pinned Proposal 170 RouterInfo
addition set is represented by one exact 43-row contract matrix: 16 available,
1 protocol-permitted neutral, and 26 unavailable. Runtime dispatch, serializers,
source ownership, bounds, tests, and documentation now consume the same
disposition. Unsupported fields fail explicitly instead of returning fabricated
success values.

The milestone closes the contract and source-reconciliation boundary. It does
not claim that the unavailable fields have been implemented. M026 is unblocked
to audit bounded inspection sources; the frozen M025 review currently identifies
no field as feasible without a new authoritative owner.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Exact 43-key set and types | `rpc::PROPOSAL_170_CONTRACT`; conformance manifest test | pass | Literal type and count assertions |
| Truthful source disposition | `ContractField` owner/reason metadata; unavailable preflight | pass | 16 available, 1 neutral, 26 unavailable |
| Base/canonical/compatibility separation | Separate inventories and partition tests | pass | `Selector` is compatibility-only |
| Direct-presence semantics | Protected dispatcher and direct RouterInfo tests | pass | Token metadata is removed first |
| One owner snapshot per request | Handler prefetch/cache paths and grouped source adapters | pass | Shared owners are acquired once |
| AddressBook shapes and requested-key filtering | Canonical literal-shape fixture | pass | Lists and path/entries objects are exact |
| I2PTunnel quick-info and bounds | M023 inventory source and count/byte guards | pass | Per-result and response bounds apply |
| Traffic and success semantics | Forwarded-counter and success-rate fixtures | pass | Zero total yields zero percent |
| Logs and clear mutation | LogRing adapter and canonical log tests | pass | Only exact clear selector mutates |
| Actual serialized response bound | Oversized log-payload regression test | pass | Post-serialization size is checked |
| Source-map parity | `router_info_source_map_documents_every_canonical_key_once` | pass | All canonical rows are documented |

## 3. Production implementation evidence

The production adapter exposes the existing M022 address-book owner, M023
startup/control-plane tunnel inventory, retained identity/configuration values,
event counters, the log ring, and the existing tunnel success counters. Router
news now returns an explicit unavailable reason because no canonical owner
exists. The clock-skew field remains nullable/neutral when no peer estimate is
available. The remaining fields are named unavailable by owner group and reason.

No router lifecycle, protocol, tunnel data-plane, transport, or NetDB owner was
invented to satisfy the administrative API.

## 4. Verification executed

### Commands run

```bash
cargo +nightly fmt --manifest-path emissary-cli/Cargo.toml -- --check
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol production_adapter -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol i2ptunnel -- --nocapture
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test
```

### Results

All focused suites passed: RouterInfo, conformance, production adapter,
AddressBook, and I2PTunnel. The complete emissary-cli I2PControl suite passed;
workspace tests passed; feature check and clippy passed with `-D warnings`.
The package-level rustfmt check passed. Workspace-wide rustfmt reports an
unrelated pre-existing style difference in `examples/rust-tutorial/src/main.rs`;
that file was not changed by M025.

## 5. Invariant review

- Every canonical key has one exact JSON type, source disposition, owner/reason,
  serializer, fixture, and bound in the matrix.
- Base selectors remain separate from the 43 additions and retain legacy
  behavior; canonical values are selected by presence, not truthiness.
- Unavailable sources are rejected before source acquisition.
- AddressBook and I2PTunnel results are bounded before being included.
- Response size is checked after serialization, including log payloads.
- RouterInfo remains a read-only administrative surface except for the exact
  log-clear mutation.

## 6. Failure and recovery review

Malformed selectors, unauthorized requests, unavailable owners, and oversized
results return deterministic JSON-RPC errors without partial success. Empty
owned collections remain valid empty collections; absent data is not converted
to a fabricated scalar. No new persistence or restart recovery path was added.
The handler snapshots shared sources before assembly, so a request does not
re-query the same owner while building multiple fields. Existing cancellation,
lock, and runtime ownership behavior remains authoritative.

## 7. Migration and compatibility review

No schema, storage, wire-protocol, or configuration migration is required.
Existing base selectors and the compatibility `Selector` form remain distinct.
Canonical additions use exact Proposal 170 keys and nested shapes. The only
behavioral correction is that fields without an owner now fail explicitly
instead of being reported as unsupported-shaped or fabricated success.

## 8. Security review

The common protected dispatcher still authenticates requests. Token metadata is
removed before selector validation. Error messages expose stable owner groups
and reasons only; they do not expose credentials, keys, paths, or payloads.
Collection and serialized-response bounds limit administrative response memory
and transport work. The log clear operation remains an exact, authenticated
mutation.

## 9. Documentation and operations

Updated documentation includes the RouterInfo contract, the frozen source map,
Proposal 170 support status, the implementation sequence, the subsystem
roadmap, and the planning registry. M026 now has a ready handoff with the
explicit frozen deferred set. M027 remains blocked until M026 and independent
final conformance/reclosure evidence are complete.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| medium | 26 Proposal 170 fields have no bounded authoritative owner | Those fields remain unavailable by design | M026 may add only demonstrated bounded owners |
| high | Independent final conformance/reclosure remains outstanding | Subsystem is not finally closed | M027 after M026 |

These are explicitly represented in the frozen matrix and are not hidden M025
defects. No critical or unbounded correctness/security finding remains in M025
scope.

## 11. Roadmap disposition

Milestone closed and the next dependency may proceed. M026 is now ready. M027
remains blocked on M026 and the final independent conformance/reclosure pass.
The overall Proposal 170 subsystem therefore remains `corrective pass required`
until M027 closes.

## 12. Registry updates

`plans/registry.md`,
`plans/implementation/i2pcontrol-proposal-170/README.md`, and
`plans/subsystems/i2pcontrol-proposal-170-roadmap.md` record M025 as implemented
and M026 as the sole dependency-ready handoff. This closure record is the formal
M025 gate. The pinned Proposal 170 source was inspected read-only; no upstream,
third-party, issue, pull-request, review, or maintainer-channel write was made.
