# Proposal 170 Conformance Matrix

Status: partial Proposal 170 support; M039 final-head closure accepted

Proposal 170 remains Open. This workstream is pinned to the revision created
and last updated on `2026-05-20`.

Historical invalidation:

- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`

Completed implementation correction:

- M028, `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`
- M030, `plans/implementation/i2pcontrol-proposal-170/030-addressbook-destination-owner-coherence.md`
- M031, `plans/implementation/i2pcontrol-proposal-170/031-client-tunnel-runtime-backend.md`
- M034, `plans/implementation/i2pcontrol-proposal-170/034-addressbook-setter-truthfulness.md`
- M035, `plans/implementation/i2pcontrol-proposal-170/035-base-compatibility-and-selector-overlap.md`
- M038, `plans/implementation/i2pcontrol-proposal-170/038-live-runtime-interoperability.md`

Controlling final closure:

- M039, `plans/implementation/i2pcontrol-proposal-170/039-operational-reclosure.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/039-closure.md`

M019 is superseded and non-controlling. M020–M027 remain retained evidence, and
M027's final subsystem disposition remains historical invalidated evidence.
M028 is implemented and closed with the required boundary evidence. M030
reviewed the actual AddressBook final head and accepted partial Proposal 170
support. M031 and M032 made only the generic control-plane `client` and `server`
backends operational; the ten other tunnel families remain explicitly
unsupported. M039 independently revalidated the final source/runtime/evidence
boundary and accepted the partial-support disposition.

## Retained machine-readable matrix

The machine-readable Proposal 170 RouterInfo authority remains:

- `emissary-cli/src/i2pcontrol/rpc.rs::router_info_keys::PROPOSAL_170_CONTRACT`

It contains exactly 43 Proposal 170 additions and records, for each selector:

- exact key;
- exact JSON type and nullability;
- direct parameter-presence behavior;
- production source owner;
- source disposition;
- serializer;
- result/byte bounds;
- fixture identifier;
- compatibility/base separation;
- residual limitation.

Retained counts:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

The separate existing-I2PControl and Emissary compatibility inventories are not
counted in the 43 additions.

## Historical invalidation context

The retained method-level conformance work is not a current defect. The
post-M027 review historically found:

1. a later merge revived superseded M019 closure language over M027;
2. top-level documentation overstates the final support disposition; and
3. the Proposal 170 AddressBook control owner is active in ordinary
   no-feature/runtime-disabled AddressBook execution.

The third item is a conformance-boundary defect because an optional
administrative API must not silently change router lookup/persistence behavior
when the feature/service is inactive.

## Retained method-level conformance

### Base I2PControl and JSON-RPC

Retained M020 evidence covers:

- standard `API`/`Password` authentication;
- numeric `API` response and opaque token;
- standard `params.Token` protected requests;
- compatibility-only header token with conflict rejection;
- exact I2PControl authentication/version errors;
- notification execution and response suppression;
- explicit-null and strict request-ID behavior;
- direct base RouterInfo compatibility.

M035 freezes the currently claimed method surface in
`rpc.rs::methods::SUPPORT_INVENTORY`: `Authenticate` and `RouterInfo` are the
implemented base methods, `AddressBook`, `TunnelManager`, and
`ClientServicesInfo` are Proposal 170 methods, and `SetSubscriptions`/
`SetConfig` are already-shipped compatibility aliases. `GetKeys`, `GetRate`,
`RouterManager`, `NetworkSetting`, and `AdvancedSettings` remain explicit
standard `METHOD_NOT_FOUND` responses. No missing base method is represented
as a partial implementation.

### TunnelManager

Retained M021/M023 evidence covers:

- seven lowercase canonical actions;
- twelve exact tunnel types;
- exact action-specific parameters and validation;
- exact structured operation results;
- exact canonical `info` and nested `rawConfig` output;
- one-publication mutation and failure atomicity;
- process-crash atomicity and prior-generation recovery, with qualified
  power-loss durability where directory synchronization is available;
- secret-safe persistence and output;
- startup-managed inventory and ownership collision rules;
- explicit resource-free unsupported runtime behavior.

Missing tunnel data planes are not implemented and are not counted as runtime
coverage.

### AddressBook

Retained enabled-mode M022 evidence covers:

- four exact book identities;
- canonical add/replace/delete and subscription/config behavior;
- destination/hostname validation;
- one coherent runtime/durable owner;
- immediate lookup visibility;
- restart and current/backup recovery;
- exact RouterInfo source objects.

M028 added and tested the missing negative boundary evidence:

- no compile-time feature: no control-state read/write/influence;
- feature compiled but runtime disabled: same no-control-state behavior;
- enabled: retained M022 behavior;
- disable/re-enable: preserve/ignore/restore state without duplicate authority.

M030 additionally proves that enabled administrative, RouterInfo, Base32, and
Base64 views share one full-destination owner, including first activation,
historical seed repair, stale-file update/delete, download merge, and
re-enable deletion semantics.

M034 closes the setter-truthfulness gap: subscription replacement is applied by
the live downloader through a bounded command seam and durably published, while
all non-empty configuration mutations are rejected before persistence. The
Proposal 170 configuration inventory is explicitly classified into
request-selected paths and unsupported runtime fields; no inert metadata is
reported as a successful mutation.

M036 closes the remaining authentication and publication-hardening gap:
password comparison uses a reviewed constant-time primitive, failed
authentication has bounded peer-keyed throttling, and I2PControl-owned
publication syncs containing directories where supported before updating live
state. Existing current/backup and generation recovery formats remain readable.

### ClientServicesInfo

Retained M023/M024 evidence covers:

- six direct selectors by presence;
- truthful startup/control-plane I2PTunnel inventory;
- actual destination provenance;
- proxy listener exit updates;
- bounded recoverable SAM sessions;
- actual I2CP listener state;
- exact `BOB: false`.

### RouterInfo

Retained M025/M026 evidence covers:

- exact 43 keys and types;
- direct selection and requested-only results;
- source preflight before assembly;
- bounded available sources;
- protocol-permitted clock-skew null;
- exhaustive sanitized unavailable behavior;
- no fabricated zero/false/empty/adjacent values;
- no partial result on source or response-bound failure.

M028 does not alter this matrix. M029 revalidated the counts and focused
fixtures after the AddressBook feature-boundary correction.

M035 additionally proves that direct and nested request modes are distinct:
the nested base inventories use legacy serializers, while direct Proposal 170
requests retain exact presence/source semantics. The three exact selector
overlaps are maintained in an explicit mode-specific table and tested against
the inventory intersection. Direct and nested parameters cannot be mixed.

## Support dimensions

Every conformance claim remains separated into:

| Dimension | Meaning |
|---|---|
| Wire | exact request/response contract |
| Source | truthful current production owner |
| Runtime | real operational backend/service |
| Persistence | process-crash atomic/recoverable mutation; qualified power-loss durability |
| Feature isolation | inactive feature/service does not alter ordinary router behavior |
| Evidence | literal fixture plus failure/restart/composition/transition proof |

Compatibility aliases, parser acceptance, stored definitions, unsupported
runtime stubs, and unavailable sources are not full operational implementation.

## M028 acceptance effect

M028 may change only the AddressBook activation/composition boundary and
directly affected dependency/docs/tests. It must not change:

- canonical wire forms;
- RouterInfo source counts;
- SAM behavior;
- tunnel runtime support;
- control-state schema;
- resolver precedence;
- missing-data-plane scope.

A change to any of those areas requires a new recorded defect and separate plan.

## M030 final-status rule

M030 selected:

- `partial Proposal 170 support` when every implemented/claimed dimension is
  exact and evidenced but one or more sources/runtimes remain unavailable;
- `closed internally against pinned revision` only when every source/runtime
  dimension is actually available and evidenced;
- `corrective pass required` for unresolved high/medium defects;
- `blocked` when the external revision changed or required evidence cannot be
  obtained.

Under the current scope and retained 16/1/26 matrix, the expected honest status
is `partial Proposal 170 support`.

No status implies upstream review, acceptance, certification, adoption,
approval, or merge.

## Internal-only boundary

All work is internal to `eggstack/emissary`.

No corrective plan authorizes upstream issues, pull requests, merge requests,
reviews, discussions, submissions, patches, maintainer outreach, contribution
packages, adoption requests, or writes to any upstream or third-party
repository. External specifications and reference sources may be inspected
read-only solely for internal correctness.
