# I2PControl Proposal 170 Milestone M035 — Final Closure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/035-base-compatibility-and-selector-overlap.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/035-implementation-disposition.md`

Frozen implementation/test head reviewed:

- `5620cb8` — `feat: separate I2PControl compatibility modes`

Review date: 2026-08-05

## Final finding

M035 is formally closed. The claimed base I2PControl and RouterInfo
compatibility boundary now matches the implementation: direct Proposal 170
requests and historical nested requests are parsed, validated, sourced, and
serialized by explicit request mode. Exact overlaps are table-driven and
tested, while missing base methods remain honest standard errors.

The subsystem remains `partial Proposal 170 support`: unavailable RouterInfo
sources and unsupported tunnel families retain their explicit roadmap status.

## Closure matrix

| Dimension | Result |
|---|---|
| Direct Proposal 170 selector semantics | pass |
| Nested base compatibility semantics | pass |
| Exact overlap inventory and mode mapping | pass |
| Address-book legacy/canonical response shapes | pass |
| Router news legacy/direct disposition | pass |
| Base method support inventory and `METHOD_NOT_FOUND` behavior | pass |
| Mixed-mode rejection before source query | pass |
| Authentication/error/notification/request-ID retention | pass |
| Literal Proposal 170 fixtures | pass |
| Scope/no-core/no-upstream guard | pass |
| Stable formatter tooling | inherited low tooling finding; repository baseline remains nightly-sensitive |

## Verification outcomes

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info` | pass, 118 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol compatibility` | pass, 8 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest` | pass, 58 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test golden_fixtures` | pass, 44 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures` | pass, 7 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass, 1292 |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `git diff --check` | pass |
| `cargo fmt --all -- --check` | inherited stable/nightly formatter mismatch; no unrelated formatter spillover retained |

## Future-plan disposition

M036, Authentication and publication hardening, is the only future plan
unblocked by this closure and is moved from `blocked` to `ready` in the active
registry and subsystem roadmap. M037 remains blocked on M036; M038 remains
blocked on M031–M037; and M039 remains blocked on M038. No future plan was
silently activated.

## Internal-only attestation

Implementation and closure evidence are internal repository records. External
specification material was accessed read-only. No upstream or third-party issue,
pull request, review, submission, adoption request, maintainer contact, or
connector write was created. The explicit maintainer directive to commit and
push authorizes publication of this repository branch only.
