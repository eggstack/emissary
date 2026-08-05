# I2PControl Proposal 170 Milestone M032 — Final Closure

Status: partial Proposal 170 support

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/032-server-tunnel-runtime-backend.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/032-implementation-disposition.md`

Frozen implementation/test head reviewed:

- `3a03aea` — `feat: add I2PControl server tunnel runtime`

Review date: 2026-08-05

## Final finding

M032 is formally closed. Generic control-plane-owned `server` definitions now
have a real, truthful start/stop/restart path through the existing Yosemite
streaming server data plane. The runtime is independently supervised per exact
tunnel name, bounded, cancellable, generation-fenced, and isolated from
startup-managed tasks. Persistent destination identity is fixed-path,
recoverable, rename-stable, and redacted.

The subsystem remains `partial Proposal 170 support`: the ten other missing
tunnel data planes remain explicit unsupported backends, and M033 still owns
StartOnLoad, post-load reconciliation, and full lifecycle transaction closure.

## Closure matrix

| Dimension | Result |
|---|---|
| Generic server lifecycle capability | pass |
| Persistent identity and secret store | pass |
| Startup ownership boundary | pass |
| Runtime cancellation/restart/recovery | pass |
| Unsupported backend truthfulness | pass |
| Actual destination inspection | pass |
| Public wire compatibility | pass |
| Feature isolation and startup build | pass |
| Scope/no-core-change guard | pass |
| Documentation and planning reconciliation | pass |
| Security and sanitized diagnostics | pass |

## Future-plan disposition

M033 is the only future plan newly unblocked by this closure and is moved from
`blocked` to `ready` in the active registry, subsystem roadmap, and handoff
README. M034 through M039 remain blocked on their named predecessors. No plan
was silently activated.

## Internal-only attestation

The implementation and closure evidence are internal repository records.
External specification material was accessed read-only. No upstream or
third-party issue, pull request, review, submission, adoption request,
maintainer contact, or connector write was created. The explicit maintainer
directive to commit and push authorizes publication of this repository branch
only.
