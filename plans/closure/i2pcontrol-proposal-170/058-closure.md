# M058 Closure Record — Non-I2PControl Fork-Delta Inventory and Containment Ledger

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/058-non-i2pcontrol-delta-inventory.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`

Repository baseline reviewed: `adb2f52543764b267b2bcb282d093111001ae4b2`

Planning head when closed: `e7109e5d132a497a01a181d65653a355527aac49`.
The commits after the fork baseline are planning-only and are not included in
the production comparison.

Upstream comparison baseline: `9b43484a21d5a1291c4881cdae62a36c527f8c0f`
(`eepnet/emissary` fork merge base pinned by the plan).

Implementation commits or pull requests:

- None. M058 is an internal planning/audit pass; its only deliverables are
  the ledger and planning/closure records. No production implementation was
  performed.

## 1. Executive finding

M058 is closed. The pinned fork delta is fully inventoried outside
`emissary-cli/src/i2pcontrol/**`: 47 non-I2PControl production/example paths
are represented exactly once in the machine-readable ledger. Each entry has
an owner, consumer, policy-leak decision, classification, next disposition,
rationale, and regression evidence.

The ledger records 5 required-composition paths, 16 required-owner-seam
paths, 14 candidate-consolidation paths, and 12 candidate-revert paths. It
contains no `uncertain` path. The exact M059 original-CLI budget and
provisional M060 core budget are frozen in the `[budgets]` section of
`058-containment-ledger.toml`.

No production, runtime, test-behavior, manifest, or release file was changed
by M058. The accepted Proposal 170 state remains partial with 43 RouterInfo
rows: 37 available, 1 protocol-permitted neutral, and 5 unavailable.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Exact fork/upstream baselines recorded | Ledger metadata and this record | pass | Fork `adb2f52`; upstream `9b43484a` |
| Compare is complete, including path categories | Pinned `git diff --name-status`/`--stat` and ledger validation | pass | 284 total changed paths; 47 non-I2PControl production paths |
| Every non-I2PControl production path appears once | Python `tomllib` validation against the pinned path list | pass | 47 ledger entries, 47 expected, 47 unique, no missing/extra |
| Every entry has required disposition fields | TOML parse and field validation | pass | No uncertain entries; all required fields present |
| Canonical owner necessity is explained | Per-path rationale and owner/consumer fields | pass | Deep SAM/transport/tunnel entries name the exact lifecycle or I/O fact |
| M059 path budget is exact | `budgets.m059_original_cli` plus conditional manifest list | pass | 11 original CLI paths; 3 manifests only if dependency cleanup is proven |
| M060 provisional path budget is exact | `budgets.m060_core` | pass | 32 core paths, grouped by generic/SAM/transport/tunnel source |
| RouterInfo disposition is unchanged | Baseline source records and closure invariant review | pass | 37/1/5 remains authoritative |
| M051 and unsupported tunnel types remain out of scope | Ledger rationales and roadmap disposition | pass | News/ban remains blocked; no data-plane source is proposed |
| M058 made no production changes | Working-tree/path audit against planning head | pass | Only ledger, plan, registry, roadmap, README, and closure artifacts changed |
| Planning artifacts are clean | `git diff --check` | pass | No whitespace errors |
| Internal-only boundary is preserved | Attestation in §9 | pass | No upstream write, review, submission, or maintainer contact |

## 3. Compare and inventory evidence

The pinned comparison range is:

```text
9b43484a21d5a1291c4881cdae62a36c527f8c0f..adb2f52543764b267b2bcb282d093111001ae4b2
```

The complete compare contained 284 paths:

| Category | Count |
|---|---:|
| `emissary-cli/src/i2pcontrol/**` production | 36 |
| original `emissary-cli/src/**` | 11 |
| `emissary-core/src/**` | 32 |
| tests | 18 |
| docs | 12 |
| plans | 168 |
| other (`Cargo*`, example, README/AGENTS/.gitignore) | 7 |
| total | 284 |

The 47 ledger paths are the 11 original CLI paths, 32 core paths, the three
workspace/package manifest paths, and the tutorial example. The only
non-production path in that 47-path audit set is the formatting-only tutorial
example; it is explicitly classified as `candidate-revert` and assigned no
future containment budget.

The pinned upstream commit is present locally and is an ancestor of the fork:
`git merge-base master 9b43484a21d5a1291c4881cdae62a36c527f8c0f` and
`git merge-base adb2f52543764b267b2bcb282d093111001ae4b2
9b43484a21d5a1291c4881cdae62a36c527f8c0f` both resolve to
`9b43484a21d5a1291c4881cdae62a36c527f8c0f`. No moving upstream branch was
used and no upstream advancement was silently incorporated.

## 4. Exact future budgets

M059 may modify exactly these original CLI/runtime paths, plus the three
manifests only when a direct code removal proves a dependency unused:

```text
emissary-cli/src/address_book.rs
emissary-cli/src/config.rs
emissary-cli/src/lib.rs
emissary-cli/src/logger.rs
emissary-cli/src/main.rs
emissary-cli/src/proxy/http/error.rs
emissary-cli/src/proxy/http/mod.rs
emissary-cli/src/proxy/http/request.rs
emissary-cli/src/proxy/socks.rs
emissary-cli/src/tunnel/client.rs
emissary-cli/src/tunnel/server.rs
Cargo.toml
Cargo.lock
emissary-cli/Cargo.toml
```

The manifest entries are conditional budget entries, not permission for
unrelated dependency cleanup. M059 must not modify core.

M060 receives the exact 32-path `budgets.m060_core` list in the ledger. It is
provisional until M059 closes and is subdivided there into generic inspection
plumbing, SAM/I2CP, transport, and tunnel owner paths. M060 must not add a new
core path without stopping for replanning.

The one `next_milestone = "none"` entry is
`examples/rust-tutorial/src/main.rs`; its pinned change is formatting-only.
There are no unresolved paths and no path is authorized for modification on
the basis of current use alone.

## 5. Invariant review

- Proposal 170 wire behavior was not changed.
- The RouterInfo source matrix remains 37 available / 1 neutral / 5
  unavailable.
- M051 remains blocked by absent substantive news/ban owners.
- Unsupported tunnel types remain unsupported and resource-free.
- No source, observer, callback, event, metric, task, state, dependency, or
  feature was introduced by M058.
- Historical M037 and M045–M057 closure records were not rewritten.
- No fabricated network-error, news, banned-peer, or transit-15-second source
  was proposed.

## 6. Failure, recovery, and contention review

M058 has no runtime lifecycle, task, lock, channel, network operation, or
contention surface. Diff/path read failures were handled by retaining the
path in the exact compare set; no path was guessed or silently omitted.
The ledger parser and set comparison passed. Re-running the inventory against
the pinned range is deterministic and yields the same 47-path set.

No production defect was discovered that requires stopping a later milestone.
The three policy-leak entries are bounded M059/M060 review targets, not
unresolved findings: `address_book.rs` owns an administrative overlay in the
current implementation, while `router/context.rs` and `router/mod.rs` carry
control-plane terminology/aggregation around otherwise neutral seams.

## 7. Compatibility and security review

There is no migration, schema, protocol, runtime, or operational effect. The
ledger records security-sensitive paths as `sensitive_path = true` where they
touch identity, sockets, sessions, transport I/O, or tunnel lifecycle. It
records no secrets or private/session material.

M059 is explicitly responsible for removing original-CLI administrative
policy without creating a second AddressBook authority. M060 is explicitly
responsible for proving that any retained deep protocol hook is necessary for
truth, ordering, or bounds. The ledger does not authorize changes to router
algorithms, transport decisions, tunnel construction/routing, cryptography,
I2NP, or unavailable Proposal 170 sources.

## 8. Verification executed

The following bounded checks were executed:

```bash
git merge-base master 9b43484a21d5a1291c4881cdae62a36c527f8c0f
git merge-base adb2f52543764b267b2bcb282d093111001ae4b2 9b43484a21d5a1291c4881cdae62a36c527f8c0f
git rev-list --count 9b43484a21d5a1291c4881cdae62a36c527f8c0f..adb2f52543764b267b2bcb282d093111001ae4b2
git diff --name-status --find-renames 9b43484a21d5a1291c4881cdae62a36c527f8c0f..adb2f52543764b267b2bcb282d093111001ae4b2
git diff --stat --find-renames 9b43484a21d5a1291c4881cdae62a36c527f8c0f..adb2f52543764b267b2bcb282d093111001ae4b2
git diff --check
python3 -c '<tomllib parse and exact 47-path set validation>'
rg -n 'Proposal 170|I2PControl|JsonRpc|RouterInfo|ClientServicesInfo|TunnelManager' emissary-core emissary-cli/src -g '!emissary-cli/src/i2pcontrol/**'
rg -n 'inspection|observer|observation|snapshot|hook' emissary-core emissary-cli/src -g '!emissary-cli/src/i2pcontrol/**'
```

Results:

- both merge-base commands resolved to the pinned upstream SHA;
- fork distance was 255 commits;
- the compare was complete and reconciled to 47 ledger entries;
- the ledger parsed with Python `tomllib`, with 47 expected, 47 unique, and
  no missing or extra paths;
- targeted searches were reviewed semantically and produced the owner/
  consumer classifications recorded in the ledger;
- `git diff --check` passed;
- no broad Rust matrix or hosted CI run was required for this audit-only
  milestone.

## 9. Internal-only attestation

Upstream source/commit identity was accessed read-only for comparison. No
upstream repository or maintainer channel was mutated; no issue, pull request,
review, merge, adoption request, submission, contribution artifact, branch,
tag, or release was created or prepared. All writes remain within the
authorized internal `eggstack/emissary` repository.

## 10. Roadmap and registry disposition

M058 is closed and its ledger is now the controlling input for containment
edits. M059 is unblocked and advanced to `ready` with the exact original-CLI
budget above. M060 remains planned behind M059 closure; M061 remains planned
behind M060 closure. No other future plan became ready.

M051 remains blocked with its accepted semantic limitation. The overall
Proposal 170 workstream remains partial at 37/1/5 RouterInfo disposition.

**Disposition: M058 closed; audit complete; production unchanged; current
non-I2PControl delta fully classified; M059 ready; M060 and M061 remain
dependency-ordered; no upstream interaction occurred.**
