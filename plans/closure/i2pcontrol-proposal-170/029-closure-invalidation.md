# I2PControl Proposal 170 M029 Closure Invalidation

Status: corrective pass required

Invalidated closure:

- `plans/closure/i2pcontrol-proposal-170/029-closure.md`

Corrective owner:

- M030, `plans/implementation/i2pcontrol-proposal-170/030-addressbook-destination-owner-coherence.md`

Repository head reviewed:

- `9c35e7f3a09613bd63b51ad12b7832fe75724ab4`

## 1. Decision

M029's final subsystem disposition is invalidated. Its retained evidence for
M020–M028 remains usable, but its claim that no high- or medium-severity
correctness or owner-coherence defect remained is no longer controlling.

The current subsystem status is `corrective pass required`.

The expected bounded status after correction remains `partial Proposal 170
support`: 26 RouterInfo selectors remain truthfully unavailable and missing
tunnel data planes remain explicit unsupported runtimes.

## 2. Newly demonstrated defects

### 2.1 Active Base64 lookup can bypass the control owner

When the Proposal 170 AddressBook owner is active,
`AddressBookHandle::resolve_base64` reads the legacy
`addressbook/destinations/<hostname>.txt` file before consulting the active
control owner.

Proposal 170 update and delete operations mutate the control owner only. A
successful published-entry update or delete can therefore leave normal Base64
resolution returning the stale legacy file even though Base32 lookup and the
administrative API reflect the new control generation.

This violates the M022/M028/M029 owner-coherence claim that a successful
AddressBook mutation is immediately visible through normal runtime lookup.

### 2.2 Published control state can contain Base32 values instead of destinations

On first owner construction, the current implementation seeds the published
book from the legacy `addresses` map. That map contains hostname-to-Base32
lookup values, not the full Base64 destinations stored under `destinations/`.

The production AddressBook adapter serializes the stored `destination` field
directly. A seeded published entry can therefore expose a Base32 value where
the Proposal 170 AddressBook and RouterInfo surfaces require a destination.

`merge_downloaded` currently uses `or_insert`, so a pre-seeded incomplete value
is not repaired by a later download of the full destination.

### 2.3 Regression evidence did not cover the failing paths

The retained mutation/restart test checks that a deleted control entry is absent
from Base32 resolution, but does not assert Base64 resolution after deletion.
It also does not begin with an existing legacy destination file for the same
hostname.

The transition tests verify feature isolation and retained control-state bytes,
but do not prove that every active published entry contains a structurally valid
full destination.

The broad passing test count therefore did not exercise the conflicting lookup
source or the incomplete published seed.

## 3. Retained evidence

The following M029 conclusions remain retained unless M030 exposes a direct
regression:

- base I2PControl authentication, token, JSON-RPC, notification, and request-ID
  behavior;
- TunnelManager wire shape, validation, atomicity, secret handling, startup
  ownership, and unsupported runtime behavior;
- ClientServicesInfo startup/proxy/I2CP/SAM source behavior;
- the exact 43-selector RouterInfo contract and the 16 available / 1 neutral /
  26 unavailable source matrix;
- M028 compile-time and runtime feature isolation;
- optional `serde_json` feature ownership;
- internal-only/no-upstream compliance.

M030 must not reimplement or broaden these areas.

## 4. Required correction boundary

M030 owns only enabled-mode AddressBook destination/source coherence and the
directly affected status/evidence records.

Production changes outside `emissary-cli/src/i2pcontrol/**` must be limited to
the smallest required additions in:

- `emissary-cli/src/address_book.rs` for owner-aware lookup precedence, bounded
  full-destination loading/validation, and focused tests;
- `emissary-cli/src/main.rs` only if one narrow activation input must be wired.

No `emissary-core` change is authorized. No resolver-policy redesign, second
AddressBook authority, bidirectional synchronization framework, new persistence
schema, generic migration engine, polling task, event bus, or background
reconciler is authorized.

## 5. Transition semantics retained

M028's explicit transition model remains controlling unless M030 demonstrates
that it cannot satisfy canonical correctness:

- disabled/default mode uses legacy files and ignores control state;
- enabled mode uses one control owner;
- disabling preserves but ignores control state;
- re-enabling restores retained control state.

M030 does not need to merge arbitrary edits made while disabled into an already
established control authority. Doing so would require provenance/tombstones or a
cross-store transaction model outside this corrective scope. The implementation
must document and test this precedence instead of silently combining stores.

First activation without an established control authority may import legacy
published entries, but it must import validated full destinations rather than
Base32 cache values.

## 6. Closure consequence

Until M030 is implemented and independently reviewed:

- M029 is historical invalidated evidence;
- `partial Proposal 170 support` is not the current controlling subsystem
  status;
- the active registry must show `corrective pass required` and M030 as the only
  dependency-ready handoff;
- no work on missing tunnel data planes or unavailable RouterInfo selectors is
  unblocked.

## 7. Internal-only attestation

This invalidation concerns internal repository correctness only. No upstream or
third-party issue, pull request, review, submission, adoption request, merge
request, maintainer outreach, or contribution artifact is authorized.