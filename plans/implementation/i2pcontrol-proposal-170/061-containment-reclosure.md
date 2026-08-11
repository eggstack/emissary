# M061 — Independent Containment Reclosure and Static-Guard Refresh

Status: planned; hard-blocked on M060 closure

Planning baseline: to be pinned by accepted M060 closure

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`

Hard dependencies:

- M058 accepted complete delta ledger;
- M059 accepted original CLI/runtime containment closure;
- M060 accepted core observation containment closure;
- final non-`i2pcontrol` production path set frozen by M060.

Milestone class: invariant / independent closure hardening

Applicable authority:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- accepted M058–M060 records;
- historical M037 containment evidence.

## 1. Bounded objective

Independently verify that the post-M060 fork has the minimum justified non-`i2pcontrol` production delta needed for the already-supported Proposal 170/I2PControl surface, then install a **current** machine-readable containment manifest/static guard so the boundary does not drift again.

M061 is production-free. It does not perform another refactor pass and does not reopen source completeness. If reclosure discovers a material containment or behavior defect, M061 must fail and create a separately scoped corrective plan rather than fixing production code inside closure.

## 2. Required outputs

M061 must produce:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` — current final machine-readable authority;
- focused static guard(s), preferably a new `emissary-cli/tests/m061_containment.rs` or a minimal clearly named successor guard rather than rewriting M037 historical meaning;
- `plans/closure/i2pcontrol-proposal-170/061-closure.md` with independent requirement-to-evidence matrix;
- lifecycle updates to roadmap/registry/index after closure.

No production Rust is authorized.

## 3. Required invariants

1. Proposal 170 operational behavior is unchanged from the accepted pre-containment baseline except purely internal placement/refactoring.
2. RouterInfo remains 43 total / 37 available / 1 protocol-permitted neutral / 5 unavailable.
3. M051 remains blocked with accepted news/ban limitation.
4. Unsupported tunnel types remain unsupported/resource-free.
5. No Proposal 170 wire/admin/support policy exists in `emissary-core`.
6. Original CLI/runtime modules contain only the exact minimal feature/config/composition/runtime-owner seams accepted by M059.
7. Every retained core path contains only the exact neutral owner/inspection seam accepted by M060.
8. No secret/live/mutable control object crosses the inspection boundary.
9. Default/no-feature Emissary behavior remains regression-equivalent.
10. No new background task, probe, sampler, event framework, persistent metric store, dependency, CI/release machinery, or unsupported data plane exists because of the containment sequence.
11. Historical closure records remain historical; M061 does not rewrite M037/M056 evidence.
12. No upstream issue, PR, review, submission, merge/adoption request, or maintainer contact is authorized.

## 4. Scope and authorized paths

Production changes: **none**.

