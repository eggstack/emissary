# I2PControl Proposal 170 Milestone M038 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/038-live-runtime-interoperability.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/038-implementation-disposition.md`

Corrective-pass dispositions:

- `plans/closure/i2pcontrol-proposal-170/038a-implementation-disposition.md`
- `plans/closure/i2pcontrol-proposal-170/038b-implementation-disposition.md`

Implementation/test head: `a5864d2` — `test: validate I2PControl live runtime interoperability`

Review date: 2026-08-05

## 1. Acceptance finding

M038 closes the live-runtime evidence gate. A real feature-enabled Emissary
child process started on loopback, served the production TLS/authentication
stack, and passed the bounded Proposal 170 administrative interoperability
scenario. Restart reused the same state directory and recovered authentication,
durable AddressBook state, tunnel definitions, and RouterInfo service health.

The closure is qualified exactly where the environment cannot provide a
reseeded I2P peer set: client/server traffic formation and public server
destination identity stability are not claimed. Subscription refresh is also
reported unavailable when the HTTP downloader is not composed. Neither limit
was replaced with a fake backend or success response.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Real production composition and TLS/authentication | live child process, HTTPS `Authenticate`, wrong-password rejection, protected calls | pass |
| JSON-RPC compatibility | notification, explicit IDs/null selectors, malformed request isolation | pass |
| AddressBook behavior | full destination add/lookup/list/delete, unsafe config rejection, documented subscription unavailable path, restart persistence | pass/qualified |
| RouterInfo and ClientServicesInfo truthfulness | available selectors, sanitized unavailable selector, live SAM/BOB inventory | pass |
| Generic client lifecycle | production create, forced bind failure, edit and restart recovery | pass/formation qualified |
| Generic server lifecycle | production create and start attempt, truthful failure/success status | pass/formation qualified |
| Unsupported and startup-owned boundaries | unsupported create/start error and startup mutation rejection | pass |
| Restart, failure isolation, cleanup | same state directory, new token, child stop/restart, bounded wait/kill, log redaction | pass |
| Scope, security, and governance | loopback/temp state, generated secrets, no CI/upstream changes | pass |

## 3. Future-plan disposition

M039 — Operational final-head reclosure — is now unblocked and marked `ready`.
It remains a distinct independent review and must choose the final truthful
subsystem status; M038 does not imply final Proposal 170 certification or
upstream acceptance.

## 4. Unresolved findings

- Local I2P formation without a reseeded peer set: low evidence limitation for
  traffic/public-destination assertions; no administrative correctness defect
  was found.
- No active HTTP downloader in the bounded configuration: low evidence
  limitation for positive subscription refresh; the service returned the
  documented unavailable error.
- Stable/nightly rustfmt option/formatting drift across the existing repository:
  low tooling finding; no unrelated changes were retained.

No unresolved high or medium correctness, security, compatibility, ownership,
containment, or scope defect remains in the implemented dimensions.

## 5. Internal-only attestation

The M038 implementation, corrective passes, evidence, documentation, and
planning updates are internal to `eggstack/emissary`. No upstream or
third-party repository, issue, review, submission, or connector was mutated.
