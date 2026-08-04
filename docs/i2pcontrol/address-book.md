# Proposal 170 AddressBook Administrative API

Status: corrective pass required for compile-time/runtime feature isolation

Current corrective owner:

- M028, `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`

The enabled-mode Proposal 170 wire, mutation, source, and persistence behavior
implemented by M022 remains retained evidence. M028 corrected the narrower
defect: the runtime control owner was constructed and used by ordinary
AddressBook execution even when I2PControl was not runtime-enabled.

M028 now provides focused proof that no-feature and runtime-disabled execution
preserve legacy AddressBook behavior without reading, writing, migrating, or
consulting Proposal 170 control state. M029 will independently review the
corrected final head.

## Overview

The AddressBook API provides administrative management of four independent
address books, a subscription set, and a configuration map.

When I2PControl is enabled, successful entry mutations must be committed by one
runtime control owner and immediately visible to normal destination lookup.
Runtime precedence remains private, local, router, then published. A hostname
collision across books is rejected rather than silently changing precedence.
The downloaded hosts source is the published source.

Canonical Proposal 170 requests use one `AddressBook` method and select exactly
one mode. The linked reference implementation returns operation details inside
the JSON-RPC `result` object, so Emissary uses
`result: {success, message}` for all mutation modes.

### Canonical entry mutation

```json
{
  "Type": "private",
  "Hostname": "example.i2p",
  "Destination": "base64-destination",
  "Delete": false
}
```

`Type` is exactly `private`, `local`, `router`, or `published`.
`Hostname` and `Destination` are required for entry operations. Presence of
`Delete` selects deletion regardless of its value; without it, the entry is
added or replaced.

### Canonical subscription/configuration modes

```json
{"SetSubscriptions": ["https://example.i2p/hosts.txt"]}
{"SetConfig": {"updateInterval": "3600"}}
```

These modes are handled inside `AddressBook`; they are not separate canonical
methods. Existing action-style and separate-method forms remain compatibility
extensions and cannot be mixed with canonical parameters.

## Methods

### Compatibility action-style AddressBook

The compatibility form performs CRUD operations on one of four books.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `book` | string | yes | `private`, `local`, `router`, or `published` |
| `request` | string | yes | `List`, `Lookup`, `Add`, `Update`, or `Delete` |
| `name` | string | operation-specific | Hostname |
| `value` | string | Add/Update | I2P destination |

Operations:

- **List** returns all entries in the selected book.
- **Lookup** returns one entry or `null`.
- **Add** creates a new entry and rejects duplicates.
- **Update** replaces an existing entry and rejects a missing hostname.
- **Delete** deletes the selected entry, or all entries in the compatibility
  delete-all form.

Compatibility forms do not count as canonical Proposal 170 coverage.

### Compatibility SetSubscriptions

The compatibility method atomically replaces the stored subscription set.
Subscriptions are not fetched synchronously by this API.

Bounds:

- maximum 1000 subscriptions;
- maximum 2048 bytes per URL.

### Compatibility SetConfig

The compatibility method atomically replaces the configuration metadata.
Values are inert strings and never choose files.

Bounds:

- maximum 1000 entries;
- maximum 256-byte keys;
- maximum 4096-byte values.

## Address books

| Book | Description |
|---|---|
| `private` | Private administrative book |
| `local` | Local administrative book |
| `router` | Router administrative book |
| `published` | Published administrative and downloaded source |

The four identities remain distinct at the API boundary.

## RouterInfo selectors

| Selector | Type | Description |
|---|---|---|
| `i2p.router.addressbook.private.list` | array | Canonical private entries |
| `i2p.router.addressbook.local.list` | array | Canonical local entries |
| `i2p.router.addressbook.router.list` | array | Canonical router entries |
| `i2p.router.addressbook.published.list` | array | Canonical published entries |
| `i2p.router.addressbook.subscriptions` | object | `{path, entries}` with truthful nullable path |
| `i2p.router.addressbook.config` | object | `{path, entries}` with truthful nullable path |

Existing shorter selector aliases remain compatibility-only.

## Enabled-mode persistence

When I2PControl is runtime-enabled, the target control owner persists one
complete bounded state with:

- JSON serialization;
- write/sync/rename publication;
- last-known-good rollback copy;
- serialized mutation ownership;
- current/backup recovery;
- deterministic collision handling.

Former I2PControl administrative generations may be one-time migration input
only when no runtime control authority exists. They must never remain a second
authority.

## Disabled/default behavior implemented by M028

Target behavior when the feature is absent or runtime-disabled:

- load normal `addressbook/addresses` and destination files;
- run existing subscription download and modified-time behavior;
- persist only normal legacy address sources;
- do not construct a Proposal 170 control owner;
- do not read, write, migrate, or consult `control-state.json`, its backup, or
  its temporary file;
- do not expose control-only entries through lookup;
- do not delete or modify retained control-state files from prior enabled use.

When I2PControl is re-enabled, retained control state is loaded again under the
enabled-mode precedence and migration rules.

The corrected M028 implementation satisfies this disabled/default boundary;
final subsystem closure remains pending M029.

## Security

- Authentication is required for all Proposal 170 operations.
- Full destinations, subscriptions, configuration values, and raw state are not
  logged.
- Input cannot select arbitrary filesystem paths.
- Path-like configuration values are inert.
- State and response sizes are bounded.
- Failed publication leaves the prior state.
- Disabled/default execution must not be influenced by stale, corrupt, or
  attacker-planted control-state files.
- Ordinary runtime lookup handles must not expose Proposal 170 mutation
  authority to unrelated consumers.

## Target runtime source map

| Execution mode | Source | Owner | Role |
|---|---|---|---|
| no feature / runtime disabled | existing `addresses` and destination files | `AddressBookManager` legacy path | lookup and downloaded published entries |
| I2PControl enabled | private/local/router/published control state plus downloaded published entries | one purpose-specific control owner sharing live lookup maps | Proposal 170 mutation, durability, and normal lookup publication |
| I2PControl disabled after prior use | legacy sources only; retained control files ignored and untouched | legacy path | no control-only lookup influence |
| I2PControl re-enabled | retained current/backup control state | control owner | restore enabled-mode authority |

I2PControl must receive only the dedicated bounded control handle. It must not
replace the resolver, control downloader task lifecycles, or write arbitrary
paths.

## Closure rule

M028 implemented the activation boundary with focused no-feature,
runtime-disabled, enabled, restart, and disable/re-enable evidence. M029 must
independently review the actual final head.

No AddressBook component document may return to a closed status before M029.
