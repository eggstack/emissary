# I2PControl Proposal 170 Milestone M031 — Final Closure

Status: partial Proposal 170 support

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/031-client-tunnel-runtime-backend.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/031-implementation-disposition.md`

Frozen implementation/test head reviewed:

- `8f635616c174e8681ba86de79f80ca3fff2cccee` — `feat: add I2PControl client tunnel runtime`

Review date: 2026-08-05

## Final finding

M031 is formally closed. Generic control-plane-owned `client` definitions now
have a real, truthful start/stop/restart path through the existing Yosemite
streaming client data plane. The runtime is independently supervised per exact
tunnel name, is bounded, cancellable, generation-fenced, and isolated from
startup-managed tasks.

The subsystem remains `partial Proposal 170 support`: generic `server` remains
blocked for M032, ten additional tunnel families remain unsupported, and the
retained RouterInfo unavailable-source boundary is unchanged.

## Closure matrix

| Dimension | Result |
|---|---|
| Client lifecycle capability | pass |
| Startup ownership boundary | pass |
| Runtime cancellation/restart/recovery | pass |
| Unsupported backend truthfulness | pass |
| Public wire compatibility | pass |
| Feature isolation and startup build | pass |
| Scope/no-core-change guard | pass |
| Documentation and planning reconciliation | pass |
| Security and sanitized diagnostics | pass |

## Future-plan disposition

M032 is the only future plan newly unblocked by this closure and is moved from
`blocked` to `ready` in the active registry, subsystem roadmap, and handoff
README. M033 remains blocked on M032 as well as M031; M034 through M039 remain
blocked on their named predecessor milestones. No plan was silently activated.

## Internal-only attestation

The implementation and closure evidence are internal repository records. No
upstream or third-party issue, pull request, review, submission, adoption
request, or maintainer contact was created. The explicit maintainer directive
to commit and push authorizes publication of this repository branch only.
