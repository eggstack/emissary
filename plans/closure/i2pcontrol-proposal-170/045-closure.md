# I2PControl Proposal 170 Milestone M045 — Closure Status

Status: conditionally closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/045-routerinfo-known-peer-directory.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md#milestone-045--known-peer-directory-sources--conditionally-closed`

Repository baseline reviewed: `b759038`

Implementation commit:

- `5ae0477` — bounded public peer-directory source and direct RouterInfo integration

## 1. Executive finding

M045 is conditionally closed. The three known-peer Proposal 170 fields now use one bounded,
owned public peer-directory source composed from the existing core inspection
snapshot. IDs are sorted in the I2PControl boundary; public RouterInfo bytes
are Base64 encoded only at the wire boundary; missing bytes fail closed.

No `emissary-core/**` production file changed. The remaining 23 unavailable
fields are not promoted by this closure.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| `i2p.router.netdb.peers` | `resolve_proposal_peer_directory`; direct handler fixture | pass | bounded deterministic public IDs |
| `i2p.router.netdb.peers.list` | same source and fixture | pass | exact array-of-string shape |
| `i2p.router.netdb.peers.info` | source lookup and fixture | pass | missing RouterInfo fails closed; no empty substitute |
| No mutable core authority crosses boundary | `CoreSnapshotPeerDirectory` owns only `CoreSnapshot` public bytes | pass | no router/session/socket/key handle |
| Exact source accounting | `PROPOSAL_170_CONTRACT`, source-map, conformance manifest | pass | 19 available, 1 neutral, 23 unavailable |
| No-feature behavior | no-feature check/test command below | pass | source is only composed under `i2pcontrol` |

## 3. Production implementation evidence

`ServerInitContext` accepts an optional `PeerDirectorySource`. The production
composition root supplies `CoreSnapshotPeerDirectory` from the existing bounded
`Router::inspection_snapshot()` seam. `ProductionRouterInfoControl` retains
only the trait object and continues to expose no mutable router handle.

The handler queries the known-peer owner once per requested M045 group, sorts
IDs, applies the existing 10,000-item and 4 MiB response limits, and rejects an
incomplete RouterInfo join rather than fabricating a value.

## 4. Verification executed

### Commands run

```bash
EMISSARY_TARGET_DIR=/tmp/emissary-codex-target CARGO_TARGET_DIR="$EMISSARY_TARGET_DIR" \
  cargo check -p emissary-cli --no-default-features --features i2pcontrol
EMISSARY_TARGET_DIR=/tmp/emissary-codex-target CARGO_TARGET_DIR="$EMISSARY_TARGET_DIR" \
  cargo test -p emissary-cli --lib --no-default-features --features i2pcontrol router_info_handler --no-fail-fast
EMISSARY_TARGET_DIR=/tmp/emissary-codex-target CARGO_TARGET_DIR="$EMISSARY_TARGET_DIR" \
  cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --no-fail-fast
EMISSARY_TARGET_DIR=/tmp/emissary-codex-target CARGO_TARGET_DIR="$EMISSARY_TARGET_DIR" \
  cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast
git diff --check
```

### Results

- feature-enabled package check: pass;
- focused RouterInfo unit suite: 31 passed;
- conformance manifest: 58 passed;
- production composition: 8 passed;
- `git diff --check`: pass at review time.

The repository-wide formatter remains subject to the documented stable/nightly
configuration mismatch; only scoped formatting was reviewed for this change.

## 5. Invariant review

- no NetDB query, polling loop, cache, selection policy, or router behavior was added;
- only public serialized RouterInfo bytes cross the inspection boundary;
- bounds are checked before response assembly and serialized response size remains enforced;
- direct presence and compatibility selector behavior are unchanged;
- unavailable M046–M051 fields remain fail-closed.

## 6. Failure and recovery review

An absent source returns the existing sanitized unavailable error. A missing
RouterInfo for a known ID returns temporary unavailability for the complete
`peers.info` request. Oversized directories and serialized payloads are rejected;
there is no partial result or persistent migration.

## 7. Migration and compatibility review

No schema, storage, configuration, authentication, or legacy nested-selector
migration was introduced. The three canonical fields retain direct
presence-by-key semantics and exact array-of-string JSON types.

## 8. Security review

The source carries no private keys, LeaseSet material, sockets, channels,
session handles, or message payloads. RouterInfo output remains bounded and
the error path exposes no internal paths or source details.

## 9. Documentation and operations

Updated the active source map, RouterInfo documentation, Proposal 170 support
counts, conformance manifest expectations, and the active planning registry.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| medium | The existing public core seam is a bounded inspection snapshot rather than a new live profile-storage handle. | Later churn after composition is not represented by this source. | M046 readiness audit must decide whether its authorized neutral live-inspection seam can replace or extend this source without violating the path budget. |

This finding is bounded to the next milestone’s readiness audit and does not
justify fabricating active-peer or transport values in M045.

## 11. Roadmap disposition

M045's implementation evidence is present, but its live-source condition remains
outstanding; M046 is not newly unblocked. M047–M052 remain blocked on their
named hard dependencies. Overall Proposal
170 status remains `partial Proposal 170 support`.

## 12. Registry updates

- M045 moved from `ready` to `conditionally closed`;
- M046 remains `blocked` pending the named live-source condition;
- active RouterInfo source counts reconciled to 19 available, 1 neutral, and
  23 unavailable;
- no upstream repository or maintainer channel was mutated; external authority
  was read-only.
