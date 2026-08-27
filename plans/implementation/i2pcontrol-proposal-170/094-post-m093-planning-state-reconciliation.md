# M094 — Post-M093 Planning-State Reconciliation

Status: closed (closure at `plans/closure/i2pcontrol-proposal-170/094-closure.md`)

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Corrective predecessors:

- M092 authorization/dependency/containment corrective and `plans/closure/i2pcontrol-proposal-170/092-closure.md`;
- M093 independent corrected-head tunnel-security reclosure and `plans/closure/i2pcontrol-proposal-170/093-closure.md`;
- `plans/003-planning-process.md` §§2.4–2.5, §§6–8, and §11.

Planning baseline: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.

M092 implementation head: `8860407a79347ce925603821cdb231e47a680623`.

M093 closure/current planning head at the planning baseline: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.

Known valid pre-M091 production/dependency baseline: `6d631d4423c7faa761b47a84e07436bbaf5d9ad4`.

Classification: documentation/evidence reconciliation corrective; no-production-change milestone.

## 1. Objective

Reconcile the remaining planning-state inconsistencies left after M092 and M093 without changing production behavior, dependency state, security policy, Proposal 170 semantics, or the accepted residual-risk disposition.

M092 successfully removed the unauthorized M091 Yosemite/core/dependency expansion and restored the pre-M091 containment boundary. M093 independently reclosed the corrected production head. The remaining defects are administrative/documentary:

1. the M092 implementation plan still says `Status: ready` despite its accepted closure;
2. parts of `plans/registry.md` still describe M092/containment restoration as a current handoff or pending action even though M092 and M093 are closed;
3. M093's plan header is closed, but its readiness section still describes M093 as the dependency-ready/current handoff;
4. M092's closure does not directly pin implementation commit `8860407a79347ce925603821cdb231e47a680623`;
5. M093 closure wording must distinguish the reviewed production head `8860407a79347ce925603821cdb231e47a680623` from the M093 closure/planning commit `4da022ec874e9915e2d38fe63c609bff537ee8ff` rather than describing one SHA as both;
6. roadmap/implementation-index wording must consistently state that the security line is production-current-head closed after M093 while M094 is only a planning-record cleanup.

M094 exists solely to make those records internally coherent. It MUST NOT reopen M091, revise M088's accepted lower-layer residual, or alter the security conclusions of M090/M092/M093.

## 2. Why prior closure missed the defect

M092 and M093 correctly prioritized the production/dependency rollback and independent security re-audit. Their verification established the important properties: M091's vendor/core delta is absent, M090 remains intact, containment semantics are restored, and the twelve Proposal 170 tunnel backends have no new high/medium production defect inside the approved boundary.

The remaining problem is cross-document state convergence. Individual documents were updated at different points in the handoff lifecycle, leaving stale `ready`/`current handoff` language and ambiguous SHA terminology after the later closure commit landed.

M094 adds an explicit post-closure consistency check so a closed handoff cannot remain represented elsewhere as ready/active/pending.

## 3. Required end state

After M094 closes:

1. `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md` says `Status: closed` and points to `plans/closure/i2pcontrol-proposal-170/092-closure.md`.
2. `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md` remains `Status: closed`, and its readiness section is rewritten as historical readiness evidence rather than claiming it is the active/current handoff.
3. `plans/closure/i2pcontrol-proposal-170/092-closure.md` directly records `8860407a79347ce925603821cdb231e47a680623` as the M092 implementation head.
4. `plans/closure/i2pcontrol-proposal-170/093-closure.md` explicitly distinguishes:
   - reviewed production head: `8860407a79347ce925603821cdb231e47a680623`;
   - M093 closure/planning commit: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.
5. `plans/registry.md` contains no stale row or section naming M092 or M093 as the current/ready handoff. M094 is the sole ready handoff while this reconciliation is open.
6. The tunnel-security roadmap and implementation README state that M092/M093 production/security work is closed and M094 is documentation-only cleanup.
7. The M062 cumulative planning allowlist contains exact M094 plan/closure entries only; no production glob, core path, dependency, lockfile, vendor path, or feature ownership is added.
8. No production, dependency, lockfile, runtime, core, router, startup, frontend, API, tunnel behavior, security policy, or persisted state changes.
9. M091 remains `blocked / superseded by M092` with its closure retained only as historical technical evidence.
10. M088 remains the accepted lower-layer/pre-accept resource/timing residual.
11. M090 remains valid retained production work.
12. M093 remains the current security/anonymity reclosure authority for the corrected production head.

## 4. Authorized path boundary

M094 may modify only planning/evidence bookkeeping:

