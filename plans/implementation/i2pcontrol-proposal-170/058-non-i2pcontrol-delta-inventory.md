# M058 — Non-I2PControl Fork-Delta Inventory and Containment Ledger

Status: ready

Planning baseline: `adb2f52543764b267b2bcb282d093111001ae4b2` — merged M057 closure head

Upstream comparison baseline: `eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`

Corrective predecessor:

- M037 containment boundary reduction and accepted closure;
- M057 accepted planning-record closure.

Milestone class: corrective infrastructure/audit

Hard dependencies:

- M057 closed;
- current Proposal 170 supported behavior and 37 available / 1 neutral / 5 unavailable RouterInfo disposition accepted;
- upstream comparison baseline resolves to the fork merge base at planning time.

Applicable authority:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`;
- `plans/closure/i2pcontrol-proposal-170/037-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/056-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/057-closure.md`.

## 1. Bounded objective

Freeze and classify the complete production delta between upstream Emissary and the current fork **outside** `emissary-cli/src/i2pcontrol/**` before any further containment edits are attempted.

M058 is audit/planning work only. It must produce a machine-readable containment ledger that identifies, for every changed non-`i2pcontrol` production path:

- why the path differs from upstream;
- the canonical runtime owner involved;
- which already-supported I2PControl/Proposal 170 behavior consumes it;
- whether the delta is required, removable, consolidatable, unrelated, or uncertain;
- whether the path contains policy that belongs in `i2pcontrol`;
- whether a later corrective milestone may touch it;
- the exact regression evidence needed before removal/consolidation.

No production behavior is changed under M058. The purpose is to prevent M059/M060 from doing security-sensitive edits based on guesses or file-count targets.

## 2. Why this corrective pass is required

M037 correctly established the containment principle, but the later RouterInfo source-completion sequence added legitimate live observations after M037. As a result, the old M037 machine-readable boundary is historical and does not describe the complete current diff.

At the M058 planning baseline, a compare against upstream shows most Proposal 170 code under `emissary-cli/src/i2pcontrol/**`, but original production paths differ across CLI integration, AddressBook, proxy/tunnel runtime, logging, core inspection/router plumbing, SAM/I2CP lifecycle, transport protocol paths, and tunnel paths.

The current evidence does **not** justify treating every such path as contamination. Some facts can only be observed at their canonical owner. Conversely, the breadth of the diff means every retained path should now be re-justified after the source-completion work stabilized.

Prior verification proved method behavior and source truthfulness, not a final path-by-path minimum-delta proof. M058 adds that missing evidence class.

## 3. Scope and authorized paths

Production code changes: **none**.

Primary authorized files:

- `plans/implementation/i2pcontrol-proposal-170/058-non-i2pcontrol-delta-inventory.md`;
- new `plans/implementation/i2pcontrol-proposal-170/058-containment-ledger.toml`;
- `plans/registry.md` and current planning indexes only for lifecycle transition;
- `plans/closure/i2pcontrol-proposal-170/058-closure.md` when closing;
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` only if repository evidence changes milestone dependency wording, not scope.

No changes are authorized under:

- `emissary-core/**`;
- `emissary-cli/src/**`;
- `emissary-cli/tests/**`;
- `Cargo.toml`, `Cargo.lock`, package manifests;
- `.github/**`;
- runtime/configuration/release files.

If audit work uncovers a production defect, record it as a finding and stop that branch of analysis. Do not fix it inside M058.

## 4. Required invariants

1. Proposal 170 wire behavior is unchanged.
2. RouterInfo remains 37 available / 1 neutral / 5 unavailable.
3. M051 news/ban limitation remains blocked and is not revisited.
4. Unsupported tunnel types remain unsupported and resource-free.
5. No new production source, observer, callback, event, metric, task, state, dependency, or feature is introduced.
6. Historical M037/M045–M057 closure evidence is not rewritten to make the present diff look cleaner.
7. Every upstream/fork comparison is read-only; no upstream issue, PR, review, branch, tag, submission, or maintainer contact is authorized.
8. File-count reduction is not itself a correctness criterion; truthful owner-local observation outranks cosmetic reversion.

## 5. Ledger model

Create `058-containment-ledger.toml` with at least:

```toml
version = 1
fork_baseline = "adb2f52543764b267b2bcb282d093111001ae4b2"
upstream_baseline = "9b43484a21d5a1291c4881cdae62a36c527f8c0f"

[[path]]
name = "emissary-core/src/..."
area = "core|cli|build"
classification = "required-composition|required-owner-seam|candidate-revert|candidate-consolidate|unrelated-or-accidental|uncertain"
owner = "..."
consumer = "..."
policy_leak = false
sensitive_path = true
next_milestone = "M059|M060|none|blocked"
rationale = "..."
required_regressions = ["..."]
```

Equivalent fields may be added, but the required information must remain machine-readable.

Classification definitions:

- `required-composition`: minimal feature/config/start-stop wiring required to expose the control service without owning policy.
- `required-owner-seam`: neutral bounded observation/callback at the only canonical owner.
- `candidate-revert`: behavior can be preserved with the upstream implementation or an already-existing higher-level seam.
- `candidate-consolidate`: fact is required, but several touched paths may be replaced by a smaller owner-level seam.
- `unrelated-or-accidental`: fork delta is not needed by the accepted Proposal 170 workstream and should be separately dispositioned; M058 must not silently delete it.
- `uncertain`: evidence is insufficient; path cannot enter a modification plan until the uncertainty is resolved.

`policy_leak = true` means Proposal 170/I2PControl policy, administrative persistence semantics, wire terminology, selector semantics, or control-plane aggregation exists in an original module and should be targeted by M059 unless moving it would create duplicate authority.

## 6. Seed inventory that must be reconciled

The ledger must reconcile the complete compare output, including at minimum these production groups visible at planning time.

### 6.1 Build/package/configuration

- root `Cargo.toml` / `Cargo.lock` where the feature introduced dependency changes;
- `emissary-cli/Cargo.toml`;
- `emissary-cli/src/config.rs`.

These are not automatically contamination. Record exactly which dependency/feature/config addition is required and whether any dependency can be removed after containment.

### 6.2 Original CLI/runtime

- `emissary-cli/src/address_book.rs`;
- `emissary-cli/src/lib.rs`;
- `emissary-cli/src/logger.rs`;
- `emissary-cli/src/main.rs`;
- `emissary-cli/src/proxy/http/error.rs`;
- `emissary-cli/src/proxy/http/mod.rs`;
- `emissary-cli/src/proxy/http/request.rs`;
- `emissary-cli/src/proxy/socks.rs`;
- `emissary-cli/src/tunnel/client.rs`;
- `emissary-cli/src/tunnel/server.rs`.

For each path, distinguish ordinary runtime ownership from Proposal 170 policy. Pay particular attention to AddressBook administrative/persistence logic, service observation hooks in proxy/tunnel paths, logger fanout/aggregation, and composition logic.

### 6.3 Core generic plumbing

- `emissary-core/src/error/mod.rs`;
- `emissary-core/src/events.rs`;
- `emissary-core/src/inspection.rs`;
- `emissary-core/src/lib.rs`;
- `emissary-core/src/primitives/router_identity.rs`;
- `emissary-core/src/router/context.rs`;
- `emissary-core/src/router/mod.rs`;
- `emissary-core/src/runtime/mod.rs`;
- `emissary-core/src/subsystem/mod.rs`.

Identify whether each change is a neutral inspection/export seam, a required DTO conversion, obsolete scaffolding, or unnecessary propagation caused by a constructor/signature choice.

### 6.4 SAM/I2CP observation

- `emissary-core/src/i2cp/socket.rs`;
- `emissary-core/src/sam/mod.rs`;
- `emissary-core/src/sam/parser.rs`;
- `emissary-core/src/sam/pending/connection.rs`;
- `emissary-core/src/sam/protocol/streaming/listener.rs`;
- `emissary-core/src/sam/protocol/streaming/mod.rs`;
- `emissary-core/src/sam/session.rs`;
- `emissary-core/src/sam/socket.rs`.

Determine which files must see lifecycle events and which only changed because observation state/types were threaded too deeply. Preserve the accepted M037/M024 complete/incomplete semantics.

### 6.5 Transport observation

- `emissary-core/src/transport/mod.rs`;
- `emissary-core/src/transport/ntcp2/mod.rs`;
- `emissary-core/src/transport/ntcp2/session/active.rs`;
- `emissary-core/src/transport/ntcp2/session/mod.rs`;
- `emissary-core/src/transport/ssu2/message/data.rs`;
- `emissary-core/src/transport/ssu2/mod.rs`;
- `emissary-core/src/transport/ssu2/peer_test/mod.rs`;
- `emissary-core/src/transport/ssu2/relay/mod.rs`;
- `emissary-core/src/transport/ssu2/session/active/mod.rs`;
- `emissary-core/src/transport/ssu2/session/pending/inbound.rs`;
- `emissary-core/src/transport/ssu2/session/terminating.rs`;
- `emissary-core/src/transport/ssu2/socket.rs`.

For each retained fact, name the exact RouterInfo/inspection consumer and explain whether the observation could be maintained at `TransportManager`/transport owner without touching packet/session-specific paths. Do not decide by intuition; inspect writers and lifecycle transitions.

### 6.6 Tunnel observation

- `emissary-core/src/tunnel/mod.rs`;
- `emissary-core/src/tunnel/pool/mod.rs`;
- `emissary-core/src/tunnel/transit/mod.rs`.

Identify exact queue/pool/participating facts and whether existing owner-level snapshots can replace lower-level instrumentation.

### 6.7 Other changed production/examples

Any additional non-`i2pcontrol` production path returned by the pinned compare must appear in the ledger, including examples if they changed solely because a public constructor/API signature changed. Constructor churn in examples/tests is evidence that a core seam may be too invasive and must be called out explicitly.

## 7. Ordered work packages

### WP1 — Verify baselines and compare completeness

Confirm:

- fork baseline equals the intended M057 merged head or record the newer head and stop if production changed since it;
- upstream baseline is still the merge base used by the fork for this audit;
- compare output is complete, including renamed/deleted files.

If upstream advanced after `9b43484a`, do **not** silently rebase the comparison. Record the newer upstream head separately. This corrective pass is about the fork delta from the audited merge-base lineage; upstream advancement is a separate rebase concern.

### WP2 — Build exact changed-path set

Generate the non-`i2pcontrol` changed production path list from version-control evidence. Exclude planning/docs/tests only after recording them as non-production categories; do not hand-curate the list from memory.

Record total counts by area and preserve the exact command/output summary in closure.

### WP3 — Trace each path to owner and consumer

For every path:

1. inspect its diff against upstream;
2. identify new/changed symbols;
3. find all current consumers;
4. determine whether the symbol is used when I2PControl is disabled;
5. identify the canonical runtime owner;
6. identify whether policy or only facts cross the boundary;
7. assign classification and required regressions.

A path cannot be `required-owner-seam` merely because current code uses it. The rationale must explain why an already-modified higher owner cannot provide an equivalent truthful fact.

### WP4 — Identify M059 original-CLI budget

Produce the exact set of original CLI/runtime paths M059 may modify. Mark each as candidate-revert, candidate-policy-extraction, or required-composition/adapter.

If a path is `uncertain`, M059 may not touch it.

### WP5 — Identify M060 core budget

Produce the exact set of core paths M060 may inspect/modify, subdivided by:

- generic inspection plumbing;
- SAM/I2CP owner hooks;
- transport owner/protocol hooks;
- tunnel owner hooks.

For each candidate-consolidation group, state the proposed higher-level target and the semantic proof required before deleting lower-level hooks.

### WP6 — Closure and registry transition

Create `plans/closure/i2pcontrol-proposal-170/058-closure.md` containing:

- exact fork/upstream baselines;
- total changed-path counts and categories;
- complete ledger validation result;
- unresolved `uncertain` paths, if any;
- M059 exact path budget;
- M060 provisional path budget;
- attestation that production files did not change;
- internal-only/no-upstream attestation.

Only after accepted M058 closure may the registry advance M059 to `ready`.

## 8. Failure, cancellation, restart, and contention semantics

M058 has no runtime lifecycle because it changes no production code.

Audit failure semantics:

- an unreadable/unresolvable diff path is `uncertain`, never guessed;
- a path with mixed unrelated and Proposal-170 changes is explicitly split by symbol/hunk rationale in the ledger;
- a production defect found during tracing blocks the relevant later milestone until separately planned;
- incomplete inventory blocks closure.

Restart semantics: rerunning M058 against the same baselines must produce the same path set/classification inputs. If the fork head changes with production commits while M058 is active, freeze the prior head and either restart the inventory at the new head or record the delta before closure; do not silently mix baselines.

No shared locks, channels, or runtime contention are introduced.

## 9. Compatibility, migration, security, and operations

Compatibility/migration: none; no code changes.

Security value: this milestone creates the audit artifact required to reason about minimum trusted-code delta. The ledger must never include secrets or runtime private material; it records paths/symbol purposes only.

Operational effects: none.

Upstream access is read-only comparison only. No upstream mutation, contribution preparation, or review request is permitted.

## 10. Verification

Required local/version-control verification:

```bash
git status --short
git merge-base master eepnet/master
git diff --name-status 9b43484a21d5a1291c4881cdae62a36c527f8c0f..adb2f52543764b267b2bcb282d093111001ae4b2
git diff --stat 9b43484a21d5a1291c4881cdae62a36c527f8c0f..adb2f52543764b267b2bcb282d093111001ae4b2
git diff --check
```

Equivalent GitHub compare evidence is acceptable when operating through the connector.

Required targeted searches include:

```bash
rg -n "Proposal 170|I2PControl|JsonRpc|RouterInfo|ClientServicesInfo|TunnelManager" \
  emissary-core emissary-cli/src \
  -g '!emissary-cli/src/i2pcontrol/**'

rg -n "inspection|observer|observation|snapshot|hook" \
  emissary-core emissary-cli/src \
  -g '!emissary-cli/src/i2pcontrol/**'
```

Search hits are reviewed semantically; generic terms are not defects by themselves.

Validate the TOML ledger parses and contains exactly one entry for every changed non-`i2pcontrol` production path.

Do **not** run a broad Rust matrix solely for this audit-only milestone. No hosted CI is required.

## 11. Documentation and static evidence

The M058 ledger becomes the controlling input for M059/M060 path budgets. It does not supersede historical M037 or RouterInfo source manifests; it records the current post-M057 fork delta.

Do not add a new general lint framework. A small parsing/check script/test is unnecessary unless the repository already has an appropriate planning validation path. Closure may validate the ledger with local shell/Python/TOML tooling.

## 12. Acceptance criteria

M058 may close only when all are true:

1. Exact fork and upstream comparison baselines are recorded.
2. Every changed production path outside `emissary-cli/src/i2pcontrol/**` is present exactly once in `058-containment-ledger.toml`.
3. Every ledger entry has classification, owner, consumer/purpose, policy-leak flag, next milestone/disposition, rationale, and required regression evidence.
4. No path is called required solely because the current implementation uses it; canonical-owner necessity is explained.
5. All original CLI/runtime candidate paths are assigned to the exact M059 path budget or explicitly retained/blocked.
6. All core candidate paths are assigned to the provisional M060 path budget or explicitly retained/blocked.
7. Any `uncertain` entry names the missing evidence and blocks modification of that path.
8. The accepted 37/1/5 RouterInfo disposition is unchanged.
9. M051 and unsupported tunnel types remain out of scope.
10. No production/source/test behavior file changed under M058.
11. `git diff --check` passes for planning artifacts.
12. Closure records complete compare/count evidence and no-production attestation.
13. Closure includes the required internal-only/no-upstream attestation.
14. M059 is not marked ready until M058 closure accepts the ledger.

## 13. Stop conditions

Stop rather than broaden M058 if:

- the fork baseline contains production commits after `adb2f525` that are not part of the reviewed Proposal 170 state;
- the upstream merge base differs materially and would require an upstream rebase/reconciliation plan;
- a current supported behavior is discovered to depend on fabricated/unowned state;
- classification requires changing production code to discover what it does;
- a path mixes unrelated work that cannot be safely separated from Proposal 170 ownership;
- a Proposal 170 external revision changed materially;
- any upstream write/review/submission action is proposed.

## 14. Expected closure disposition

Successful M058 closure should state:

- audit complete;
- production unchanged;
- current non-`i2pcontrol` delta fully classified;
- M059 exact original-CLI budget ready;
- M060 remains planned behind M059;
- overall Proposal 170 support remains partial at 37/1/5 RouterInfo source disposition;
- no upstream interaction occurred.