Authorized files:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`;
- focused containment test under `emissary-cli/tests/`;
- `docs/i2pcontrol/**` only for final ownership/path-boundary documentation if stale;
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` lifecycle/status;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/closure/i2pcontrol-proposal-170/061-closure.md`;
- M061 plan status line.

Explicitly forbidden:

- `emissary-core/**` production changes;
- `emissary-cli/src/**` production changes;
- package dependency/feature changes;
- `.github/**`;
- release/runtime/configuration changes.

If a production edit seems needed, closure fails and a new corrective implementation plan is required.

## 5. Final containment manifest contract

Create `061-containment-boundary.toml` as the current authority. It should include at least:

```toml
version = 1
fork_baseline = "<M060 accepted head>"
upstream_baseline = "9b43484a21d5a1291c4881cdae62a36c527f8c0f"
policy_root = "emissary-cli/src/i2pcontrol/"

[allowed]
composition = ["..."]
original_runtime_adapters = ["..."]
core_inspection = ["..."]
core_owner_hooks = ["..."]
build_feature_paths = ["..."]

[prohibited]
production_prefixes = ["..."]
forbidden_terms_outside_policy_root = ["..."]
```

The manifest must list **exact paths**, not broad `emissary-core/src/transport/` or `emissary-core/src/tunnel/` prefixes, for retained high-sensitivity hooks. Directory-prefix allowance is acceptable for `emissary-cli/src/i2pcontrol/**` itself and planning/docs/tests, not for deep core production areas.

For every allowed original/core path, record or cross-reference:

- owner;
- purpose;
- supported consumer;
- why upstream-equivalent code is insufficient;
- security sensitivity class;
- whether the seam is feature/composition, snapshot, or passive lifecycle hook.

The final manifest supersedes **only as current authority** the historical M037 boundary. Do not edit M037 to erase its historical scope.

## 6. Static guard requirements

The current guard must fail if a future change:

- adds a new allowed core/original-runtime path without updating the manifest;
- places `JsonRpc`, Proposal 170 selector strings, TunnelManager wire policy, or administrative I2PControl persistence types in core;
- causes original proxy/tunnel/AddressBook runtime modules to import control-plane handler/request DTOs;
- exposes forbidden live/secret types through the inspection/event declarations checked by the manifest;
- broadens allowed high-sensitivity prefixes instead of naming exact paths;
- changes unsupported tunnel backends to allocate/listen/spawn runtime resources.

The guard does **not** need to introspect Git history during normal `cargo test`. It enforces current source/manifest invariants. Version-control diff checks remain closure evidence.

Do not build a generalized policy engine, proc macro, build script, or new dependency for this test. Simple source/manifest parsing is sufficient.

## 7. Ordered work packages

### WP1 — Freeze independent review baseline

Pin:

- accepted M060 implementation/closure head;
- upstream merge-base reference;
- M058 ledger;
- M059/M060 retained-path tables.

Verify no production commit landed after the M060 head before starting closure. If it did, M061 must rebase its review baseline or stop.

### WP2 — Recompute upstream/fork production diff

Independently recompute the changed production path set. Do not copy the M060 result table as proof.

Compare the recomputed set with:

- M058 original ledger;
- M059 closure original-runtime final set;
- M060 closure core final set.

Every difference must be explained.

### WP3 — Requirement-to-path audit

For every retained non-`i2pcontrol` path, independently verify:

1. current changed symbols/hunks;
2. canonical owner;
3. accepted control-plane consumer;
4. absence of policy leakage;
5. necessity of the seam;
6. security sensitivity;
7. focused regression coverage.

A retained deep core path without a clear necessity explanation is a closure defect.

### WP4 — Build current manifest

Encode the accepted final path set exactly in `061-containment-boundary.toml`.

No path may be added simply to make the guard pass. The manifest follows accepted evidence; it does not legitimize unexplained drift.

### WP5 — Add focused current containment guard

Add the minimal test described above. Prefer pure std plus dependencies already present in `emissary-cli` test/dev dependencies.

Run it against the final source tree and demonstrate a local negative fixture/change if practical without committing destructive test mutations; at minimum inspect that each assertion would fail on a representative prohibited string/path.

### WP6 — Integrated regression verification

Run the focused behavior tests from M059/M060 together at the same frozen head, including no-feature/default checks and the current containment guard.

Do not expand into unrelated workspace tests unless a changed shared API requires them.

### WP7 — Documentation reconciliation

Update ownership/security/inspection docs to reference M061 current boundary if needed.

Historical documents should continue to state their original accepted scope.

### WP8 — Closure and registry finalization

Create `061-closure.md` with:

- exact baseline/head;
- recomputed compare evidence;
- requirement-to-path matrix;
- retained path rationale;
- static guard results;
- integrated behavior verification;
- default/no-feature proof;
- unresolved findings/severity;
- compatibility/security/performance review;
- explicit no-production-change attestation for M061;
- internal-only/no-upstream attestation.

After accepted closure, registry should return to no dependency-ready containment handoff. M051 remains the unrelated blocked source-completion item.

## 8. Failure, cancellation, restart, and contention semantics

M061 introduces no runtime state or production behavior.

Closure semantics:

- any unexplained production path = fail;
- any supported behavior regression = fail;
- any policy leakage outside the accepted owner = fail;
- any live/secret boundary violation = fail;
- any production edit required to repair the above = `corrective pass required`, not an in-place closure fix.

If review is interrupted/restarted, rerun the compare against the same frozen head. If the repository head changed, confirm whether changes are planning/test-only; otherwise freeze a new review head.

No runtime contention effects exist.

## 9. Compatibility, migration, security, and operations

No migration or compatibility change is authorized.

Security review must explicitly assess whether the post-M060 diff is smaller/equally bounded relative to the M058 baseline and whether any retained deep protocol hook increases attack surface or mutable authority. “Read-only” alone is insufficient if a hook changes timing/blocking behavior; verify passivity.

Operationally, I2PControl remains opt-in/feature-gated as before. No default listener/task/state is added by this roadmap.

## 10. Verification commands

Version-control evidence:

```bash
git diff --check
git diff --name-only 9b43484a21d5a1291c4881cdae62a36c527f8c0f..<M060_HEAD>
git diff --name-only <M061_BASE>..HEAD
```

Current containment/static checks:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m037_containment
```

Integrated supported behavior:

```bash
cargo check -p emissary-core
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test client_services_live
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m033_tunnel_lifecycle
```

Add focused core SAM/transport/tunnel tests named by M060 closure. Do not blindly run every workspace target if unchanged/unrelated.

Clippy should be run for changed/tested packages if M059/M060 did not already close on the exact same head:

```bash
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

No hosted CI, coverage, fuzz, soak, release, or generated evidence bundle is required.

## 11. Acceptance criteria

M061 may close only when all are true:

1. M058, M059, and M060 are accepted closed.
2. M061 independently recomputed the final upstream/fork production changed-path set.
3. Every retained non-`i2pcontrol` production path has exact owner, consumer, necessity, sensitivity, and regression evidence.
4. No unexplained or `uncertain` production path remains.
5. `061-containment-boundary.toml` exactly represents the accepted current boundary and names high-sensitivity core paths individually.
6. Current static guard passes and would reject representative policy leakage/new core path/live-secret boundary violations.
7. Core contains no Proposal 170 selector/wire/admin/support policy.
8. Original CLI/runtime modules contain only accepted feature/config/composition/neutral owner adapters.
9. Unsupported tunnel backends remain resource-free.
10. Default/no-feature compilation and focused behavior regressions pass, except explicitly recorded unrelated pre-existing failures that do not compromise closure.
11. RouterInfo remains 37/1/5 and M051 remains blocked; no unavailable field was promoted.
12. M061 itself changes no production Rust/package/runtime/workflow/release file.
13. No high or medium containment/security/behavior finding remains open.
14. Documentation identifies M061 as the current containment authority without rewriting historical M037 evidence.
15. Registry/roadmap/index show containment sequence closed and no dependency-ready successor.
16. Closure includes the required internal-only/no-upstream attestation.

## 12. Stop conditions

M061 must disposition `corrective pass required` rather than close if:

- a retained path cannot be independently justified;
- a core path contains control-plane policy rather than neutral facts;
- default/no-feature behavior changed;
- a supported method/source/lifecycle regressed;
- a new production edit is required;
- final path set exceeds M060 closure without a planning-only explanation;
- a new unavailable-source implementation is proposed;
- external Proposal 170 authority changed materially;
- any upstream write/review/submission activity is proposed.

## 13. Expected final disposition

Successful M061 closure should state:

- Proposal 170 containment corrective sequence closed;
- supported API remains operational with the same accepted partial source disposition;
- the remaining non-`i2pcontrol` fork delta is minimal and individually justified at the current baseline;
- `061-containment-boundary.toml` plus the focused static guard is the current containment authority;
- M051 remains independently blocked;
- no production changes occurred during M061;
- no upstream interaction occurred.