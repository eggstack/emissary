# M037 Implementation Disposition — I2PControl Containment Boundary Reduction

Status: implemented; closure accepted

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Implementation commit:

- `5e003fe` — `feat: reduce I2PControl containment coupling`

Frozen implementation/test head: `5e003fe`

## Disposition

M037 extracted Proposal 170 administrative ownership into two feature-gated
I2PControl modules. `address_book_runtime.rs` now owns the administrative
runtime book, DTOs, persistence/control-state policy, migration/import/repair,
subscription control, bounds, and validation. `sam_observer.rs` now owns SAM
observation DTOs, bounded aggregation, generation/recovery policy, and the
snapshot used by ClientServicesInfo.

The core SAM implementation now exposes only a passive optional hook with
sanitized lifecycle facts. The hook is absent by default and is composed only
when I2PControl is enabled. The hook has no live handles, keys, sockets,
destinations, channels, or mutable session state. Direct synchronous
publication preserves authoritative activation/removal order without an event
queue or polling loop.

## Before/after production boundary

The baseline contained three Proposal 170 policy-bearing production files
outside `i2pcontrol/**`: `emissary-cli/src/address_book.rs` and the two core SAM
lifecycle files `emissary-core/src/sam/mod.rs` and
`emissary-core/src/sam/session.rs`.

After M037, the two policy owners are inside `i2pcontrol/**`. Four core files
(`lib.rs`, `router/mod.rs`, `sam/mod.rs`, and `sam/session.rs`) contain only
public/composition plumbing or the passive hook, not Proposal 170 aggregation
policy. The original AddressBook file retains a narrow runtime adapter and
legacy ownership. Thus the policy-bearing non-I2PControl count is reduced from
3 to 0; the retained adapter/composition surface is 6 production files, each
listed below and covered by the changed-path manifest.

## Changed-file classification

I2PControl-owned production files:

- `emissary-cli/src/i2pcontrol/address_book_runtime.rs` — administrative
  AddressBook owner, persistence, repair/import, validation, bounds, and
  subscription control.
- `emissary-cli/src/i2pcontrol/sam_observer.rs` — sanitized event consumer,
  bounded state, recovery, overflow handling, and snapshot serialization.
- `emissary-cli/src/i2pcontrol/client_services.rs` — consumes the local SAM
  snapshot owner.

Narrow adapter/composition files outside `i2pcontrol/**`:

- `emissary-cli/src/address_book.rs` — retains the legacy downloader, normal
  `AddressBook` runtime, path-confined legacy destination loading, and the
  bounded runtime command adapter. The administrative owner and its policy are
  no longer implemented here. The typed `RuntimeAddressBookHandle` facade is
  retained to avoid a second public runtime API and delegates to the owner.
- `emissary-cli/src/main.rs` — feature-gated composition only: constructs the
  optional observer and passes it into the router; it owns no aggregation or
  serialization policy.
- `emissary-core/src/lib.rs` — exports the passive hook types only.
- `emissary-core/src/router/mod.rs` — passes an optional hook through router
  construction; it does not create router behavior or retain observation state.
- `emissary-core/src/sam/mod.rs` — emits sanitized session lifecycle facts and
  holds the optional callback; no maps, recovery limits, DTO serialization, or
  administrative handle remain.
- `emissary-core/src/sam/session.rs` — emits sanitized socket lifecycle facts;
  session/socket ownership and protocol behavior remain unchanged.
- `emissary-cli/src/i2pcontrol/server.rs` and `mod.rs` — module/consumer wiring
  only; they do not import JSON-RPC handlers into legacy runtime modules.

Tests and boundary evidence:

- `emissary-cli/tests/m037_containment.rs` — manifest, import, hook-sensitivity,
  and unsupported-resource static guards.
- `plans/implementation/i2pcontrol-proposal-170/037-changed-path-boundary.toml`
  — machine-readable M037 changed-path classification.

## Retained blocks and reasons

| Retained block | Reason tied to runtime ownership |
|---|---|
| Legacy downloader and normal `AddressBook` trait in `address_book.rs` | The legacy runtime owns refresh/download scheduling and the normal address resolution trait; moving it would duplicate or replace non-Proposal-170 behavior. |
| Path-confined legacy destination import in `address_book.rs` | It is the adapter for the legacy on-disk source. Administrative import/repair policy and resulting state now belong to `address_book_runtime.rs`. |
| `RuntimeRefreshContext` and bounded command adapter | The existing runtime refresh task needs a bounded, async-safe bridge. The command types and administrative owner are now defined under I2PControl; the original module retains only the adapter-facing loop. |
| `RuntimeAddressBookHandle` facade | Existing runtime callers need a typed handle. It is a compatibility facade over the I2PControl owner, not a second store or policy implementation. |
| Core SAM hook points | Only core SAM owns authoritative activation/removal order. The retained code publishes sanitized facts and does not aggregate, serialize, recover, or control sessions. |
| Router/lib composition plumbing | Router construction and public exports are the narrow path needed to install an optional hook; they retain no Proposal 170 state. |
| `main.rs` observer wiring | The application composition root is the only place that can provide the feature-gated owner to the router; it performs no policy or lifecycle control. |

No retained block is a Proposal 170 wire handler, administrative persistence
owner, SAM aggregation owner, or unsupported tunnel backend.

## Boundary and security evidence

The changed-path manifest names every production change. Static tests reject
JSON-RPC dependencies in original runtime modules, live/secret types in the
core event contract, resource allocation in unsupported backends, and paths
outside the approved boundary. The observer has fixed session/socket bounds,
direct synchronous publication, explicit failure, and no global queue.

No core behavior, transport, protocol, router, crypto, tunnel algorithm,
frontend, CI/release, or upstream scope was added. No high or medium finding
remains. The repository's stable/nightly rustfmt mismatch remains a low,
pre-existing tooling finding and is documented in the closure record.

## Internal-only attestation

External specification material, if consulted, was read-only. No upstream or
third-party issue, pull request, review, submission, adoption request,
maintainer contact, or connector write was created. The push applies only to
this internal repository branch.
