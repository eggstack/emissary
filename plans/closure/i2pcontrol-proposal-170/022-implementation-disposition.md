# M022 Implementation Disposition — AddressBook Runtime Bridge

Status: implemented

Selected model: Model A, the existing runtime address-book owner gains a narrow
administrative handle. `AddressBookManager` owns the mutable snapshot,
persistence, effective lookup index, and mutation serialization. The production
I2PControl adapter receives only an `AddressBookHandle`; it does not construct
or retain an administrative store as a second authority.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| One coherent owner | `emissary-cli/src/address_book.rs` runtime owner and `AddressBookHandle`; production composition passes the handle from `main.rs` into `ServerInitContext`. |
| Add/replace/delete coherence | Runtime handle mutations commit the snapshot, rebuild the normal lookup index, and are delegated by `ProductionAddressBookControl`; `runtime_control_mutation_is_visible_and_restart_safe` and production CRUD tests cover this. |
| Four-book isolation and precedence | Runtime books remain separate DTO maps; effective lookup order remains private, local, router, published. Handler isolation and runtime restart tests pass. |
| Atomic failure behavior | Mutations serialize through one owner lock and persist before publication; `runtime_owner_rejects_persistence_failure_without_runtime_change` proves a failed write leaves the prior value active. |
| Migration and collision policy | Legacy generations are imported once only when no runtime authority exists. Hostname collisions fail closed; migration and collision tests cover restart and rejection. Legacy files are retained as migration input and are not auto-deleted. |
| Validation and path boundary | Hostnames reject controls, separators, and oversized values. Destinations are base64-decoded and structurally parsed with Emissary primitives. No request value selects a path. |
| Canonical source objects | Subscription/config RouterInfo selectors return `{path, entries}` objects. Emissary has no path-backed owner for these metadata maps, so `path` is explicitly `null`; no fabricated path is returned. |
| Compatibility behavior | Existing router `AddressBookHandle` methods route their state through the same owner, and their serialized-address lock scopes were kept non-reentrant. |
| Security boundary | Runtime persistence is bounded, temporary-file/sync/rename based, and keeps a rollback copy. Values and roots are not included in errors or tracing. |

## Deliberate residuals

- Downloaded legacy hosts entries continue to arrive through the existing
  downloader and are represented in the runtime published book. Legacy sources
  may carry a base32 value rather than a full destination, so that source's
  existing representation is preserved; API mutations require structurally
  valid full destinations.
- Subscription fetching and router address-book scheduling remain owned by the
  existing asynchronous manager. I2PControl handlers do not fetch, spawn, or
  edit arbitrary files.
- The old `addressbooks/` generations remain on disk for operator recovery but
  are ignored after runtime authority is established.
- Final 43-selector integration and whole-subsystem status remain owned by
  M025–M027.

## External-source and scope attestation

Proposal 170 source review was read-only and limited to the official proposal
page: <https://i2p.net/en/proposals/170-i2pcontrol-expansion/>. No upstream
repository, issue, pull request, or external service was modified. Changes stay
within the M022 composition seams, runtime address-book owner, focused tests,
and directly affected documentation.

