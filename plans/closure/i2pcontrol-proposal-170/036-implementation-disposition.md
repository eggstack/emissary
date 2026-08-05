# M036 Implementation Disposition — Authentication and Persistent Publication Hardening

Status: implemented; closure accepted

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/036-auth-and-publication-hardening.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Implementation commits:

- `fc54ebd` — `feat: harden I2PControl auth and publication`
- `518e05b` — `test: cover publication cancellation semantics`

Frozen implementation/test head: `518e05b`

## Disposition

M036 is implemented within the I2PControl boundary. Password equality now uses
the existing workspace `subtle` dependency through a fixed-size padded
comparison with explicit length and size handling; the former hand-written
comparator and artificial timing comments are gone. Failed authentication is
bounded by accepted TCP peer address, a 256-entry deterministic eviction table,
a monotonic 60-second window, and a delay capped at one second. The lock is
released before sleeping, completed successful authentication clears the peer
state, and restart constructs an empty throttle.

Token capacity now evicts one oldest token rather than clearing every session.
Tokens remain random, opaque, memory-only, and restart-invalidated. No password,
token, or peer key is logged or returned in an error.

I2PControl-owned publication now shares bounded file writing, restrictive
permissions, stale-temporary rejection, atomic rename, and directory-sync
helpers. Generation stores sync the temporary file and containing directory
before updating live state. Fixed current/backup stores preserve the prior
generation and reject symlinks, irregular files, and stale temporary content.
The server destination store serializes concurrent writers. The runtime
AddressBook control-state publisher consumes the same helper without changing
its ownership or file format.

Documentation now distinguishes process-crash atomicity and prior-generation
recovery from power-loss durability, which is claimed only when directory
synchronization is available and succeeds. Existing generation, current/backup,
configuration, token, and Proposal 170 wire formats remain compatible.

No `emissary-core/**`, frontend, CI/release, protocol, router-algorithm, or
upstream files changed. The only dependency change reuses the already-present
audited `subtle` crate; no general authentication or storage framework was
introduced.

## Changed-file classification

Production:

- `emissary-cli/src/i2pcontrol/auth.rs` — constant-time comparison, throttle,
  deterministic token eviction, and focused tests.
- `emissary-cli/src/i2pcontrol/server.rs` — peer identity propagation and auth
  gate integration.
- `emissary-cli/src/i2pcontrol/stores/publication.rs` — bounded publication
  primitives.
- `emissary-cli/src/i2pcontrol/stores/generation_store.rs` — directory sync,
  stale-temp filtering, failure injection, and recovery tests.
- `emissary-cli/src/i2pcontrol/server_secret_store.rs` — shared publication,
  serialized writers, and recovery/contention tests.
- `emissary-cli/src/address_book.rs` — runtime control-state publication helper
  consumption only.

Documentation and dependency metadata were updated directly alongside the
implementation.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Reviewed constant-time password primitive | `subtle::ConstantTimeEq`, fixed-size padding, oversized-value test, no custom comparator | pass |
| Bounded failed-login throttle | peer-keyed `AuthThrottle`, capacity/window/delay constants, churn and server tests | pass |
| Successful reset and no lock-held sleep | success-reset test and delay-before-record implementation | pass |
| Token capacity remains bounded without global clear | oldest-token eviction test | pass |
| Fixed, bounded publication helper | shared helper and fixed store call sites | pass |
| File and directory sync before live update | synced-file/directory implementation and directory-sync failure test | pass |
| Prior-generation and corrupt-current recovery | generation fallback and server-secret current/backup tests | pass |
| Stale temporary files cannot win recovery | generation and server-secret stale-temp tests | pass |
| Permissions, redaction, and symlink rejection | existing permission/redaction tests plus server-secret symlink test | pass |
| Wire, formats, ownership, and no-core boundary retained | full package/conformance suite, changed-path review | pass |

## Unresolved findings

No M036 high or medium security, persistence, compatibility, or ownership
finding remains. Unsupported tunnel families, unavailable RouterInfo sources,
and the broader containment issue remain explicitly assigned to later roadmap
work.

## Internal-only attestation

External specification material, if consulted, was read-only. No upstream or
third-party issue, pull request, review, submission, adoption request,
maintainer contact, or connector write was created. The requested push applies
only to this internal repository branch.