- `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md`;
- `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md`;
- this M094 plan;
- `plans/closure/i2pcontrol-proposal-170/092-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/093-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/094-closure.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `emissary-cli/tests/m062_dependency_containment.rs` only for exact M094 planning/closure path allowlist entries.

No other path is authorized.

The `m062_dependency_containment.rs` allowance is bookkeeping only. It MUST NOT change any semantic production/dependency assertion or authorize a new production path.

## 5. Explicit non-goals

M094 MUST NOT:

- modify any file under `emissary-cli/src/**`;
- modify any file under `emissary-core/**`;
- modify `Cargo.toml`, `Cargo.lock`, or any package manifest;
- add, remove, patch, vendor, fork, or change Yosemite or any other dependency;
- modify M090 runtime behavior;
- reinstate any M091 pre-accept concurrency implementation;
- change M088's residual-risk disposition;
- add lower-layer admission, rate limiting, Sybil resistance, jitter, padding, or new DoS controls;
- change HTTP, IRC, Streamr, server, client, SOCKS, CONNECT, SAM, or tunnel semantics;
- add Proposal 170 fields, methods, aliases, statuses, or tunnel types;
- change RouterInfo 37/1/5 or M051;
- create hosted CI/fuzz/soak/release machinery;
- prepare or request upstream review, merge, submission, issue, PR, or maintainer contact.

## 6. Ordered work packages

### A. Reconcile M092 state

Change the M092 plan status from `ready` to `closed` and add its closure pointer. Do not rewrite the plan body or its original authorization analysis.

Pin `8860407a79347ce925603821cdb231e47a680623` directly in the M092 closure as the implementation head.

### B. Reconcile M093 state

Keep the M093 plan status closed. Rewrite only stale readiness/current-handoff wording so it reads as historical sequencing evidence.

Correct the M093 closure terminology so `8860407a79347ce925603821cdb231e47a680623` is the reviewed production head and `4da022ec874e9915e2d38fe63c609bff537ee8ff` is the closure/planning commit. Do not alter the underlying audit findings or verification outcomes.

### C. Reconcile registry, roadmap, and handoff index

While M094 is open, make it the sole `ready` tunnel-security handoff. Remove stale wording that names M092/M093 as current, pending, or dependency-ready.

The documents must continue to state that production/security is already closed by M093; M094 is administrative cleanup and does not reopen production security review.

### D. Register exact planning paths

Add only:

- `plans/implementation/i2pcontrol-proposal-170/094-post-m093-planning-state-reconciliation.md`;
- `plans/closure/i2pcontrol-proposal-170/094-closure.md`

to the M062 exact planning-path allowlist.

Do not touch the production/dependency path predicates or lockfile assertions.

### E. Verify cross-document convergence

Search the active planning documents for stale combinations such as:

- `M092` + `Status: ready`;
- `M092` + `current handoff`;
- `M093` + `active implementation handoff`;
- `M093` + `dependency-ready next`;
- `8860407` described as both reviewed and closure head.

Historical narrative may state that a plan *was* ready before execution; active-state wording must not.

## 7. Required regression evidence

M094 closure must prove:

1. M092 plan status is closed and linked to its closure.
2. M093 plan contains no live/active readiness claim.
3. M092 closure pins implementation SHA `8860407a79347ce925603821cdb231e47a680623`.
4. M093 closure differentiates reviewed production SHA from closure/planning SHA.
5. Registry lists M094 as the only ready tunnel-security handoff while M094 is open.
6. Roadmap and README describe M094 as documentation-only reconciliation.
7. M091 remains blocked/superseded and is not reauthorized.
8. M088 remains the accepted lower-layer residual.
9. M062's semantic containment assertions are byte-identical to the M093 baseline except for the two exact M094 planning-path strings.
10. No production/dependency file differs because of M094.

## 8. Verification

At minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Also verify the M094 changed-path set exactly. A suitable review is:

```text
git diff --name-only 4da022ec874e9915e2d38fe63c609bff537ee8ff..HEAD
```

Every changed path must be in Section 4.

Because M094 changes no production code, a full `emissary-core` or I2PControl runtime suite is not required solely for this documentation reconciliation. If an implementation agent changes anything that can compile into production/runtime behavior, that is a stop condition rather than a reason to expand verification.

For containment-test integrity, inspect the diff to `emissary-cli/tests/m062_dependency_containment.rs` and require that it consists only of the two exact M094 planning/closure allowlist entries.

## 9. Compatibility, security, lifecycle, and migration effects

None.

M094 changes no public API, wire behavior, filesystem format, key ownership, routing behavior, admission policy, dependency provenance, startup behavior, tunnel lifecycle, cancellation semantics, task ownership, timeout, or state migration.

The accepted security state remains:

- M090 retained;
- M091 rolled back and non-authoritative;
- M088 lower-layer/pre-accept residual explicitly accepted;
- M092 closed rollback authority;
- M093 current corrected-head security reclosure authority.

## 10. Acceptance and stop conditions

M094 closes only if:

- all active-state inconsistencies enumerated in Section 1 are reconciled;
- exact SHA roles are unambiguous;
- M062 still passes with only exact M094 planning entries added;
- no production/dependency file changed;
- no historical security conclusion or residual-risk disposition changed;
- no upstream interaction occurred.

Stop and open a separate numbered corrective rather than widening M094 if review discovers:

- an actual production/runtime defect;
- a dependency or containment semantic defect beyond stale planning text;
- a new high/medium security finding;
- a need to modify M090/M088/M091 technical behavior;
- a need for any upstream action.

## 11. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/094-closure.md` containing:

- planning baseline and closure head;
- exact changed-path matrix;
- before/after status evidence for M092 and M093;
- corrected M092/M093 SHA-role evidence;
- registry/roadmap/README convergence evidence;
- exact M062 planning-allowlist diff;
- `m062_dependency_containment` test outcome;
- `git diff --check` outcome;
- proof no production/dependency path changed;
- unresolved findings and severity;
- explicit statement that M088/M090/M091/M092/M093 technical dispositions are unchanged;
- internal-only/no-upstream attestation.

M094 returned the tunnel-security planning line to `closed / no ready handoff`. It does not claim a new production security review; M093 remains that authority.

## 12. Internal-only rule

All writes are confined to `eggstack/emissary`.

External I2P, I2P+, Yosemite, specifications, issues, commits, and pull requests may be read only as correctness evidence. No upstream issue, PR, review, submission, merge request, contribution preparation, repository write, or maintainer contact is authorized.
