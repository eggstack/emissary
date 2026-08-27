# M094 Closure — Post-M093 Planning-State Reconciliation

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/094-post-m093-planning-state-reconciliation.md`.

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Predecessors:

- M092 rollback/containment corrective: `plans/closure/i2pcontrol-proposal-170/092-closure.md`;
- M093 corrected-head security reclosure: `plans/closure/i2pcontrol-proposal-170/093-closure.md`;
- `plans/003-planning-process.md` §§2.4–2.5, §§6–8, and §11.

Planning baseline: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.
Pre-closure planning head: `8a4e8158c020fa8e3bd4b2c03a5bc430627e2d1f` (the M094 registration head).
Closure head: the final internal commit containing this record; its exact identifier is reported
with the repository handoff.
Review date: 2026-08-27.

## 1. Disposition

M094 is closed. It reconciled planning and evidence bookkeeping only. Production/runtime tunnel
security remained current-head closed by M093 throughout this work. No production, dependency,
lockfile, runtime, core, router, startup, frontend, API, security-policy, or persisted-state
behavior changed.

The active planning line now has no ready tunnel-security handoff. M093 remains the current
production/security reclosure authority; M094 does not perform or claim a new security review.

## 2. Exact changed-path matrix

The M094 authorized path set was Section 4 of the implementation plan. The final changed paths
were:

| Path | Change | Authority |
|---|---|---|
| `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md` | Changed stale `Status: ready` to closed and linked the accepted M092 closure | M094 §6A |
| `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md` | Recast readiness/current-handoff text as historical sequencing evidence | M094 §6B |
| `plans/implementation/i2pcontrol-proposal-170/094-post-m093-planning-state-reconciliation.md` | Closed the plan after reconciliation | M094 plan |
| `plans/closure/i2pcontrol-proposal-170/092-closure.md` | Pinned the M092 implementation head | M094 §6A |
| `plans/closure/i2pcontrol-proposal-170/093-closure.md` | Distinguished reviewed production head from closure/planning commit | M094 §6B |
| `plans/closure/i2pcontrol-proposal-170/094-closure.md` | Added this closure record | M094 §11 |
| `plans/implementation/i2pcontrol-proposal-170/README.md` | Converged M094 to closed/no-ready-handoff wording | M094 §6C |
| `plans/registry.md` | Converged the active registry to closed/no-ready-handoff wording | M094 §6C |
| `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | Converged roadmap status, sequence, milestone, and final-state wording | M094 §6C |
| `emissary-cli/tests/m062_dependency_containment.rs` | Contains only the two exact M094 planning-path entries registered before closure | M094 §6D |

No path outside Section 4 was changed.

## 3. Status and SHA-role evidence

M092 now has:

- plan status: `closed (closure at plans/closure/i2pcontrol-proposal-170/092-closure.md)`;
- closure implementation head: `8860407a79347ce925603821cdb231e47a680623`.

M093 remains closed and its former readiness text now states that dependency-ready/current-handoff
sequencing was historical. Its closure record now separates:

- reviewed production head: `8860407a79347ce925603821cdb231e47a680623`;
- M093 closure/planning commit: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.

The first SHA is the corrected production tree audited by M093. The second is the accepted
M093 closure/planning record; neither is described as both roles.

## 4. Registry, roadmap, and README convergence

The three active planning surfaces agree that:

- M092 and M093 are closed;
- M094 is closed and documentation/evidence-only;
- production/security is current-head closed after M093;
- no tunnel-security handoff is currently ready;
- no future tunnel-security implementation plan was unblocked or registered by M094.

M091 remains corrective-pass-required/superseded by M092. M088 remains the accepted lower-layer /
pre-accept residual. M090 remains retained valid production work.

The unrelated M051 RouterInfo blocker remains unchanged, as do the accepted RouterInfo 37/1/5
disposition and the separately partial Proposal 170 source/truthfulness state.

## 5. M062 planning-allowlist evidence

The diff from the M094 planning baseline to the pre-closure registration head for
`emissary-cli/tests/m062_dependency_containment.rs` is exactly:

```text
+            | "plans/implementation/i2pcontrol-proposal-170/094-post-m093-planning-state-reconciliation.md"
+            | "plans/closure/i2pcontrol-proposal-170/094-closure.md"
```

These are exact planning paths only. No production glob, core path, dependency rule, lockfile
assertion, feature ownership, or tunnel-runtime allowance was added or changed.

## 6. Verification outcomes

| Command | Outcome |
|---|---|
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | pass; 19 tests |
| `git diff --check` | pass |
| `git diff --name-only 4da022ec874e9915e2d38fe63c609bff537ee8ff..HEAD` | all paths are in the M094 Section 4 matrix |
| `git diff -- emissary-cli/tests/m062_dependency_containment.rs` | exactly the two M094 planning-path additions; no semantic changes |
| `git status --short` | clean at closure |

The M062 test continues to enforce the pre-M091 semantic containment assertions. No full runtime
or core suite was required because M094 made no production change.

## 7. Production/dependency containment proof

The final M094 diff contains no production or dependency path. In particular, it contains no
`emissary-cli/src/**`, `emissary-core/**`, `Cargo.toml`, `Cargo.lock`, `vendor/**`, runtime,
router, startup, frontend, API, or security-policy change. The only path outside `plans/**` is
`emissary-cli/tests/m062_dependency_containment.rs`, and its diff is limited to the two exact
planning-path strings in §5.

No production behavior, Proposal 170 contract, dependency provenance, M090 behavior, M088
residual-risk disposition, or accepted security conclusion was changed.

## 8. Unresolved findings and severity

- High severity: none.
- Medium severity: none.
- Low severity: none introduced by M094.
- Future planning candidates: none unblocked by M094. The pre-existing M051 blocker and the
  accepted M088 lower-layer and Streamr availability limitations remain as documented.

## 9. Technical dispositions preserved

M094 explicitly leaves unchanged:

- M088's accepted lower-layer/pre-accept resource/timing residual;
- M090's retained resolver-free loopback and IRC half-close production correction;
- M091's blocked technical history and supersession by M092;
- M092's rollback, dependency restoration, and containment authority;
- M093's independent corrected-head security/anonymity reclosure and finding disposition.

M094 introduces no new production security review and no new successor implementation handoff.

## 10. Internal-only attestation

All writes were confined to the internal `eggstack/emissary` repository and the authorized paths
listed in the M094 plan. External specifications, source, issues, commits, and pull requests
were read-only evidence only. No upstream issue, pull request, review, submission, merge request,
contribution artifact, maintainer contact, or external repository write was opened, drafted,
requested, or pushed.

**Disposition: closed.**
