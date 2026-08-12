# M061 Closure Record — Independent Containment Reclosure and Static-Guard Refresh

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-reclosure.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`

Accepted predecessor:

- M060 closure at `6085eca` (`c958c4d` planning close head)

Repository review baseline: `c958c4d998b1abde9ace730b4bdadcf5a838afc6`.

Upstream comparison baseline:
`9b43484a21d5a1291c4881cdae62a36c527f8c0f`, accessed read-only.

Implementation commit:

- `77a2555` — current exact-path containment manifest and focused static guard.

## 1. Executive disposition

M061 is closed. The independent review recomputed the pinned fork delta and
accepted the minimum justified non-`i2pcontrol` boundary: nine original
CLI/runtime paths and 23 core paths. The current authority is
`061-containment-boundary.toml`, enforced by `m061_containment.rs`.

M061 made no production Rust, package, runtime, configuration, workflow, or
release change. The accepted Proposal 170 support disposition remains 43
RouterInfo rows: 37 available, 1 protocol-permitted neutral, and 5
unavailable. M051 remains independently blocked because substantive news and
banned-peer owners are absent. No containment successor became dependency-ready.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| M058, M059, and M060 are accepted closed | Prior closure records and `c958c4d` baseline | pass |
| Pinned fork/upstream baselines are frozen | Manifest metadata and `git diff --name-only 9b43484..c958c4d -- emissary-cli/src emissary-core/src` | pass |
| Final non-policy path set was independently recomputed | Pinned compare: 36 `i2pcontrol` policy paths, 9 original CLI/runtime paths, 23 core paths; no unexplained path | pass |
| Original composition paths are justified | Manifest evidence for `config.rs`, `lib.rs`, `logger.rs`, and `main.rs`; M059 composition regressions | pass |
| Original runtime adapters are justified | Manifest evidence for AddressBook, HTTP/SOCKS, and client/server tunnel owners; M059 regression suite | pass |
| Core inspection paths are justified | Manifest evidence for 13 exact inspection/composition paths; M060 inspection/SAM/transport/tunnel tests | pass |
| Deep core hooks are individually justified | Manifest evidence for 10 exact SAM/NTCP2/SSU2/tunnel owner paths; M060 retained-owner review | pass |
| No uncertain or broad high-sensitivity allowance remains | Exact path arrays and evidence-set equality in `m061_containment` | pass |
| Current static guard is enforced | `m061_containment`: 7 tests passed | pass |
| Representative policy leakage is rejected | Manifest forbidden-term inventory and production-source scan | pass |
| Live/secret inspection boundary is rejected | SAM event and inspection declaration checks for socket, stream, key, and channel types | pass |
| Unsupported tunnel backends remain resource-free | Unsupported backend source guard; M037 and production composition tests | pass |
| Supported behavior remains regression-equivalent | Integrated CLI and core focused suites below | pass |
| RouterInfo and unavailable-source semantics remain unchanged | RouterInfo truthfulness suite; accepted M056/M060 matrix; M051 blocker | pass |
| No upstream or maintainer interaction occurred | Internal-only attestation in §8 | pass |

## 3. Exact retained boundary

The machine-readable manifest records every allowed path with owner, purpose,
consumer, upstream-necessity rationale, sensitivity, seam class, and prior
closure reference. Its exact groups are:

- feature/composition: `config.rs`, `lib.rs`, `logger.rs`, `main.rs`;
- original runtime adapters: `address_book.rs`, HTTP listener, SOCKS listener,
  client tunnel, and server tunnel;
- core inspection/composition: 13 exact paths covering public identity,
  inspection DTOs, router composition, SAM propagation, transport composition,
  and tunnel composition;
- core owner hooks: 10 exact SAM, NTCP2, SSU2, tunnel-pool, and transit paths.

No directory prefix is used as an allowance for a deep core path. The nine
M058 original-CLI budget paths that M060/M059 restored or did not retain are
not in the current compare set. No core path outside the accepted M060 23-path
set is present.

The retained deep facts are authoritative only at their named owners: SAM
lifecycle ordering; exact NTCP2/SSU2 I/O counts; directional tunnel-pool
lifecycle/queue facts; and transit admission/expiry/TBM queue facts. All
publications are passive, bounded, sanitized where text exists, and contain no
socket, key, mutable owner, payload, or command channel.

## 4. Verification executed

Passed:

```text
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m037_containment
rtk cargo check -p emissary-core
rtk cargo check -p emissary-cli --no-default-features
rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition (9)
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter (22)
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness (36)
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test client_services_live (22)
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m033_tunnel_lifecycle (3)
rtk cargo test -p emissary-core inspection --no-fail-fast (7)
rtk cargo test -p emissary-core sam --no-fail-fast (149)
rtk cargo test -p emissary-core transport::ntcp2 --no-fail-fast (43)
rtk cargo test -p emissary-core transport::ssu2 --no-fail-fast (253)
rtk cargo test -p emissary-core tunnel --no-fail-fast (138 passed, 1 ignored)
rtk cargo clippy -p emissary-core --all-targets -- -D warnings
rtk git diff --check
```

The M061 guard was also rerun after formatting the new test and passed 7/7.
The default/no-feature and feature-enabled package checks both completed
successfully.

Known non-blocking repository/toolchain results:

- CLI clippy reports the pre-existing frozen-path warning
  `clippy::to_string_in_format_args` at `emissary-cli/src/proxy/socks.rs:543`.
  Fixing it would be a forbidden M061 production edit.
- Stable `cargo fmt --all -- --check` reports the repository’s existing
  stable/nightly rustfmt option drift and formatting differences across frozen
  core, utility, CLI, and I2PControl files. The new M061 guard was formatted
  individually; `git diff --check` passes and no broad rewrite was made.

## 5. Invariant, failure, and security review

- No Proposal 170 selector, wire, administrative persistence, support policy,
  or JSON-RPC type is present in core or original runtime production paths.
- No unsupported tunnel backend allocates, binds, listens, or spawns.
- The RouterInfo matrix remains 37/1/5 and M051 remains blocked; no unavailable
  source was promoted.
- No new task, probe, sampler, event framework, metric store, dependency,
  persistence schema, or CI/release machinery was introduced.
- The guard is deterministic and fail-closed: a new non-policy source path
  changes the pinned compare set; a new policy term or live/secret declaration
  fails source checks; a broad core prefix fails exact-path checks.
- M061 has no runtime state, cancellation, restart, lock, or contention
  surface. Existing owner failure/recovery semantics remain covered by the
  M059/M060 focused suites.
- Retained deep hooks are passive and no-op when unused; they do not affect
  protocol decisions, timing, retransmission, congestion, tunnel selection,
  or lifecycle control.

## 6. Documentation and planning evidence

The M061 manifest is now the current containment authority. M037 remains
historical and was not rewritten. The implementation README, containment
roadmap, registry, and M061 plan now all record M061 closed. The registry has
no dependency-ready containment successor; M051 remains the only named future
blocker and is unchanged.

No migration, compatibility, operational, or configuration change occurred.
The supported I2PControl service remains opt-in and the default Emissary path
remains regression-equivalent.

## 7. Unresolved findings

No high or medium containment, security, behavior, compatibility, or scope
finding remains open. The pre-existing CLI clippy warning and repository-wide
rustfmt drift are low-severity tooling findings and do not compromise the
production-free containment closure.

## 8. Internal-only attestation

The pinned upstream source and commit identity were accessed read-only for
comparison. No upstream repository or maintainer channel was mutated; no
upstream issue, pull request, review, merge, adoption request, submission,
contribution artifact, branch, tag, release, or feedback request was created
or prepared. The only authorized remote operation is the push of this internal
`eggstack/emissary` branch.

**Disposition: M061 closed; containment corrective sequence complete; overall
Proposal 170 support remains partial; M051 remains independently blocked.**
