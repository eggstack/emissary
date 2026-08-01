# I2PControl Proposal 170 Milestone M026 — Closure Status

Status: closed internally against the pinned Proposal 170 revision

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/026-bounded-router-inspection-sources.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline reviewed: `f44d70e`

Implementation commits or pull requests:

- `42c2204` — source-count and exhaustive unavailable-field regression guards, documentation, and planning closure.
- No pull request was created or prepared.

## 1. Executive finding

M026 is complete as a bounded owner-audit milestone. The frozen M025 matrix
contains no feasible adjacent source, so adding a production snapshot would
have violated the plan's ownership, semantic, or no-new-history constraints.
All 26 unavailable fields remain explicit and fail closed. M027 is unblocked
for exact conformance and independent reclosure.

This is not a claim that every Proposal 170 RouterInfo field is available. The
current truthful source counts remain 16 available, 1 protocol-permitted
neutral, and 26 unavailable.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| M025 matrix is the complete input | M025 section 13 and implementation disposition | pass | Zero `M026 feasible` fields |
| No feasible group is silently skipped | Owner-grouped disposition below | pass | All traffic, network, tunnel-pool, NetDB, peer-limit, ban, peer-list, and peer-stats groups are retained |
| Unavailable fields remain truthful | Frozen source map and exhaustive handler test | pass | No zero, false, empty, or partial success is fabricated |
| Bounded request behavior remains intact | Existing preflight and response-bound tests; RouterInfo suite | pass | No new collection or history buffer |
| Read-only ownership remains intact | No production owner changes; existing adapter contract | pass | No event receiver, lock-across-await, or task lifecycle change |
| Documentation and status handoff are complete | Source map, support docs, roadmap, registry, and M027 status | pass | M027 is ready |

## 3. Owner-group disposition

| Owner group | Result | Reason retained |
|---|---|---|
| traffic-metrics | deferred unavailable | Exact rolling 15-second windows are not tracked; no sampler added |
| network | deferred unavailable | No exhaustive transport-specific integer status/error/testing mapping exists |
| tunnel-pool | out of scope | No bounded pool/detail/queue owner is exposed; ownership redesign is out of scope |
| netdb | deferred unavailable | No bounded current NetDB/RouterInfo snapshot owner exists |
| peer-limits | deferred unavailable | No authoritative NTCP/SSU limit owner is exposed |
| ban-list | deferred unavailable | No authoritative ban owner exists |
| peer-list | deferred unavailable | No bounded known/active peer snapshot owner exists |
| peer-stats | deferred unavailable | No bounded per-peer statistics owner exists |

## 4. Verification executed

### Commands run

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol production_composition -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest -- --nocapture
cargo check -p emissary-core
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo +nightly fmt --manifest-path emissary-cli/Cargo.toml -- --check
```

### Results

All required focused and package-scoped checks passed. The RouterInfo suite
passed 101 tests, including the exhaustive 26-field unavailable guard; the
conformance-manifest filter passed 6 tests; and the production-composition
filter passed 1 test. Core check, tests, and clippy passed. CLI feature check,
full feature test suite, and `-D warnings` clippy passed. The touched CLI
manifest rustfmt check passed. The workspace has unrelated pre-existing
stable-rustfmt differences outside this change; no workspace formatting was
rewritten.

## 5. Invariant review

- No new source mutates the router or changes timing/network behavior.
- No source consumes a single-owner receiver or holds a lock across an await.
- No unbounded collection, serialized peer RouterInfo, history buffer, sampler,
  poller, database, or background task was added.
- No aggregate metric is used to approximate a transport, peer, queue, or
  tunnel-pool field.
- Unavailable requests fail before unrelated source acquisition and return no
  partial result.
- The source-count guard prevents accidental reclassification without an
  explicit matrix update.

## 6. Failure, restart, and contention review

M026 adds no owner state and therefore adds no restart or recovery path. The
existing handler preflight rejects unavailable selectors before querying any
source; existing failure tests preserve no-partial-response behavior. Request
cancellation drops only local request values, and concurrent owner behavior is
unchanged because no owner snapshot seam was added. Existing bounded response
and serialization-failure tests remain in the focused suite.

## 7. Migration and compatibility review

There is no schema, storage, configuration, or wire migration. Existing base
selectors, canonical direct selectors, and compatibility forms are unchanged.
The only new behavior is test enforcement of the already-documented source
dispositions.

## 8. Security review

No new data is copied or serialized. Existing authenticated RouterInfo
dispatch, sanitized inspection errors, response bounds, log redaction, and
private-material exclusions remain unchanged. No credentials, keys, payloads,
private paths, or mutable owner handles are introduced.

## 9. Documentation and operations

The RouterInfo documentation now records M026's completed bounded-source audit
and the unchanged 16/1/26 disposition. Planning documents move M026 to closed
and M027 to ready. The source map remains the machine-readable contract's
projection, and the regression guards fail if the frozen source counts or
unavailable behavior drift.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| medium (capability limitation) | 26 pinned RouterInfo fields lack authoritative bounded sources | Those fields remain unavailable by design | M027 must state the final partial-support disposition honestly |
| high (evidence gate) | Final literal conformance and independent reclosure remain outstanding | The subsystem cannot yet claim final closure | M027, now ready |

These are tracked limitations and successor-gate work, not unresolved M026
implementation defects. No critical or M026-scoped high/medium correctness or
security defect remains.

## 11. Roadmap disposition

M026 is closed internally against the pinned revision. Its frozen source
disposition unblocks M027; the overall Proposal 170 subsystem remains
`corrective pass required` until M027 records the final status.

## 12. Registry updates

The registry, implementation README, subsystem roadmap, M026 plan status, M027
plan status, RouterInfo source map, support documentation, and RouterInfo
documentation were updated. M026 is in recently closed milestones, M027 is the
sole dependency-ready handoff, and no successor remains blocked on M026.

The pinned external specification was inspected read-only. No upstream or
third-party repository, issue, pull request, review, maintainer channel, or
other external write was made or prepared.

Frozen implementation/test head: `42c2204`
