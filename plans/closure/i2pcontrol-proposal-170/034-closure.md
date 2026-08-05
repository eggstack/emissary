# I2PControl Proposal 170 Milestone M034 — Final Closure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/034-addressbook-setter-truthfulness.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/034-implementation-disposition.md`

Frozen implementation/test head reviewed:

- `be7bc16` — `feat: make I2PControl address book setters truthful`

Review date: 2026-08-05

## Final finding

M034 is formally closed. Proposal 170 AddressBook setters no longer report
success for inert runtime behavior. `SetSubscriptions` now changes the live
downloader source set through one bounded manager-owned command seam, publishes
accepted state durably, and schedules bounded refresh work through the existing
proxy/download/parse/merge path. `SetConfig` accepts only the empty no-op set;
all pinned non-empty keys are rejected truthfully before persistence.

The subsystem remains `partial Proposal 170 support`: unavailable RouterInfo
sources and unsupported tunnel families retain their explicit roadmap status.

## Closure matrix

| Dimension | Result |
|---|---|
| Active subscription owner behavior | pass |
| Durable/active generation coherence | pass |
| Restart restoration | pass |
| Queue failure and cancellation semantics | pass |
| Contention and refresh bounds | pass |
| URL/count/aggregate validation | pass |
| Full-destination owner coherence | pass |
| Exhaustive SetConfig disposition | pass |
| Inert metadata truthfulness | pass |
| Disabled/no-feature isolation | pass |
| Public wire compatibility | pass |
| Documentation and static guard evidence | pass |
| Security and sanitized diagnostics | pass |
| Scope/no-core-change guard | pass |
| Stable formatter tooling | inherited low finding; no implementation defect |

## Future-plan disposition

M035, Base compatibility and selector overlap, is the only future plan newly
unblocked by this closure and is moved from `blocked` to `ready` in the active
registry and subsystem roadmap. M036 through M039 remain blocked on their named
predecessors. No future plan was silently activated.

## Internal-only attestation

The implementation and closure evidence are internal repository records.
External specification material was accessed read-only. No upstream or
third-party issue, pull request, review, submission, adoption request,
maintainer contact, or connector write was created. The explicit maintainer
directive to commit and push authorizes publication of this repository branch
only.
