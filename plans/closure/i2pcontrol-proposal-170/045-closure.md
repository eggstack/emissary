# I2PControl Proposal 170 Milestone M045 — Closure Status

Status: blocked

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/045-routerinfo-known-peer-directory.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md#milestone-045--known-peer-directory-sources--blocked`

Repository baseline reviewed: `b759038`

Implementation attempt:

- `5ae0477` — bounded public peer-directory source and direct RouterInfo integration

## 1. Executive finding

M045 is blocked. The attempted implementation uses one bounded, owned public peer-directory source
composed from `Router::inspection_snapshot()` at I2PControl startup. That source is not live and
therefore cannot satisfy the plan's canonical ProfileStorage directory requirement. The three
Proposal 170 fields remain unavailable until a separately authorized neutral live ProfileStorage
enumeration seam exists.

No `emissary-core/**` production file changed. The attempted promotion of the three fields is not
accepted; the truthful source matrix remains 16 available, 1 neutral, and 26 unavailable.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| `i2p.router.netdb.peers` | attempted handler/source fixture | qualified | shape and bounds pass, but source is startup-stale and field promotion is rejected |
| `i2p.router.netdb.peers.list` | attempted handler/source fixture | qualified | shape and bounds pass, but source is startup-stale and field promotion is rejected |
| `i2p.router.netdb.peers.info` | attempted source lookup fixture | qualified | fail-closed join passes, but source is startup-stale and field promotion is rejected |
| Live canonical ProfileStorage owner | `Router::inspection_snapshot()` composition | failed | one-shot snapshot is not a request-time live directory |
| Public API sufficiency | `RouterContext::profile_storage()` and `ProfileStorage::get_router_ids` | blocked | `Bucket` is private to `emissary-core::profile`; CLI cannot enumerate without a new core seam |
| Exact source accounting | `PROPOSAL_170_CONTRACT`, source-map, conformance manifest | failed | attempted 19/1/23 promotion is not accepted; truthful state is 16/1/26 |
| No-feature behavior | no-feature check below | pass | no new work is composed without `i2pcontrol` |

## 3. Production implementation evidence

The prior attempt added optional `PeerDirectorySource` plumbing and supplied
`CoreSnapshotPeerDirectory` from the existing bounded `Router::inspection_snapshot()` seam. That
plumbing has been removed from the active composition because it is precisely the rejected
stale-source path. A valid implementation requires a separately authorized neutral, cloneable
ProfileStorage enumeration primitive; M045 cannot add it under its zero-core-production-change
budget.

The handler fixture queries the attempted source, sorts IDs, applies the existing 10,000-item and
4 MiB response limits, and rejects an incomplete RouterInfo join. These are useful local guards but
do not establish live production evidence.

## 4. Verification executed

### Commands run

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
cargo fmt --all -- --check
```

### Results

- no-feature package check: pass;
- no-feature CLI tests: 56 passed;
- feature-enabled package check: pass;
- focused RouterInfo suite: 118 passed;
- conformance manifest: 58 passed;
- production composition: 8 passed;
- full feature-enabled CLI tests: 1,345 passed;
- feature-enabled clippy with `-D warnings`: pass;
- `git diff --check`: pass;
- repository-wide `cargo fmt --all -- --check`: qualified failure because the configured unstable
  rustfmt options are unavailable on the installed stable toolchain and pre-existing files require
  formatting; no broad formatting rewrite was applied.

The repository-wide formatter remains subject to the documented stable/nightly configuration
mismatch; only the touched Rust files were kept formatted without rewriting unrelated files.

## 5. Invariant review

- no NetDB query, polling loop, cache, selection policy, or router behavior was added;
- the rejected startup snapshot was public-only and carried no mutable authority;
- the active contract keeps the three fields unavailable, with no fabricated response path;
- direct presence and compatibility selector behavior are unchanged;
- unavailable M046–M051 fields remain fail-closed.

## 6. Failure and recovery review

The active path returns the existing sanitized unavailable error for all three fields. The rejected
attempt had fail-closed missing-RouterInfo, oversized-directory, and serialized-payload handling,
but those fixtures do not establish live production evidence. There is no partial result or
persistent migration.

## 7. Migration and compatibility review

No schema, storage, configuration, authentication, or legacy nested-selector
migration was introduced. The three canonical fields retain direct
presence-by-key semantics and exact array-of-string JSON types.

## 8. Security review

The rejected source carried no private keys, LeaseSet material, sockets, channels, session handles,
or message payloads. It is not active, and the current unavailable path exposes no internal paths
or source details.

## 9. Documentation and operations

Updated the active source map, RouterInfo documentation, Proposal 170 support
counts, conformance manifest expectations, and the active planning registry.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| high | The public `ProfileStorage` API does not expose a usable live directory enumeration seam to `emissary-cli`; `Bucket` is private and the only available inspection snapshot is startup-stale. | M045 cannot truthfully serve any of the three fields under its authorized path budget. | Create and authorize a separate neutral core read-only seam, then reopen M045; do not promote the startup snapshot. |

This finding blocks M045 and does not justify fabricating known-peer, active-peer, or transport
values.

## 11. Roadmap disposition

M045's implementation evidence is not accepted because its source is not live. M046 is not newly
unblocked. M047–M052 remain blocked on their named hard dependencies. Overall Proposal 170 status
remains `partial Proposal 170 support`.

## 12. Registry updates

- M045 is `blocked` pending a separately authorized neutral ProfileStorage enumeration seam;
- M046 remains `blocked`; no future plan became dependency-ready;
- active RouterInfo source counts remain 16 available, 1 neutral, and 26 unavailable;
- no upstream repository or maintainer channel was mutated; external authority
  was read-only.
