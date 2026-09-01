# Proposal 170 AddressBook Administrative API

Status: partial Proposal 170 support; M034 closed

Historical corrective implementation:

- M028, `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`
- M030, `plans/implementation/i2pcontrol-proposal-170/030-addressbook-destination-owner-coherence.md`
- M034, `plans/implementation/i2pcontrol-proposal-170/034-addressbook-setter-truthfulness.md`

The enabled-mode Proposal 170 wire, mutation, source, and persistence behavior
implemented by M022 remains retained evidence. M028 corrected the narrower
defect: the runtime control owner was constructed and used by ordinary
AddressBook execution even when I2PControl was not runtime-enabled.

M028 provides focused proof that no-feature and runtime-disabled execution
preserve legacy AddressBook behavior without reading, writing, migrating, or
consulting Proposal 170 control state. M030 corrected enabled-mode destination
and lookup coherence and independently closed this AddressBook dimension.

## Overview

The AddressBook API provides administrative management of four independent
address books, a live subscription source set, and a versioned configuration
view. M096 gives all thirteen pinned configuration keys explicit operational
semantics, with `theme` retained as inert metadata.

When I2PControl is enabled, successful entry mutations must be committed by one
runtime control owner and immediately visible to normal destination lookup.
Runtime precedence remains private, local, router, then published. The same
valid hostname may intentionally appear in multiple independent books; normal
lookup deterministically uses the first matching book in that order, while
typed list and lookup operations retain each book's own entry.
When enabled, the runtime owner is authoritative for both Base32 and Base64
lookup; a missing owner entry never falls through to a stale legacy destination
file. Published entries always contain validated full Base64 destinations.
First activation imports bounded destination files, and established state is
repaired only when a matching full destination exists. Re-enabling an
established owner does not resurrect deleted entries from stale legacy files.

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
{"SetConfig": {}}
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

### SetSubscriptions

`SetSubscriptions` replaces the complete bounded source set used by the active
AddressBook downloader. The manager accepts one bounded command at a time and
coalesces refresh work to the newest complete generation. A successful result
means the active source set and durable control state were both updated; remote
download success is not part of the setter's success condition. Refresh is
bounded follow-up work, so a worker that becomes unavailable after commit may
emit a diagnostic but cannot turn the completed mutation into an error. If the
downloader is unavailable before the manager accepts the command, the request
fails and the previous set remains active and recoverable. On platforms with directory
synchronization, the publication also reaches the documented power-loss
durability point; other platforms retain process-crash atomicity only.

Bounds:

- maximum 1000 subscriptions;
- maximum 2048 bytes per URL.
- URLs must be HTTP or HTTPS URLs with a host;
- aggregate subscription text is bounded to 4 MiB.

### SetConfig

M096 makes the complete pinned Proposal 170 configuration set operational. The
configuration is stored as a versioned typed part of the AddressBook owner’s
durable generation. A successful non-empty request means validation, configured
artifact preparation, and the active-generation publication point were reached.

Path values are relative to the enabled AddressBook administrative root. `.` and
`..` are normalized only when the result remains inside that root; absolute paths,
control characters, symlink escapes, reserved runtime artifacts, and non-regular
files are rejected before mutation. Configured files are replaced atomically on
their owning filesystem. The four book paths select AddressBook-owned JSON
snapshots; subscription, ETag, and Last-Modified paths feed the one existing
bounded downloader worker. The configured `log` path receives only a bounded
AddressBook diagnostic artifact and never redirects global logging.

| Proposal 170 key class | Keys | Operational meaning |
|---|---|---|
| Confined path | `subscriptions`, `published_addressbook`, `router_addressbook`, `local_addressbook`, `private_addressbook`, `etags`, `last_modified`, `log` | Selects an AddressBook-owned artifact or metadata file |
| Runtime behavior | `update_delay`, `proxy_port`, `proxy_host`, `should_publish` | Controls the existing refresh worker, outbound proxy, and bounded publication |
| Administrative metadata | `theme` | Durable round-trip only; no frontend/router side effect |

The table is exhaustive against the pinned Proposal 170 key inventory. Unknown
keys remain invalid. Legacy rejected/inert configuration metadata is not revived
during migration.

Bounds:

- maximum 1000 entries;
- maximum 256-byte keys;
- maximum 4096-byte values.
- `update_delay` is an integer number of hours from 1 through 720.
- `proxy_port` is non-zero and `proxy_host` is a bounded host/IP value.
- `should_publish` accepts only the strings `true` and `false`.

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
  - deterministic cross-book shadowing and precedence handling.

Former I2PControl administrative generations may be one-time migration input
only when no runtime control authority exists. They must never remain a second
authority.

The legacy runtime `addresses` cache is used only for derived Base32 lookup
indexes. It is never stored or emitted as a Proposal 170 destination. Legacy
destination files are read only through the bounded first-activation/repair
seam described above.

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

The corrected M028 implementation satisfies this disabled/default boundary, and
M030 revalidated it against the frozen final repository head.

## Security

- Authentication is required for all Proposal 170 operations.
- Full destinations, subscriptions, configuration values, and raw state are not
  logged.
- Input cannot select arbitrary filesystem paths.
- Configuration paths are confined and rejected before persistence when they are absolute,
  escaping, symlinked, reserved, or non-regular.
- Subscription commands are capacity-bounded; no request creates a detached
  refresh task or an unbounded queue.
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
replace the resolver, create a second downloader authority, or write arbitrary
paths. The original AddressBook module contains no JSON-RPC policy or request
DTOs.

## Closure rule

M028 implemented the feature boundary. M030 implemented and independently
reviewed the destination-owner activation, repair, lookup, download, restart,
and disable/re-enable evidence on the frozen final head.

No AddressBook document claims support for unavailable sources or unsupported
tunnel runtimes; the feature-isolation and destination-owner boundaries are
closed by M030.
